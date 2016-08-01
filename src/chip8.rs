use std::error::Error;
use std::io::Read;
use std::fs::File;
use std::path::Path;

/// The Chip8
pub struct Chip8 {
    // Addressable memory
    pub memory: [u8; 4096],

    // Program counter
    pc: u16,

    // Registers
    v: [u8; 16],
    index: u16,

    // Stack
    stack: [u8; 16],
    sp: u8,

    // Timers
    delay: u8,
    sound: u8,

    // Frame buffer
    fb: [u8; 64 * 32],
}

impl Chip8 {
    /// Construct a new Chip8.
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            pc: 0x200,
            v: [0; 16],
            index: 0,
            stack: [0; 16],
            sp: 0,
            delay: 0,
            sound: 0,
            fb: [0; 64 * 32],
        }
    }

    /// Load the contents of `program` into Chip8's memory. 
    pub fn load_program(&mut self, program: String) {
        // Attempt to load the program from a file
        let bin = match File::open(&program) {
            Ok(bin) => bin,
            Err(err) => panic!("Couldn't open {}: {}", program, err.description()),
        };

        // Copy bytes into Chip8's memory
        let mut pc_init = self.pc;
        for byte in bin.bytes() {
            self.memory[pc_init as usize] = byte.unwrap();
            pc_init += 1;
        }
    }

    /// Execute a single instruction, emulating a CPU cycle.
    pub fn execute_cycle(&mut self) {
        // Fetch instruction
        let instr = (self.memory[self.pc as usize] as u16) << 8 |
                    self.memory[(self.pc + 1) as usize] as u16;
        // Get opcode, which is the first byte of the instruction
        let opcode = instr & 0xF000;

        match opcode {
            0x0000 => {
                match instr {
                    0x00E0 => println!("Clear the screen"),
                    0x00EE => println!("Return from subroutine"),
                    _ => println!("Unrecognized instruction: {:x}", instr),
                }
            },
            0x1000 => self.jp_addr(instr),
            0x2000 => self.call_addr(instr),
            0x5000 => self.se_vx_vy(instr),
            0xA000 => self.ld_index_addr(instr),
            0xB000 => self.jp_v0_addr(instr),
            0xD000 => self.drw_vx_vy_nib(instr),
            _ => {
                println!("Unrecognized opcode in instruction: {:x}", instr);
                self.pc += 0x2;
            },
        }
    }

    /// Instruction: 0x1NNN
    ///
    /// Jump to location 0xNNN.
    fn jp_addr(&mut self, instr: u16) {
        println!("Jump to {:x}", instr & 0x0FFF);
        self.pc = instr & 0x0FFF;
    }

    /// Instruction: 0x2NNN
    ///
    /// Call subroutine at 0xNNN.
    fn call_addr(&mut self, instr: u16) {
        println!("Call subroutine at {:x}", instr & 0x0FFF);
        self.pc += 0x2;
    }

    /// Instruction: 0x5XY0
    ///
    /// Skip next instruction if `v[X]` equals `v[Y]`.
    fn se_vx_vy(&mut self, instr: u16) {
        println!("Skip next instruction if v[X] == v[Y]");
        let x = (instr & 0x0F00) as usize;
        let y = (instr & 0x00F0) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 0x4;
            return;
        }
        self.pc += 0x2;
    }

    /// Instruction: 0xANNN
    ///
    /// Set index register to 0xNNN.
    fn ld_index_addr(&mut self, instr: u16) {
        println!("Set index to {:x}", instr & 0x0FFF);
        self.index = instr & 0x0FFF;
        self.pc += 0x2;
    }

    /// Instruction: 0xBNNN
    ///
    /// Jump to location 0xNNN + V[0].
    fn jp_v0_addr(&mut self, instr: u16) {
        println!("Jump to address {:x} + V[0]", instr & 0x0FFF);
        self.pc = (instr & 0x0FFF) + (self.v[0] as u16);
    }

    /// Instruction: 0xDXYN
    ///
    /// Draw sprite.
    fn drw_vx_vy_nib(&mut self, instr: u16) {
        println!("Draw sprite");
        self.pc += 0x2;
    }
    
}

