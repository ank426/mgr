mod readlist;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    generate: bool,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    if args.generate {
        if !args.path.exists() {
            eprintln!("Path does not exist: {}", args.path.display());
            std::process::exit(1);
        }

        if !args.path.is_dir() {
            eprintln!("Path is not a directory: {}", args.path.display());
            std::process::exit(1);
        }

        match readlist::generate(&args.path) {
            Ok(output_path) => {
                println!("Generated {}", output_path.display());
            }
            Err(err) => {
                eprintln!("Failed to generate progress file: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    println!("{:?}", args);
}
