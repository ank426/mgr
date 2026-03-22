import { initObservers } from "./observers.js";
import { onKey } from "./navigation.js";
import { Volume } from "./volume.js";
import { scheduleReconcile } from "./reconcile.js";
import { Progress, withScrollRestore } from "./progress.js";

async function init() {
    const viewer = createViewer(window.CONFIG);
    initDom(viewer);
    initObservers(viewer);
    await Progress.fetchAndJump(viewer);
    addEventListeners(viewer);
}

function createViewer(config) {
    return {
        config,
        pagesRoot: document.getElementById("pages"),
        volumeByName: new Map(),
        saveTimer: null,
        state: {
            nearPages: new Set(),
            loadedPages: new Set(),
            progress: new Progress(),
            expandedStart: 0,
            expandedEnd: -1,
            reconcilePending: false,
            lockDepth: 0,
            zoom: 100,
        },
        observers: {
            nearPage: null,
            activePage: null,
        },
    };
}

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

function addEventListeners(viewer) {
    window.addEventListener(
        "scroll",
        () => viewer.state.progress.update(viewer, viewer.state.progress.page),
        { passive: true },
    );
    window.addEventListener("resize", () => withScrollRestore(viewer, () => scheduleReconcile(viewer)));
    window.addEventListener("keydown", (event) => onKey(viewer, event));
}

init().catch((error) => console.error(error));
