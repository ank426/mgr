import { scheduleReconcile } from "./reconcile.js";

export function initObservers(viewer) {
    viewer.observers.nearPage = new IntersectionObserver((entries) => onNearIntersect(viewer, entries), {
        root: null,
        rootMargin: `${viewer.config.prefetch[0] * 100}% 0px ${viewer.config.prefetch[1] * 100}% 0px`,
        threshold: 0,
    });
    viewer.observers.activePage = new IntersectionObserver((entries) => onActiveIntersect(viewer, entries), {
        root: null,
        rootMargin: "0px 0px -99.9% 0px",
        threshold: 0,
    });
}

function onNearIntersect(viewer, entries) {
    for (const entry of entries) {
        const page = getPageFromEntry(viewer, entry);
        if (!page) continue;
        if (entry.isIntersecting) viewer.state.nearPages.add(page);
        else viewer.state.nearPages.delete(page);
    }
    scheduleReconcile(viewer);
}

function onActiveIntersect(viewer, entries) {
    for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const page = getPageFromEntry(viewer, entry);
        if (!page) continue;
        if (page !== viewer.state.progress.page) {
            viewer.state.progress.update(viewer, page);
            scheduleReconcile(viewer);
        }
    }
}

function getPageFromEntry(viewer, entry) {
    const section = entry.target.closest("section");
    if (!section) return;
    return viewer.volumeByName.get(section.dataset.volume)?.pages?.get(Number(entry.target.dataset.page));
}
