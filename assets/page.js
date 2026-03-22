export class Page {
    constructor(volumeName, pageNumber, dimensions, slot) {
        this.volumeName = volumeName;
        this.pageNumber = pageNumber;
        this.dimensions = dimensions;
        this.slot = slot;
        this.status = "unloaded";
        this.image = null;
    }

    load() {
        if (this.status === "loading" || this.status === "loaded" || this.status === "failed") return;

        const image = new Image();
        image.decoding = "async";
        image.alt = `volume ${this.volumeName} page ${this.pageNumber}`;

        this.status = "loading";
        this.image = image;

        image.addEventListener(
            "load",
            () => {
                if (this.image !== image) return;

                this.status = "loaded";
                this.slot.replaceChildren(image);
            },
            { once: true },
        );

        image.addEventListener(
            "error",
            () => {
                if (this.image !== image) return;

                this.status = "failed";
                this.image = null;
                this.slot.replaceChildren();
                console.error(`Failed to load volume ${this.volumeName} page ${this.pageNumber}`);
            },
            { once: true },
        );

        image.src = `/volume/${encodeURIComponent(this.volumeName)}/page/${this.pageNumber}`;
    }

    unload() {
        if (this.image) {
            this.image.removeAttribute("srcset");
            this.image.src = "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==";
            this.image.removeAttribute("src");
            this.image.remove();
        }

        this.image = null;
        this.slot.replaceChildren();
        this.status = "unloaded";
    }
}
