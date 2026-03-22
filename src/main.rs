mod cbz;
mod error;
mod handlers;
mod image;
mod manga;
mod readlist;
mod routes;
mod server;

use clap::Parser;
use std::path::PathBuf;

use crate::error::AppResult;

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    generate: bool,

    #[arg(long, default_value = ".mgr.toml")]
    readlist_file: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(short = 'o', long = "open")]
    open: bool,

    #[arg(long, default_value_t = 6)]
    prefetch_back: u32,

    #[arg(long, default_value_t = 8)]
    prefetch_forward: u32,

    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run(Args::parse()).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run(args: Args) -> AppResult<()> {
    if args.generate {
        if args.paths.len() != 1 {
            return Err("--generate expects a single directory path".into());
        }
        return handlers::handle_generate(&args.paths[0], &args.readlist_file).await;
    }

    let prefetch = (args.prefetch_back, args.prefetch_forward);

    if args.paths[0].is_dir() {
        if args.paths.len() != 1 {
            return Err("Directory path must be provided alone".into());
        }
        return handlers::handle_serve_readlist_directory(
            &args.paths[0],
            &args.readlist_file,
            args.port,
            prefetch,
            args.open,
        )
        .await;
    }

    for path in &args.paths {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()).into());
        }
        if !path.is_file() {
            return Err(format!("Multiple paths only supports files, found: {}", path.display()).into());
        }
    }

    handlers::handle_serve_files(args.paths.as_slice(), args.port, prefetch, args.open).await
}
