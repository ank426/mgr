const config = window.MGR_CONFIG;
const pagesRoot = document.getElementById("pages");
const topSpacer = document.getElementById("top-spacer");
const bottomSpacer = document.getElementById("bottom-spacer");

const state = {
  pageCount: config.pageCount,
  prefetchBack: config.prefetchBack,
  prefetchForward: config.prefetchForward,
  firstLoadedIndex: 0,
  lastLoadedIndex: -1,
  pageElements: new Map(),
  inflightLoads: new Map(),
  updateScheduled: false,
  updateRunning: false,
  trimmedTopHeight: 0,
  topSafetyHeight: 0,
  spacersInitialized: false,
};

async function init() {
  if (state.pageCount === 0) {
    return;
  }

  const initialLastIndex = Math.min(
    state.pageCount - 1,
    state.prefetchForward,
  );

  for (let index = 0; index <= initialLastIndex; index += 1) {
    await appendPage(index);
  }

  syncSpacers();
  scheduleUpdate();
}

function scheduleUpdate() {
  if (state.updateScheduled) {
    return;
  }

  state.updateScheduled = true;
  requestAnimationFrame(runUpdate);
}

async function runUpdate() {
  state.updateScheduled = false;
  if (state.updateRunning) {
    scheduleUpdate();
    return;
  }

  state.updateRunning = true;
  try {
    await updateWindow();
  } finally {
    state.updateRunning = false;
  }
}

async function updateWindow() {
  const anchorIndex = findAnchorPageIndex();
  if (anchorIndex === null) {
    return;
  }

  const targetStart = Math.max(0, anchorIndex - state.prefetchBack);
  const targetEnd = Math.min(state.pageCount - 1, anchorIndex + state.prefetchForward);

  let changed = false;

  while (state.firstLoadedIndex > targetStart) {
    await prependPage(state.firstLoadedIndex - 1);
    changed = true;
  }

  while (state.lastLoadedIndex < targetEnd) {
    await appendPage(state.lastLoadedIndex + 1);
    changed = true;
  }

  while (state.firstLoadedIndex < targetStart) {
    trimTopPage();
    changed = true;
  }

  while (state.lastLoadedIndex > targetEnd) {
    trimBottomPage();
    changed = true;
  }

  if (changed) {
    scheduleUpdate();
  }
}

function findAnchorPageIndex() {
  if (pagesRoot.childElementCount === 0) {
    return null;
  }

  const viewportBottom = viewportHeight();
  let bestIndex = null;
  let bestVisiblePixels = -1;

  for (const element of pagesRoot.children) {
    const rect = element.getBoundingClientRect();
    const visiblePixels = Math.min(rect.bottom, viewportBottom) - Math.max(rect.top, 0);
    if (visiblePixels > bestVisiblePixels) {
      bestVisiblePixels = visiblePixels;
      bestIndex = Number(element.dataset.pageIndex);
    }
  }

  if (bestVisiblePixels > 0 && bestIndex !== null) {
    return bestIndex;
  }

  const first = pagesRoot.firstElementChild;
  if (first && first.getBoundingClientRect().top > 0) {
    return Number(first.dataset.pageIndex);
  }

  const last = pagesRoot.lastElementChild;
  return last ? Number(last.dataset.pageIndex) : null;
}

async function appendPage(index) {
  const element = await getPageElement(index);
  pagesRoot.appendChild(element);
  state.lastLoadedIndex = index;
  syncSpacers();
}

async function prependPage(index) {
  const element = await getPageElement(index);
  pagesRoot.prepend(element);
  state.firstLoadedIndex = index;
  const height = element.getBoundingClientRect().height;

  if (state.trimmedTopHeight > 0) {
    state.trimmedTopHeight = Math.max(0, state.trimmedTopHeight - height);
  }

  syncSpacers();
}

function trimTopPage() {
  const element = pagesRoot.firstElementChild;
  if (!element) {
    return;
  }

  const height = element.getBoundingClientRect().height;
  state.pageElements.delete(Number(element.dataset.pageIndex));
  element.remove();
  state.firstLoadedIndex += 1;
  state.trimmedTopHeight += height;
  syncSpacers();
}

function trimBottomPage() {
  const element = pagesRoot.lastElementChild;
  if (!element) {
    return;
  }

  state.pageElements.delete(Number(element.dataset.pageIndex));
  element.remove();
  state.lastLoadedIndex -= 1;
  syncSpacers();
}

async function getPageElement(index) {
  const existing = state.pageElements.get(index);
  if (existing) {
    return existing;
  }

  const inflight = state.inflightLoads.get(index);
  if (inflight) {
    return inflight;
  }

  const loadPromise = loadPageElement(index)
    .then((element) => {
      state.pageElements.set(index, element);
      state.inflightLoads.delete(index);
      return element;
    })
    .catch((error) => {
      state.inflightLoads.delete(index);
      throw error;
    });

  state.inflightLoads.set(index, loadPromise);
  return loadPromise;
}

function loadPageElement(index) {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.alt = `page ${index}`;
    image.src = `/page/${index}`;

    image.addEventListener(
      "load",
      () => {
        const element = document.createElement("article");
        element.dataset.pageIndex = String(index);
        element.appendChild(image);
        resolve(element);
      },
      { once: true },
    );

    image.addEventListener(
      "error",
      () => reject(new Error(`Failed to load page ${index}`)),
      { once: true },
    );
  });
}

function viewportHeight() {
  return window.innerHeight || document.documentElement.clientHeight || 0;
}

function syncSpacers() {
  const reserve = safetyReserveHeight();
  const targetTopSafety = state.firstLoadedIndex > 0 ? reserve : 0;
  const bottomSafety = state.lastLoadedIndex < state.pageCount - 1 ? reserve : 0;
  const topHeight = Math.max(0, Math.round(state.trimmedTopHeight + targetTopSafety));

  topSpacer.style.height = `${topHeight}px`;
  bottomSpacer.style.height = `${Math.max(0, Math.round(bottomSafety))}px`;

  if (!state.spacersInitialized) {
    state.spacersInitialized = true;
    state.topSafetyHeight = targetTopSafety;
    return;
  }

  const safetyDelta = targetTopSafety - state.topSafetyHeight;
  state.topSafetyHeight = targetTopSafety;
  if (safetyDelta !== 0) {
    window.scrollBy(0, safetyDelta);
  }
}

function safetyReserveHeight() {
  return Math.max(4000, Math.round(viewportHeight() * 6));
}

window.addEventListener("scroll", scheduleUpdate, { passive: true });
window.addEventListener("resize", scheduleUpdate);

init().catch((error) => {
  console.error(error);
});
