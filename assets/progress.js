// @ts-check

import { scheduleReconcile } from "./reconcile.js";

/** @typedef {import("./page.js").Page} Page */
/** @typedef {import("./viewer.js").Viewer} Viewer */

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
        await viewer.state.progress.jumpTo(viewer, target.file, target.page, target.scroll);
    }

    /** @param {Viewer} viewer @param {string} volumeName @param {number} pageNumber @param {number} scroll */
    async jumpTo(viewer, volumeName, pageNumber, scroll) {
        const volume = viewer.volumeByName.get(volumeName);
        if (!volume) throw new Error(`Unknown volume: ${volumeName}`);
        await volume.expand(viewer);
        const page = volume.pages?.get(pageNumber);
        if (!page) throw new Error(`Unknown page: ${volumeName}#${pageNumber}`);
        viewer.withScrollRestore(() => {
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

    /** @param {Viewer} viewer @param {Page} [activePage] */
    update(viewer, activePage) {
        if (viewer.state.lockDepth > 0) return;
        this.page = activePage ?? this.page;
        const scroll = (scrollY - this.page.slot.offsetTop) / this.page.slot.offsetHeight;
        this.scroll = Math.min(1, Math.max(0, scroll));
        clearTimeout(viewer.timeouts.save);
        viewer.timeouts.save = window.setTimeout(() => this.save(), viewer.config.saveDebounce);
        this.updateOverlay(viewer);
    }

    /** @param {Viewer} viewer */
    updateOverlay(viewer) {
        if (!viewer.state.overlayMode) return;
        const vol = viewer.volumeByName.get(this.page.volumeName);
        if (!vol) return;
        let text = "";
        switch (viewer.state.overlayMode) {
            case "page":
                text = `${this.page.pageNumber} / ${vol.pageInfos.length}`;
                break;
            case "scroll": {
                let scrolled = 0;
                for (let i = 0; i < this.page.pageNumber - 1; i++)
                    scrolled += vol.pageInfos[i].dims[1] / vol.pageInfos[i].dims[0];
                let total = scrolled;
                scrolled += this.scroll * (this.page.dimensions[1] / this.page.dimensions[0]);
                for (let i = this.page.pageNumber - 1; i < vol.pageInfos.length; i++)
                    total += vol.pageInfos[i].dims[1] / vol.pageInfos[i].dims[0];
                text = `${Math.round((scrolled / total) * 100)}%`;
                break;
            }
            case "volume":
                text = this.page.volumeName;
                break;
        }
        viewer.progressOverlay.children[0].textContent = text;
    }
}
