use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::Canvas;
use sdl3::video::Window;

use crate::CHIP8_HEIGHT;
use crate::CHIP8_WIDTH;
use crate::Chip8;

pub const SCALE_FACTOR: u32 = 15;
pub const WINDOW_WIDTH: u32 = (CHIP8_WIDTH as u32) * SCALE_FACTOR;
pub const WINDOW_HEIGHT: u32 = (CHIP8_HEIGHT as u32) * SCALE_FACTOR;

pub fn draw_screen(emu: &Chip8, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % CHIP8_WIDTH) as u32;
            let y = (i / CHIP8_WIDTH) as u32;

            let rect = Rect::new(
                (x * SCALE_FACTOR) as i32,
                (y * SCALE_FACTOR) as i32,
                SCALE_FACTOR,
                SCALE_FACTOR,
            );
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
