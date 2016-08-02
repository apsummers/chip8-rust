use std::env;

pub mod chip8;

fn main() {
    // Quit if a program to run was not specified on the command line
    if env::args().len() != 2 {
        panic!("Usage: chip8-rust PROGRAM");
    }

    let mut chip8 = chip8::Chip8::new();
    let program = env::args().nth(1).unwrap();

    chip8.load_program(program);

    for _ in 0..200 {
        chip8.execute_cycle();
    }

    println!("{:#?}", chip8);
}
