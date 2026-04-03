use std::fs;
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

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
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        toml::from_str::<ReadList>(&fs::read_to_string(path)?).context(format!("Failed to parse {}", path.display()))
    }

    pub async fn save(&self, path: &Path) -> anyhow::Result<()> {
        Ok(tokio::fs::write(path, toml::to_string(self).context("Failed to serialize readlist")?).await?)
    }
}

pub async fn generate(dir: &Path, readlist_file_name: &str) -> anyhow::Result<PathBuf> {
    let output_path = dir.join(readlist_file_name);

    let mut cbz_files: Vec<String> = fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            entry.file_type().ok()?.is_file().then_some(())?;
            path.extension()?.to_str()?.eq_ignore_ascii_case("cbz").then_some(())?;
            Some(path.file_name()?.to_str()?.to_owned())
        })
        .collect();

    cbz_files.sort_by(|a, b| compare_str(a, b));

    if cbz_files.is_empty() {
        bail!("No .cbz files found in {}", dir.display());
    }

    let mut readlist = if output_path.is_file() {
        ReadList::new(&output_path)?
    } else {
        ReadList { progress: Progress { file: cbz_files[0].clone(), page: 1, scroll: 0.0 }, files: Vec::new() }
    };

    readlist.files = cbz_files
        .into_iter()
        .map(|file_name| {
            let mokuro_name = Path::new(&file_name).with_extension("mokuro");
            let mokuro = dir.join(&mokuro_name).is_file().then(|| mokuro_name.to_string_lossy().into_owned());
            FileEntry { name: file_name.clone(), mokuro }
        })
        .collect();

    readlist.save(&output_path).await?;
    Ok(output_path)
}
