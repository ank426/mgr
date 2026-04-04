mod cbz;
mod config;
mod image;
mod manga;
mod readlist;
mod routes;
mod server;

use anyhow::{bail, ensure};
use clap::Parser;
use std::path::PathBuf;

use config::Config;
use manga::Manga;
use readlist::ReadList;

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    generate: bool,

    #[command(flatten)]
    config: Config,

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
        let [path] = args.paths.as_slice() else { bail!("--generate expects a single directory path") };
        ensure!(path.is_dir(), "No directory exists at: {}", path.display());
        println!("Generated {}", readlist::generate(path, &args.config.readlist_file).await?.display());
        return Ok(());
    }

    args.config.validate()?;

    if let Some(path) = args.paths.first()
        && path.is_dir()
    {
        let [path] = args.paths.as_slice() else { bail!("Directory path must be provided alone") };
        let readlist_path = path.join(&args.config.readlist_file);
        ensure!(readlist_path.is_file(), "No {} found in {}. Run: mgr -g", args.config.readlist_file, path.display());
        let readlist = ReadList::new(&readlist_path)?;
        let manga = Manga::from_readlist(path, &readlist)?;
        return server::serve(manga, args.config, Some(readlist_path), Some(readlist)).await;
    }

    server::serve(Manga::new(args.paths.as_slice())?, args.config, None, None).await
}
