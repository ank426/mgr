#include "headers.h"
#include "globals.h"

#define STB_IMAGE_IMPLEMENTATION
#include <stb/stb_image.h>
#include <zip.h>

int update_pages_from_zip()
{
    int err;
    zip_t *archive = zip_open(files[curr_file], ZIP_RDONLY, &err);
    assert(archive != nullptr);

    int total_entries = zip_get_num_entries(archive, 0);
    assert(total_entries >= 0);

    free(pages);
    pages = malloc(total_entries * sizeof(struct page));
    assert(pages != nullptr);

    int total_pages = 0;

    for (int i = 0; i < total_entries; i++) {
        zip_stat_t stat;
        assert(zip_stat_index(archive, i, 0, &stat) == 0);

        if (stat.size == 0) // directories
            continue;

        zip_file_t *file = zip_fopen_index(archive, i, 0);
        assert(file != nullptr);

        unsigned char *buffer = malloc(stat.size); // stack can't handle large allocations
        assert(buffer != nullptr);

        zip_int64_t bytes_read = zip_fread(file, buffer, stat.size);
        assert(bytes_read >= 0);

        int w, h, channels;
        assert(stbi_info_from_memory(buffer, stat.size, &w, &h, &channels));

        strncpy(pages[total_pages].name, stat.name, 256);
        pages[total_pages].width = w;
        pages[total_pages].height = h;
        pages[total_pages].wide = w > h;

        total_pages++;

        free(buffer);
        zip_fclose(file);
    }

    zip_close(archive);

    pages = realloc(pages, total_pages * sizeof(struct page));
    return total_pages;
}

SDL_Texture *load_image_from_zip(int index)
{
    int err;
    zip_t *archive = zip_open(files[curr_file], ZIP_RDONLY, &err);
    assert(archive != nullptr);

    zip_stat_t stat;
    assert(zip_stat(archive, pages[index].name, 0, &stat) == 0);

    zip_file_t *file = zip_fopen(archive, pages[index].name, 0);
    assert(file != nullptr);

    char *buffer = malloc(stat.size);
    assert(buffer != nullptr);

    long bytes_read = zip_fread(file, buffer, stat.size);
    assert(bytes_read >= 0);

    SDL_IOStream *io = SDL_IOFromConstMem(buffer, bytes_read);
    assert(io != nullptr);

    SDL_Texture *image = IMG_LoadTexture_IO(renderer, io, true);
    assert(image != nullptr);

    free(buffer);
    zip_fclose(file);
    zip_close(archive);

    return image;
}
