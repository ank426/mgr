use std::path::{Path, PathBuf};

use anyhow::{bail, ensure};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::cbz::{Volume, is_cbz};
use crate::readlist::ReadList;

pub struct Manga {
    pub path: PathBuf,
    pub title: String,
    pub volumes: Vec<Volume>,
}

impl Manga {
    pub fn new(paths: &[PathBuf]) -> anyhow::Result<Self> {
        ensure!(!paths.is_empty(), "No files provided");

        let mut volumes = Vec::with_capacity(paths.len());
        for path in paths {
            ensure!(path.exists(), "Path does not exist: {}", path.display());
            ensure!(path.is_file(), "Path is not a file: {}", path.display());
            ensure!(is_cbz(path), "Unsupported file type: {} (expected .cbz)", path.display());

            let mokuro_path = path.with_extension("mokuro");
            let mokuro = mokuro_path.is_file().then_some(mokuro_path.to_string_lossy().into_owned());
            volumes.push(Volume::new(path, path.to_string_lossy().into_owned(), mokuro)?);
        }

        let title = if volumes.len() == 1 { volumes[0].name.clone() } else { "mgr".to_string() };
        Ok(Self { path: PathBuf::from("."), title, volumes })
    }

    pub fn from_readlist(dir_path: &Path, readlist: &ReadList) -> anyhow::Result<Self> {
        ensure!(!readlist.files.is_empty(), "Readlist has no files");

        ensure!(
            readlist.files.iter().any(|entry| entry.name == readlist.progress.file),
            "progress.file '{}' is not present in files",
            readlist.progress.file,
        );

        ensure!(
            readlist.progress.scroll.is_finite() && (0.0..=1.0).contains(&readlist.progress.scroll),
            "progress.scroll must be a finite value in [0.0, 1.0], found {}",
            readlist.progress.scroll,
        );

        ensure!(readlist.progress.page >= 1, "progress.page must be >= 1");

        let mut resolved = Vec::with_capacity(readlist.files.len());
        for entry in &readlist.files {
            let file_path = dir_path.join(&entry.name);
            ensure!(file_path.exists(), "Readlist file '{}' does not exist", file_path.display());
            ensure!(file_path.is_file(), "Readlist entry '{}' is not a file", file_path.display());
            ensure!(is_cbz(&file_path), "Readlist file '{}' is not a supported archive (.cbz)", file_path.display());
            resolved.push((entry, file_path));
        }

        let results: Vec<_> = resolved
            .into_par_iter()
            .map(|(entry, file_path)| (entry, Volume::new(&file_path, entry.name.clone(), entry.mokuro.clone())))
            .collect();

        let mut volumes = Vec::with_capacity(results.len());
        for (entry, result) in results {
            let volume = result?;

            if entry.name == readlist.progress.file && readlist.progress.page > volume.pages.len() as u32 {
                bail!(
                    "progress.page {} is out of range for '{}' (has {} pages)",
                    readlist.progress.page,
                    entry.name,
                    volume.pages.len(),
                );
            }

            volumes.push(volume);
        }

        let title = dir_path.file_name().and_then(|name| name.to_str()).unwrap_or("mgr").to_string();
        Ok(Self { path: dir_path.to_path_buf(), title, volumes })
    }
}
