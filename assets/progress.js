import { state } from "./globals.js";

export function withProgressLock(action) {
    if (state.progressLocked) {
        action();
        return;
    }
    state.progressLocked = true;
    action();
    requestAnimationFrame(() => {
        if (state.progress) restoreProgress(state.progress);
        requestAnimationFrame(() => {
            state.progressLocked = false;
        });
    });
}

export function updateProgress(activePage = state.progress.page) {
    if (state.progressLocked) return;
    const top = window.scrollY || 0;
    state.progress = {
        page: activePage,
        scroll: Math.min(1, Math.max(0, (top - activePage.slot.offsetTop) / activePage.slot.offsetHeight)),
    };
}

export function restoreProgress(progress) {
    window.scrollTo({
        top: progress.page.slot.offsetTop + progress.scroll * progress.page.slot.offsetHeight,
    });
}

export function saveProgress() {
    fetch("/api/progress", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
            file: state.progress.page.volumeName,
            page: state.progress.page.pageNumber,
            scroll: state.progress.scroll,
        }),
    }).catch((error) => {
        console.error("Failed to save progress:", error);
    });
}
