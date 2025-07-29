#include "headers.h"

extern SDL_Window *window;
extern int width, height;

extern char **files;


void quit(const char *args, struct appstate *s)
{
    SDL_PushEvent(&(SDL_Event){SDL_EVENT_QUIT});
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
        s->mode = calc_mode(s->pages);
    } else
        assert(false);

    fix_page(s);
    update_images(s);
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

    fix_page(s);
    update_images(s);
}

void offset(const char *args, struct appstate *s)
{
    if (s->pages[s->start].wide)
        return;

    if (strcmp(args, "true") == 0)
        get_interval(s->start)->offset = true;
    else if (strcmp(args, "false") == 0)
        get_interval(s->start)->offset = false;
    else if (strcmp(args, "toggle") == 0)
        get_interval(s->start)->offset ^= 1;
    else
        assert(false);

    fix_page(s);
    update_images(s);
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
    s->start = 0;
    s->scroll = 0;
    fix_page(s);
    update_images(s);
}

void bottom(const char *args, struct appstate *s)
{
    s->start = arrlen(s->pages) - 1;
    if (s->mode != STRIP) {
        s->scroll = 0;
        fix_page(s);
        update_images(s);
    } else {
        s->scroll = arrlast(s->pages).height;
        scroll("-1", s);
    }
}

void file(const char *args, struct appstate *s)
{
    if (strcmp(args, "next") == 0) {
        if (s->file == arrlen(files) - 1) {
            s->scroll = s->mode == STRIP ? s->pages[s->start].height : 0;
            return;
        }
        load_file(files[++s->file], s);
        s->start = 0;
        s->scroll = 0;
    }

    else if (strcmp(args, "prev") == 0) {
        if (s->file == 0) {
            s->scroll = 0;
            return;
        }
        load_file(files[--s->file], s);
        s->start = arrlen(s->pages) - 1;
        s->scroll = s->mode == STRIP ? s->pages[s->start].height : 0;
    }

    else
        assert(false);

    fix_page(s);
    update_images(s);
}

void page(const char *args, struct appstate *s)
{
    if (strcmp(args, "next") == 0) {
        if (s->start == arrlen(s->pages) - 1)
            return file("next", s);
        s->start++;
    }

    else if (strcmp(args, "prev") == 0) {
        if (s->start == 0)
            return file("prev", s);
        s->start--;
    }

    else
        assert(false);

    s->scroll = 0;
    fix_page(s);
    update_images(s);
}

void flip(const char *args, struct appstate *s)
{
    if (strcmp(args, "next") == 0) {
        if (s->end == arrlen(s->pages) - 1)
            return file("next", s);
        s->start = s->end + 1;
    }
    else if (strcmp(args, "prev") == 0) {
        if (s->start == 0)
            return file("prev", s);
        s->start--;
    }
    else
        assert(false);

    s->scroll = 0;
    fix_page(s);
    update_images(s);
}

void scroll(const char *args, struct appstate *s)
{
    s->scroll += atof(args) * height * s->pages[s->start].width / s->zoom / width;

    while (true)
        if (s->scroll >= s->pages[s->start].height) {
            if (s->start == arrlen(s->pages) - 1)
                return file("next", s);
            s->scroll -= s->pages[s->start++].height;
            s->scroll *= s->pages[s->start].width / s->pages[s->start-1].width;
        }

        else if (s->scroll < 0) {
            if (s->start == 0)
                return file("prev", s);
            s->scroll *= s->pages[s->start-1].width / s->pages[s->start].width;
            s->scroll += s->pages[--s->start].height;
        }

        else
            break;

    fix_page(s);
    update_images(s);
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
    fix_page(s);
    update_images(s);
}

void set_zoom(const char *args, struct appstate *s)
{
    if (args[0] == '+')
        s->zoom += atof(args + 1);
    else if (args[0] == '-')
        s->zoom -= atof(args + 1);
    else
        s->zoom = atof(args);
    fix_page(s);
    update_images(s);
}

void set_hzoom(const char *args, struct appstate *s)
{
    if (args[0] == '+')
        s->hzoom += atof(args + 1);
    else if (args[0] == '-')
        s->hzoom -= atof(args + 1);
    else
        s->hzoom = atof(args);
    fix_page(s);
    update_images(s);
}
