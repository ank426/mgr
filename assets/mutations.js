export function withMutation(viewer, action, restore) {
    viewer.state.lockDepth++;
    try {
        action();
    } finally {
        if (--viewer.state.lockDepth === 0) restore();
    }
}

export function withAnchor(viewer, action) {
    withMutation(viewer, action, () => {
        window.scrollTo({
            top:
                viewer.state.progress.page.slot.offsetTop +
                viewer.state.progress.scroll * viewer.state.progress.page.slot.offsetHeight,
        });
    });
}
