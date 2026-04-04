use std::path::{Path, PathBuf};

use anyhow::ensure;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::cbz::{Volume, is_cbz};
use crate::readlist::ReadList;

pub struct Manga {
    pub path: PathBuf,
    pub title: String,
    pub volumes: Vec<Volume>,
}

impl Manga {
    pub fn new(paths: &[PathBuf]) -> anyhow::Result<Self> {
        let volumes: Vec<Volume> = paths
            .iter()
            .map(|path| {
                ensure!(path.is_file(), "No file exists at: {}", path.display());
                ensure!(is_cbz(path), "Unsupported file type: {} (expected .cbz)", path.display());
                let mokuro_path = path.with_extension("mokuro");
                let mokuro = mokuro_path.is_file().then_some(mokuro_path.to_string_lossy().into_owned());
                Volume::new(path, path.to_string_lossy().into_owned(), mokuro)
            })
            .collect::<anyhow::Result<_>>()?;
        let title = match volumes.as_slice() {
            [volume] => volume.name.clone(),
            _ => "mgr".to_string(),
        };
        Ok(Self { path: PathBuf::from("."), title, volumes })
    }

    pub fn from_readlist(dir_path: &Path, readlist: &ReadList) -> anyhow::Result<Self> {
        ensure!(!readlist.files.is_empty(), "Readlist has no files");
        ensure!(readlist.progress.page >= 1, "progress.page must be >= 1");
        ensure!(
            (0.0..=1.0).contains(&readlist.progress.scroll),
            "progress.scroll must be in [0.0, 1.0], found {}",
            readlist.progress.scroll,
        );
        ensure!(
            readlist.files.iter().any(|entry| entry.name == readlist.progress.file),
            "progress.file '{}' is not present in files",
            readlist.progress.file,
        );
        let volumes: Vec<_> = readlist
            .files
            .par_iter()
            .map(|entry| {
                let file_path = dir_path.join(&entry.name);
                ensure!(file_path.is_file(), "No file exists at: {}", file_path.display());
                ensure!(is_cbz(&file_path), "Unsupported file type: {} (expected .cbz)", file_path.display());
                let volume = Volume::new(&file_path, entry.name.clone(), entry.mokuro.clone())?;
                ensure!(
                    entry.name != readlist.progress.file || readlist.progress.page <= volume.pages.len() as u32,
                    "progress.page {} is out of range for '{}' (has {} pages)",
                    readlist.progress.page,
                    entry.name,
                    volume.pages.len(),
                );
                Ok(volume)
            })
            .collect::<anyhow::Result<_>>()?;
        let title = dir_path.file_name().and_then(|name| name.to_str()).unwrap_or("mgr").to_string();
        Ok(Self { path: dir_path.to_path_buf(), title, volumes })
    }
}
