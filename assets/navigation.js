import { scheduleReconcile } from "./reconcile.js";
import { updateZoom, withProgressLock } from "./progress.js";
import { expandVolume } from "./volume.js";

export function handleKeydown(viewer, event) {
    switch (event.key) {
        case "=":
        case "+":
            updateZoom(viewer, 5);
            break;
        case "-":
            updateZoom(viewer, -5);
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
            jumpToVolumePage(viewer, targetIndex, 1);
            break;
        }
        case "l": {
            event.preventDefault();
            const currentIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName).index;
            if (currentIndex < viewer.config.volumes.length - 1) jumpToVolumePage(viewer, currentIndex + 1, 1);
            else jumpToVolumePage(viewer, currentIndex, viewer.config.volumes[currentIndex].pageDims.length, true);
            break;
        }
        case "g":
            event.preventDefault();
            jumpToVolumePage(viewer, 0, 1);
            break;
        case "G": {
            event.preventDefault();
            const lastVolumeIndex = viewer.config.volumes.length - 1;
            jumpToVolumePage(viewer, lastVolumeIndex, viewer.config.volumes[lastVolumeIndex].pageDims.length, true);
            break;
        }
        default:
            break;
    }
}

function jumpToVolumePage(viewer, volumeIndex, pageNumber, scrollToEnd = false) {
    const volume = viewer.config.volumes[volumeIndex];
    if (!viewer.pagesByVolume.has(volume.name)) expandVolume(viewer, volumeIndex);
    withProgressLock(viewer, () => {
        viewer.state.progress = {
            page: viewer.pagesByVolume.get(volume.name).get(pageNumber),
            scroll: scrollToEnd ? 1 : 0,
        };
        scheduleReconcile(viewer);
    });
}
