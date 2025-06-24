#include <SDL3/SDL.h>

struct bind {
    SDL_Keymod mod;
    SDL_Scancode key;
    void (*fn)(const char *args);
    const char *args;
} extern single_binds[], book_binds[], strip_binds[];
int extern n_single_binds, n_book_binds, n_strip_binds;
