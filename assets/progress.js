import { scheduleReconcile } from "./reconcile.js";
import { expandVolume } from "./volume.js";

export async function fetchProgress() {
    const response = await fetch("/api/progress");
    if (!response.ok) throw new Error(`Failed to fetch initial progress: ${response.status}`);
    return response.json();
}

export function withScrollRestore(viewer, action) {
    viewer.state.lockDepth++;
    try {
        action();
    } finally {
        if (--viewer.state.lockDepth === 0) {
            window.scrollTo({
                top:
                    viewer.state.progress.page.slot.offsetTop +
                    viewer.state.progress.scroll * viewer.state.progress.page.slot.offsetHeight,
            });
        }
    }
}

export function jumpToProgress(viewer, progress) {
    const volumeIndex = viewer.volumeByName.get(progress.file).index;
    const volume = viewer.config.volumes[volumeIndex];
    expandVolume(viewer, volumeIndex);

    withScrollRestore(viewer, () => {
        viewer.state.progress = {
            page: viewer.pagesByVolume.get(volume.name).get(progress.page),
            scroll: progress.scroll,
        };
    });

    scheduleReconcile(viewer);
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
