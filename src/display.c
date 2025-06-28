#include "headers.h"
#include "globals.h"

void display_single(SDL_Texture **img_ptr)
{
    SDL_FRect dst;
    SDL_GetTextureSize(*img_ptr, &dst.w, &dst.h);

    float sw = width / dst.w;
    float sh = height / dst.h;
    float scale = sw < sh ? sw : sh;
    dst.x = (width / scale - dst.w) / 2;
    dst.y = (height / scale - dst.h) / 2;

    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTexture(renderer, *img_ptr, nullptr, &dst);
}

void display_book(SDL_Texture **img_ptr1, SDL_Texture **img_ptr2)
{
    SDL_FRect dst1, dst2;
    SDL_GetTextureSize(*img_ptr1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(*img_ptr2, &dst2.w, &dst2.h);

    float scale2 = dst1.h / dst2.h;
    float sw = width / (dst1.w + scale2 * dst2.w);
    float sh = height / dst1.h;
    float scale = sw < sh ? sw : sh;

    dst1.x = (width / scale - dst1.w - scale2 * dst2.w) / 2;
    dst1.y = (height / scale - dst1.h) / 2;
    dst2.x = (width / (scale*scale2) + dst1.w / scale2 - dst2.w) / 2;
    dst2.y = (height / (scale*scale2) - dst2.h) / 2;

    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTexture(renderer, *img_ptr1, nullptr, &dst1);
    SDL_SetRenderScale(renderer, scale*scale2, scale*scale2);
    SDL_RenderTexture(renderer, *img_ptr2, nullptr, &dst2);
}

void display_strip(SDL_Texture **img_ptr1, SDL_Texture **img_ptr2)
{
    SDL_FRect dst1, dst2;
    SDL_GetTextureSize(*img_ptr1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(*img_ptr2, &dst2.w, &dst2.h);

    float scale = zoom * width / dst1.w;
    float scale2 = dst1.w / dst2.w;

    dst1.x = (width / scale - dst1.w) / 2;
    dst1.y = -scrolled;
    dst2.x = (width / (scale*scale2) - dst2.w) / 2;
    dst2.y = (-scrolled + dst1.h) / scale2;

    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTexture(renderer, *img_ptr1, nullptr, &dst1);
    SDL_SetRenderScale(renderer, scale*scale2, scale*scale2);
    SDL_RenderTexture(renderer, *img_ptr2, nullptr, &dst2);
}

void display_progress()
{
    int w, h;
    TTF_GetTextSize(progress_text, &w, &h);

    int x = (width - w) / 2;
    int y = (height - h) / 2;

    int mx = 0.3 * TTF_GetFontSize(progress_font);
    int my = 0.1 * h;

    SDL_SetRenderScale(renderer, 1, 1);
    SDL_SetRenderDrawColorFloat(renderer, 0, 0, 0, 0.7);
    SDL_RenderFillRect(renderer, &(SDL_FRect){x - mx, y - my, w + 2*mx, h + 2*my});
    TTF_DrawRendererText(progress_text, x, y);
}
