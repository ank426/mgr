use std::path::Path;

use crate::manga::{self, Manga};
use crate::{cbz, readlist, server};

pub fn handle_generate(path: &Path, readlist_file_name: &str) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let output_path = readlist::generate(path, readlist_file_name)
        .map_err(|err| format!("Failed to generate progress file: {err}"))?;
    println!("Generated {}", output_path.display());
    Ok(())
}

pub async fn handle_serve_file(
    path: &Path,
    port: u16,
    prefetch_back: u32,
    prefetch_forward: u32,
) -> Result<(), String> {
    if !cbz::is_supported_archive_file(path) {
        return Err(format!("Unsupported file type: {} (expected .cbz or .zip)", path.display()));
    }

    let volume =
        cbz::load_volume(path).map_err(|err| format!("Failed to load manga file {}: {err}", path.display()))?;
    if volume.pages.is_empty() {
        return Err(format!("No supported image pages found in {}", path.display()));
    }

    let series = Manga::from_volume(volume);
    server::serve(series, port, prefetch_back, prefetch_forward, 0, 0, 0.0).await;
    Ok(())
}

pub async fn handle_serve_readlist_directory(
    path: &Path,
    readlist_file_name: &str,
    port: u16,
    prefetch_back: u32,
    prefetch_forward: u32,
) -> Result<(), String> {
    let readlist_path = path.join(readlist_file_name);
    if !readlist_path.is_file() {
        return Err(format!(
            "No {readlist_file_name} found in {}. Run: mgr --readlist-file {} --generate {}",
            path.display(),
            readlist_file_name,
            path.display(),
        ));
    }

    let readlist = readlist::load(&readlist_path).map_err(|err| format!("Failed to load readlist: {err}"))?;
    let runtime = manga::build_from_readlist(path, readlist)?;
    server::serve(
        runtime.manga,
        port,
        prefetch_back,
        prefetch_forward,
        runtime.initial_volume_index,
        runtime.initial_page_index,
        runtime.initial_scroll,
    )
    .await;
    Ok(())
}
