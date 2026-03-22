// @ts-check

import { scheduleReconcile } from "./reconcile.js";

/** @typedef {import("./viewer.js").Viewer} Viewer */
/** @typedef {import("./page.js").Page} Page */

export class Progress {
    constructor() {
        /** @type {Page} */
        this.page = /** @type {any} */ (null);

        /** @type {number} */
        this.scroll = 0;
    }

    /** @param {Viewer} viewer @returns {Promise<void>} */
    static async fetchAndJump(viewer) {
        const response = await fetch("/api/progress");
        if (!response.ok) throw new Error(`Failed to fetch initial progress: ${response.status}`);
        const target = await response.json();
        viewer.state.progress.jumpTo(viewer, target.file, target.page, target.scroll);
    }

    /** @param {Viewer} viewer @param {string} volumeName @param {number} pageNumber @param {number} scroll */
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

    /** @returns {void} */
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

    /** @param {Viewer} viewer @param {Page} activePage */
    update(viewer, activePage) {
        if (viewer.state.lockDepth > 0) return;
        const scroll = (window.scrollY - activePage.slot.offsetTop) / activePage.slot.offsetHeight;
        this.page = activePage;
        this.scroll = Math.min(1, Math.max(0, scroll));
        if (viewer.saveTimer) clearTimeout(viewer.saveTimer);
        viewer.saveTimer = setTimeout(() => this.save(), 200);
    }
}

/** @param {Viewer} viewer @param {() => void} action */
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
