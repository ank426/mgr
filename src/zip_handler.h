#include <SDL3/SDL.h>

void update_pages_from_zip(struct page **ptr_pages, const char *path);
SDL_Texture *load_image_from_zip(char *pagename, const char *path);
