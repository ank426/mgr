#include "headers.h"

#include <pthread.h>

extern char *const dirpath;
extern char **const files;
extern SDL_Event event_refresh;

static const int load_before = 2;
static const int load_after = 4;
static SDL_Surface **surfaces = nullptr;
static int surf_start = 0;

static int *work_queue = nullptr;
static _Atomic bool active = false;

static const int num_threads = 4;
static pthread_t *threads = nullptr;

static pthread_cond_t cond = PTHREAD_COND_INITIALIZER;
static pthread_mutex_t queue_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_mutex_t surfaces_lock = PTHREAD_MUTEX_INITIALIZER;

void *job(void *arg)
{
    struct appstate *s = arg;
    while (active) {
        pthread_mutex_lock(&queue_lock);
        while (active && arrlen(work_queue) == 0)
            pthread_cond_wait(&cond, &queue_lock);
        if (!active) {
            pthread_mutex_unlock(&queue_lock);
            break;
        }
        int idx = work_queue[0];
        arrdel(work_queue, 0);
        pthread_mutex_unlock(&queue_lock);

        pthread_mutex_lock(&surfaces_lock);
        bool outside = idx < surf_start || surf_start + arrlen(surfaces) <= idx;
        pthread_mutex_unlock(&surfaces_lock);
        if (outside)
            continue;

        char *relpath = files[s->file];
        char *path = malloc(strlen(dirpath) + 1 + strlen(relpath) + 1);
        sprintf(path, "%s/%s", dirpath, relpath);
        SDL_Surface *image = load_image_from_zip(s->pages[idx].name, path);
        free(path);

        pthread_mutex_lock(&surfaces_lock);
        if (surf_start <= idx && idx < surf_start + arrlen(surfaces)) {
            SDL_DestroySurface(surfaces[idx - surf_start]);
            surfaces[idx - surf_start] = image;
        } else
            outside = true;
        pthread_mutex_unlock(&surfaces_lock);

        if (outside)
            SDL_DestroySurface(image);

        SDL_PushEvent(&event_refresh);
    }
    return nullptr;
}

void threads_init(struct appstate *s)
{
    active = true;
    threads = malloc(sizeof(pthread_t[num_threads]));
    for (int i = 0; i < num_threads; i++)
        pthread_create(threads+i, nullptr, job, s);
}

void threads_free()
{
    if (!active)
        return;

    active = false;

    pthread_mutex_lock(&queue_lock);
    pthread_cond_broadcast(&cond);
    pthread_mutex_unlock(&queue_lock);

    for (int i = 0; i < num_threads; i++)
        pthread_join(threads[i], nullptr);
    free(threads);

    arrfree(work_queue);

    for (int i = 0; i < arrlen(surfaces); i++)
        SDL_DestroySurface(surfaces[i]);
    arrfree(surfaces);
}

void update_surfaces(struct appstate *s)
{
    const int start = fmaxf(s->start - load_before, 0);
    const int end = fminf(s->end + load_after, arrlen(s->pages) - 1);

    pthread_mutex_lock(&surfaces_lock);

    while (arrlen(surfaces) > 0 && end < surf_start + arrlen(surfaces) - 1) {
        SDL_DestroySurface(arrlast(surfaces));
        arrpop(surfaces);
    }

    while (arrlen(surfaces) > 0 && surf_start < start) {
        SDL_DestroySurface(surfaces[0]);
        arrdel(surfaces, 0);
        surf_start++;
    }

    if (arrlen(surfaces) == 0)
        surf_start = start;

    pthread_mutex_lock(&queue_lock);

    while (surf_start + arrlen(surfaces) <= end) {
        arrput(work_queue, surf_start + arrlen(surfaces));
        arrput(surfaces, nullptr);
    }

    while (start < surf_start) {
        arrput(work_queue, --surf_start);
        arrins(surfaces, 0, nullptr);
    }

    pthread_cond_signal(&cond);
    pthread_mutex_unlock(&queue_lock);
    pthread_mutex_unlock(&surfaces_lock);
}

SDL_Surface *get_surface(int idx)
{
    SDL_Surface *surf;
    pthread_mutex_lock(&surfaces_lock);
    if (surf_start <= idx && idx < surf_start + arrlen(surfaces))
        surf = surfaces[idx - surf_start];
    else
        surf = nullptr;
    pthread_mutex_unlock(&surfaces_lock);
    return surf;
}
