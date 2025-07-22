#include <SDL3/SDL.h>

void display_single(SDL_Texture *image);
void display_book(SDL_Texture *image1, SDL_Texture *image2);
void display_strip(SDL_Texture *image1, SDL_Texture *image2, SDL_Texture *image3, struct appstate *s);
void display_strip_rotated(SDL_Texture *image1, SDL_Texture *image2, SDL_Texture *image3, struct appstate *s);
void display_progress(void);
