#include "headers.h"

#include <getopt.h>

extern bool readlist;
extern char **files;
extern bool automode;
extern enum modes mode;

bool gen = false;
static char **args = nullptr;


void get_args(const int argc, char *const *const argv)
{
    struct option longopts[] = {
        {"generate", no_argument, nullptr, 'g'},
        {"help", no_argument, nullptr, 'h'},
        {"mode", required_argument, nullptr, 'm'},
    };

    int c;
    while ((c = getopt_long(argc, argv, "gm:h", longopts, nullptr)) != -1) {
        switch (c) {
        case 'g':
            gen = true;
            break;

        case 'm':
            if (strcmp(optarg, "single") == 0)
                mode = SINGLE;
            else if (strcmp(optarg, "book") == 0)
                mode = BOOK;
            else if (strcmp(optarg, "strip") == 0)
                mode = STRIP;
            else
                assert(false);
            automode = strcmp(optarg, "auto") == 0;
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

void process_args()
{
    if (arrlen(args) == 1) {
        SDL_PathInfo info;
        SDL_GetPathInfo(args[0], &info);

        if (info.type == SDL_PATHTYPE_DIRECTORY) {
            read_readlist(args[0]);
            readlist = true;
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
        generate_readlist(args[0]);
        cleanup();
        exit(EXIT_SUCCESS);
    }

    process_args();
    cleanup();
}
