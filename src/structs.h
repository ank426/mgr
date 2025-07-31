#pragma once

#include <SDL3/SDL.h>

struct page {
    char *name;
    float width, height;
    bool wide;
};

struct interval {
    int start, end;
    bool offset;
};

enum modes {
    SINGLE,
    BOOK,
    STRIP,
};

struct config {
    bool automode;
    enum modes mode;
    float wzoom, hzoom;
    bool start_fullscreen;
};

struct appstate {
    int width, height;
    int file;
    struct page *pages;
    int start, end;
    bool automode;
    enum modes mode;
    bool rotated;
    float scroll;
    float wzoom, hzoom;
    bool show_progress;
};

struct bind {
    SDL_Keymod mod;
    SDL_Scancode key;
    void (*fn)(const char *args, struct appstate *s);
    const char *args;
};
