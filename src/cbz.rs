use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use anyhow::ensure;
use zip::ZipArchive;

use crate::image;

#[derive(Clone, Debug)]
pub struct Page {
    pub name: String,
    pub mime: &'static str,
    pub dimensions: (u32, u32),
}

impl Page {
    pub async fn load_bytes(&self, archive_path: PathBuf) -> anyhow::Result<Vec<u8>> {
        let page_name = self.name.clone();
        Ok(tokio::task::spawn_blocking(move || Self::load_bytes_sync(&archive_path, &page_name)).await??)
    }

    fn load_bytes_sync(archive_path: &Path, page_name: &str) -> anyhow::Result<Vec<u8>> {
        let mut archive = ZipArchive::new(File::open(archive_path)?)?;
        let mut entry = archive.by_name(page_name)?;
        let mut data = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut data)?;
        Ok(data)
    }
}

#[derive(Clone, Debug)]
pub struct Volume {
    pub name: String,
    pub mokuro: Option<String>,
    pub pages: Vec<Page>,
}

impl Volume {
    pub fn new(path: &Path, name: String, mokuro: Option<String>) -> anyhow::Result<Self> {
        let mut archive = ZipArchive::new(File::open(path)?)?;

        let mut pages = Vec::with_capacity(archive.len());
        for idx in 0..archive.len() {
            let mut entry = archive.by_index(idx)?;
            if entry.is_dir() {
                continue;
            }
            if let Some((mime, dimensions)) = image::read_image_info(&mut entry)? {
                pages.push(Page { name: entry.name().to_string(), mime, dimensions });
            }
        }

        pages.sort_by(|a, b| compare_str(&a.name, &b.name));
        ensure!(!pages.is_empty(), "No supported image pages found in {}", path.display());

        Ok(Self { name, pages, mokuro })
    }
}

pub fn is_cbz(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("cbz"))
}

