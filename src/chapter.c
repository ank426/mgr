#include "headers.h"
#include "structs.h"

extern const int width, height;
extern const struct page * const pages;
extern const int curr_page;
extern const enum modes mode;
extern const float scrolled;
extern const float zoom;
extern TTF_Text * const progress_text;


void load_chapter(const char * const path)
{
    update_pages_from_zip(path);
    nat_sort_pages();
    update_intervals();
}

void calculate_progress()
{
    char string[32];

    switch (mode) {
    case SINGLE:
        snprintf(string, 32, "%d/%d", curr_page+1, arrlen(pages));
        break;

    case BOOK:
        if (num_images() == 1)
            snprintf(string, 32, "%d/%d", curr_page+1, arrlen(pages));
        else
            snprintf(string, 32, "%d-%d/%d", curr_page+1, curr_page+2, arrlen(pages));
        break;

    case STRIP:
        if (pages[curr_page].height - scrolled > height * pages[curr_page].width / zoom / width)
            snprintf(string, 32, "%d/%d", curr_page+1, arrlen(pages));
        else
            snprintf(string, 32, "%d-%d/%d", curr_page+1, curr_page+2, arrlen(pages));
        break;
    }

    TTF_SetTextString(progress_text, string, 32);
}
