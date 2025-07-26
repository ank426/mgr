#include "headers.h"

extern char *const dirpath;
extern char **const files;
extern const int width, height;

SDL_Texture *load_img(int idx, struct appstate *s)
{
    char *relpath = files[s->file];
    char *path = malloc(strlen(dirpath) + 1 + strlen(relpath) + 1);
    sprintf(path, "%s/%s", dirpath, relpath);
    return load_image_from_zip(s->pages[idx].name, path);
}

void fix_page(struct appstate *s)
{
    s->end = s->start;

    switch (s->mode) {
    case SINGLE:
        break;

    case BOOK:
        if (s->pages[s->start].wide)
            break;
        struct interval *curr_int = get_interval(s->start);
        if (s->start & 1 ^ curr_int->start & 1 ^ !curr_int->offset) {
            if (s->start + 1 < curr_int->end)
                s->end++;
        } else {
            if (s->start - 1 >= curr_int->start)
                s->start--;
        }
        break;

    case STRIP:
        float d = !s->rotated ? height / s->zoom / width : width / s->hzoom / height;
        d += s->scroll / s->pages[s->end].width;
        while (d > 0 && s->end < arrlen(s->pages)) {
            d -= s->pages[s->end].height / s->pages[s->end].width;
            s->end++;
        }
        s->end--;
        break;
    }
}

void update_images(struct appstate *s)
{
    for (int i = 0; i < arrlen(s->images); i++)
        SDL_DestroyTexture(s->images[i]);
    arrfree(s->images);
    for (int i = s->start; i <= s->end; i++)
        arrput(s->images, load_img(i, s));
}
