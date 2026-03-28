// @ts-check

import { Progress } from "./progress.js";

/** @typedef {import("./main.js").Config} Config */
/** @typedef {import("./page.js").Page} Page */
/** @typedef {import("./volume.js").Volume} Volume */

export class Viewer {
    /** @param {Config} config */
    constructor(config) {
        const pagesRoot = document.querySelector("main");
        const progressOverlay = document.getElementById("progress-overlay");
        if (!pagesRoot || !progressOverlay) throw new Error();

        /** @type {Config} */
        this.config = config;

        /** @type {Map<string, Volume>} */
        this.volumeByName = new Map();

        /** @type {HTMLElement} */
        this.pagesRoot = pagesRoot;

        /** @type {HTMLElement} */
        this.progressOverlay = progressOverlay;

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

            /** @type {"page" | "scroll" | "volume" | null} */
            overlayMode: null,
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
            if (--this.state.lockDepth === 0 && this.state.progress.page) {
                scrollTo({
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
