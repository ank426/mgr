mod cbz;
mod handlers;
mod manga;
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
        return handlers::handle_generate(&args.path);
    }

    if !args.path.exists() {
        return Err(format!("Path does not exist: {}", args.path.display()));
    }

    if args.path.is_file() {
        return handlers::handle_serve_file(&args.path, args.port, args.prefetch_back, args.prefetch_forward).await;
    }

    if args.path.is_dir() {
        return handlers::handle_serve_readlist_directory(&args.path);
    }

    Err(format!(
        "Unsupported path type: {} (expected file or directory)",
        args.path.display()
    ))
}
