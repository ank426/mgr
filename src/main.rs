mod cbz;
mod readlist;
mod server;

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

#[tokio::main]
async fn main() {
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

    if !args.path.exists() {
        eprintln!("Path does not exist: {}", args.path.display());
        std::process::exit(1);
    }

    if args.path.is_file() {
        let is_supported = args
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("cbz") || ext.eq_ignore_ascii_case("zip"));
        if !is_supported {
            eprintln!(
                "Unsupported file type: {} (expected .cbz or .zip)",
                args.path.display()
            );
            std::process::exit(1);
        }

        match cbz::load_manga(&args.path) {
            Ok(manga) => {
                if manga.pages.is_empty() {
                    eprintln!("No supported image pages found in {}", args.path.display());
                    std::process::exit(1);
                }
                server::serve(manga, args.port).await;
            }
            Err(err) => {
                eprintln!("Failed to load manga file {}: {err}", args.path.display());
                std::process::exit(1);
            }
        }
        return;
    }

    eprintln!(
        "Path {} is a directory. Use --generate for directories, or pass a .cbz/.zip file.",
        args.path.display()
    );
    std::process::exit(1);
}
