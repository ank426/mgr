use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn stage(label: impl Into<String>) -> StageTimer {
    StageTimer { label: label.into(), start: Instant::now(), enabled: enabled() }
}

pub struct StageTimer {
    label: String,
    start: Instant,
    enabled: bool,
}

impl Drop for StageTimer {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }

        let seconds = self.start.elapsed().as_secs_f64();
        eprintln!("[timing] {}: {:.3}s", self.label, seconds);
    }
}
