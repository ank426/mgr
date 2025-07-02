#include "headers.h"
#include "globals.h"

void quit(const char *args)
{
    SDL_Event *quit_event = malloc(sizeof(SDL_Event));
    *quit_event = (SDL_Event){SDL_EVENT_QUIT};
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

void fullscreen(const char *args)
{
    if (strcmp(args, "true") == 0)
        SDL_SetWindowFullscreen(window, true);
    else if (strcmp(args, "false") == 0)
        SDL_SetWindowFullscreen(window, false);
    else if (strcmp(args, "toggle") == 0)
        SDL_SetWindowFullscreen(window, !(SDL_GetWindowFlags(window) & SDL_WINDOW_FULLSCREEN));
    else
        assert(false);
}

void offset(const char *args)
{
    if (pages[current_page].wide)
        return;

    if (strcmp(args, "true") == 0)
        get_current_interval()->offset = true;
    else if (strcmp(args, "false") == 0)
        get_current_interval()->offset = false;
    else if (strcmp(args, "toggle") == 0)
        get_current_interval()->offset ^= 1;
    else
        assert(false);

    load_images();
}

void progress(const char *args)
{
    if (strcmp(args, "true") == 0)
        show_progress = true;
    else if (strcmp(args, "false") == 0)
        show_progress = false;
    else if (strcmp(args, "toggle") == 0)
        show_progress ^= 1;
    else
        assert(false);
}

void top(const char *args)
{
    current_page = 0;
    load_images();
    scrolled = 0;
}

void bottom(const char *args)
{
    current_page = total_pages - 1;
    load_images();
    scrolled = 0;
}

void chapter(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (curr_file == arrlen(files) - 1)
            return;
        curr_file++;
        load_chapter();
        current_page = 0;
        scrolled = 0;
    }

    else if (strcmp(args, "prev") == 0) {
        if (curr_file == 0)
            return;
        curr_file--;
        load_chapter();
        current_page = total_pages - 1;
        scrolled = mode == STRIP ? pages[current_page].height : 0;
    }

    else
        assert(false);

    load_images();
}

void page(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (current_page == total_pages - 1)
            return chapter("next");
        current_page++;
    }

    else if (strcmp(args, "prev") == 0) {
        if (current_page == 0)
            return chapter("prev");
        current_page--;
    }

    else
        assert(false);

    load_images();
    scrolled = 0;
}

void flip(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (current_page == total_pages - 1 || current_page == total_pages - 2 && image1 != nullptr && image2 != nullptr)
            return chapter("next");
        if (image1 == nullptr || image2 == nullptr || current_page == total_pages - 2)
            current_page++;
        else
            current_page += 2;
    }

    else if (strcmp(args, "prev") == 0) {
        if (current_page == 0)
            return chapter("prev");
        current_page--;
    }

    else
        assert(false);

    load_images();
    scrolled = 0;
}

void scroll(const char *args)
{
    float val = atof(args);
    if (val == 0)
        return;

    scrolled += val * height * pages[current_page].width / zoom / width;

    if (val > 0) {
        if (scrolled < pages[current_page].height)
            return;

        if (current_page == total_pages - 1) {
            scrolled = pages[current_page].height;
            chapter("next");
        }
        else {
            scrolled -= pages[current_page++].height;
            scrolled *= pages[current_page].width / pages[current_page-1].width;
            load_images_next();
        }
    }

    else {
        if (scrolled > 0)
            return;

        if (current_page == 0) {
            scrolled = 0;
            chapter("prev");
        }
        else {
            scrolled *= pages[current_page-1].width / pages[current_page].width;
            scrolled += pages[--current_page].height;
            load_images_prev();
        }
    }
}
