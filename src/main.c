#include "globals.h"
#include "display.h"
#include "interval.h"
#include "zip_handler.h"

#define SDL_MAIN_USE_CALLBACKS 1

#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3_image/SDL_image.h>

#include <stdbool.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>

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
            if (current_page ^ curr_int->start ^ !curr_int->offset & 1) {
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
    assert(mode == STRIP);
    SDL_DestroyTexture(image1);
    image1 = image2;
    if (current_page < total_pages - 1)
        image2 = load_image_from_zip(current_page + 1);
    else
        image2 = NULL;
}

void load_images_prev()
{
    assert(mode == STRIP);
    SDL_DestroyTexture(image2);
    image2 = image1;
    image1 = load_image_from_zip(current_page);
}


SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{
    if (!SDL_CreateWindowAndRenderer("mgr", 0, 0, SDL_WINDOW_RESIZABLE, &window, &renderer)) {
        SDL_Log("Couldn't create window and renderer: %s\n", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    strncpy(path, argv[1], 256);
    total_pages = get_num_entries_from_zip();
    update_dims_from_zip();
    update_intervals();
    load_images();

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppEvent(void *appstate, SDL_Event *event)
{
    if (event->type == SDL_EVENT_QUIT) return SDL_APP_SUCCESS;

    if (event->type == SDL_EVENT_KEY_DOWN) {
        switch (event->key.scancode) {
            case SDL_SCANCODE_Q:
                return SDL_APP_SUCCESS;

            case SDL_SCANCODE_M:
                mode = (mode + 1) % 3;
                load_images();
                break;

            case SDL_SCANCODE_O:
                get_current_interval()->offset ^= 1;
                load_images();
                break;

            case SDL_SCANCODE_J:
                if (current_page == total_pages - 1) break;
                current_page += 2 - (mode == STRIP || image1 == NULL || image2 == NULL || current_page == total_pages-2);
                load_images();
                break;

            case SDL_SCANCODE_K:
                if (current_page == 0) break;
                current_page--;
                load_images();
                break;

            case SDL_SCANCODE_D:
                if (mode != STRIP) break;

                scroll += 0.5 * height / scale;
                if (scroll < dims[current_page].height) break;

                if (current_page == total_pages - 1)
                    scroll = dims[current_page].height;
                else {
                    scroll -= dims[current_page++].height;
                    load_images_next();
                }
                break;

            case SDL_SCANCODE_U:
                if (mode != STRIP) break;

                scroll -= 0.5 * height / scale;
                if (scroll > 0) break;

                if (current_page == 0)
                    scroll = 0;
                else {
                    scroll += dims[--current_page].height;
                    load_images_prev();
                }
                break;

            default:
                break;
        }
    }

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppIterate(void *appstate)
{
    SDL_GetRenderOutputSize(renderer, &width, &height);
    switch (mode) {
        case SINGLE:
            display_single(&image1);
            break;

        case BOOK:
            if (image1 == NULL)
                display_single(&image2);
            else if (image2 == NULL)
                display_single(&image1);
            else
                display_book(&image2, &image1);
            break;

        case STRIP:
            display_strip(&image1, &image2);
            break;

    }

    return SDL_APP_CONTINUE;
}

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
    free(dims);
    free(intervals);
}
