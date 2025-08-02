#include "headers.h"

#define STB_IMAGE_IMPLEMENTATION
#include <stb/stb_image.h>
#include <zip.h>

void update_pages_from_zip(struct page **const ptr_pages, const char *const path)
{
    int err;
    zip_t *archive = zip_open(path, ZIP_RDONLY, &err);
    assert(archive != nullptr);

    zip_int64_t total_entries = zip_get_num_entries(archive, 0);
    assert(total_entries >= 0);

    arrfree(*ptr_pages);

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

        arrput(*ptr_pages, ((struct page){strdup(stat.name), w, h, (float)h/w}));

        free(buffer);
        zip_fclose(file);
    }

    zip_close(archive);
}

SDL_Surface *load_image_from_zip(char *const pagename, const char *const path)
{
    int err;
    zip_t *archive = zip_open(path, ZIP_RDONLY, &err);
    assert(archive != nullptr);

    zip_stat_t stat;
    assert(zip_stat(archive, pagename, 0, &stat) == 0);

    zip_file_t *file = zip_fopen(archive, pagename, 0);
    assert(file != nullptr);

    char *buffer = malloc(stat.size);
    assert(buffer != nullptr);

    zip_int64_t bytes_read = zip_fread(file, buffer, stat.size);
    assert(bytes_read >= 0);

    SDL_IOStream *io = SDL_IOFromConstMem(buffer, bytes_read);
    assert(io != nullptr);

    SDL_Surface *image = IMG_Load_IO(io, true);
    assert(image != nullptr);

    free(buffer);
    zip_fclose(file);
    zip_close(archive);

    return image;
}
