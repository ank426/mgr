use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use alphanumeric_sort::compare_str;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

use crate::cbz;
use crate::error::AppResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub file: String,
    pub page: u32,
    pub scroll: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mokuro: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadList {
    pub progress: Progress,
    pub files: Vec<FileEntry>,
}

impl ReadList {
    pub fn load_volumes(&self, root: &Path) -> AppResult<Vec<cbz::Volume>> {
        if self.files.is_empty() {
            return Err("Readlist has no files".into());
        }

        if !self.files.iter().any(|entry| entry.name == self.progress.file) {
            return Err(format!("progress.file '{}' is not present in files", self.progress.file).into());
        }

        if !self.progress.scroll.is_finite() || !(0.0..=1.0).contains(&self.progress.scroll) {
            return Err(format!(
                "progress.scroll must be a finite value in [0.0, 1.0], found {}",
                self.progress.scroll
            )
            .into());
        }

        if self.progress.page == 0 {
            return Err("progress.page must be >= 1".into());
        }

        let mut volumes = Vec::with_capacity(self.files.len());
        let started_at = Instant::now();
        let progress = ProgressBar::new(self.files.len() as u64);
        progress.set_style(
            ProgressStyle::with_template("Loading volumes [{bar:24}] {pos}/{len} {elapsed_precise} {msg}")
                .expect("valid progress bar template")
                .progress_chars("=> "),
        );

        for entry in &self.files {
            progress.set_message(entry.name.clone());

            let file_path = root.join(&entry.name);
            if !file_path.exists() {
                return Err(format!("Readlist file '{}' does not exist", file_path.display()).into());
            }
            if !file_path.is_file() {
                return Err(format!("Readlist entry '{}' is not a file", file_path.display()).into());
            }
            if !cbz::is_cbz(&file_path) {
                return Err(format!("Readlist file '{}' is not a supported archive (.cbz)", file_path.display()).into());
            }

            let volume = cbz::load_volume(&file_path)?;

            if entry.name == self.progress.file && self.progress.page > volume.pages.len() as u32 {
                return Err(format!(
                    "progress.page {} is out of range for '{}' (has {} pages)",
                    self.progress.page,
                    entry.name,
                    volume.pages.len()
                )
                .into());
            }

            volumes.push(volume);
            progress.inc(1);
        }

        let elapsed = started_at.elapsed().as_secs_f64();
        progress.finish_and_clear();
        eprintln!("Loaded {} volumes in {:.3}s", self.files.len(), elapsed);

        Ok(volumes)
    }
}

pub fn generate(dir: &Path, readlist_file_name: &str) -> AppResult<PathBuf> {
    let output_path = dir.join(readlist_file_name);

    let mut cbz_files: Vec<String> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?.to_str()?;
            if !ext.eq_ignore_ascii_case("cbz") {
                return None;
            }
            path.file_name()?.to_str().map(ToOwned::to_owned)
        })
        .collect();

    cbz_files.sort_by(|a, b| compare_str(a, b));

    let mut readlist = if output_path.is_file() {
        load(&output_path)?
    } else {
        ReadList {
            progress: Progress { file: cbz_files.first().cloned().unwrap_or_default(), page: 1, scroll: 0.0 },
            files: Vec::new(),
        }
    };

    readlist.files = cbz_files
        .iter()
        .map(|file_name| {
            let mokuro_name = Path::new(file_name).with_extension("mokuro");
            let mokuro_path = dir.join(&mokuro_name);
            let mokuro = if mokuro_path.is_file() { Some(mokuro_name.to_string_lossy().to_string()) } else { None };

            FileEntry { name: file_name.clone(), mokuro }
        })
        .collect();

    let output = toml::to_string(&readlist).map_err(|err| format!("Failed to serialize readlist: {err}"))?;
    fs::write(&output_path, output)?;

    Ok(output_path)
}

pub fn load(path: &Path) -> AppResult<ReadList> {
    let content = fs::read_to_string(path)?;
    toml::from_str::<ReadList>(&content).map_err(|err| format!("Failed to parse {}: {err}", path.display()).into())
}
