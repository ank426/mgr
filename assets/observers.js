import { scheduleReconcile } from "./reconcile.js";
import { updateProgress } from "./progress.js";

export function initObservers(viewer) {
    viewer.observers.page = new IntersectionObserver((entries) => handleIntersections(viewer, entries), {
        root: null,
        rootMargin: `${viewer.config.prefetch[0] * 100}% 0px ${viewer.config.prefetch[1] * 100}% 0px`,
        threshold: 0,
    });

    viewer.observers.firstVisiblePage = new IntersectionObserver(
        (entries) => handleVisiblePageIntersections(viewer, entries),
        {
            root: null,
            rootMargin: "0px 0px -99.9% 0px",
            threshold: 0,
        },
    );
}

function handleIntersections(viewer, entries) {
    for (const entry of entries) {
        const section = entry.target.closest("section");
        if (!section) continue;
        const page = viewer.pagesByVolume.get(section.dataset.volume)?.get(Number(entry.target.dataset.page));
        if (!page) continue;
        if (entry.isIntersecting) viewer.state.nearVisiblePages.add(page);
        else viewer.state.nearVisiblePages.delete(page);
    }

    scheduleReconcile(viewer);
}

function handleVisiblePageIntersections(viewer, entries) {
    if (viewer.state.progressLocked) return;
    let reconcile = false;
    for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const section = entry.target.closest("section");
        if (!section) continue;
        const page = viewer.pagesByVolume.get(section.dataset.volume)?.get(Number(entry.target.dataset.page));
        if (!page) continue;
        if (page !== viewer.state.progress.page) {
            updateProgress(viewer, page);
            reconcile = true;
        }
    }
    if (reconcile) scheduleReconcile(viewer);
}
