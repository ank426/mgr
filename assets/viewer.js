export function createViewer(config) {
    const pages = document.getElementById("pages");
    if (!pages) throw new Error("Missing pages container");

    return {
        config,
        elements: { pages },
        volumeByName: new Map(),
        pagesByVolume: new Map(),
        state: {
            nearPages: new Set(),
            loadedPages: new Set(),
            progress: null,
            expandedStart: 0,
            expandedEnd: -1,
            reconcilePending: false,
            locked: false,
            zoom: 100,
        },
        observers: {
            page: null,
            activePage: null,
        },
        timeouts: {
            save: null,
        },
    };
}
