// @ts-check

/** @typedef {{ box: [number, number, number, number], vertical: boolean, lines: string[] }} MokuroBlockData */
/** @typedef {{ img_path: string, img_width: number, img_height: number, blocks: MokuroBlockData[] }[]} MokuroPageData */
/** @typedef {MokuroBlockData & { darkTheme: boolean }} MokuroBlock */

export class MokuroPage {
    /** @param {string} imgPath @param {number} imgWidth @param {number} imgHeight @param {MokuroBlockData[]} blocks */
    constructor(imgPath, imgWidth, imgHeight, blocks) {
        /** @type {string} */
        this.imgPath = imgPath;

        /** @type {number} */
        this.imgW = imgWidth;

        /** @type {number} */
        this.imgH = imgHeight;

        /** @type {MokuroBlock[]} */
        this.blocks = blocks
            .filter((block) => /[\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Han}]/u.test(block.lines.join("")))
            .map((block) => ({ ...block, darkTheme: false }));
    }

    /** @param {HTMLImageElement} img @returns {HTMLDivElement} */
    createOverlay(img) {
        const overlay = document.createElement("div");
        this.computeThemes(img);

        for (const block of this.blocks) {
            const [x1, y1, x2, y2] = block.box;
            const boxW = x2 - x1;
            const boxH = y2 - y1;
            if (boxW <= 0 || boxH <= 0) continue;

            const div = document.createElement("div");
            this.setBlockText(div, block);
            div.addEventListener("mouseleave", () => window.getSelection()?.removeAllRanges());
            if (block.darkTheme) div.classList.add("theme-dark");

            div.style.left = `${(x1 / this.imgW) * 100}%`;
            div.style.top = `${(y1 / this.imgH) * 100}%`;
            div.style.width = `${(boxW / this.imgW) * 100}%`;
            div.style.height = `${(boxH / this.imgH) * 100}%`;

            const nLines = block.lines.length;
            const maxChars = Math.max(1, ...block.lines.map((l) => l.length));

            if (block.vertical) {
                div.style.writingMode = "vertical-rl";
                const fontSize = Math.min(boxW / nLines, boxH / maxChars);
                div.style.fontSize = `${(fontSize / this.imgW) * 100}cqw`;
                div.style.lineHeight = `${boxW / nLines / fontSize}em`;
            } else {
                const fontSize = Math.min(boxH / nLines, boxW / maxChars);
                div.style.fontSize = `${(fontSize / this.imgW) * 100}cqw`;
                div.style.lineHeight = `${boxH / nLines / fontSize}em`;
            }

            overlay.appendChild(div);
        }

        return overlay;
    }

    /** @param {HTMLDivElement} div @param {MokuroBlock} block @returns {void} */
    setBlockText(div, block) {
        for (const line of block.lines) {
            const lineElement = document.createElement("span");

            if (!block.vertical) {
                lineElement.textContent = line;
                div.append(lineElement);
                continue;
            }

            for (const [partIndex, part] of line.split(/([０-９]+)/u).entries()) {
                if (!part) continue;
                if (partIndex % 2 === 1 && (part.length === 2 || part.length === 3)) {
                    const span = document.createElement("span");
                    span.textContent = part;
                    lineElement.append(span);
                } else lineElement.append(part);
            }

            div.append(lineElement);
        }
    }

    /** @param {HTMLImageElement} img @returns {void} */
    computeThemes(img) {
        const canvas = document.createElement("canvas");
        const context = canvas.getContext("2d", { willReadFrequently: true });
        if (!context) return;

        const maxSize = 256;
        const scale = Math.min(1, maxSize / Math.max(img.naturalWidth, img.naturalHeight));
        canvas.width = Math.max(1, Math.round(img.naturalWidth * scale));
        canvas.height = Math.max(1, Math.round(img.naturalHeight * scale));
        context.drawImage(img, 0, 0, canvas.width, canvas.height);

        let pixels;
        try {
            pixels = context.getImageData(0, 0, canvas.width, canvas.height).data;
        } catch (err) {
            console.error(`Failed to analyze mokuro image region for ${this.imgPath}:`, err);
            return;
        }

        for (const block of this.blocks) {
            const [x1, y1, x2, y2] = block.box;
            const startX = Math.max(0, Math.min(canvas.width - 1, Math.floor((x1 / this.imgW) * canvas.width)));
            const startY = Math.max(0, Math.min(canvas.height - 1, Math.floor((y1 / this.imgH) * canvas.height)));
            const endX = Math.max(startX + 1, Math.min(canvas.width, Math.ceil((x2 / this.imgW) * canvas.width)));
            const endY = Math.max(startY + 1, Math.min(canvas.height, Math.ceil((y2 / this.imgH) * canvas.height)));

            let darkPixels = 0;
            let brightPixels = 0;

            for (let y = startY; y < endY; y++)
                for (let x = startX; x < endX; x++) {
                    const offset = (y * canvas.width + x) * 4;
                    if (pixels[offset] + pixels[offset + 1] + pixels[offset + 2] < 384) darkPixels++;
                    else brightPixels++;
                }

            block.darkTheme = darkPixels > brightPixels;
        }
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
        /** @type {MokuroPageData} */
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
