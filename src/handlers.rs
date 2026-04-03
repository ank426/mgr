use std::path::{Path, PathBuf};

use anyhow::ensure;

use crate::manga::Manga;
use crate::readlist::ReadList;
use crate::{readlist, server};

pub async fn generate(path: &Path, readlist_file_name: &str) -> anyhow::Result<()> {
    ensure!(path.exists(), "Path does not exist: {}", path.display());
    ensure!(path.is_dir(), "Path is not a directory: {}", path.display());
    println!("Generated {}", readlist::generate(path, readlist_file_name).await?.display());
    Ok(())
}

pub async fn serve_files(paths: &[PathBuf], port: u16, prefetch: (f32, f32), open: bool) -> anyhow::Result<()> {
    server::serve(Manga::new(paths)?, port, prefetch, open, None, None).await;
    Ok(())
}

pub async fn serve_readlist(
    path: &Path,
    readlist_file_name: &str,
    port: u16,
    prefetch: (f32, f32),
    open: bool,
) -> anyhow::Result<()> {
    let readlist_path = path.join(readlist_file_name);
    ensure!(
        readlist_path.is_file(),
        "No {readlist_file_name} found in {path}. Run: mgr --readlist-file {readlist_file_name} --generate {path}",
        path = path.display(),
    );
    let readlist = ReadList::new(&readlist_path)?;
    let manga = Manga::from_readlist(path, &readlist)?;
    server::serve(manga, port, prefetch, open, Some(readlist_path), Some(readlist)).await;
    Ok(())
}
