// @ts-check

import { onKey } from "./navigation.js";
import { initObservers } from "./observers.js";
import { Progress } from "./progress.js";
import { scheduleReconcile } from "./reconcile.js";
import { Viewer } from "./viewer.js";
import { Volume } from "./volume.js";

/** @typedef {{ name: string, dims: [number, number] }} PageInfo */
/** @typedef {{ name: string, mokuro?: string | null, pageInfos: PageInfo[] }} VolumeInfo */
/** @typedef {{ volumes: VolumeInfo[], prefetch: [number, number] }} Config */

/** @returns {Promise<void>} */
async function init() {
    const viewer = new Viewer(getConfig());
    initDom(viewer);
    initObservers(viewer);
    await Progress.fetchAndJump(viewer);
    addEventListeners(viewer);
}

/** @returns {Config} */
function getConfig() {
    const configText = document.getElementById("config")?.textContent;
    if (!configText) throw new Error();
    return JSON.parse(configText);
}

/** @param {Viewer} viewer */
function initDom(viewer) {
    const fragment = document.createDocumentFragment();
    for (const [index, data] of viewer.config.volumes.entries()) {
        const article = document.createElement("article");
        article.dataset.volume = data.name;
        viewer.volumeByName.set(data.name, new Volume(index, data, article));
        fragment.appendChild(article);
    }
    viewer.pagesRoot.replaceChildren(fragment);
}

/** @param {Viewer} viewer */
function addEventListeners(viewer) {
    window.addEventListener("scroll", () => viewer.state.progress.update(viewer), { passive: true });
    window.addEventListener("resize", () => viewer.withScrollRestore(() => scheduleReconcile(viewer)));
    window.addEventListener("keydown", (event) => onKey(viewer, event));
}

init().catch((error) => console.error(error));
