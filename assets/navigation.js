import { pagesByVolume, state, volumeByName, volumes } from "./globals.js";
import { scheduleReconcile } from "./reconcile.js";
import { withProgressLock } from "./progress.js";
import { expandVolume } from "./volume.js";

export function handleKeydown(event) {
    switch (event.key) {
        case "=":
        case "+":
            updateZoom(5);
            break;
        case "-":
            updateZoom(-5);
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
            const currentIndex = volumeByName.get(state.progress.page.volumeName).index;
            const atVolumeStart = state.progress.page.pageNumber === 1 && state.progress.scroll <= 0.001;
            const targetIndex = atVolumeStart ? Math.max(0, currentIndex - 1) : currentIndex;
            jumpToVolumePage(targetIndex, 1);
            break;
        }
        case "l": {
            event.preventDefault();
            const currentIndex = volumeByName.get(state.progress.page.volumeName).index;
            if (currentIndex < volumes.length - 1) jumpToVolumePage(currentIndex + 1, 1);
            else jumpToVolumePage(currentIndex, volumes[currentIndex].pageDims.length, true);
            break;
        }
        case "g":
            event.preventDefault();
            jumpToVolumePage(0, 1);
            break;
        case "G": {
            event.preventDefault();
            const lastVolumeIndex = volumes.length - 1;
            jumpToVolumePage(lastVolumeIndex, volumes[lastVolumeIndex].pageDims.length, true);
            break;
        }
        default:
            break;
    }
}

function jumpToVolumePage(volumeIndex, pageNumber, scrollToEnd = false) {
    const volume = volumes[volumeIndex];
    if (!pagesByVolume.has(volume.name)) expandVolume(volumeIndex);
    withProgressLock(() => {
        state.progress = {
            page: pagesByVolume.get(volume.name).get(pageNumber),
            scroll: scrollToEnd ? 1 : 0,
        };
        scheduleReconcile();
    });
}

function updateZoom(delta) {
    state.zoom = Math.min(500, Math.max(10, state.zoom + delta));
    withProgressLock(() => document.documentElement.style.setProperty("--viewer-zoom", `${state.zoom}%`));
}
