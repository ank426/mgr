use std::io;
use std::sync::Arc;

use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::cbz;
use crate::manga::Manga;

pub async fn serve(manga: Manga, port: u16, prefetch_back: u32, prefetch_forward: u32) {
    let state = Arc::new(manga);

    let html = build_html(state.title(), state.page_count(), prefetch_back, prefetch_forward);
    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let state_for_route = Arc::clone(&state);
    let page_route = warp::path!("page" / u32)
        .and(warp::any().map(move || Arc::clone(&state_for_route)))
        .and_then(page_response);

    let routes = html_route.or(page_route);
    let addr = ([127, 0, 0, 1], port);
    println!("Open http://127.0.0.1:{port}");
    warp::serve(routes).run(addr).await;
}

async fn page_response(index: u32, state: Arc<Manga>) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let Some(page) = state.page(index) else {
        return Ok(not_found_response());
    };

    let archive_path = page.archive_path.clone();
    let page_name = page.page_name.clone();
    let data = match load_page_bytes(archive_path, page_name).await {
        Ok(data) => data,
        Err(err) => {
            let message = format!("Failed to load page {index}: {err}");
            return Ok(error_response(message));
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

fn build_html(title: &str, count: usize, prefetch_back: u32, prefetch_forward: u32) -> String {
    include_str!("viewer.html")
        .replace("{title}", title)
        .replace("{page_count}", &count.to_string())
        .replace("{prefetch_back}", &prefetch_back.to_string())
        .replace("{prefetch_forward}", &prefetch_forward.to_string())
        .replace("{viewer_script}", &include_str!("viewer.js").replace("</script", "<\\/script"))
}

fn not_found_response() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "text/plain; charset=utf-8")
        .body(b"Not Found".to_vec())
        .expect("valid response")
}

fn error_response(message: String) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "text/plain; charset=utf-8")
        .body(message.into_bytes())
        .expect("valid response")
}

async fn load_page_bytes(archive_path: std::path::PathBuf, page_name: String) -> io::Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || cbz::load_page_bytes(&archive_path, &page_name))
        .await
        .map_err(|err| io::Error::other(format!("Page load task failed: {err}")))?
}
