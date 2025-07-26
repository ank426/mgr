#include "headers.h"

extern const int width, height;
extern char **const files;
extern TTF_Text *const progress_text;


void load_file(char *const path, struct appstate *s)
{
    update_pages_from_zip(&s->pages, path);
    nat_sort_pages(s->pages);
    update_intervals(s->pages);
    if (s->automode)
        s->mode = calc_mode(s->pages);
}

enum modes calc_mode(struct page *pages)
{
    int n = 0;
    for (int i = 0; i < arrlen(pages); i++)
        if (pages[i].height > 2 * pages[i].width)
            n++;
    return n > arrlen(pages) / 2 ? STRIP : BOOK;
}

void calc_progress(struct appstate *s)
{
    char string[32];
    if (s->start == s->end)
        snprintf(string, 32, "%d/%d", s->start+1, arrlen(s->pages));
    else
        snprintf(string, 32, "%d-%d/%d", s->start+1, s->end+1, arrlen(s->pages));

    TTF_SetTextString(progress_text, string, 32);
}
