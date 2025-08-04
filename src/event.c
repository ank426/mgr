#include "structs.h"
#include "headers.h"

extern SDL_Renderer *const renderer;
extern const bool readlist;
extern char **const files;

SDL_AppResult handle_event(SDL_Event *event, struct appstate *s)
{
    switch (event->type) {
    case SDL_EVENT_QUIT:
        return SDL_APP_SUCCESS;

    case SDL_EVENT_WINDOW_RESIZED:
        SDL_GetRenderOutputSize(renderer, &s->width, &s->height);
        break;

    case SDL_EVENT_KEY_DOWN:
        struct bind *binds = get_binds(s->mode);
        int num_binds = get_num_binds(s->mode);
        for (int i = 0; i < num_binds; i++)
            if (event->key.mod == binds[i].mod && event->key.scancode == binds[i].key)
                binds[i].fn(binds[i].args, s);
        break;

    case SDL_EVENT_FINGER_UP:
        if (s->mode == SINGLE)
            page(event->tfinger.y > 0.5 ? "next" : "prev", s);
        else if (s->mode == BOOK)
            flip(event->tfinger.y > 0.3 ? "next" : "prev", s);
        break;

    case SDL_EVENT_FINGER_MOTION:
        if (s->mode == STRIP) {
            char buffer[9];
            snprintf(buffer, sizeof(buffer), "%f", -1.6 * 5 * event->tfinger.dy);
            scroll(buffer, s);
        }
        break;
    }

    return SDL_APP_CONTINUE;
}
