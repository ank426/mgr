import { initObservers } from "./observers.js";
import { onKey } from "./navigation.js";
import { jumpToProgress, scheduleReconcile } from "./reconcile.js";
import { fetchProgress, saveProgress, updateProgress, withLock } from "./progress.js";

async function init() {
    const viewer = createViewer(window.CONFIG);
    initDom(viewer);
    initObservers(viewer);
    jumpToProgress(viewer, await fetchProgress());
    addEventListeners(viewer);
}

function createViewer(config) {
    return {
        config,
        elements: { pages: document.getElementById("pages") },
        volumeByName: new Map(),
        pagesByVolume: new Map(),
        state: {
            nearPages: new Set(),
            loadedPages: new Set(),
            progress: null,
            expandedStart: 0,
            expandedEnd: -1,
            reconcilePending: false,
            locked: false,
            zoom: 100,
        },
        observers: {
            page: null,
            activePage: null,
        },
        timeouts: {
            save: null,
        },
    };
}

function initDom(viewer) {
    const fragment = document.createDocumentFragment();
    for (const [index, volume] of viewer.config.volumes.entries()) {
        const section = document.createElement("section");
        section.dataset.volume = volume.name;
        viewer.volumeByName.set(volume.name, { index, section });
        fragment.appendChild(section);
    }
    viewer.elements.pages.replaceChildren(fragment);
}

function addEventListeners(viewer) {
    window.addEventListener("resize", () => withLock(viewer.state, () => scheduleReconcile(viewer)));

    window.addEventListener(
        "scroll",
        () => {
            if (viewer.state.locked) return;
            updateProgress(viewer.state, viewer.state.progress.page);
            const timeout = viewer.timeouts.save;
            if (timeout) clearTimeout(timeout);
            viewer.timeouts.save = setTimeout(() => saveProgress(viewer.state.progress), 200);
        },
        { passive: true },
    );

    window.addEventListener("keydown", (event) => onKey(viewer, event));
}

init().catch((error) => console.error(error));
