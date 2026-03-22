use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, RwLock};

use warp::Filter;
use warp::Reply;

use crate::manga::Manga;
use crate::readlist::{Progress, ReadList};
use crate::routes;

pub async fn serve(
    manga: Manga,
    port: u16,
    prefetch: (u32, u32),
    open: bool,
    readlist_path: Option<PathBuf>,
    readlist: Option<ReadList>,
) {
    let html = routes::build_html(&manga, prefetch);
    let manga = Arc::new(manga);
    let readlist_path = Arc::new(readlist_path);
    let shared_readlist = Arc::new(RwLock::new(readlist));

    let html_route = warp::path::end().map(move || warp::reply::html(html.clone()).into_response());

    let page_route = {
        let manga = Arc::clone(&manga);
        warp::path!("volume" / String / "page" / u32)
            .and(warp::any().map(move || Arc::clone(&manga)))
            .and_then(routes::page_response)
    };

    let mokuro_route = {
        let manga = Arc::clone(&manga);
        warp::path!("volume" / String / "mokuro")
            .and(warp::any().map(move || Arc::clone(&manga)))
            .and_then(routes::mokuro_response)
    };

    let assets_route = warp::path!("assets" / String).and_then(routes::asset_response);

    let get_progress_route = {
        let manga = Arc::clone(&manga);
        let shared_readlist = Arc::clone(&shared_readlist);
        warp::path!("api" / "progress")
            .and(warp::get())
            .map(move || warp::reply::json(&routes::get_progress(&manga, &shared_readlist)))
    };

    let save_progress_route = {
        let readlist_path = Arc::clone(&readlist_path);
        let shared_readlist = Arc::clone(&shared_readlist);
        warp::path!("api" / "progress")
            .and(warp::put())
            .and(warp::body::json())
            .map(move |progress: Progress| routes::save_progress(progress, &readlist_path, &shared_readlist))
    };

    println!("Open http://127.0.0.1:{port}");
    if open {
        open_browser(port);
    }

    let routes =
        html_route.or(page_route).or(mokuro_route).or(assets_route).or(get_progress_route).or(save_progress_route);
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
