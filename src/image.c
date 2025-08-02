#include "headers.h"

extern SDL_Renderer *renderer;
extern char *const dirpath;
extern char **const files;

static const int load_before = 2;
static const int load_after = 4;
static SDL_Surface **surfaces = nullptr;
static int surf_start = 0;

static SDL_Texture **textures = nullptr;
static int tex_start = 0;

SDL_Surface *load_img(int idx, struct appstate *s)
{
    char *relpath = files[s->file];
    char *path = malloc(strlen(dirpath) + 1 + strlen(relpath) + 1);
    sprintf(path, "%s/%s", dirpath, relpath);
    SDL_Surface *image = load_image_from_zip(s->pages[idx].name, path);
    free(path);
    return image;
}

void update_surfaces(struct appstate *s)
{
    const int start = fmaxf(s->start - load_before, 0);
    const int end = fminf(s->end + load_after, arrlen(s->pages) - 1);

    while (arrlen(surfaces) > 0 && end < surf_start + arrlen(surfaces) - 1) {
        SDL_DestroySurface(arrlast(surfaces));
        arrpop(surfaces);
    }

    while (arrlen(surfaces) > 0 && surf_start < start) {
        SDL_DestroySurface(surfaces[0]);
        arrdel(surfaces, 0);
        surf_start++;
    }

    if (arrlen(surfaces) == 0)
        surf_start = start;

    while (start < surf_start)
        arrins(surfaces, 0, load_img(--surf_start, s));

    while (surf_start + arrlen(surfaces) <= end)
        arrput(surfaces, load_img(surf_start + arrlen(surfaces), s));
}

void update_textures(int start, int end)
{
    while (arrlen(textures) > 0 && end < tex_start + arrlen(textures) - 1) {
        SDL_DestroyTexture(arrlast(textures));
        arrpop(textures);
    }

    while (arrlen(textures) > 0 && tex_start < start) {
        SDL_DestroyTexture(textures[0]);
        arrdel(textures, 0);
        tex_start++;
    }

    if (arrlen(textures) == 0)
        tex_start = start;

    while (start < tex_start)
        arrins(textures, 0, SDL_CreateTextureFromSurface(renderer, surfaces[--tex_start - surf_start]));

    while (tex_start + arrlen(textures) <= end)
        arrput(textures, SDL_CreateTextureFromSurface(renderer, surfaces[tex_start + arrlen(textures) - surf_start]));
}

SDL_Texture **get_images(struct appstate *s)
{
    update_surfaces(s);
    update_textures(s->start, s->end);
    return textures;
}

void free_surfaces()
{
    for (int i = 0; i < arrlen(surfaces); i++)
        SDL_DestroySurface(surfaces[i]);
    arrfree(surfaces);
}

void free_textures()
{
    for (int i = 0; i < arrlen(textures); i++)
        SDL_DestroyTexture(textures[i]);
    arrfree(textures);
}
