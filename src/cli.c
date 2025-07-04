#include "headers.h"

extern char **files;

static char **opts = nullptr;
static char **args = nullptr;
static char *curr_dir = nullptr;
static char pathsep = '\0';


void get_args(int argc, char **argv)
{
    for (int i = 1; i < argc; i++)
        if (argv[i][0] == '-')
            arrput(opts, strdup(argv[i]+1));
        else
            arrput(args, strdup(argv[i]));
}

void generate_playlist(char *rel_path)
{
    char *dir_path = malloc(strlen(rel_path) + 1);
    sprintf(dir_path, "%s%c", rel_path, pathsep);

    char *file_path = malloc(strlen(dir_path) + strlen(".mgr"));
    sprintf(file_path, "%s.mgr", dir_path);

    int n;
    char **files = SDL_GlobDirectory(dir_path, "*.cbz", 0, &n);
    assert(files != nullptr);

    nat_sort(files, n);

    FILE *fh = fopen(file_path, "w");
    assert(fh != nullptr);
    for (char **t = files; *t != nullptr; t++)
        fprintf(fh, "%s\n", *t);
    fclose(fh);

    free(dir_path);
    free(file_path);
    SDL_free(files);
}

bool process_args()
{
    for (int i = 0; i < arrlen(opts); i++) {
        if (strcmp(opts[i], "g") == 0 || strcmp(opts[i], "generate") == 0) {
            generate_playlist(arrlen(args) > 0 ? args[0] : ".");
            return true;
        } else
            assert(false);
    }

    if (arrlen(args) == 0)
        arrput(args, strdup("."));

    if (arrlen(args) == 1) {
        SDL_PathInfo info;
        SDL_GetPathInfo(args[0], &info);

        if (info.type == SDL_PATHTYPE_DIRECTORY) {
            char *file_path = malloc(strlen(args[0]) + 1 + strlen(".mgr"));
            sprintf(file_path, "%s%c.mgr", args[0], pathsep);

            FILE *fh = fopen(file_path, "r");
            assert(fh != nullptr);
            char line[4096];
            while (fgets(line, sizeof(line), fh) != nullptr) {
                line[strlen(line) - 1] = '\0';
                char *file = malloc(strlen(args[0]) + 1 + strlen(line));
                sprintf(file, "%s%c%s", args[0], pathsep, line);
                arrput(files, file);
            }
            fclose(fh);

            free(file_path);

        } else if (info.type == SDL_PATHTYPE_FILE)
            arrput(files, strdup(args[0]));

        else
            assert(false);
    }

    else
        for (int i = 0; i < arrlen(args); i++)
            arrput(files, strdup(args[i]));

    return false;
}

void cleanup()
{
    for (int i = 0; i < arrlen(opts); i++)
        free(opts[i]);
    arrfree(opts);

    for (int i = 0; i < arrlen(args); i++)
        free(args[i]);
    arrfree(args);
}

bool cli(int argc, char **argv)
{
    curr_dir = SDL_GetCurrentDirectory();
    pathsep = curr_dir[strlen(curr_dir) - 1];

    get_args(argc, argv);
    bool done = process_args();
    cleanup();

    return done;
}
