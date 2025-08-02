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
    float iars[] = {1.422, 0.711};
    int n_iars = sizeof(iars) / sizeof(float);

    int n = 0;
    for (int i = 0; i < arrlen(pages); i++)
        for (int j = 0; j < n_iars; j++)
            if (fabs(pages[i].inv_ar - iars[j]) < 1e-3) {
                n++;
                break;
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
            if (s->start == arrlen(s->pages) - 1)
                return file("next", s);
            s->scroll -= s->pages[s->start++].inv_ar;
        }

        while (s->scroll < (s->start == 0 ? 0 : sr)) {
            if (s->start == 0)
                return file("prev", s);
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
