use std::path::{Path, PathBuf};

use crate::error::AppResult;
use crate::{cbz, readlist, server};

pub fn handle_generate(path: &Path, readlist_file_name: &str) -> AppResult<()> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()).into());
    }

    let output_path = readlist::generate(path, readlist_file_name)?;
    println!("Generated {}", output_path.display());
    Ok(())
}

pub async fn handle_serve_files(paths: &[PathBuf], port: u16, prefetch: (u32, u32)) -> AppResult<()> {
    if paths.is_empty() {
        return Err("No files provided".into());
    }

    let mut volumes = Vec::with_capacity(paths.len());
    for path in paths {
        if !cbz::is_cbz(path) {
            return Err(format!("Unsupported file type: {} (expected .cbz)", path.display()).into());
        }
        volumes.push(cbz::load_volume(path)?);
    }

    let title = if volumes.len() == 1 {
        volumes[0].archive_path.file_name().and_then(|name| name.to_str()).unwrap_or("mgr").to_string()
    } else {
        "mgr".to_string()
    };
    let progress = readlist::Progress {
        file: paths[0].file_name().and_then(|name| name.to_str()).unwrap_or_default().to_string(),
        page: 1,
        scroll: 0.0,
    };
    server::serve(title, volumes, progress, port, prefetch).await;
    Ok(())
}

pub async fn handle_serve_readlist_directory(
    path: &Path,
    readlist_file_name: &str,
    port: u16,
    prefetch: (u32, u32),
) -> AppResult<()> {
    let readlist_path = path.join(readlist_file_name);
    if !readlist_path.is_file() {
        return Err(format!(
            "No {readlist_file_name} found in {}. Run: mgr --readlist-file {} --generate {}",
            path.display(),
            readlist_file_name,
            path.display(),
        )
        .into());
    }

    let readlist = readlist::load(&readlist_path)?;
    let volumes = readlist.load_volumes(path)?;
    let title = path.file_name().and_then(|name| name.to_str()).unwrap_or("manga").to_string();
    server::serve(title, volumes, readlist.progress.clone(), port, prefetch).await;
    Ok(())
}
