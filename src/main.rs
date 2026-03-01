mod cbz;
mod readlist;
mod server;
mod window;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    generate: bool,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value_t = 2)]
    prefetch_back: u32,

    #[arg(long, default_value_t = 4)]
    prefetch_forward: u32,

    #[arg(default_value = ".")]
    path: PathBuf,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run(Args::parse()).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<(), String> {
    if args.generate {
        return handle_generate(&args.path);
    }

    if args.path.is_file() {
        return handle_serve_file(&args).await;
    }

    if !args.path.exists() {
        return Err(format!("Path does not exist: {}", args.path.display()));
    }

    if args.path.is_dir() {
        return Err(format!(
            "Path {} is a directory. Use --generate for directories, or pass a .cbz/.zip file.",
            args.path.display()
        ));
    }

    Err(format!(
        "Unsupported path type: {} (expected file or directory)",
        args.path.display()
    ))
}

fn handle_generate(path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let output_path = readlist::generate(path).map_err(|err| format!("Failed to generate progress file: {err}"))?;
    println!("Generated {}", output_path.display());
    Ok(())
}

async fn handle_serve_file(args: &Args) -> Result<(), String> {
    if !is_supported_archive_file(&args.path) {
        return Err(format!(
            "Unsupported file type: {} (expected .cbz or .zip)",
            args.path.display()
        ));
    }

    let manga = cbz::load_manga(&args.path)
        .map_err(|err| format!("Failed to load manga file {}: {err}", args.path.display()))?;
    if manga.pages.is_empty() {
        return Err(format!("No supported image pages found in {}", args.path.display()));
    }

    server::serve(manga, args.port, args.prefetch_back, args.prefetch_forward).await;
    Ok(())
}

fn is_supported_archive_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cbz") || ext.eq_ignore_ascii_case("zip"))
}
