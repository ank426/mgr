export function loadPage(page) {
    if (page.status === "loading" || page.status === "loaded" || page.status === "failed") return;

    const image = new Image();
    image.decoding = "async";
    image.alt = `volume ${page.volumeName} page ${page.pageNumber}`;

    page.status = "loading";
    page.image = image;

    image.addEventListener(
        "load",
        () => {
            if (page.image !== image) return;

            page.status = "loaded";
            page.slot.replaceChildren(image);
        },
        { once: true },
    );

    image.addEventListener(
        "error",
        () => {
            if (page.image !== image) return;

            page.status = "failed";
            page.image = null;
            page.slot.replaceChildren();
            console.error(`Failed to load volume ${page.volumeName} page ${page.pageNumber}`);
        },
        { once: true },
    );

    image.src = page.url;
}

export function unloadPage(page) {
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
