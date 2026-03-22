// @ts-check

export class Page {
    /** @param {string} volumeName
     ** @param {number} pageNumber
     ** @param {[number, number]} dimensions
     ** @param {HTMLDivElement} slot */
    constructor(volumeName, pageNumber, dimensions, slot) {
        /** @type {string} */
        this.volumeName = volumeName;

        /** @type {number} */
        this.pageNumber = pageNumber;

        /** @type {[number, number]} */
        this.dimensions = dimensions;

        /** @type {HTMLDivElement} */
        this.slot = slot;

        /** @type {"unloaded" | "loading" | "loaded" | "failed"} */
        this.status = "unloaded";

        /** @type {HTMLImageElement | null} */
        this.image = null;
    }

    /** @returns {void} */
    load() {
        if (this.status !== "unloaded") return;
        this.status = "loading";
        const img = (this.image = new Image());
        img.decoding = "async";
        img.onload = () => {
            this.status = "loaded";
            this.slot.replaceChildren(img);
        };
        img.onerror = () => {
            this.status = "failed";
            this.image = null;
            this.slot.replaceChildren();
            console.error(`Failed to load volume ${this.volumeName} page ${this.pageNumber}`);
        };
        img.src = `/volume/${encodeURIComponent(this.volumeName)}/page/${this.pageNumber}`;
    }

    /** @returns {void} */
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
