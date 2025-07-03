#include "headers.h"
#include "structs.h"

extern const struct page * const pages;
extern const int curr_page;

static struct interval *intervals = nullptr;

void update_intervals()
{
    arrfree(intervals);

    int s = 0;
    bool in = false;
    for (int i = 0; i < arrlen(pages); i++) {
        if (!in && !pages[i].wide) {
            s = i;
            in = true;
        }
        if (in && pages[i].wide) {
            arrput(intervals, ((struct interval){s, i, false}));
            in = false;
        }
    }
    if (in)
        arrput(intervals, ((struct interval){s, arrlen(pages), false}));

    if (intervals[0].start == 0)
        intervals[0].offset = intervals[0].end & 1;
}

struct interval *get_current_interval()
{
    int l = 0, r = arrlen(intervals);
    while (l < r) {
        int m = (l + r) / 2;
        if (curr_page < intervals[m].start)
            r = m;
        else if (intervals[m].end <= curr_page)
            l = m + 1;
        else
            return &intervals[m];
    }
    return nullptr;
}

void free_intervals()
{
    arrfree(intervals);
}
