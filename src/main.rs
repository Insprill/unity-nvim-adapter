use anyhow::{Result, bail};
use clap::{Parser, arg, command};
use env_logger::Env;
use log::{error, info};
use std::{
    env,
    path::PathBuf,
    process::{Command, exit},
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
        info!(
            "Opening file '{}' to line {} column {}",
            path_line_column.path,
            path_line_column.line.unwrap_or_default(),
            path_line_column.column.unwrap_or_default()
        );
        let _ = Command::new("nvim")
            .arg("--server")
            .arg("localhost:6969")
            .arg("--remote-send")
            .arg(format!(
                "<C-\\><C-N>:n {}<CR>|:call cursor({},{})<CR>",
                path_line_column.path,
                path_line_column.line.unwrap_or_default(),
                path_line_column.column.unwrap_or_default(),
            ))
            .output()
            .unwrap();
    } else {
        error!("No args!");
    }

    Ok(())
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
