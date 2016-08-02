use std::error::Error;
use std::fs::File;
use std::io::Read;

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
    stack: [u16; 16],
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
                    0x00E0 => self.cls(instr),
                    0x00EE => self.ret(instr),
                    _ => println!("Unrecognized instruction: {:#X}", instr),
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
                    0x0007 => self.ld_vx_dt(instr),
                    0x001E => self.add_index_vx(instr),
                    0x0015 => self.ld_dt_vx(instr),
                    _ => println!("{:#X}: Unrecognized instruction", instr),
                }
            },
            _ => {
                println!("{:#X}: Unrecognized opcode", instr);
                self.pc += 0x2;
            },
        }

        // Update timers
        if self.dt > 0 {
            self.dt -= 1;
        }
    }

    /// Instruction: 0x00E0
    ///
    /// Clear the display.
    /// TODO: Implement
    fn cls(&mut self, instr: u16) {
        self.pc += 0x2;
        println!("{:#X}: CLS", instr);
    }

    /// Instruction: 0x00EE
    ///
    /// Return from a subroutine.
    /// TODO: Implement
    fn ret(&mut self, instr: u16) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
        println!("{:#X}: RET", instr);
    }

    /// Instruction: 0x1NNN
    ///
    /// Jump to location 0xNNN.
    fn jp_addr(&mut self, instr: u16) {
        self.pc = instr & 0x0FFF;
        println!("{:#X}: JP {:#X}", instr, instr & 0x0FFF);
    }

    /// Instruction: 0x2NNN
    ///
    /// Call subroutine at 0xNNN.
    fn call_addr(&mut self, instr: u16) {
        self.sp += 0x2;
        self.stack[self.sp as usize] = self.pc;
        self.pc = instr & 0x0FFF;
        println!("{:#X}: CALL {:#X}", instr, instr & 0x0FFF);
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
        println!("{:#X}: SN V[{:X}], {:#X}",
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
        println!("{:#X}: SE V[{:X}], V[{:X}]", instr, x, y);
    }

    /// Instruction: 0x6XNN
    ///
    /// Load `NN` into register V[X].
    fn ld_vx_byte(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        self.v[reg] = (instr & 0x00FF) as u8;
        self.pc += 2;
        println!("{:#X}: LD {:#X}, V[{:X}]", instr, (instr & 0x00FF), reg);
    }

    /// Instruction: 0x7XNN
    ///
    /// Add V[X] and `NN` and store the result in V[X].
    fn add_vx_byte(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        self.v[reg] += ((instr & 0x00FF) << 8) as u8;
        self.pc += 0x2;
        println!("{:#X}: V[{:X}] += {:#X}", instr, reg, instr & 0x00FF);
    }

    /// Instruction: 0xANNN
    ///
    /// Set index register to 0xNNN.
    fn ld_index_addr(&mut self, instr: u16) {
        self.index = instr & 0x0FFF;
        self.pc += 0x2;
        println!("{:#X}: LD index, {:#X}", instr, instr & 0x0FFF);
    }

    /// Instruction: 0xBNNN
    ///
    /// Jump to location 0xNNN + V[0].
    fn jp_v0_addr(&mut self, instr: u16) {
        self.pc = (instr & 0x0FFF) + (self.v[0] as u16);
        println!("{:#X}: JP V[0], {:#X}", instr, instr & 0x0FFF);
    }

    /// Instruction: 0xDXYN
    ///
    /// Draw sprite.
    fn drw_vx_vy_nib(&mut self, instr: u16) {
        let reg_x = ((instr & 0x0F00) >> 8) as usize;
        let reg_y = ((instr & 0x00F0) >> 4) as usize;
        let byte = (instr & 0x000F) as u8;
        self.pc += 0x2;
        println!("{:#X}: DRW V[{:X}], V[{:X}], {:#X}",
                 instr, reg_x, reg_y, byte);
    }

    /// Instruction: 0xFX07
    ///
    /// Set delay timer to V[X].
    fn ld_vx_dt(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        self.v[reg] = self.dt;
        self.pc += 0x2;
        println!("{:#X}: LD V[{:X}], dt", instr, reg);
    }

    /// Instruction: 0xFN15
    ///
    /// Set delay timer to V[X].
    fn ld_dt_vx(&mut self, instr: u16) {
        let reg = ((instr & 0x0F00) >> 8) as usize;
        self.dt = self.v[reg];
        self.pc += 0x2;
        println!("{:#X}: dt = V[{:X}]", instr, reg);
    }

    /// Instruction: 0xFN1E
    ///
    /// Add index and V[X] and store the result in index.
    fn add_index_vx(&mut self, instr: u16) {
        let byte = (instr & 0x0F00) >> 8;
        self.index += byte;
        self.pc += 0x2;
        println!("{:#X}: index += {:#X}", instr, byte);
    }

}

