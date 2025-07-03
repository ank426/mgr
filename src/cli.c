#include "headers.h"

extern char **files;

static char **opts;
static char **args;

void cleanup()
{
    arrfree(opts);
    arrfree(args);
}

void generate_playlist(char *rel_path)
{
    char *path = SDL_GetCurrentDirectory();
    char path_sep = path[strlen(path) - 1];
    path = realloc(path, strlen(path) + strlen(rel_path) + 1);
    strcat(strcat(path, rel_path), (char []){path_sep, '\0'});

    int n;
    char **files = SDL_GlobDirectory(path, "*.cbz", 0, &n);
    assert(files != nullptr);

    nat_sort(files, n);

    FILE *fh = fopen(strcat(path, ".mgr"), "w");
    for (char **t = files; *t != nullptr; t++)
        fprintf(fh, "%s\n", *t);
    fclose(fh);

    SDL_free(files);
}

void get_args(int argc, char **argv)
{
    for (int i = 1; i < argc; i++)
        if (argv[i][0] == '-')
            arrput(opts, strdup(argv[i]+1));
        else
            arrput(args, strdup(argv[i]));
}

bool process_args()
{
    for (int i = 0; i < arrlen(opts); i++) {
        if (strcmp(opts[i], "g") == 0 || strcmp(opts[i], "generate") == 0) {
            generate_playlist(arrlen(args) > 0 ? args[0] : "");
            cleanup();
            return true;
        } else
            assert(false);
    }

    if (arrlen(args) == 0)
        arrput(args, strdup("."));

    if (arrlen(args) == 1) {
        char *path = SDL_GetCurrentDirectory();
        char path_sep = path[strlen(path)-1];
        path = realloc(path, strlen(path) + strlen(args[0]));
        strcat(path, args[0]);
        SDL_PathInfo info;
        SDL_GetPathInfo(path, &info);

        if (info.type == SDL_PATHTYPE_DIRECTORY) {
            path = strcat(path, (char []){path_sep, '\0'});
            char cache_path[strlen(path) + strlen(".mgr")];
            strcpy(cache_path, path);
            strcat(cache_path, ".mgr");
            FILE *file = fopen(cache_path, "r");
            assert(file != nullptr);
            char line[4096];
            while (fgets(line, sizeof(line), file)) {
                line[strlen(line)-1] = '\0';
                arrput(files, strdup(line));
            }
            fclose(file);

        } else if (info.type == SDL_PATHTYPE_FILE) {
            arrput(files, strdup(args[0]));
            cleanup();
            return false;

        } else
            assert(false);

        SDL_free(path);
    }

    else
        for (int i = 0; i < arrlen(args); i++)
            arrput(files, strdup(args[i]));

    cleanup();
    return false;
}
