#include "headers.h"

extern SDL_Renderer *const renderer;
extern const int progress_font_size;
extern TTF_Text *const progress_text;

void display_single(SDL_Texture *image, double rotation, int width, int height)
{
    SDL_FRect dst;
    SDL_GetTextureSize(image, &dst.w, &dst.h);

    double c = fabs(cospi(rotation/180)), s = fabs(sinpi(rotation/180));
    double nw = dst.w * c + dst.h * s;
    double nh = dst.w * s + dst.h * c;
    float scale = fminf(width / nw, height / nh);

    dst.x = (width / scale - dst.w) / 2;
    dst.y = (height / scale - dst.h) / 2;

    SDL_SetRenderScale(renderer, scale, scale);
    SDL_RenderTextureRotated(renderer, image, nullptr, &dst, rotation, nullptr, SDL_FLIP_NONE);
}

void display_book(SDL_Texture *image1, SDL_Texture *image2, double rotation, int width, int height)
{
    SDL_FRect dst1, dst2;
    SDL_GetTextureSize(image1, &dst1.w, &dst1.h);
    SDL_GetTextureSize(image2, &dst2.w, &dst2.h);

    float s2_s1 = dst1.h / dst2.h;
    double w = dst1.w + dst2.w * s2_s1;
    double c = fabs(cospi(rotation/180)), s = fabs(sinpi(rotation/180));
    double nw = w * c + dst1.h * s;
    double nh = w * s + dst1.h * c;
    float scale1 = fminf(width / nw, height / nh);
    float scale2 = scale1 * s2_s1;

    dst1.x = (width / scale1 - dst1.w - dst2.w * s2_s1) / 2;
    dst2.x = (width / scale2 - dst2.w + dst1.w / s2_s1) / 2;
    dst1.y = (height / scale1 - dst1.h) / 2;
    dst2.y = (height / scale2 - dst2.h) / 2;

    SDL_FPoint center1 = {(dst1.w + dst2.w * s2_s1) / 2, dst1.h / 2};
    SDL_FPoint center2 = {(dst2.w - dst1.w / s2_s1) / 2, dst2.h / 2};

    SDL_SetRenderScale(renderer, scale1, scale1);
    SDL_RenderTextureRotated(renderer, image1, nullptr, &dst1, rotation, &center1, SDL_FLIP_NONE);
    SDL_SetRenderScale(renderer, scale2, scale2);
    SDL_RenderTextureRotated(renderer, image2, nullptr, &dst2, rotation, &center2, SDL_FLIP_NONE);
}

void display_strip(SDL_Texture **images, double rotation, float scroll,
                   float wzoom, float hzoom, int width, int height)
{
    double c = fabs(cospi(rotation/180)), s = fabs(sinpi(rotation/180));

    double wh = wzoom * width * c * c + hzoom * height * s * s;

    double tip_x = ((wh * c - width) / s + height) / 2;
    double tip_y = ((wh * s - height) / c + height) / 2;
    float distance = -scroll * wh + fmax(tip_x, tip_y);

    for (int i = 0; i < arrlen(images); i++) {
        SDL_Texture *image = images[i];

        SDL_FRect dst;
        SDL_GetTextureSize(image, &dst.w, &dst.h);

        float scale = wh / dst.w;
        dst.x = (width / scale - dst.w) / 2;
        dst.y = distance / scale;

        SDL_FPoint center = {dst.w / 2, (height / 2. - distance) / scale};

        SDL_SetRenderScale(renderer, scale, scale);
        SDL_RenderTextureRotated(renderer, image, nullptr, &dst, rotation, &center, SDL_FLIP_NONE);

        distance += dst.h * scale;
    }
}

void display_progress(int width, int height)
{
    int w, h;
    TTF_GetTextSize(progress_text, &w, &h);

    int x = (width - w) / 2;
    int y = (height - h) / 2;

    int mx = 0.3 * progress_font_size;
    int my = 0.1 * h;

    SDL_SetRenderScale(renderer, 1, 1);
    SDL_SetRenderDrawColorFloat(renderer, 0, 0, 0, 0.7);
    SDL_RenderFillRect(renderer, &(SDL_FRect){x - mx, y - my, w + 2*mx, h + 2*my});
    TTF_DrawRendererText(progress_text, x, y);
}

void display(struct appstate *s)
{
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);

    SDL_Texture **images = get_images(s);

    switch (s->mode) {
    case SINGLE:
        display_single(images[0], s->rotation, s->width, s->height);
        break;

    case BOOK:
        if (s->start == s->end)
            display_single(images[0], s->rotation, s->width, s->height);
        else
            display_book(images[1], images[0], s->rotation, s->width, s->height);
        break;

    case STRIP:
        display_strip(images, s->rotation, s->scroll, s->wzoom, s->hzoom, s->width, s->height);
        break;
    }

    if (s->show_progress) {
        update_progress_text(s);
        display_progress(s->width, s->height);
    }

    SDL_RenderPresent(renderer);
}
