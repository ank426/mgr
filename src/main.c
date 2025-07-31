#define STB_DS_IMPLEMENTATION
#include "headers.h"

#define SDL_MAIN_USE_CALLBACKS 1
#include <SDL3/SDL_main.h>


SDL_Window *window = nullptr;
SDL_Renderer *renderer = nullptr;

char *dirpath = nullptr;
char **files = nullptr;
bool readlist = false;
struct config conf;


void init_conf()
{
    conf = (struct config){
        .automode = true,
        .mode = SINGLE,
        .wzoom = 0.45,
        .hzoom = 0.7,
        .start_fullscreen = true,
    };
}

void init_state(struct appstate *s)
{
    *s = (struct appstate){
        .width = 0,
        .height = 0,
        .file = 0,
        .pages = nullptr,
        .start = 0,
        .end = 0,
        .automode = conf.automode,
        .mode = conf.mode,
        .rotated = false,
        .scroll = 0.0,
        .wzoom = conf.wzoom,
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
    SDL_GetRenderOutputSize(renderer, &s->width, &s->height);

    init_text();

    load_file(files[s->file], s);
    fix_page(s);

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppEvent(void *appstate, SDL_Event *event)
{
    struct appstate *s = appstate;

    if (event->type == SDL_EVENT_QUIT)
        return SDL_APP_SUCCESS;

    handle_event(event, s);
    display(s);

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppIterate(void *appstate)
{
    SDL_Delay(16);
    return SDL_APP_CONTINUE;
}

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{
    struct appstate *s = appstate;

    free_intervals();
    free_text();

    free(dirpath);

    for (int i = 0; i < arrlen(s->pages); i++)
        free(s->pages[i].name);
    arrfree(s->pages);

    for (int i = 0; i < arrlen(files); i++)
        free(files[i]);
    arrfree(files);

    free(s);
}
