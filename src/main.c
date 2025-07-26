#define STB_DS_IMPLEMENTATION
#include "headers.h"

#define SDL_MAIN_USE_CALLBACKS 1
#include <SDL3/SDL_main.h>


SDL_Window *window = nullptr;
SDL_Renderer *renderer = nullptr;
int width = 0, height = 0;

char *dirpath = nullptr;
char **files = nullptr;
bool readlist = false;
struct config conf;


void init_conf()
{
    conf = (struct config){
        .automode = true,
        .mode = SINGLE,
        .zoom = 0.45,
        .hzoom = 0.7,
        .start_fullscreen = true,
    };
}

void init_state(struct appstate *s)
{
    *s = (struct appstate){
        .file = 0,
        .pages = nullptr,
        .start = 0,
        .end = 0,
        .images = nullptr,
        .automode = conf.automode,
        .mode = conf.mode,
        .scroll = 0.0,
        .rotated = false,
        .zoom = conf.zoom,
        .hzoom = conf.hzoom,
        .show_progress = false,
    };
}

SDL_AppResult SDL_AppInit(void **ptr_appstate, int argc, char *argv[])
{
    struct appstate *s = *ptr_appstate = malloc(sizeof(struct appstate));

    init_conf();
    get_args(argc, argv);
    init_state(s);
    process_args(s);

    SDL_WindowFlags flags = SDL_WINDOW_RESIZABLE | (conf.start_fullscreen ? SDL_WINDOW_FULLSCREEN : 0);
    assert(SDL_CreateWindowAndRenderer("mgr", 0, 0, flags, &window, &renderer));
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);
    SDL_GetRenderOutputSize(renderer, &width, &height);

    init_text();

    load_file(files[s->file], s);
    fix_page(s);
    update_images(s);

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppEvent(void *appstate, SDL_Event *event)
{
    struct appstate *s = appstate;

    if (event->type == SDL_EVENT_QUIT)
        return SDL_APP_SUCCESS;

    if (event->type == SDL_EVENT_KEY_DOWN) {
        struct bind *binds;
        int n_binds;
        switch (s->mode) {
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
                binds[i].fn(binds[i].args, s);

        if (readlist)
            write_readlist(files[s->file], s->start, s->scroll);
    }

    if (event->type == SDL_EVENT_FINGER_UP) {
        if (s->mode == SINGLE)
            page(event->tfinger.y > 0.5 ? "next" : "prev", s);
        else if (s->mode == BOOK)
            flip(event->tfinger.y > 0.3 ? "next" : "prev", s);
    }

    if (event->type == SDL_EVENT_FINGER_MOTION) {
        if (s->mode == STRIP) {
            char buffer[9];
            snprintf(buffer, sizeof(buffer), "%f", -1.6 * 5 * event->tfinger.dy);
            scroll(buffer, s);
        }
    }

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppIterate(void *appstate)
{
    struct appstate *s = appstate;

    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_GetRenderOutputSize(renderer, &width, &height);

    switch (s->mode) {
    case SINGLE:
        display_single(s->images[0]);
        break;

    case BOOK:
        if (s->start == s->end)
            display_single(s->images[0]);
        else
            display_book(s->images[1], s->images[0]);
        break;

    case STRIP:
        !s->rotated ? display_strip(s) : display_strip_rotated(s);
        break;
    }

    if (s->show_progress) {
        calc_progress(s);
        display_progress();
    }

    SDL_RenderPresent(renderer);
    return SDL_APP_CONTINUE;
}

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{
    struct appstate *s = appstate;

    free_intervals();
    free_text();

    free(dirpath);

    for (int i = 0; i < arrlen(s->images); i++)
        SDL_DestroyTexture(s->images[i]);
    arrfree(s->images);

    for (int i = 0; i < arrlen(s->pages); i++)
        free(s->pages[i].name);
    arrfree(s->pages);

    for (int i = 0; i < arrlen(files); i++)
        free(files[i]);
    arrfree(files);
}
