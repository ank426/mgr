#include "headers.h"

extern SDL_Renderer *renderer;
extern char *const dirpath;
extern char **const files;

static bool *blank = nullptr;
static SDL_Texture **textures = nullptr;
static int tex_start = 0;

SDL_Texture *make_texture(SDL_Surface *surf, int width, int height)
{
    SDL_Texture *ret = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_RGBA32, SDL_TEXTUREACCESS_TARGET, width, height);
    SDL_Texture *target = SDL_GetRenderTarget(renderer);
    SDL_SetRenderTarget(renderer, ret);
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_SetRenderTarget(renderer, target);
    return ret;
}

void update_textures(int start, int end, struct page *pages)
{
    while (arrlen(textures) > 0 && end < tex_start + arrlen(textures) - 1) {
        SDL_DestroyTexture(arrlast(textures));
        arrpop(textures);
        arrpop(blank);
    }

    while (arrlen(textures) > 0 && tex_start < start) {
        SDL_DestroyTexture(textures[0]);
        arrdel(textures, 0);
        arrdel(blank, 0);
        tex_start++;
    }

    if (arrlen(textures) == 0)
        tex_start = start;

    for (int i = 0; i < arrlen(textures); i++) {
        if (!blank[i])
            continue;
        SDL_Surface *surf = get_surface(tex_start + i);
        if (surf != nullptr) {
            SDL_DestroyTexture(textures[i]);
            textures[i] = SDL_CreateTextureFromSurface(renderer, surf);
            blank[i] = false;
        }
    }

    while (start < tex_start) {
        SDL_Surface *surf = get_surface(--tex_start);
        if (surf != nullptr) {
            arrins(textures, 0, SDL_CreateTextureFromSurface(renderer, surf));
            arrins(blank, 0, false);
        } else {
            arrins(textures, 0, make_texture(surf, pages[tex_start].width, pages[tex_start].height));
            arrins(blank, 0, true);
        }
    }

    while (tex_start + arrlen(textures) <= end) {
        SDL_Surface *surf = get_surface(tex_start + arrlen(textures));
        if (surf != nullptr) {
            arrput(textures, SDL_CreateTextureFromSurface(renderer, surf));
            arrput(blank, false);
        } else {
            arrput(textures, make_texture(surf, pages[tex_start + arrlen(textures)].width, pages[tex_start + arrlen(textures)].height));
            arrput(blank, true);
        }
    }
}

SDL_Texture **get_images(struct appstate *s)
{
    update_surfaces(s);
    update_textures(s->start, s->end, s->pages);
    return textures;
}

void free_textures()
{
    for (int i = 0; i < arrlen(textures); i++)
        SDL_DestroyTexture(textures[i]);
    arrfree(textures);
    arrfree(blank);
}
