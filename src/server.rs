use std::sync::Arc;

use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::manga::Manga;
use crate::window::Window;

pub async fn serve(manga: Manga, port: u16, prefetch_back: u32, prefetch_forward: u32) {
    let state = Arc::new(Window::new(manga, prefetch_back, prefetch_forward));

    let html = build_html(state.title(), state.page_count());
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

async fn page_response(index: u32, state: Arc<Window>) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let Some(mime) = state.page_mime(index) else {
        return Ok(not_found_response());
    };

    let data = match state.get_page_with_prefetch(index).await {
        Ok(data) => data,
        Err(err) => {
            let message = format!("Failed to load page {index}: {err}");
            return Ok(error_response(message));
        }
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .body(data.as_ref().clone())
        .expect("valid response"))
}

fn build_html(title: &str, count: usize) -> String {
    let mut images = String::new();
    for idx in 0..count {
        images.push_str(&format!(
            "<img src=\"/page/{idx}\" loading=\"lazy\" decoding=\"async\" alt=\"page {idx}\" />\n"
        ));
    }

    include_str!("viewer.html")
        .replace("{title}", title)
        .replace("{images}", &images)
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
