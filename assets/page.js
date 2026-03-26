// @ts-check

/** @typedef {import("./mokuro.js").MokuroPage} MokuroPage */

export class Page {
    /** @param {string} volumeName
     ** @param {number} pageNumber
     ** @param {[number, number]} dimensions
     ** @param {HTMLDivElement} slot
     ** @param {MokuroPage | null} mokuroPage */
    constructor(volumeName, pageNumber, dimensions, slot, mokuroPage) {
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

        /** @type {MokuroPage | null} */
        this.mokuroPage = mokuroPage;
    }

    /** @returns {void} */
    load() {
        if (this.status !== "unloaded") return;
        this.status = "loading";

        const frame = document.createElement("div");
        const img = (this.image = new Image());
        frame.className = "page-frame";
        img.decoding = "async";
        frame.append(img);

        img.onload = () => {
            this.status = "loaded";
            if (this.mokuroPage) frame.append(this.mokuroPage.createOverlay());
            this.slot.replaceChildren(frame);
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
