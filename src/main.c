#include "zip_handler.h"

#define SDL_MAIN_USE_CALLBACKS 1

#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3_image/SDL_image.h>

#include <string.h>

SDL_Window *window = NULL;
SDL_Renderer *renderer = NULL;

char path[255];
int current_page = 0;
int total_pages = 0;
SDL_Texture *image = NULL;

SDL_AppResult SDL_AppInit(void **appstate, int argc, char *argv[])
{
    if (!SDL_CreateWindowAndRenderer("mgr", 0, 0, SDL_WINDOW_RESIZABLE, &window, &renderer)) {
        SDL_Log("Couldn't create window and renderer: %s\n", SDL_GetError());
        return SDL_APP_FAILURE;
    }

    strncpy(path, argv[1], 255);
    total_pages = get_num_entries_from_zip(path);
    image = load_image_from_zip(path, current_page, renderer);

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppEvent(void *appstate, SDL_Event *event)
{
    if (event->type == SDL_EVENT_QUIT ||
        event->type == SDL_EVENT_KEY_DOWN && event->key.scancode == SDL_SCANCODE_Q) {
        return SDL_APP_SUCCESS;
    }

    if (event->type == SDL_EVENT_KEY_DOWN && event->key.scancode == SDL_SCANCODE_J) {
        if (current_page < total_pages - 1)
            image = load_image_from_zip(path, ++current_page, renderer);
    }

    if (event->type == SDL_EVENT_KEY_DOWN && event->key.scancode == SDL_SCANCODE_K) {
        if (current_page > 0)
            image = load_image_from_zip(path, --current_page, renderer);
    }

    return SDL_APP_CONTINUE;
}

SDL_AppResult SDL_AppIterate(void *appstate)
{
    int w = 0, h = 0;
    SDL_GetRenderOutputSize(renderer, &w, &h);

    SDL_FRect dst;
    SDL_GetTextureSize(image, &dst.w, &dst.h);

    float scale, sw, sh;
    sw = w / dst.w;
    sh = h / dst.h;
    scale = sw < sh ? sw : sh;

    SDL_SetRenderScale(renderer, scale, scale);
    dst.x = (w / scale - dst.w) / 2;
    dst.y = (h / scale - dst.h) / 2;

    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    SDL_RenderTexture(renderer, image, NULL, &dst);
    SDL_RenderPresent(renderer);

    return SDL_APP_CONTINUE;
}

void SDL_AppQuit(void *appstate, SDL_AppResult result)
{

}
