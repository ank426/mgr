#include "headers.h"
#include "globals.h"

void load_images()
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
    image1 = image2 = NULL;

    switch (mode) {
        case SINGLE:
            image1 = load_image_from_zip(current_page);
            break;

        case BOOK:
            if (dims[current_page].wide) {
                image1 = load_image_from_zip(current_page);
                break;
            }
            struct interval *curr_int = get_current_interval();
            if (current_page ^ curr_int->start ^ ~curr_int->offset & 1) {
                image1 = load_image_from_zip(current_page);
                if (current_page + 1 < curr_int->end)
                    image2 = load_image_from_zip(current_page + 1);
            } else {
                image2 = load_image_from_zip(current_page);
                if (current_page - 1 >= curr_int->start)
                    image1 = load_image_from_zip(--current_page);
            }
            break;

        case STRIP:
            image1 = load_image_from_zip(current_page);
            if (current_page < total_pages - 1)
                image2 = load_image_from_zip(current_page + 1);
            break;
    }
}

void load_images_next()
{
    SDL_DestroyTexture(image1);
    image1 = image2;
    if (current_page < total_pages - 1)
        image2 = load_image_from_zip(current_page + 1);
    else
        image2 = NULL;
}

void load_images_prev()
{
    SDL_DestroyTexture(image2);
    image2 = image1;
    image1 = load_image_from_zip(current_page);
}
