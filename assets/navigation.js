import { scheduleReconcile } from "./reconcile.js";
import { withLock, zoomBy } from "./progress.js";
import { expandVolume } from "./volume.js";

export function onKey(viewer, event) {
    switch (event.key) {
        case "=":
        case "+":
            zoomBy(viewer.state, 5);
            break;
        case "-":
            zoomBy(viewer.state, -5);
            break;
        case "j":
            event.preventDefault();
            window.scrollTo({ top: window.scrollY + window.innerHeight / 2, behavior: "auto" });
            break;
        case "k":
            event.preventDefault();
            window.scrollTo({ top: window.scrollY - window.innerHeight / 2, behavior: "auto" });
            break;
        case "h": {
            event.preventDefault();
            const currentIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName).index;
            const atVolumeStart = viewer.state.progress.page.pageNumber === 1 && viewer.state.progress.scroll <= 0.001;
            const targetIndex = atVolumeStart ? Math.max(0, currentIndex - 1) : currentIndex;
            jumpTo(viewer, targetIndex, 1, false);
            break;
        }
        case "l": {
            event.preventDefault();
            const currentIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName).index;
            if (currentIndex < viewer.config.volumes.length - 1) jumpTo(viewer, currentIndex + 1, 1, false);
            else jumpTo(viewer, currentIndex, viewer.config.volumes[currentIndex].pageDims.length, true);
            break;
        }
        case "g":
            event.preventDefault();
            jumpTo(viewer, 0, 1, false);
            break;
        case "G": {
            event.preventDefault();
            const lastVolumeIndex = viewer.config.volumes.length - 1;
            jumpTo(viewer, lastVolumeIndex, viewer.config.volumes[lastVolumeIndex].pageDims.length, true);
            break;
        }
        default:
            break;
    }
}

function jumpTo(viewer, volumeIndex, pageNumber, scrollToEnd) {
    const volume = viewer.config.volumes[volumeIndex];
    if (!viewer.pagesByVolume.has(volume.name)) expandVolume(viewer, volumeIndex);
    withLock(viewer.state, () => {
        viewer.state.progress = {
            page: viewer.pagesByVolume.get(volume.name).get(pageNumber),
            scroll: scrollToEnd ? 1 : 0,
        };
        scheduleReconcile(viewer);
    });
}
