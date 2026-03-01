use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, RwLock};

use crate::cbz::{self, Manga};

pub struct Window {
    manga: Manga,
    cache: RwLock<HashMap<u32, Arc<Vec<u8>>>>,
    inflight_prefetch: Mutex<HashSet<u32>>,
    latest_center: AtomicU32,
    prefetch_running: AtomicBool,
    prefetch_back: u32,
    prefetch_forward: u32,
}

impl Window {
    pub fn new(manga: Manga, prefetch_back: u32, prefetch_forward: u32) -> Self {
        Self {
            manga,
            cache: RwLock::new(HashMap::new()),
            inflight_prefetch: Mutex::new(HashSet::new()),
            latest_center: AtomicU32::new(0),
            prefetch_running: AtomicBool::new(false),
            prefetch_back,
            prefetch_forward,
        }
    }

    pub fn title(&self) -> &str {
        &self.manga.title
    }

    pub fn page_count(&self) -> usize {
        self.manga.pages.len()
    }

    pub fn page_mime(&self, index: u32) -> Option<&'static str> {
        self.manga.pages.get(index as usize).map(|page| page.mime)
    }

    pub async fn get_page_with_prefetch(self: &Arc<Self>, index: u32) -> io::Result<Arc<Vec<u8>>> {
        let data = self.get_or_load_page(index).await?;
        self.latest_center.store(index, Ordering::Relaxed);
        self.maybe_spawn_prefetch();
        Ok(data)
    }

    async fn prefetch_window(self: &Arc<Self>, center: u32) {
        let Some((start, end)) = window_bounds(
            center,
            self.manga.pages.len(),
            self.prefetch_back,
            self.prefetch_forward,
        ) else {
            return;
        };

        for idx in start..=end {
            if let Err(err) = self.prefetch_page(idx).await {
                eprintln!("Prefetch failed for page {idx}: {err}");
            }
        }

        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|idx, _| *idx >= start && *idx <= end);
        }
    }

    fn maybe_spawn_prefetch(self: &Arc<Self>) {
        let started = self
            .prefetch_running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        if !started {
            return;
        }

        let state = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                let center = state.latest_center.load(Ordering::Acquire);
                state.prefetch_window(center).await;
                let latest = state.latest_center.load(Ordering::Acquire);
                if latest == center {
                    break;
                }
            }
            state.prefetch_running.store(false, Ordering::Release);
        });
    }

    async fn prefetch_page(&self, index: u32) -> io::Result<()> {
        if self.get_cached_page(index).is_some() {
            return Ok(());
        }

        {
            let mut inflight = self.inflight_prefetch.lock().expect("inflight prefetch lock poisoned");
            if inflight.contains(&index) {
                return Ok(());
            }
            inflight.insert(index);
        }

        let loaded = self.load_page(index).await;

        let mut inflight = self.inflight_prefetch.lock().expect("inflight prefetch lock poisoned");
        inflight.remove(&index);

        if let Ok(data) = loaded
            && let Ok(mut cache) = self.cache.write()
        {
            cache.entry(index).or_insert(data);
        }

        Ok(())
    }

    async fn get_or_load_page(&self, index: u32) -> io::Result<Arc<Vec<u8>>> {
        if let Some(cached) = self.get_cached_page(index) {
            return Ok(cached);
        }

        let loaded = self.load_page(index).await?;
        if let Ok(mut cache) = self.cache.write() {
            let entry = cache.entry(index).or_insert_with(|| Arc::clone(&loaded));
            return Ok(Arc::clone(entry));
        }
        Ok(loaded)
    }

    fn get_cached_page(&self, index: u32) -> Option<Arc<Vec<u8>>> {
        self.cache
            .read()
            .ok()
            .and_then(|cache| cache.get(&index).map(Arc::clone))
    }

    async fn load_page(&self, index: u32) -> io::Result<Arc<Vec<u8>>> {
        let archive_path = self.manga.archive_path.clone();
        let page_name = self.manga.pages[index as usize].name.clone();
        let bytes = tokio::task::spawn_blocking(move || cbz::load_page_bytes(&archive_path, &page_name))
            .await
            .map_err(|err| io::Error::other(format!("Page load task failed: {err}")))??;
        Ok(Arc::new(bytes))
    }
}

fn window_bounds(center: u32, total_pages: usize, prefetch_back: u32, prefetch_forward: u32) -> Option<(u32, u32)> {
    if total_pages == 0 {
        return None;
    }
    let last = total_pages as u32 - 1;
    let start = center.saturating_sub(prefetch_back);
    let end = center.saturating_add(prefetch_forward).min(last);
    Some((start, end))
}
