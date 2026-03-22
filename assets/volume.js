import { Page } from "./page.js";

export class Volume {
    constructor(index, data, section) {
        this.index = index;
        this.name = data.name;
        this.pageDims = data.pageDims;
        this.section = section;
        this.pages = null;
    }

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
            const page = new Page(this.name, pageNumber, dimensions, slot);
            this.pages.set(pageNumber, page);
            viewer.observers.nearPage.observe(slot);
            viewer.observers.activePage.observe(slot);
        }

        this.section.replaceChildren(fragment);
    }

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

export function getVolumeByIndex(viewer, index) {
    const volumeInfo = viewer.config.volumes[index];
    if (!volumeInfo) return;
    return viewer.volumeByName.get(volumeInfo.name);
}
