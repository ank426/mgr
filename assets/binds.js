// @ts-check

/** @typedef {import("./viewer.js").Viewer} Viewer */

/** @param {Viewer} viewer @param {KeyboardEvent} event */
export async function onKey(viewer, event) {
    switch (event.key) {
        case "=":
        case "-": {
            if (event.key === "=") viewer.state.zoom += 5;
            if (event.key === "-") viewer.state.zoom -= 5;
            viewer.state.zoom = Math.max(10, Math.min(500, viewer.state.zoom));
            const centerRatio = (scrollX + innerWidth / 2) / document.documentElement.scrollWidth;
            viewer.withScrollRestore(() => (viewer.pagesRoot.style.width = `${viewer.state.zoom}%`));
            scrollTo({ left: centerRatio * document.documentElement.scrollWidth - innerWidth / 2 });
            break;
        }
        case "j":
            event.preventDefault();
            scrollTo({ top: scrollY + innerHeight / 2 });
            break;
        case "k":
            event.preventDefault();
            scrollTo({ top: scrollY - innerHeight / 2 });
            break;
        case "h": {
            event.preventDefault();
            const currentIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName)?.index;
            if (currentIndex === undefined) break;
            const atVolumeStart = viewer.state.progress.page.pageNumber === 1 && viewer.state.progress.scroll <= 0.001;
            const targetIndex = atVolumeStart ? Math.max(0, currentIndex - 1) : currentIndex;
            await viewer.state.progress.jumpTo(viewer, viewer.volumes[targetIndex].name, 1, 0);
            break;
        }
        case "l": {
            event.preventDefault();
            const currentVolume = viewer.volumeByName.get(viewer.state.progress.page.volumeName);
            if (!currentVolume) break;
            const nextIndex = currentVolume.index + 1;
            if (nextIndex < viewer.volumes.length)
                await viewer.state.progress.jumpTo(viewer, viewer.volumes[nextIndex].name, 1, 0);
            else await viewer.state.progress.jumpTo(viewer, currentVolume.name, currentVolume.pageInfos.length, 1);
            break;
        }
        case "g":
            event.preventDefault();
            await viewer.state.progress.jumpTo(viewer, viewer.volumes[0].name, 1, 0);
            break;
        case "G": {
            event.preventDefault();
            const lastInfo = viewer.volumes[viewer.volumes.length - 1];
            await viewer.state.progress.jumpTo(viewer, lastInfo.name, lastInfo.pageInfos.length, 1);
            break;
        }
        case "i":
            event.preventDefault();
            document.body.style.filter = document.body.style.filter ? "" : "invert(1)";
            break;
        case "s":
        case "d":
        case "S": {
            event.preventDefault();
            const mode = event.key === "s" ? "page" : event.key === "d" ? "scroll" : "volume";
            if (viewer.state.overlayMode === mode) {
                viewer.state.overlayMode = null;
                viewer.progressOverlay.style.display = "";
            } else {
                viewer.state.overlayMode = mode;
                viewer.state.progress.updateOverlay(viewer);
                viewer.progressOverlay.style.display = "grid";
            }
            break;
        }
        default:
            break;
    }
}
