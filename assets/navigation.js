// @ts-check

/** @typedef {import("./viewer.js").Viewer} Viewer */

/** @param {Viewer} viewer @param {KeyboardEvent} event */
export async function onKey(viewer, event) {
    switch (event.key) {
        case "=":
        case "+":
            viewer.state.zoom = Math.min(500, viewer.state.zoom + 5);
            viewer.withScrollRestore(() =>
                document.documentElement.style.setProperty("--viewer-zoom", `${viewer.state.zoom}%`),
            );
            break;
        case "-":
            viewer.state.zoom = Math.max(10, viewer.state.zoom - 5);
            viewer.withScrollRestore(() =>
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
            const currentIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName)?.index;
            if (currentIndex === undefined) break;
            const atVolumeStart = viewer.state.progress.page.pageNumber === 1 && viewer.state.progress.scroll <= 0.001;
            const targetIndex = atVolumeStart ? Math.max(0, currentIndex - 1) : currentIndex;
            await viewer.state.progress.jumpTo(viewer, viewer.config.volumes[targetIndex].name, 1, 0);
            break;
        }
        case "l": {
            event.preventDefault();
            const currentVolume = viewer.volumeByName.get(viewer.state.progress.page.volumeName);
            if (!currentVolume) break;
            const nextIndex = currentVolume.index + 1;
            if (nextIndex < viewer.config.volumes.length)
                await viewer.state.progress.jumpTo(viewer, viewer.config.volumes[nextIndex].name, 1, 0);
            else await viewer.state.progress.jumpTo(viewer, currentVolume.name, currentVolume.pageInfos.length, 1);
            break;
        }
        case "g":
            event.preventDefault();
            await viewer.state.progress.jumpTo(viewer, viewer.config.volumes[0].name, 1, 0);
            break;
        case "G": {
            event.preventDefault();
            const lastInfo = viewer.config.volumes[viewer.config.volumes.length - 1];
            await viewer.state.progress.jumpTo(viewer, lastInfo.name, lastInfo.pageInfos.length, 1);
            break;
        }
        default:
            break;
    }
}
