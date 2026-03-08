const volumes = window.MGR_CONFIG.volumes;
const [prefetchBack, prefetchForward] = window.MGR_CONFIG.prefetch;
const initialVolumeName = window.MGR_CONFIG.initialVolumeName;
const initialPageNumber = window.MGR_CONFIG.initialPageNumber;
const pagesContainer = document.getElementById("pages");
const topSpacer = document.getElementById("top-spacer");
const bottomSpacer = document.getElementById("bottom-spacer");
const orderedPages = buildOrderedPages(volumes);
const pageCount = orderedPages.length;

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

  const initialOrdinal = Math.max(0, findInitialOrdinal());
  state.firstLoadedIndex = initialOrdinal;
  state.lastLoadedIndex = initialOrdinal - 1;

  for (let count = initialOrdinal; count <= Math.min(pageCount - 1, initialOrdinal + prefetchForward); count++) {
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
  state.loadedPageElements.delete(Number(element.dataset.pageIndex));
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
    const pageRef = orderedPages[index];
    const image = new Image();
    image.decoding = "async";
    image.alt = `volume ${pageRef.volumeName} page ${pageRef.pageNumber}`;
    image.src = `/volume/${encodeURIComponent(pageRef.volumeName)}/page/${pageRef.pageNumber}`;

    image.addEventListener(
      "load",
      () => {
        const element = document.createElement("article");
        element.dataset.pageIndex = String(index);
        element.dataset.volumeIndex = String(pageRef.volumeIndex);
        element.dataset.volumeName = pageRef.volumeName;
        element.dataset.volumePageNumber = String(pageRef.pageNumber);
        element.appendChild(image);
        resolve(element);
      },
      { once: true },
    );

    image.addEventListener(
      "error",
      () => reject(new Error(`Failed to load volume ${pageRef.volumeName} page ${pageRef.pageNumber}`)),
      { once: true },
    );
  });
}

function buildOrderedPages(volumesConfig) {
  const pages = [];
  volumesConfig.forEach((volume, volumeIndex) => {
    for (let pageNumber = 1; pageNumber <= volume.pageDims.length; pageNumber++) {
      pages.push({ volumeIndex, volumeName: volume.name, pageNumber, dimensions: volume.pageDims[pageNumber - 1] });
    }
  });
  return pages;
}

function findInitialOrdinal() {
  let ordinal = 0;
  for (const volume of volumes) {
    const pageCountForVolume = volume.pageDims.length;
    if (volume.name === initialVolumeName) {
      const cappedPageNumber = Math.min(Math.max(initialPageNumber, 1), Math.max(pageCountForVolume, 1));
      return ordinal + (cappedPageNumber - 1);
    }
    ordinal += pageCountForVolume;
  }

  return 0;
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
