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

    switch (s->mode) {
    case SINGLE:
        snprintf(string, 32, "%d/%d", s->page+1, arrlen(s->pages));
        break;

    case BOOK:
        if (num_images() == 1)
            snprintf(string, 32, "%d/%d", s->page+1, arrlen(s->pages));
        else
            snprintf(string, 32, "%d-%d/%d", s->page+1, s->page+2, arrlen(s->pages));
        break;

    case STRIP:
        int d = s->rotated ? width / s->hzoom / height : height / s->zoom / width;
        int d1 = (s->pages[s->page].height - s->scroll) / s->pages[s->page].width;
        if (s->page == arrlen(s->pages)-1 || d1 > d)
            snprintf(string, 32, "%d/%d", s->page+1, arrlen(s->pages));
        else if (s->page == arrlen(s->pages)-2 || d1 + s->pages[s->page+1].height / s->pages[s->page+1].width > d)
            snprintf(string, 32, "%d-%d/%d", s->page+1, s->page+2, arrlen(s->pages));
        else
            snprintf(string, 32, "%d-%d/%d", s->page+1, s->page+3, arrlen(s->pages));
        break;
    }

    TTF_SetTextString(progress_text, string, 32);
}
