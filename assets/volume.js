import { pagesByVolume, state, volumeByName, volumes } from "./globals.js";
import { unloadPage } from "./pages.js";

export function expandVolume(index) {
    const volume = volumes[index];
    if (pagesByVolume.has(volume.name)) return;
    const section = volumeByName.get(volume.name).section;
    const volumePages = new Map();
    const fragment = document.createDocumentFragment();

    for (let pageNumber = 1; pageNumber <= volume.pageDims.length; pageNumber++) {
        const dimensions = volume.pageDims[pageNumber - 1];
        const slot = document.createElement("div");

        slot.className = "page-slot";
        slot.dataset.page = String(pageNumber);
        slot.style.aspectRatio = `${dimensions[0]} / ${dimensions[1]}`;

        fragment.appendChild(slot);
        const page = {
            slot,
            volumeName: volume.name,
            pageNumber,
            dimensions,
            url: `/volume/${encodeURIComponent(volume.name)}/page/${pageNumber}`,
            status: "unloaded",
            image: null,
        };
        volumePages.set(pageNumber, page);
        state.observers.page.observe(slot);
        state.observers.firstVisiblePage.observe(slot);
    }

    section.replaceChildren(fragment);
    pagesByVolume.set(volume.name, volumePages);
}

export function collapseVolume(index) {
    const volumeName = volumes[index].name;
    for (const page of pagesByVolume.get(volumeName).values()) {
        unloadPage(page);
        state.loadedPages.delete(page);
        state.nearVisiblePages.delete(page);
        state.observers.page.unobserve(page.slot);
        state.observers.firstVisiblePage.unobserve(page.slot);
    }
    volumeByName.get(volumeName).section.replaceChildren();
    pagesByVolume.delete(volumeName);
}
