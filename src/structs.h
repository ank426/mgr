#pragma once

struct page {
    char *name;
    float width, height;
    bool wide;
};

struct interval {
    int start, end;
    bool offset;
};

enum modes {
    SINGLE,
    BOOK,
    STRIP,
};
