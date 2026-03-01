use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use alphanumeric_sort::compare_str;
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

pub fn generate(dir: &Path) -> io::Result<PathBuf> {
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

    let mut doc = DocumentMut::new();

    let first_file = cbz_files.first().cloned().unwrap_or_default();

    let mut progress = Table::new();
    progress["file"] = value(first_file.clone());
    progress["page"] = value(1);
    progress["scroll"] = value(0.0);
    doc["progress"] = Item::Table(progress);

    let mut files = ArrayOfTables::new();
    for file_name in &cbz_files {
        let mut entry = Table::new();
        entry["name"] = value(file_name.clone());

        let mokuro_name = Path::new(file_name).with_extension("mokuro");
        let mokuro_path = dir.join(&mokuro_name);
        if mokuro_path.is_file() {
            entry["mokuro"] = value(mokuro_name.to_string_lossy().to_string());
        }

        files.push(entry);
    }

    doc["files"] = Item::ArrayOfTables(files);

    let output_path = dir.join(".mgr.toml");
    fs::write(&output_path, doc.to_string())?;

    Ok(output_path)
}
