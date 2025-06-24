#include "globals.h"

#include "display.h"
#include "interval.h"
#include "keybinds.h"
#include "keybinds_impl.h"
#include "load.h"
#include "zip_handler.h"

#define SDL_MAIN_USE_CALLBACKS 1

#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3_image/SDL_image.h>

#include <stdbool.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>


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
    if (event->type == SDL_EVENT_QUIT)
        return SDL_APP_SUCCESS;

    if (event->type == SDL_EVENT_KEY_DOWN) {
        struct bind *binds;
        int n_binds;
        switch(mode) {
            case SINGLE:
                binds = single_binds;
                n_binds = n_single_binds;
                break;
            case BOOK:
                binds = book_binds;
                n_binds = n_book_binds;
                break;
            case STRIP:
                binds = strip_binds;
                n_binds = n_strip_binds;
                break;
        }

        for (int i = 0; i < n_binds; i++)
            if (event->key.mod == binds[i].mod && event->key.scancode == binds[i].key)
                binds[i].fn(binds[i].args);
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
