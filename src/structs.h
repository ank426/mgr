#pragma once

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
    int page;

    bool automode;
    enum modes mode;

    float scroll;
    bool rotated;
    float zoom;
    float hzoom;

    bool show_progress;
};

struct config {
    bool automode;
    enum modes mode;

    float zoom;
    float hzoom;

    bool start_fullscreen;
};
