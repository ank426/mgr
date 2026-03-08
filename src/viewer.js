const pageCount = window.MGR_CONFIG.pageCount;
const prefetchBack = window.MGR_CONFIG.prefetchBack;
const prefetchForward = window.MGR_CONFIG.prefetchForward;
const pagesContainer = document.getElementById("pages");
const topSpacer = document.getElementById("top-spacer");
const bottomSpacer = document.getElementById("bottom-spacer");

const state = {
  firstLoadedIndex: 0,
  lastLoadedIndex: -1,
  loadedPageElements: new Map(),
  loadingPagePromises: new Map(),
  updatePending: false,
  updateInProgress: false,
  trimmedTopHeightPx: 0,
  previousTopSafetyPx: null,
};

async function initializeViewer() {
  if (pageCount === 0) {
    return;
  }

  for (let count = 0; count <= Math.min(pageCount - 1, prefetchForward); count++) {
    await appendPage();
  }

  requestWindowUpdate();
}

function requestWindowUpdate() {
  if (state.updatePending) {
    return;
  }

  state.updatePending = true;

  if (state.updateInProgress) {
    return;
  }

  requestAnimationFrame(flushWindowUpdate);
}

async function flushWindowUpdate() {
  if (state.updateInProgress) {
    return;
  }

  state.updateInProgress = true;

  try {
    while (state.updatePending) {
      state.updatePending = false;
      await reconcileWindow();
    }
  } finally {
    state.updateInProgress = false;
  }
}

async function reconcileWindow() {
  const visibleRange = getVisiblePageRange();
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
    await prependPage();
    changed = true;
  }

  while (state.lastLoadedIndex < targetEnd) {
    await appendPage();
    changed = true;
  }

  while (state.firstLoadedIndex < targetStart) {
    removeFirstPage();
    changed = true;
  }

  while (state.lastLoadedIndex > targetEnd) {
    removeLastPage();
    changed = true;
  }

  if (changed) {
    requestWindowUpdate();
  }
}

function getVisiblePageRange() {
  if (pagesContainer.childElementCount === 0) {
    return null;
  }

  const viewportBottom = window.innerHeight || document.documentElement.clientHeight || 0;
  let firstVisible = null;
  let lastVisible = null;

  for (const element of pagesContainer.children) {
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

  const first = pagesContainer.firstElementChild;
  const last = pagesContainer.lastElementChild;
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

async function prependPage() {
  const element = await getOrLoadMountedPageElement(state.firstLoadedIndex - 1);
  pagesContainer.prepend(element);
  state.firstLoadedIndex--;
  state.trimmedTopHeightPx = Math.max(0, state.trimmedTopHeightPx - element.getBoundingClientRect().height);
  syncVirtualSpacers();
}

async function appendPage() {
  const element = await getOrLoadMountedPageElement(state.lastLoadedIndex + 1);
  pagesContainer.appendChild(element);
  state.lastLoadedIndex++;
  syncVirtualSpacers();
}

function removeFirstPage() {
  const element = pagesContainer.firstElementChild;
  state.firstLoadedIndex++;
  state.trimmedTopHeightPx += element.getBoundingClientRect().height;
  removePageElement(element);
}

function removeLastPage() {
  const element = pagesContainer.lastElementChild;
  state.lastLoadedIndex--;
  removePageElement(element);
}

function removePageElement(element) {
  const pageIndex = Number(element.dataset.pageIndex);
  state.loadedPageElements.delete(pageIndex);
  const image = element.querySelector("img");

  if (image) {
    image.removeAttribute("srcset");
    image.src = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
    image.remove();
  }

  element.remove();
  syncVirtualSpacers();
}

function getOrLoadMountedPageElement(index) {
  const loadedElement = state.loadedPageElements.get(index);
  if (loadedElement) {
    return Promise.resolve(loadedElement);
  }

  const loadingPromise = state.loadingPagePromises.get(index);
  if (loadingPromise) {
    return loadingPromise;
  }

  const newLoadPromise = createPageElement(index)
    .then((element) => {
      state.loadedPageElements.set(index, element);
      return element;
    })
    .finally(() => {
      state.loadingPagePromises.delete(index);
    });

  state.loadingPagePromises.set(index, newLoadPromise);
  return newLoadPromise;
}

function createPageElement(index) {
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

function syncVirtualSpacers() {
  const viewportHeight = window.innerHeight || document.documentElement.clientHeight || 0;
  const reserve = Math.max(4000, Math.round(viewportHeight * 6));
  const targetTopSafety = state.firstLoadedIndex > 0 ? reserve : 0;
  const bottomSafety = state.lastLoadedIndex < pageCount - 1 ? reserve : 0;
  const topHeight = Math.max(0, Math.round(state.trimmedTopHeightPx + targetTopSafety));

  topSpacer.style.height = `${topHeight}px`;
  bottomSpacer.style.height = `${bottomSafety}px`;

  if (state.previousTopSafetyPx === null) {
    state.previousTopSafetyPx = targetTopSafety;
    return;
  }

  const safetyDelta = targetTopSafety - state.previousTopSafetyPx;
  state.previousTopSafetyPx = targetTopSafety;
  if (safetyDelta !== 0) {
    window.scrollBy(0, safetyDelta);
  }
}

window.addEventListener("scroll", requestWindowUpdate, { passive: true });
window.addEventListener("resize", requestWindowUpdate);

initializeViewer().catch((error) => {
  console.error(error);
});
