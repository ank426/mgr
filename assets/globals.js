export function loadConfig() {
    const { volumes, prefetch } = window.MGR_CONFIG;
    const [prefetchBack, prefetchForward] = prefetch;
    return { volumes, prefetchBack, prefetchForward };
}
