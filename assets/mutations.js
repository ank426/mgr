export function withMutation(viewer, action, restore) {
    viewer.state.lockDepth++;
    try {
        action();
    } finally {
        if (--viewer.state.lockDepth === 0 && restore) restore();
    }
}

export function withAnchor(viewer, action) {
    const anchor = viewer.state.progress?.page?.slot;
    const anchorTop = anchor?.getBoundingClientRect().top;

    withMutation(viewer, action, () => {
        if (!anchor || !anchor.isConnected || anchorTop === undefined) {
            if (viewer.state.progress) {
                window.scrollTo({
                    top:
                        viewer.state.progress.page.slot.offsetTop +
                        viewer.state.progress.scroll * viewer.state.progress.page.slot.offsetHeight,
                });
            }
            return;
        }
        const nextTop = anchor.getBoundingClientRect().top;
        window.scrollBy(0, nextTop - anchorTop);
    });
}
