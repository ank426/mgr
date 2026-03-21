import { withProgressLock } from "./progress.js";
import { collapseVolume, expandVolume } from "./volume.js";
import { loadPage, unloadPage } from "./pages.js";

export function scheduleReconcile(viewer) {
    if (viewer.state.reconcileScheduled) return;

    viewer.state.reconcileScheduled = true;
    requestAnimationFrame(() => {
        viewer.state.reconcileScheduled = false;
        withProgressLock(viewer.state, () => {
            reconcileVolumes(viewer);
            reconcilePages(viewer);
        });
    });
}

export function initializeVolumeWindow(viewer, progress) {
    const initialVolumeIndex = viewer.volumeByName.get(progress.file).index;
    viewer.state.expandedStart = Math.max(0, initialVolumeIndex - 1);
    viewer.state.expandedEnd = Math.min(viewer.config.volumes.length - 1, initialVolumeIndex + 1);
    for (let idx = viewer.state.expandedStart; idx <= viewer.state.expandedEnd; idx++) {
        expandVolume(viewer, idx);
    }
}

function reconcileVolumes(viewer) {
    if (viewer.state.expandedEnd < viewer.state.expandedStart) return;

    const activeVolumeIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName).index;
    const start = Math.max(0, activeVolumeIndex - 1);
    const end = Math.min(viewer.config.volumes.length - 1, activeVolumeIndex + 1);

    for (let idx = start; idx <= end && idx < viewer.state.expandedStart; idx++) expandVolume(viewer, idx);
    for (let idx = end; idx >= start && idx > viewer.state.expandedEnd; idx--) expandVolume(viewer, idx);
    for (let idx = viewer.state.expandedStart; idx < start && idx <= viewer.state.expandedEnd; idx++) {
        collapseVolume(viewer, idx);
    }
    for (let idx = viewer.state.expandedEnd; idx > end && idx >= viewer.state.expandedStart; idx--) {
        collapseVolume(viewer, idx);
    }

    viewer.state.expandedStart = start;
    viewer.state.expandedEnd = end;
}

function reconcilePages(viewer) {
    for (const page of viewer.state.nearVisiblePages) {
        if (!viewer.state.loadedPages.has(page)) {
            loadPage(page);
            viewer.state.loadedPages.add(page);
        }
    }

    for (const page of viewer.state.loadedPages) {
        if (!viewer.state.nearVisiblePages.has(page)) {
            unloadPage(page);
            viewer.state.loadedPages.delete(page);
        }
    }
}
