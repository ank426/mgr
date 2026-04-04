use std::path::{Path, PathBuf};
use std::sync::RwLock;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};
use rust_embed::RustEmbed;
use serde_json::json;

use crate::cbz::Volume;
use crate::config::Config;
use crate::readlist::{Progress, ReadList};

pub type Resp = Response<Full<Bytes>>;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn index(title: &str) -> Resp {
    let html =
        String::from_utf8(Assets::get("index.html").unwrap().data.into_owned()).unwrap().replace("{title}", title);
    ok_response("text/html", html.into_bytes())
}

pub fn get_config(config: &Config) -> Resp {
    ok_response("application/json", serde_json::to_vec(config).unwrap())
}

pub fn get_volumes(volumes: &[Volume]) -> Resp {
    let volumes = volumes
        .iter()
        .map(|vol| {
            let prefix =
                Path::new(&vol.name).file_stem().and_then(|s| s.to_str()).map(|s| format!("{s}/")).unwrap_or_default();
            let strip = if vol.pages.iter().all(|p| p.name.starts_with(&prefix)) { prefix.len() } else { 0 };
            json!({
                "name": &vol.name,
                "hasMokuro": vol.mokuro.is_some(),
                "pageInfos": vol.pages.iter().map(|page| json!({
                    "name": &page.name[strip..],
                    "dims": page.dimensions,
                })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    ok_response("application/json", serde_json::to_vec(&volumes).unwrap())
}

pub fn asset(name: &str) -> Resp {
    match Assets::get(name) {
        Some(file) => {
            let mime = match Path::new(name).extension().and_then(|ext| ext.to_str()) {
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("html") => "text/html",
                _ => "application/octet-stream",
            };
            ok_response(mime, file.data.into_owned())
        }
        None => error_response(StatusCode::NOT_FOUND, format!("Asset not found: {name}")),
    }
}

pub fn get_progress(vols: &[Volume], readlist: &RwLock<Option<ReadList>>) -> Resp {
    let progress =
        readlist.read().unwrap().as_ref().map_or_else(|| Progress::new(vols[0].name.clone()), |r| r.progress.clone());
    ok_response("application/json", serde_json::to_vec(&progress).unwrap())
}

pub async fn save_progress(
    req: Request<Incoming>,
    readlist_path: &Option<PathBuf>,
    readlist_lock: &RwLock<Option<ReadList>>,
) -> Resp {
    let Some(path) = readlist_path.as_ref() else { return no_content_response() };
    let body = match req.collect().await {
        Ok(b) => b.to_bytes(),
        Err(err) => return error_response(StatusCode::BAD_REQUEST, format!("Failed to read body: {err}")),
    };
    let progress: Progress = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, format!("Invalid JSON: {err}")),
    };
    let snapshot = {
        let mut guard = readlist_lock.write().unwrap();
        let readlist = guard.as_mut().unwrap();
        readlist.progress = progress;
        readlist.clone()
    };
    match snapshot.save(path).await {
        Ok(()) => no_content_response(),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save progress: {err}")),
    }
}

pub async fn page(volume_name: &str, page_number: usize, path: &Path, volumes: &[Volume]) -> Resp {
    let Some(volume) = volumes.iter().find(|v| v.name == volume_name) else {
        return error_response(StatusCode::NOT_FOUND, format!("Volume not found: {volume_name}"));
    };
    let Some(page) = page_number.checked_sub(1).and_then(|i| volume.pages.get(i)) else {
        return error_response(StatusCode::NOT_FOUND, format!("Page not found: {volume_name} page {page_number}"));
    };
    match page.load_bytes(path.join(&volume.name)).await {
        Ok(data) => ok_response(page.mime, data),
        Err(err) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load volume {volume_name} page {page_number}: {err}"),
        ),
    }
}

pub async fn mokuro(volume_name: &str, path: &Path, volumes: &[Volume]) -> Resp {
    let Some(path) =
        volumes.iter().find(|v| v.name == volume_name).and_then(|v| v.mokuro.as_ref()).map(|m| path.join(m))
    else {
        return error_response(StatusCode::NOT_FOUND, format!("Mokuro not found: {volume_name}"));
    };
    match tokio::fs::read(&path).await {
        Ok(data) => ok_response("application/json", data),
        Err(err) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load mokuro {}: {err}", path.display()),
        ),
    }
}

pub fn not_found(path: &str) -> Resp {
    error_response(StatusCode::NOT_FOUND, format!("Not found: {path}"))
}

fn no_content_response() -> Resp {
    Response::builder().status(StatusCode::NO_CONTENT).body(Full::new(Bytes::new())).unwrap()
}

fn error_response(status: StatusCode, message: String) -> Resp {
    Response::builder().status(status).body(Full::new(Bytes::from(message))).unwrap()
}

fn ok_response(mime: &str, data: Vec<u8>) -> Resp {
    Response::builder()
        .header("content-type", mime)
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(Full::new(Bytes::from(data)))
        .unwrap()
}
