use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Progress {
    file: String,
    page: u32,
    scroll: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mokuro: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadList {
    progress: Progress,
    files: Vec<FileEntry>,
}

pub struct LoadedReadList {
    pub progress_file: String,
    pub progress_page: u32,
    pub progress_scroll: f64,
    pub files: Vec<String>,
}

pub fn generate(dir: &Path) -> io::Result<PathBuf> {
    let output_path = dir.join(".mgr.toml");

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
        let content = fs::read_to_string(&output_path)?;
        toml::from_str::<ReadList>(&content).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse {}: {err}", output_path.display()),
            )
        })?
    } else {
        ReadList {
            progress: Progress {
                file: cbz_files.first().cloned().unwrap_or_default(),
                page: 1,
                scroll: 0.0,
            },
            files: Vec::new(),
        }
    };

    readlist.files = cbz_files
        .iter()
        .map(|file_name| {
            let mokuro_name = Path::new(file_name).with_extension("mokuro");
            let mokuro_path = dir.join(&mokuro_name);
            let mokuro = if mokuro_path.is_file() {
                Some(mokuro_name.to_string_lossy().to_string())
            } else {
                None
            };

            FileEntry {
                name: file_name.clone(),
                mokuro,
            }
        })
        .collect();

    let output = toml::to_string_pretty(&readlist).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize readlist: {err}"),
        )
    })?;
    fs::write(&output_path, output)?;

    Ok(output_path)
}

pub fn load(path: &Path) -> io::Result<LoadedReadList> {
    let content = fs::read_to_string(path)?;
    let readlist = toml::from_str::<ReadList>(&content).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse {}: {err}", path.display()),
        )
    })?;

    Ok(LoadedReadList {
        progress_file: readlist.progress.file,
        progress_page: readlist.progress.page,
        progress_scroll: readlist.progress.scroll,
        files: readlist.files.into_iter().map(|entry| entry.name).collect(),
    })
}
