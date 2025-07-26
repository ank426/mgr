#include "headers.h"

extern char *dirpath;
extern char **files;

static char *filepath;

void calc_filepath()
{
    filepath = malloc(strlen(dirpath) + strlen("/.mgr") + 1);
    sprintf(filepath, "%s/.mgr", dirpath);
}

void free_filepath()
{
    free(filepath);
}

void write_readlist(char *filename, int page, float scroll)
{
    FILE *const fh = fopen(filepath, "w");
    assert(fh != nullptr);

    fprintf(fh, "current file: %s\n", filename);
    fprintf(fh, "current page: %d\n", page + 1);
    fprintf(fh, "current scroll: %f\n", scroll);

    fputc('\n', fh);
    fputs("readlist:\n", fh);
    for (char **t = files; *t != nullptr; t++)
        fprintf(fh, "%s\n", *t);

    fclose(fh);
}

void generate_readlist(struct appstate *s)
{
    calc_filepath();

    char *filename = nullptr;

    FILE *fh;
    if ((fh = fopen(filepath, "r")) != nullptr) {
        fclose(fh);
        read_readlist(s);
        filename = files[s->file];
    }

    int n;
    files = SDL_GlobDirectory(dirpath, "*.cbz", 0, &n);
    assert(files != nullptr);
    nat_sort(files, n);

    write_readlist(filename != nullptr ? filename : files[0], s->start, s->scroll);

    free(files);
}

void read_readlist(struct appstate *s)
{
    calc_filepath();

    FILE *const fh = fopen(filepath, "r");
    assert(fh != nullptr);

    char filename[4096];

    fscanf(fh, " ");
    assert(fscanf(fh, "current file: %[^\n] ", filename) == 1);
    assert(fscanf(fh, "current page: %d ", &s->start) == 1);
    assert(fscanf(fh, "current scroll: %f ", &s->scroll) == 1);
    s->start--;
    assert(strlen(filename) != 0 && s->start >= 0 && s->scroll >= 0);

    int n = -1;
    fscanf(fh, "readlist:%n ", &n);
    assert(n == strlen("readlist:"));

    s->file = -1;
    int i = 0;
    char line[4096];
    while (fgets(line, sizeof(line), fh) != nullptr) {
        line[strlen(line) - 1] = '\0';
        arrput(files, strdup(line));

        if (strcmp(line, filename) == 0) {
            assert(s->file == -1);
            s->file = i;
        }
        i++;
    }
    assert(s->file >= 0);

    fclose(fh);
}
