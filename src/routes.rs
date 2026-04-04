use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::Context;
use percent_encoding::percent_decode_str;
use rust_embed::RustEmbed;
use serde_json::json;
use tokio::fs;
use warp::http::{Response, StatusCode};

use crate::manga::Manga;
use crate::readlist::{Progress, ReadList};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn build_html(manga: &Manga, prefetch: (f32, f32)) -> anyhow::Result<String> {
    let volumes_json = serde_json::to_string(
        &manga
            .volumes
            .iter()
            .map(|volume| {
                let prefix = Path::new(&volume.name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| format!("{s}/"))
                    .unwrap_or_default();
                let strip = if volume.pages.iter().all(|p| p.name.starts_with(&prefix)) { prefix.len() } else { 0 };
                json!({
                    "name": &volume.name,
                    "mokuro": &volume.mokuro,
                    "pageInfos": volume.pages.iter().map(|page| json!({
                        "name": &page.name[strip..],
                        "dims": page.dimensions,
                    })).collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>(),
    )?;

    Ok(String::from_utf8(Assets::get("index.html").context("index.html not found")?.data.into_owned())?
        .replace("{title}", &manga.title)
        .replace("{volumes}", &volumes_json)
        .replace("{prefetch}", &format!("[{}, {}]", prefetch.0, prefetch.1)))
}

pub fn get_progress(manga: Arc<Manga>, readlist_lock: Arc<RwLock<Option<ReadList>>>) -> warp::reply::Json {
    let progress = readlist_lock
        .read()
        .unwrap()
        .as_ref()
        .map_or_else(|| Progress::new(manga.volumes[0].name.clone()), |r| r.progress.clone());
    warp::reply::json(&progress)
}

pub async fn save_progress(
    progress: Progress,
    readlist_path: Arc<Option<PathBuf>>,
    readlist_lock: Arc<RwLock<Option<ReadList>>>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let Some(path) = readlist_path.as_ref() else { return Ok(no_content_response()) };
    let snapshot = {
        let mut guard = readlist_lock.write().unwrap();
        let Some(readlist) = guard.as_mut() else { return Ok(no_content_response()) };
        readlist.progress = progress;
        readlist.clone()
    };
    match snapshot.save(path).await {
        Ok(()) => Ok(no_content_response()),
        Err(err) => Ok(internal_server_error_response(format!("Failed to save progress: {err}"))),
    }
}

pub async fn page_response(
    volume_name: String,
    page_number: usize,
    state: Arc<Manga>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let decoded_volume_name = percent_decode_str(&volume_name).decode_utf8_lossy();
    let Some(volume) = state.volumes.iter().find(|v| v.name == decoded_volume_name) else {
        return Ok(not_found_response());
    };
    let Some(page) = page_number.checked_sub(1).and_then(|i| volume.pages.get(i)) else {
        return Ok(not_found_response());
    };
    match page.load_bytes(state.path.join(&volume.name)).await {
        Ok(data) => Ok(ok_response(page.mime, data)),
        Err(err) => {
            Ok(internal_server_error_response(format!("Failed to load volume {volume_name} page {page_number}: {err}")))
        }
    }
}

pub async fn mokuro_response(volume_name: String, state: Arc<Manga>) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let decoded_volume_name = percent_decode_str(&volume_name).decode_utf8_lossy();
    let Some(volume) = state.volumes.iter().find(|volume| volume.name == decoded_volume_name) else {
        return Ok(not_found_response());
    };
    let Some(mokuro_name) = volume.mokuro.as_ref() else {
        return Ok(ok_response("application/json; charset=utf-8", b"{}".to_vec()));
    };
    let mokuro_path = Path::new(mokuro_name);
    let mokuro_path = if mokuro_path.is_absolute() { mokuro_path.to_path_buf() } else { state.path.join(mokuro_name) };
    match fs::read(&mokuro_path).await {
        Ok(data) => Ok(ok_response("application/json; charset=utf-8", data)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(not_found_response()),
        Err(err) => {
            Ok(internal_server_error_response(format!("Failed to load mokuro file {}: {err}", mokuro_path.display())))
        }
    }
}

pub async fn asset_response(asset_name: String) -> Result<Response<Vec<u8>>, warp::Rejection> {
    match Assets::get(&asset_name) {
        Some(file) => Ok(ok_response(asset_mime(&asset_name), file.data.into_owned())),
        None => Ok(not_found_response()),
    }
}

fn asset_mime(asset_name: &str) -> &'static str {
    match Path::new(asset_name).extension().and_then(|extension| extension.to_str()) {
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn not_found_response() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "text/plain; charset=utf-8")
        .body(b"Not Found".to_vec())
        .expect("valid response")
}

fn no_content_response() -> Response<Vec<u8>> {
    Response::builder().status(StatusCode::NO_CONTENT).body(Vec::new()).expect("valid response")
}

fn internal_server_error_response(message: String) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "text/plain; charset=utf-8")
        .body(message.into_bytes())
        .expect("valid response")
}

fn ok_response(mime: &str, data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(data)
        .expect("valid response")
}
