use std::sync::Arc;

use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::cbz::{self, Volume};
use crate::readlist::Progress;

pub async fn serve(title: String, volumes: Vec<Volume>, progress: Progress, port: u16, prefetch: (u32, u32)) {
    let html = build_html(&title, &volumes, &progress, prefetch);
    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let state = Arc::new(volumes);
    let state_for_route = Arc::clone(&state);
    let page_route = warp::path!("volume" / String / "page" / u32)
        .and(warp::any().map(move || Arc::clone(&state_for_route)))
        .and_then(page_response);

    let routes = html_route.or(page_route);
    let addr = ([127, 0, 0, 1], port);
    println!("Open http://127.0.0.1:{port}");
    warp::serve(routes).run(addr).await;
}

fn build_html(title: &str, volumes: &[Volume], progress: &Progress, prefetch: (u32, u32)) -> String {
    let volume_page_counts =
        volumes.iter().map(|volume| (volume.pages.len() as u32).to_string()).collect::<Vec<_>>().join(", ");
    let volume_names = volumes.iter().map(|volume| format!("{:?}", volume.file_name())).collect::<Vec<_>>().join(", ");

    include_str!("viewer.html")
        .replace("{title}", title)
        .replace("{volume_page_counts}", &format!("[{volume_page_counts}]"))
        .replace("{volume_names}", &format!("[{volume_names}]"))
        .replace("{prefetch_back}", &prefetch.0.to_string())
        .replace("{prefetch_forward}", &prefetch.1.to_string())
        .replace("{initial_volume_name}", &format!("{:?}", progress.file.as_str()))
        .replace("{initial_page_index}", &(progress.page - 1).to_string())
        .replace("{initial_scroll}", &progress.scroll.to_string())
        .replace("__VIEWER_SCRIPT__", &include_str!("viewer.js").replace("</script", "<\\/script"))
}

async fn page_response(
    volume_name: String,
    page_index: u32,
    state: Arc<Vec<Volume>>,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let Some(volume) = state.iter().find(|volume| volume.file_name() == volume_name) else {
        return Ok(not_found_response());
    };
    let Some(page) = volume.pages.get(page_index as usize) else {
        return Ok(not_found_response());
    };

    let data = match cbz::load_page_bytes(volume.archive_path.clone(), page.name.clone()).await {
        Ok(data) => data,
        Err(err) => {
            return Ok(internal_server_error_response(format!(
                "Failed to load volume {volume_name} page {page_index}: {err}"
            )));
        }
    };

    Ok(ok_image_response(page.mime, data))
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
