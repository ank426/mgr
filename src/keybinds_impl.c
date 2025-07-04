#include "headers.h"
#include "structs.h"

extern SDL_Window *window;
extern int width, height;

extern char **files;
extern int curr_file;

extern struct page *pages;
extern int curr_page;

extern enum modes mode;
extern float scrolled;
extern float zoom;
extern bool show_progress;


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
    load_images(files[curr_file]);
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
    if (pages[curr_page].wide)
        return;

    if (strcmp(args, "true") == 0)
        get_interval(curr_page)->offset = true;
    else if (strcmp(args, "false") == 0)
        get_interval(curr_page)->offset = false;
    else if (strcmp(args, "toggle") == 0)
        get_interval(curr_page)->offset ^= 1;
    else
        assert(false);

    load_images(files[curr_file]);
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
    curr_page = 0;
    load_images(files[curr_file]);
    scrolled = 0;
}

void bottom(const char *args)
{
    curr_page = arrlen(pages) - 1;
    load_images(files[curr_file]);
    scrolled = 0;
}

void chapter(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (curr_file == arrlen(files) - 1)
            return;
        curr_file++;
        load_chapter(files[curr_file]);
        curr_page = 0;
        scrolled = 0;
    }

    else if (strcmp(args, "prev") == 0) {
        if (curr_file == 0)
            return;
        curr_file--;
        load_chapter(files[curr_file]);
        curr_page = arrlen(pages) - 1;
        scrolled = mode == STRIP ? pages[curr_page].height : 0;
    }

    else
        assert(false);

    load_images(files[curr_file]);
}

void page(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (curr_page == arrlen(pages) - 1)
            return chapter("next");
        curr_page++;
    }

    else if (strcmp(args, "prev") == 0) {
        if (curr_page == 0)
            return chapter("prev");
        curr_page--;
    }

    else
        assert(false);

    load_images(files[curr_file]);
    scrolled = 0;
}

void flip(const char *args)
{
    if (strcmp(args, "next") == 0) {
        if (curr_page == arrlen(pages) - 1 || curr_page == arrlen(pages) - 2 && num_images() == 2)
            return chapter("next");
        if (num_images() == 1 || curr_page == arrlen(pages) - 2)
            curr_page++;
        else
            curr_page += 2;
    }

    else if (strcmp(args, "prev") == 0) {
        if (curr_page == 0)
            return chapter("prev");
        curr_page--;
    }

    else
        assert(false);

    load_images(files[curr_file]);
    scrolled = 0;
}

void scroll(const char *args)
{
    float val = atof(args);
    if (val == 0)
        return;

    scrolled += val * height * pages[curr_page].width / zoom / width;

    if (val > 0) {
        if (scrolled < pages[curr_page].height)
            return;

        if (curr_page == arrlen(pages) - 1) {
            scrolled = pages[curr_page].height;
            chapter("next");
        }
        else {
            scrolled -= pages[curr_page++].height;
            scrolled *= pages[curr_page].width / pages[curr_page-1].width;
            load_images_next(files[curr_file]);
        }
    }

    else {
        if (scrolled > 0)
            return;

        if (curr_page == 0) {
            scrolled = 0;
            chapter("prev");
        }
        else {
            scrolled *= pages[curr_page-1].width / pages[curr_page].width;
            scrolled += pages[--curr_page].height;
            load_images_prev(files[curr_file]);
        }
    }
}
