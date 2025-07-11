#include "headers.h"
#include "structs.h"

extern const int width, height;
extern struct page *const pages;
extern const int curr_page;
extern const bool automode;
extern enum modes mode;
extern const bool rotated;
extern const float scrolled;
extern const float zoom;
extern const float hzoom;
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
        int d = rotated ? width / hzoom / height : height / zoom / width;
        int d1 = (pages[curr_page].height - scrolled) / pages[curr_page].width;
        if (curr_page == arrlen(pages)-1 || d1 > d)
            snprintf(string, 32, "%d/%d", curr_page+1, arrlen(pages));
        else if (curr_page == arrlen(pages)-2 || d1 + pages[curr_page+1].height / pages[curr_page+1].width > d)
            snprintf(string, 32, "%d-%d/%d", curr_page+1, curr_page+2, arrlen(pages));
        else
            snprintf(string, 32, "%d-%d/%d", curr_page+1, curr_page+3, arrlen(pages));
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
