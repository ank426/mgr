#include "headers.h"

#define MIN(a, b) (a) > (b) ? (b) : (a)
#define MAX(a, b) (a) < (b) ? (b) : (a)

extern char *const dirpath;
extern char **const files;
extern const int width, height;

static const int load_before = 2;
static const int load_after = 4;
static SDL_Texture **loaded = nullptr;
static int lstart = 0;

SDL_Texture *load_img(int idx, struct appstate *s)
{
    char *relpath = files[s->file];
    char *path = malloc(strlen(dirpath) + 1 + strlen(relpath) + 1);
    sprintf(path, "%s/%s", dirpath, relpath);
    SDL_Texture *image = load_image_from_zip(s->pages[idx].name, path);
    free(path);
    return image;
}

void update_images(struct appstate *s)
{
    int start = MAX(s->start - load_before, 0);
    int end = MIN(s->end + load_after, arrlen(s->pages) - 1);

    if (arrlen(loaded) == 0 || end < lstart || lstart + arrlen(loaded) <= start) {
        arrfree(loaded);
        lstart = start;
        while (lstart + arrlen(loaded) <= end)
            arrput(loaded, load_img(lstart + arrlen(loaded), s));
    }

    else {
        while (end < lstart + arrlen(loaded) - 1) {
            SDL_DestroyTexture(arrlast(loaded));
            arrpop(loaded);
        }

        while (lstart < start) {
            SDL_DestroyTexture(loaded[0]);
            arrdel(loaded, 0);
            lstart++;
        }

        while (start < lstart)
            arrins(loaded, 0, load_img(--lstart, s));

        while (lstart + arrlen(loaded) <= end)
            arrput(loaded, load_img(lstart + arrlen(loaded), s));
    }

    arrfree(s->images);
    for (int i = s->start; i <= s->end; i++)
        arrput(s->images, loaded[i-lstart]);
}
