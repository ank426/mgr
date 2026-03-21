export function createViewer(config) {
    const pages = document.getElementById("pages");
    if (!pages) throw new Error("Missing pages container");

    return {
        config,
        elements: { pages },
        volumeByName: new Map(),
        pagesByVolume: new Map(),
        state: {
            nearVisiblePages: new Set(),
            loadedPages: new Set(),
            progress: null,
            expandedStart: 0,
            expandedEnd: -1,
            reconcileScheduled: false,
            progressLocked: false,
            zoom: 100,
        },
        observers: {
            page: null,
            firstVisiblePage: null,
        },
        timeouts: {
            saveProgress: null,
        },
    };
}
