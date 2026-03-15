use std::fs;
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use serde::{Deserialize, Serialize};

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
    pub fn new(path: &Path) -> AppResult<Self> {
        let content = fs::read_to_string(path)?;
        toml::from_str::<ReadList>(&content).map_err(|err| format!("Failed to parse {}: {err}", path.display()).into())
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        let output = toml::to_string(self).map_err(|err| format!("Failed to serialize readlist: {err}"))?;
        fs::write(path, output)?;
        Ok(())
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
        ReadList::new(&output_path)?
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

    readlist.save(&output_path)?;
    Ok(output_path)
}
