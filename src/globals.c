#include "globals.h"

SDL_Window *window = nullptr;
SDL_Renderer *renderer = nullptr;

int width = 0;
int height = 0;

char path[256];
int current_page = 0;
int total_pages = 0;
float scrolled = 0;
float zoom = 0.5;
bool show_progress = false;

TTF_Font *progress_font = nullptr;
TTF_TextEngine *engine = nullptr;
TTF_Text *progress_text = nullptr;

SDL_Texture *image1 = nullptr;
SDL_Texture *image2 = nullptr;

enum modes mode = SINGLE;

struct page *pages;

int n_int = 0;
struct interval *intervals = nullptr;
