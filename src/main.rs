mod cbz;
mod handlers;
mod image;
mod manga;
mod readlist;
mod routes;
mod server;

use anyhow::{bail, ensure};
use clap::Parser;
use std::path::PathBuf;

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    generate: bool,

    #[arg(long, default_value = ".mgr.toml")]
    readlist_file: String,

    #[arg(short, long, default_value_t = 7169)]
    port: u16,

    #[arg(short = 'o', long = "open")]
    open: bool,

    #[arg(long, default_value_t = 6.0)]
    prefetch_back: f32,

    #[arg(long, default_value_t = 8.0)]
    prefetch_forward: f32,

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

async fn run(args: Args) -> anyhow::Result<()> {
    if args.generate {
        let [path] = args.paths.as_slice() else {
            bail!("--generate expects a single directory path");
        };
        return handlers::generate(path, &args.readlist_file).await;
    }

    ensure!(args.prefetch_back.is_finite() && args.prefetch_forward.is_finite(), "prefetch values must be finite");
    let prefetch = (args.prefetch_back, args.prefetch_forward);

    if let Some(path) = args.paths.first()
        && path.is_dir()
    {
        let [path] = args.paths.as_slice() else {
            bail!("Directory path must be provided alone");
        };
        return handlers::serve_readlist(path, &args.readlist_file, args.port, prefetch, args.open).await;
    }

    handlers::serve_files(args.paths.as_slice(), args.port, prefetch, args.open).await
}
