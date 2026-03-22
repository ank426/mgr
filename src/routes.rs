use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

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

pub fn build_html(manga: &Manga, prefetch: (f32, f32)) -> String {
    let volumes_json = serde_json::to_string(
        &manga
            .volumes
            .iter()
            .map(|volume| {
                json!({
                    "name": &volume.name,
                    "mokuro": &volume.mokuro,
                    "pageDims": volume.pages.iter().map(|page| page.dimensions).collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>(),
    )
    .expect("valid viewer volumes json");

    include_str!("../assets/index.html")
        .replace("{title}", &manga.title)
        .replace("{volumes}", &volumes_json)
        .replace("{prefetch}", &format!("[{}, {}]", prefetch.0, prefetch.1))
}

pub fn get_progress(manga: &Manga, shared_readlist: &RwLock<Option<ReadList>>) -> Progress {
    if let Ok(readlist) = shared_readlist.read()
        && let Some(readlist) = readlist.as_ref()
    {
        return readlist.progress.clone();
    }

    Progress { file: manga.volumes.first().map(|volume| volume.name.clone()).unwrap_or_default(), page: 1, scroll: 0.0 }
}

pub async fn save_progress(
    progress: Progress,
    readlist_path: Arc<Option<PathBuf>>,
    shared_readlist: Arc<RwLock<Option<ReadList>>>,
) -> Result<StatusCode, warp::Rejection> {
    let save_data = if let Ok(mut guard) = shared_readlist.write()
        && let Some(readlist) = guard.as_mut()
        && let Some(path) = readlist_path.as_ref()
    {
        readlist.progress = progress;
        Some((readlist.clone(), path.clone()))
    } else {
        None
    };

    if let Some((readlist, path)) = save_data
        && let Err(err) = readlist.save(&path).await
    {
        eprintln!("Failed to save progress: {err}");
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn page_response(
    volume_name: String,
    page_number: u32,
    state: Arc<Manga>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let decoded_volume_name = percent_decode_str(&volume_name).decode_utf8_lossy();
    let Some(volume) = state.volumes.iter().find(|volume| volume.name == decoded_volume_name) else {
        return Ok(not_found_response());
    };
    let Some(page) = volume.pages.get((page_number - 1) as usize) else {
        return Ok(not_found_response());
    };
    let mime = match page.mime() {
        Some(mime) => mime,
        None => {
            return Ok(internal_server_error_response(format!(
                "Unsupported image type for volume {volume_name} page {page_number}: {}",
                page.name
            )));
        }
    };
    match page.load_bytes(state.path.join(&volume.name)).await {
        Ok(data) => Ok(ok_image_response(mime, data)),
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
        return Ok(ok_json_response(b"{}".to_vec()));
    };
    let mokuro_path = PathBuf::from(mokuro_name);
    let mokuro_path = if mokuro_path.is_absolute() { mokuro_path } else { state.path.join(mokuro_name) };
    match fs::read(&mokuro_path).await {
        Ok(data) => Ok(ok_json_response(data)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(not_found_response()),
        Err(err) => {
            Ok(internal_server_error_response(format!("Failed to load mokuro file {}: {err}", mokuro_path.display())))
        }
    }
}

pub async fn asset_response(asset_name: String) -> Result<Response<Vec<u8>>, warp::Rejection> {
    match Assets::get(&asset_name) {
        Some(file) => Ok(ok_asset_response(asset_mime(&asset_name), file.data.into_owned())),
        None => Ok(not_found_response()),
    }
}

fn asset_mime(asset_name: &str) -> &'static str {
    match Path::new(asset_name).extension().and_then(|extension| extension.to_str()) {
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("wasm") => "application/wasm",
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

fn internal_server_error_response(message: String) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "text/plain; charset=utf-8")
        .body(message.into_bytes())
        .expect("valid response")
}

fn ok_image_response(mime: &str, data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(data)
        .expect("valid response")
}

fn ok_json_response(data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json; charset=utf-8")
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(data)
        .expect("valid response")
}

fn ok_asset_response(mime: &str, data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(data)
        .expect("valid response")
}
