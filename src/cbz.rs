use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use alphanumeric_sort::compare_str;
use zip::ZipArchive;

#[derive(Clone, Debug)]
pub struct Page {
    pub name: String,
    pub mime: &'static str,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Manga {
    pub title: String,
    pub pages: Vec<Page>,
}

pub fn load_manga(path: &Path) -> io::Result<Manga> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    let mut pages = Vec::new();
    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let Some(mime) = mime_for_path(&name) else {
            continue;
        };

        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        pages.push(Page { name, mime, data });
    }

    pages.sort_by(|a, b| compare_str(&a.name, &b.name));

    let title = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("manga")
        .to_string();

    Ok(Manga { title, pages })
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
