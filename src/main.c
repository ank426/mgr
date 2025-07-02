#define STB_DS_IMPLEMENTATION
#include "headers.h"
#include "globals.h"

#define SDL_MAIN_USE_CALLBACKS 1
#include <SDL3/SDL_main.h>

void calculate_progress()
{
    char string[32];

    switch (mode) {
    case SINGLE:
        snprintf(string, 32, "%d/%d", current_page+1, total_pages);
        break;

    case BOOK:
        if (image1 == nullptr || image2 == nullptr)
            snprintf(string, 32, "%d/%d", current_page+1, total_pages);
        else
            snprintf(string, 32, "%d-%d/%d", current_page+1, current_page+2, total_pages);
        break;

    case STRIP:
        if (pages[current_page].height - scrolled > height * pages[current_page].width / zoom / width)
            snprintf(string, 32, "%d/%d", current_page+1, total_pages);
        else
            snprintf(string, 32, "%d-%d/%d", current_page+1, current_page+2, total_pages);
        break;
    }

    TTF_SetTextString(progress_text, string, 32);
}

SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{
    get_args(argc, argv);
    if (process_args())
        return SDL_APP_SUCCESS;

    assert(SDL_CreateWindowAndRenderer("mgr", 0, 0, SDL_WINDOW_RESIZABLE, &window, &renderer));
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);

    assert(TTF_Init());
    engine = TTF_CreateRendererTextEngine(renderer);
    assert(engine != nullptr);
    progress_font = TTF_OpenFont("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf", 50);
    assert(progress_font != nullptr);
    progress_text = TTF_CreateText(engine, progress_font, "", 0);
    assert(progress_text != nullptr);

    load_chapter();
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

    switch (mode) {
    case SINGLE:
        display_single(&image1);
        break;

    case BOOK:
        if (image1 == nullptr)
            display_single(&image2);
        else if (image2 == nullptr)
            display_single(&image1);
        else
            display_book(&image2, &image1);
        break;

    case STRIP:
        display_strip(&image1, &image2);
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
    SDL_DestroyTexture(image1);
    SDL_DestroyTexture(image2);

    TTF_DestroyRendererTextEngine(engine);
    TTF_CloseFont(progress_font);
    TTF_DestroyText(progress_text);
    TTF_Quit();

    free(pages);
    free(intervals);

    for (int i = 0; i < arrlen(files); i++)
        free(files[i]);
    arrfree(files);
}
