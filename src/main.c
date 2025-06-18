#include "zip_handler.h"
#include "display.h"

#define SDL_MAIN_USE_CALLBACKS 1

#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3_image/SDL_image.h>

#include <stdbool.h>
#include <string.h>

SDL_Window *window = NULL;
SDL_Renderer *renderer = NULL;

char path[255];
int current_page = 0;
int total_pages = 0;
SDL_Texture *image1 = NULL;
SDL_Texture *image2 = NULL;

enum mode {
    SINGLE,
    BOOK,
    STRIP,
} mode = SINGLE;


void load_images()
{
    if (current_page != -1)
        image1 = load_image_from_zip(path, current_page, renderer);
    else
        image1 = NULL;

    if (current_page != total_pages - 1)
        image2 = load_image_from_zip(path, current_page+1, renderer);
    else
        image2 = NULL;
}

SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{
    if (!SDL_CreateWindowAndRenderer("mgr", 0, 0, SDL_WINDOW_RESIZABLE, &window, &renderer)) {
        SDL_Log("Couldn't create window and renderer: %s\n", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    strncpy(path, argv[1], 255);
    total_pages = get_num_entries_from_zip(path);
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
                break;

            case SDL_SCANCODE_J:
                switch (mode) {
                    case SINGLE:
                        if (current_page < total_pages - 1) {
                            current_page++;
                            load_images();
                        }
                        break;
                    case BOOK:
                        if (current_page < total_pages - 2) {
                            current_page += 2;
                            load_images();
                        }
                        break;
                    case STRIP:
                        break;
                }
                break;

            case SDL_SCANCODE_K:
                switch (mode) {
                    case SINGLE:
                        if (current_page > 0) {
                            current_page--;
                            load_images();
                        }
                        break;
                    case BOOK:
                        if (current_page > 0) {
                            current_page -= 2;
                            load_images();
                        }
                        break;
                    case STRIP:
                        break;
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
    switch (mode) {
        case SINGLE:
            display_single(&image1, renderer);
            break;

        case BOOK:
            if (current_page == -1)
                display_single(&image2, renderer);
            else if (current_page == total_pages - 1)
                display_single(&image1, renderer);
            else
                display_book(&image2, &image1, renderer);
            break;

        case STRIP:
            break;
    }

    return SDL_APP_CONTINUE;
}

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{

}
