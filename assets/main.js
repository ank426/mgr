import { createViewer } from "./viewer.js";
import { initObservers } from "./observers.js";
import { handleKeydown } from "./navigation.js";
import { initializeVolumeWindow, scheduleReconcile } from "./reconcile.js";
import { fetchProgress, restoreProgress, saveProgress, updateProgress, withProgressLock } from "./progress.js";

async function initializeViewer() {
    const viewer = createViewer(window.MGR_CONFIG);
    buildDom(viewer);
    initObservers(viewer);

    const initialProgress = await fetchProgress();
    initializeVolumeWindow(viewer, initialProgress);

    viewer.state.progress = {
        page: viewer.pagesByVolume.get(initialProgress.file).get(initialProgress.page),
        scroll: initialProgress.scroll,
    };
    restoreProgress(viewer.state.progress);

    window.addEventListener("resize", () => withProgressLock(viewer.state, () => scheduleReconcile(viewer)));

    window.addEventListener(
        "scroll",
        () => {
            if (viewer.state.progressLocked) return;
            updateProgress(viewer.state);
            const timeout = viewer.timeouts.saveProgress;
            if (timeout) clearTimeout(timeout);
            viewer.timeouts.saveProgress = setTimeout(() => saveProgress(viewer.state.progress), 200);
        },
        { passive: true },
    );

    window.addEventListener("keydown", (event) => handleKeydown(viewer, event));
}

function buildDom(viewer) {
    const fragment = document.createDocumentFragment();
    for (const [index, volume] of viewer.config.volumes.entries()) {
        const section = document.createElement("section");
        section.dataset.volume = volume.name;
        viewer.volumeByName.set(volume.name, { index, section });
        fragment.appendChild(section);
    }
    viewer.elements.pages.replaceChildren(fragment);
}

initializeViewer().catch((error) => console.error(error));
