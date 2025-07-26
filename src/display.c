#include "headers.h"

extern SDL_Renderer *renderer;
extern const int width, height;
extern const int progress_font_size;
extern TTF_Text *const progress_text;

void display_single(SDL_Texture *image)
{
    SDL_FRect dst;
    SDL_GetTextureSize(image, &dst.w, &dst.h);

    float sw = width / dst.w;
    float sh = height / dst.h;
    float scale = sw < sh ? sw : sh;
    dst.x = (width / scale - dst.w) / 2;
    dst.y = (height / scale - dst.h) / 2;

    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTexture(renderer, image, nullptr, &dst);
}

void display_book(SDL_Texture *image1, SDL_Texture *image2)
{
    SDL_FRect dst1, dst2;
    SDL_GetTextureSize(image1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(image2, &dst2.w, &dst2.h);

    float s2_s1 = dst1.h / dst2.h;
    float sw = width / (dst1.w + s2_s1 * dst2.w);
    float sh = height / dst1.h;
    float scale1 = sw < sh ? sw : sh;
    float scale2 = scale1 * s2_s1;

    dst1.x = ((width - scale2 * dst2.w) / scale1 - dst1.w) / 2;
    dst1.y = (height / scale1 - dst1.h) / 2;
    dst2.x = ((width + scale1 * dst1.w) / scale2 - dst2.w) / 2;
    dst2.y = (height / scale2 - dst2.h) / 2;

    SDL_SetRenderScale(renderer, scale1, scale1);
    SDL_RenderTexture(renderer, image1, nullptr, &dst1);
    SDL_SetRenderScale(renderer, scale2, scale2);
    SDL_RenderTexture(renderer, image2, nullptr, &dst2);
}

void display_strip(struct appstate *s)
{
    float w;
    SDL_GetTextureSize(s->images[0], &w, nullptr);
    float distance = -s->scroll * s->zoom * width / w;

    for (int i = 0; i < arrlen(s->images); i++) {
        SDL_Texture *image = s->images[i];

        SDL_FRect dst;
        SDL_GetTextureSize(image, &dst.w, &dst.h);

        float scale = s->zoom * width / dst.w;
        dst.x = (width / scale - dst.w) / 2;
        dst.y = distance / scale;
        distance += dst.h * scale;

        SDL_SetRenderScale(renderer, scale, scale);
        SDL_RenderTexture(renderer, image, nullptr, &dst);
    }
}

void display_strip_rotated(struct appstate *s)
{
    float w;
    SDL_GetTextureSize(s->images[0], &w, nullptr);
    float distance = width + s->scroll * s->hzoom * height / w;

    for (int i = 0; i < arrlen(s->images); i++) {
        SDL_Texture *image = s->images[i];

        SDL_FRect dst;
        SDL_GetTextureSize(image, &dst.w, &dst.h);

        float scale = s->hzoom * height / dst.w;
        dst.x = distance / scale - (dst.w + dst.h) / 2;
        dst.y = (height / scale - dst.h) / 2;
        distance -= dst.h * scale;

        SDL_SetRenderScale(renderer, scale, scale);
        SDL_RenderTextureRotated(renderer, image, nullptr, &dst, 90, nullptr, SDL_FLIP_NONE);
    }
}

void display_progress()
{
    int w, h;
    TTF_GetTextSize(progress_text, &w, &h);

    int x = (width - w) / 2;
    int y = (height - h) / 2;

    int mx = 0.3 * progress_font_size;
    int my = 0.1 * h;

    SDL_SetRenderScale(renderer, 1, 1);
    SDL_SetRenderDrawColorFloat(renderer, 0, 0, 0, 0.7);
    SDL_RenderFillRect(renderer, &(SDL_FRect){x - mx, y - my, w + 2*mx, h + 2*my});
    TTF_DrawRendererText(progress_text, x, y);
}
