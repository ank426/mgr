use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use imagesize::reader_size;
use zip::ZipArchive;

#[derive(Clone, Debug)]
pub struct Page {
    pub name: String,
    pub mime: &'static str,
    pub dimensions: [u32; 2],
}

#[derive(Clone, Debug)]
pub struct Volume {
    pub archive_path: PathBuf,
    pub title: String,
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

        let dimensions = page_dimensions(entry)?;

        pages.push(Page { name, mime, dimensions });
    }

    pages.sort_by(|a, b| compare_str(&a.name, &b.name));

    let title = path.file_name().and_then(|name| name.to_str()).unwrap_or("manga").to_string();

    Ok(Volume { archive_path: path.to_path_buf(), title, pages })
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

pub fn is_cbz(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("cbz"))
}

fn zip_invalid_data(err: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}

fn page_dimensions<R: Read>(reader: R) -> io::Result<[u32; 2]> {
    let size = reader_size(CachedReadSeeker::new(reader))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to read image dimensions: {err}")))?;

    let width = u32::try_from(size.width)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Image width {} exceeds u32", size.width)))?;
    let height = u32::try_from(size.height)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Image height {} exceeds u32", size.height)))?;

    Ok([width, height])
}

struct CachedReadSeeker<R> {
    reader: R,
    buffer: Vec<u8>,
    position: usize,
    reached_eof: bool,
}

impl<R: Read> CachedReadSeeker<R> {
    fn new(reader: R) -> Self {
        Self { reader, buffer: Vec::new(), position: 0, reached_eof: false }
    }

    fn ensure_available(&mut self, end: usize) -> io::Result<()> {
        while !self.reached_eof && self.buffer.len() < end {
            let mut chunk = [0_u8; 8192];
            let bytes_read = self.reader.read(&mut chunk)?;
            if bytes_read == 0 {
                self.reached_eof = true;
                break;
            }
            self.buffer.extend_from_slice(&chunk[..bytes_read]);
        }

        Ok(())
    }

    fn read_to_end(&mut self) -> io::Result<()> {
        while !self.reached_eof {
            self.ensure_available(self.buffer.len() + 8192)?;
        }

        Ok(())
    }
}

impl<R: Read> Read for CachedReadSeeker<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        self.ensure_available(self.position + buf.len())?;
        let available = self.buffer.len().saturating_sub(self.position);
        let bytes_to_copy = available.min(buf.len());

        if bytes_to_copy == 0 {
            return Ok(0);
        }

        buf[..bytes_to_copy].copy_from_slice(&self.buffer[self.position..self.position + bytes_to_copy]);
        self.position += bytes_to_copy;
        Ok(bytes_to_copy)
    }
}

impl<R: Read> BufRead for CachedReadSeeker<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.ensure_available(self.position + 1)?;
        Ok(&self.buffer[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position = self.position.saturating_add(amt).min(self.buffer.len());
    }
}

impl<R: Read> Seek for CachedReadSeeker<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(offset) => i128::from(offset),
            SeekFrom::Current(offset) => self.position as i128 + i128::from(offset),
            SeekFrom::End(offset) => {
                self.read_to_end()?;
                self.buffer.len() as i128 + i128::from(offset)
            }
        };

        if target < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot seek before start of stream"));
        }

        let target = usize::try_from(target)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Seek target exceeds usize"))?;

        self.ensure_available(target)?;
        if target > self.buffer.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Cannot seek past end of stream"));
        }

        self.position = target;
        u64::try_from(self.position)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Seek position exceeds u64"))
    }
}
