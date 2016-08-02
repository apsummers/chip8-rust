use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

/// The Chip8
pub struct Chip8 {
    // Addressable memory
    pub memory: [u8; 4096],

    // Program counter
    pc: u16,
    
    // Currently executing instruction
    instr: u16,

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
            memory: [0x0; 4096],
            pc: 0x200,
            instr: 0x0,
            v: [0x0; 16],
            index: 0x0,
            stack: [0x0; 16],
            sp: 0x0,
            dt: 0x0,
            st: 0x0,
            fb: [0x0; 64 * 32],
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
        self.instr = (self.memory[self.pc as usize] as u16) << 8 |
                    self.memory[(self.pc + 1) as usize] as u16;
        // Get opcode, which is the first byte of the instruction
        let opcode = self.instr & 0xF000;

        match opcode {
            0x0000 => {
                match self.instr {
                    0x00E0 => self.cls(),
                    0x00EE => self.ret(),
                    _ => println!("Unrecognized instruction: {:#06X}",
                                  self.instr),
                }
            },
            0x1000 => self.jp_addr(),
            0x2000 => self.call_addr(),
            0x3000 => self.se_vx_byte(),
            0x5000 => self.se_vx_vy(),
            0x6000 => self.ld_vx_byte(),
            0x7000 => self.add_vx_byte(),
            0x8000 => {
                match self.instr & 0x000F {
                    0x0000 => self.ld_vx_vy(),
                    _ => println!("{:#06X}: Unrecognized instruction",
                                  self.instr),
                };
            },
            0xA000 => self.ld_index_addr(),
            0xB000 => self.jp_v0_addr(),
            0xD000 => self.drw_vx_vy_nib(),
            0xE000 => {
                match self.instr & 0x00FF {
                    0x009E => self.skp_vx(),
                    0x00A1 => self.sknp_vx(),
                    _ => println!("{:#06X}: Unrecognized instruction",
                                  self.instr),
                }
            },
            0xF000 => {
                match self.instr & 0x00FF {
                    0x0007 => self.ld_vx_dt(),
                    0x001E => self.add_index_vx(),
                    0x0015 => self.ld_dt_vx(),
                    _ => println!("{:#06X}: Unrecognized instruction",
                                  self.instr),
                }
            },
            _ => {
                println!("{:#06X}: Unrecognized opcode", self.instr);
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
    fn cls(&mut self) {
        self.pc += 0x2;
        println!("{:#06X}: CLS", self.instr);
    }

    /// Instruction: 0x00EE
    ///
    /// Return from a subroutine.
    fn ret(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 0x2;
        println!("{:#06X}: RET", self.instr);
    }

    /// Instruction: 0x1NNN
    ///
    /// Jump to location 0xNNN.
    fn jp_addr(&mut self) {
        self.pc = self.instr & 0x0FFF;
        println!("{:#06X}: JP {:#06X}", self.instr, self.instr & 0x0FFF);
    }

    /// Instruction: 0x2NNN
    ///
    /// Call subroutine at 0xNNN.
    fn call_addr(&mut self) {
        self.sp += 0x2;
        self.stack[self.sp as usize] = self.pc;
        self.pc = self.instr & 0x0FFF;
        println!("{:#06X}: CALL {:#06X}", self.instr, self.instr & 0x0FFF);
    }

    /// Instruction: 0x3XNN
    ///
    /// Skip next instruction if `v[X]` == `NN`.
    fn se_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        let byte = ((self.instr & 0x00FF) << 8) as u8;
        if self.v[reg] == byte {
            self.pc += 0x4;
        } else {
            self.pc += 0x2;
        }
        println!("{:#06X}: SN V[{:X}], {:#06X}",
                 self.instr, (self.instr & 0x0F00 >> 8), (self.instr & 0x00FF));
    }

    /// Instruction: 0x5XY0
    ///
    /// Skip next instruction if `v[X]` == `v[Y]`.
    fn se_vx_vy(&mut self) {
        let x = (self.instr & 0x0F00) as usize;
        let y = (self.instr & 0x00F0) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 0x4;
            return;
        }
        self.pc += 0x2;
        println!("{:#06X}: SE V[{:X}], V[{:X}]", self.instr, x, y);
    }

    /// Instruction: 0x6XNN
    ///
    /// Load `NN` into register V[X].
    fn ld_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.v[reg] = (self.instr & 0x00FF) as u8;
        self.pc += 2;
        println!("{:#06X}: LD {:#06X}, V[{:X}]",
                 self.instr, (self.instr & 0x00FF), reg);
    }

    /// Instruction: 0x7XNN
    ///
    /// Add V[X] and `NN` and store the result in V[X].
    fn add_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.v[reg] += ((self.instr & 0x00FF) << 8) as u8;
        self.pc += 0x2;
        println!("{:#06X}: V[{:X}] += {:#06X}",
                 self.instr, reg, self.instr & 0x00FF);
    }

    /// Instruction: 0x8XY0
    ///
    /// Load V[Y] into V[X].
    fn ld_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        self.v[reg_x] = self.v[reg_y];
        self.pc += 0x2;
        println!("{:#06X}: LD V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
    }

    /// Instruction: 0xANNN
    ///
    /// Set index register to 0xNNN.
    fn ld_index_addr(&mut self) {
        self.index = self.instr & 0x0FFF;
        self.pc += 0x2;
        println!("{:#06X}: LD index, {:#06X}", self.instr, self.instr & 0x0FFF);
    }

    /// Instruction: 0xBNNN
    ///
    /// Jump to location 0xNNN + V[0].
    fn jp_v0_addr(&mut self) {
        self.pc = (self.instr & 0x0FFF) + (self.v[0] as u16);
        println!("{:#06X}: JP V[0], {:#06X}", self.instr, self.instr & 0x0FFF);
    }

    /// Instruction: 0xDXYN
    ///
    /// Draw sprite.
    fn drw_vx_vy_nib(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        let byte = (self.instr & 0x000F) as u8;
        self.pc += 0x2;
        println!("{:#06X}: DRW V[{:X}], V[{:X}], {:#06X}",
                 self.instr, reg_x, reg_y, byte);
    }

    /// Instruction: 0xEX9E
    ///
    /// Skip next instruction if the key with the value of V[X] is pressed.
    /// TODO: Implement
    fn skp_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.pc += 0x2;
    }

    /// Instruction: 0xEXA1
    ///
    /// Skip next instruction if the key with the value of V[X] is not pressed.
    /// TODO: Implement
    fn sknp_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.pc += 0x2;
    }

    /// Instruction: 0xFX07
    ///
    /// Set delay timer to V[X].
    fn ld_vx_dt(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.v[reg] = self.dt;
        self.pc += 0x2;
        println!("{:#06X}: LD V[{:X}], dt", self.instr, reg);
    }

    /// Instruction: 0xFN15
    ///
    /// Set delay timer to V[X].
    fn ld_dt_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.dt = self.v[reg];
        self.pc += 0x2;
        println!("{:#06X}: dt = V[{:X}]", self.instr, reg);
    }

    /// Instruction: 0xFN1E
    ///
    /// Add index and V[X] and store the result in index.
    fn add_index_vx(&mut self) {
        let byte = (self.instr & 0x0F00) >> 8;
        self.index += byte;
        self.pc += 0x2;
        println!("{:#06X}: index += {:#06X}", self.instr, byte);
    }

}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
"Chip8 {{
    pc: {:#06X}\tinstr: {:#06X}\tsp: {:#06X}\tindex: {:#06X}
}}", 
        self.pc, self.instr, self.sp, self.index)
    }
}

