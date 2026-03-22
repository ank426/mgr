import { Progress, withScrollRestore } from "./progress.js";

export function onKey(viewer, event) {
    switch (event.key) {
        case "=":
        case "+":
            zoomBy(viewer, 5);
            break;
        case "-":
            zoomBy(viewer, -5);
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
            new Progress(viewer, viewer.config.volumes[targetIndex].name, 1, 0).jumpTo(viewer);
            break;
        }
        case "l": {
            event.preventDefault();
            const currentVolume = viewer.volumeByName.get(viewer.state.progress.page.volumeName);
            const nextIndex = currentVolume.index + 1;
            if (nextIndex < viewer.config.volumes.length)
                new Progress(viewer, viewer.config.volumes[nextIndex].name, 1, 0).jumpTo(viewer);
            else new Progress(viewer, currentVolume.name, currentVolume.pageDims.length, 1).jumpTo(viewer);
            break;
        }
        case "g":
            event.preventDefault();
            new Progress(viewer, viewer.config.volumes[0].name, 1, 0).jumpTo(viewer);
            break;
        case "G": {
            event.preventDefault();
            const lastInfo = viewer.config.volumes[viewer.config.volumes.length - 1];
            new Progress(viewer, lastInfo.name, lastInfo.pageDims.length, 1).jumpTo(viewer);
            break;
        }
        default:
            break;
    }
}

function zoomBy(viewer, delta) {
    viewer.state.zoom = Math.min(500, Math.max(10, viewer.state.zoom + delta));
    withScrollRestore(viewer, () =>
        document.documentElement.style.setProperty("--viewer-zoom", `${viewer.state.zoom}%`),
    );
}
