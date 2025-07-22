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

void display_strip(SDL_Texture *image1, SDL_Texture *image2, SDL_Texture *image3, struct appstate *s)
{
    SDL_FRect dst1, dst2, dst3;
    SDL_GetTextureSize(image1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(image2, &dst2.w, &dst2.h);
    SDL_GetTextureSize(image3, &dst3.w, &dst3.h);

    float scale1 = s->zoom * width / dst1.w;
    float scale2 = s->zoom * width / dst2.w;
    float scale3 = s->zoom * width / dst3.w;

    dst1.x = (width / scale1 - dst1.w) / 2;
    dst1.y = -s->scroll;
    dst2.x = (width / scale2 - dst2.w) / 2;
    dst2.y = (dst1.h - s->scroll) * scale1 / scale2;
    dst3.x = (width / scale3 - dst3.w) / 2;
    dst3.y = ((dst1.h - s->scroll) * scale1 + dst2.h * scale2) / scale3;

    SDL_SetRenderScale(renderer, scale1, scale1);
    SDL_RenderTexture(renderer, image1, nullptr, &dst1);
    SDL_SetRenderScale(renderer, scale2, scale2);
    SDL_RenderTexture(renderer, image2, nullptr, &dst2);
    SDL_SetRenderScale(renderer, scale3, scale3);
    SDL_RenderTexture(renderer, image3, nullptr, &dst3);
}

void display_strip_rotated(SDL_Texture *image1, SDL_Texture *image2, SDL_Texture *image3, struct appstate *s)
{
    SDL_FRect dst1, dst2, dst3;
    SDL_GetTextureSize(image1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(image2, &dst2.w, &dst2.h);
    SDL_GetTextureSize(image3, &dst3.w, &dst3.h);

    float scale1 = s->hzoom * height / dst1.w;
    float scale2 = s->hzoom * height / dst2.w;
    float scale3 = s->hzoom * height / dst3.w;

    dst1.x = width / scale1 - (dst1.w + dst1.h) / 2 + s->scroll;
    dst1.y = (height / scale1 - dst1.h) / 2;
    dst2.x = width / scale2 - (dst2.w + dst2.h) / 2 + (s->scroll - dst1.h) * scale1 / scale2;
    dst2.y = (height / scale2 - dst2.h) / 2;
    dst3.x = width / scale3 - (dst3.w + dst3.h) / 2 + ((s->scroll - dst1.h) * scale1 - dst2.h * scale2) / scale3;
    dst3.y = (height / scale3 - dst3.h) / 2;

    SDL_SetRenderScale(renderer, scale1, scale1);
    SDL_RenderTextureRotated(renderer, image1, nullptr, &dst1, 90, nullptr, SDL_FLIP_NONE);
    SDL_SetRenderScale(renderer, scale2, scale2);
    SDL_RenderTextureRotated(renderer, image2, nullptr, &dst2, 90, nullptr, SDL_FLIP_NONE);
    SDL_SetRenderScale(renderer, scale3, scale3);
    SDL_RenderTextureRotated(renderer, image3, nullptr, &dst3, 90, nullptr, SDL_FLIP_NONE);
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
