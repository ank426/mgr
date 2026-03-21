export async function fetchProgress() {
    const response = await fetch("/api/progress");
    if (!response.ok) throw new Error(`Failed to fetch initial progress: ${response.status}`);
    return response.json();
}

export function withLock(state, action) {
    if (state.locked) {
        action();
        return;
    }

    state.locked = true;
    action();
    requestAnimationFrame(() => {
        if (state.progress) restoreScroll(state.progress);
        state.locked = false;
    });
}

export function updateProgress(state, activePage = state.progress.page) {
    if (state.locked) return;
    const top = window.scrollY || 0;
    state.progress = {
        page: activePage,
        scroll: Math.min(1, Math.max(0, (top - activePage.slot.offsetTop) / activePage.slot.offsetHeight)),
    };
}

export function restoreScroll(progress) {
    window.scrollTo({
        top: progress.page.slot.offsetTop + progress.scroll * progress.page.slot.offsetHeight,
    });
}

export function saveProgress(progress) {
    fetch("/api/progress", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
            file: progress.page.volumeName,
            page: progress.page.pageNumber,
            scroll: progress.scroll,
        }),
    }).catch((error) => {
        console.error("Failed to save progress:", error);
    });
}

export function zoomBy(state, delta) {
    state.zoom = Math.min(500, Math.max(10, state.zoom + delta));
    withLock(state, () =>
        document.documentElement.style.setProperty("--viewer-zoom", `${state.zoom}%`),
    );
}
