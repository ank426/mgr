#ifndef GLOBALS_H
#define GLOBALS_H

#include <SDL3/SDL.h>

extern SDL_Window *window;
extern SDL_Renderer *renderer;

extern int width;
extern int height;

extern char path[256];
extern int current_page;
extern int total_pages;
extern SDL_Texture *image1;
extern SDL_Texture *image2;

enum modes {
    SINGLE,
    BOOK,
    STRIP,
} extern mode;

struct dim {
    int width;
    int height;
    bool wide;
} extern *dims;

extern int n_int;
struct interval {
    int start;
    int end;
    bool offset;
} extern *intervals;

extern float scroll;
extern float scale;

#endif
