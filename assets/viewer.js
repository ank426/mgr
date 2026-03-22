// @ts-check

import { Volume } from "./volume.js";
import { scheduleReconcile } from "./reconcile.js";
import { Progress, withScrollRestore } from "./progress.js";
import { onKey } from "./navigation.js";

/** @typedef {import("./page.js").Page} Page */
/** @typedef {import("./page.js").PageDimensions} PageDimensions */
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

    /** @returns {void} */
    initDom() {
        const fragment = document.createDocumentFragment();
        for (const [index, data] of this.config.volumes.entries()) {
            const section = document.createElement("section");
            section.dataset.volume = data.name;
            this.volumeByName.set(data.name, new Volume(index, data, section));
            fragment.appendChild(section);
        }
        this.pagesRoot.replaceChildren(fragment);
    }

    /** @returns {void} */
    bindEvents() {
        window.addEventListener("scroll", () => this.state.progress.update(this, this.state.progress.page), {
            passive: true,
        });
        window.addEventListener("resize", () => withScrollRestore(this, () => scheduleReconcile(this)));
        window.addEventListener("keydown", (event) => onKey(this, event));
    }
}
