// @ts-check

import { onKey } from "./binds.js";
import { initObservers } from "./observers.js";
import { Progress } from "./progress.js";
import { scheduleReconcile } from "./reconcile.js";
import { Viewer } from "./viewer.js";
import { Volume } from "./volume.js";

/** @typedef {{ name: string, dims: [number, number] }} PageInfo */
/** @typedef {{ name: string, hasMokuro: boolean, pageInfos: PageInfo[] }} VolumeInfo */
/** @typedef {{ prefetchBack: number, prefetchForward: number, cursorTimeout: number, saveDebounce: number, zoom: number, zoomMin: number, zoomMax: number }} Config */

/** @returns {Promise<void>} */
async function init() {
    const [volumes, config] = await Promise.all([
        fetch("/api/volumes").then((r) => r.json()),
        fetch("/api/config").then((r) => r.json()),
    ]);
    const viewer = new Viewer(volumes, config);
    initDom(viewer);
    initObservers(viewer);
    await Progress.fetchAndJump(viewer);
    addEventListeners(viewer);
}

/** @param {Viewer} viewer */
function initDom(viewer) {
    const fragment = document.createDocumentFragment();
    for (const [index, data] of viewer.volumes.entries()) {
        const article = document.createElement("article");
        article.dataset.volume = data.name;
        viewer.volumeByName.set(data.name, new Volume(index, data, article));
        fragment.appendChild(article);
    }
    viewer.pagesRoot.replaceChildren(fragment);
}

/** @param {Viewer} viewer */
function addEventListeners(viewer) {
    addEventListener("scroll", () => viewer.state.progress.update(viewer), { passive: true });
    addEventListener("resize", () => viewer.withScrollRestore(() => scheduleReconcile(viewer)));
    addEventListener("keydown", (event) => onKey(viewer, event));
    addEventListener("mousemove", () => {
        document.body.style.cursor = "auto";
        clearTimeout(viewer.timeouts.cursor);
        viewer.timeouts.cursor = window.setTimeout(
            () => (document.body.style.cursor = "none"),
            viewer.config.cursorTimeout,
        );
    });
}

init().catch((error) => console.error(error));
