use std::convert::Infallible;
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
    let page_route = warp::path!("page" / usize)
        .and(with_pages(pages_for_route))
        .map(page_response);

    let routes = html_route.or(page_route);
    let addr = ([127, 0, 0, 1], port);
    println!("Open http://127.0.0.1:{port}");
    warp::serve(routes).run(addr).await;
}

fn with_pages(
    pages: Arc<Vec<Page>>,
) -> impl Filter<Extract = (Arc<Vec<Page>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&pages))
}

fn page_response(index: usize, pages: Arc<Vec<Page>>) -> Response<Vec<u8>> {
    if let Some(page) = pages.get(index) {
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

    format!(
        "<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
  <title>{title}</title>
  <style>
    :root {{
      color-scheme: light;
    }}
    body {{
      margin: 0;
      background: #0e0e0e;
      color: #f6f6f6;
      font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif;
    }}
    main {{
      width: 100%;
    }}
    img {{
      display: block;
      width: 100%;
      height: auto;
      margin: 0;
    }}
  </style>
</head>
<body>
  <main>
    {images}
  </main>
</body>
</html>"
    )
}
