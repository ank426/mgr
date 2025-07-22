#include "headers.h"

extern SDL_Window *window;
extern int width, height;

extern char **files;


void quit(const char *args, struct appstate *s)
{
    SDL_Event *quit_event = malloc(sizeof(SDL_Event));
    *quit_event = (SDL_Event){SDL_EVENT_QUIT};
    SDL_PushEvent(quit_event);
}

void set_mode(const char *args, struct appstate *s)
{
    s->automode = false;
    if (strcmp(args, "single") == 0)
        s->mode = SINGLE;
    else if (strcmp(args, "book") == 0)
        s->mode = BOOK;
    else if (strcmp(args, "strip") == 0)
        s->mode = STRIP;
    else if (strcmp(args, "auto") == 0) {
        s->automode = true;
        set_mode_auto(s);
    } else
        assert(false);
    load_images(files[s->file], s);
}

void fullscreen(const char *args, struct appstate *s)
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

void offset(const char *args, struct appstate *s)
{
    if (s->pages[s->page].wide)
        return;

    if (strcmp(args, "true") == 0)
        get_interval(s->page)->offset = true;
    else if (strcmp(args, "false") == 0)
        get_interval(s->page)->offset = false;
    else if (strcmp(args, "toggle") == 0)
        get_interval(s->page)->offset ^= 1;
    else
        assert(false);

    load_images(files[s->file], s);
}

void progress(const char *args, struct appstate *s)
{
    if (strcmp(args, "true") == 0)
        s->show_progress = true;
    else if (strcmp(args, "false") == 0)
        s->show_progress = false;
    else if (strcmp(args, "toggle") == 0)
        s->show_progress ^= 1;
    else
        assert(false);
}

void top(const char *args, struct appstate *s)
{
    s->page = 0;
    load_images(files[s->file], s);
    s->scroll = 0;
}

void bottom(const char *args, struct appstate *s)
{
    s->page = arrlen(s->pages) - 1;
    load_images(files[s->file], s);
    s->scroll = 0;
}

void chapter(const char *args, struct appstate *s)
{
    if (strcmp(args, "next") == 0) {
        if (s->file == arrlen(files) - 1)
            return;
        s->file++;
        load_chapter(files[s->file], s);
        s->page = 0;
        s->scroll = 0;
    }

    else if (strcmp(args, "prev") == 0) {
        if (s->file == 0)
            return;
        s->file--;
        load_chapter(files[s->file], s);
        s->page = arrlen(s->pages) - 1;
        s->scroll = s->mode == STRIP ? s->pages[s->page].height : 0;
    }

    else
        assert(false);

    load_images(files[s->file], s);
}

void page(const char *args, struct appstate *s)
{
    if (strcmp(args, "next") == 0) {
        if (s->page == arrlen(s->pages) - 1)
            return chapter("next", s);
        s->page++;
    }

    else if (strcmp(args, "prev") == 0) {
        if (s->page == 0)
            return chapter("prev", s);
        s->page--;
    }

    else
        assert(false);

    load_images(files[s->file], s);
    s->scroll = 0;
}

void flip(const char *args, struct appstate *s)
{
    if (strcmp(args, "next") == 0) {
        if (s->page == arrlen(s->pages) - 1 || s->page == arrlen(s->pages) - 2 && num_images() == 2)
            return chapter("next", s);
        if (num_images() == 1 || s->page == arrlen(s->pages) - 2)
            s->page++;
        else
            s->page += 2;
    }

    else if (strcmp(args, "prev") == 0) {
        if (s->page == 0)
            return chapter("prev", s);
        s->page--;
    }

    else
        assert(false);

    load_images(files[s->file], s);
    s->scroll = 0;
}

void scroll(const char *args, struct appstate *s)
{
    float val = atof(args);
    if (val == 0)
        return;

    s->scroll += val * height * s->pages[s->page].width / s->zoom / width;

    if (val > 0) {
        if (s->scroll < s->pages[s->page].height)
            return;

        if (s->page == arrlen(s->pages) - 1) {
            s->scroll = s->pages[s->page].height;
            chapter("next", s);
        }
        else {
            s->scroll -= s->pages[s->page++].height;
            s->scroll *= s->pages[s->page].width / s->pages[s->page-1].width;
            load_images_next(files[s->file], s);
        }
    }

    else {
        if (s->scroll > 0)
            return;

        if (s->page == 0) {
            s->scroll = 0;
            chapter("prev", s);
        }
        else {
            s->scroll *= s->pages[s->page-1].width / s->pages[s->page].width;
            s->scroll += s->pages[--s->page].height;
            load_images_prev(files[s->file], s);
        }
    }
}

void rotate(const char *args, struct appstate *s)
{
    if (strcmp(args, "true") == 0)
        s->rotated = true;
    else if (strcmp(args, "false") == 0)
        s->rotated = false;
    else if (strcmp(args, "toggle") == 0)
        s->rotated ^= 1;
    else
        assert(false);
}

void set_zoom(const char *args, struct appstate *s)
{
    if (args[0] == '+')
        s->zoom += atof(args + 1);
    else if (args[0] == '-')
        s->zoom -= atof(args + 1);
    else
        s->zoom = atof(args);
}

void set_hzoom(const char *args, struct appstate *s)
{
    if (args[0] == '+')
        s->hzoom += atof(args + 1);
    else if (args[0] == '-')
        s->hzoom -= atof(args + 1);
    else
        s->hzoom = atof(args);
}
