#include "headers.h"

extern char **files;
extern bool gen;

static char *path = nullptr;

void update_readlist(struct appstate *s)
{
    char *file_path = malloc(strlen(path) + strlen("/.mgr") + 1);
    sprintf(file_path, "%s/.mgr", path);

    FILE *const fh = fopen(file_path, "w");
    assert(fh != nullptr);

    fprintf(fh, "current file: %s\n", files[s->file] + (gen ? 0 : strlen(path) + 1));
    fprintf(fh, "current page: %d\n", s->page + 1);
    fprintf(fh, "current scroll: %f\n", s->scroll);

    fputc('\n', fh);
    fputs("readlist:\n", fh);
    for (char **t = files; *t != nullptr; t++)
        fprintf(fh, "%s\n", *t + (gen ? 0 : strlen(path) + 1));

    fclose(fh);
    free(file_path);
}

void generate_readlist(const char *const _path, struct appstate *s)
{
    path = strdup(_path);

    int n;
    files = SDL_GlobDirectory(path, "*.cbz", 0, &n);
    assert(files != nullptr);
    nat_sort(files, n);

    s->page = 0;
    s->scroll = 0;

    update_readlist(s);

    free(files);
    free_path();
}

void read_readlist(const char *const _path, struct appstate *s)
{
    path = strdup(_path);

    char *file_path = malloc(strlen(path) + strlen("/.mgr") + 1);
    sprintf(file_path, "%s/.mgr", path);

    FILE *const fh = fopen(file_path, "r");
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
        char *file = malloc(strlen(path) + 1 + strlen(line) + 1);
        sprintf(file, "%s/%s", path, line);
        arrput(files, file);

        if (strcmp(line, filename) == 0) {
            assert(s->file == -1);
            s->file = i;
        }
        i++;
    }
    assert(s->file >= 0);

    fclose(fh);
    free(file_path);
}

void free_path()
{
    free(path);
}
