extern crate rand;
extern crate sdl2;
extern crate sera;

#[macro_use]
extern crate lazy_static;

use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sera::*;
use std::time::Duration;
use std::{mem, slice};
use rand::{Rng, ThreadRng};

const MAX_FPS: u32 = 60;

const STROKE: [[i32; 2]; 8] = [
    [-1, -1],
    [-1, 0],
    [-1, 1],
    [-0, -1],
    [-0, 1],
    [1, -1],
    [1, 0],
    [1, 1],
];

lazy_static! {
    static ref IMAGE: Buffer = Buffer::file("tests/cat.png").unwrap();
    static ref TEXT: Buffer = {
        let txt = Font::default(None) .render("AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz");
        let mut buf = Buffer::new(txt.w, txt.h);
        buf.clear(Pixel::color(0, 0, 0));
        buf.draw(&txt, 0, 0, None, None);
        buf
    };
}

fn random_color(rng: &mut ThreadRng) -> Pixel {
    Pixel { word: rng.gen() }
}

fn random_t(rng: &mut ThreadRng, high: i32) -> (i32, i32) {
    (rng.gen_range(0, high), rng.gen_range(0, high))
}

fn draw_noise(buf: &mut Buffer, rng: &mut ThreadRng) {
    let mut b = Buffer::new(128, 128);
    b.noise(rng.gen(), 0, rng.gen_range(127, 255), false);
    buf.copy_pixels(&b, 0, 0, None, 4.0, 4.0);
}

fn draw_flood_fill(buf: &mut Buffer, rng: &mut ThreadRng) {
    buf.flood_fill(random_color(rng), 0, 0);
}

fn draw_pixel(buf: &mut Buffer, rng: &mut ThreadRng) {
    let (x, y) = random_t(rng, 512);
    buf.draw_pixel(random_color(rng), x, y);
    for i in 0..STROKE.len() {
        buf.draw_pixel(random_color(rng), x + STROKE[i][0], y + STROKE[i][1])
    }
}

fn draw_line(buf: &mut Buffer, rng: &mut ThreadRng) {
    let (x, y) = random_t(rng, 512);
    let (x1, y1) = random_t(rng, 512);
    buf.draw_line(random_color(rng), x, y, x1, y1);
}

fn draw_rect(buf: &mut Buffer, rng: &mut ThreadRng) {
    let (x, y) = random_t(rng, 512);
    let (w, h) = random_t(rng, 255);
    buf.draw_rect(random_color(rng), x, y, w, h);
}

fn draw_box(buf: &mut Buffer, rng: &mut ThreadRng) {
    let (x, y) = random_t(rng, 512);
    let (w, h) = random_t(rng, 255);
    buf.draw_box(random_color(rng), x, y, w, h);
}

fn draw_circle(buf: &mut Buffer, rng: &mut ThreadRng) {
    let (x, y) = random_t(rng, 512);
    let (r, _) = random_t(rng, 255);
    buf.draw_circle(random_color(rng), x, y, r);
}

fn draw_ring(buf: &mut Buffer, rng: &mut ThreadRng) {
    let (x, y) = random_t(rng, 512);
    let (r, _) = random_t(rng, 255);
    buf.draw_ring(random_color(rng), x, y, r);
}

fn draw_buffer_basic(buf: &mut Buffer, _: &mut ThreadRng) {
    buf.draw(&IMAGE, 0, 0, None, None);
    buf.draw(&TEXT, 0, 0, None, None)
}

fn draw_buffer_scaled(buf: &mut Buffer, _: &mut ThreadRng) {
    buf.draw(
        &IMAGE,
        0,
        0,
        None,
        Some(Transform::new(0.0, 0.0, 0.0, 1.5, 1.5)),
    );
}

fn draw_buffer_rotate_scaled(buf: &mut Buffer, _: &mut ThreadRng) {
    buf.clear(Pixel { word: 0xffffffff });
    static mut TICKS: f32 = 0.0;
    static mut ROT: f32 = 0.0;
    unsafe {
        TICKS += 0.2;
        ROT += 1.6;
        buf.draw(
            &IMAGE,
            256,
            256,
            None,
            Some(Transform::new(
                IMAGE.w as f32 / 2.0,
                IMAGE.h as f32 / 2.0,
                ROT.to_radians(),
                TICKS.sin().abs() + 0.4,
                TICKS.sin().abs() + 0.4,
            )),
        );
    }
}

#[inline]
pub fn as_bytes<T: Copy>(array: &[T]) -> &mut [u8] {
    unsafe {
        slice::from_raw_parts_mut(
            mem::transmute(array.as_ptr()),
            mem::size_of::<T>() * array.len(),
        )
    }
}

#[test]
fn draw_test() {
    let ctx = sdl2::init().unwrap();
    let sys = ctx.video().unwrap();
    let win: Window = sys.window("draw-test", 512u32, 512u32)
        .position_centered()
        .build()
        .unwrap();
    let mut buffer = Buffer::new(512, 512);
    let mut event_pump = ctx.event_pump().unwrap();
    let mut rng = rand::thread_rng();

    let tests: Vec<fn(&mut Buffer, &mut ThreadRng)> = vec![
        draw_flood_fill,
        draw_noise,
        draw_pixel,
        draw_line,
        draw_rect,
        draw_box,
        draw_circle,
        draw_ring,
        draw_buffer_basic,
        draw_buffer_scaled,
        draw_buffer_rotate_scaled,
    ];
    let mut current = 0;
    let count: usize = tests.len();
    'running: loop {
        {
            let surface = win.surface(&event_pump);
            match surface {
                Ok(mut surface) => {
                    surface.with_lock_mut(|pixels| {
                        let data = as_bytes(&buffer.pixels[..]);
                        pixels[..(data.len())].copy_from_slice(data);
                    });
                    surface.finish().unwrap();
                }
                Err(err) => panic!("{}", err),
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / MAX_FPS));
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    current = (current + 1) % count;
                    buffer.clear(Pixel { word: 0xffffffff });
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    current = if current > 1 {
                        (current - 1) % count
                    } else {
                        count - 1
                    };
                    buffer.clear(Pixel { word: 0xffffffff });
                }
                _ => {}
            }
        }
        tests[current](&mut buffer, &mut rng);
    }
}
