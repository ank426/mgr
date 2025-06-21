#include "zip_handler.c"
#include "display.c"

#define SDL_MAIN_USE_CALLBACKS 1

#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3_image/SDL_image.h>

#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

SDL_Window *window = NULL;
SDL_Renderer *renderer = NULL;

char path[256];
int current_page = 0;
int total_pages = 0;
SDL_Texture *image1 = NULL;
SDL_Texture *image2 = NULL;

bool book_mode = false;

bool *file_wides = NULL;

int n_int = 0;
struct interval {
    int start, end;
    bool offset;
} *intervals, *current_interval;


void update_intervals()
{
    current_interval = NULL;
    if (intervals != NULL) free(intervals);
    intervals = malloc(total_pages * sizeof(struct interval));

    n_int = 0;
    int s = 0;
    bool in = false;
    for (int i = 0; i < total_pages; i++) {
        if (!in && !file_wides[i]) {
            s = i;
            in = true;
        }
        if (in && file_wides[i]) {
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

    assert(file_wides[current_page] || current_interval->start <= current_page && current_page < current_interval->end);

    if (!book_mode || file_wides[current_page]) {
        image1 = load_image_from_zip(path, current_page, renderer);
    }
    else {
        if ((current_page - current_interval->start) % 2 == current_interval->offset) {
            image1 = load_image_from_zip(path, current_page, renderer);
            if (current_page + 1 < current_interval->end)
                image2 = load_image_from_zip(path, current_page+1, renderer);
        } else {
            image2 = load_image_from_zip(path, current_page, renderer);
            if (current_page - 1 >= current_interval->start)
                image1 = load_image_from_zip(path, --current_page, renderer);
        }
    }
}


SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{
    if (!SDL_CreateWindowAndRenderer("mgr", 0, 0, SDL_WINDOW_RESIZABLE, &window, &renderer)) {
        SDL_Log("Couldn't create window and renderer: %s\n", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    strncpy(path, argv[1], 256);
    total_pages = get_num_entries_from_zip(path);
    update_wides_from_zip(path, total_pages, &file_wides);
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
                book_mode = !book_mode;
                load_images();
                break;

            case SDL_SCANCODE_O:
                current_interval->offset = !current_interval->offset;
                load_images();
                break;

            case SDL_SCANCODE_J:
                if (current_page == total_pages-1) break;

                current_page += 2 - (image1 == NULL || image2 == NULL || current_page == total_pages-2);

                if (!file_wides[current_page] && current_page >= current_interval->end)
                    current_interval++;

                load_images();
                break;

            case SDL_SCANCODE_K:
                if (current_page == 0) break;

                current_page--;

                if (!file_wides[current_page] && current_page < current_interval->start)
                    current_interval--;

                load_images();
                break;

            default:
                break;
        }
    }

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppIterate(void *appstate)
{
    if (image1 == NULL && image2 == NULL)
        return SDL_APP_FAILURE;
    else if (image1 == NULL)
        display_single(&image2, renderer);
    else if (image2 == NULL)
        display_single(&image1, renderer);
    else
        display_book(&image2, &image1, renderer);

    return SDL_APP_CONTINUE;
}

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);
    free(file_wides);
    free(intervals);
}
