import { pagesByVolume, prefetchBack, prefetchForward, state } from "./globals.js";
import { scheduleReconcile } from "./reconcile.js";
import { updateProgress } from "./progress.js";

export function createObservers() {
    state.observers.page = new IntersectionObserver(handleIntersections, {
        root: null,
        rootMargin: `${prefetchBack * 100}% 0px ${prefetchForward * 100}% 0px`,
        threshold: 0,
    });

    state.observers.firstVisiblePage = new IntersectionObserver(handleVisiblePageIntersections, {
        root: null,
        rootMargin: "0px 0px -99.9% 0px",
        threshold: 0,
    });
}

function handleIntersections(entries) {
    for (const entry of entries) {
        const section = entry.target.closest("section");
        if (!section) continue;
        const page = pagesByVolume.get(section.dataset.volume)?.get(Number(entry.target.dataset.page));
        if (!page) continue;
        if (entry.isIntersecting) state.nearVisiblePages.add(page);
        else state.nearVisiblePages.delete(page);
    }

    scheduleReconcile();
}

function handleVisiblePageIntersections(entries) {
    if (state.progressLocked) return;
    let reconcile = false;
    for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const section = entry.target.closest("section");
        if (!section) continue;
        const page = pagesByVolume.get(section.dataset.volume)?.get(Number(entry.target.dataset.page));
        if (!page) continue;
        if (page !== state.progress.page) {
            updateProgress(page);
            reconcile = true;
        }
    }
    if (reconcile) scheduleReconcile();
}
