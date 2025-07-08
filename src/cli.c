#include "headers.h"

#include <getopt.h>

extern bool playlist;
extern char **files;

static bool gen = false;
static char **args = nullptr;


void get_args(const int argc, char *const *const argv)
{
    struct option longopts[] = {
        {"generate", no_argument, nullptr, 'g'},
        {"help", no_argument, nullptr, 'h'},
    };

    int c;
    while ((c = getopt_long(argc, argv, "gh", longopts, nullptr)) != -1) {
        switch (c) {
        case 'g':
            gen = true;
            break;

        case 'h':
            printf("todo: help\n");
            exit(EXIT_SUCCESS);
            break;

        default:
            printf("todo: help\n");
            exit(EXIT_FAILURE);
        }
    }

    if (optind == argc)
        arrput(args, strdup("."));
    else
        while (optind < argc)
            arrput(args, strdup(argv[optind++]));

    if (gen)
        assert(arrlen(args) < 2);
}

void generate_playlist(const char *const path)
{
    char *file_path = malloc(strlen(path) + 6);
    sprintf(file_path, "%s/.mgr", path);

    int n;
    char **files = SDL_GlobDirectory(path, "*.cbz", 0, &n);
    assert(files != nullptr);

    nat_sort(files, n);

    FILE *const fh = fopen(file_path, "w");
    assert(fh != nullptr);
    for (char **t = files; *t != nullptr; t++)
        fprintf(fh, "%s\n", *t);
    fclose(fh);

    SDL_free(files);
    free(file_path);
}

void read_playlist(const char *const path)
{
    char *file_path = malloc(strlen(path) + 6);
    sprintf(file_path, "%s/.mgr", path);

    FILE *const fh = fopen(file_path, "r");
    assert(fh != nullptr);
    char line[4096];
    while (fgets(line, sizeof(line), fh) != nullptr) {
        line[strlen(line) - 1] = '\0';
        char *file = malloc(strlen(path) + strlen(line) + 2);
        sprintf(file, "%s/%s", path, line);
        arrput(files, file);
    }
    fclose(fh);

    free(file_path);
}

void process_args()
{
    if (arrlen(args) == 1) {
        SDL_PathInfo info;
        SDL_GetPathInfo(args[0], &info);

        if (info.type == SDL_PATHTYPE_DIRECTORY) {
            read_playlist(args[0]);
            playlist = true;
        } else if (info.type == SDL_PATHTYPE_FILE)
            arrput(files, strdup(args[0]));
        else
            assert(false);
    }

    else
        for (int i = 0; i < arrlen(args); i++)
            arrput(files, strdup(args[i]));
}

void cleanup()
{
    for (int i = 0; i < arrlen(args); i++)
        free(args[i]);
    arrfree(args);
}

void cli(const int argc, char *const *const argv)
{
    get_args(argc, argv);

    if (gen) {
        generate_playlist(args[0]);
        cleanup();
        exit(EXIT_SUCCESS);
    }

    process_args();
    cleanup();
}
