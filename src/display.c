#include "display.h"

#include <SDL3/SDL.h>
#include <SDL3_image/SDL_image.h>

#include <stdio.h>

extern SDL_Renderer *renderer;
extern int width, height;
extern float scroll, scale;

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
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_RenderTexture(renderer, *img_ptr, NULL, &dst);
    SDL_RenderPresent(renderer);
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

    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTexture(renderer, *img_ptr1, NULL, &dst1);
    SDL_SetRenderScale(renderer, scale*scale2, scale*scale2);
    SDL_RenderTexture(renderer, *img_ptr2, NULL, &dst2);
    SDL_RenderPresent(renderer);
}

void display_strip(SDL_Texture **img_ptr1, SDL_Texture **img_ptr2)
{
    SDL_FRect dst1, dst2;
    SDL_GetTextureSize(*img_ptr1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(*img_ptr2, &dst2.w, &dst2.h);

    float scale2 = dst1.w / dst2.w;

    scale = 0.5 * width / dst1.w;

    dst1.x = (width / scale - dst1.w) / 2;
    dst1.y = -scroll;
    dst2.x = (width / (scale*scale2) - dst2.w) / 2;
    dst2.y = (-scroll + dst1.h) / scale2;

    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTexture(renderer, *img_ptr1, NULL, &dst1);
    SDL_SetRenderScale(renderer, scale*scale2, scale*scale2);
    SDL_RenderTexture(renderer, *img_ptr2, NULL, &dst2);
    SDL_RenderPresent(renderer);
}
