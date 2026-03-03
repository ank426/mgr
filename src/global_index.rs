use std::path::{Path, PathBuf};

use crate::cbz::{self, Manga};

#[derive(Clone, Debug)]
pub struct GlobalPageRef {
    pub archive_path: PathBuf,
    pub page_name: String,
    pub mime: &'static str,
}

#[derive(Clone, Debug)]
pub struct GlobalIndex {
    title: String,
    pages: Vec<GlobalPageRef>,
}

impl GlobalIndex {
    pub fn from_manga(manga: Manga) -> Self {
        let title = manga.title.clone();
        Self::from_mangas(title, vec![manga])
    }

    pub fn from_mangas(title: String, mangas: Vec<Manga>) -> Self {
        let pages = flatten_pages(mangas);
        Self { title, pages }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn page(&self, index: u32) -> Option<&GlobalPageRef> {
        self.pages.get(index as usize)
    }
}

pub fn collect_readlist_page_counts(root: &Path, files: &[String]) -> Result<Vec<(String, u32)>, String> {
    let mut page_counts = Vec::with_capacity(files.len());
    for name in files {
        let file_path = root.join(name);
        let manga = cbz::load_manga(&file_path)
            .map_err(|err| format!("Failed to load manga file {}: {err}", file_path.display()))?;
        if manga.pages.is_empty() {
            return Err(format!("No supported image pages found in {}", file_path.display()));
        }
        page_counts.push((name.clone(), manga.pages.len() as u32));
    }
    Ok(page_counts)
}

fn flatten_pages(mangas: Vec<Manga>) -> Vec<GlobalPageRef> {
    let mut pages = Vec::new();
    for manga in mangas {
        let archive_path = manga.archive_path;
        for page in manga.pages {
            pages.push(GlobalPageRef {
                archive_path: archive_path.clone(),
                page_name: page.name,
                mime: page.mime,
            });
        }
    }
    pages
}
