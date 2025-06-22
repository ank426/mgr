#include "zip_handler.h"
#include "display.h"

#define SDL_MAIN_USE_CALLBACKS 1

#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3_image/SDL_image.h>

#include <stdbool.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>

SDL_Window *window = NULL;
SDL_Renderer *renderer = NULL;

int width = 0;
int height = 0;

char path[256];
int current_page = 0;
int total_pages = 0;
SDL_Texture *image1 = NULL;
SDL_Texture *image2 = NULL;

enum {
    SINGLE,
    BOOK,
    STRIP,
} mode = SINGLE;

int (*dims)[2] = NULL;
bool *wides = NULL;

int n_int = 0;
struct interval {
    int start, end;
    bool offset;
} *intervals = NULL, *current_interval = NULL;

float scroll = 0;
float scale = 0;


void update_intervals()
{
    current_interval = NULL;
    if (intervals != NULL) free(intervals);
    intervals = malloc(total_pages * sizeof(struct interval));

    n_int = 0;
    int s = 0;
    bool in = false;
    for (int i = 0; i < total_pages; i++) {
        if (!in && !wides[i]) {
            s = i;
            in = true;
        }
        if (in && wides[i]) {
            intervals[n_int++] = (struct interval) { s, i, false };
            in = false;
        }
    }
    if (in)
        intervals[n_int++] = (struct interval) { s, total_pages, false };

    intervals[0].offset = (intervals[0].end - intervals[0].start) % 2;

    intervals = realloc(intervals, n_int * sizeof(struct interval));
}

void load_images()
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
    image1 = image2 = NULL;

    assert(wides[current_page] || current_interval->start <= current_page && current_page < current_interval->end);

    switch (mode) {
        case SINGLE:
            image1 = load_image_from_zip(path, current_page, renderer);
            break;

        case BOOK:
            if ((current_page - current_interval->start) % 2 == current_interval->offset) {
                image1 = load_image_from_zip(path, current_page, renderer);
                if (current_page + 1 < current_interval->end)
                    image2 = load_image_from_zip(path, current_page+1, renderer);
            } else {
                image2 = load_image_from_zip(path, current_page, renderer);
                if (current_page - 1 >= current_interval->start)
                    image1 = load_image_from_zip(path, --current_page, renderer);
            }
            break;

        case STRIP:
            image1 = load_image_from_zip(path, current_page, renderer);
            if (current_page < total_pages - 1)
                image2 = load_image_from_zip(path, current_page+1, renderer);
            break;
    }
}

void load_images_next()
{
    assert(mode == STRIP);
    SDL_DestroyTexture(image1);
    image1 = image2;
    if (current_page < total_pages - 1)
        image2 = load_image_from_zip(path, current_page+1, renderer);
    else
        image2 = NULL;
}

void load_images_prev()
{
    assert(mode == STRIP);
    SDL_DestroyTexture(image2);
    image2 = image1;
    image1 = load_image_from_zip(path, current_page, renderer);
}


SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{
    if (!SDL_CreateWindowAndRenderer("mgr", 0, 0, SDL_WINDOW_RESIZABLE, &window, &renderer)) {
        SDL_Log("Couldn't create window and renderer: %s\n", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    strncpy(path, argv[1], 256);
    total_pages = get_num_entries_from_zip(path);
    update_dims_from_zip(path, total_pages);
    update_intervals();
    current_interval = intervals;
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
                current_interval->offset = !current_interval->offset;
                load_images();
                break;

            case SDL_SCANCODE_J:
                if (current_page == total_pages-1) break;

                current_page += 2 - (mode == STRIP || image1 == NULL || image2 == NULL || current_page == total_pages-2);

                if (!wides[current_page] && current_page >= current_interval->end)
                    current_interval++;

                load_images();
                break;

            case SDL_SCANCODE_K:
                if (current_page == 0) break;

                current_page--;

                if (!wides[current_page] && current_page < current_interval->start)
                    current_interval--;

                load_images();
                break;

            case SDL_SCANCODE_D:
                if (mode != STRIP) break;

                scroll += 0.5 * height / scale;
                if (scroll >= dims[current_page][1]) {
                    if (current_page == total_pages-1)
                        scroll = dims[current_page][1];
                    else {
                        scroll -= dims[current_page++][1];

                        if (!wides[current_page] && current_page >= current_interval->end)
                            current_interval++;

                        load_images_next();
                    }
                }
                break;

            case SDL_SCANCODE_U:
                if (mode != STRIP) break;

                scroll -= 0.5 * height / scale;
                if (scroll < 0) {
                    if (current_page == 0)
                        scroll = 0;
                    else {
                        scroll += dims[--current_page][1];

                        if (!wides[current_page] && current_page >= current_interval->end)
                            current_interval++;

                        load_images_prev();
                    }
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
    free(wides);
    free(intervals);
}
