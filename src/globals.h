#pragma once

#include <SDL3/SDL.h>
#include <SDL3_ttf/SDL_ttf.h>

extern SDL_Window *window;
extern SDL_Renderer *renderer;

extern int width;
extern int height;

extern char path[256];
extern int current_page;
extern int total_pages;
extern float scrolled;
extern float zoom;
extern bool show_progress;

extern TTF_TextEngine *engine;
extern TTF_Font *progress_font;
extern TTF_Text *progress_text;

extern SDL_Texture *image1;
extern SDL_Texture *image2;

enum modes {
    SINGLE,
    BOOK,
    STRIP,
} extern mode;

struct page {
    char name[256];
    float width;
    float height;
    bool wide;
} extern *pages;

extern int n_int;
struct interval {
    int start;
    int end;
    bool offset;
} extern *intervals;
