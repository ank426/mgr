use std::process::Command;
use std::sync::Arc;

use serde_json::json;
use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::manga::Manga;
use crate::readlist::Progress;

pub async fn serve(manga: Manga, progress: Progress, port: u16, prefetch: (u32, u32), open: bool) {
    let html = build_html(&manga, &progress, prefetch);
    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let state = Arc::new(manga);
    let state_for_route = Arc::clone(&state);
    let page_route = warp::path!("volume" / String / "page" / u32)
        .and(warp::any().map(move || Arc::clone(&state_for_route)))
        .and_then(page_response);

    let routes = html_route.or(page_route);
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

fn build_html(manga: &Manga, progress: &Progress, prefetch: (u32, u32)) -> String {
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
        .replace("{initial_volume_name}", &format!("{:?}", progress.file.as_str()))
        .replace("{initial_page_number}", &progress.page.to_string())
        .replace("{initial_scroll}", &progress.scroll.to_string())
        .replace("__VIEWER_SCRIPT__", &include_str!("viewer.js").replace("</script", "<\\/script"))
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
