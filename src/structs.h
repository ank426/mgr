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

struct appstate {
    int file;
    struct page *pages;

    int start, end;
    SDL_Texture **images;

    bool automode;
    enum modes mode;

    float scroll;
    bool rotated;
    float zoom, hzoom;

    bool show_progress;
};

struct config {
    bool automode;
    enum modes mode;

    float zoom, hzoom;

    bool start_fullscreen;
};
