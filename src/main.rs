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

    info!(
        "Parsing args '{}'",
        env::args().collect::<Vec<String>>().join(" ")
    );

    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };

    if let Some(path) = args.goto {
        let path_line_column = parse_line_and_column_aware(&path).unwrap();
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
        error!("No args!!");
    }
}

// https://github.com/microsoft/vscode/blob/e2a3691756246693948f868f8603588c91c563d2/src/vs/base/common/extpath.ts#L353-L386

#[derive(Debug)]
pub struct PathWithLineAndColumn {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

pub fn parse_line_and_column_aware(raw_path: &str) -> Result<PathWithLineAndColumn, String> {
    // Split the input on colons.
    let segments = raw_path.split(':');

    let mut path: Option<String> = None;
    let mut line: Option<usize> = None;
    let mut column: Option<usize> = None;

    for segment in segments {
        // Try to parse the segment as a number.
        match segment.parse::<usize>() {
            Ok(num) => {
                if line.is_none() {
                    line = Some(num);
                } else if column.is_none() {
                    column = Some(num);
                }
                // If both line and column are already set, extra numbers are ignored.
            }
            Err(_) => {
                // If the segment isn't a number, it's part of the file path.
                if let Some(ref mut existing_path) = path {
                    // Append with colon if the path is already partially constructed.
                    existing_path.push(':');
                    existing_path.push_str(segment);
                } else {
                    path = Some(segment.to_string());
                }
            }
        }
    }

    // Ensure we got a valid file path.
    let path = match path {
        Some(p) if !p.is_empty() => p,
        _ => return Err("Format for `--goto` should be: `FILE:LINE(:COLUMN)`".to_string()),
    };

    // If a line number was specified but no column, default column to 1.
    if line.is_some() && column.is_none() {
        column = Some(1);
    }

    Ok(PathWithLineAndColumn { path, line, column })
}
