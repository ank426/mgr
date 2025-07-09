#include "headers.h"
#include "structs.h"

extern const int width, height;
extern struct page *const pages;
extern const int curr_page;
extern const bool automode;
extern enum modes mode;
extern const float scrolled;
extern const float zoom;
extern TTF_Text *const progress_text;


void load_chapter(const char *const path)
{
    update_pages_from_zip(path);
    nat_sort_pages(pages);
    update_intervals(pages);
    if (automode)
        set_mode_auto();
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

void set_mode_auto()
{
    int n = 0;
    for (int i = 0; i < arrlen(pages); i++)
        if (pages[i].height > 2 * pages[i].width)
            n++;
    mode = n > arrlen(pages) / 2 ? STRIP : BOOK;
}
