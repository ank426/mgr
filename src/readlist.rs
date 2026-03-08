use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use serde::{Deserialize, Serialize};
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value, value};

use crate::cbz;

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
    pub pages: u32,
    pub page_dims: Vec<[u32; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadList {
    pub progress: Progress,
    pub files: Vec<FileEntry>,
}

pub fn generate(dir: &Path, readlist_file_name: &str) -> io::Result<PathBuf> {
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
        let content = fs::read_to_string(&output_path)?;
        toml::from_str::<ReadList>(&content).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse {}: {err}", output_path.display()))
        })?
    } else {
        ReadList {
            progress: Progress { file: cbz_files.first().cloned().unwrap_or_default(), page: 1, scroll: 0.0 },
            files: Vec::new(),
        }
    };

    readlist.files = cbz_files
        .iter()
        .map(|file_name| {
            let file_path = dir.join(file_name);
            let volume = cbz::load_volume(&file_path)?;
            let mokuro_name = Path::new(file_name).with_extension("mokuro");
            let mokuro_path = dir.join(&mokuro_name);
            let mokuro = if mokuro_path.is_file() { Some(mokuro_name.to_string_lossy().to_string()) } else { None };

            Ok(FileEntry {
                name: file_name.clone(),
                mokuro,
                pages: volume.pages.len() as u32,
                page_dims: volume.pages.into_iter().map(|page| page.dimensions).collect(),
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    let output = format_readlist(&readlist)?;
    fs::write(&output_path, output)?;

    Ok(output_path)
}

pub fn load(path: &Path) -> io::Result<ReadList> {
    let content = fs::read_to_string(path)?;
    toml::from_str::<ReadList>(&content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse {}: {err}", path.display())))
}

fn format_readlist(readlist: &ReadList) -> io::Result<String> {
    let mut doc = DocumentMut::new();
    let mut progress = Table::new();
    progress["file"] = value(&readlist.progress.file);
    progress["page"] = value(i64::from(readlist.progress.page));
    progress["scroll"] = value(readlist.progress.scroll);
    doc["progress"] = Item::Table(progress);
    doc["files"] = Item::ArrayOfTables(format_files_array(&readlist.files));
    Ok(format!("{}\n# vim: set nowrap:\n", doc))
}

fn format_files_array(entries: &[FileEntry]) -> ArrayOfTables {
    let mut files = ArrayOfTables::new();

    for entry in entries {
        let mut file = Table::new();
        file["name"] = value(&entry.name);
        if let Some(mokuro) = &entry.mokuro {
            file["mokuro"] = value(mokuro);
        }
        file["pages"] = value(i64::from(entry.pages));
        file["page_dims"] = Item::Value(Value::Array(format_page_dims(&entry.page_dims)));
        files.push(file);
    }

    files
}

fn format_page_dims(page_dims: &[[u32; 2]]) -> Array {
    let mut dims = Array::new();

    for [width, height] in page_dims {
        let mut pair = Array::new();
        pair.push(i64::from(*width));
        pair.push(i64::from(*height));
        pair.fmt();
        dims.push(Value::Array(pair));
    }

    dims.fmt();
    dims
}
