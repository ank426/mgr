#define STB_DS_IMPLEMENTATION
#include "headers.h"
#include "structs.h"

#define SDL_MAIN_USE_CALLBACKS 1
#include <SDL3/SDL_main.h>


SDL_Window *window = nullptr;
SDL_Renderer *renderer = nullptr;
int width = 0, height = 0;

char **files = nullptr;
int curr_file = 0;

struct page *pages = nullptr;
int curr_page = 0;

enum modes mode = SINGLE;
float scrolled = 0;
float zoom = 0.5;
bool show_progress = false;


SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{
    if (cli(argc, argv))
        return SDL_APP_SUCCESS;

    assert(SDL_CreateWindowAndRenderer("mgr", 0, 0, SDL_WINDOW_RESIZABLE, &window, &renderer));
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);

    init_text();

    load_chapter(files[curr_file]);
    load_images(files[curr_file]);

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppEvent(void *appstate, SDL_Event *event)
{
    if (event->type == SDL_EVENT_QUIT)
        return SDL_APP_SUCCESS;

    if (event->type == SDL_EVENT_KEY_DOWN) {
        struct bind *binds;
        int n_binds;
        switch (mode) {
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

    if (event->type == SDL_EVENT_FINGER_UP) {
        if (mode == SINGLE)
            page(event->tfinger.y > 0.5 ? "next" : "prev");
        else if (mode == BOOK)
            flip(event->tfinger.x < 0.5 ? "next" : "prev");
    }

    if (event->type == SDL_EVENT_FINGER_MOTION) {
        if (mode == STRIP) {
            char buffer[9];
            snprintf(buffer, sizeof(buffer), "%f", -1.6 * event->tfinger.dy);
            scroll(buffer);
        }
    }

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppIterate(void *appstate)
{
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_GetRenderOutputSize(renderer, &width, &height);

    extern SDL_Texture *image1, *image2;

    switch (mode) {
    case SINGLE:
        display_single(image1);
        break;

    case BOOK:
        if (image1 == nullptr)
            display_single(image2);
        else if (image2 == nullptr)
            display_single(image1);
        else
            display_book(image2, image1);
        break;

    case STRIP:
        display_strip(image1, image2);
        break;
    }

    if (show_progress) {
        calculate_progress();
        display_progress();
    }

    SDL_RenderPresent(renderer);
    return SDL_APP_CONTINUE;
}

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{
    free_images();
    free_intervals();
    free_text();

    for (int i = 0; i < arrlen(pages); i++)
        free(pages[i].name);
    arrfree(pages);

    for (int i = 0; i < arrlen(files); i++)
        free(files[i]);
    arrfree(files);
}
