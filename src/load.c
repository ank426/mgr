#include "headers.h"
#include "structs.h"

extern const struct page *const pages;
extern int curr_page;
extern const enum modes mode;

SDL_Texture *image1 = nullptr;
SDL_Texture *image2 = nullptr;


int num_images()
{
    return (image1 != nullptr) + (image2 != nullptr);
}

void load_images(const char *const path)
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
    image1 = image2 = nullptr;

    switch (mode) {
    case SINGLE:
        image1 = load_image_from_zip(curr_page, path);
        break;

    case BOOK:
        if (pages[curr_page].wide) {
            image1 = load_image_from_zip(curr_page, path);
            break;
        }
        struct interval *curr_int = get_interval(curr_page);
        if (curr_page & 1 ^ curr_int->start & 1 ^ !curr_int->offset) {
            image1 = load_image_from_zip(curr_page, path);
            if (curr_page + 1 < curr_int->end)
                image2 = load_image_from_zip(curr_page + 1, path);
        } else {
            image2 = load_image_from_zip(curr_page, path);
            if (curr_page - 1 >= curr_int->start)
                image1 = load_image_from_zip(--curr_page, path);
        }
        break;

    case STRIP:
        image1 = load_image_from_zip(curr_page, path);
        if (curr_page < arrlen(pages) - 1)
            image2 = load_image_from_zip(curr_page + 1, path);
        break;
    }
}

void load_images_next(const char *const path)
{
    SDL_DestroyTexture(image1);
    image1 = image2;
    if (curr_page < arrlen(pages) - 1)
        image2 = load_image_from_zip(curr_page + 1, path);
    else
        image2 = nullptr;
}

void load_images_prev(const char *const path)
{
    SDL_DestroyTexture(image2);
    image2 = image1;
    image1 = load_image_from_zip(curr_page, path);
}

void free_images()
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
}
