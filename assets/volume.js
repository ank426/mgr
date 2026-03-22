// @ts-check
import { Page } from "./page.js";

/** @typedef {import("./viewer.js").Viewer} Viewer */
/** @typedef {{ name: string, pageDims: [number, number][] }} VolumeData */

export class Volume {
    /** @param {number} index @param {VolumeData} data @param {HTMLElement} section */
    constructor(index, data, section) {
        /** @type {number} */
        this.index = index;

        /** @type {string} */
        this.name = data.name;

        /** @type {[number, number][]} */
        this.pageDims = data.pageDims;

        /** @type {HTMLElement} */
        this.section = section;

        /** @type {Map<number, Page> | null} */
        this.pages = null;
    }

    /** @param {Viewer} viewer */
    expand(viewer) {
        if (this.pages) return;
        this.pages = new Map();
        const fragment = document.createDocumentFragment();

        for (let pageNumber = 1; pageNumber <= this.pageDims.length; pageNumber++) {
            const dimensions = this.pageDims[pageNumber - 1];
            const slot = document.createElement("div");

            slot.className = "page-slot";
            slot.dataset.page = String(pageNumber);
            slot.style.aspectRatio = `${dimensions[0]} / ${dimensions[1]}`;

            fragment.appendChild(slot);
            this.pages.set(pageNumber, new Page(this.name, pageNumber, dimensions, slot));
            viewer.observers.nearPage.observe(slot);
            viewer.observers.activePage.observe(slot);
        }

        this.section.replaceChildren(fragment);
    }

    /** @param {Viewer} viewer */
    collapse(viewer) {
        if (!this.pages) return;

        for (const page of this.pages.values()) {
            page.unload();
            viewer.state.loadedPages.delete(page);
            viewer.state.nearPages.delete(page);
            viewer.observers.nearPage.unobserve(page.slot);
            viewer.observers.activePage.unobserve(page.slot);
        }

        this.section.replaceChildren();
        this.pages = null;
    }
}
