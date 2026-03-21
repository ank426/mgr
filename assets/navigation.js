import { jumpToProgress } from "./reconcile.js";
import { zoomBy } from "./progress.js";

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
            jumpToProgress(viewer, { file: viewer.config.volumes[targetIndex].name, page: 1, scroll: 0 });
            break;
        }
        case "l": {
            event.preventDefault();
            const currentIndex = viewer.volumeByName.get(viewer.state.progress.page.volumeName).index;
            if (currentIndex < viewer.config.volumes.length - 1) {
                jumpToProgress(viewer, { file: viewer.config.volumes[currentIndex + 1].name, page: 1, scroll: 0 });
            } else {
                jumpToProgress(viewer, {
                    file: viewer.config.volumes[currentIndex].name,
                    page: viewer.config.volumes[currentIndex].pageDims.length,
                    scroll: 1,
                });
            }
            break;
        }
        case "g":
            event.preventDefault();
            jumpToProgress(viewer, { file: viewer.config.volumes[0].name, page: 1, scroll: 0 });
            break;
        case "G": {
            event.preventDefault();
            const lastVolumeIndex = viewer.config.volumes.length - 1;
            jumpToProgress(viewer, {
                file: viewer.config.volumes[lastVolumeIndex].name,
                page: viewer.config.volumes[lastVolumeIndex].pageDims.length,
                scroll: 1,
            });
            break;
        }
        default:
            break;
    }
}
