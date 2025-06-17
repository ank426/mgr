#include "zip_handler.h"

#include <SDL3/SDL.h>
#include <SDL3_image/SDL_image.h>
#include <zip.h>

#include <assert.h>

int get_num_entries_from_zip(const char *path)
{
    int err;
    zip_t *archive = zip_open(path, ZIP_RDONLY, &err);
    assert(archive != NULL);

    int num = zip_get_num_entries(archive, 0);
    assert(num >= 0);

    zip_close(archive);

    return num;
}

SDL_Texture *load_image_from_zip(const char *path, int index, SDL_Renderer *renderer)
{
    int err;
    zip_t *archive = zip_open(path, ZIP_RDONLY, &err);
    assert(archive != NULL);

    zip_stat_t stat;
    assert(zip_stat_index(archive, index, 0, &stat) == 0);

    zip_file_t *file = zip_fopen_index(archive, index, 0);
    assert(file != NULL);

    char buffer[stat.size];
    long bytes_read = zip_fread(file, buffer, stat.size);
    assert(bytes_read >= 0);

    SDL_IOStream *io = SDL_IOFromMem(buffer, bytes_read);
    assert(io != NULL);

    SDL_Texture *image = IMG_LoadTexture_IO(renderer, io, true);
    assert(image != NULL);

    zip_fclose(file);
    zip_close(archive);

    return image;
}
