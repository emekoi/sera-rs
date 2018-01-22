extern crate rand;
extern crate sdl2;
extern crate sera;

use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sera::*;
use std::time::Duration;
use std::{mem, slice};
use rand::Rng;

fn draw_noise(buf: &mut Buffer) {
    let mut rng = rand::thread_rng();
    let mut b = Buffer::new(128, 128);
    b.noise(rng.gen::<u32>(), 0, 255, false);
    buf.copy_pixels(&b, 0, 0, None, 4.0, 4.0);
}

// fn draw_flood_fill(buf: &mut Buffer) {
//    buf.flood_fill(Pixel::color(0, 0, 0), 0, 0);
// }

fn draw_pixel(buf: &mut Buffer) {
    buf.draw_pixel(Pixel::color(255, 255, 0), 255, 255);
}

fn draw_line(buf: &mut Buffer) {
    buf.draw_line(Pixel::color(255, 0, 255), 512, 512, 0, 0);
}

fn draw_rect(buf: &mut Buffer) {
    buf.draw_rect(Pixel::color(0, 255, 255), 0, 0, 255, 255);
}

fn draw_box(buf: &mut Buffer) {
    buf.draw_box(Pixel::color(255, 255, 255), 0, 0, 255, 255);
}

fn draw_circle(buf: &mut Buffer) {
    let d = (512 / 2) - (255 / 2);
    buf.draw_circle(Pixel::color(255, 255, 0), d, d, 255);
}

fn draw_ring(buf: &mut Buffer) {
    let d = (512 / 2) - (255 / 2);
    buf.draw_ring(Pixel::color(255, 0, 255), d, d, 255);
}

fn draw_buffer_basic(buf: &mut Buffer) {
    let mut b = Buffer::new(256, 256);
    let mut rng = rand::thread_rng();
    let d = (512 / 2) - (255 / 2);
    b.noise(rng.gen::<u32>(), 0, 255, true);
    b.draw_line(Pixel::color(255, 0, 255), 512, 512, 0, 0);
    b.draw_rect(Pixel::color(0, 255, 255), 0, 0, 64, 64);
    b.draw_box(Pixel::color(255, 255, 255), 0, 0, 255, 255);
    b.draw_circle(Pixel::color(255, 255, 0), d, d, 16);
    b.draw_ring(Pixel::color(255, 0, 255), d, d, 96);
    b.draw_pixel(Pixel::color(255, 255, 255), 255, 255);
    buf.draw(&b, 0, 0, None, None);
}

fn draw_buffer_scaled(buf: &mut Buffer) {
    let mut b = Buffer::new(128, 128);
    let mut rng = rand::thread_rng();
    let d = (512 / 2) - (255 / 2);
    b.noise(rng.gen::<u32>(), 0, 255, true);
    b.draw_line(Pixel::color(255, 0, 255), 128, 128, 0, 0);
    b.draw_rect(Pixel::color(0, 255, 255), 0, 0, 64, 64);
    b.draw_box(Pixel::color(255, 255, 255), 0, 0, 128, 128);
    b.draw_circle(Pixel::color(255, 255, 0), d, d, 16);
    b.draw_ring(Pixel::color(255, 0, 255), d, d, 96);
    b.draw_pixel(Pixel::color(255, 255, 255), 128, 128);
    buf.draw(
        &b,
        0,
        0,
        None,
        Some(Transform::new(0.0, 0.0, 0.0, 2.0, 6.0)),
    );
}

fn draw_buffer_rotate_scaled(buf: &mut Buffer) {
    let mut b = Buffer::new(128, 128);
    let mut rng = rand::thread_rng();
    let d = (512 / 2) - (255 / 2);
    b.noise(rng.gen::<u32>(), 0, 255, true);
    b.draw_line(Pixel::color(255, 0, 255), 128, 128, 0, 0);
    b.draw_rect(Pixel::color(0, 255, 255), 0, 0, 64, 64);
    b.draw_box(Pixel::color(255, 255, 255), 0, 0, 128, 128);
    b.draw_circle(Pixel::color(255, 255, 0), d, d, 16);
    b.draw_ring(Pixel::color(255, 0, 255), d, d, 96);
    b.draw_pixel(Pixel::color(255, 255, 255), 128, 128);
    static mut TICKS: f32 = 0.0;
    unsafe {
        TICKS = (TICKS + 0.1) % 3.0;
        buf.draw(
            &b,
            0,
            0,
            None,
            Some(Transform::new(
                -63.0,
                63.0,
                45.0f32.to_radians(),
                TICKS - 0.4,
                TICKS,
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
    let mut buffer = Buffer::new(512i32, 512i32);
    buffer.set_alpha(255);
    let mut event_pump = ctx.event_pump().unwrap();
    let max_fps = 60;
    let tests: Vec<fn(&mut Buffer)> = vec![
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
            buffer.clear(Pixel::color(0, 0, 0));
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / max_fps));
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
                } => current = (current + 1) % count,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    current = if current > 1 {
                        (current - 1) % count
                    } else {
                        count - 1
                    }
                }
                _ => {}
            }
        }
        tests[current](&mut buffer);
    }
}
