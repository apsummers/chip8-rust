extern crate env_logger;
#[macro_use]
extern crate log;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::env;
use std::thread::sleep;
use std::time::Duration;

pub mod chip8;
pub mod display;

fn main() {
    // Quit if a program to run was not specified on the command line
    if env::args().len() != 2 {
        panic!("Usage: chip8-rust PROGRAM");
    }

    env_logger::init().unwrap();

    // Initialize Chip8
    let mut chip8 = chip8::Chip8::new();
    let program = env::args().nth(1).unwrap();

    chip8.load_font_set();
    chip8.load_program(program);

    // Initialize window and renderer
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("chip8-rust",
                                        64 * display::PIXEL_SIZE,
                                        32 * display::PIXEL_SIZE)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();

    renderer.set_draw_color(Color::RGB(0, 0, 0));
    renderer.clear();
    renderer.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut pause_emulation = false;

    // Main loop
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::LCtrl), .. } => {
                    pause_emulation = !pause_emulation;
                },
                _ => { }
            };
        }

        if !pause_emulation {
            chip8.execute_cycle();

            if chip8.redraw {
                display::render(&chip8.fb, &mut renderer);
                chip8.redraw = false;
            }
            debug!("{:#?}\n", chip8);
            sleep(Duration::from_millis(15));
        }
    }

}
