const volumes = window.MGR_CONFIG.volumes;
const [prefetchBack, prefetchForward] = window.MGR_CONFIG.prefetch;
const initialProgress = window.MGR_CONFIG.initialProgress;
const pagesContainer = document.getElementById("pages");

const pagesByVolume = new Map();

const state = {
  nearVisiblePages: new Set(),
  loadedPages: new Set(),
  reconcileScheduled: false,
};

function initializeViewer() {
  if (!pagesContainer) {
    throw new Error("Missing pages container");
  }

  buildDom();
  recomputeAllSlotHeights();

  const initialPage = pagesByVolume.get(initialProgress.file).get(initialProgress.page);
  window.scrollTo({ top: initialPage.slot.offsetTop + initialProgress.scroll * initialPage.slot.offsetHeight });

  const observer = new IntersectionObserver(handleIntersections, {
    root: null,
    rootMargin: `${prefetchBack * 100}% 0px ${prefetchForward * 100}% 0px`,
    threshold: 0,
  });

  forEachPage((page) => observer.observe(page.slot));

  window.addEventListener("resize", () => {
    recomputeAllSlotHeights();
    scheduleReconcile();
  });
}

function buildDom() {
  const fragment = document.createDocumentFragment();

  for (const volume of volumes) {
    const volumeSection = document.createElement("section");
    volumeSection.dataset.volume = volume.name;
    const volumePages = new Map();

    for (let pageNumber = 1; pageNumber <= volume.pageDims.length; pageNumber++) {
      const dimensions = volume.pageDims[pageNumber - 1];
      const slot = document.createElement("div");

      slot.className = "page-slot";
      slot.dataset.page = String(pageNumber);

      volumeSection.appendChild(slot);
      volumePages.set(pageNumber, {
        slot,
        volumeName: volume.name,
        pageNumber,
        dimensions,
        url: `/volume/${encodeURIComponent(volume.name)}/page/${pageNumber}`,
        status: "unloaded",
        image: null,
      });
    }

    pagesByVolume.set(volume.name, volumePages);
    fragment.appendChild(volumeSection);
  }

  pagesContainer.replaceChildren(fragment);
}

function recomputeAllSlotHeights() {
  const containerWidth = pagesContainer.clientWidth || window.innerWidth || 1;

  forEachPage((page) => {
    const [sourceWidth, sourceHeight] = page.dimensions;
    const safeWidth = Math.max(sourceWidth || 1, 1);
    const safeHeight = Math.max(sourceHeight || 1, 1);
    const slotHeight = Math.max(1, Math.round((containerWidth * safeHeight) / safeWidth));
    page.slot.style.height = `${slotHeight}px`;
  });
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

function scheduleReconcile() {
  if (state.reconcileScheduled) {
    return;
  }

  state.reconcileScheduled = true;
  requestAnimationFrame(() => {
    state.reconcileScheduled = false;
    reconcileWindow();
  });
}

function reconcileWindow() {
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

function forEachPage(callback) {
  for (const volumePages of pagesByVolume.values()) {
    for (const page of volumePages.values()) {
      callback(page);
    }
  }
}

function loadPage(page) {
  if (!page || page.status === "loading" || page.status === "loaded" || page.status === "failed") {
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
  if (!page || page.status === "failed") {
    return;
  }

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

initializeViewer();
