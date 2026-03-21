export const volumes = window.MGR_CONFIG.volumes;
export const [prefetchBack, prefetchForward] = window.MGR_CONFIG.prefetch;

export const volumeByName = new Map();
export const pagesByVolume = new Map();

export const state = {
    nearVisiblePages: new Set(),
    loadedPages: new Set(),
    progress: null,
    expandedStart: 0,
    expandedEnd: -1,
    reconcileScheduled: false,
    progressLocked: false,
    zoom: 100,
    observers: {
        page: null,
        firstVisiblePage: null,
    },
    timeouts: {
        saveProgress: null,
    },
};
