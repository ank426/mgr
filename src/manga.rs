use std::path::{Path, PathBuf};

use crate::cbz::{self, Volume};

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

pub fn collect_readlist_page_counts(root: &Path, files: &[String]) -> Result<Vec<(String, u32)>, String> {
    let mut page_counts = Vec::with_capacity(files.len());
    for name in files {
        let file_path = root.join(name);
        let volume = cbz::load_volume(&file_path)
            .map_err(|err| format!("Failed to load manga file {}: {err}", file_path.display()))?;
        if volume.pages.is_empty() {
            return Err(format!("No supported image pages found in {}", file_path.display()));
        }
        page_counts.push((name.clone(), volume.pages.len() as u32));
    }
    Ok(page_counts)
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
