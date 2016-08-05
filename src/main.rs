#[macro_use]
extern crate log;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::env;
use std::thread::sleep;
use std::time::Duration;

pub mod chip8;

fn main() {
    // Quit if a program to run was not specified on the command line
    if env::args().len() != 2 {
        panic!("Usage: chip8-rust PROGRAM");
    }

    let mut chip8 = chip8::Chip8::new();
    let program = env::args().nth(1).unwrap();

    chip8.load_font_set();
    chip8.load_program(program);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("chip8-rust", 64 * 8, 32 * 8)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();

    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.clear();
    renderer.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => { }
            };
        }

        chip8.execute_cycle();

        if chip8.needs_redraw {
            renderer.set_draw_color(Color::RGB(0, 0, 0));
            renderer.clear();
            for (i, val) in chip8.fb.iter().enumerate() {
                let x = ((i as i32) % 64) * 8;
                let y = ((i as i32) / 64) * 8;

                if *val == 0 {
                    renderer.set_draw_color(Color::RGB(0, 0, 0));
                } else {
                    renderer.set_draw_color(Color::RGB(255, 255, 255));
                }

                let pixel = Rect::new(x, y, 8, 8);

                match renderer.fill_rect(pixel) {
                    Ok(_) => { },
                    Err(err) => debug!("Couldn't fill pixel: {}", err),
                }
            }
            renderer.present();
            chip8.needs_redraw = false;
        }
        debug!("{:#?}\n", chip8);
        sleep(Duration::from_millis(15));
    }

}
