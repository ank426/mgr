use std::io;
use std::sync::Arc;

use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::cbz::{self, Volume};

pub async fn serve(
    title: String,
    volumes: Vec<Volume>,
    port: u16,
    prefetch_back: u32,
    prefetch_forward: u32,
    initial_volume_index: u32,
    initial_page_index: u32,
    initial_scroll: f64,
) {
    let volume_page_counts = volumes.iter().map(|volume| volume.pages.len() as u32).collect::<Vec<_>>();
    let html = build_html(
        &title,
        &volume_page_counts,
        prefetch_back,
        prefetch_forward,
        initial_volume_index,
        initial_page_index,
        initial_scroll,
    );
    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let state = Arc::new(volumes);
    let state_for_route = Arc::clone(&state);
    let page_route = warp::path!("volume" / u32 / "page" / u32)
        .and(warp::any().map(move || Arc::clone(&state_for_route)))
        .and_then(page_response);

    let routes = html_route.or(page_route);
    let addr = ([127, 0, 0, 1], port);
    println!("Open http://127.0.0.1:{port}");
    warp::serve(routes).run(addr).await;
}

fn build_html(
    title: &str,
    volume_page_counts: &[u32],
    prefetch_back: u32,
    prefetch_forward: u32,
    initial_volume_index: u32,
    initial_page_index: u32,
    initial_scroll: f64,
) -> String {
    let volume_page_counts = volume_page_counts.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");

    include_str!("viewer.html")
        .replace("{title}", title)
        .replace("{volume_page_counts}", &format!("[{volume_page_counts}]"))
        .replace("{prefetch_back}", &prefetch_back.to_string())
        .replace("{prefetch_forward}", &prefetch_forward.to_string())
        .replace("{initial_volume_index}", &initial_volume_index.to_string())
        .replace("{initial_page_index}", &initial_page_index.to_string())
        .replace("{initial_scroll}", &initial_scroll.to_string())
        .replace("__VIEWER_SCRIPT__", &include_str!("viewer.js").replace("</script", "<\\/script"))
}

async fn page_response(
    volume_index: u32,
    page_index: u32,
    state: Arc<Vec<Volume>>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let Some(volume) = state.get(volume_index as usize) else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/plain; charset=utf-8")
            .body(b"Not Found".to_vec())
            .expect("valid response"));
    };
    let Some(page) = volume.pages.get(page_index as usize) else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/plain; charset=utf-8")
            .body(b"Not Found".to_vec())
            .expect("valid response"));
    };

    let data = match load_page_bytes(volume.archive_path.clone(), page.name.clone()).await {
        Ok(data) => data,
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "text/plain; charset=utf-8")
                .body(format!("Failed to load volume {volume_index} page {page_index}: {err}").into_bytes())
                .expect("valid response"));
        }
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", page.mime)
        .header("cache-control", "no-store, no-cache, must-revalidate, max-age=0")
        .header("pragma", "no-cache")
        .header("expires", "0")
        .body(data)
        .expect("valid response"))
}

async fn load_page_bytes(archive_path: std::path::PathBuf, page_name: String) -> io::Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || cbz::load_page_bytes(&archive_path, &page_name))
        .await
        .map_err(|err| io::Error::other(format!("Page load task failed: {err}")))?
}
