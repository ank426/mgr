#include "headers.h"

extern SDL_Window *const window;

extern char **const files;


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
    if (s->pages[s->start].inv_ar < 1)
        return;

    if (strcmp(args, "true") == 0)
        get_interval(s->start)->offset = true;
    else if (strcmp(args, "false") == 0)
        get_interval(s->start)->offset = false;
    else if (strcmp(args, "toggle") == 0)
        get_interval(s->start)->offset ^= 1;
    else
        assert(false);
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
}

void bottom(const char *args, struct appstate *s)
{
    s->start = arrlen(s->pages) - 1;
    if (s->mode != STRIP)
        s->scroll = 0;
    else {
        s->scroll = arrlast(s->pages).inv_ar - 1e-6;
        scroll("-1", s);
    }
}

void file(const char *args, struct appstate *s)
{
    if (strcmp(args, "next") == 0) {
        if (s->file == arrlen(files) - 1) {
            s->scroll = s->mode == STRIP ? s->pages[s->start].inv_ar - 1e-6 : 0;
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
        s->scroll = s->mode == STRIP ? s->pages[s->start].inv_ar - 1e-6 : 0;
    }
    else
        assert(false);
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
}

void scroll(const char *args, struct appstate *s)
{
    s->scroll += atof(args) * s->height / s->wzoom / s->width;
}

void rotate(const char *args, struct appstate *s)
{
    if (strncmp(args, "set", sizeof("set") - 1) == 0)
        s->rotation = atof(args + sizeof("set"));
    else if (strncmp(args, "toggle", sizeof("toggle") - 1) == 0)
        s->rotation = s->rotation == 0 ? atof(args + sizeof("toggle")) : 0;
    else
        s->rotation += strtold(args, nullptr);

    if (s->rotation > 180)
        s->rotation -= 360 * (int)((s->rotation + 180) / 360 - 1e-6);
    if (s->rotation <= -180)
        s->rotation += 360 * (int)(-(s->rotation - 180) / 360);
}

void set_wzoom(const char *args, struct appstate *s)
{
    if (args[0] == '+')
        s->wzoom += atof(args + 1);
    else if (args[0] == '-')
        s->wzoom -= atof(args + 1);
    else
        s->wzoom = atof(args);
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
