#include "headers.h"

extern char **const files;


void load_file(char *const path, struct appstate *s)
{
    free_textures();
    threads_free();
    threads_init(s);
    update_pages_from_zip(&s->pages, path);
    nat_sort_pages(s->pages);
    update_intervals(s->pages);
    if (s->automode)
        s->mode = calc_mode(s->pages);
}

enum modes calc_mode(struct page *pages)
{
    int n = 0;
    for (int i = 0; i < arrlen(pages); i++) {
        float iar = pages[i].inv_ar;
        if (1.4 < iar && iar <= 1.5 || 0.7 < iar && iar <= 0.75)
            n++;
    }
    return n >= arrlen(pages) / 2 ? BOOK : STRIP;
}

void fix_page(struct appstate *s)
{
    switch (s->mode) {
    case SINGLE:
        s->end = s->start;
        break;

    case BOOK:
        s->end = s->start;
        if (s->pages[s->start].inv_ar < 1)
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
        double cr = fabs(cospi(s->rotation/180)), sr = fabs(sinpi(s->rotation/180));

        while (s->scroll >= s->pages[s->start].inv_ar) {
            if (s->start == arrlen(s->pages) - 1) {
                file("next", s);
                break;
            }
            s->scroll -= s->pages[s->start++].inv_ar;
        }

        while (s->scroll < (s->start == 0 ? 0 : sr)) {
            if (s->start == 0) {
                file("prev", s);
                break;
            }
            s->scroll += s->pages[--s->start].inv_ar;
        }

        double wh = s->wzoom * s->width * cr * cr + s->hzoom * s->height * sr * sr;

        float distance;
        if (cr == 0)
            distance = s->width / sr / wh;
        else if (sr == 0)
            distance = s->height / cr / wh;
        else
            distance = fmin(s->height / cr, s->width / sr) / wh;
        distance += s->scroll;

        s->end = s->start;
        while (distance > 0 && s->end < arrlen(s->pages))
            distance -= s->pages[s->end++].inv_ar;
        s->end--;

        break;
    }
}
