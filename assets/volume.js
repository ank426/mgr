import { observeSlot, unobserveSlot } from "./observers.js";
import { unloadPage } from "./pages.js";

export function expandVolume(viewer, index) {
    const volume = viewer.config.volumes[index];
    if (viewer.pagesByVolume.has(volume.name)) return;
    const section = viewer.volumeByName.get(volume.name).section;
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
        observeSlot(viewer, slot);
    }

    section.replaceChildren(fragment);
    viewer.pagesByVolume.set(volume.name, volumePages);
}

export function collapseVolume(viewer, index) {
    const volumeName = viewer.config.volumes[index].name;
    for (const page of viewer.pagesByVolume.get(volumeName).values()) {
        unloadPage(page);
        viewer.state.loadedPages.delete(page);
        viewer.state.nearVisiblePages.delete(page);
        unobserveSlot(viewer, page.slot);
    }
    viewer.volumeByName.get(volumeName).section.replaceChildren();
    viewer.pagesByVolume.delete(volumeName);
}
