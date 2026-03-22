// @ts-check

import { Progress } from "./progress.js";

/** @typedef {import("./page.js").Page} Page */
/** @typedef {import("./page.js").PageDimensions} PageDimensions */
/** @typedef {import("./volume.js").Volume} Volume */
/** @typedef {{ name: string, pageDims: PageDimensions[] }} VolumeInfo */
/** @typedef {{ volumes: VolumeInfo[], prefetch: [number, number] }} ViewerConfig */

export class Viewer {
    /** @param {ViewerConfig} config */
    constructor(config) {
        const pagesRoot = document.getElementById("pages");
        if (!pagesRoot) throw new Error("Element #pages not found");

        /** @type {ViewerConfig} */
        this.config = config;

        /** @type {HTMLElement} */
        this.pagesRoot = pagesRoot;

        /** @type {Map<string, Volume>} */
        this.volumeByName = new Map();

        /** @type {ReturnType<typeof setTimeout> | null} */
        this.saveTimer = null;

        this.state = {
            /** @type {Set<Page>} */
            nearPages: new Set(),

            /** @type {Set<Page>} */
            loadedPages: new Set(),

            /** @type {Progress} */
            progress: new Progress(),

            /** @type {number} */
            expandedStart: 0,

            /** @type {number} */
            expandedEnd: -1,

            /** @type {boolean} */
            reconcilePending: false,

            /** @type {number} */
            lockDepth: 0,

            /** @type {number} */
            zoom: 100,
        };

        this.observers = {
            /** @type {IntersectionObserver} */
            nearPage: /** @type {any} */ (null),

            /** @type {IntersectionObserver} */
            activePage: /** @type {any} */ (null),
        };
    }

    /** @param {() => void} action */
    withScrollRestore(action) {
        this.state.lockDepth++;
        try {
            action();
        } finally {
            if (--this.state.lockDepth === 0) {
                window.scrollTo({
                    top:
                        this.state.progress.page.slot.offsetTop +
                        this.state.progress.scroll * this.state.progress.page.slot.offsetHeight,
                });
            }
        }
    }

    /** @param {number} index @returns {Volume | undefined} */
    getVolumeByIndex(index) {
        const volumeInfo = this.config.volumes[index];
        if (!volumeInfo) return;
        return this.volumeByName.get(volumeInfo.name);
    }
}
