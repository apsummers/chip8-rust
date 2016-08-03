use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

extern crate rand;

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

    /// Load the font set into memory, starting at address 0x0.
    pub fn load_font_set(&mut self) {
        let font_set = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        let mut i = 0x0;
        for byte in font_set.bytes() {
            self.memory[i] = byte.unwrap();
            i += 1;
        }
    }

    /// Load the contents of `program` into Chip8's memory. 
    pub fn load_program(&mut self, program: String) {
        // Attempt to load the program from a file
        let bin = match File::open(&program) {
            Ok(bin) => bin,
            Err(err) => panic!("Couldn't open {}: {}",
                               program, err.description()),
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
            0x4000 => self.sne_vx_byte(),
            0x5000 => self.se_vx_vy(),
            0x6000 => self.ld_vx_byte(),
            0x7000 => self.add_vx_byte(),
            0x8000 => {
                match self.instr & 0x000F {
                    0x0000 => self.ld_vx_vy(),
                    0x0001 => self.or_vx_vy(),
                    0x0002 => self.and_vx_vy(),
                    0x0003 => self.xor_vx_vy(),
                    0x0004 => self.add_vx_vy(),
                    0x0005 => self.sub_vx_vy(),
                    0x0006 => self.shr_vx(),
                    0x0007 => self.subn_vx_vy(),
                    0x000E => self.shl_vx(),
                    _ => println!("{:#06X}: Unrecognized instruction",
                                  self.instr),
                };
            },
            0x9000 => self.sne_vx_vy(),
            0xA000 => self.ld_index_addr(),
            0xB000 => self.jp_v0_addr(),
            0xC000 => self.rnd_vx_byte(),
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
                    0x000A => self.ld_vx_key(),
                    0x0015 => self.ld_dt_vx(),
                    0x0018 => self.ld_st_vx(),
                    0x001E => self.add_index_vx(),
                    0x0029 => self.ld_index_vx_sprite(),
                    0x0033 => self.ld_bcd_vx(),
                    0x0055 => self.ld_index_imm_vx(),
                    0x0065 => self.ld_vx_index_imm(),
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
    fn cls(&mut self) {
        self.fb = [0; 64 * 32];
        self.pc += 0x2;
        println!("{:#06X}: CLS", self.instr);
    }

    /// Instruction: 0x00EE
    ///
    /// Return from a subroutine.
    fn ret(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 0x1;
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
        self.sp += 0x1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = self.instr & 0x0FFF;
        println!("{:#06X}: CALL {:#06X}", self.instr, self.instr & 0x0FFF);
    }

    /// Instruction: 0x3XNN
    ///
    /// Skip next instruction if v[X] == NN.
    fn se_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        let byte = (self.instr & 0x00FF) as u8;
        if self.v[reg] == byte {
            self.pc += 0x4;
        } else {
            self.pc += 0x2;
        }
        println!("{:#06X}: SN V[{:X}], {:#06X}", self.instr, reg, byte);
    }

    /// Instruction: 0x4XNN
    ///
    /// Skip next instruction if V[X] != NN.
    fn sne_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        let byte = (self.instr & 0x00FF) as u8;
        if self.v[reg] != byte {
            self.pc += 0x4;
        } else {
            self.pc += 0x2;
        }
        println!("{:#06X}: SNE V[{:X}], {:#06X}", self.instr, reg, byte);
    }

    /// Instruction: 0x5XY0
    ///
    /// Skip next instruction if v[X] == v[Y].
    fn se_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        if self.v[reg_x] == self.v[reg_y] {
            self.pc += 0x4;
        } else {
            self.pc += 0x2;
        }
        println!("{:#06X}: SE V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
    }

    /// Instruction: 0x6XNN
    ///
    /// Load NN into register V[X].
    fn ld_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.v[reg] = (self.instr & 0x00FF) as u8;
        self.pc += 2;
        println!("{:#06X}: LD V[{:X}], {:#06X}",
                 self.instr, reg, (self.instr & 0x00FF));
    }

    /// Instruction: 0x7XNN
    ///
    /// Add V[X] and NN and store the result in V[X].
    fn add_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.v[reg] += (self.instr & 0x00FF) as u8;
        self.pc += 0x2;
        println!("{:#06X}: ADD V[{:X}], {:#06X}",
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

    /// Instruction: 0x8XY1
    ///
    /// Take bitwise OR of V[X] and V[Y] and store the result in V[X].
    fn or_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        self.v[reg_x] |= self.v[reg_y];
        self.pc += 0x2;
        println!("{:#06X}: OR V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
    }

    /// Instruction: 0x8XY2
    ///
    /// Take bitwise AND of V[X] and V[Y] and store the result in V[X].
    fn and_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        self.v[reg_x] &= self.v[reg_y];
        self.pc += 0x2;
        println!("{:#06X}: AND V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
    }

    /// Instruction: 0x8XY3
    ///
    /// Take bitwise XOR of V[X] and V[Y] and store the result in V[X].
    fn xor_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        self.v[reg_x] ^= self.v[reg_y];
        self.pc += 0x2;
        println!("{:#06X}: XOR V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
    }

    /// Instruction: 0x8XY4
    ///
    /// Add V[X] and V[Y] and store the result in V[X]. Set V[F] to 1 if there
    /// is a carry (i.e. result > 255), otherwise 0. Only the lowest 8 bits are
    /// kept.
    fn add_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        if (self.v[reg_x] as u16) + (self.v[reg_y] as u16) > 0xFF {
            self.v[reg_x] = 0xFF;
            self.v[0xF] = 0x1;
        } else {
            self.v[reg_x] += self.v[reg_y];
            self.v[0xF] = 0x0;
        }
        self.pc += 0x2;
    }

    /// Instruction: 0x8XY5
    ///
    /// Subtract V[Y] from V[X] and store the result in V[X]. If V[X] < V[Y],
    /// set V[F] to 1 and subtract V[X] from V[Y].
    fn sub_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        if self.v[reg_x] - self.v[reg_y] > 0x0 {
            self.v[reg_x] -= self.v[reg_y]; 
            self.v[0xF] = 0x0;
        } else {
            self.v[reg_x] = self.v[reg_y] - self.v[reg_x];
            self.v[0xF] = 0x1;
        }
        self.pc += 0x2;
        println!("{:#06X}: SUB V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
    }

    /// Instruction: 0x8XY6
    ///
    /// Shift V[X] right by one bit and store the result in V[X]. Store the
    /// value of the least significant bit of V[X] in V[F] before shifting. The
    /// value for register Y is unused.
    fn shr_vx(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[reg_x] & 0x01;
        self.v[reg_x] = self.v[reg_x] >> 1;
        self.pc += 0x2;
        println!("{:#06X}: SHR V[{:X}]", self.instr, reg_x);
    }

    /// Instruction: 0x8XY7
    ///
    /// Subtract V[X] from V[Y] and store the result in V[X]. If V[X] < V[Y],
    /// set V[F] to 1 and subtract V[X] from V[Y].
    fn subn_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        if self.v[reg_y] - self.v[reg_x] > 0x0 {
            self.v[reg_x] = self.v[reg_y] - self.v[reg_x]; 
            self.v[0xF] = 0x1;
        } else {
            self.v[reg_x] = self.v[reg_x] - self.v[reg_y];
            self.v[0xF] = 0x0;
        }
        self.pc += 0x2;
        println!("{:#06X}: SUBN V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
    }

    /// Instruction: 0x8XYE
    ///
    /// Shift V[X] left by one bit and store the result in V[X]. Store the
    /// value of the most significant bit of V[X] in V[F] before shifting. The
    /// value for register Y is unused.
    fn shl_vx(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[reg_x] & 0x80;
        self.v[reg_x] = self.v[reg_x] << 1;
        self.pc += 0x2;
        println!("{:#06X}: SHL V[{:X}]", self.instr, reg_x);
    }

    /// Instruction: 0x9XY0
    ///
    /// Skip next instruction if V[X] != V[Y].
    fn sne_vx_vy(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        if self.v[reg_x] != self.v[reg_y] {
            self.pc += 0x4;
        } else {
            self.pc += 0x2;
        }
        println!("{:#06X}: SNE V[{:X}], V[{:X}]", self.instr, reg_x, reg_y);
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

    /// Instruction: 0xCXNN
    ///
    /// Set V[X] to NN AND a random byte.
    fn rnd_vx_byte(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        let byte = (self.instr & 0x00FF) as u8;
        let rand_byte = rand::random::<u8>();
        self.v[reg] = rand_byte & byte;
        self.pc += 0x2;
        println!("{:#06X}: RND V[{:X}], {:#06X}", self.instr, reg, byte);
    }

    /// Instruction: 0xDXYN
    ///
    /// Draw sprite.
    fn drw_vx_vy_nib(&mut self) {
        let reg_x = ((self.instr & 0x0F00) >> 8) as usize;
        let reg_y = ((self.instr & 0x00F0) >> 4) as usize;
        let nib = (self.instr & 0x000F) as u8;
        self.pc += 0x2;
        println!("{:#06X}: DRW V[{:X}], V[{:X}], {:#06X}",
                 self.instr, reg_x, reg_y, nib);
    }

    /// Instruction: 0xEX9E
    ///
    /// Skip next instruction if the key with the value of V[X] is pressed.
    /// TODO: Implement
    fn skp_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.pc += 0x2;
        println!("{:#06X}: SKP V[{:X}]", self.instr, reg);
    }

    /// Instruction: 0xEXA1
    ///
    /// Skip next instruction if the key with the value of V[X] is not pressed.
    /// TODO: Implement
    fn sknp_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.pc += 0x2;
        println!("{:#06X}: SKNP V[{:X}]", self.instr, reg);
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

    /// Instruction: 0xFX0A
    ///
    /// Wait for a key press and store the value of the key in V[X].
    /// TODO: Implement
    fn ld_vx_key(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.pc += 0x2;
        println!("{:#06X}: LD V[{:X}], KEY", self.instr, reg);
    }

    /// Instruction: 0xFX15
    ///
    /// Set delay timer to V[X].
    fn ld_dt_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.dt = self.v[reg];
        self.pc += 0x2;
        println!("{:#06X}: dt = V[{:X}]", self.instr, reg);
    }

    /// Instruction: 0xFX18
    ///
    /// Set sound timer to V[X].
    fn ld_st_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.st = self.v[reg];
        self.pc += 0x2;
        println!("{:#06X}: st = V[{:X}]", self.instr, reg);
    }

    /// Instruction: 0xFX1E
    ///
    /// Add index and V[X] and store the result in index.
    fn add_index_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.index += self.v[reg] as u16;
        self.pc += 0x2;
        println!("{:#06X}: ADD index, {:#06X}", self.instr, reg);
    }

    /// Instruction: 0xFX29
    ///
    /// Set index to location of sprite for digit V[X].
    /// TODO: Implement
    fn ld_index_vx_sprite(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        self.pc += 0x2;
        println!("{:#06X}: LD index, V[{:X}]", self.instr, reg);
    }

    /// Instruction: 0xFX33
    ///
    /// Store the BCD (binary coded decimal) representation of V[X] in memory
    /// locations index, index + 1 and index + 2.
    /// TODO: Implement
    fn ld_bcd_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        let mut value = self.v[reg];
        self.pc += 0x2;
        println!("{:#06X}: LD BCD, V[{:X}]", self.instr, reg);
    }

    /// Instruction: 0xFX55
    ///
    /// Store V[0] to V[X] in memory starting at the address in the index
    /// register. Set index to index + X + 1.
    fn ld_index_imm_vx(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        for i in 0x0..reg + 0x1 {
            self.memory[self.index as usize] = self.v[i];
            self.index += 0x1;
        }
        self.index += 0x1;
        self.pc += 0x2;
        println!("{:#06X}: LD [index], V[{:X}]", self.instr, reg);
    }

    /// Instruction: 0xFX65
    ///
    /// Load V[0] to V[X] with values from memory starting at the address in
    /// the index register. Set index to index + X + 1.
    fn ld_vx_index_imm(&mut self) {
        let reg = ((self.instr & 0x0F00) >> 8) as usize;
        for i in 0x0..reg + 0x1 {
            self.v[i] = self.memory[self.index as usize];
            self.index += 0x1;
        }
        self.index += 0x1;
        self.pc += 0x2;
        println!("{:#06X}: LD V[{:X}], [index]", self.instr, reg);
    }

}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
"Chip8 {{
    pc: {:#06X}\tinstr: {:#06X}\tsp: {:#06X}\tindex: {:#06X}\n
    v[0]: {:#06X}, v[1]: {:#06X}, v[2]: {:#06X}, v[3]: {:#06X}
    v[4]: {:#06X}, v[5]: {:#06X}, v[6]: {:#06X}, v[7]: {:#06X}
    v[8]: {:#06X}, v[9]: {:#06X}, v[A]: {:#06X}, v[B]: {:#06X}
    v[C]: {:#06X}, v[D]: {:#06X}, v[E]: {:#06X}, v[F]: {:#06X}\n
    dt: {:#06X}\tst: {:#06X}
}}", 
               self.pc, self.instr, self.sp, self.index,
               self.v[0x0], self.v[0x1], self.v[0x2], self.v[0x3],
               self.v[0x4], self.v[0x5], self.v[0x6], self.v[0x7],
               self.v[0x8], self.v[0x9], self.v[0xA], self.v[0xB],
               self.v[0xC], self.v[0xD], self.v[0xE], self.v[0xF],
               self.dt, self.st
        )
    }
}

