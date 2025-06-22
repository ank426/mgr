#include <SDL3/SDL.h>

int get_num_entries_from_zip(const char *path);
void update_wides_from_zip(const char *path, int n, bool **ret);
void update_dims_from_zip(const char *path, int n);
SDL_Texture *load_image_from_zip(const char *path, int index, SDL_Renderer *renderer);
