use std::{fmt, mem, f32};

//macro_rules! clamp {
//  ($x:expr, $a:expr, $b:expr) => {
//      $x.min($b).max($a)
//  };
//}

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

fn xdiv(n: i32, x: i32) -> i32 {
    match x {
        0 => 0,
        _ => n / x,
    }
}

fn clip_rect(r: &mut Rect, to: &Rect) {
    let x1 = r.x.max(to.x);
    let y1 = r.y.max(to.y);
    let x2 = (r.x + r.w).min(to.x + to.w);
    let y2 = (r.y + r.h).min(to.y + to.h);
    r.x = x1;
    r.w = x2 - x1;
    r.y = y1;
    r.h = y2 - y1;
}

fn clip_rect_offset(r: &mut Rect, x: &mut i32, y: &mut i32, to: &mut Rect) {
    let mut d = to.x - *x;
    if d > 0 {
        *x += d;
        r.w -= d;
        r.x += d
    }
    d = to.y - *y;
    if d > 0 {
        *y += d;
        r.h -= d;
        r.y += d
    }
    d = (*x + r.w) - (to.x + to.w);
    if d > 0 {
        r.x -= d;
    }
    d = (*y + r.h) - (to.y + to.h);
    if d > 0 {
        r.y -= d;
    }
}

fn copy_pixel_basic(b: &mut Buffer, src: &Buffer, mut x: i32, mut y: i32, mut s: Rect) {
    clip_rect_offset(&mut s, &mut x, &mut y, &mut b.clip);
    if s.w == 0 || s.h == 0 {
        return;
    }
    for i in 0..s.h {
        let b_offset = i * b.w;
        let s_offset = i * src.w;
        b.pixels[b_offset as usize..(b.w + b_offset) as usize]
            .copy_from_slice(&src.pixels[s_offset as usize..(src.w + s_offset) as usize]);
    }
}

fn copy_pixels_scaled(
    b: &mut Buffer,
    src: &Buffer,
    mut x: i32,
    mut y: i32,
    mut s: Rect,
    scalex: f32,
    scaley: f32,
) {
    let mut width = (s.w as f32 * scalex) as i32;
    let mut height = (s.h as f32 * scaley) as i32;
    let inx = (FX_UNIT as f32 / scalex) as i32;
    let iny = (FX_UNIT as f32 / scaley) as i32;

    let delta = b.clip.x - x;
    if delta > 0 {
        x += delta;
        s.x += (delta as f32 / scalex) as i32;
        width -= delta;
    }
    let delta = b.clip.y - y;
    if delta > 0 {
        y += delta;
        s.y += (delta as f32 / scaley) as i32;
        height -= delta;
    }
    let delta = (x + width) - (b.clip.x + b.clip.w);
    if delta > 0 {
        width -= delta;
    }
    let delta = (y + height) - (b.clip.y + b.clip.h);
    if delta > 0 {
        height -= delta;
    }

    if width == 0 || height == 0 {
        return;
    }
    let mut sy = s.y << FX_BITS;
    for dy in y..(y + height) {
        let pixels = &src.pixels[((s.x >> FX_BITS) + src.w * (sy >> FX_BITS)) as usize..];
        let mut sx = 0;
        let mut dx = x + b.w * dy;
        let edx = dx + width;
        while dx < edx {
            dx += 1;
            b.pixels[(dx - 1) as usize] = pixels[(sx >> FX_BITS) as usize];
            sx += inx;
        }
        sy += iny;
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

mod draw_buffer {
    use super::*;

    pub fn basic(b: &mut Buffer, src: &Buffer, mut x: i32, mut y: i32, mut s: Rect) {
        clip_rect_offset(&mut s, &mut x, &mut y, &mut b.clip);
        if s.w <= 0 || s.h <= 0 {
            return;
        }
        let mut dst_ptr = b.pixels.as_mut_ptr();
        let src_ptr = src.pixels.as_ptr();
        for iy in 0..(s.h as usize) {
            unsafe {
                let mut pd = dst_ptr.offset((x + (y + iy as i32) * b.w) as isize);
                let ps = src_ptr.offset((s.x + (s.y + iy as i32) * src.w) as isize);
                let (mut d_off, mut s_off) = (0, 0);
                for _ in (s.w as usize)..0 {
                    blend_pixel(&b.mode, &mut *(pd.offset(d_off)), *(ps.offset(s_off)));
                    d_off += 1;
                    s_off += 1;
                }
            }
        }
    }

    pub fn scaled(b: &mut Buffer, src: &Buffer, mut x: i32, mut y: i32, mut s: Rect, a: Transform) {
        
    }

}

#[cfg(feature = "MODE_RGBA")]
const RGB_MASK: u32 = 0xffffff;
#[cfg(feature = "MODE_ARGB")]
const RGB_MASK: u32 = 0xffffff00;
#[cfg(feature = "MODE_ABGR")]
const RGB_MASK: u32 = 0xffffff00;
#[cfg(feature = "MODE_BGRA")]
const RGB_MASK: u32 = 0xffffff;

static mut INITED: bool = false;
static mut DIV8_TABLE: [[u8; 256]; 256] = [[0; 256]; 256];

const FX_BITS: u32 = 12;
const FX_UNIT: u32 = 1 << FX_BITS;
const FX_MASK: u32 = FX_UNIT - 1;

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

#[derive(Debug, Copy, Clone)]
pub enum PixelFormat {
    BGRA,
    RGBA,
    ARGB,
    ABGR,
}

#[derive(Debug, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
#[cfg(feature = "MODE_ARGB")]
#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
#[cfg(feature = "MODE_ABGR")]
#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub a: u8,
    pub b: u8,
    pub g: u8,
    pub r: u8,
}
#[cfg(feature = "MODE_BGRA")]
#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl Channel {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Channel {
        Channel { r, g, b, a }
    }
}

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

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DrawMode {
    pub color: Pixel,
    pub blend: BlendMode,
    pub alpha: u8,
}

impl DrawMode {
    fn new(color: Pixel, blend: BlendMode, alpha: u8) -> DrawMode {
        DrawMode {
            color,
            blend,
            alpha,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub ox: f32,
    pub oy: f32,
    pub r: f32,
    pub sx: f32,
    pub sy: f32,
}

impl Transform {
    fn new(ox: f32, oy: f32, r: f32, sx: f32, sy: f32) -> Transform {
        Transform { ox, oy, r, sx, sy }
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub mode: DrawMode,
    pub clip: Rect,
    pub pixels: Vec<Pixel>,
    pub w: i32,
    pub h: i32,
}

impl Buffer {
    pub fn new(w: i32, h: i32) -> Buffer {
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

    pub fn load_pixels(&mut self, src: &Vec<u32>, fmt: PixelFormat) {
        let (sr, sg, sb, sa) = match fmt {
            PixelFormat::BGRA => (16, 8, 0, 24),
            PixelFormat::RGBA => (0, 8, 16, 24),
            PixelFormat::ARGB => (8, 16, 24, 0),
            PixelFormat::ABGR => (24, 16, 8, 0),
        };

        unsafe {
            for i in (self.w * self.h) as usize..0 {
                self.pixels[i].rgba.r = ((src[i] >> sr) & 0xff) as u8;
                self.pixels[i].rgba.g = ((src[i] >> sg) & 0xff) as u8;
                self.pixels[i].rgba.b = ((src[i] >> sb) & 0xff) as u8;
                self.pixels[i].rgba.a = ((src[i] >> sa) & 0xff) as u8;
            }
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

    pub fn set_alpha(&mut self, alpha: u8) {
        self.mode.alpha = alpha;
    }

    pub fn set_blend(&mut self, mode: BlendMode) {
        self.mode.blend = mode;
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
        if x < self.w && y < self.h {
            return self.pixels[(x + y * self.w) as usize];
        }
        Pixel { word: 0 }
    }

    pub fn set_pixel(&mut self, c: Pixel, x: i32, y: i32) {
        if x < self.w && y < self.h {
            self.pixels[(x + y * self.w) as usize] = c;
        }
    }

    pub fn copy_pixels(
        &mut self,
        src: &Buffer,
        x: i32,
        y: i32,
        sub: Option<Rect>,
        mut sx: f32,
        mut sy: f32,
    ) {
        sx = sx.abs();
        sy = sy.abs();
        if sx == 0f32 || sy == 0f32 {
            return;
        }
        let s = match sub {
            Some(s) => {
                if s.w == 0 || s.h == 0 {
                    return;
                }
                if !(s.x + s.w <= src.w && s.y + s.h <= src.h) {
                    panic!("sub rectangle out of bounds");
                }
                s
            }
            None => Rect::new(0, 0, src.w, src.h),
        };
        if (sx - 1f32).abs() < f32::EPSILON && (sy - 1f32).abs() < f32::EPSILON {
            copy_pixel_basic(self, src, x, y, s);
        } else {
            copy_pixels_scaled(self, src, x, y, s, sx, sy);
        }
    }

    pub fn noise(&mut self, seed: u32, high: u8, low: u8, grey: bool) {
        let mut s = RandState::new(seed);
        let low = 0xfe.min(low);
        let high = high.max(low + 1);
        unsafe {
            if grey {
                for i in 0..(self.w * self.h) as usize {
                    let px = (low + s.rand() as u8) % (high - low);
                    self.pixels[i].rgba = Channel::new(px, px, px, 0xff);
                }
            } else {
                for i in 0..(self.w * self.h) as usize {
                    self.pixels[i].word = s.rand() | !RGB_MASK;
                    self.pixels[i].rgba = Channel::new(
                        low + self.pixels[i].rgba.r % (high - low),
                        low + self.pixels[i].rgba.g % (high - low),
                        low + self.pixels[i].rgba.b % (high - low),
                        self.pixels[i].rgba.a,
                    );
                }
            }
        }
    }

//    overflows the stack
//    pub fn flood_fill(&mut self, c: Pixel, x: i32, y: i32) {
//        let px = self.get_pixel(x, y);
//        flood_fill(self, c, px, x, y);
//    }

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

    pub fn draw_box(&mut self, c: Pixel, mut x: i32, mut y: i32, w: i32, h: i32) {
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

    pub fn draw(&mut self, src: &Buffer, x: i32, y: i32, sub: Option<Rect>, t: Option<Transform>) {
        let s = match sub {
            Some(s) => {
                if s.w <= 0 || s.h <= 0 {
                    return;
                } else {
                    if !(s.x >= 0 && s.y >= 0 && s.x + s.w <= src.w && s.y + s.h <= src.h) {
                        panic!("sub rectangle out of bounds");
                    } else {
                        s
                    }
                }
            }
            None => Rect::new(0, 0, src.w, src.h),
        };
        match t {
            None => draw_buffer::basic(self, src, x, y, s),
            Some(t) => {

            }
        }
    }
}

struct Point {
    x: i32,
    y: i32,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise() {
        let mut b = Buffer::new(512, 512);
        b.noise(54535957, 0, 255, false);
    }

//    #[test]
//    fn flood_fill() {
//        let mut b = Buffer::new(512, 512);
//        b.flood_fill(Pixel::color(0, 0, 0), 0, 0);
//    }

    #[test]
    fn draw_pixel() {
        let mut b = Buffer::new(512, 512);
        b.draw_pixel(Pixel::color(255, 0, 255), 255, 255);
    }

    #[test]
    fn draw_line() {
        let mut b = Buffer::new(512, 512);
        b.draw_line(Pixel::color(255, 0, 255), 255, 255, 0, 0);
    }

    #[test]
    fn draw_rect() {
        let mut b = Buffer::new(512, 512);
        b.draw_rect(Pixel::color(255, 0, 255), 0, 0, 255, 255);
    }

    #[test]
    fn draw_box() {
        let mut b = Buffer::new(512, 512);
        b.draw_box(Pixel::color(255, 0, 255), 0, 0, 255, 255);
    }

    #[test]
    fn draw_circle() {
        let mut b = Buffer::new(512, 512);
        b.draw_circle(Pixel::color(255, 0, 255), 0, 0, 255);
    }

    #[test]
    fn draw_ring() {
        let mut b = Buffer::new(512, 512);
        b.draw_ring(Pixel::color(255, 0, 255), 0, 0, 255);
    }

}

fn main() {
    println!("HELLO WORLD");
}
