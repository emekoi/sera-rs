use super::*;
use super::util::*;

pub fn basic(b: &mut Buffer, src: &Buffer, mut x: i32, mut y: i32, mut sub: Rect) {
    /* Clip to destination buffer */
    clip_rect_offset(&mut sub, &mut x, &mut y, b.clip);
    /* Clipped off screen? */
    if sub.w <= 0 || sub.h <= 0 {
        return;
    }
    /* Copy pixels */
    for i in 0..sub.h {
        for j in 0..sub.w {
            b.pixels[(x + (y + i) * b.w + j) as usize] =
                src.pixels[(sub.x + (sub.y + i) * src.w + j) as usize];
        }
    }
}

pub fn scaled(
    b: &mut Buffer,
    src: &Buffer,
    mut x: i32,
    mut y: i32,
    mut sub: Rect,
    scalex: f32,
    scaley: f32,
) {
    let mut width = (sub.w as f32 * scalex) as i32;
    let mut height = (sub.h as f32 * scaley) as i32;
    let inx = (FX_UNIT_12 as f32 / scalex) as i32;
    let iny = (FX_UNIT_12 as f32 / scaley) as i32;
    /* Clip to destination buffer */
    let _d = b.clip.x - x;
    if _d > 0 {
        x += _d;
        sub.x += (_d as f32 / scalex) as i32;
        width -= _d;
    }
    let _d = b.clip.y - y;
    if _d > 0 {
        y += _d;
        sub.y += (_d as f32 / scaley) as i32;
        height -= _d;
    }
    let _d = (x + width) - (b.clip.x + b.clip.w);
    if _d > 0 {
        width -= _d;
    }
    let _d = (y + height) - (b.clip.y + b.clip.h);
    if _d > 0 {
        height -= _d;
    }
    /* Clipped offscreen? */
    if width == 0 || height == 0 {
        return;
    }
    /* Draw */
    let mut sy = sub.y << FX_BITS_12;
    for dy in y..(y + height) {
        let mut sx = 0;
        let mut dx = x + b.w * dy;
        let edx = dx + width;
        while dx < edx {
            b.pixels[dx as usize] = src.pixels[(((sub.x >> FX_BITS_12) + src.w * (sy >> FX_BITS_12))
                                                   + (sx >> FX_BITS_12))
                                                   as usize];
            sx += inx;
            dx += 1;
        }
        sy += iny;
    }
}
