use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, RwLock};

use percent_encoding::percent_decode_str;
use serde_json::json;
use tokio::fs;
use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::manga::Manga;
use crate::readlist::{Progress, ReadList};

pub async fn serve(
    manga: Manga,
    port: u16,
    prefetch: (u32, u32),
    open: bool,
    readlist_path: Option<PathBuf>,
    readlist: Option<ReadList>,
) {
    let html = build_html(&manga, prefetch);
    let manga = Arc::new(manga);
    let readlist_path = Arc::new(readlist_path);
    let shared_readlist = Arc::new(RwLock::new(readlist));

    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let page_route = {
        let manga = Arc::clone(&manga);
        warp::path!("volume" / String / "page" / u32)
            .and(warp::any().map(move || Arc::clone(&manga)))
            .and_then(page_response)
    };

    let mokuro_route = {
        let manga = Arc::clone(&manga);
        warp::path!("volume" / String / "mokuro")
            .and(warp::any().map(move || Arc::clone(&manga)))
            .and_then(mokuro_response)
    };

    let assets_route = warp::path!("assets" / String).and_then(asset_response);

    let get_progress_route = {
        let manga = Arc::clone(&manga);
        let shared_readlist = Arc::clone(&shared_readlist);
        warp::path!("api" / "progress")
            .and(warp::get())
            .map(move || warp::reply::json(&get_progress(&manga, &shared_readlist)))
    };

    let save_progress_route = {
        let readlist_path = Arc::clone(&readlist_path);
        let shared_readlist = Arc::clone(&shared_readlist);
        warp::path!("api" / "progress")
            .and(warp::put())
            .and(warp::body::json())
            .map(move |progress: Progress| save_progress(progress, &readlist_path, &shared_readlist))
    };

    let routes =
        html_route.or(page_route).or(mokuro_route).or(assets_route).or(get_progress_route).or(save_progress_route);
    let addr = ([127, 0, 0, 1], port);
    println!("Open http://127.0.0.1:{port}");
    if open {
        open_browser(port);
    }
    warp::serve(routes).run(addr).await;
}

fn open_browser(port: u16) {
    let url = format!("http://localhost:{port}");
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", &url]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&url).spawn()
    } else {
        Command::new("xdg-open").arg(&url).spawn()
    };

    if let Err(err) = result {
        eprintln!("Failed to open browser: {err}");
    }
}

fn build_html(manga: &Manga, prefetch: (u32, u32)) -> String {
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

fn get_progress(manga: &Manga, shared_readlist: &RwLock<Option<ReadList>>) -> Progress {
    if let Ok(readlist) = shared_readlist.read()
        && let Some(readlist) = readlist.as_ref()
    {
        return readlist.progress.clone();
    }

    Progress { file: manga.volumes.first().map(|volume| volume.name.clone()).unwrap_or_default(), page: 1, scroll: 0.0 }
}

fn save_progress(
    progress: Progress,
    readlist_path: &Option<PathBuf>,
    shared_readlist: &RwLock<Option<ReadList>>,
) -> StatusCode {
    let mut readlist_to_save = None;
    let mut path_to_save = None;

    if let Ok(mut readlist) = shared_readlist.write()
        && let Some(readlist) = readlist.as_mut()
    {
        readlist.progress = progress;
        if let Some(path) = readlist_path.as_ref() {
            readlist_to_save = Some(readlist.clone());
            path_to_save = Some(path.clone());
        }
    }

    if let (Some(readlist), Some(path)) = (readlist_to_save, path_to_save) {
        tokio::task::spawn_blocking(move || {
            if let Err(err) = readlist.save(&path) {
                eprintln!("Failed to save progress: {err}");
            }
        });
    }

    StatusCode::NO_CONTENT
}

async fn page_response(
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

async fn mokuro_response(volume_name: String, state: Arc<Manga>) -> Result<Response<Vec<u8>>, warp::Rejection> {
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

async fn asset_response(asset_name: String) -> Result<Response<Vec<u8>>, warp::Rejection> {
    match asset_name.as_str() {
        "main.js" => Ok(ok_js_response(include_str!("../assets/main.js").as_bytes().to_vec())),
        "globals.js" => Ok(ok_js_response(include_str!("../assets/globals.js").as_bytes().to_vec())),
        "navigation.js" => Ok(ok_js_response(include_str!("../assets/navigation.js").as_bytes().to_vec())),
        "observers.js" => Ok(ok_js_response(include_str!("../assets/observers.js").as_bytes().to_vec())),
        "pages.js" => Ok(ok_js_response(include_str!("../assets/pages.js").as_bytes().to_vec())),
        "progress.js" => Ok(ok_js_response(include_str!("../assets/progress.js").as_bytes().to_vec())),
        "reconcile.js" => Ok(ok_js_response(include_str!("../assets/reconcile.js").as_bytes().to_vec())),
        "volume.js" => Ok(ok_js_response(include_str!("../assets/volume.js").as_bytes().to_vec())),
        _ => Ok(not_found_response()),
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

fn ok_js_response(data: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/javascript; charset=utf-8")
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(data)
        .expect("valid response")
}
