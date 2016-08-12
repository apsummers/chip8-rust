use chip8::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn scan_keyboard(chip8: &mut Chip8, event: Event) {
    match event {
        Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
            chip8.keyboard = 0x0;
        },
        Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
            chip8.keyboard = 0x1;
        },
        Event::KeyDown { keycode: Some(Keycode::Num3), .. } => {
            chip8.keyboard = 0x2;
        },
        Event::KeyDown { keycode: Some(Keycode::Num4), .. } => {
            chip8.keyboard = 0x3;
        },
        Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
            chip8.keyboard = 0x4;
        },
        Event::KeyDown { keycode: Some(Keycode::W), .. } => {
            chip8.keyboard = 0x5;
        },
        Event::KeyDown { keycode: Some(Keycode::E), .. } => {
            chip8.keyboard = 0x6;
        },
        Event::KeyDown { keycode: Some(Keycode::R), .. } => {
            chip8.keyboard = 0x7;
        },
        Event::KeyDown { keycode: Some(Keycode::A), .. } => {
            chip8.keyboard = 0x8;
        },
        Event::KeyDown { keycode: Some(Keycode::S), .. } => {
            chip8.keyboard = 0x9;
        },
        Event::KeyDown { keycode: Some(Keycode::D), .. } => {
            chip8.keyboard = 0xA;
        },
        Event::KeyDown { keycode: Some(Keycode::F), .. } => {
            chip8.keyboard = 0xB;
        },
        Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
            chip8.keyboard = 0xC;
        },
        Event::KeyDown { keycode: Some(Keycode::X), .. } => {
            chip8.keyboard = 0xD;
        },
        Event::KeyDown { keycode: Some(Keycode::C), .. } => {
            chip8.keyboard = 0xE;
        },
        Event::KeyDown { keycode: Some(Keycode::V), .. } => {
            chip8.keyboard = 0xF;
        },
        _ => { }
    };
}
