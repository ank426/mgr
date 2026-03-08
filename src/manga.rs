use std::path::{Path, PathBuf};

use crate::cbz::{self, Volume};
use crate::readlist::ReadList;

#[derive(Clone, Debug)]
pub struct MangaPageRef {
    pub archive_path: PathBuf,
    pub page_name: String,
    pub mime: &'static str,
}

#[derive(Clone, Debug)]
pub struct MangaVolume {
    pages: Vec<MangaPageRef>,
}

#[derive(Clone, Debug)]
pub struct Manga {
    title: String,
    volumes: Vec<MangaVolume>,
}

pub struct ReadlistRuntime {
    pub manga: Manga,
    pub initial_volume_index: u32,
    pub initial_page_index: u32,
    pub initial_scroll: f64,
}

impl Manga {
    pub fn from_volume(volume: Volume) -> Self {
        let title = volume.title.clone();
        Self::from_volumes(title, vec![volume])
    }

    pub fn from_volumes(title: String, volumes: Vec<Volume>) -> Self {
        let volumes = map_volumes(volumes);
        Self { title, volumes }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn volume_page_counts(&self) -> Vec<u32> {
        self.volumes.iter().map(|volume| volume.pages.len() as u32).collect()
    }

    pub fn page(&self, volume_index: u32, page_index: u32) -> Option<&MangaPageRef> {
        self.volumes.get(volume_index as usize)?.pages.get(page_index as usize)
    }
}

pub fn build_from_readlist(root: &Path, readlist: ReadList) -> Result<ReadlistRuntime, String> {
    if readlist.files.is_empty() {
        return Err(format!("Readlist has no files in {}", root.display()));
    }
    if !readlist.files.iter().any(|entry| entry.name == readlist.progress.file) {
        return Err(format!("progress.file '{}' is not present in files", readlist.progress.file));
    }
    if !readlist.progress.scroll.is_finite() || !(0.0..=1.0).contains(&readlist.progress.scroll) {
        return Err(format!(
            "progress.scroll must be a finite value in [0.0, 1.0], found {}",
            readlist.progress.scroll
        ));
    }

    let mut volumes = Vec::with_capacity(readlist.files.len());
    let mut initial_volume_index = 0_u32;
    let mut initial_page_index = 0_u32;

    for (volume_index, entry) in readlist.files.iter().enumerate() {
        if entry.pages == 0 {
            return Err(format!("Readlist file '{}' has pages = 0", entry.name));
        }
        if entry.page_dims.len() != entry.pages as usize {
            return Err(format!(
                "Readlist file '{}' has {} page_dims entries but pages = {}",
                entry.name,
                entry.page_dims.len(),
                entry.pages
            ));
        }

        let file_path = root.join(&entry.name);
        if !file_path.exists() {
            return Err(format!("Readlist file '{}' does not exist", file_path.display()));
        }
        if !file_path.is_file() {
            return Err(format!("Readlist entry '{}' is not a file", file_path.display()));
        }
        if !cbz::is_supported_archive_file(&file_path) {
            return Err(format!("Readlist file '{}' is not a supported archive (.cbz/.zip)", file_path.display()));
        }

        let volume = cbz::load_volume(&file_path)
            .map_err(|err| format!("Failed to load manga file {}: {err}", file_path.display()))?;
        if volume.pages.is_empty() {
            return Err(format!("No supported image pages found in {}", file_path.display()));
        }
        if volume.pages.len() != entry.pages as usize {
            return Err(format!(
                "Readlist file '{}' declares {} pages but archive has {}",
                entry.name,
                entry.pages,
                volume.pages.len()
            ));
        }

        for (page, expected_dims) in volume.pages.iter().zip(&entry.page_dims) {
            if page.dimensions != *expected_dims {
                return Err(format!(
                    "Readlist file '{}' has mismatched dimensions for page '{}': expected {}x{}, found {}x{}",
                    entry.name, page.name, expected_dims[0], expected_dims[1], page.dimensions[0], page.dimensions[1]
                ));
            }
        }

        if entry.name == readlist.progress.file {
            if readlist.progress.page == 0 {
                return Err("progress.page must be >= 1".to_string());
            }
            let local_index = readlist.progress.page - 1;
            if local_index >= volume.pages.len() as u32 {
                return Err(format!(
                    "progress.page {} is out of range for '{}' (has {} pages)",
                    readlist.progress.page,
                    entry.name,
                    volume.pages.len()
                ));
            }
            initial_volume_index = volume_index as u32;
            initial_page_index = local_index;
        }

        volumes.push(volume);
    }

    let title = root.file_name().and_then(|name| name.to_str()).unwrap_or("manga").to_string();
    let manga = Manga::from_volumes(title, volumes);

    Ok(ReadlistRuntime { manga, initial_volume_index, initial_page_index, initial_scroll: readlist.progress.scroll })
}

fn map_volumes(volumes: Vec<Volume>) -> Vec<MangaVolume> {
    let mut mapped = Vec::with_capacity(volumes.len());
    for volume in volumes {
        let archive_path = volume.archive_path;
        let pages = volume
            .pages
            .into_iter()
            .map(|page| MangaPageRef { archive_path: archive_path.clone(), page_name: page.name, mime: page.mime })
            .collect();
        mapped.push(MangaVolume { pages });
    }
    mapped
}
