use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use zip::ZipArchive;

#[derive(Clone, Debug)]
pub struct Page {
    pub name: String,
    pub mime: &'static str,
}

#[derive(Clone, Debug)]
pub struct Volume {
    pub archive_path: PathBuf,
    pub title: String,
    pub pages: Vec<Page>,
}

pub fn load_volume(path: &Path) -> io::Result<Volume> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_invalid_data)?;

    let mut pages = Vec::new();
    for idx in 0..archive.len() {
        let entry = archive.by_index(idx).map_err(zip_invalid_data)?;

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let Some(mime) = mime_for_path(&name) else {
            continue;
        };

        pages.push(Page { name, mime });
    }

    pages.sort_by(|a, b| compare_str(&a.name, &b.name));

    let title = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("manga")
        .to_string();

    Ok(Volume {
        archive_path: path.to_path_buf(),
        title,
        pages,
    })
}

pub fn load_page_bytes(archive_path: &Path, page_name: &str) -> io::Result<Vec<u8>> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_invalid_data)?;
    let mut entry = archive.by_name(page_name).map_err(zip_invalid_data)?;

    let mut data = Vec::new();
    entry.read_to_end(&mut data)?;
    Ok(data)
}

fn mime_for_path(path: &str) -> Option<&'static str> {
    let ext = Path::new(path).extension()?.to_str()?.to_ascii_lowercase();
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

pub fn is_supported_archive_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cbz") || ext.eq_ignore_ascii_case("zip"))
}

fn zip_invalid_data(err: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}
