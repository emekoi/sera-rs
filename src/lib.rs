#[macro_use]
extern crate lazy_static;
extern crate stb_image;
/*
https://github.com/redox-os/rusttype/issues/61
*/

use std::path::Path;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::{fmt, mem, slice, f32};

use stb_image::image;

macro_rules! lerp {
  ($bits:expr, $a:expr, $b:expr, $p:expr) => {
      u32::from($a) + (((u32::from($b) - u32::from($a)) * u32::from($p)) >> $bits)
  };
}

macro_rules! tu32 {
    ($a:expr) => { u32::from($a) }
}

macro_rules! impl_add {
    ($type: ident, $add: expr) => {
        impl Add<$type> for $type {
            type Output = $type;
            fn add(self, rhs: $type) -> $type {
                $add(self, rhs)
            }
        }
    }
}

macro_rules! impl_sub {
    ($type: ident, $sub: expr) => {
        impl Sub<$type> for $type {
            type Output = $type;
            fn sub(self, rhs: $type) -> $type {
                $sub(self, rhs)
            }
        }
    }
}

macro_rules! impl_mul {
    ($type: ident, $mul: expr) => {
        impl Mul<$type> for $type {
            type Output = $type;
            fn mul(self, rhs: $type) -> $type {
                $mul(self, rhs)
            }
        }
    }
}

macro_rules! impl_div {
    ($type: ident, $div: expr) => {
        impl Div<$type> for $type {
            type Output = $type;
            fn div(self, rhs: $type) -> $type {
                $div(self, rhs)
            }
        }
    }
}

macro_rules! impl_neg {
    ($type: ident, $neg: expr) => {
        impl Neg for $type {
            type Output = $type;
            fn neg(self) -> $type {
                $neg(self)
            }
        }
    }
}

const FX_BITS_12: u32 = 12;
const FX_UNIT_12: u32 = 1 << FX_BITS_12;
// const FX_MASK_12: u32 = FX_UNIT_12 - 1;

const FX_BITS_10: u32 = 10;
const FX_UNIT_10: u32 = 1 << FX_BITS_10;
const FX_MASK_10: u32 = FX_UNIT_10 - 1;

const PI2: f32 = ::std::f32::consts::PI * 2f32;

#[inline]
fn xdiv_i32(n: i32, x: i32) -> i32 {
    match x {
        0 => n,
        _ => n / x,
    }
}

#[inline]
fn xdiv_f32(n: f32, x: f32) -> f32 {
    if x == 0f32 {
        n
    } else {
        n / x
    }
}

#[inline]
fn clip_rect(r: &mut Rect, to: &Rect) {
    let x1 = r.x.max(to.x);
    let y1 = r.y.max(to.y);
    let x2 = (r.x + r.w).min(to.x + to.w);
    let y2 = (r.y + r.h).min(to.y + to.h);
    r.x = x1;
    r.y = y1;
    r.w = (x2 - x1).max(0);
    r.h = (y2 - y1).max(0);
}

#[inline]
fn clip_rect_offset(r: &mut Rect, x: &mut i32, y: &mut i32, to: Rect) {
    let _d = to.x - *x;
    if _d > 0 {
        *x += _d;
        r.w -= _d;
        r.x += _d
    }
    let _d = to.y - *y;
    if _d > 0 {
        *y += _d;
        r.h -= _d;
        r.y += _d
    }
    let _d = (*x + r.w) - (to.x + to.w);
    if _d > 0 {
        r.w -= _d;
    }
    let _d = (*y + r.h) - (to.y + to.h);
    if _d > 0 {
        r.h -= _d;
    }
}

fn blend_pixel(m: &DrawMode, d: &mut Pixel, mut s: Pixel) {
    unsafe {
        let alpha = ((tu32!(s.rgba.a) * tu32!(m.alpha)) >> 8) as u8;
        if alpha <= 1 {
            return;
        }
        /* Color */
        if m.color != RGB_MASK {
            s.rgba.r = ((tu32!(s.rgba.r) * tu32!(m.color.rgba.r)) >> 8) as u8;
            s.rgba.g = ((tu32!(s.rgba.g) * tu32!(m.color.rgba.g)) >> 8) as u8;
            s.rgba.b = ((tu32!(s.rgba.b) * tu32!(m.color.rgba.b)) >> 8) as u8;
        }
        /* Blend */
        match m.blend {
            BlendMode::ALPHA => {}
            BlendMode::COLOR => s = m.color,
            BlendMode::ADD => {
                s.rgba.r = 0xff.min(tu32!(d.rgba.r) + tu32!(s.rgba.r)) as u8;
                s.rgba.g = 0xff.min(tu32!(d.rgba.g) + tu32!(s.rgba.g)) as u8;
                s.rgba.b = 0xff.min(tu32!(d.rgba.b) + tu32!(s.rgba.b)) as u8;
            }
            BlendMode::SUBTRACT => {
                s.rgba.r = 0i32.min(i32::from(d.rgba.r) - i32::from(s.rgba.r)) as u8;
                s.rgba.g = 0i32.min(i32::from(d.rgba.g) - i32::from(s.rgba.g)) as u8;
                s.rgba.b = 0i32.min(i32::from(d.rgba.b) - i32::from(s.rgba.b)) as u8;
            }
            BlendMode::MULTIPLY => {
                s.rgba.r = ((u32::from(s.rgba.r) * tu32!(d.rgba.r)) >> 8) as u8;
                s.rgba.g = ((u32::from(s.rgba.g) * tu32!(d.rgba.g)) >> 8) as u8;
                s.rgba.b = ((u32::from(s.rgba.b) * tu32!(d.rgba.b)) >> 8) as u8;
            }
            BlendMode::LIGHTEN => {
                s = if s.rgba.r + s.rgba.g + s.rgba.b > d.rgba.r + d.rgba.g + d.rgba.b {
                    s
                } else {
                    *d
                }
            }
            BlendMode::DARKEN => {
                s = if s.rgba.r + s.rgba.g + s.rgba.b < d.rgba.r + d.rgba.g + d.rgba.b {
                    s
                } else {
                    *d
                }
            }
            BlendMode::SCREEN => {
                s.rgba.r = 0xff - ((tu32!(0xff - d.rgba.r) * tu32!(0xff - s.rgba.r)) >> 8) as u8;
                s.rgba.g = 0xff - ((tu32!(0xff - d.rgba.g) * tu32!(0xff - s.rgba.g)) >> 8) as u8;
                s.rgba.b = 0xff - ((tu32!(0xff - d.rgba.b) * tu32!(0xff - s.rgba.b)) >> 8) as u8;
            }
            BlendMode::DIFFERENCE => {
                s.rgba.r = (i32::from(s.rgba.r) - i32::from(d.rgba.r)).abs() as u8;
                s.rgba.g = (i32::from(s.rgba.g) - i32::from(d.rgba.g)).abs() as u8;
                s.rgba.b = (i32::from(s.rgba.b) - i32::from(d.rgba.b)).abs() as u8;
            }
        }
        /* Write */
        if alpha >= 254 {
            *d = s;
        } else if d.rgba.a >= 254 {
            d.rgba.r = lerp!(8u32, d.rgba.r, s.rgba.r, alpha) as u8;
            d.rgba.g = lerp!(8u32, d.rgba.g, s.rgba.g, alpha) as u8;
            d.rgba.b = lerp!(8u32, d.rgba.b, s.rgba.b, alpha) as u8;
        } else {
            let a = (0xff - ((tu32!(0xff - d.rgba.a) * tu32!(0xff - alpha)) >> 8)) as u8;
            let _z = (tu32!(d.rgba.a * (0xff - alpha)) >> 8) as u8;
            d.rgba.r = DIV8_TABLE[(((tu32!(d.rgba.r) * tu32!(_z)) >> 8)
                                      + ((tu32!(s.rgba.r) * tu32!(alpha)) >> 8))
                                      as usize][a as usize];
            d.rgba.g = DIV8_TABLE[(((tu32!(d.rgba.g) * tu32!(_z)) >> 8)
                                      + ((tu32!(s.rgba.g) * tu32!(alpha)) >> 8))
                                      as usize][a as usize];
            d.rgba.b = DIV8_TABLE[(((tu32!(d.rgba.b) * tu32!(_z)) >> 8)
                                      + ((tu32!(s.rgba.b) * tu32!(alpha)) >> 8))
                                      as usize][a as usize];
            d.rgba.a = a;
        }
    }
    return;
}

mod copy_pixel {
    use super::*;

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
                b.pixels[dx as usize] =
                    src.pixels[(((sub.x >> FX_BITS_12) + src.w * (sy >> FX_BITS_12))
                                   + (sx >> FX_BITS_12)) as usize];
                sx += inx;
                dx += 1;
            }
            sy += iny;
        }
    }
}

mod draw_buffer {
    use super::*;

    pub fn basic(b: &mut Buffer, src: &Buffer, mut x: i32, mut y: i32, mut sub: Rect) {
        /* Clip to destination buffer */
        clip_rect_offset(&mut sub, &mut x, &mut y, b.clip);
        /* Clipped off screen? */
        if sub.w <= 0 || sub.h <= 0 {
            return;
        }
        /* Draw */
        for iy in 0..(sub.h as usize) {
            let (mut d_off, mut s_off) = (0, 0);
            for _ in (0..sub.w).rev() {
                blend_pixel(
                    &b.mode,
                    &mut b.pixels[(x + (y + iy as i32) * b.w + d_off) as usize],
                    src.pixels[(sub.x + (sub.y + iy as i32) * src.w + s_off) as usize],
                );
                d_off += 1;
                s_off += 1;
            }
        }
    }

    pub fn scaled(b: &mut Buffer, src: &Buffer, x: i32, y: i32, mut sub: Rect, t: Transform) {
        let abs_sx =
            if t.sx < 0.0 { -t.sx } else { t.sx };
        let abs_sy =
            if t.sy < 0.0 { -t.sy } else { t.sy };
        let mut width = (sub.w as f32 * abs_sx + 0.5).floor() as i32;
        let mut height = (sub.h as f32 * abs_sy + 0.5).floor() as i32;
        let osx = if t.sx < 0.0 {
            (sub.w << FX_BITS_12) - 1
        } else {
            0
        };
        let osy = if t.sy < 0.0 {
            (sub.h << FX_BITS_12) - 1
        } else {
            0
        };
        let ix = ((sub.w << FX_BITS_12) as f32 / t.sx / sub.w as f32) as i32;
        let iy = ((sub.h << FX_BITS_12) as f32 / t.sy / sub.h as f32) as i32;
        /* Adjust x/y depending on origin */
        let x =
            (x as f32
                - ((if t.sx < 0.0 { width } else { 0 }) - (if t.sx < 0.0 { -1 } else { 1 })) as f32
                    * t.ox * abs_sx) as i32;
        let y =
            (y as f32
                - ((if t.sy < 0.0 { height } else { 0 }) - (if t.sy < 0.0 { -1 } else { 1 })) as f32
                    * t.oy * abs_sy) as i32;
        /* Clipped completely offscreen horizontally? */
        if x + width < b.clip.x || x > b.clip.x + b.clip.w {
            return;
        }
        /* Adjust for clipping */
        let mut dy = 0;
        let mut odx = 0;
        let _d = b.clip.y - y;
        if _d > 0 {
            dy = _d;
            sub.y += (_d as f32 / t.sy) as i32;
        }
        let _d = b.clip.x - x;
        if _d > 0 {
            odx = _d;
            sub.x += (_d as f32 / t.sx) as i32;
        }
        let _d = (y + height) - (b.clip.y + b.clip.h);
        if _d > 0 {
            height -= _d;
        }
        let _d = (x + width) - (b.clip.x + b.clip.w);
        if _d > 0 {
            width -= _d;
        }
        /* Draw */
        let mut sy = osy;
        while dy < height {
            let mut dx = odx;
            let mut sx = osx;
            while dx < width {
                blend_pixel(
                    &b.mode,
                    &mut b.pixels[((x + dx) + (y + dy) * b.w) as usize],
                    src.pixels[((sub.x + (sx >> FX_BITS_12)) + (sub.y + (sy >> FX_BITS_12)) * src.w)
                                   as usize],
                );
                sx += ix;
                dx += 1;
            }
            sy += iy;
            dy += 1;
        }
    }

    fn scan_line(
        b: &mut Buffer,
        src: &Buffer,
        sub: &Rect,
        dy: i32,
        t: Transform,
        sx_incr: i32,
        sy_incr: i32,
    ) {
        /* Adjust for clipping */
        if dy < b.clip.y || dy >= b.clip.y + b.clip.h {
            return;
        }
        let mut left = t.ox as i32;
        let mut right = t.oy as i32;
        let mut sx = t.sx as i32;
        let mut sy = t.sy as i32;
        let _d = b.clip.x - left;
        if _d > 0 {
            left += _d;
            sx += _d * sx_incr;
            sy += _d * sy_incr;
        }
        let _d = right - (b.clip.x + b.clip.w);
        if _d > 0 {
            right -= _d;
        }
        let (mut dx, mut x, mut y);
        /* Does the scaline length go out of bounds of our `sub` rect? If so we
         * should adjust the scan line and the source coordinates accordingly */
        'checkSourceLeft: loop {
            x = sx >> FX_BITS_12;
            y = sy >> FX_BITS_12;
            if x < sub.x || y < sub.y || x >= sub.x + sub.w || y >= sub.y + sub.h {
                left += 1;
                sx += sx_incr;
                sy += sy_incr;
                if left >= right {
                    return;
                }
            } else {
                break 'checkSourceLeft;
            }
        }
        'checkSourceRight: loop {
            x = (sx + sx_incr * (right - left)) >> FX_BITS_12;
            y = (sy + sy_incr * (right - left)) >> FX_BITS_12;
            if x < sub.x || y < sub.y || x >= sub.x + sub.w || y >= sub.y + sub.h {
                right -= 1;
                if left >= right {
                    return;
                }
            } else {
                break 'checkSourceRight;
            }
        }
        /* Draw */
        dx = left;
        while dx < right {
            blend_pixel(
                &b.mode,
                &mut b.pixels[(dx + dy * b.w) as usize],
                src.pixels[((sx >> FX_BITS_12) + (sy >> FX_BITS_12) * src.w) as usize],
            );
            sx += sx_incr;
            sy += sy_incr;
            dx += 1;
        }
    }

    pub fn rotate_scaled(b: &mut Buffer, src: &Buffer, x: i32, y: i32, sub: Rect, t: Transform) {
        let mut points: [Point; 4] = [
            Point::new(0, 0),
            Point::new(0, 0),
            Point::new(0, 0),
            Point::new(0, 0),
        ];
        let cosr = t.r.cos();
        let sinr = t.r.sin();
        let abs_sx =
            if t.sx < 0f32 { -t.sx } else { t.sx };
        let abs_sy =
            if t.sy < 0f32 { -t.sy } else { t.sy };
        let inv_x = t.sx < 0f32;
        let inv_y = t.sy < 0f32;
        let width = (sub.w as f32 * abs_sx) as i32;
        let height = (sub.h as f32 * abs_sy) as i32;
        let _q = (t.r * 4f32 / PI2) as i32;
        let cosq = (_q as f32 * PI2 / 4f32).cos();
        let sinq = (_q as f32 * PI2 / 4f32).sin();
        let ox = (if inv_x {
            sub.w as f32 - t.ox
        } else {
            t.ox
        }) * abs_sx;
        let oy = (if inv_y {
            sub.h as f32 - t.oy
        } else {
            t.oy
        }) * abs_sy;
        /* Store rotated corners as points */
        points[0].x = x + (cosr * (-ox) - sinr * (-oy)) as i32;
        points[0].y = y + (sinr * (-ox) + cosr * (-oy)) as i32;
        points[1].x = x + (cosr * (-ox + width as f32) - sinr * (-oy)) as i32;
        points[1].y = y + (sinr * (-ox + width as f32) + cosr * (-oy)) as i32;
        points[2].x = x + (cosr * (-ox + width as f32) - sinr * (-oy + height as f32)) as i32;
        points[2].y = y + (sinr * (-ox + width as f32) + cosr * (-oy + height as f32)) as i32;
        points[3].x = x + (cosr * (-ox) - sinr * (-oy + height as f32)) as i32;
        points[3].y = y + (sinr * (-ox) + cosr * (-oy + height as f32)) as i32;
        /* Set named points based on rotation */
        let top = &points[((-_q) & 3) as usize];
        let right = &points[((-_q + 1) & 3) as usize];
        let bottom = &points[((-_q + 2) & 3) as usize];
        let left = &points[((-_q + 3) & 3) as usize];
        /* Clipped completely off screen? */
        if bottom.y < b.clip.y || top.y >= b.clip.y + b.clip.h {
            return;
        }
        if right.x < b.clip.x || left.x >= b.clip.x + b.clip.w {
            return;
        }
        /* Destination */
        let mut xr = top.x << FX_BITS_12;
        let mut xl = xr;
        let mut il = xdiv_i32((left.x - top.x) << FX_BITS_12, left.y - top.y);
        let mut ir = xdiv_i32((right.x - top.x) << FX_BITS_12, right.y - top.y);
        /* Source */
        let sxi = (xdiv_i32(sub.w << FX_BITS_12, width) as f32 * (-t.r).cos()) as i32;
        let syi = (xdiv_i32(sub.h << FX_BITS_12, height) as f32 * (-t.r).sin()) as i32;
        let mut sxoi = (xdiv_i32(sub.w << FX_BITS_12, left.y - top.y) as f32 * sinq) as i32;
        let mut syoi = (xdiv_i32(sub.h << FX_BITS_12, left.y - top.y) as f32 * cosq) as i32;
        let (mut sx, mut sy) = match _q {
            1 => (sub.x << FX_BITS_12, ((sub.y + sub.h) << FX_BITS_12) - 1),
            2 => (
                ((sub.x + sub.w) << FX_BITS_12) - 1,
                ((sub.y + sub.h) << FX_BITS_12) - 1,
            ),
            3 => (((sub.x + sub.w) << FX_BITS_12) - 1, sub.y << FX_BITS_12),
            _ => (sub.x << FX_BITS_12, sub.y << FX_BITS_12),
        };
        /* Draw */
        let mut dy = if left.y == top.y || right.y == top.y {
            /* Adjust for right-angled rotation */
            top.y - 1
        } else {
            top.y
        };
        while dy <= bottom.y {
            /* Invert source iterators & increments if we are scaled negatively */
            let (tsx, tsxi) = if inv_x {
                (((sub.x * 2 + sub.w) << FX_BITS_12) - sx - 1, -sxi)
            } else {
                (sx, sxi)
            };
            let (tsy, tsyi) = if inv_y {
                (((sub.y * 2 + sub.h) << FX_BITS_12) - sy - 1, -syi)
            } else {
                (sy, syi)
            };
            /* Draw row */
            scan_line(
                b,
                src,
                &sub,
                dy,
                Transform::new(
                    (xl >> FX_BITS_12) as f32,
                    (xr >> FX_BITS_12) as f32,
                    0f32,
                    tsx as f32,
                    tsy as f32,
                ),
                tsxi,
                tsyi,
            );
            sx += sxoi;
            sy += syoi;
            xl += il;
            xr += ir;
            dy += 1;
            /* Modify increments if we've reached the left or right corner */
            if dy == left.y {
                il = xdiv_i32((bottom.x - left.x) << FX_BITS_12, bottom.y - left.y);
                sxoi = (xdiv_i32(sub.w << FX_BITS_12, bottom.y - left.y) as f32 * cosq) as i32;
                syoi = (xdiv_i32(sub.h << FX_BITS_12, bottom.y - left.y) as f32 * -sinq) as i32;
            }
            if dy == right.y {
                ir = xdiv_i32((bottom.x - right.x) << FX_BITS_12, bottom.y - right.y);
            }
        }
    }
}

//const DEFAULT_FONT_DATA: &[u8] = include_bytes!("fonts/TinyUnicode.ttf");
//const DEFAULT_FONT_SIZE: usize = 16;

#[cfg(feature = "MODE_RGBA")]
const RGB_MASK: u32 = 0xff_ffff;
#[cfg(feature = "MODE_ARGB")]
const RGB_MASK: u32 = 0xffffff00;
#[cfg(feature = "MODE_ABGR")]
const RGB_MASK: u32 = 0xffffff00;

#[cfg(any(feature = "MODE_BGRA",
          all(not(feature = "MODE_RGBA"), not(feature = "MODE_ARGB"),
              not(feature = "MODE_ABGR"))))]
const RGB_MASK: u32 = 0xff_ffff;

lazy_static! {
    static ref DIV8_TABLE: [[u8; 256]; 256] = {
        let mut div8 = [[0; 256]; 256];
        for b in 1..256 {
            for (a, t) in div8.iter_mut().enumerate().take(256).skip(1) {
                t[b] = ((a << 8) / b) as u8;
            }
        }
        div8
    };

    static ref SIN_TABLE: [i32; FX_UNIT_10 as usize] = {
        let mut table = [0; FX_UNIT_10 as usize];
        for i in 0..FX_UNIT_10 {
            let tmp = ((i as f32 / FX_UNIT_10 as f32) * PI2).sin();
            table[i as usize] = (tmp * FX_UNIT_10 as f32) as i32;
        }
        table
    };
}

fn fxsin(n: i32) -> i32 {
    SIN_TABLE[(n & FX_MASK_10 as i32) as usize]
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PixelFormat {
    BGRA,
    RGBA,
    ARGB,
    ABGR,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BlendMode {
    ALPHA,
    COLOR,
    ADD,
    SUBTRACT,
    MULTIPLY,
    LIGHTEN,
    DARKEN,
    SCREEN,
    DIFFERENCE,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ColorChannel {
    R,
    B,
    G,
    A,
}

#[cfg(feature = "MODE_RGBA")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Channel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
#[cfg(feature = "MODE_ARGB")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Channel {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
#[cfg(feature = "MODE_ABGR")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Channel {
    pub a: u8,
    pub b: u8,
    pub g: u8,
    pub r: u8,
}
#[cfg(any(feature = "MODE_BGRA",
          all(not(feature = "MODE_RGBA"), not(feature = "MODE_ARGB"),
              not(feature = "MODE_ABGR"))))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Channel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl Channel {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Channel {
        Channel { r, g, b, a }
    }
}

impl_add!(Channel, |s: Channel, rhs: Channel| -> Channel {
    Channel {
        r: s.r.wrapping_add(rhs.r),
        g: s.g.wrapping_add(rhs.g),
        b: s.b.wrapping_add(rhs.b),
        a: s.a.wrapping_add(rhs.a),
    }
});

impl_sub!(Channel, |s: Channel, rhs: Channel| -> Channel {
    Channel {
        r: s.r.wrapping_sub(rhs.r),
        g: s.g.wrapping_sub(rhs.g),
        b: s.b.wrapping_sub(rhs.b),
        a: s.a.wrapping_sub(rhs.a),
    }
});

impl_mul!(Channel, |s: Channel, rhs: Channel| -> Channel {
    Channel {
        r: s.r.wrapping_mul(rhs.r),
        g: s.g.wrapping_mul(rhs.g),
        b: s.b.wrapping_mul(rhs.b),
        a: s.a.wrapping_mul(rhs.a),
    }
});

impl_div!(Channel, |s: Channel, rhs: Channel| -> Channel {
    Channel {
        r: match rhs.r {
            0 => 0,
            _ => s.r.wrapping_div(rhs.r),
        },
        g: match rhs.g {
            0 => 0,
            _ => s.g.wrapping_div(rhs.g),
        },
        b: match rhs.b {
            0 => 0,
            _ => s.b.wrapping_div(rhs.b),
        },
        a: match rhs.a {
            0 => 0,
            _ => s.a.wrapping_div(rhs.a),
        },
    }
});

#[derive(Clone, Copy)]
pub union Pixel {
    pub word: u32,
    pub rgba: Channel,
}

impl Pixel {
    pub fn pixel(r: u8, g: u8, b: u8, a: u8) -> Pixel {
        Pixel {
            rgba: Channel { r, g, b, a },
        }
    }

    pub fn color(r: u8, g: u8, b: u8) -> Pixel {
        Pixel {
            rgba: Channel { r, g, b, a: 0xff },
        }
    }
}

impl PartialEq<Pixel> for Pixel {
    fn eq(&self, other: &Pixel) -> bool {
        unsafe { self.word == other.word }
    }
}

impl PartialEq<u32> for Pixel {
    fn eq(&self, other: &u32) -> bool {
        unsafe { self.word == *other }
    }
}

impl fmt::Debug for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(
                f,
                "({}, {}, {}, {}) -> {}",
                self.rgba.r, self.rgba.g, self.rgba.b, self.rgba.a, self.word
            )
        }
    }
}

impl_add!(Pixel, |s: Pixel, rhs: Pixel| -> Pixel {
    unsafe {
        Pixel {
            rgba: s.rgba + rhs.rgba,
        }
    }
});

impl_sub!(Pixel, |s: Pixel, rhs: Pixel| -> Pixel {
    unsafe {
        Pixel {
            rgba: s.rgba - rhs.rgba,
        }
    }
});

impl_mul!(Pixel, |s: Pixel, rhs: Pixel| -> Pixel {
    unsafe {
        Pixel {
            rgba: s.rgba * rhs.rgba,
        }
    }
});

impl_div!(Pixel, |s: Pixel, rhs: Pixel| -> Pixel {
    unsafe {
        Pixel {
            rgba: s.rgba / rhs.rgba,
        }
    }
});

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }
}

impl_add!(Rect, |s: Rect, rhs: Rect| -> Rect {
    Rect {
        x: s.x + rhs.x,
        y: s.y + rhs.y,
        w: s.w + rhs.w,
        h: s.h + rhs.h,
    }
});

impl_sub!(Rect, |s: Rect, rhs: Rect| -> Rect {
    Rect {
        x: s.x - rhs.x,
        y: s.y - rhs.y,
        w: s.w - rhs.w,
        h: s.h - rhs.h,
    }
});

impl_mul!(Rect, |s: Rect, rhs: Rect| -> Rect {
    Rect {
        x: s.x * rhs.x,
        y: s.y * rhs.y,
        w: s.w * rhs.w,
        h: s.h * rhs.h,
    }
});

impl_div!(Rect, |s: Rect, rhs: Rect| -> Rect {
    Rect {
        x: xdiv_i32(s.x, rhs.x),
        y: xdiv_i32(s.y, rhs.y),
        w: xdiv_i32(s.w, rhs.w),
        h: xdiv_i32(s.h, rhs.h),
    }
});

impl_neg!(Rect, |s: Rect| -> Rect {
    Rect {
        x: -s.x,
        y: -s.y,
        w: -s.w,
        h: -s.h,
    }
});

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawMode {
    pub color: Pixel,
    pub blend: BlendMode,
    pub alpha: u8,
}

impl DrawMode {
    pub fn new(color: Pixel, blend: BlendMode, alpha: u8) -> DrawMode {
        DrawMode {
            color,
            blend,
            alpha,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform {
    pub ox: f32,
    pub oy: f32,
    pub r: f32,
    pub sx: f32,
    pub sy: f32,
}

impl Transform {
    pub fn new(ox: f32, oy: f32, r: f32, sx: f32, sy: f32) -> Transform {
        Transform { ox, oy, r, sx, sy }
    }
}

impl_add!(Transform, |s: Transform, rhs: Transform| -> Transform {
    Transform {
        ox: s.ox + rhs.ox,
        oy: s.oy + rhs.oy,
        r: s.r + rhs.r,
        sx: s.sx + rhs.sx,
        sy: s.sy + rhs.sy,
    }
});

impl_sub!(Transform, |s: Transform, rhs: Transform| -> Transform {
    Transform {
        ox: s.ox - rhs.ox,
        oy: s.oy - rhs.oy,
        r: s.r - rhs.r,
        sx: s.sx - rhs.sx,
        sy: s.sy - rhs.sy,
    }
});

impl_mul!(Transform, |s: Transform, rhs: Transform| -> Transform {
    Transform {
        ox: s.ox * rhs.ox,
        oy: s.oy * rhs.oy,
        r: s.r * rhs.r,
        sx: s.sx * rhs.sx,
        sy: s.sy * rhs.sy,
    }
});

impl_div!(Transform, |s: Transform, rhs: Transform| -> Transform {
    Transform {
        ox: xdiv_f32(s.ox, rhs.ox),
        oy: xdiv_f32(s.oy, rhs.oy),
        r: xdiv_f32(s.r, rhs.r),
        sx: xdiv_f32(s.sx, rhs.sy),
        sy: xdiv_f32(s.sy, rhs.sx),
    }
});

impl_neg!(Transform, |s: Transform| -> Transform {
    Transform {
        ox: -s.ox,
        oy: -s.oy,
        r: -s.r,
        sx: -s.sx,
        sy: -s.sy,
    }
});

#[derive(Debug, Clone, PartialEq)]
pub struct Buffer {
    pub mode: DrawMode,
    pub clip: Rect,
    pub pixels: Vec<Pixel>,
    pub w: i32,
    pub h: i32,
}

impl Buffer {
    pub fn new(w: i32, h: i32) -> Buffer {
        if w < 1 {
            panic!("expected width of 1 or greater")
        }
        if h < 1 {
            panic!("expected height of 1 or greater")
        }
        let black = Pixel::color(0, 0, 0);
        let mut buf = Buffer {
            w,
            h,
            clip: Rect::new(0, 0, w, h),
            pixels: vec![black; (w * h) as usize],
            mode: DrawMode::new(black, BlendMode::ALPHA, 0xff),
        };
        buf.reset();
        buf
    }

    pub fn file<T: AsRef<Path>>(file: T) -> Option<Buffer> {
        let res = image::load_with_depth(file, 4, false);
        if let image::LoadResult::ImageU8(img) = res {
            unsafe {
                let mut buf = Buffer::new(img.width as i32, img.height as i32);
                buf.load_pixels(
                    slice::from_raw_parts_mut(
                        mem::transmute(img.data.as_ptr()),
                        mem::size_of::<u8>() * img.data.len(),
                    ),
                    PixelFormat::RGBA,
                );
                return Some(buf);
            }
        }
        None
    }

    pub fn bytes<T: AsRef<[u8]>>(bytes: T) -> Option<Buffer> {
        let res = image::load_from_memory_with_depth(bytes.as_ref(), 4, false);
        if let image::LoadResult::ImageU8(img) = res {
            unsafe {
                let mut buf = Buffer::new(img.width as i32, img.height as i32);
                buf.load_pixels(
                    slice::from_raw_parts_mut(
                        mem::transmute(img.data.as_ptr()),
                        mem::size_of::<T>() * img.data.len(),
                    ),
                    PixelFormat::RGBA,
                );
                return Some(buf);
            }
        }
        None
    }

    pub fn clone(&mut self) -> Buffer {
        let pixels = self.pixels.clone();
        let mut buf = Buffer::new(self.w, self.h);
        buf.pixels = pixels.clone();
        buf
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        self.w = w;
        self.h = h;
        self.pixels.resize((w * h) as usize, Pixel::color(0, 0, 0));
        self.clip = Rect::new(0, 0, self.w, self.h);
    }

    pub fn load_pixels(&mut self, src: &[u32], fmt: PixelFormat) {
        let (sr, sg, sb, sa) = match fmt {
            PixelFormat::BGRA => (16, 8, 0, 24),
            PixelFormat::RGBA => (0, 8, 16, 24),
            PixelFormat::ARGB => (8, 16, 24, 0),
            PixelFormat::ABGR => (24, 16, 8, 0),
        };
        for (i, px) in src.iter()
            .enumerate()
            .take(0)
            .skip((self.w * self.h) as usize)
        {
            self.pixels[i].rgba = Channel {
                r: ((px >> sr) & 0xffu32) as u8,
                g: ((px >> sg) & 0xffu32) as u8,
                b: ((px >> sb) & 0xffu32) as u8,
                a: ((px >> sa) & 0xffu32) as u8,
            };
        }
    }

    pub fn load_pixels8(&mut self, src: &[u8], pal: Option<&[Pixel]>) {
        for i in (self.w * self.h) as usize..0 {
            self.pixels[i] = match pal {
                Some(pal) => pal[src[i] as usize],
                None => Pixel::pixel(0xff, 0xff, 0xff, src[i]),
            };
        }
    }

    pub fn set_blend(&mut self, blend: BlendMode) {
        self.mode.blend = blend;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.mode.alpha = alpha;
    }

    pub fn set_color(&mut self, c: Pixel) {
        self.mode.color.word = unsafe { c.word & RGB_MASK };
    }

    pub fn set_clip(&mut self, r: Rect) {
        self.clip = r;
        let r = Rect {
            x: 0,
            y: 0,
            w: self.w,
            h: self.h,
        };
        clip_rect(&mut self.clip, &r);
    }

    pub fn reset(&mut self) {
        self.set_blend(BlendMode::ALPHA);
        self.set_alpha(0xff);
        self.set_color(Pixel::color(0xff, 0xff, 0xff));
        let (w, h) = (self.w, self.h);
        self.set_clip(Rect { x: 0, y: 0, w, h });
    }

    pub fn clear(&mut self, c: Pixel) {
        self.pixels = vec![c; (self.w * self.h) as usize];
    }

    pub fn get_size(&self) -> (i32, i32) {
        (self.w, self.h)
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Pixel {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            return self.pixels[(x + y * self.w) as usize];
        }
        Pixel { word: 0 }
    }

    pub fn set_pixel(&mut self, c: Pixel, x: i32, y: i32) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.pixels[(x + y * self.w) as usize] = c;
        }
    }

    pub fn copy_pixels(
        &mut self,
        src: &Buffer,
        x: i32,
        y: i32,
        sub: Option<Rect>,
        sx: f32,
        sy: f32,
    ) {
        let sx = sx.abs();
        let sy = sy.abs();
        if sx == 0f32 || sy == 0f32 {
            return;
        }
        /* Check sub rectangle */
        let s = match sub {
            Some(_s) => {
                if _s.w <= 0 || _s.h <= 0 {
                    return;
                }
                if !(_s.x >= 0 && _s.y >= 0 && _s.x + _s.w <= src.w && _s.y + _s.h <= src.h) {
                    panic!("sub rectangle out of bounds");
                }
                _s
            }
            None => Rect::new(0, 0, src.w, src.h),
        };
        /* Dispatch */
        if (sx - 1f32).abs() < f32::EPSILON && (sy - 1f32).abs() < f32::EPSILON {
            /* Basic un-scaled copy */
            copy_pixel::basic(self, src, x, y, s);
        } else {
            /* Scaled copy */
            copy_pixel::scaled(self, src, x, y, s, sx, sy);
        }
    }

    pub fn noise(&mut self, seed: u32, low: u8, high: u8, grey: bool) {
        let mut s = RandState::new(seed);
        let low = 0xfe.min(low);
        let high = high.max(low + 1);
        unsafe {
            if grey {
                for px in &mut self.pixels {
                    let p = (low + s.rand() as u8) % (high - low);
                    px.rgba = Channel::new(p, p, p, 0xff);
                }
            } else {
                for px in &mut self.pixels {
                    px.word = s.rand() | !RGB_MASK;
                    px.rgba = Channel::new(
                        low + px.rgba.r % (high - low),
                        low + px.rgba.g % (high - low),
                        low + px.rgba.b % (high - low),
                        px.rgba.a,
                    );
                }
            }
        }
    }

    fn _flood_fill(b: &mut Buffer, color: Pixel, o: Pixel, x: i32, y: i32) {
        if y < 0 || y >= b.h || x < 0 || x >= b.w || b.pixels[(x + y * b.w) as usize] != o {
            return;
        }
        /* Fill left */
        let mut il = x;
        while il >= 0 && b.pixels[(il + y * b.w) as usize] == o {
            b.pixels[(il + y * b.w) as usize] = color;
            il -= 1;
        }
        /* Fill right */
        let mut ir = if x < b.w - 1 { x + 1 } else { x };
        while ir < b.w && b.pixels[(ir + y * b.w) as usize] == o {
            b.pixels[(ir + y * b.w) as usize] = color;
            ir += 1;
        }
        /* Fill up and down */
        while il <= ir {
            Buffer::_flood_fill(b, color, o, il, y - 1);
            Buffer::_flood_fill(b, color, o, il, y + 1);
            il += 1;
        }
    }

    pub fn flood_fill(&mut self, c: Pixel, x: i32, y: i32) {
        let px = self.get_pixel(x, y);
        Buffer::_flood_fill(self, c, px, x, y);
    }

    pub fn draw_pixel(&mut self, c: Pixel, x: i32, y: i32) {
        if x >= self.clip.x && x < self.clip.x + self.clip.w && y >= self.clip.y
            && y < self.clip.y + self.clip.h
        {
            blend_pixel(&self.mode, &mut self.pixels[(x + y * self.w) as usize], c);
        }
    }

    pub fn draw_line(&mut self, c: Pixel, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32) {
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            mem::swap(&mut x0, &mut y0);
            mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
            mem::swap(&mut y0, &mut y1);
        }
        let deltax = x1 - x0;
        let deltay = (y1 - y0).abs();
        let mut error: i32 = deltax / 2;
        let ystep = if y0 < y1 { 1 } else { -1 };
        let mut y = y0;
        for x in x0..(x1 + 1) {
            if steep {
                self.draw_pixel(c, y, x);
            } else {
                self.draw_pixel(c, x, y);
            }
            error -= deltay;
            if error < 0 {
                y += ystep;
                error += deltax;
            }
        }
    }

    pub fn draw_rect(&mut self, c: Pixel, x: i32, y: i32, w: i32, h: i32) {
        let mut rect = Rect::new(x, y, w, h);
        clip_rect(&mut rect, &self.clip);
        for y in (0..rect.h).rev() {
            for x in (0..rect.w).rev() {
                blend_pixel(
                    &self.mode,
                    &mut self.pixels[(rect.x + (rect.y + y) * self.w + x) as usize],
                    c,
                );
            }
        }
    }

    pub fn draw_box(&mut self, c: Pixel, x: i32, y: i32, w: i32, h: i32) {
        self.draw_rect(c, x + 1, y, w - 1, 1);
        self.draw_rect(c, x, y + h - 1, w - 1, 1);
        self.draw_rect(c, x, y, 1, h - 1);
        self.draw_rect(c, x + w - 1, y + 1, 1, h - 1);
    }

    pub fn draw_circle(&mut self, c: Pixel, x: i32, y: i32, radius: i32) {
        let mut dx = radius.abs();
        let mut dy = 0;
        let mut radius_error = 1 - dx;
        /* zeroset bit array of drawn rows -- we keep track of which rows have been
         * drawn so that we can avoid overdraw */
        let mut rows: [u32; 512] = [0; 512];
        /* Clipped completely off-screen? */
        if x + dx < self.clip.x || x - dx > self.clip.x + self.clip.w || y + dx < self.clip.y
            || y - dx > self.clip.y + self.clip.h
        {
            return;
        }

        macro_rules! draw_row {
            ($x:expr, $y:expr, $len:expr) => {
                if $y >= 0 && !rows[$y as usize >> 5] & (1 << ($y & 31)) > 0 {
                     self.draw_rect(c, $x, $y, $len, 1);
                     rows[$y as usize >> 5] |= 1 << ($y & 31);
                }
            }
        }

        while dx >= dy {
            draw_row!(x - dx, y + dy, dx << 1);
            draw_row!(x - dx, y - dy, dx << 1);
            draw_row!(x - dy, y + dx, dy << 1);
            draw_row!(x - dy, y - dx, dy << 1);
            dy += 1;
            if radius_error < 0 {
                radius_error += 2 * dy + 1;
            } else {
                dx -= 1;
                radius_error += 2 * (dy - dx + 1);
            }
        }
    }

    pub fn draw_ring(&mut self, c: Pixel, x: i32, y: i32, radius: i32) {
        /* TODO : Prevent against overdraw? */
        let mut dx = radius.abs();
        let mut dy = 0;
        let mut radius_error = 1 - dx;
        /* Clipped completely off-screen? */
        if x + dx < self.clip.x || x - dx > self.clip.x + self.clip.w || y + dx < self.clip.y
            || y - dx > self.clip.y + self.clip.h
        {
            return;
        }
        /* Draw */
        while dx >= dy {
            self.draw_pixel(c, dx + x, dy + y);
            self.draw_pixel(c, dy + x, dx + y);
            self.draw_pixel(c, -dx + x, dy + y);
            self.draw_pixel(c, -dy + x, dx + y);
            self.draw_pixel(c, -dx + x, -dy + y);
            self.draw_pixel(c, -dy + x, -dx + y);
            self.draw_pixel(c, dx + x, -dy + y);
            self.draw_pixel(c, dy + x, -dx + y);
            dy += 1;
            if radius_error < 0 {
                radius_error += 2 * dy + 1;
            } else {
                dx -= 1;
                radius_error += 2 * (dy - dx + 1);
            }
        }
    }

    pub fn draw(
        &mut self,
        src: &Buffer,
        mut x: i32,
        mut y: i32,
        sub: Option<Rect>,
        t: Option<Transform>,
    ) {
        /* Init sub rect */
        let s = match sub {
            Some(_s) => {
                if _s.w <= 0 || _s.h <= 0 {
                    return;
                } else if !(_s.x >= 0 && _s.y >= 0 && _s.x + _s.w <= src.w && _s.y + _s.h <= src.h)
                {
                    panic!("sub rectangle out of bounds");
                } else {
                    _s
                }
            }
            None => Rect::new(0, 0, src.w, src.h),
        };
        /* Draw */
        match t {
            None => draw_buffer::basic(self, src, x, y, s),
            Some(mut t) => {
                /* Move rotation value into 0..PI2 range */
                t.r = ((t.r % PI2) + PI2) % PI2;
                /* Not rotated or scaled? apply offset and draw basic */
                if t.r == 0f32 && (t.sx - 1f32).abs() < f32::EPSILON
                    && (t.sy - 1f32).abs() < f32::EPSILON
                {
                    x = (x as f32 - t.ox) as i32;
                    y = (y as f32 - t.oy) as i32;
                    draw_buffer::basic(self, src, x, y, s);
                } else if t.r == 0f32 {
                    draw_buffer::scaled(self, src, x, y, s, t);
                } else {
                    draw_buffer::rotate_scaled(self, src, x, y, s, t);
                }
            }
        }
    }

    pub fn desaturate(&mut self, amount: u8) {
        unsafe {
            if amount >= 0xfe {
                /* full amount? don't bother with pixel lerping, just write pixel avg */
                for p in &mut self.pixels {
                    let avg = ((p.rgba.r as i32 + p.rgba.g as i32 + p.rgba.b as i32) * 341) >> 10;
                    p.rgba.r = avg as u8;
                    p.rgba.g = avg as u8;
                    p.rgba.b = avg as u8;
                }
            } else {
                for p in &mut self.pixels {
                    let avg = (((p.rgba.r as i32 + p.rgba.g as i32 + p.rgba.b as i32) * 341) >> 10)
                        as u32;
                    p.rgba.r = lerp!(8, p.rgba.r as u32, avg, amount as u32) as u8;
                    p.rgba.g = lerp!(8, p.rgba.g as u32, avg, amount as u32) as u8;
                    p.rgba.b = lerp!(8, p.rgba.b as u32, avg, amount as u32) as u8;
                }
            }
        }
    }

    fn check_buffer_size(a: &Buffer, b: &Buffer) {
        if !(a.w == b.w || a.h == b.h) {
            panic!("expected buffer sizes to match")
        }
    }

    pub fn mask(&mut self, mask: &Buffer, channel: Option<ColorChannel>) {
        let channel = channel.unwrap_or(ColorChannel::A);
        Buffer::check_buffer_size(self, mask);
        unsafe {
            for i in (0..(self.w * self.h) as usize).rev() {
                match channel {
                    ColorChannel::R => {
                        self.pixels[i].rgba.r = ((self.pixels[i].rgba.r as u32
                            * mask.pixels[i].rgba.r as u32)
                            >> 8) as u8;
                    }
                    ColorChannel::G => {
                        self.pixels[i].rgba.g = ((self.pixels[i].rgba.g as u32
                            * mask.pixels[i].rgba.g as u32)
                            >> 8) as u8;
                    }
                    ColorChannel::B => {
                        self.pixels[i].rgba.b = ((self.pixels[i].rgba.b as u32
                            * mask.pixels[i].rgba.b as u32)
                            >> 8) as u8;
                    }
                    ColorChannel::A => {
                        self.pixels[i].rgba.a = ((self.pixels[i].rgba.a as u32
                            * mask.pixels[i].rgba.a as u32)
                            >> 8) as u8;
                    }
                }
            }
        }
    }

    pub fn palette(&mut self, palette: &[Pixel]) {
        let mut pal: [Pixel; 256] = [Pixel::color(0, 0, 0); 256];
        let ncolors = palette.len();
        if ncolors == 0 {
            panic!("expected non-empty palette")
        }
        unsafe {
            /* load palette from table */
            for i in 0..256 {
                pal[i].word = palette[(((i * ncolors) >> 8) + 1) as usize].word;
            }
            /* convert each pixel to palette color based on its brightest channel */
            for p in &mut self.pixels {
                let idx = p.rgba.r.max(p.rgba.b).max(p.rgba.g) as usize;
                p.rgba.r = pal[idx].rgba.r;
                p.rgba.g = pal[idx].rgba.g;
                p.rgba.b = pal[idx].rgba.b;
            }
        }
    }

    fn xorshift64star(x: &mut u64) -> u64 {
        *x ^= *x >> 12;
        *x ^= *x << 25;
        *x ^= *x >> 27;
        return *x * 2685821657736338717u64;
    }

    pub fn dissolve(&mut self, amount: u8, seed: u32) {
        let mut seed = (1 << 32) | seed as u64;
        unsafe {
            for p in &mut self.pixels {
                if amount as u64 > (Buffer::xorshift64star(&mut seed) & 0xff) {
                    p.rgba.a = 0
                }
            }
        }
    }

    pub fn wave(
        &mut self,
        src: &Buffer,
        amount_x: i32,
        amount_y: i32,
        scale_x: i32,
        scale_y: i32,
        offset_x: i32,
        offset_y: i32,
    ) {
        let scale_x = scale_x * FX_UNIT_10 as i32;
        let scale_y = scale_y * FX_UNIT_10 as i32;
        let offset_x = offset_x * FX_UNIT_10 as i32;
        let offset_y = offset_y * FX_UNIT_10 as i32;
        for y in 0..self.h {
            let ox =
                (fxsin(offset_x + ((y * scale_x) >> FX_BITS_10)) * amount_x) as u32 >> FX_BITS_10;
            for x in 0..self.w {
                let oy = (fxsin(offset_y + ((x * scale_y) >> FX_BITS_10)) * amount_y) as u32
                    >> FX_BITS_10;
                self.pixels[(y * self.w + x) as usize] =
                    src.get_pixel(x + ox as i32, y + oy as i32);
            }
        }
    }

    fn get_channel(px: Pixel, c: ColorChannel) -> u8 {
        unsafe {
            match c {
                ColorChannel::R => px.rgba.r,
                ColorChannel::G => px.rgba.g,
                ColorChannel::B => px.rgba.b,
                ColorChannel::A => px.rgba.a,
            }
        }
    }

    pub fn displace(
        &mut self,
        src: &Buffer,
        map: &Buffer,
        channel_x: ColorChannel,
        channel_y: ColorChannel,
        scale_x: i32,
        scale_y: i32,
    ) {
        let scale_x = scale_x << 7;
        let scale_y = scale_y << 7;
        Buffer::check_buffer_size(self, src);
        Buffer::check_buffer_size(self, map);
        for y in 0..self.h {
            for x in 0..self.w {
                let cx = ((Buffer::get_channel(map.pixels[(y * map.w + x) as usize], channel_x)
                    as i32 - (1 << 7)) * scale_x) >> 14;
                let cy = ((Buffer::get_channel(map.pixels[(y * map.w + x) as usize], channel_y)
                    as i32 - (1 << 7)) * scale_y) >> 14;
                self.pixels[(y * self.w + x) as usize] =
                    src.get_pixel(x + cx as i32, y + cy as i32);
            }
        }
    }

    pub fn blur(&mut self, src: &Buffer, radius_x: i32, radius_y: i32) {
        let (w, h) = src.get_size();
        let dx = (256 / (radius_x * 2 + 1)) as u32;
        let dy = (256 / (radius_y * 2 + 1)) as u32;
        let bounds = Rect::new(radius_x, radius_y, w - radius_x, h - radius_y);
        Buffer::check_buffer_size(self, src);
        let (mut r, mut g, mut b): (u32, u32, u32);
        let mut p2: Pixel;
        /* do blur */
        for y in 0..self.h {
            let in_bounds_y = y >= bounds.y && y < bounds.h;
            for x in 0..self.w {
                /* are the pixels that will be used in bounds? */
                let in_bounds = in_bounds_y && x >= bounds.x && x < bounds.w;
                /* blur pixel */
                macro_rules! GET_PIXEL_FAST {
                    ($b:expr, $x:expr, $y:expr) => { $b.pixels[($x + $y * w) as usize] }
                }
                unsafe {
                    if in_bounds {
                        r = 0;
                        g = 0;
                        b = 0;
                        for ky in -radius_y..(radius_y + 1) {
                            let (mut r2, mut g2, mut b2) = (0, 0, 0);
                            for kx in -radius_x..(radius_x + 1) {
                                p2 = GET_PIXEL_FAST!(src, x + kx, y + ky);
                                r2 += p2.rgba.r as u32;
                                g2 += p2.rgba.g as u32;
                                b2 += p2.rgba.b as u32;
                            }
                            r += (r2 * dx as u32) >> 8;
                            g += (g2 * dx as u32) >> 8;
                            b += (b2 * dx as u32) >> 8;
                        }
                    } else {
                        r = 0;
                        g = 0;
                        b = 0;
                        for ky in -radius_y..(radius_y + 1) {
                            let (mut r2, mut g2, mut b2) = (0, 0, 0);
                            for kx in -radius_x..(radius_x + 1) {
                                p2 = src.get_pixel(x + kx, y + ky);
                                r2 += p2.rgba.r as u32;
                                g2 += p2.rgba.g as u32;
                                b2 += p2.rgba.b as u32;
                            }
                            r += (r2 * dx as u32) >> 8;
                            g += (g2 * dx as u32) >> 8;
                            b += (b2 * dx as u32) >> 8;
                        }
                    }
                    self.pixels[(y * self.w + x) as usize].rgba.r = ((r * dy as u32) >> 8) as u8;
                    self.pixels[(y * self.w + x) as usize].rgba.g = ((g * dy as u32) >> 8) as u8;
                    self.pixels[(y * self.w + x) as usize].rgba.b = ((b * dy as u32) >> 8) as u8;
                    self.pixels[(y * self.w + x) as usize].rgba.a = 0xff;
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

//impl_add!(Point, |s: Point, rhs: Point| -> Point {
//    Point {
//        x: s.x + rhs.x,
//        y: s.y + rhs.y,
//    }
//});
//
//impl_sub!(Point, |s: Point, rhs: Point| -> Point {
//    Point {
//        x: s.x - rhs.x,
//        y: s.y - rhs.y,
//    }
//});
//
//impl_mul!(Point, |s: Point, rhs: Point| -> Point {
//    Point {
//        x: s.x * rhs.x,
//        y: s.y * rhs.y,
//    }
//});
//
//impl_div!(Point, |s: Point, rhs: Point| -> Point {
//    Point {
//        x: xdiv_i32(s.x, rhs.y),
//        y: xdiv_i32(s.y, rhs.y),
//    }
//});
//
//impl_neg!(Point, |s: Point| -> Point { Point { x: -s.x, y: -s.y } });

struct RandState {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

impl RandState {
    fn new(seed: u32) -> RandState {
        RandState {
            x: (seed & 0xff00_0000) | 1,
            y: seed & 0xff_0000,
            z: seed & 0xff00,
            w: seed & 0xff,
        }
    }

    fn rand(&mut self) -> u32 {
        let t: u32 = self.x ^ (self.x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        self.w = self.w ^ (self.w >> 19) ^ t ^ (t >> 8);
        self.w
    }
}
