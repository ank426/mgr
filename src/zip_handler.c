#include "headers.h"
#include "globals.h"

#define STB_IMAGE_IMPLEMENTATION
#include <stb/stb_image.h>
#include <zip.h>

int get_num_entries_from_zip()
{
    int err;
    zip_t *archive = zip_open(path, ZIP_RDONLY, &err);
    assert(archive != NULL);

    int num = zip_get_num_entries(archive, 0);
    assert(num >= 0);

    zip_close(archive);

    return num;
}

void update_files_from_zip()
{
    if (pages != NULL) free(pages);
    pages = malloc(total_pages * sizeof(struct page));
    assert(pages != NULL);

    int err;
    zip_t *archive = zip_open(path, ZIP_RDONLY, &err);
    assert(archive != NULL);

    for (int i = 0; i < total_pages; i++) {
        zip_stat_t stat;
        assert(zip_stat_index(archive, i, 0, &stat) == 0);

        zip_file_t *file = zip_fopen_index(archive, i, 0);
        assert(file != NULL);

        unsigned char buffer[stat.size];
        zip_int64_t bytes_read = zip_fread(file, buffer, stat.size);

        int w, h, channels;
        assert(stbi_info_from_memory(buffer, stat.size, &w, &h, &channels));

        strncpy(pages[i].name, stat.name, 256);
        pages[i].width = w;
        pages[i].height = h;
        pages[i].wide = w > h;

        zip_fclose(file);
    }

    zip_close(archive);
}

SDL_Texture *load_image_from_zip(int index)
{
    int err;
    zip_t *archive = zip_open(path, ZIP_RDONLY, &err);
    assert(archive != NULL);

    zip_stat_t stat;
    assert(zip_stat(archive, pages[index].name, 0, &stat) == 0);

    zip_file_t *file = zip_fopen(archive, pages[index].name, 0);
    assert(file != NULL);

    char buffer[stat.size];
    long bytes_read = zip_fread(file, buffer, stat.size);
    assert(bytes_read >= 0);

    SDL_IOStream *io = SDL_IOFromConstMem(buffer, bytes_read);
    assert(io != NULL);

    SDL_Texture *image = IMG_LoadTexture_IO(renderer, io, true);
    assert(image != NULL);

    zip_fclose(file);
    zip_close(archive);

    return image;
}
