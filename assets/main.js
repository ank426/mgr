// @ts-check

import { onKey } from "./navigation.js";
import { initObservers } from "./observers.js";
import { Progress } from "./progress.js";
import { scheduleReconcile } from "./reconcile.js";
import { Viewer } from "./viewer.js";
import { Volume } from "./volume.js";

/** @returns {Promise<void>} */
async function init() {
    // @ts-ignore
    const viewer = new Viewer(window.CONFIG);
    initDom(viewer);
    initObservers(viewer);
    await Progress.fetchAndJump(viewer);
    addEventListeners(viewer);
}

/** @param {Viewer} viewer */
function initDom(viewer) {
    const fragment = document.createDocumentFragment();
    for (const [index, data] of viewer.config.volumes.entries()) {
        const section = document.createElement("section");
        section.dataset.volume = data.name;
        viewer.volumeByName.set(data.name, new Volume(index, data, section));
        fragment.appendChild(section);
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
