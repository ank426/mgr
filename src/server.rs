use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::config::Config;
use crate::manga::Manga;
use crate::readlist::ReadList;
use crate::routes::{self, Resp};

struct State {
    manga: Manga,
    config: Config,
    readlist_path: Option<PathBuf>,
    readlist: RwLock<Option<ReadList>>,
}

pub async fn serve(
    manga: Manga,
    config: Config,
    readlist_path: Option<PathBuf>,
    readlist: Option<ReadList>,
) -> anyhow::Result<()> {
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, config.port)).await?;
    announce(config.port, config.open);
    let state = Arc::new(State { manga, config, readlist_path, readlist: RwLock::new(readlist) });
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, _) = result?;
                let state = state.clone();
                tokio::spawn(async move {
                    let _ = http1::Builder::new()
                        .serve_connection(TokioIo::new(stream), service_fn(|req| route(req, state.clone())))
                        .await;
                });
            }
            _ = &mut shutdown => break,
        }
    }
    Ok(())
}

async fn route(req: Request<Incoming>, state: Arc<State>) -> Result<Resp, std::convert::Infallible> {
    let path = req.uri().path();
    Ok(match (req.method(), path) {
        (&Method::GET, "/") => routes::index(&state.manga.title),
        (&Method::GET, "/api/config") => routes::get_config(&state.config),
        (&Method::GET, "/api/volumes") => routes::get_volumes(&state.manga.volumes),
        (&Method::GET, "/api/progress") => routes::get_progress(&state.manga.volumes, &state.readlist),
        (&Method::PUT, "/api/progress") => routes::save_progress(req, &state.readlist_path, &state.readlist).await,
        (&Method::GET, _) if path.starts_with("/assets/") => routes::asset(&path["/assets/".len()..]),
        (&Method::GET, _) if path.starts_with("/volume/") => route_volume(&path["/volume/".len()..], &state).await,
        _ => routes::not_found(path),
    })
}

async fn route_volume(rest: &str, state: &State) -> Resp {
    let Some((volume, suffix)) = rest.split_once('/') else { return routes::not_found(rest) };
    let volume = percent_encoding::percent_decode_str(volume).decode_utf8_lossy();
    match suffix.split_once('/') {
        Some(("page", page_num)) => match page_num.parse() {
            Ok(page_num) => routes::page(&volume, page_num, &state.manga.path, &state.manga.volumes).await,
            Err(_) => routes::not_found(rest),
        },
        None if suffix == "mokuro" => routes::mokuro(&volume, &state.manga.path, &state.manga.volumes).await,
        _ => routes::not_found(rest),
    }
}

fn announce(port: u16, open: bool) {
    let url = format!("http://localhost:{port}");
    println!("Serving on {url}");
    if open {
        match open::that(&url) {
            Ok(()) => println!("Opening in browser"),
            Err(err) => eprintln!("Failed to open browser: {err}"),
        }
    }
}
