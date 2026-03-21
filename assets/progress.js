export async function fetchProgress() {
    const response = await fetch("/api/progress");
    if (!response.ok) throw new Error(`Failed to fetch initial progress: ${response.status}`);
    return response.json();
}

export function withProgressLock(viewer, action) {
    if (viewer.state.progressLocked) {
        action();
        return;
    }

    viewer.state.progressLocked = true;
    action();
    requestAnimationFrame(() => {
        if (viewer.state.progress) restoreProgress(viewer.state.progress);
        viewer.state.progressLocked = false;
    });
}

export function updateProgress(viewer, activePage = viewer.state.progress.page) {
    if (viewer.state.progressLocked) return;
    const top = window.scrollY || 0;
    viewer.state.progress = {
        page: activePage,
        scroll: Math.min(1, Math.max(0, (top - activePage.slot.offsetTop) / activePage.slot.offsetHeight)),
    };
}

export function restoreProgress(progress) {
    window.scrollTo({
        top: progress.page.slot.offsetTop + progress.scroll * progress.page.slot.offsetHeight,
    });
}

export function saveProgress(viewer) {
    fetch("/api/progress", {
        method: "PUT",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
            file: viewer.state.progress.page.volumeName,
            page: viewer.state.progress.page.pageNumber,
            scroll: viewer.state.progress.scroll,
        }),
    }).catch((error) => {
        console.error("Failed to save progress:", error);
    });
}

export function updateZoom(viewer, delta) {
    viewer.state.zoom = Math.min(500, Math.max(10, viewer.state.zoom + delta));
    withProgressLock(viewer, () =>
        document.documentElement.style.setProperty("--viewer-zoom", `${viewer.state.zoom}%`),
    );
}
