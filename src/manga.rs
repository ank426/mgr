use std::path::{Path, PathBuf};

use crate::cbz::{Volume, is_cbz};
use crate::error::AppResult;
use crate::readlist::ReadList;

pub struct Manga {
    pub path: PathBuf,
    pub title: String,
    pub volumes: Vec<Volume>,
}

impl Manga {
    pub fn new(paths: &[PathBuf]) -> AppResult<Self> {
        if paths.is_empty() {
            return Err("No files provided".into());
        }

        let mut volumes = Vec::with_capacity(paths.len());
        for path in paths {
            if !path.exists() {
                return Err(format!("Path does not exist: {}", path.display()).into());
            }
            if !path.is_file() {
                return Err(format!("Path is not a file: {}", path.display()).into());
            }
            if !is_cbz(path) {
                return Err(format!("Unsupported file type: {} (expected .cbz)", path.display()).into());
            }

            volumes.push(Volume::new(path, path.to_string_lossy().into_owned())?);
        }

        Ok(Self { path: PathBuf::from("."), title: "mgr".to_string(), volumes })
    }

    pub fn from_readlist(dir_path: &Path, readlist: &ReadList) -> AppResult<Self> {
        if readlist.files.is_empty() {
            return Err("Readlist has no files".into());
        }

        if !readlist.files.iter().any(|entry| entry.name == readlist.progress.file) {
            return Err(format!("progress.file '{}' is not present in files", readlist.progress.file).into());
        }

        if !readlist.progress.scroll.is_finite() || !(0.0..=1.0).contains(&readlist.progress.scroll) {
            return Err(format!(
                "progress.scroll must be a finite value in [0.0, 1.0], found {}",
                readlist.progress.scroll
            )
            .into());
        }

        if readlist.progress.page == 0 {
            return Err("progress.page must be >= 1".into());
        }

        let mut volumes = Vec::with_capacity(readlist.files.len());

        for entry in &readlist.files {
            let file_path = dir_path.join(&entry.name);
            if !file_path.exists() {
                return Err(format!("Readlist file '{}' does not exist", file_path.display()).into());
            }
            if !file_path.is_file() {
                return Err(format!("Readlist entry '{}' is not a file", file_path.display()).into());
            }
            if !is_cbz(&file_path) {
                return Err(format!("Readlist file '{}' is not a supported archive (.cbz)", file_path.display()).into());
            }

            let volume = Volume::new(&file_path, entry.name.clone())?;

            if entry.name == readlist.progress.file && readlist.progress.page > volume.pages.len() as u32 {
                return Err(format!(
                    "progress.page {} is out of range for '{}' (has {} pages)",
                    readlist.progress.page,
                    entry.name,
                    volume.pages.len()
                )
                .into());
            }

            volumes.push(volume);
        }

        let title = dir_path.file_name().and_then(|name| name.to_str()).unwrap_or("mgr").to_string();
        Ok(Self { path: dir_path.to_path_buf(), title, volumes })
    }
}
