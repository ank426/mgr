import { state, volumeByName, volumes } from "./globals.js";
import { withProgressLock } from "./progress.js";
import { collapseVolume, expandVolume } from "./volume.js";
import { loadPage, unloadPage } from "./pages.js";

export function scheduleReconcile() {
    if (state.reconcileScheduled) return;

    state.reconcileScheduled = true;
    requestAnimationFrame(() => {
        state.reconcileScheduled = false;
        withProgressLock(() => {
            reconcileVolumes();
            reconcilePages();
        });
    });
}

export function initializeVolumeWindow(progress) {
    const initialVolumeIndex = volumeByName.get(progress.file).index;
    state.expandedStart = Math.max(0, initialVolumeIndex - 1);
    state.expandedEnd = Math.min(volumes.length - 1, initialVolumeIndex + 1);
    for (let idx = state.expandedStart; idx <= state.expandedEnd; idx++) expandVolume(idx);
}

function reconcileVolumes() {
    if (state.expandedEnd < state.expandedStart) return;

    const activeVolumeIndex = volumeByName.get(state.progress.page.volumeName).index;
    const start = Math.max(0, activeVolumeIndex - 1);
    const end = Math.min(volumes.length - 1, activeVolumeIndex + 1);

    for (let idx = start; idx <= end && idx < state.expandedStart; idx++) expandVolume(idx);
    for (let idx = end; idx >= start && idx > state.expandedEnd; idx--) expandVolume(idx);
    for (let idx = state.expandedStart; idx < start && idx <= state.expandedEnd; idx++) collapseVolume(idx);
    for (let idx = state.expandedEnd; idx > end && idx >= state.expandedStart; idx--) collapseVolume(idx);

    state.expandedStart = start;
    state.expandedEnd = end;
}

function reconcilePages() {
    for (const page of state.nearVisiblePages) {
        if (!state.loadedPages.has(page)) {
            loadPage(page);
            state.loadedPages.add(page);
        }
    }

    for (const page of state.loadedPages) {
        if (!state.nearVisiblePages.has(page)) {
            unloadPage(page);
            state.loadedPages.delete(page);
        }
    }
}
