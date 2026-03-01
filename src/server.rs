use std::sync::Arc;

use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::cbz::{Manga, Page};

pub async fn serve(manga: Manga, port: u16) {
    let pages = Arc::new(manga.pages);
    let title = manga.title;

    let html = build_html(&title, pages.len());
    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let pages_for_route = Arc::clone(&pages);
    let page_route = warp::path!("page" / u32)
        .and(warp::any().map(move || Arc::clone(&pages_for_route)))
        .map(page_response);

    let routes = html_route.or(page_route);
    let addr = ([127, 0, 0, 1], port);
    println!("Open http://127.0.0.1:{port}");
    warp::serve(routes).run(addr).await;
}

fn page_response(index: u32, pages: Arc<Vec<Page>>) -> Response<Vec<u8>> {
    if let Some(page) = pages.get(index as usize) {
        return Response::builder()
            .status(StatusCode::OK)
            .header("content-type", page.mime)
            .body(page.data.clone())
            .expect("valid response");
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "text/plain; charset=utf-8")
        .body(b"Not Found".to_vec())
        .expect("valid response")
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
