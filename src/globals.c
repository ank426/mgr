#include "globals.h"

SDL_Window *window = NULL;
SDL_Renderer *renderer = NULL;

int width = 0;
int height = 0;

char path[256];
int current_page = 0;
int total_pages = 0;
SDL_Texture *image1 = NULL;
SDL_Texture *image2 = NULL;

enum modes mode = SINGLE;

struct file *files;

int n_int = 0;
struct interval *intervals = NULL;

float scrolled = 0;
float scale = 0;
