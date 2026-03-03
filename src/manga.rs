use std::path::{Path, PathBuf};

use crate::cbz::{self, Volume};
use crate::readlist::LoadedReadList;

#[derive(Clone, Debug)]
pub struct MangaPageRef {
    pub archive_path: PathBuf,
    pub page_name: String,
    pub mime: &'static str,
}

#[derive(Clone, Debug)]
pub struct Manga {
    title: String,
    pages: Vec<MangaPageRef>,
}

pub struct ReadlistRuntime {
    pub manga: Manga,
    pub initial_page_index: u32,
    pub initial_scroll: f64,
}

impl Manga {
    pub fn from_volume(volume: Volume) -> Self {
        let title = volume.title.clone();
        Self::from_volumes(title, vec![volume])
    }

    pub fn from_volumes(title: String, volumes: Vec<Volume>) -> Self {
        let pages = flatten_pages(volumes);
        Self { title, pages }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn page(&self, index: u32) -> Option<&MangaPageRef> {
        self.pages.get(index as usize)
    }
}

pub fn build_from_readlist(root: &Path, readlist: LoadedReadList) -> Result<ReadlistRuntime, String> {
    if readlist.files.is_empty() {
        return Err(format!("Readlist has no files in {}", root.display()));
    }
    if !readlist.files.iter().any(|name| name == &readlist.progress_file) {
        return Err(format!(
            "progress.file '{}' is not present in [[files]]",
            readlist.progress_file
        ));
    }
    if !readlist.progress_scroll.is_finite() || !(0.0..=1.0).contains(&readlist.progress_scroll) {
        return Err(format!(
            "progress.scroll must be a finite value in [0.0, 1.0], found {}",
            readlist.progress_scroll
        ));
    }

    let mut volumes = Vec::with_capacity(readlist.files.len());
    let mut initial_page_index = 0_u32;
    let mut page_offset = 0_u32;

    for name in &readlist.files {
        let file_path = root.join(name);
        if !file_path.exists() {
            return Err(format!("Readlist file '{}' does not exist", file_path.display()));
        }
        if !file_path.is_file() {
            return Err(format!("Readlist entry '{}' is not a file", file_path.display()));
        }
        if !cbz::is_supported_archive_file(&file_path) {
            return Err(format!(
                "Readlist file '{}' is not a supported archive (.cbz/.zip)",
                file_path.display()
            ));
        }

        let volume = cbz::load_volume(&file_path)
            .map_err(|err| format!("Failed to load manga file {}: {err}", file_path.display()))?;
        if volume.pages.is_empty() {
            return Err(format!("No supported image pages found in {}", file_path.display()));
        }

        if name == &readlist.progress_file {
            if readlist.progress_page == 0 {
                return Err("progress.page must be >= 1".to_string());
            }
            let local_index = readlist.progress_page - 1;
            if local_index >= volume.pages.len() as u32 {
                return Err(format!(
                    "progress.page {} is out of range for '{}' (has {} pages)",
                    readlist.progress_page,
                    name,
                    volume.pages.len()
                ));
            }
            initial_page_index = page_offset + local_index;
        }

        page_offset += volume.pages.len() as u32;
        volumes.push(volume);
    }

    let title = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("manga")
        .to_string();
    let manga = Manga::from_volumes(title, volumes);

    Ok(ReadlistRuntime {
        manga,
        initial_page_index,
        initial_scroll: readlist.progress_scroll,
    })
}

fn flatten_pages(volumes: Vec<Volume>) -> Vec<MangaPageRef> {
    let mut pages = Vec::new();
    for volume in volumes {
        let archive_path = volume.archive_path;
        for page in volume.pages {
            pages.push(MangaPageRef {
                archive_path: archive_path.clone(),
                page_name: page.name,
                mime: page.mime,
            });
        }
    }
    pages
}
