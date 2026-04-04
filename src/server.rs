use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use warp::Filter;
use warp::Reply;

use crate::manga::Manga;
use crate::readlist::ReadList;
use crate::routes;

pub async fn serve(
    manga: Manga,
    port: u16,
    prefetch: (f32, f32),
    open: bool,
    readlist_path: Option<PathBuf>,
    readlist: Option<ReadList>,
) -> anyhow::Result<()> {
    let html = warp::hyper::body::Bytes::from(routes::build_html(&manga, prefetch)?);
    let manga = Arc::new(manga);
    let readlist_path = Arc::new(readlist_path);
    let shared_readlist = Arc::new(RwLock::new(readlist));

    let routes = warp::path::end()
        .map(move || warp::reply::html(html.clone()).into_response())
        .or(warp::path!("assets" / String).and_then(routes::asset_response))
        .or(warp::path!("volume" / String / "page" / u32).and(with(manga.clone())).and_then(routes::page_response))
        .or(warp::path!("volume" / String / "mokuro").and(with(manga.clone())).and_then(routes::mokuro_response))
        .or(warp::path!("api" / "progress")
            .and(warp::get())
            .and(with(manga.clone()))
            .and(with(shared_readlist.clone()))
            .map(|manga: Arc<Manga>, rl: Arc<RwLock<Option<ReadList>>>| {
                warp::reply::json(&routes::get_progress(&manga, &rl))
            }))
        .or(warp::path!("api" / "progress")
            .and(warp::put())
            .and(warp::body::json())
            .and(with(readlist_path.clone()))
            .and(with(shared_readlist.clone()))
            .and_then(routes::save_progress));

    opening_port(port, open);

    warp::serve(routes)
        .bind(([127, 0, 0, 1], port))
        .await
        .graceful(async { tokio::signal::ctrl_c().await.unwrap() })
        .run()
        .await;
    Ok(())
}

fn with<T: Clone + Send>(value: T) -> impl warp::Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || value.clone())
}

fn opening_port(port: u16, open: bool) {
    let url = format!("http://localhost:{port}");
    println!("Serving on {url}");
    if open {
        match open::that(&url) {
            Ok(()) => println!("Opening in browser"),
            Err(err) => eprintln!("Failed to open browser: {err}"),
        }
    }
}
