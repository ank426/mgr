#include "headers.h"

extern char *dirpath;
extern char **files;

void update_readlist(int file, int page, float scroll)
{
    char *filepath = malloc(strlen(dirpath) + strlen("/.mgr") + 1);
    sprintf(filepath, "%s/.mgr", dirpath);

    FILE *const fh = fopen(filepath, "w");
    assert(fh != nullptr);

    fprintf(fh, "current file: %s\n", files[file]);
    fprintf(fh, "current page: %d\n", page + 1);
    fprintf(fh, "current scroll: %f\n", scroll);

    fputc('\n', fh);
    fputs("readlist:\n", fh);
    for (char **t = files; *t != nullptr; t++)
        fprintf(fh, "%s\n", *t);

    fclose(fh);
    free(filepath);
}

void generate_readlist()
{
    int n;
    files = SDL_GlobDirectory(dirpath, "*.cbz", 0, &n);
    assert(files != nullptr);
    nat_sort(files, n);

    update_readlist(0, 0, 0);

    free(files);
    free_path();
}

void read_readlist(struct appstate *s)
{
    char *filepath = malloc(strlen(dirpath) + strlen("/.mgr") + 1);
    sprintf(filepath, "%s/.mgr", dirpath);

    FILE *const fh = fopen(filepath, "r");
    assert(fh != nullptr);

    char filename[4096];

    fscanf(fh, " ");
    assert(fscanf(fh, "current file: %[^\n] ", filename) == 1);
    assert(fscanf(fh, "current page: %d ", &s->page) == 1);
    assert(fscanf(fh, "current scroll: %f ", &s->scroll) == 1);
    s->page--;
    assert(strlen(filename) != 0 && s->page >= 0 && s->scroll >= 0);

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
    free(filepath);
}

void free_path()
{
    free(dirpath);
}
