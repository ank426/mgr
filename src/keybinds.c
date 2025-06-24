#include "headers.h"

struct bind single_binds[] = {
    { SDL_KMOD_NONE, SDL_SCANCODE_Q, &quit, "" },
    { SDL_KMOD_NONE, SDL_SCANCODE_M, &set_mode, "book" },
    { SDL_KMOD_NONE, SDL_SCANCODE_J, &page, "next" },
    { SDL_KMOD_NONE, SDL_SCANCODE_K, &page, "prev" },
};
int n_single_binds = sizeof(single_binds) / sizeof(struct bind);

struct bind book_binds[] = {
    { SDL_KMOD_NONE, SDL_SCANCODE_Q, &quit, "" },
    { SDL_KMOD_NONE, SDL_SCANCODE_M, &set_mode, "strip" },
    { SDL_KMOD_NONE, SDL_SCANCODE_J, &flip, "next" },
    { SDL_KMOD_NONE, SDL_SCANCODE_K, &flip, "prev" },
    { SDL_KMOD_NONE, SDL_SCANCODE_O, &toggle_offset, "" },
};
int n_book_binds = sizeof(book_binds) / sizeof(struct bind);

struct bind strip_binds[] = {
    { SDL_KMOD_NONE, SDL_SCANCODE_Q, &quit, "" },
    { SDL_KMOD_NONE, SDL_SCANCODE_M, &set_mode, "single" },
    { SDL_KMOD_NONE, SDL_SCANCODE_D, &scroll, "+0.5" },
    { SDL_KMOD_NONE, SDL_SCANCODE_U, &scroll, "-0.5" },
};
int n_strip_binds = sizeof(strip_binds) / sizeof(struct bind);
