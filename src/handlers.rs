use std::path::{Path, PathBuf};

use crate::error::AppResult;
use crate::manga::Manga;
use crate::readlist::ReadList;
use crate::{readlist, server};

pub async fn generate(path: &Path, readlist_file_name: &str) -> AppResult<()> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()).into());
    }

    let output_path = readlist::generate(path, readlist_file_name).await?;
    println!("Generated {}", output_path.display());
    Ok(())
}

pub async fn serve_files(paths: &[PathBuf], port: u16, prefetch: (f32, f32), open: bool) -> AppResult<()> {
    let manga = Manga::new(paths)?;
    server::serve(manga, port, prefetch, open, None, None).await;
    Ok(())
}

pub async fn serve_readlist(
    path: &Path,
    readlist_file_name: &str,
    port: u16,
    prefetch: (f32, f32),
    open: bool,
) -> AppResult<()> {
    let readlist_path = path.join(readlist_file_name);
    if !readlist_path.is_file() {
        return Err(format!(
            "No {readlist_file_name} found in {path}. Run: mgr --readlist-file {readlist_file_name} --generate {path}",
            path = path.display(),
        )
        .into());
    }
    let readlist = ReadList::new(&readlist_path)?;
    let manga = Manga::from_readlist(path, &readlist)?;
    server::serve(manga, port, prefetch, open, Some(readlist_path), Some(readlist)).await;
    Ok(())
}
