const volumes = window.MGR_CONFIG.volumes;
const [prefetchBack, prefetchForward] = window.MGR_CONFIG.prefetch;
const pagesContainer = document.getElementById("pages");

const pageSlots = [];

const state = {
  nearVisibleIndices: new Set(),
  windowStart: 0,
  windowEnd: -1,
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
    rootMargin: "100% 0px 100% 0px",
    threshold: 0,
  });

  for (const page of pageSlots) {
    observer.observe(page.slot);
  }

  applyWindow(0, Math.min(pageSlots.length - 1, prefetchForward));

  window.addEventListener("resize", () => {
    recomputeAllSlotHeights();
    scheduleReconcile();
  });
}

function buildDom() {
  const fragment = document.createDocumentFragment();
  let globalIndex = 0;

  for (const volume of volumes) {
    const volumeSection = document.createElement("section");
    volumeSection.dataset.volumeName = volume.name;

    for (let pageOffset = 0; pageOffset < volume.pageDims.length; pageOffset++) {
      const pageNumber = pageOffset + 1;
      const dimensions = volume.pageDims[pageOffset];

      const slot = document.createElement("div");
      slot.dataset.pageSlot = "1";
      slot.dataset.pageIndex = String(globalIndex);
      slot.dataset.volumeName = volume.name;
      slot.dataset.pageNumber = String(pageNumber);

      volumeSection.appendChild(slot);

      pageSlots.push({
        index: globalIndex,
        slot,
        volumeName: volume.name,
        pageNumber,
        dimensions,
        src: `/volume/${encodeURIComponent(volume.name)}/page/${pageNumber}`,
        status: "unloaded",
        image: null,
      });

      globalIndex++;
    }

    fragment.appendChild(volumeSection);
  }

  pagesContainer.replaceChildren(fragment);
}

function recomputeAllSlotHeights() {
  const width = pagesContainer.clientWidth || window.innerWidth || 1;

  for (const page of pageSlots) {
    const [sourceWidth, sourceHeight] = page.dimensions;
    const safeWidth = Math.max(sourceWidth || 1, 1);
    const safeHeight = Math.max(sourceHeight || 1, 1);
    const renderedHeight = Math.max(1, Math.round((width * safeHeight) / safeWidth));
    page.slot.style.height = `${renderedHeight}px`;
  }
}

function handleIntersections(entries) {
  for (const entry of entries) {
    const index = Number(entry.target.dataset.pageIndex);
    if (entry.isIntersecting) {
      state.nearVisibleIndices.add(index);
    } else {
      state.nearVisibleIndices.delete(index);
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
  if (pageSlots.length === 0) {
    return;
  }

  if (state.nearVisibleIndices.size === 0) {
    return;
  }

  let minIndex = pageSlots.length - 1;
  let maxIndex = 0;

  for (const index of state.nearVisibleIndices) {
    if (index < minIndex) {
      minIndex = index;
    }
    if (index > maxIndex) {
      maxIndex = index;
    }
  }

  const nextStart = Math.max(0, minIndex - prefetchBack);
  const nextEnd = Math.min(pageSlots.length - 1, maxIndex + prefetchForward);
  applyWindow(nextStart, nextEnd);
}

function applyWindow(start, end) {
  if (state.windowEnd >= state.windowStart) {
    for (let index = state.windowStart; index <= state.windowEnd; index++) {
      if (index < start || index > end) {
        unloadPage(index);
      }
    }
  }

  for (let index = start; index <= end; index++) {
    loadPage(index);
  }

  state.windowStart = start;
  state.windowEnd = end;
}

function loadPage(index) {
  const page = pageSlots[index];
  if (!page || page.status === "loading" || page.status === "loaded" || page.status === "failed") {
    return;
  }

  const image = new Image();
  image.decoding = "async";
  image.alt = `volume ${page.volumeName} page ${page.pageNumber}`;
  image.dataset.pageIndex = String(index);

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

  image.src = page.src;
}

function unloadPage(index) {
  const page = pageSlots[index];
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
