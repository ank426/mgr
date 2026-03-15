const volumes = window.MGR_CONFIG.volumes;
const [prefetchBack, prefetchForward] = window.MGR_CONFIG.prefetch;
const pagesContainer = document.getElementById("pages");

const pageSlots = [];

const state = {
  nearVisibleIndices: new Set(),
  loadedStart: 0,
  loadedEnd: -1,
  reconcileScheduled: false,
};

function initializeViewer() {
  if (!pagesContainer) {
    throw new Error("Missing pages container");
  }

  buildDom();
  recomputeAllSlotHeights();

  if (pageSlots.length === 0) {
    return;
  }

  const observer = new IntersectionObserver(handleIntersections, {
    root: null,
    rootMargin: `${prefetchBack * 100}% 0px ${prefetchForward * 100}% 0px`,
    threshold: 0,
  });

  for (const page of pageSlots) {
    observer.observe(page.slot);
  }

  window.addEventListener("resize", () => {
    recomputeAllSlotHeights();
    scheduleReconcile();
  });
}

function buildDom() {
  const fragment = document.createDocumentFragment();
  let pageIndex = 0;

  for (const volume of volumes) {
    const volumeSection = document.createElement("section");
    volumeSection.dataset.volume = volume.name;

    for (let pageNumber = 1; pageNumber <= volume.pageDims.length; pageNumber++) {
      const dimensions = volume.pageDims[pageNumber - 1];
      const slot = document.createElement("div");

      slot.className = "page-slot";
      slot.dataset.index = String(pageIndex);
      slot.dataset.page = String(pageNumber);

      volumeSection.appendChild(slot);
      pageSlots.push({
        slot,
        volumeName: volume.name,
        pageNumber,
        dimensions,
        url: `/volume/${encodeURIComponent(volume.name)}/page/${pageNumber}`,
        status: "unloaded",
        image: null,
      });

      pageIndex++;
    }

    fragment.appendChild(volumeSection);
  }

  pagesContainer.replaceChildren(fragment);
}

function recomputeAllSlotHeights() {
  const containerWidth = pagesContainer.clientWidth || window.innerWidth || 1;

  for (const page of pageSlots) {
    const [sourceWidth, sourceHeight] = page.dimensions;
    const safeWidth = Math.max(sourceWidth || 1, 1);
    const safeHeight = Math.max(sourceHeight || 1, 1);
    const slotHeight = Math.max(1, Math.round((containerWidth * safeHeight) / safeWidth));
    page.slot.style.height = `${slotHeight}px`;
  }
}

function handleIntersections(entries) {
  for (const entry of entries) {
    const pageIndex = Number(entry.target.dataset.index);
    if (Number.isNaN(pageIndex)) {
      continue;
    }

    if (entry.isIntersecting) {
      state.nearVisibleIndices.add(pageIndex);
    } else {
      state.nearVisibleIndices.delete(pageIndex);
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
  if (pageSlots.length === 0 || state.nearVisibleIndices.size === 0) {
    return;
  }

  const indices = Array.from(state.nearVisibleIndices);
  const start = Math.min(...indices);
  const end = Math.max(...indices);

  for (let pageIndex = state.loadedStart; pageIndex <= state.loadedEnd && pageIndex < start; pageIndex++) {
    unloadPage(pageIndex);
  }

  for (let pageIndex = state.loadedEnd; pageIndex >= state.loadedStart && pageIndex > end; pageIndex--) {
    unloadPage(pageIndex);
  }

  for (let pageIndex = start; pageIndex <= end && pageIndex < state.loadedStart; pageIndex++) {
    loadPage(pageIndex);
  }

  for (let pageIndex = end; pageIndex >= start && pageIndex > state.loadedEnd; pageIndex--) {
    loadPage(pageIndex);
  }

  state.loadedStart = start;
  state.loadedEnd = end;
}

function loadPage(pageIndex) {
  const page = pageSlots[pageIndex];
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

function unloadPage(pageIndex) {
  const page = pageSlots[pageIndex];
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
