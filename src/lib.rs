#[allow(unused)]
macro_rules! min {
    ($a:expr, $b:expr) => {
        if $b < $a { $b } else { $a }
    };
}

#[allow(unused)]
macro_rules! max {
    ($a:expr, $b:expr) => {
        if $b > $a { $b } else { $a }
    };
}

#[allow(unused)]
macro_rules! clamp {
    ($x:expr, $a:expr, $b:expr) => {
        max!($a, min!($x, $b))
    };
}

#[allow(unused)]
macro_rules! lerp {
    ($bits:expr, $a:expr, $b:expr, $p:expr) => {
        ($a) + (((($b) - ($a)) * ($p)) >> ($bits))
    };
}

#[allow(unused)]
macro_rules! swap {
    ($a:expr, $b:expr) => {
        {
            let tmp = $a;
            $a = $b;
            $b = tmp;
        }
    };
}

#[allow(dead_code)]
pub mod sera {
    static mut PIXELFMT: PixelFormat = PixelFormat::BGRA;
    static mut INITED: bool = false;
    static mut DIV8_TABLE: [[u8; 256]; 256] = [[0; 256]; 256];

    const PI: f64 = 3.14159265358979323846264338327950288f64;
    const PI2: f64 = PI * 2f64;

    const FX_BITS: u32 = 12;
    const FX_UNIT: u32 = 1 << FX_BITS;
    const FX_MASK: u32 = FX_UNIT - 1;

    pub fn init(fmt: PixelFormat) { unsafe { PIXELFMT = fmt; } }

    fn init_8bit() {
        unsafe {
            if INITED  { return }
            for b in 1..256 {
                for a in 1..256 {
                    DIV8_TABLE[a][b] = ((a << 8) / b) as u8;
                    println!("{}", ((a << 8) / b) as u8);
                }
            }
            INITED = true;
        }
    }

    fn rgb_mask() -> u32 {
        unsafe {
            match PIXELFMT {
                //   #define SR_CHANNELS r, g, b, a
                PixelFormat::RGBA => return 0x00ffffff,
                //   #define SR_CHANNELS a, r, g, b
                PixelFormat::ARGB => return 0xffffff00,
                //   #define SR_CHANNELS a, b, g, r
                PixelFormat::ABGR => return 0xffffff00,
                //   #define SR_CHANNELS b, g, r, a
                PixelFormat::BGRA => return 0x00ffffff,
            }
        }
    }

    fn xdiv(n: u32, x: u32) -> u32 {
        match x { 0 => 0, _ => n / x }
    }

    fn clip_rect(r: &mut Rect, to: &Rect) {
        let x1 = max!(r.x, to.x);
        let y1 = max!(r.y, to.y);
        let x2 = min!(r.x + r.w, to.x + to.w);
        let y2 = min!(r.y + r.h, to.y + to.h);
        r.x = x1; r.w = x2 - x1;
        r.y = y1; r.h = y2 - y1;
    }

    fn clip_rect_offset(r: &mut Rect, x: &mut u32, y: &mut u32, to: &mut Rect) {
        let mut d = to.x - *x;
        if d > 0 { *x += d; r.w -= d; r.x += d }
        d = to.y - *y;
        if d > 0 { *y += d; r.h -= d; r.y += d }
        d = (*x + r.w) - (to.x + to.w);
        if d > 0 { r.x -= d; }
        d = (*y + r.h) - (to.y + to.h);
        if d > 0 { r.y -= d; }
    }


//    static void copyPixelsBasic(
//    sr_Buffer *b, sr_Buffer *src, int x, int y, sr_Rect s
//    ) {
//    int i;
//    /* Clip to destination buffer */
//    clipRectAndOffset(&s, &x, &y, &b->clip);
//    /* Clipped off screen? */
//    if (s.w <= 0 || s.h <= 0) return;
//    /* Copy pixels */
//    for (i = 0; i < s.h; i++) {
//    memcpy(b->pixels + x + (y + i) * b->w,
//    src->pixels + s.x + (s.y + i) * src->w,
//    s.w * sizeof(*b->pixels));
//    }
//    }
//
//
//    static void copyPixelsScaled(
//    sr_Buffer *b, sr_Buffer *src, int x, int y, sr_Rect s,
//    float scalex, float scaley
//    ) {
//    int d, dx, dy,  edx, sx, sy, inx, iny;
//    sr_Pixel *p;
//    int w = s.w * scalex;
//    int h = s.h * scaley;
//    inx = FX_UNIT / scalex;
//    iny = FX_UNIT / scaley;
//    /* Clip to destination buffer */
//    if ((d = (b->clip.x - x)) > 0) { x += d; s.x += d / scalex; w -= d; }
//    if ((d = (b->clip.y - y)) > 0) { y += d; s.y += d / scaley; h -= d; }
//    if ((d = ((x + w) - (b->clip.x + b->clip.w))) > 0) { w -= d; }
//    if ((d = ((y + h) - (b->clip.y + b->clip.h))) > 0) { h -= d; }
//    /* Clipped offscreen? */
//    if (w == 0 || h == 0) return;
//    /* Draw */
//    sy = s.y << FX_BITS;
//    for (dy = y; dy < y + h; dy++) {
//    p = src->pixels + (s.x >> FX_BITS) + src->w * (sy >> FX_BITS);
//    sx = 0;
//    dx = x + b->w * dy;
//    edx = dx + w;
//    while (dx < edx) {
//    b->pixels[dx++] = p[sx >> FX_BITS];
//    sx += inx;
//    }
//    sy += iny;
//    }
//    }

    pub enum PixelFormat {
      BGRA,
      RGBA,
      ARGB,
      ABGR,
    }

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

    #[derive(Clone, Copy)]
    pub union Pixel {
        word: u32,
        rgba: (u8, u8, u8, u8),
    }

    impl Pixel {
        pub fn pixel(r: u8, g: u8, b: u8, a: u8) -> Pixel {
            Pixel { rgba: (r, g, b, a) }
        }

        pub fn color(r: u8, g: u8, b: u8) -> Pixel {
            Pixel { rgba: (r, g, b, 0) }
        }
    }

    pub struct Rect {
        x: u32, y: u32, w: u32, h: u32,
    }

    impl Rect {
        fn new(x: u32, y: u32, w: u32, h: u32) -> Rect {
             Rect { x, y, w, h }
        }
    }

    pub struct DrawMode {
        color: Pixel,
        blend: BlendMode,
        alpha: u8,
    }

    impl DrawMode {
        fn new(color: Pixel, blend: BlendMode, alpha: u8) -> DrawMode {
            DrawMode { color, blend, alpha }
        }
    }

    pub struct Transform {
        ox: f32, oy: f32, r: f32, sx: f32, sy: f32,
    }

    impl Transform {
        fn new(ox: f32, oy: f32, r: f32, sx: f32, sy: f32) -> Transform {
            Transform { ox, oy, r, sx, sy }
        }
    }

    pub struct Buffer {
        mode: DrawMode,
        clip: Rect,
        pixels: Vec<Pixel>,
        w: u32, h: u32,
        // flags: u16
    }

    impl Buffer {
        // sr_Buffer *sr_newBuffer(int w, int h);
        pub fn new (w: u32, h: u32) -> Buffer {
            init_8bit();
            let black = Pixel::color(0, 0, 0);
            let mut buf = Buffer {
                w, h, clip: Rect::new(0, 0, w, h),
                pixels: vec![black; (w * h) as usize],
                mode: DrawMode::new(black, BlendMode::ALPHA, 0xff)
            };
            buf.reset();
            return buf;
        }

        // sr_Buffer *sr_cloneBuffer(sr_Buffer *src);
        pub fn clone(&mut self) -> Buffer {
            let pixels = self.pixels.clone();
            let mut buf = Buffer::new(self.w, self.h);
            buf.pixels = pixels;
            return buf;
        }

        // void sr_loadPixels(sr_Buffer *b, void *src, int fmt);
        pub fn load_pixels(&mut self, src: &Vec<u32>, fmt: PixelFormat) {
            let (sr, sg, sb, sa) = match fmt {
                PixelFormat::BGRA => (16,  8,  0, 24),
                PixelFormat::RGBA => ( 0,  8, 16, 24),
                PixelFormat::ARGB => ( 8, 16, 24,  0),
                PixelFormat::ABGR => (24, 16,  8,  0)
            };

            unsafe {
                for i in (self.w * self.h) as usize..0 {
                    self.pixels[i].rgba.0 = ((src[i] >> sr) & 0xff) as u8;
                    self.pixels[i].rgba.1 = ((src[i] >> sg) & 0xff) as u8;
                    self.pixels[i].rgba.2 = ((src[i] >> sb) & 0xff) as u8;
                    self.pixels[i].rgba.3 = ((src[i] >> sa) & 0xff) as u8;
                }
            }
        }

        // void sr_loadPixels8(sr_Buffer *b, unsigned char *src, sr_Pixel *pal);
        pub fn load_pixels8(&mut self, src: &Vec<u8>, pal: Option<&Vec<Pixel>>) {
            for i in (self.w * self.h) as usize..0 {
                self.pixels[i] = match pal {
                    Some(pal) => pal[src[i] as usize],
                    None      => Pixel::pixel(0xff, 0xff, 0xff, src[i]),
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
            self.mode.color.word = unsafe { c.word & rgb_mask() };
        }

        pub fn set_clip(&mut self, r: Rect) {
            self.clip = r;
            let r = Rect { x: 0, y: 0, w: self.w, h: self.h };
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

        pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
          if x < self.w && y < self.h {
            return self.pixels[(x + y * self.w) as usize];
          }
          return Pixel { word: 0 };
        }

        pub fn set_pixel(&mut self, c: Pixel, x: u32, y: u32) {
          if x < self.w && y < self.h {
            self.pixels[(x + y * self.w) as usize] = c;
          }
        }

        // void sr_copyPixels(sr_Buffer *b, sr_Buffer *src, int x, int y, sr_Rect *sub, float sx, float sy);

        // void sr_noise(sr_Buffer *b, unsigned seed, int low, int high, int grey);
        pub fn noise(&mut self, seed: u32, high: u8, low: u8, grey: bool) {
            let mut s = RandState::new(seed);
            let low = min!(low, 0xfe);
            let high = max!(high, low + 1);
            unsafe {
                if grey {
                    for i in (self.w * self.h) as usize..0 {
                        let px = low + s.rand() as u8 % (high - low);
                        self.pixels[i].rgba = (px, px, px, 0xff);
                    }
                } else {
                    let mask = rgb_mask();
                    for i in (self.w * self.h) as usize..0 {
                        self.pixels[i].word = s.rand() | !mask;
                        self.pixels[i].rgba = (
                            low + self.pixels[i].rgba.0 % (high - low),
                            low + self.pixels[i].rgba.1 % (high - low),
                            low + self.pixels[i].rgba.2 % (high - low),
                            self.pixels[i].rgba.3
                        );
                    }
                }
            }
        }

        // void sr_floodFill(sr_Buffer *b, sr_Pixel c, int x, int y);

        // void sr_drawPixel(sr_Buffer *b, sr_Pixel c, int x, int y);
        // void sr_drawLine(sr_Buffer *b, sr_Pixel c, int x0, int y0, int x1, int y1);
        // void sr_drawRect(sr_Buffer *b, sr_Pixel c, int x, int y, int w, int h);
        // void sr_drawBox(sr_Buffer *b, sr_Pixel c, int x, int y, int w, int h);
        // void sr_drawCircle(sr_Buffer *b, sr_Pixel c, int x, int y, int r);
        // void sr_drawRing(sr_Buffer *b, sr_Pixel c, int x, int y, int r);
        // void sr_drawBuffer(sr_Buffer *b, sr_Buffer *src, int x, int y, sr_Rect *sub, sr_Transform *t);
    }

    struct Point {
        x: u32, y: u32,
    }

    struct RandState {
        x: u32, y: u32, z: u32, w: u32
    }

    impl RandState {
        fn new(seed: u32) -> RandState {
          return RandState {
              x: (seed & 0xff000000) | 1,
              y: seed & 0xff0000,
              z: seed & 0xff00,
              w: seed & 0xff,
          };
        }

        fn rand(&mut self) -> u32 {
          let t: u32 = self.x ^ (self.x << 11);
          self.x = self.y;
          self.y = self.z;
          self.z = self.w;
          self.w = self.w ^ (self.w >> 19) ^ t ^ (t >> 8);
          return self.w;
        }
    }
}


#[cfg(test)]
mod tests {
    // use super::*;
}
