import { pagesByVolume, prefetchBack, prefetchForward, state, volumeByName, volumes } from "./globals.js";
import { createObservers } from "./observers.js";
import { expandVolume } from "./volume.js";
import { handleKeydown } from "./navigation.js";
import { scheduleReconcile } from "./reconcile.js";
import { restoreProgress, saveProgress, updateProgress, withProgressLock } from "./progress.js";

async function initializeViewer() {
    buildDom();

    const initialProgressResponse = await fetch("/api/progress");
    if (!initialProgressResponse.ok)
        throw new Error(`Failed to fetch initial progress: ${initialProgressResponse.status}`);
    const initialProgress = await initialProgressResponse.json();

    createObservers(prefetchBack, prefetchForward);

    const initialVolumeIndex = volumeByName.get(initialProgress.file).index;
    state.expandedStart = Math.max(0, initialVolumeIndex - 1);
    state.expandedEnd = Math.min(volumes.length - 1, initialVolumeIndex + 1);
    for (let idx = state.expandedStart; idx <= state.expandedEnd; idx++) expandVolume(idx);

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
