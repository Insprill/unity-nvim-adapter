use anyhow::{Context, Result, bail};
use clap::{Parser, arg, command};
use env_logger::Env;
use log::{error, info, warn};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, exit},
    thread,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    project_dir: PathBuf,
    #[arg(short, long)]
    goto: Option<String>,
}

fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::builder()
        .parse_env(env)
        .format_target(false)
        .init();

    if let Err(err) = run() {
        error!("{:#}", err);
        exit(1);
    }
}

fn run() -> Result<()> {
    info!(
        "Parsing args '{}'",
        env::args().collect::<Vec<String>>().join(" ")
    );

    let args = Args::try_parse()?;

    if let Some(path) = args.goto {
        let path_line_column = parse_line_and_column_aware(&path)?;
        let pipe_path = get_or_start_unity_adapter(&path_line_column.path)?;
        info!(
            "Opening file '{}' to line {} column {}",
            path_line_column.path,
            path_line_column.line.unwrap_or_default(),
            path_line_column.column.unwrap_or_default()
        );
        let output = Command::new("nvim")
            .arg("--server")
            .arg(pipe_path)
            .arg("--remote-send")
            .arg(format!(
                "<C-\\><C-N>:n {}<CR>|:call cursor({},{})<CR>",
                path_line_column.path,
                path_line_column.line.unwrap_or_default(),
                path_line_column.column.unwrap_or_default(),
            ))
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            info!("nvim: {}", stdout.trim());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            error!("nvim: {}", stderr.trim());
        }
        match output.status.success() {
            true => info!("nvim: exited with {}", output.status),
            false => warn!("nvim: exited with {}", output.status),
        }
    } else {
        error!("No args!");
    }

    Ok(())
}

fn get_or_start_unity_adapter(start_dir: &str) -> Result<PathBuf> {
    let unity_root = find_unity_root(start_dir).context("Not a Unity project!")?;
    let pipe_path = get_unity_adapter_pipe(&unity_root)?;
    if pipe_path.exists() {
        return Ok(pipe_path);
    }

    fs::create_dir_all(pipe_path.parent().context("")?)?;

    let terminal = env::var("TERMINAL").context("TERMINAL not set, unsure which to use")?;
    let args = [
        "-e",
        "bash",
        "-c",
        "nvim",
        "--listen",
        &pipe_path.to_string_lossy(),
    ];

    info!(
        "Starting Neovim with command '{terminal} {}'",
        args.join(" ")
    );

    Command::new(&terminal)
        .current_dir(unity_root)
        .args(args)
        .spawn()?;

    const SLEEP_TIME: u64 = 25;
    const MAX_WAIT_TIME: u64 = 1000 / SLEEP_TIME;

    for _ in 0..MAX_WAIT_TIME {
        thread::sleep(Duration::from_millis(SLEEP_TIME));
        if pipe_path.exists() {
            return Ok(pipe_path);
        }
    }

    bail!("Neovim or its server didn't start!")
}

fn get_unity_adapter_pipe(unity_root: &Path) -> Result<PathBuf> {
    Ok(unity_root.join("Temp").join("unity_adapter_pipe"))
}

fn find_unity_root(start_dir: &str) -> Option<PathBuf> {
    let mut current_dir = PathBuf::from(start_dir);

    loop {
        let project_version_path = current_dir
            .join("ProjectSettings")
            .join("ProjectVersion.txt");

        if project_version_path.exists() {
            return Some(current_dir);
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

// https://github.com/microsoft/vscode/blob/16f58dd3ac0b855df43dcd6a9d32a0911dca320f/src/vs/base/common/extpath.ts#L353-L386

#[derive(Debug)]
pub struct PathWithLineAndColumn {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

// Disable all formatting so we can match the source as close as possible
#[rustfmt::skip]
#[allow(clippy::all)]
pub fn parse_line_and_column_aware(raw_path: &str) -> Result<PathWithLineAndColumn> {
    let segments = raw_path.split(':'); // C:\file.txt:<line>:<column>

    let mut path: Option<String> = Option::None;
    let mut line: Option<usize> = Option::None;
    let mut column: Option<usize> = Option::None;

    for segment in segments {
        let segment_as_number = segment.parse::<usize>();
        if !segment_as_number.is_ok() {
            path = Some(match path { Some(p) => format!("{p}:{segment}"), None => segment.to_string() });
        } else if line.is_none() {
            line = Some(segment_as_number.unwrap());
        } else if column.is_none() {
            column = Some(segment_as_number.unwrap());
        }
    }

    if !path.is_some() {
        bail!("Format for `--goto` should be: `FILE:LINE(:COLUMN)`")
    }

    Ok(PathWithLineAndColumn {
        path: path.unwrap(),
        line,
        column: if column.is_some() { column } else if line.is_some() { Some(1) } else { None }, // If we have a line, make sure column is also set
    })
}
