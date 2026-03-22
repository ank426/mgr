import { scheduleReconcile } from "./reconcile.js";

export class Progress {
    constructor(viewer, volumeName, pageNumber, scroll) {
        const volume = viewer.volumeByName.get(volumeName);
        if (!volume) throw new Error(`Unknown volume: ${volumeName}`);
        volume.expand(viewer);
        const page = volume.pages?.get(pageNumber);
        if (!page) throw new Error(`Unknown page: ${volumeName}#${pageNumber}`);
        this.page = page;
        this.scroll = scroll;
    }

    static async fetch(viewer) {
        const response = await fetch("/api/progress");
        if (!response.ok) throw new Error(`Failed to fetch initial progress: ${response.status}`);
        const target = await response.json();
        return new Progress(viewer, target.file, target.page, target.scroll);
    }

    jumpTo(viewer) {
        withScrollRestore(viewer, () => {
            viewer.state.progress = this;
        });
        scheduleReconcile(viewer);
    }

    save() {
        fetch("/api/progress", {
            method: "PUT",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({
                file: this.page.volumeName,
                page: this.page.pageNumber,
                scroll: this.scroll,
            }),
        }).catch((error) => {
            console.error("Failed to save progress:", error);
        });
    }
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

export function updateProgress(viewer, activePage) {
    if (viewer.state.lockDepth > 0) return;
    viewer.state.progress.page = activePage;
    const scroll = (window.scrollY - activePage.slot.offsetTop) / activePage.slot.offsetHeight;
    viewer.state.progress.scroll = Math.min(1, Math.max(0, scroll));
    if (viewer.saveTimer) clearTimeout(viewer.saveTimer);
    viewer.saveTimer = setTimeout(() => viewer.state.progress?.save(), 200);
}
