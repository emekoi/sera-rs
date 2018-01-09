/**
 * Copyright (c) 2015 rxi
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms of the MIT license. See LICENSE for details.
 */

#ifndef SERA_H
#define SERA_H

#if SR_MODE_RGBA
  #define SR_CHANNELS r, g, b, a
  #define SR_RGB_MASK 0xffffff
#elif SR_MODE_ARGB
  #define SR_CHANNELS a, r, g, b
  #define SR_RGB_MASK 0xffffff00
#elif SR_MODE_ABGR
  #define SR_CHANNELS a, b, g, r
  #define SR_RGB_MASK 0xffffff00
#else
  #define SR_CHANNELS b, g, r, a
  #define SR_RGB_MASK 0xffffff
#endif

typedef union {
  unsigned int word;
  struct { unsigned char SR_CHANNELS; } rgba;
} sr_Pixel; /* DONE */

typedef struct {
  int x, y, w, h;
} sr_Rect; /* DONE */

typedef struct {
  sr_Pixel color;
  unsigned char alpha, blend;
} sr_DrawMode; /* DONE */

typedef struct {
  float ox, oy, r, sx, sy;
} sr_Transform; /* DONE */

typedef struct {
  sr_DrawMode mode;
  sr_Rect clip;
  sr_Pixel *pixels;
  int w, h;
  char flags;
} sr_Buffer; /* DONE */

enum {
  SR_FMT_BGRA, /* DONE */
  SR_FMT_RGBA, /* DONE */
  SR_FMT_ARGB, /* DONE */
  SR_FMT_ABGR /* DONE */
};

enum {
  SR_BLEND_ALPHA, /* DONE */
  SR_BLEND_COLOR, /* DONE */
  SR_BLEND_ADD, /* DONE */
  SR_BLEND_SUBTRACT, /* DONE */
  SR_BLEND_MULTIPLY, /* DONE */
  SR_BLEND_LIGHTEN, /* DONE */
  SR_BLEND_DARKEN, /* DONE */
  SR_BLEND_SCREEN, /* DONE */
  SR_BLEND_DIFFERENCE /* DONE */
};


/* DONE */ sr_Pixel sr_pixel(int r, int g, int b, int a);
/* DONE */ sr_Pixel sr_color(int r, int g, int b);
/* DONE */ sr_Transform sr_transform(float ox, float oy, float r, float sx, float sy);
/* DONE */ sr_Rect sr_rect(int x, int y, int w, int h);

/* DONE */ sr_Buffer *sr_newBuffer(int w, int h);
/* DONE */ sr_Buffer *sr_cloneBuffer(sr_Buffer *src);

/* DONE */ void sr_loadPixels(sr_Buffer *b, void *src, int fmt);
/* DONE */ void sr_loadPixels8(sr_Buffer *b, unsigned char *src, sr_Pixel *pal);

/* DONE */ void sr_setAlpha(sr_Buffer* b, int alpha);
/* DONE */ void sr_setBlend(sr_Buffer* b, int blend);
/* DONE */ void sr_setColor(sr_Buffer* b, sr_Pixel c);
/* DONE */ void sr_setClip(sr_Buffer *b, sr_Rect r);
/* DONE */ void sr_reset(sr_Buffer *b);

/* DONE */ void sr_clear(sr_Buffer *b, sr_Pixel c);
/* DONE */ sr_Pixel sr_getPixel(sr_Buffer *b, int x, int y);
/* DONE */ void sr_setPixel(sr_Buffer *b, sr_Pixel c, int x, int y);
/* DONE */ void sr_copyPixels(sr_Buffer *b, sr_Buffer *src, int x, int y,
                   sr_Rect *sub, float sx, float sy);
/* DONE */ void sr_noise(sr_Buffer *b, unsigned seed, int low, int high, int grey);
/* DONE */ void sr_floodFill(sr_Buffer *b, sr_Pixel c, int x, int y);

/* DONE */ void sr_drawPixel(sr_Buffer *b, sr_Pixel c, int x, int y);
/* DONE */ void sr_drawLine(sr_Buffer *b, sr_Pixel c, int x0, int y0, int x1, int y1);
/* DONE */ void sr_drawRect(sr_Buffer *b, sr_Pixel c, int x, int y, int w, int h);
/* DONE */ void sr_drawBox(sr_Buffer *b, sr_Pixel c, int x, int y, int w, int h);
/* DONE */ void sr_drawCircle(sr_Buffer *b, sr_Pixel c, int x, int y, int r);
/* DONE */ void sr_drawRing(sr_Buffer *b, sr_Pixel c, int x, int y, int r);
void sr_drawBuffer(sr_Buffer *b, sr_Buffer *src, int x, int y,
                   sr_Rect *sub, sr_Transform *t);

#endif
