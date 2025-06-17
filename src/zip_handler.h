#include <SDL3/SDL.h>

SDL_Texture *load_image_from_zip(const char *path, int index, SDL_Renderer *renderer);
int get_num_entries_from_zip(const char *path);
