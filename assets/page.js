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
        if (this.status !== "unloaded") return;
        this.status = "loading";
        this.image = new Image();
        this.image.decoding = "async";
        this.image.onload = () => {
            this.status = "loaded";
            this.slot.replaceChildren(this.image);
        };
        this.image.onerror = () => {
            this.status = "failed";
            this.image = null;
            this.slot.replaceChildren();
            console.error(`Failed to load volume ${this.volumeName} page ${this.pageNumber}`);
        };
        this.image.src = `/volume/${encodeURIComponent(this.volumeName)}/page/${this.pageNumber}`;
    }

    unload() {
        if (this.image) {
            this.image.onload = null;
            this.image.onerror = null;
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
