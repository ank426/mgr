#include "headers.h"

static struct bind single_binds[] = {
    { SDL_KMOD_NONE,   SDL_SCANCODE_Q, &quit,       ""       },
    { SDL_KMOD_NONE,   SDL_SCANCODE_M, &set_mode,   "book"   },
    { SDL_KMOD_NONE,   SDL_SCANCODE_F, &fullscreen, "toggle" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_S, &progress,   "toggle" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_G, &top,        ""       },
    { SDL_KMOD_LSHIFT, SDL_SCANCODE_G, &bottom,     ""       },
    { SDL_KMOD_NONE,   SDL_SCANCODE_R, &rotate,     "+90"    },
    { SDL_KMOD_NONE,   SDL_SCANCODE_E, &rotate,     "-90"    },
    { SDL_KMOD_RALT,   SDL_SCANCODE_R, &rotate,     "+45"    },
    { SDL_KMOD_RALT,   SDL_SCANCODE_E, &rotate,     "-45"    },
    { SDL_KMOD_NONE,   SDL_SCANCODE_J, &page,       "next"   },
    { SDL_KMOD_NONE,   SDL_SCANCODE_K, &page,       "prev"   },
};

static struct bind book_binds[] = {
    { SDL_KMOD_NONE,   SDL_SCANCODE_Q, &quit,       ""       },
    { SDL_KMOD_NONE,   SDL_SCANCODE_M, &set_mode,   "strip"  },
    { SDL_KMOD_NONE,   SDL_SCANCODE_F, &fullscreen, "toggle" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_S, &progress,   "toggle" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_O, &offset,     "toggle" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_G, &top,        ""       },
    { SDL_KMOD_LSHIFT, SDL_SCANCODE_G, &bottom,     ""       },
    { SDL_KMOD_NONE,   SDL_SCANCODE_R, &rotate,     "+90"    },
    { SDL_KMOD_NONE,   SDL_SCANCODE_E, &rotate,     "-90"    },
    { SDL_KMOD_RALT,   SDL_SCANCODE_R, &rotate,     "+45"    },
    { SDL_KMOD_RALT,   SDL_SCANCODE_E, &rotate,     "-45"    },
    { SDL_KMOD_NONE,   SDL_SCANCODE_J, &flip,       "next"   },
    { SDL_KMOD_NONE,   SDL_SCANCODE_K, &flip,       "prev"   },
};

static struct bind strip_binds[] = {
    { SDL_KMOD_NONE,   SDL_SCANCODE_Q,      &quit,       ""       },
    { SDL_KMOD_NONE,   SDL_SCANCODE_M,      &set_mode,   "single" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_F,      &fullscreen, "toggle" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_S,      &progress,   "toggle" },
    { SDL_KMOD_NONE,   SDL_SCANCODE_G,      &top,        ""       },
    { SDL_KMOD_LSHIFT, SDL_SCANCODE_G,      &bottom,     ""       },
    { SDL_KMOD_NONE,   SDL_SCANCODE_R,      &rotate,     "+90"    },
    { SDL_KMOD_NONE,   SDL_SCANCODE_E,      &rotate,     "-90"    },
    { SDL_KMOD_RALT,   SDL_SCANCODE_R,      &rotate,     "+45"    },
    { SDL_KMOD_RALT,   SDL_SCANCODE_E,      &rotate,     "-45"    },
    { SDL_KMOD_NONE,   SDL_SCANCODE_J,      &scroll,     "+0.5"   },
    { SDL_KMOD_NONE,   SDL_SCANCODE_K,      &scroll,     "-0.5"   },
    { SDL_KMOD_NONE,   SDL_SCANCODE_MINUS,  &set_wzoom,  "-0.05"  },
    { SDL_KMOD_NONE,   SDL_SCANCODE_EQUALS, &set_wzoom,  "+0.05"  },
    { SDL_KMOD_LSHIFT, SDL_SCANCODE_MINUS,  &set_hzoom,  "-0.05"  },
    { SDL_KMOD_LSHIFT, SDL_SCANCODE_EQUALS, &set_hzoom,  "+0.05"  },
};


struct bind *get_binds(enum modes mode) {
    switch (mode) {
    case SINGLE:
        return single_binds;
    case BOOK:
        return book_binds;
    case STRIP:
        return strip_binds;
    }
}

int get_num_binds(enum modes mode) {
    switch (mode) {
    case SINGLE:
        return sizeof(single_binds)/sizeof(struct bind);
    case BOOK:
        return sizeof(book_binds)/sizeof(struct bind);
    case STRIP:
        return sizeof(strip_binds)/sizeof(struct bind);
    }
}
