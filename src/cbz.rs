use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use zip::ZipArchive;

use crate::image;

#[derive(Clone, Debug)]
pub struct Page {
    pub name: String,
    pub dimensions: (u32, u32),
}

impl Page {
    pub fn new<R: Read>(mut entry: zip::read::ZipFile<'_, R>) -> io::Result<Self> {
        let name = entry.name().to_string();
        let dimensions = image::read_dimensions(&mut entry)?;
        Ok(Self { name, dimensions })
    }

    pub fn mime(&self) -> Option<&'static str> {
        let ext = Path::new(&self.name).extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => Some("image/jpeg"),
            "png" => Some("image/png"),
            "webp" => Some("image/webp"),
            "gif" => Some("image/gif"),
            "bmp" => Some("image/bmp"),
            "avif" => Some("image/avif"),
            _ => None,
        }
    }

    pub async fn load_bytes(&self, archive_path: PathBuf) -> io::Result<Vec<u8>> {
        let page_name = self.name.clone();
        tokio::task::spawn_blocking(move || Self::load_bytes_sync(&archive_path, &page_name))
            .await
            .map_err(|err| io::Error::other(format!("Page load task failed: {err}")))?
    }

    fn load_bytes_sync(archive_path: &Path, page_name: &str) -> io::Result<Vec<u8>> {
        let file = File::open(archive_path)?;
        let mut archive = ZipArchive::new(file).map_err(zip_invalid_data)?;
        let mut entry = archive.by_name(page_name).map_err(zip_invalid_data)?;

        let mut data = Vec::new();
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
    pub fn new(path: &Path, name: String, mokuro: Option<String>) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file).map_err(zip_invalid_data)?;

        let mut pages = Vec::with_capacity(archive.len());
        for idx in 0..archive.len() {
            let entry = archive.by_index(idx).map_err(zip_invalid_data)?;
            if entry.is_dir() {
                continue;
            }
            pages.push(Page::new(entry)?);
        }

        pages.sort_by(|a, b| compare_str(&a.name, &b.name));
        if pages.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("No supported image pages found in {}", path.display()),
            ));
        }

        Ok(Self { name, pages, mokuro })
    }
}

pub fn is_cbz(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("cbz"))
}

fn zip_invalid_data(err: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}
