#include "headers.h"

#include <getopt.h>

extern char *dirpath;
extern char **files;
extern bool readlist;
extern struct config conf;

bool gen = false;
static char **args = nullptr;


void cleanup()
{
    for (int i = 0; i < arrlen(args); i++)
        free(args[i]);
    arrfree(args);
}

void get_args(const int argc, char *const argv[])
{
    struct option longopts[] = {
        {"generate", no_argument,       nullptr, 'g'},
        {"help",     no_argument,       nullptr, 'h'},
        {"mode",     required_argument, nullptr, 'm'},
    };

    int c;
    while ((c = getopt_long(argc, argv, "gm:h", longopts, nullptr)) != -1) {
        switch (c) {
        case 'g':
            gen = true;
            break;

        case 'm':
            if (strcmp(optarg, "single") == 0)
                conf.mode = SINGLE;
            else if (strcmp(optarg, "book") == 0)
                conf.mode = BOOK;
            else if (strcmp(optarg, "strip") == 0)
                conf.mode = STRIP;
            else
                assert(false);
            conf.automode = strcmp(optarg, "auto") == 0;
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

    if (gen) {
        assert(arrlen(args) == 1);
    }
}

void process_args(struct appstate *s)
{
    if (gen) {
        dirpath = strdup(args[0]);
        generate_readlist(s);
        cleanup();
        exit(EXIT_SUCCESS);
    }

    if (arrlen(args) == 1) {
        SDL_PathInfo info;
        SDL_GetPathInfo(args[0], &info);

        if (info.type == SDL_PATHTYPE_DIRECTORY) {
            dirpath = strdup(args[0]);
            read_readlist(s);
            readlist = true;
        }
        else if (info.type == SDL_PATHTYPE_FILE) {
            dirpath = strdup(".");
            arrput(files, strdup(args[0]));
        }
        else
            assert(false);
    }

    else {
        dirpath = strdup(".");
        for (int i = 0; i < arrlen(args); i++)
            arrput(files, strdup(args[i]));
    }

    cleanup();
}
