use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use alphanumeric_sort::compare_str;
use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};

use crate::cbz::is_archive;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub file: String,
    pub page: usize,
    pub scroll: f64,
}

impl Progress {
    pub fn new(file: String) -> Self {
        Self { file, page: 1, scroll: 0.0 }
    }
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
        let contents = toml::to_string(self).context("Failed to serialize readlist")?;
        write_atomic(path, contents.as_bytes()).await
    }
}

async fn write_atomic(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    let file_name = path.file_name().context("Readlist path has no filename")?;
    let mut temp_name = OsString::from(".");
    temp_name.push(file_name);
    temp_name.push(format!(".{}.{}.tmp", std::process::id(), TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)));
    let temp_path = path.with_file_name(temp_name);

    let result = async {
        tokio::fs::write(&temp_path, contents).await?;
        tokio::fs::rename(&temp_path, path).await?;
        Ok::<_, std::io::Error>(())
    }
    .await;

    if let Err(err) = result {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(err).with_context(|| format!("Failed to save readlist {}", path.display()));
    }
    Ok(())
}

pub async fn generate(dir: &Path, readlist_file_name: &str) -> anyhow::Result<PathBuf> {
    let output_path = dir.join(readlist_file_name);

    let mut archive_files: Vec<String> = fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            entry.file_type().ok()?.is_file().then_some(())?;
            is_archive(&path).then_some(())?;
            Some(path.file_name()?.to_str()?.to_owned())
        })
        .collect();

    ensure!(!archive_files.is_empty(), "No .cbz or .zip files found in {}", dir.display());
    archive_files.sort_by(|a, b| compare_str(a, b));

    let mut readlist = if output_path.is_file() {
        ReadList::new(&output_path)?
    } else {
        ReadList { progress: Progress::new(archive_files[0].clone()), files: Vec::new() }
    };

    readlist.files = archive_files
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
