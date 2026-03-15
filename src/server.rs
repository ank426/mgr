use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use serde_json::json;
use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::manga::Manga;
use crate::readlist;
use crate::readlist::Progress;

pub async fn serve(manga: Manga, port: u16, prefetch: (u32, u32), open: bool, readlist_path: Option<PathBuf>) {
    let html = build_html(&manga, prefetch);
    let manga = Arc::new(manga);
    let readlist_path = Arc::new(readlist_path);
    let save_lock = Arc::new(Mutex::new(()));

    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let page_route = {
        let manga = Arc::clone(&manga);
        warp::path!("volume" / String / "page" / u32)
            .and(warp::any().map(move || Arc::clone(&manga)))
            .and_then(page_response)
    };

    let get_progress_route = {
        let manga = Arc::clone(&manga);
        let readlist_path = Arc::clone(&readlist_path);
        let save_lock = Arc::clone(&save_lock);
        warp::path!("api" / "progress")
            .and(warp::get())
            .map(move || warp::reply::json(&get_progress(&manga, &readlist_path, &save_lock)))
    };

    let save_progress_route = {
        let readlist_path = Arc::clone(&readlist_path);
        let save_lock = Arc::clone(&save_lock);
        warp::path!("api" / "progress")
            .and(warp::put())
            .and(warp::body::json())
            .map(move |progress: Progress| save_progress(progress, &readlist_path, &save_lock))
    };

    let routes = html_route.or(page_route).or(get_progress_route).or(save_progress_route);
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
                    "pageDims": volume.pages.iter().map(|page| page.dimensions).collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>(),
    )
    .expect("valid viewer volumes json");

    include_str!("viewer.html")
        .replace("{title}", &manga.title)
        .replace("{volumes}", &volumes_json)
        .replace("{prefetch}", &format!("[{}, {}]", prefetch.0, prefetch.1))
        .replace("__VIEWER_SCRIPT__", &include_str!("viewer.js").replace("</script", "<\\/script"))
}

fn get_progress(manga: &Manga, readlist_path: &Option<PathBuf>, save_lock: &Mutex<()>) -> Progress {
    if let Some(path) = readlist_path.as_ref()
        && let Ok(_guard) = save_lock.lock()
        && let Ok(readlist) = readlist::ReadList::new(path)
    {
        return readlist.progress;
    }

    Progress { file: manga.volumes.first().map(|volume| volume.name.clone()).unwrap_or_default(), page: 1, scroll: 0.0 }
}

fn save_progress(progress: Progress, readlist_path: &Option<PathBuf>, save_lock: &Mutex<()>) -> StatusCode {
    if let Some(path) = readlist_path.as_ref()
        && let Ok(_guard) = save_lock.lock()
        && let Err(err) = readlist::update_progress(path, progress)
    {
        eprintln!("Failed to save progress: {err}");
    }

    StatusCode::NO_CONTENT
}

async fn page_response(
    volume_name: String,
    page_number: u32,
    state: Arc<Manga>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let Some(volume) = state.volumes.iter().find(|volume| volume.name == volume_name) else {
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
    let volume_path = state.path.join(&volume.name);
    let data = match page.load_bytes(volume_path).await {
        Ok(data) => data,
        Err(err) => {
            return Ok(internal_server_error_response(format!(
                "Failed to load volume {volume_name} page {page_number}: {err}"
            )));
        }
    };

    Ok(ok_image_response(mime, data))
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
