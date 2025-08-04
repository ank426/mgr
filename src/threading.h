#include "structs.h"

void threads_free(void);
void threads_init(struct appstate *s);
void update_surfaces(struct appstate *s);
SDL_Surface *get_surface(int idx);
