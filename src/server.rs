use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::sync::{Mutex, RwLock};

use warp::Filter;
use warp::Reply;
use warp::http::{Response, StatusCode};

use crate::cbz::{self, Manga};

struct ServerState {
    manga: Manga,
    cache: RwLock<HashMap<u32, Arc<Vec<u8>>>>,
    inflight_prefetch: Mutex<HashSet<u32>>,
    latest_center: AtomicU32,
    prefetch_running: AtomicBool,
    prefetch_back: u32,
    prefetch_forward: u32,
}

pub async fn serve(manga: Manga, port: u16, prefetch_back: u32, prefetch_forward: u32) {
    let state = Arc::new(ServerState {
        manga,
        cache: RwLock::new(HashMap::new()),
        inflight_prefetch: Mutex::new(HashSet::new()),
        latest_center: AtomicU32::new(0),
        prefetch_running: AtomicBool::new(false),
        prefetch_back,
        prefetch_forward,
    });

    let html = build_html(&state.manga.title, state.manga.pages.len());
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

async fn page_response(index: u32, state: Arc<ServerState>) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let page_count = state.manga.pages.len();
    if index as usize >= page_count {
        return Ok(not_found_response());
    }

    let mime = state.manga.pages[index as usize].mime;
    let data = match get_or_load_page(index, &state).await {
        Ok(data) => data,
        Err(err) => {
            let message = format!("Failed to load page {index}: {err}");
            return Ok(error_response(message));
        }
    };

    state.latest_center.store(index, Ordering::Relaxed);
    maybe_spawn_prefetch(Arc::clone(&state));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", mime)
        .body(data.as_ref().clone())
        .expect("valid response"))
}

fn maybe_spawn_prefetch(state: Arc<ServerState>) {
    let started = state
        .prefetch_running
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok();
    if !started {
        return;
    }

    tokio::spawn(async move {
        loop {
            let center = state.latest_center.load(Ordering::Acquire);
            prefetch_window(center, &state).await;
            let latest = state.latest_center.load(Ordering::Acquire);
            if latest == center {
                break;
            }
        }
        state.prefetch_running.store(false, Ordering::Release);
    });
}

async fn prefetch_window(center: u32, state: &Arc<ServerState>) {
    let Some((start, end)) = window_bounds(
        center,
        state.manga.pages.len(),
        state.prefetch_back,
        state.prefetch_forward,
    ) else {
        return;
    };

    for idx in start..=end {
        if let Err(err) = prefetch_page(idx, state).await {
            eprintln!("Prefetch failed for page {idx}: {err}");
        }
    }

    if let Ok(mut cache) = state.cache.write() {
        cache.retain(|idx, _| *idx >= start && *idx <= end);
    }
}

async fn prefetch_page(index: u32, state: &Arc<ServerState>) -> io::Result<()> {
    if get_cached_page(index, state).is_some() {
        return Ok(());
    }

    {
        let mut inflight = state
            .inflight_prefetch
            .lock()
            .expect("inflight prefetch lock poisoned");
        if inflight.contains(&index) {
            return Ok(());
        }
        inflight.insert(index);
    }

    let loaded = load_page(index, state).await;

    let mut inflight = state
        .inflight_prefetch
        .lock()
        .expect("inflight prefetch lock poisoned");
    inflight.remove(&index);

    if let Ok(data) = loaded {
        if let Ok(mut cache) = state.cache.write() {
            cache.entry(index).or_insert(data);
        }
    }

    Ok(())
}

async fn get_or_load_page(index: u32, state: &Arc<ServerState>) -> io::Result<Arc<Vec<u8>>> {
    if let Some(cached) = get_cached_page(index, state) {
        return Ok(cached);
    }

    let loaded = load_page(index, state).await?;
    if let Ok(mut cache) = state.cache.write() {
        let entry = cache.entry(index).or_insert_with(|| Arc::clone(&loaded));
        return Ok(Arc::clone(entry));
    }
    Ok(loaded)
}

fn get_cached_page(index: u32, state: &Arc<ServerState>) -> Option<Arc<Vec<u8>>> {
    state
        .cache
        .read()
        .ok()
        .and_then(|cache| cache.get(&index).map(Arc::clone))
}

async fn load_page(index: u32, state: &Arc<ServerState>) -> io::Result<Arc<Vec<u8>>> {
    let archive_path = state.manga.archive_path.clone();
    let page_name = state.manga.pages[index as usize].name.clone();
    let bytes = tokio::task::spawn_blocking(move || cbz::load_page_bytes(&archive_path, &page_name))
        .await
        .map_err(|err| io::Error::other(format!("Page load task failed: {err}")))??;
    Ok(Arc::new(bytes))
}

fn window_bounds(
    center: u32,
    total_pages: usize,
    prefetch_back: u32,
    prefetch_forward: u32,
) -> Option<(u32, u32)> {
    if total_pages == 0 {
        return None;
    }
    let last = total_pages as u32 - 1;
    let start = center.saturating_sub(prefetch_back);
    let end = center.saturating_add(prefetch_forward).min(last);
    Some((start, end))
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
