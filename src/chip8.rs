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
    dt: u8,
    st: u8,

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
            dt: 0,
            st: 0,
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
                    0x00E0 => self.cls(),
                    0x00EE => self.ret(),
                    _ => println!("Unrecognized instruction: {:x}", instr),
                }
            },
            0x1000 => self.jp_addr(instr),
            0x2000 => self.call_addr(instr),
            0x3000 => self.se_vx_byte(instr),
            0x5000 => self.se_vx_vy(instr),
            0x6000 => self.ld_vx_byte(instr),
            0x7000 => self.add_vx_byte(instr),
            0xA000 => self.ld_index_addr(instr),
            0xB000 => self.jp_v0_addr(instr),
            0xD000 => self.drw_vx_vy_nib(instr),
            0xF000 => {
                match instr & 0x00FF {
                    0x001E => self.add_index_vx(instr),
                    0x0015 => self.ld_dt_vx(instr),
                    _ => println!("Unrecognized instruction: {:x}", instr),
                }
            },
            _ => {
                println!("Unrecognized opcode in instruction: {:x}", instr);
                self.pc += 0x2;
            },
        }
    }

    /// Instruction: 0x00E0
    ///
    /// Clear the display.
    fn cls(&mut self) {
        println!("Clear display");
    }

    fn ret(&mut self) {
        println!("Return from subroutine");
        self.pc += 0x2;
    }

    /// Instruction: 0x1NNN
    ///
    /// Jump to location 0xNNN.
    fn jp_addr(&mut self, instr: u16) {
        self.pc = instr & 0x0FFF;
        println!("{:x}: Jump to {:x}", instr, instr & 0x0FFF);
    }

    /// Instruction: 0x2NNN
    ///
    /// Call subroutine at 0xNNN.
    fn call_addr(&mut self, instr: u16) {
        println!("{:x}: Call subroutine at {:x}", instr, instr & 0x0FFF);
        self.pc += 0x2;
    }

    /// Instruction: 0x3XNN
    ///
    /// Skip next instruction if `v[X]` == `NN`.
    fn se_vx_byte(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        let byte = ((instr & 0x00FF) << 8) as u8;
        if self.v[reg] == byte {
            self.pc += 0x4;
        } else {
            self.pc += 0x2;
        }
        println!("{:x}: Skip next instruction if v[{:x}] == {:x}",
                 instr, (instr & 0x0F00 >> 8), (instr & 0x00FF));
    }

    /// Instruction: 0x5XY0
    ///
    /// Skip next instruction if `v[X]` == `v[Y]`.
    fn se_vx_vy(&mut self, instr: u16) {
        let x = (instr & 0x0F00) as usize;
        let y = (instr & 0x00F0) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 0x4;
            return;
        }
        self.pc += 0x2;
        println!("{:x}: Skip next instruction if v[{:x}] == v[{:x}]",
                 instr, x, y);
    }

    /// Instruction: 0x6XNN
    ///
    /// Load `NN` into register V[X].
    fn ld_vx_byte(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        self.v[reg] = (instr & 0x00FF) as u8;
        self.pc += 2;
        println!("{:x}: Load {:x} into register V[{:x}]",
                 instr, (instr & 0x00FF), reg);
    }

    /// Instruction: 0x7XNN
    ///
    /// Add V[X] and `NN` and store the result in V[X].
    fn add_vx_byte(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        self.v[reg] += ((instr & 0x00FF) << 8) as u8;
        self.pc += 0x2;
        println!("{:x}: V[{:x}] += {:x}", instr, reg, instr & 0x00FF);
    }

    /// Instruction: 0xANNN
    ///
    /// Set index register to 0xNNN.
    fn ld_index_addr(&mut self, instr: u16) {
        println!("{:x}: Set index to {:x}", instr, instr & 0x0FFF);
        self.index = instr & 0x0FFF;
        self.pc += 0x2;
    }

    /// Instruction: 0xBNNN
    ///
    /// Jump to location 0xNNN + V[0].
    fn jp_v0_addr(&mut self, instr: u16) {
        println!("{:x}: Jump to address {:x} + V[0]", instr, instr & 0x0FFF);
        self.pc = (instr & 0x0FFF) + (self.v[0] as u16);
    }

    /// Instruction: 0xDXYN
    ///
    /// Draw sprite.
    fn drw_vx_vy_nib(&mut self, instr: u16) {
        println!("{:x}: Draw sprite", instr);
        self.pc += 0x2;
    }

    /// Instruction: 0xFN15
    ///
    /// Set delay timer to V[X].
    fn ld_dt_vx(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        self.dt = self.v[reg];
        self.pc += 0x2;
        println!("{:x}: dt = V[{:x}]", instr, reg);
    }

    /// Instruction: 0xFN1E
    ///
    /// Add index and V[X] and store the result in index.
    fn add_index_vx(&mut self, instr: u16) {
        let byte = (instr & 0x0F00) >> 8;
        self.index += byte;
        self.pc += 0x2;
        println!("{:x}: index += {:x}", instr, byte);
    }

}

