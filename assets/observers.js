// @ts-check

import { scheduleReconcile } from "./reconcile.js";

/** @typedef {import("./page.js").Page} Page */
/** @typedef {import("./viewer.js").Viewer} Viewer */

/** @param {Viewer} viewer */
export function initObservers(viewer) {
    viewer.observers.nearPage = new IntersectionObserver((entries) => onNearIntersect(viewer, entries), {
        root: null,
        rootMargin: `${viewer.config.prefetchBack * 100}% 0px ${viewer.config.prefetchForward * 100}% 0px`,
        threshold: 0,
    });
    viewer.observers.activePage = new IntersectionObserver((entries) => onActiveIntersect(viewer, entries), {
        root: null,
        rootMargin: "0px 0px -99.9% 0px",
        threshold: 0,
    });
}

/** @param {Viewer} viewer @param {IntersectionObserverEntry[]} entries */
function onNearIntersect(viewer, entries) {
    for (const entry of entries) {
        const page = getPageFromEntry(viewer, entry);
        if (!page) continue;
        if (entry.isIntersecting) viewer.state.nearPages.add(page);
        else viewer.state.nearPages.delete(page);
    }
    scheduleReconcile(viewer);
}

/** @param {Viewer} viewer @param {IntersectionObserverEntry[]} entries */
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

/** @param {Viewer} viewer @param {IntersectionObserverEntry} entry @returns {Page | undefined} */
function getPageFromEntry(viewer, entry) {
    const target = /** @type {HTMLElement} */ (entry.target);
    const volumeName = target.closest("article")?.dataset.volume;
    if (!volumeName) return;
    return viewer.volumeByName.get(volumeName)?.pages?.get(Number(target.dataset.page));
}
