const volumes = window.MGR_CONFIG.volumes;
const [prefetchBack, prefetchForward] = window.MGR_CONFIG.prefetch;
const pagesContainer = document.getElementById("pages");

const volumeByName = new Map();
const pagesByVolume = new Map();

const state = {
  nearVisiblePages: new Set(),
  loadedPages: new Set(),
  expandedStart: 0,
  expandedEnd: -1,
  reconcileScheduled: false,
};

let pageObserver;
let saveProgressTimeout;

async function initializeViewer() {
  if (!pagesContainer) {
    throw new Error("Missing pages container");
  }

  buildDom();

  const initialProgress = await fetch("/api/progress").then((response) => response.json());

  pageObserver = new IntersectionObserver(handleIntersections, {
    root: null,
    rootMargin: `${prefetchBack * 100}% 0px ${prefetchForward * 100}% 0px`,
    threshold: 0,
  });

  const initialVolumeIndex = volumeByName.get(initialProgress.file).index;
  state.expandedStart = Math.max(0, initialVolumeIndex - 1);
  state.expandedEnd = Math.min(volumes.length - 1, initialVolumeIndex + 1);
  for (let idx = state.expandedStart; idx <= state.expandedEnd; idx++) {
    expandVolume(idx);
  }

  const initialPage = pagesByVolume.get(initialProgress.file).get(initialProgress.page);
  window.scrollTo({ top: initialPage.slot.offsetTop + initialProgress.scroll * initialPage.slot.offsetHeight });

  window.addEventListener("resize", handleViewportChange);
  window.addEventListener("scroll", handleViewportChange, { passive: true });
}

function buildDom() {
  const fragment = document.createDocumentFragment();

  for (const [index, volume] of volumes.entries()) {
    const section = document.createElement("section");
    section.dataset.volume = volume.name;
    volumeByName.set(volume.name, { index, section });
    fragment.appendChild(section);
  }

  pagesContainer.replaceChildren(fragment);
}

function handleIntersections(entries) {
  for (const entry of entries) {
    const page = pagesByVolume
      .get(entry.target.closest("section").dataset.volume)
      .get(Number(entry.target.dataset.page));

    if (entry.isIntersecting) {
      state.nearVisiblePages.add(page);
    } else {
      state.nearVisiblePages.delete(page);
    }
  }

  scheduleReconcile();
}

function handleViewportChange() {
  scheduleReconcile();
  scheduleProgressSave();
}

function scheduleReconcile() {
  if (state.reconcileScheduled) {
    return;
  }

  state.reconcileScheduled = true;
  requestAnimationFrame(() => {
    state.reconcileScheduled = false;
    reconcileVolumes();
    reconcilePages();
  });
}

function reconcileVolumes() {
  if (state.expandedEnd < state.expandedStart) {
    return;
  }

  const activeVolumeIndex = volumeByName.get(getActivePage().volumeName).index;
  const start = Math.max(0, activeVolumeIndex - 1);
  const end = Math.min(volumes.length - 1, activeVolumeIndex + 1);

  for (let idx = start; idx <= end && idx < state.expandedStart; idx++) {
    expandVolume(idx);
  }
  for (let idx = end; idx >= start && idx > state.expandedEnd; idx--) {
    expandVolume(idx);
  }
  for (let idx = state.expandedStart; idx < start && idx <= state.expandedEnd; idx++) {
    collapseVolume(idx);
  }
  for (let idx = state.expandedEnd; idx > end && idx >= state.expandedStart; idx--) {
    collapseVolume(idx);
  }

  state.expandedStart = start;
  state.expandedEnd = end;
}

function reconcilePages() {
  if (state.nearVisiblePages.size === 0) {
    return;
  }

  for (const page of Array.from(state.loadedPages)) {
    if (!state.nearVisiblePages.has(page)) {
      unloadPage(page);
      state.loadedPages.delete(page);
    }
  }

  for (const page of state.nearVisiblePages) {
    if (!state.loadedPages.has(page)) {
      loadPage(page);
      state.loadedPages.add(page);
    }
  }
}

function scheduleProgressSave() {
  clearTimeout(saveProgressTimeout);
  saveProgressTimeout = setTimeout(saveProgress, 200);
}

function saveProgress() {
  const activePage = getActivePage();
  const top = window.scrollY || window.pageYOffset || 0;

  fetch("/api/progress", {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      file: activePage.volumeName,
      page: activePage.pageNumber,
      scroll: Math.min(1, Math.max(0, (top - activePage.slot.offsetTop) / activePage.slot.offsetHeight)),
    }),
  });
}

function expandVolume(index) {
  const volume = volumes[index];
  const section = volumeByName.get(volume.name).section;
  const volumePages = new Map();
  const fragment = document.createDocumentFragment();

  for (let pageNumber = 1; pageNumber <= volume.pageDims.length; pageNumber++) {
    const dimensions = volume.pageDims[pageNumber - 1];
    const slot = document.createElement("div");

    slot.className = "page-slot";
    slot.dataset.page = String(pageNumber);
    slot.style.aspectRatio = `${dimensions[0]} / ${dimensions[1]}`;

    fragment.appendChild(slot);
    const page = {
      slot,
      volumeName: volume.name,
      pageNumber,
      dimensions,
      url: `/volume/${encodeURIComponent(volume.name)}/page/${pageNumber}`,
      status: "unloaded",
      image: null,
    };
    volumePages.set(pageNumber, page);
    pageObserver.observe(slot);
  }

  section.replaceChildren(fragment);
  pagesByVolume.set(volume.name, volumePages);
}

function collapseVolume(index) {
  const volumeName = volumes[index].name;
  for (const page of pagesByVolume.get(volumeName).values()) {
    unloadPage(page);
    state.loadedPages.delete(page);
    state.nearVisiblePages.delete(page);
    pageObserver.unobserve(page.slot);
  }
  volumeByName.get(volumeName).section.replaceChildren();
  pagesByVolume.delete(volumeName);
}

function getActivePage() {
  const top = window.scrollY || window.pageYOffset || 0;
  for (let idx = state.expandedStart; idx <= state.expandedEnd; idx++) {
    for (const page of pagesByVolume.get(volumes[idx].name).values()) {
      if (top >= page.slot.offsetTop && top < page.slot.offsetTop + page.slot.offsetHeight) {
        return page;
      }
    }
  }
  throw new Error("No active page found");
}

function loadPage(page) {
  if (page.status === "loading" || page.status === "loaded" || page.status === "failed") {
    return;
  }

  const image = new Image();
  image.decoding = "async";
  image.alt = `volume ${page.volumeName} page ${page.pageNumber}`;

  page.status = "loading";
  page.image = image;

  image.addEventListener(
    "load",
    () => {
      if (page.image !== image) {
        return;
      }

      page.status = "loaded";
      page.slot.replaceChildren(image);
    },
    { once: true },
  );

  image.addEventListener(
    "error",
    () => {
      if (page.image !== image) {
        return;
      }

      page.status = "failed";
      page.image = null;
      page.slot.replaceChildren();
      console.error(`Failed to load volume ${page.volumeName} page ${page.pageNumber}`);
    },
    { once: true },
  );

  image.src = page.url;
}

function unloadPage(page) {
  if (page.image) {
    page.image.removeAttribute("srcset");
    page.image.src = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
    page.image.removeAttribute("src");
    page.image.remove();
  }

  page.image = null;
  page.slot.replaceChildren();
  page.status = "unloaded";
}

initializeViewer().catch((error) => {
  console.error(error);
});
