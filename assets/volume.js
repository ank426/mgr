// @ts-check

import { fetchMokuroPages } from "./mokuro.js";
import { Page } from "./page.js";

/** @typedef {import("./viewer.js").Viewer} Viewer */
/** @typedef {import("./main.js").PageInfo} PageInfo */
/** @typedef {import("./main.js").VolumeInfo} VolumeInfo */

export class Volume {
    /** @param {number} index @param {VolumeInfo} data @param {HTMLElement} section */
    constructor(index, data, section) {
        /** @type {number} */
        this.index = index;

        /** @type {string} */
        this.name = data.name;

        /** @type {string | null} */
        this.mokuro = data.mokuro ?? null;

        /** @type {PageInfo[]} */
        this.pageInfos = data.pageInfos;

        /** @type {HTMLElement} */
        this.section = section;

        /** @type {Map<number, Page> | null} */
        this.pages = null;

        /** @type {Promise<void> | null} */
        this._expandPromise = null;

        /** @type {boolean} */
        this._expanded = false;
    }

    /** @param {Viewer} viewer @returns {Promise<void>} */
    expand(viewer) {
        this._expanded = true;
        this._expandPromise ??= this._doExpand(viewer);
        return this._expandPromise;
    }

    /** @param {Viewer} viewer */
    async _doExpand(viewer) {
        const mokuroPagesByName = this.mokuro ? await fetchMokuroPages(this.name) : null;
        if (!this._expanded) {
            this._expandPromise = null;
            return;
        }
        this.pages = new Map();
        const fragment = document.createDocumentFragment();

        for (let pageNumber = 1; pageNumber <= this.pageInfos.length; pageNumber++) {
            const pageInfo = this.pageInfos[pageNumber - 1];
            const slot = document.createElement("section");

            slot.dataset.page = String(pageNumber);
            slot.dataset.path = pageInfo.name;
            slot.style.aspectRatio = `${pageInfo.dims[0]} / ${pageInfo.dims[1]}`;

            fragment.appendChild(slot);
            const mokuroPage = mokuroPagesByName?.get(pageInfo.name) ?? null;
            this.pages.set(pageNumber, new Page(this.name, pageNumber, pageInfo.dims, slot, mokuroPage));
            viewer.observers.nearPage.observe(slot);
            viewer.observers.activePage.observe(slot);
        }

        viewer.withScrollRestore(() => this.section.replaceChildren(fragment));
    }

    /** @param {Viewer} viewer */
    collapse(viewer) {
        this._expanded = false;
        if (!this.pages) return;

        for (const page of this.pages.values()) {
            page.unload();
            viewer.state.loadedPages.delete(page);
            viewer.state.nearPages.delete(page);
            viewer.observers.nearPage.unobserve(page.slot);
            viewer.observers.activePage.unobserve(page.slot);
        }

        viewer.withScrollRestore(() => this.section.replaceChildren());
        this.pages = null;
        this._expandPromise = null;
    }
}
