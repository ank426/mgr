use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use warp::Filter;

use crate::config::Config;
use crate::manga::Manga;
use crate::readlist::ReadList;
use crate::routes;

pub async fn serve(
    manga: Manga,
    config: Config,
    readlist_path: Option<PathBuf>,
    readlist: Option<ReadList>,
) -> anyhow::Result<()> {
    let port = config.port;
    let open = config.open;

    let path = with(manga.path);
    let vols = with(manga.volumes);
    let readlist_lock = with(RwLock::new(readlist));

    let routes = (warp::path::end().and(with(manga.title)).map(routes::index))
        .or(warp::path!("assets" / String).and_then(routes::asset))
        .or(warp::path!("volume" / String / "page" / usize).and(path.clone()).and(vols.clone()).and_then(routes::page))
        .or(warp::path!("volume" / String / "mokuro").and(path).and(vols.clone()).and_then(routes::mokuro))
        .or(warp::path!("api" / "config").and(warp::get()).and(with(config)).map(routes::get_config))
        .or(warp::path!("api" / "volumes").and(warp::get()).and(vols.clone()).map(routes::get_volumes))
        .or(warp::path!("api" / "progress")
            .and(warp::get())
            .and(vols)
            .and(readlist_lock.clone())
            .map(routes::get_progress))
        .or(warp::path!("api" / "progress")
            .and(warp::put())
            .and(warp::body::json())
            .and(with(readlist_path))
            .and(readlist_lock)
            .and_then(routes::save_progress));

    opening_port(port, open);

    let shutdown = async { tokio::signal::ctrl_c().await.unwrap() };
    warp::serve(routes).bind(([127, 0, 0, 1], port)).await.graceful(shutdown).run().await;
    Ok(())
}

fn with<T: Send + Sync + 'static>(
    value: T,
) -> impl warp::Filter<Extract = (Arc<T>,), Error = std::convert::Infallible> + Clone {
    let arc = Arc::new(value);
    warp::any().map(move || arc.clone())
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
