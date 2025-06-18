#include "display.h"

#include <SDL3/SDL.h>
#include <SDL3_image/SDL_image.h>

void display_single(SDL_Texture **img_ptr, SDL_Renderer *renderer)
{
    int w = 0, h = 0;
    SDL_GetRenderOutputSize(renderer, &w, &h);

    SDL_FRect dst;
    SDL_GetTextureSize(*img_ptr, &dst.w, &dst.h);

    float sw = w / dst.w;
    float sh = h / dst.h;
    float scale = sw < sh ? sw : sh;
    dst.x = (w / scale - dst.w) / 2;
    dst.y = (h / scale - dst.h) / 2;

    SDL_SetRenderScale(renderer, scale, scale);
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_RenderTexture(renderer, *img_ptr, NULL, &dst);
    SDL_RenderPresent(renderer);
}

void display_book(SDL_Texture **img_ptr1, SDL_Texture **img_ptr2, SDL_Renderer *renderer)
{
    int w = 0, h = 0;
    SDL_GetRenderOutputSize(renderer, &w, &h);

    SDL_FRect dst1, dst2;
    SDL_GetTextureSize(*img_ptr1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(*img_ptr2, &dst2.w, &dst2.h);

    float scale2 = dst1.h / dst2.h;

    float sw = w / (dst1.w + scale2*dst2.w);
    float sh = h / dst1.h;
    float scale = sw < sh ? sw : sh;

    dst1.x = (w / scale - dst1.w - scale2*dst2.w) / 2;
    dst1.y = (h / scale - dst1.h) / 2;
    dst2.x = (w / (scale*scale2) + dst1.w - scale2*dst2.w) / 2;
    dst2.y = (h / (scale*scale2) - dst2.h) / 2;

    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTexture(renderer, *img_ptr1, NULL, &dst1);
    SDL_SetRenderScale(renderer, scale*scale2, scale*scale2);
    SDL_RenderTexture(renderer, *img_ptr2, NULL, &dst2);
    SDL_RenderPresent(renderer);
}

void display_strip(SDL_Texture **img_ptr1, SDL_Texture **img_ptr2, SDL_Renderer *renderer)
{

}
