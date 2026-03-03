use std::path::Path;

use crate::{cbz, global_index, readlist, server};

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

    let index = global_index::GlobalIndex::from_manga(manga);
    server::serve(index, port, prefetch_back, prefetch_forward).await;
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

    let readlist = readlist::load(&readlist_path).map_err(|err| format!("Failed to load readlist: {err}"))?;
    if readlist.files.is_empty() {
        return Err(format!("Readlist has no files in {}", readlist_path.display()));
    }

    if !readlist.files.iter().any(|name| name == &readlist.progress_file) {
        return Err(format!(
            "progress.file '{}' is not present in [[files]] in {}",
            readlist.progress_file,
            readlist_path.display()
        ));
    }

    for name in &readlist.files {
        let file_path = path.join(name);
        if !file_path.exists() {
            return Err(format!(
                "Readlist file '{}' is missing on disk under {}",
                file_path.display(),
                path.display()
            ));
        }
        if !file_path.is_file() {
            return Err(format!("Readlist entry '{}' is not a file", file_path.display()));
        }
        if !is_supported_archive_file(&file_path) {
            return Err(format!(
                "Readlist file '{}' is not a supported archive (.cbz/.zip)",
                file_path.display()
            ));
        }
    }
    let page_counts = global_index::collect_readlist_page_counts(path, &readlist.files)?;

    Err(format!(
        "Readlist validated and page counts loaded in-memory for {} files in {} (progress file='{}', page={}, scroll={:.6}), but readlist serving is not implemented yet.",
        page_counts.len(),
        path.display(),
        readlist.progress_file,
        readlist.progress_page,
        readlist.progress_scroll
    ))
}

fn is_supported_archive_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cbz") || ext.eq_ignore_ascii_case("zip"))
}
