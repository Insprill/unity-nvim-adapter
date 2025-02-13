use clap::{Parser, arg, command};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    goto: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(path) = args.goto {
        let split_path: Vec<&str> = path.split(':').collect();
        let file = split_path[0];
        Command::new("nvim")
            .arg("--remote")
            .arg(file)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}
