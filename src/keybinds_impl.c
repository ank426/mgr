#include "headers.h"
#include "globals.h"

void quit(const char *args)
{
    SDL_Event *quit_event = malloc(sizeof(SDL_Event));
    *quit_event = (SDL_Event) { SDL_EVENT_QUIT };
    SDL_PushEvent(quit_event);
}

void set_mode(const char *args)
{
    if (strcmp(args, "single") == 0)
        mode = SINGLE;
    else if (strcmp(args, "book") == 0)
        mode = BOOK;
    else if (strcmp(args, "strip") == 0)
        mode = STRIP;
    else
        assert(false);
    load_images();
}

void toggle_fullscreen(const char *args)
{
    SDL_SetWindowFullscreen(window, !(SDL_GetWindowFlags(window) & SDL_WINDOW_FULLSCREEN));
}

void toggle_offset(const char *args)
{
    if (files[current_page].wide)
        return;
    get_current_interval()->offset ^= 1;
    load_images();
}

void page(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (current_page == total_pages - 1)
            return;
        current_page++;
    }

    else if (strcmp(args, "prev") == 0) {
        if (current_page == 0)
            return;
        current_page--;
    }

    else
        assert(false);

    load_images();
}

void flip(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (current_page == total_pages - 1)
            return;
        if (image1 == NULL || image2 == NULL || current_page == total_pages - 2)
            current_page++;
        else
            current_page += 2;
    }

    else if (strcmp(args, "prev") == 0) {
        if (current_page == 0)
            return;
        current_page--;
    }

    else
        assert(false);

    load_images();
}

void scroll(const char *args)
{
    float val = atof(args);
    scrolled += val * height / scale;

    if (val == 0)
        return;

    if (val > 0) {
        if (scrolled < files[current_page].height)
            return;

        if (current_page == total_pages - 1)
            scrolled = files[current_page].height;
        else {
            scrolled -= files[current_page++].height;
            load_images_next();
        }
    }

    else {
        if (scrolled > 0)
            return;

        if (current_page == 0)
            scrolled = 0;
        else {
            scrolled += files[--current_page].height;
            load_images_prev();
        }
    }
}

void top(const char *args)
{
    current_page = 0;
    load_images();
}

void bottom(const char *args)
{
    current_page = total_pages - 1;
    load_images();
}
