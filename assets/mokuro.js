// @ts-check

/** @typedef {{ box: [number, number, number, number], vertical: boolean, lines: string[] }} MokuroBlock */

export class MokuroPage {
    /** @param {string} img_path @param {number} img_width @param {number} img_height @param {MokuroBlock[]} blocks */
    constructor(img_path, img_width, img_height, blocks) {
        /** @type {string} */
        this.img_path = img_path;

        /** @type {number} */
        this.img_width = img_width;

        /** @type {number} */
        this.img_height = img_height;

        /** @type {MokuroBlock[]} */
        this.blocks = blocks;
    }

    /** @returns {HTMLDivElement} */
    createOverlay() {
        const overlay = document.createElement("div");

        for (const block of this.blocks) {
            const [x1, y1, x2, y2] = block.box;
            const boxW = x2 - x1;
            const boxH = y2 - y1;
            if (boxW <= 0 || boxH <= 0) continue;

            const text = block.lines.join("\n");
            if (!text) continue;

            const div = document.createElement("div");
            div.textContent = text;
            div.addEventListener("mouseleave", () => window.getSelection()?.removeAllRanges());

            div.style.left = `${(x1 / this.img_width) * 100}%`;
            div.style.top = `${(y1 / this.img_height) * 100}%`;
            div.style.width = `${(boxW / this.img_width) * 100}%`;
            div.style.height = `${(boxH / this.img_height) * 100}%`;

            const nLines = block.lines.length;
            const maxChars = Math.max(1, ...block.lines.map((l) => l.length));

            if (block.vertical) {
                div.style.writingMode = "vertical-rl";
                const fontSize = Math.min(boxW / nLines, boxH / maxChars);
                div.style.fontSize = `${(fontSize / this.img_width) * 100}cqw`;
                div.style.lineHeight = `${boxW / nLines / fontSize}em`;
            } else {
                const fontSize = Math.min(boxH / nLines, boxW / maxChars);
                div.style.fontSize = `${(fontSize / this.img_width) * 100}cqw`;
                div.style.lineHeight = `${boxH / nLines / fontSize}em`;
            }

            overlay.appendChild(div);
        }

        return overlay;
    }
}

/**
 * @param {string} volumeName
 * @returns {Promise<Map<string, MokuroPage> | null>}
 */
export async function fetchMokuroPages(volumeName) {
    try {
        const resp = await fetch(`/volume/${encodeURIComponent(volumeName)}/mokuro`);
        if (!resp.ok) return null;
        /** @type {{ img_path: string, img_width: number, img_height: number, blocks: MokuroBlock[] }[]} */
        const pages = (await resp.json()).pages;
        return new Map(
            pages
                .filter((p) => p.img_path)
                .map((p) => [p.img_path, new MokuroPage(p.img_path, p.img_width, p.img_height, p.blocks)]),
        );
    } catch (err) {
        console.error(`Failed to fetch mokuro for ${volumeName}:`, err);
        return null;
    }
}
