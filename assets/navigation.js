import { withScrollRestore } from "./progress.js";

export function onKey(viewer, event) {
    switch (event.key) {
        case "=":
        case "+":
            viewer.state.zoom = Math.min(500, viewer.state.zoom + 5);
            withScrollRestore(viewer, () =>
                document.documentElement.style.setProperty("--viewer-zoom", `${viewer.state.zoom}%`),
            );
            break;
        case "-":
            viewer.state.zoom = Math.max(10, viewer.state.zoom - 5);
            withScrollRestore(viewer, () =>
                document.documentElement.style.setProperty("--viewer-zoom", `${viewer.state.zoom}%`),
            );
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
            viewer.state.progress.jumpTo(viewer, viewer.config.volumes[targetIndex].name, 1, 0);
            break;
        }
        case "l": {
            event.preventDefault();
            const currentVolume = viewer.volumeByName.get(viewer.state.progress.page.volumeName);
            const nextIndex = currentVolume.index + 1;
            if (nextIndex < viewer.config.volumes.length)
                viewer.state.progress.jumpTo(viewer, viewer.config.volumes[nextIndex].name, 1, 0);
            else viewer.state.progress.jumpTo(viewer, currentVolume.name, currentVolume.pageDims.length, 1);
            break;
        }
        case "g":
            event.preventDefault();
            viewer.state.progress.jumpTo(viewer, viewer.config.volumes[0].name, 1, 0);
            break;
        case "G": {
            event.preventDefault();
            const lastInfo = viewer.config.volumes[viewer.config.volumes.length - 1];
            viewer.state.progress.jumpTo(viewer, lastInfo.name, lastInfo.pageDims.length, 1);
            break;
        }
        default:
            break;
    }
}
