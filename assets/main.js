import { pagesByVolume, state, volumeByName, volumes } from "./globals.js";
import { createObservers } from "./observers.js";
import { handleKeydown } from "./navigation.js";
import { initializeVolumeWindow, scheduleReconcile } from "./reconcile.js";
import { fetchProgress, restoreProgress, saveProgress, updateProgress, withProgressLock } from "./progress.js";

async function initializeViewer() {
    buildDom();

    const initialProgress = await fetchProgress();

    createObservers();

    initializeVolumeWindow(initialProgress);

    state.progress = {
        page: pagesByVolume.get(initialProgress.file).get(initialProgress.page),
        scroll: initialProgress.scroll,
    };
    restoreProgress(state.progress);

    window.addEventListener("resize", () => withProgressLock(scheduleReconcile));

    window.addEventListener(
        "scroll",
        () => {
            if (state.progressLocked) return;
            updateProgress();
            const timeout = state.timeouts.saveProgress;
            if (timeout) clearTimeout(timeout);
            state.timeouts.saveProgress = setTimeout(saveProgress, 200);
        },
        { passive: true },
    );

    window.addEventListener("keydown", handleKeydown);
}

function buildDom() {
    const fragment = document.createDocumentFragment();
    for (const [index, volume] of volumes.entries()) {
        const section = document.createElement("section");
        section.dataset.volume = volume.name;
        volumeByName.set(volume.name, { index, section });
        fragment.appendChild(section);
    }
    document.getElementById("pages").replaceChildren(fragment);
}

initializeViewer().catch((error) => console.error(error));
