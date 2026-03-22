// @ts-check

import { initObservers } from "./observers.js";
import { Progress } from "./progress.js";
import { Viewer } from "./viewer.js";

/** @returns {Promise<void>} */
async function init() {
    // @ts-ignore
    const viewer = new Viewer(window.CONFIG);
    viewer.initDom();
    initObservers(viewer);
    await Progress.fetchAndJump(viewer);
    viewer.bindEvents();
}

init().catch((error) => console.error(error));
