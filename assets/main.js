import { createViewer } from "./viewer.js";
import { initObservers } from "./observers.js";
import { onKey } from "./navigation.js";
import { initVolumes, scheduleReconcile } from "./reconcile.js";
import { fetchProgress, restoreScroll, saveProgress, updateProgress, withLock } from "./progress.js";

async function init() {
    const viewer = createViewer(window.MGR_CONFIG);
    initDom(viewer);
    initObservers(viewer);

    const initialProgress = await fetchProgress();
    initVolumes(viewer, initialProgress);

    viewer.state.progress = {
        page: viewer.pagesByVolume.get(initialProgress.file).get(initialProgress.page),
        scroll: initialProgress.scroll,
    };
    restoreScroll(viewer.state.progress);

    window.addEventListener("resize", () => withLock(viewer.state, () => scheduleReconcile(viewer)));

    window.addEventListener(
        "scroll",
        () => {
            if (viewer.state.locked) return;
            updateProgress(viewer.state);
            const timeout = viewer.timeouts.save;
            if (timeout) clearTimeout(timeout);
            viewer.timeouts.save = setTimeout(() => saveProgress(viewer.state.progress), 200);
        },
        { passive: true },
    );

    window.addEventListener("keydown", (event) => onKey(viewer, event));
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

init().catch((error) => console.error(error));
