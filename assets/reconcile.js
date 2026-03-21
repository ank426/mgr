import { withAnchor, withMutation } from "./mutations.js";
import { collapseVolume, expandVolume } from "./volume.js";
import { loadPage, unloadPage } from "./pages.js";

export function scheduleReconcile(viewer) {
    if (viewer.state.reconcilePending) return;

    viewer.state.reconcilePending = true;
    requestAnimationFrame(() => {
        viewer.state.reconcilePending = false;
        withAnchor(viewer, () => {
            reconcileVolumes(viewer);
            reconcilePages(viewer);
        });
    });
}

export function jumpToProgress(viewer, progress) {
    const volumeIndex = viewer.volumeByName.get(progress.file).index;
    const volume = viewer.config.volumes[volumeIndex];
    expandVolume(viewer, volumeIndex);

    withMutation(
        viewer,
        () => {
            viewer.state.progress = {
                page: viewer.pagesByVolume.get(volume.name).get(progress.page),
                scroll: progress.scroll,
            };
            window.scrollTo({ top: progress.page.slot.offsetTop + progress.scroll * progress.page.slot.offsetHeight });
            scheduleReconcile(viewer);
        },
        () => {},
    );
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
    for (const page of viewer.state.nearPages) {
        if (!viewer.state.loadedPages.has(page)) {
            loadPage(page);
            viewer.state.loadedPages.add(page);
        }
    }

    for (const page of viewer.state.loadedPages) {
        if (!viewer.state.nearPages.has(page)) {
            unloadPage(page);
            viewer.state.loadedPages.delete(page);
        }
    }
}
