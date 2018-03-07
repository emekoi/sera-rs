use super::*;

#[inline]
pub fn xdiv_i32(n: i32, x: i32) -> i32 {
    match x {
        0 => n,
        _ => n / x,
    }
}

#[inline]
pub fn xdiv_f32(n: f32, x: f32) -> f32 {
    if x == 0f32 {
        n
    } else {
        n / x
    }
}

#[inline]
pub fn clip_rect(r: &mut Rect, to: &Rect) {
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
pub fn clip_rect_offset(r: &mut Rect, x: &mut i32, y: &mut i32, to: Rect) {
    let mut _d = 0;
    if {
        _d = to.x - *x;
        _d
    } > 0
    {
        *x += _d;
        r.w -= _d;
        r.x += _d
    }
    if {
        _d = to.y - *y;
        _d
    } > 0
    {
        *y += _d;
        r.h -= _d;
        r.y += _d
    }
    if {
        _d = (*x + r.w) - (to.x + to.w);
        _d
    } > 0
    {
        r.w -= _d;
    }
    if {
        _d = (*y + r.h) - (to.y + to.h);
        _d
    } > 0
    {
        r.h -= _d;
    }
}

pub fn blend_pixel(m: &DrawMode, d: &mut Pixel, mut s: Pixel) {
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
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

pub struct RandState {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

impl RandState {
    pub fn new(seed: u32) -> RandState {
        RandState {
            x: (seed & 0xff00_0000) | 1,
            y: seed & 0xff_0000,
            z: seed & 0xff00,
            w: seed & 0xff,
        }
    }

    pub fn rand(&mut self) -> u32 {
        let t: u32 = self.x ^ (self.x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        self.w = self.w ^ (self.w >> 19) ^ t ^ (t >> 8);
        self.w
    }
}
