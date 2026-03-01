use std::path::Path;

use crate::{cbz, readlist, server};

const READLIST_FILE_NAME: &str = ".mgr.toml";

pub fn handle_generate(path: &Path) -> Result<(), String> {
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

pub async fn handle_serve_file(
    path: &Path,
    port: u16,
    prefetch_back: u32,
    prefetch_forward: u32,
) -> Result<(), String> {
    if !is_supported_archive_file(path) {
        return Err(format!(
            "Unsupported file type: {} (expected .cbz or .zip)",
            path.display()
        ));
    }

    let manga = cbz::load_manga(path).map_err(|err| format!("Failed to load manga file {}: {err}", path.display()))?;
    if manga.pages.is_empty() {
        return Err(format!("No supported image pages found in {}", path.display()));
    }

    server::serve(manga, port, prefetch_back, prefetch_forward).await;
    Ok(())
}

pub fn handle_serve_readlist_directory(path: &Path) -> Result<(), String> {
    let readlist_path = path.join(READLIST_FILE_NAME);
    if !readlist_path.is_file() {
        return Err(format!(
            "No {READLIST_FILE_NAME} found in {}. Run: mgr --generate {}",
            path.display(),
            path.display()
        ));
    }

    Err(format!(
        "{READLIST_FILE_NAME} found in {}, but readlist mode is not implemented yet (Phase 1 only).",
        path.display()
    ))
}

fn is_supported_archive_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cbz") || ext.eq_ignore_ascii_case("zip"))
}
