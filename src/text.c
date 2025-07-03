#include "headers.h"

extern SDL_Renderer *renderer;

static TTF_TextEngine *engine = nullptr;
static TTF_Font *progress_font = nullptr;
TTF_Text *progress_text = nullptr;

int progress_font_size = 0;

void init_text()
{
    assert(TTF_Init());

    engine = TTF_CreateRendererTextEngine(renderer);
    assert(engine != nullptr);

    progress_font_size = 50;
    progress_font = TTF_OpenFont("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf", progress_font_size);
    assert(progress_font != nullptr);

    progress_text = TTF_CreateText(engine, progress_font, "", 0);
    assert(progress_text != nullptr);
}

void free_text()
{
    TTF_DestroyRendererTextEngine(engine);
    TTF_CloseFont(progress_font);
    TTF_DestroyText(progress_text);
    TTF_Quit();
}
