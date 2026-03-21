export async function fetchProgress() {
    const response = await fetch("/api/progress");
    if (!response.ok) throw new Error(`Failed to fetch initial progress: ${response.status}`);
    return response.json();
}

export function updateProgress(viewer, activePage) {
    if (viewer.state.lockDepth > 0) return;
    viewer.state.progress = {
        page: activePage,
        scroll: Math.min(1, Math.max(0, (window.scrollY - activePage.slot.offsetTop) / activePage.slot.offsetHeight)),
    };
    if (viewer.saveTimer) clearTimeout(viewer.saveTimer);
    viewer.saveTimer = setTimeout(() => saveProgress(viewer.state.progress), 200);
}

function saveProgress(progress) {
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
