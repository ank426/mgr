const pageCount = window.MGR_CONFIG.pageCount;
const prefetchBack = window.MGR_CONFIG.prefetchBack;
const prefetchForward = window.MGR_CONFIG.prefetchForward;
const pagesRoot = document.getElementById("pages");
const topSpacer = document.getElementById("top-spacer");
const bottomSpacer = document.getElementById("bottom-spacer");

const state = {
  firstLoadedIndex: 0,
  lastLoadedIndex: -1,
  loadedPages: new Map(),
  loadingPages: new Map(),
  needsUpdate: false,
  updateRunning: false,
  trimmedTopHeight: 0,
  topSafetyHeight: null,
};

async function init() {
  if (pageCount === 0) {
    return;
  }

  for (let idx = 0; idx <= Math.min(pageCount - 1, prefetchForward); idx++) {
    await insertPage(idx, false);
  }

  syncSpacers();
  scheduleUpdate();
}

function scheduleUpdate() {
  if (state.needsUpdate) {
    return;
  }

  state.needsUpdate = true;

  if (state.updateRunning) {
    return;
  }

  requestAnimationFrame(runUpdate);
}

async function runUpdate() {
  if (!state.needsUpdate || state.updateRunning) {
    return;
  }

  state.needsUpdate = false;
  state.updateRunning = true;

  try {
    await updateWindow();
  } finally {
    state.updateRunning = false;

    if (state.needsUpdate) {
      requestAnimationFrame(runUpdate);
    }
  }
}

async function updateWindow() {
  const visibleRange = findVisibleRange();
  if (!visibleRange) {
    return;
  }

  const targetStart = Math.max(0, visibleRange.first - prefetchBack);
  const targetEnd = Math.min(pageCount - 1, visibleRange.last + prefetchForward);

  if (state.firstLoadedIndex === targetStart && state.lastLoadedIndex === targetEnd) {
    return;
  }

  let changed = false;

  while (state.firstLoadedIndex > targetStart) {
    await insertPage(state.firstLoadedIndex - 1, true);
    changed = true;
  }

  while (state.lastLoadedIndex < targetEnd) {
    await insertPage(state.lastLoadedIndex + 1, false);
    changed = true;
  }

  while (state.firstLoadedIndex < targetStart) {
    removePage(true);
    changed = true;
  }

  while (state.lastLoadedIndex > targetEnd) {
    removePage(false);
    changed = true;
  }

  if (changed) {
    scheduleUpdate();
  }
}

function findVisibleRange() {
  if (pagesRoot.childElementCount === 0) {
    return null;
  }

  const viewportBottom = window.innerHeight || document.documentElement.clientHeight || 0;
  let firstVisible = null;
  let lastVisible = null;

  for (const element of pagesRoot.children) {
    const rect = element.getBoundingClientRect();
    const intersectsViewport = rect.bottom > 0 && rect.top < viewportBottom;
    if (!intersectsViewport) {
      continue;
    }

    const idx = Number(element.dataset.pageIndex);
    if (firstVisible === null || idx < firstVisible) {
      firstVisible = idx;
    }
    if (lastVisible === null || idx > lastVisible) {
      lastVisible = idx;
    }
  }

  if (firstVisible !== null && lastVisible !== null) {
    return { first: firstVisible, last: lastVisible };
  }

  const first = pagesRoot.firstElementChild;
  const last = pagesRoot.lastElementChild;
  if (!first || !last) {
    return null;
  }

  const firstIndex = Number(first.dataset.pageIndex);
  const lastIndex = Number(last.dataset.pageIndex);
  if (first.getBoundingClientRect().top > 0) {
    return { first: firstIndex, last: firstIndex };
  }
  if (last.getBoundingClientRect().bottom < 0) {
    return { first: lastIndex, last: lastIndex };
  }

  return { first: state.firstLoadedIndex, last: state.lastLoadedIndex };
}

async function insertPage(index, prepend) {
  const element = await getOrLoadPageElement(index);
  if (prepend) {
    pagesRoot.prepend(element);
    state.firstLoadedIndex = index;
    const height = element.getBoundingClientRect().height;
    if (state.trimmedTopHeight > 0) {
      state.trimmedTopHeight = Math.max(0, state.trimmedTopHeight - height);
    }
  } else {
    pagesRoot.appendChild(element);
    state.lastLoadedIndex = index;
  }

  syncSpacers();
}

function removePage(fromStart) {
  const element = fromStart
    ? pagesRoot.firstElementChild
    : pagesRoot.lastElementChild;

  if (!element) {
    return;
  }

  const height = fromStart ? element.getBoundingClientRect().height : 0;
  const pageIndex = Number(element.dataset.pageIndex);
  state.loadedPages.delete(pageIndex);
  const image = element.querySelector("img");

  if (image) {
    image.removeAttribute("srcset");
    image.src = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
    image.remove();
  }

  if (fromStart) {
    state.firstLoadedIndex++;
    state.trimmedTopHeight += height;
  } else {
    state.lastLoadedIndex--;
  }

  element.remove();
  syncSpacers();
}

function getOrLoadPageElement(index) {
  const loadedElement = state.loadedPages.get(index);
  if (loadedElement) {
    return Promise.resolve(loadedElement);
  }

  const loadingPromise = state.loadingPages.get(index);
  if (loadingPromise) {
    return loadingPromise;
  }

  const newLoadPromise = loadPageElement(index)
    .then((element) => {
      state.loadedPages.set(index, element);
      return element;
    })
    .finally(() => {
      state.loadingPages.delete(index);
    });

  state.loadingPages.set(index, newLoadPromise);
  return newLoadPromise;
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

function syncSpacers() {
  const viewportHeight = window.innerHeight || document.documentElement.clientHeight || 0;
  const reserve = Math.max(4000, Math.round(viewportHeight * 6));
  const targetTopSafety = state.firstLoadedIndex > 0 ? reserve : 0;
  const bottomSafety = state.lastLoadedIndex < pageCount - 1 ? reserve : 0;
  const topHeight = Math.max(0, Math.round(state.trimmedTopHeight + targetTopSafety));

  topSpacer.style.height = `${topHeight}px`;
  bottomSpacer.style.height = `${Math.max(0, Math.round(bottomSafety))}px`;

  if (state.topSafetyHeight === null) {
    state.topSafetyHeight = targetTopSafety;
    return;
  }

  const safetyDelta = targetTopSafety - state.topSafetyHeight;
  state.topSafetyHeight = targetTopSafety;
  if (safetyDelta !== 0) {
    window.scrollBy(0, safetyDelta);
  }
}

window.addEventListener("scroll", scheduleUpdate, { passive: true });
window.addEventListener("resize", scheduleUpdate);

init().catch((error) => {
  console.error(error);
});
