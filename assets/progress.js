import { scheduleReconcile } from "./reconcile.js";

export class Progress {
    constructor() {
        this.page = null;
        this.scroll = 0;
    }

    static async fetchAndJump(viewer) {
        const response = await fetch("/api/progress");
        if (!response.ok) throw new Error(`Failed to fetch initial progress: ${response.status}`);
        const target = await response.json();
        viewer.state.progress.jumpTo(viewer, target.file, target.page, target.scroll);
    }

    jumpTo(viewer, volumeName, pageNumber, scroll) {
        const volume = viewer.volumeByName.get(volumeName);
        if (!volume) throw new Error(`Unknown volume: ${volumeName}`);
        volume.expand(viewer);
        const page = volume.pages?.get(pageNumber);
        if (!page) throw new Error(`Unknown page: ${volumeName}#${pageNumber}`);
        withScrollRestore(viewer, () => {
            this.page = page;
            this.scroll = scroll;
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

    update(viewer, activePage) {
        if (viewer.state.lockDepth > 0) return;
        const scroll = (window.scrollY - activePage.slot.offsetTop) / activePage.slot.offsetHeight;
        this.page = activePage;
        this.scroll = Math.min(1, Math.max(0, scroll));
        if (viewer.saveTimer) clearTimeout(viewer.saveTimer);
        viewer.saveTimer = setTimeout(() => this.save(), 200);
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
