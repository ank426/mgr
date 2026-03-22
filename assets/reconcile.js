export function scheduleReconcile(viewer) {
    if (viewer.state.reconcilePending) return;
    viewer.state.reconcilePending = true;
    requestAnimationFrame(() => {
        viewer.state.reconcilePending = false;
        reconcileVolumes(viewer);
        reconcilePages(viewer);
    });
}

function reconcileVolumes(viewer) {
    const activeVolumeIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName).index;
    const start = Math.max(0, activeVolumeIndex - 1);
    const end = Math.min(viewer.config.volumes.length - 1, activeVolumeIndex + 1);

    for (let idx = start; idx <= end && idx < viewer.state.expandedStart; idx++)
        getVolumeByIndex(viewer, idx)?.expand(viewer);
    for (let idx = end; idx >= start && idx > viewer.state.expandedEnd; idx--)
        getVolumeByIndex(viewer, idx)?.expand(viewer);
    for (let idx = viewer.state.expandedStart; idx < start && idx <= viewer.state.expandedEnd; idx++)
        getVolumeByIndex(viewer, idx)?.collapse(viewer);
    for (let idx = viewer.state.expandedEnd; idx > end && idx >= viewer.state.expandedStart; idx--)
        getVolumeByIndex(viewer, idx)?.collapse(viewer);

    viewer.state.expandedStart = start;
    viewer.state.expandedEnd = end;
}

function reconcilePages(viewer) {
    for (const page of viewer.state.nearPages)
        if (!viewer.state.loadedPages.has(page)) {
            page.load();
            viewer.state.loadedPages.add(page);
        }

    for (const page of viewer.state.loadedPages)
        if (!viewer.state.nearPages.has(page)) {
            page.unload();
            viewer.state.loadedPages.delete(page);
        }
}

function getVolumeByIndex(viewer, index) {
    const volumeInfo = viewer.config.volumes[index];
    if (!volumeInfo) return;
    return viewer.volumeByName.get(volumeInfo.name);
}
