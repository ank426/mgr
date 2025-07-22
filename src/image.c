#include "headers.h"

SDL_Texture *image1 = nullptr;
SDL_Texture *image2 = nullptr;
SDL_Texture *image3 = nullptr;

extern char *dirpath;

int num_images()
{
    return (image1 != nullptr) + (image2 != nullptr) + (image3 != nullptr);
}

SDL_Texture *load_img(int idx, const char *relpath, struct appstate *s)
{
    char *path = malloc(strlen(dirpath) + 1 + strlen(relpath) + 1);
    sprintf(path, "%s/%s", dirpath, relpath);
    return load_image_from_zip(s->pages[idx].name, path);
}

void load_images(const char *const path, struct appstate *s)
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
    SDL_DestroyTexture(image3);
    image1 = image2 = image3 = nullptr;

    switch (s->mode) {
    case SINGLE:
        image1 = load_img(s->page, path, s);
        break;

    case BOOK:
        if (s->pages[s->page].wide) {
            image1 = load_img(s->page, path, s);
            break;
        }
        struct interval *curr_int = get_interval(s->page);
        if (s->page & 1 ^ curr_int->start & 1 ^ !curr_int->offset) {
            image1 = load_img(s->page, path, s);
            if (s->page + 1 < curr_int->end)
                image2 = load_img(s->page + 1, path, s);
        } else {
            image2 = load_img(s->page, path, s);
            if (s->page - 1 >= curr_int->start)
                image1 = load_img(--s->page, path, s);
        }
        break;

    case STRIP:
        image1 = load_img(s->page, path, s);
        if (s->page < arrlen(s->pages) - 1)
            image2 = load_img(s->page + 1, path, s);
        if (s->page < arrlen(s->pages) - 2)
            image3 = load_img(s->page + 2, path, s);
        break;
    }
}

void load_images_next(const char *const path, struct appstate *s)
{
    SDL_DestroyTexture(image1);
    image1 = image2;
    image2 = image3;
    if (s->page < arrlen(s->pages) - 2)
        image3 = load_img(s->page + 2, path, s);
    else
        image3 = nullptr;
}

void load_images_prev(const char *const path, struct appstate *s)
{
    SDL_DestroyTexture(image3);
    image3 = image2;
    image2 = image1;
    image1 = load_img(s->page, path, s);
}

void free_images()
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
    SDL_DestroyTexture(image3);
}
