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
	const BUFFER_SHARED: u32 = 1 << 0;

    const PI: f64 = 3.14159265358979323846264338327950288f64;
    const PI2: f64 = PI * 2f64;

    const FX_BITS: u32 = 12;
    const FX_UNIT: u32 = 1 << FX_BITS;
    const FX_MASK: u32 = FX_UNIT - 1;

    // const DIV8TABLE: [[u8; 256]; 256] = [[0; 256]; 256].iter();
    // let div: Vec<Vec<u8>> = (1..256).map(|a| (0..256).map(|b| ((a as u8) << 8) / (b as u8)).collect() ).collect();

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

    pub union Pixel {
        word: u32,
        rgba: (u8, u8, u8, u8),
    }

    pub struct Rect {
        x: i32, y: i32, w: i32, h: i32,
    }

    pub struct DrawMode {
        color: Pixel,
		blend: BlendMode,
        alpha: u8,
    }

    pub struct Transform {
        ox: f32, oy: f32, r: f32, sx: f32, sy: f32,
    }

    pub struct Buffer {
        mode: DrawMode,
        clip: Rect,
        pixels: Vec<Pixel>,
        w: i32, h: i32,
        flags: char,
    }

    struct Point {
        x: i32, y: i32,
    }

    struct RandState {
        x: u32, y: u32, z: u32, w: u32
    }

    impl Pixel {
        pub fn pixel(r: u8, g: u8, b: u8, a: u8) -> Pixel {
            Pixel { rgba: (r, g, b, a) }
        }

        pub fn color(r: u8, g: u8, b: u8) -> Pixel {
            Pixel { rgba: (r, g, b, 0) }
        }
    }



	impl Buffer {
		// sr_Buffer *sr_newBuffer(int w, int h);
//         pub fn new (w: u32, h:u32) -> Buffer {
            // Buffer { w, h,
            //         pixels: Vec::new(),
            //         mode: DrawMode::(3) }
//         }
		// sr_Buffer *sr_newBufferShared(void *pixels, int w, int h);
        // sr_Buffer *sr_cloneBuffer(sr_Buffer *src);
        // void sr_destroyBuffer(sr_Buffer* b);

        // void sr_loadPixels(sr_Buffer *b, void *src, int fmt);
    	// void sr_loadPixels8(sr_Buffer *b, unsigned char *src, sr_Pixel *pal);

    	// void sr_setAlpha(sr_Buffer* b, int alpha);
		pub fn setAlpha(&mut self, alpha: u8) {
			self.mode.alpha = alpha;
		}
    	// void sr_setBlend(sr_Buffer* b, int blend);
		pub fn setBlend(&mut self, mode: BlendMode) {
			self.mode.blend = mode;
		}
    	// void sr_setColor(sr_Buffer* b, sr_Pixel c);
		pub fn setColor(&mut self, c: Pixel) {
			self.mode.color.word = unsafe { c.word & RGB_MASK };
		}
		
		fn clipRect(r: &mut Rect, to: &Rect) {
  			let x1 = max!(r.x, to.x);
  			let y1 = max!(r.y, to.y);
  			let x2 = min!(r.x + r.w, to.x + to.w);
 			let y2 = min!(r.y + r.h, to.y + to.h);
  			r.x = x1;
  			r.y = y1;
  			r.w = max!(x2 - x1, 0);
  			r.h = max!(y2 - y1, 0);
		}
		
    	// void sr_setClip(sr_Buffer *b, sr_Rect r);
		pub fn setClip(&mut self, r: mut Rect) {
			self.clip = r;
  			r = Rect { x: 0, y: 0, w: self.w, h: self.h };
  			Buffer::clipRect(&self.clip, &r);
		}
    	// void sr_reset(sr_Buffer *b);
		pub fn reset(&self) {
			
		}
        //
    	// void sr_clear(sr_Buffer *b, sr_Pixel c);
    	// sr_Pixel sr_getPixel(sr_Buffer *b, int x, int y);
    	// void sr_setPixel(sr_Buffer *b, sr_Pixel c, int x, int y);
    	// void sr_copyPixels(sr_Buffer *b, sr_Buffer *src, int x, int y, sr_Rect *sub, float sx, float sy);
    	// void sr_noise(sr_Buffer *b, unsigned seed, int low, int high, int grey);
    	// void sr_floodFill(sr_Buffer *b, sr_Pixel c, int x, int y);
        //
    	// void sr_drawPixel(sr_Buffer *b, sr_Pixel c, int x, int y);
    	// void sr_drawLine(sr_Buffer *b, sr_Pixel c, int x0, int y0, int x1, int y1);
    	// void sr_drawRect(sr_Buffer *b, sr_Pixel c, int x, int y, int w, int h);
    	// void sr_drawBox(sr_Buffer *b, sr_Pixel c, int x, int y, int w, int h);
    	// void sr_drawCircle(sr_Buffer *b, sr_Pixel c, int x, int y, int r);
    	// void sr_drawRing(sr_Buffer *b, sr_Pixel c, int x, int y, int r);
    	// void sr_drawBuffer(sr_Buffer *b, sr_Buffer *src, int x, int y, sr_Rect *sub, sr_Transform *t);
	}

}
