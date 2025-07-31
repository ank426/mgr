#include "structs.h"
#include "headers.h"

extern SDL_Renderer *const renderer;
extern const bool readlist;
extern char **const files;

extern struct bind single_binds[], book_binds[], strip_binds[];
extern int n_single_binds, n_book_binds, n_strip_binds;

void handle_event(SDL_Event *event, struct appstate *s)
{

    if (event->type == SDL_EVENT_WINDOW_RESIZED)
        SDL_GetRenderOutputSize(renderer, &s->width, &s->height);

    if (event->type == SDL_EVENT_KEY_DOWN) {
        struct bind *binds;
        int n_binds;
        switch (s->mode) {
        case SINGLE:
            binds = single_binds;
            n_binds = n_single_binds;
            break;
        case BOOK:
            binds = book_binds;
            n_binds = n_book_binds;
            break;
        case STRIP:
            binds = strip_binds;
            n_binds = n_strip_binds;
            break;
        }

        for (int i = 0; i < n_binds; i++)
            if (event->key.mod == binds[i].mod && event->key.scancode == binds[i].key)
                binds[i].fn(binds[i].args, s);

        if (readlist)
            write_readlist(files[s->file], s->start, s->scroll);
    }

    if (event->type == SDL_EVENT_FINGER_UP) {
        if (s->mode == SINGLE)
            page(event->tfinger.y > 0.5 ? "next" : "prev", s);
        else if (s->mode == BOOK)
            flip(event->tfinger.y > 0.3 ? "next" : "prev", s);
    }

    if (event->type == SDL_EVENT_FINGER_MOTION) {
        if (s->mode == STRIP) {
            char buffer[9];
            snprintf(buffer, sizeof(buffer), "%f", -1.6 * 5 * event->tfinger.dy);
            scroll(buffer, s);
        }
    }
}
