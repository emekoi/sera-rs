// TODO: copy_pixel::basic
// TODO: copy_pixel::scaled
// AT: Buffer::noise

use std::ops::{Add, Div, Mul, Neg, Sub};
use std::{fmt, mem, f32};

macro_rules! lerp {
  ($bits:expr, $a:expr, $b:expr, $p:expr) => {
      u32::from($a) + (((u32::from($b) - u32::from($a)) * u32::from($p)) >> $bits)
  };
}

macro_rules! swap {
    ($a:expr, $b:expr) => {
        {
            mem::swap(&mut $a, &mut $b)
        }
    };
}

macro_rules! sh8 {
    ($a:expr, $b:tt) => { ($a) $b 8 }
}

macro_rules! tu32 {
    ($a:expr) => { u32::from($a) }
}

macro_rules! draw_row {
    ($buf: expr, $rows: expr, $c:expr, $x:expr, $y:expr, $len:expr) => {
        let y__ = $y;
        if y__ >= 0 && !$rows[y__ as usize >> 5] & (1 << (y__ & 31)) > 0 {
            $buf.draw_rect($c, $x, y__, $len, 1);
            $rows[y__ as usize >> 5] |= 1 << (y__ & 31);
        }
    };
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

#[inline]
fn xdiv_i32(n: i32, x: i32) -> i32 {
    match x {
        0 => 0,
        _ => n / x,
    }
}

#[inline]
fn xdiv_f32(n: f32, x: f32) -> f32 {
    if x == 0f32 {
        0f32
    } else {
        n / x
    }
}

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

fn clip_rect_offset(r: &mut Rect, x: &mut i32, y: &mut i32, to: &mut Rect) {
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

//fn flood_fill(b: &mut Buffer, color: Pixel, o: Pixel, x: i32, y: i32) {
//    unsafe {
//        if y < 0 || y >= b.h || x < 0 || x >= b.w || b.pixels[(x + y * b.w) as usize] != o {
//            return;
//        }
//        let mut il = x;
//        while il >= 0 && b.pixels[(il + y * b.w) as usize] == o {
//            b.pixels[(il + y * b.w) as usize] = color;
//            il -= 1;
//        }
//        let mut ir = if x < b.w - 1 { x + 1 } else { x };
//        while ir < b.w && b.pixels[(ir + y * b.w) as usize] == o {
//            b.pixels[(ir + y * b.w) as usize] = color;
//            ir += 1;
//        }
//        while il <= ir {
//            flood_fill(b, color, o, il, y - 1);
//            flood_fill(b, color, o, il, y + 1);
//            il += 1;
//        }
//    }
//}

fn blend_pixel(m: &DrawMode, d: &mut Pixel, mut s: Pixel) {
    unsafe {
        let alpha = sh8!(tu32!(s.rgba.a) * tu32!(m.alpha), >>) as u8;
        if alpha <= 1 {
            return;
        }
        if m.color != RGB_MASK {
            s.rgba.r = sh8!(tu32!(s.rgba.r) * tu32!(m.color.rgba.r), >>) as u8;
            s.rgba.g = sh8!(tu32!(s.rgba.g) * tu32!(m.color.rgba.g), >>) as u8;
            s.rgba.b = sh8!(tu32!(s.rgba.b) * tu32!(m.color.rgba.b), >>) as u8;
        }

        match m.blend {
            BlendMode::ALPHA => {}
            BlendMode::COLOR => s = m.color,
            BlendMode::ADD => {
                s.rgba.r = 0xff.min(tu32!(d.rgba.r) + tu32!(s.rgba.r)) as u8;
                s.rgba.g = 0xff.min(tu32!(d.rgba.g) + tu32!(s.rgba.g)) as u8;
                s.rgba.b = 0xff.min(tu32!(d.rgba.b) + tu32!(s.rgba.b)) as u8;
            }
            BlendMode::SUBTRACT => {
                s.rgba.r = 0x00i32.min(i32::from(d.rgba.r) - i32::from(s.rgba.r)) as u8;
                s.rgba.g = 0x00i32.min(i32::from(d.rgba.g) - i32::from(s.rgba.g)) as u8;
                s.rgba.b = 0x00i32.min(i32::from(d.rgba.b) - i32::from(s.rgba.b)) as u8;
            }
            BlendMode::MULTIPLY => {
                s.rgba.r = sh8!(u32::from(s.rgba.r) * tu32!(d.rgba.r), >>) as u8;
                s.rgba.g = sh8!(u32::from(s.rgba.g) * tu32!(d.rgba.g), >>) as u8;
                s.rgba.b = sh8!(u32::from(s.rgba.b) * tu32!(d.rgba.b), >>) as u8;
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
                s.rgba.r = 0xff - sh8!(tu32!(0xff - d.rgba.r) * tu32!(0xff - s.rgba.r), >>) as u8;
                s.rgba.g = 0xff - sh8!(tu32!(0xff - d.rgba.g) * tu32!(0xff - s.rgba.g), >>) as u8;
                s.rgba.b = 0xff - sh8!(tu32!(0xff - d.rgba.b) * tu32!(0xff - s.rgba.b), >>) as u8;
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
            let zeta = (tu32!(d.rgba.a * (0xff - alpha)) >> 8) as u8;
            d.rgba.r = DIV8_TABLE[(sh8!(tu32!(d.rgba.r) * tu32!(zeta), >>)
                                      + sh8!(tu32!(s.rgba.r) * tu32!(alpha), >>))
                                      as usize][a as usize];
            d.rgba.g = DIV8_TABLE[(sh8!(tu32!(d.rgba.g) * tu32!(zeta), >>)
                                      + sh8!(tu32!(s.rgba.g) * tu32!(alpha), >>))
                                      as usize][a as usize];
            d.rgba.b = DIV8_TABLE[(sh8!(tu32!(d.rgba.b) * tu32!(zeta), >>)
                                      + sh8!(tu32!(s.rgba.b) * tu32!(alpha), >>))
                                      as usize][a as usize];
            d.rgba.a = a;
        }
    }
    return;
}

mod copy_pixel {
    use super::*;

    // TODO: test with clipping rect and sub rect
    pub fn basic(b: &mut Buffer, src: &Buffer, mut x: i32, mut y: i32, mut sub: Rect) {
        clip_rect_offset(&mut sub, &mut x, &mut y, &mut b.clip);
        if sub.w <= 0 || sub.h <= 0 {
            return;
        }
        for i in 0..sub.h {
            let b_offset = i * b.w;
            let s_offset = i * src.w;
            b.pixels[b_offset as usize..(b.w + b_offset) as usize]
                .copy_from_slice(&src.pixels[s_offset as usize..(src.w + s_offset) as usize]);
        }
    }

    // TODO: test with clipping rect and sub rect
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
        let inx = (FX_UNIT as f32 / scalex) as i32;
        let iny = (FX_UNIT as f32 / scaley) as i32;

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
        if width == 0 || height == 0 {
            return;
        }
        let mut sy = sub.y << FX_BITS;
        for dy in y..(y + height) {
            let pixel = &src.pixels[((sub.x >> FX_BITS) + src.w * (sy >> FX_BITS)) as usize..];
            let mut sx = 0;
            let mut dx = x + b.w * dy;
            let edx = dx + width;
            while dx < edx {
                b.pixels[dx as usize] = pixel[(sx >> FX_BITS) as usize];
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
        clip_rect_offset(&mut sub, &mut x, &mut y, &mut b.clip);
        if sub.w <= 0 || sub.h <= 0 {
            return;
        }
        let dst_ptr = b.pixels.as_mut_ptr();
        let src_ptr = src.pixels.as_ptr();
        for iy in 0..(sub.h as usize) {
            unsafe {
                let mut pd = dst_ptr.offset((x + (y + iy as i32) * b.w) as isize);
                let ps = src_ptr.offset((sub.x + (sub.y + iy as i32) * src.w) as isize);
                let (mut d_off, mut s_off) = (0, 0);
                let mut ix = sub.w;
                while ix > 0 {
                    blend_pixel(&b.mode, &mut *(pd.offset(d_off)), *(ps.offset(s_off)));
                    d_off += 1;
                    s_off += 1;
                    ix -= 1;
                }
            }
        }
    }

    pub fn scaled(
        b: &mut Buffer,
        src: &Buffer,
        mut x: i32,
        mut y: i32,
        mut sub: Rect,
        t: Transform,
    ) {
        let abs_sx =
            if t.sx < 0.0 { -t.sx } else { t.sx };
        let abs_sy =
            if t.sy < 0.0 { -t.sy } else { t.sy };
        let mut width = (sub.w as f32 * abs_sx + 0.5).floor() as i32;
        let mut height = (sub.h as f32 * abs_sy + 0.5).floor() as i32;
        let osx = if t.sx < 0.0 {
            (sub.w << FX_BITS) - 1
        } else {
            0
        };
        let osy = if t.sy < 0.0 {
            (sub.h << FX_BITS) - 1
        } else {
            0
        };
        let ix = ((sub.w << FX_BITS) as f32 / t.sx / sub.w as f32) as i32;
        let iy = ((sub.h << FX_BITS) as f32 / t.sy / sub.h as f32) as i32;
        x = (x as f32
            - ((if t.sx < 0.0 { width } else { 0 }) - (if t.sx < 0.0 { -1 } else { 1 })) as f32
                * t.ox * abs_sx) as i32;
        y = (y as f32
            - ((if t.sy < 0.0 { height } else { 0 }) - (if t.sy < 0.0 { -1 } else { 1 })) as f32
                * t.oy * abs_sy) as i32;
        if x + width < b.clip.x || x > b.clip.x + b.clip.w {
            return;
        }
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
        let mut sy = osy;
        while dy < height {
            let mut dx = odx;
            let mut sx = osx;
            while dx < width {
                blend_pixel(
                    &b.mode,
                    &mut b.pixels[((x + dx) + (y + dy) * b.w) as usize],
                    src.pixels
                        [((sub.x + (sx >> FX_BITS)) + (sub.y + (sy >> FX_BITS)) * src.w) as usize],
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
        'checkSourceLeft: loop {
            x = sx >> FX_BITS;
            y = sy >> FX_BITS;
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
            x = (sx + sx_incr * (right - left)) >> FX_BITS;
            y = (sy + sy_incr * (right - left)) >> FX_BITS;
            if x < sub.x || y < sub.y || x >= sub.x + sub.w || y >= sub.y + sub.h {
                right -= 1;
                if left >= right {
                    return;
                }
            } else {
                break 'checkSourceRight;
            }
        }
        dx = left;
        while dx < right {
            blend_pixel(
                &b.mode,
                &mut b.pixels[(dx + dy * b.w) as usize],
                src.pixels[((sx >> FX_BITS) + (sy >> FX_BITS) * src.w) as usize],
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
        points[0].x = x + (cosr * (-ox) - sinr * (-oy)) as i32;
        points[0].y = y + (sinr * (-ox) + cosr * (-oy)) as i32;
        points[1].x = x + (cosr * (-ox + width as f32) - sinr * (-oy)) as i32;
        points[1].y = y + (sinr * (-ox + width as f32) + cosr * (-oy)) as i32;
        points[2].x = x + (cosr * (-ox + width as f32) - sinr * (-oy + height as f32)) as i32;
        points[2].y = y + (sinr * (-ox + width as f32) + cosr * (-oy + height as f32)) as i32;
        points[3].x = x + (cosr * (-ox) - sinr * (-oy + height as f32)) as i32;
        points[3].y = y + (sinr * (-ox) + cosr * (-oy + height as f32)) as i32;
        let top = &points[(-_q & 3) as usize];
        let right = &points[((-_q + 1) & 3) as usize];
        let bottom = &points[((-_q + 2) & 3) as usize];
        let left = &points[((-_q + 3) & 3) as usize];
        if bottom.y < b.clip.y || top.y >= b.clip.y + b.clip.h {
            return;
        }
        if right.x < b.clip.x || left.x >= b.clip.x + b.clip.w {
            return;
        }
        let mut xr = top.x << FX_BITS;
        let mut xl = xr;
        let mut il = xdiv_i32((left.x - top.x) << FX_BITS, left.y - top.y);
        let mut ir = xdiv_i32((right.x - top.x) << FX_BITS, right.y - top.y);
        let sxi = (xdiv_i32(sub.w << FX_BITS, width) as f32 * (-t.r).cos()) as i32;
        let syi = (xdiv_i32(sub.h << FX_BITS, height) as f32 * (-t.r).sin()) as i32;
        let mut sxoi = (xdiv_i32(sub.w << FX_BITS, left.y - top.y) as f32 * sinq) as i32;
        let mut syoi = (xdiv_i32(sub.h << FX_BITS, left.y - top.y) as f32 * cosq) as i32;
        let (mut sx, mut sy) = match _q {
            1 => (sub.x << FX_BITS, ((sub.y + sub.h) << FX_BITS) - 1),
            2 => (
                ((sub.x + sub.w) << FX_BITS) - 1,
                ((sub.y + sub.h) << FX_BITS) - 1,
            ),
            3 => (((sub.x + sub.w) << FX_BITS) - 1, sub.y << FX_BITS),
            _ => (sub.x << FX_BITS, sub.y << FX_BITS),
        };
        let mut dy = if left.y == top.y || right.y == top.y {
            top.y - 1
        } else {
            top.y
        };
        while dy <= bottom.y {
            let (tsx, tsxi) = if inv_x {
                (((sub.x * 2 + sub.w) << FX_BITS) - sx - 1, -sxi)
            } else {
                (sx, sxi)
            };
            let (tsy, tsyi) = if inv_y {
                (((sub.y * 2 + sub.h) << FX_BITS) - sy - 1, -syi)
            } else {
                (sy, syi)
            };
            scan_line(
                b,
                src,
                &sub,
                dy,
                Transform::new(
                    (xl >> FX_BITS) as f32,
                    (xr >> FX_BITS) as f32,
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
            if dy == left.y {
                il = xdiv_i32((bottom.x - left.x) << FX_BITS, bottom.y - left.y);
                sxoi = (xdiv_i32(sub.w << FX_BITS, bottom.y - left.y) as f32 * cosq) as i32;
                syoi = (xdiv_i32(sub.h << FX_BITS, bottom.y - left.y) as f32 * -sinq) as i32;
            }
            if dy == right.y {
                ir = xdiv_i32((bottom.x - right.x) << FX_BITS, bottom.y - right.y);
            }
        }
    }
}

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

static mut INITED: bool = false;
static mut DIV8_TABLE: [[u8; 256]; 256] = [[0; 256]; 256];

const FX_BITS: u32 = 12;
const FX_UNIT: u32 = 1 << FX_BITS;
// const FX_MASK: u32 = FX_UNIT - 1;

const PI2: f32 = ::std::f32::consts::PI * 2f32;

fn init_8bit() {
    unsafe {
        if INITED {
            return;
        }
        for b in 1..256 {
            for (a, t) in DIV8_TABLE.iter_mut().enumerate().take(256).skip(1) {
                t[b] = ((a << 8) / b) as u8;
            }
        }
        INITED = true;
    }
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

        init_8bit();
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
        if (sx - 1f32).abs() < f32::EPSILON && (sy - 1f32).abs() < f32::EPSILON {
            copy_pixel::basic(self, src, x, y, s);
        } else {
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

    // overflows the stack
    // pub fn flood_fill(&mut self, c: Pixel, x: i32, y: i32) {
    //    let px = self.get_pixel(x, y);
    //    flood_fill(self, c, px, x, y);
    // }

    pub fn draw_pixel(&mut self, c: Pixel, x: i32, y: i32) {
        if x >= self.clip.x && x < self.clip.x + self.clip.w && y >= self.clip.y
            && y < self.clip.y + self.clip.h
        {
            blend_pixel(&self.mode, &mut self.pixels[(x + y * self.w) as usize], c);
        }
    }

    pub fn draw_line(&mut self, c: Pixel, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32) {
        let steep: bool = {
            let v0 = ((y1 as i32) - (y0 as i32)).abs();
            let v1 = ((x1 as i32) - (x0 as i32)).abs();
            v0 > v1
        };

        if steep {
            swap!(x0, y0);
            swap!(x1, y1);
        }

        if x0 > x1 {
            swap!(x0, x1);
            swap!(y0, y1);
        }

        let deltax = x1 - x0;
        let deltay = ((y1 as i32) - (y0 as i32)).abs() as i32;
        let mut error: i32 = (deltax as i32) / 2;
        let ystep = if y0 < y1 { 1 } else { -1i32 };
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

    pub fn draw_rect(&mut self, c: Pixel, mut x: i32, mut y: i32, w: i32, h: i32) {
        let mut r = Rect::new(x, y, w, h);
        clip_rect(&mut r, &self.clip);
        y = r.h;
        while y > 0 {
            y -= 1;
            x = r.w;
            let p = &mut self.pixels[(r.x + (r.y + y) * self.w) as usize..];
            let mut i = 0;
            while x > 0 {
                x -= 1;
                blend_pixel(&self.mode, &mut p[i], c);
                i += 1;
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
        let mut rows: [u32; 512] = [0; 512];
        if x + dx < self.clip.x || x - dx > self.clip.x + self.clip.w || y + dx < self.clip.y
            || y - dx > self.clip.y + self.clip.h
        {
            return;
        }
        while dx >= dy {
            draw_row!(self, rows, c, x - dx, y + dy, dx << 1);
            draw_row!(self, rows, c, x - dx, y - dy, dx << 1);
            draw_row!(self, rows, c, x - dy, y + dx, dy << 1);
            draw_row!(self, rows, c, x - dy, y - dx, dy << 1);
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
        let mut dx = radius.abs();
        let mut dy = 0;
        let mut radius_error = 1 - dx;
        if x + dx < self.clip.x || x - dx > self.clip.x + self.clip.w || y + dx < self.clip.y
            || y - dx > self.clip.y + self.clip.h
        {
            return;
        }
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
        let s = match sub {
            Some(s) => {
                if s.w <= 0 || s.h <= 0 {
                    return;
                } else if !(s.x >= 0 && s.y >= 0 && s.x + s.w <= src.w && s.y + s.h <= src.h) {
                    panic!("sub rectangle out of bounds");
                } else {
                    s
                }
            }
            None => Rect::new(0, 0, src.w, src.h),
        };
        match t {
            None => draw_buffer::basic(self, src, x, y, s),
            Some(mut t) => {
                t.r = ((t.r % PI2) + PI2) % PI2;
                // (sx - 1f32).abs() < f32::EPSILON && (sy - 1f32).abs() < f32::EPSILON
                if t.r == 0f32 && (t.sx - 1f32).abs() < f32::EPSILON
                    && (t.sy - 1f32).abs() < f32::EPSILON
                {
                    x -= t.ox as i32;
                    y -= t.oy as i32;
                    draw_buffer::basic(self, src, x, y, s);
                } else if t.r == 0f32 {
                    draw_buffer::scaled(self, src, x, y, s, t);
                } else {
                    draw_buffer::rotate_scaled(self, src, x, y, s, t);
                }
            }
        }
    }
}

#[derive(PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

impl_add!(Point, |s: Point, rhs: Point| -> Point {
    Point {
        x: s.x + rhs.x,
        y: s.y + rhs.y,
    }
});

impl_sub!(Point, |s: Point, rhs: Point| -> Point {
    Point {
        x: s.x - rhs.x,
        y: s.y - rhs.y,
    }
});

impl_mul!(Point, |s: Point, rhs: Point| -> Point {
    Point {
        x: s.x * rhs.x,
        y: s.y * rhs.y,
    }
});

impl_div!(Point, |s: Point, rhs: Point| -> Point {
    Point {
        x: xdiv_i32(s.x, rhs.y),
        y: xdiv_i32(s.y, rhs.y),
    }
});

impl_neg!(Point, |s: Point| -> Point { Point { x: -s.x, y: -s.y } });

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

fn main() {
    println!("HELLO WORLD");
}
