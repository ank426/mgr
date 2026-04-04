use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

use percent_encoding::percent_decode_str;
use rust_embed::RustEmbed;
use serde_json::json;
use tokio::fs;
use warp::http::{Response, StatusCode};

use crate::cbz::Volume;
use crate::config::Config;
use crate::readlist::{Progress, ReadList};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn index(title: Arc<String>) -> Response<Vec<u8>> {
    let html =
        String::from_utf8(Assets::get("index.html").unwrap().data.into_owned()).unwrap().replace("{title}", &title);
    ok_response("text/html", html.into_bytes())
}

pub fn get_config(config: Arc<Config>) -> Response<Vec<u8>> {
    ok_response("application/json", serde_json::to_vec(&*config).unwrap())
}

pub fn get_volumes(volumes: Arc<Vec<Volume>>) -> Response<Vec<u8>> {
    let volumes = volumes
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
                "hasMokuro": volume.mokuro.is_some(),
                "pageInfos": volume.pages.iter().map(|page| json!({
                    "name": &page.name[strip..],
                    "dims": page.dimensions,
                })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    ok_response("application/json", serde_json::to_vec(&volumes).unwrap())
}

pub async fn asset(asset_name: String) -> Result<Response<Vec<u8>>, warp::Rejection> {
    match Assets::get(&asset_name) {
        Some(file) => {
            let mime = match Path::new(&asset_name).extension().and_then(|ext| ext.to_str()) {
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("html") => "text/html",
                _ => "application/octet-stream",
            };
            Ok(ok_response(mime, file.data.into_owned()))
        }
        None => Ok(error_response(StatusCode::NOT_FOUND, format!("Asset not found: {asset_name}"))),
    }
}

pub fn get_progress(volumes: Arc<Vec<Volume>>, rl_lock: Arc<RwLock<Option<ReadList>>>) -> Response<Vec<u8>> {
    let progress =
        rl_lock.read().unwrap().as_ref().map_or_else(|| Progress::new(volumes[0].name.clone()), |r| r.progress.clone());
    ok_response("application/json", serde_json::to_vec(&progress).unwrap())
}

pub async fn save_progress(
    progress: Progress,
    readlist_path: Arc<Option<PathBuf>>,
    readlist_lock: Arc<RwLock<Option<ReadList>>>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let Some(path) = readlist_path.as_ref() else { return Ok(no_content_response()) };
    let snapshot = {
        let mut guard = readlist_lock.write().unwrap();
        let readlist = guard.as_mut().unwrap();
        readlist.progress = progress;
        readlist.clone()
    };
    match snapshot.save(path).await {
        Ok(()) => Ok(no_content_response()),
        Err(err) => Ok(error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save progress: {err}"))),
    }
}

pub async fn page(
    volume_name: String,
    page_number: usize,
    path: Arc<PathBuf>,
    volumes: Arc<Vec<Volume>>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let decoded_volume_name = percent_decode_str(&volume_name).decode_utf8_lossy();
    let Some(volume) = volumes.iter().find(|v| v.name == decoded_volume_name) else {
        return Ok(error_response(StatusCode::NOT_FOUND, format!("Volume not found: {volume_name}")));
    };
    let Some(page) = page_number.checked_sub(1).and_then(|i| volume.pages.get(i)) else {
        return Ok(error_response(StatusCode::NOT_FOUND, format!("Page not found: {volume_name} page {page_number}")));
    };
    match page.load_bytes(path.join(&volume.name)).await {
        Ok(data) => Ok(ok_response(page.mime, data)),
        Err(err) => Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load volume {volume_name} page {page_number}: {err}"),
        )),
    }
}

pub async fn mokuro(
    volume_name: String,
    path: Arc<PathBuf>,
    volumes: Arc<Vec<Volume>>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let decoded_volume_name = percent_decode_str(&volume_name).decode_utf8_lossy();
    let Some(mokuro_path) =
        volumes.iter().find(|v| v.name == decoded_volume_name).and_then(|v| v.mokuro.as_ref()).map(|m| path.join(m))
    else {
        return Ok(error_response(StatusCode::NOT_FOUND, format!("Mokuro not found: {volume_name}")));
    };
    match fs::read(&mokuro_path).await {
        Ok(data) => Ok(ok_response("application/json", data)),
        Err(err) => Ok(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load volume {volume_name} mokuro {}: {err}", mokuro_path.display()),
        )),
    }
}

fn no_content_response() -> Response<Vec<u8>> {
    Response::builder().status(StatusCode::NO_CONTENT).body(Vec::new()).unwrap()
}

fn error_response(status: StatusCode, message: String) -> Response<Vec<u8>> {
    Response::builder().status(status).body(message.into_bytes()).unwrap()
}

fn ok_response(mime: &str, data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .header("content-type", mime)
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(data)
        .unwrap()
}
