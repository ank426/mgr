use std::path::PathBuf;
use std::process::Command;
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
) {
    let html = warp::hyper::body::Bytes::from(routes::build_html(&manga, prefetch));
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

    println!("Open http://127.0.0.1:{port}");
    if open {
        open_browser(port);
    }

    warp::serve(routes)
        .bind(([127, 0, 0, 1], port))
        .await
        .graceful(async {
            match tokio::signal::ctrl_c().await {
                Ok(()) => println!("\nShutting down..."),
                Err(err) => eprintln!("Failed to install CTRL+C handler: {err}"),
            }
        })
        .run()
        .await;
}

fn with<T: Clone + Send>(value: T) -> impl warp::Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || value.clone())
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
