#include "headers.h"

extern char **const files;


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

void fix_page(struct appstate *s)
{
    s->end = s->start;

    switch (s->mode) {
    case SINGLE:
        break;

    case BOOK:
        if (s->pages[s->start].wide)
            break;
        struct interval *curr_int = get_interval(s->start);
        if (s->start & 1 ^ curr_int->start & 1 ^ !curr_int->offset) {
            if (s->start + 1 < curr_int->end)
                s->end++;
        } else {
            if (s->start - 1 >= curr_int->start)
                s->start--;
        }
        break;

    case STRIP:
        float d = !s->rotated ? s->height / s->wzoom / s->width : s->width / s->hzoom / s->height;
        d += s->scroll / s->pages[s->end].width;
        while (d > 0 && s->end < arrlen(s->pages)) {
            d -= s->pages[s->end].height / s->pages[s->end].width;
            s->end++;
        }
        s->end--;
        break;
    }
}
