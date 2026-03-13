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

#[derive(Clone, Debug)]
pub struct Volume {
    pub archive_path: PathBuf,
    pub pages: Vec<Page>,
}

impl Volume {
    pub fn file_name(&self) -> &str {
        self.archive_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("loaded volume archive has a UTF-8 file name")
    }
}

impl Page {
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
}

pub fn load_volume(path: &Path) -> io::Result<Volume> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_invalid_data)?;

    let mut pages = Vec::with_capacity(archive.len());
    for idx in 0..archive.len() {
        let entry = archive.by_index(idx).map_err(zip_invalid_data)?;

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let dimensions = image::read_dimensions(entry)?;

        pages.push(Page { name, dimensions });
    }

    pages.sort_by(|a, b| compare_str(&a.name, &b.name));
    if pages.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("No supported image pages found in {}", path.display()),
        ));
    }

    Ok(Volume { archive_path: path.to_path_buf(), pages })
}

pub async fn load_page_bytes(archive_path: PathBuf, page_name: String) -> io::Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || load_page_bytes_sync(&archive_path, &page_name))
        .await
        .map_err(|err| io::Error::other(format!("Page load task failed: {err}")))?
}

fn load_page_bytes_sync(archive_path: &Path, page_name: &str) -> io::Result<Vec<u8>> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_invalid_data)?;
    let mut entry = archive.by_name(page_name).map_err(zip_invalid_data)?;

    let mut data = Vec::new();
    entry.read_to_end(&mut data)?;
    Ok(data)
}

pub fn is_cbz(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("cbz"))
}

fn zip_invalid_data(err: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}
