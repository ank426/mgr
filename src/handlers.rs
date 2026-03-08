use std::path::Path;

use crate::error::AppResult;
use crate::{cbz, readlist, server, timing};

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

pub async fn handle_serve_file(path: &Path, port: u16, prefetch: (u32, u32)) -> AppResult<()> {
    if !cbz::is_cbz(path) {
        return Err(format!("Unsupported file type: {} (expected .cbz)", path.display()).into());
    }

    let volume = {
        let _stage = timing::stage("startup load single volume");
        cbz::load_volume(path)?
    };
    let title = volume.title.clone();
    let volumes = vec![volume];
    let progress = readlist::Progress {
        file: path.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_string(),
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
    readlist.validate(path)?;
    let volumes = readlist.load_volumes(path)?;
    let title = path.file_name().and_then(|name| name.to_str()).unwrap_or("manga").to_string();
    server::serve(title, volumes, readlist.progress.clone(), port, prefetch).await;
    Ok(())
}
