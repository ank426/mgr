#include "headers.h"
#include "globals.h"

void update_intervals()
{
    if (intervals != nullptr) free(intervals);
    intervals = malloc(total_pages * sizeof(struct interval));

    n_int = 0;
    int s = 0;
    bool in = false;
    for (int i = 0; i < total_pages; i++) {
        if (!in && !pages[i].wide) {
            s = i;
            in = true;
        }
        if (in && pages[i].wide) {
            intervals[n_int++] = (struct interval) { s, i, false };
            in = false;
        }
    }
    if (in)
        intervals[n_int++] = (struct interval) { s, total_pages, false };

    if (intervals[0].start == 0)
        intervals[0].offset = intervals[0].end & 1;

    intervals = realloc(intervals, n_int * sizeof(struct interval));
}

struct interval *get_current_interval()
{
    int l = 0, r = n_int;
    while (l < r) {
        int m = (l + r) / 2;
        if (current_page < intervals[m].start)
            r = m;
        else if (intervals[m].end <= current_page)
            l = m + 1;
        else
            return &intervals[m];
    }
    return nullptr;
}
