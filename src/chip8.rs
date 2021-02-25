// See: https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description

// font sprites
const FONT: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const DISPLAY_OFFSET: u16 = 0x0F00;

pub struct Chip8 {
    // main memory
    //
    // 0x000 -> 0x200 System reserved
    // 0x200 -> 0xF00 Programs
    // 0xF00 -> 0XFFF Display
    //
    memory: [u8; 4096],

    // display update flag
    display_updated: bool,

    // registers
    pc: u16,     // PC: Program counter (current instruction)
    i: u16,      // I: Index register
    v: [u8; 16], // V[N] where N: 0x0..=0xf (General purpose registers)

    // timers
    delay_timer: u8, // decremented at 60Hz
    sound_timer: u8, // play a sound while this is non-zero

    // stack pointer
    sp: usize,        // current location on the stack
    stack: [u16; 16], // stores return addresses for subroutines

    // inputs
    keypad: [bool; 16], // whether each of the keys (0x0..=0xf) are pressed
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut result = Chip8 {
            memory: [0; 4096],
            display_updated: false,
            pc: 0x200,
            i: 0,
            v: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            sp: 0,
            stack: [0; 16],
            keypad: [false; 16],
        };

        // load font
        result.memory[0..80].copy_from_slice(&FONT);

        result
    }

    pub fn display_updated(&self) -> bool {
        self.display_updated
    }

    pub fn cycle(&mut self) {
        self.display_updated = false;

        let instruction = self.fetch();
        self.execute(instruction);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            println!("\x07");
        }
    }
    pub fn write_cmd(&mut self, addr: u16, val: u16) {
        let addr = addr as usize;
        if addr >= self.memory.len() {
            panic!("Invalid memory write request");
        }
        self.memory[addr] = ((val & 0xFF00) >> 8) as u8;
        self.memory[addr + 1] = (val & 0x00FF) as u8;
    }

    pub fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        if addr >= self.memory.len() {
            panic!("Invalid memory read request");
        }
        self.memory[addr]
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        if addr >= self.memory.len() {
            panic!("Invalid memory write request");
        }
        self.memory[addr] = val;
    }

    pub fn write_keypad(&mut self, addr: u8, val: bool) {
        self.keypad[(addr%16) as usize] = val;
    }
    fn read_keypad(&self, addr: u8) -> bool {
        self.keypad[(addr % 16) as usize]
    }

    fn fetch(&mut self) -> u16 {
        let result = ((self.read(self.pc) as u16) << 8) | self.read(self.pc + 1) as u16;
        self.pc += 2;
        result
    }
    fn execute(&mut self, instruction: u16) {
        let first_nibble = (instruction & 0xF000) >> 12;
        let second_nibble = (instruction & 0x0F00) >> 8;
        let third_nibble = (instruction & 0x00F0) >> 4;
        let fourth_nibble = instruction & 0x000F;
        let second_byte: u8 = (instruction & 0x00FF) as u8;

        let x = second_nibble as usize;
        let y = third_nibble as usize;

        match first_nibble {
            0x0 => {
                match instruction {
                    0x00E0 => {
                        // 0x00E0
                        // Clear display
                        self.memory[0xF00..=0xFFF].fill(0);
                        self.display_updated = true;
                    }
                    0x00EE => {
                        // 0x00EE
                        // Return from subroutine
                        if self.sp == 0 {
                            panic!("Reached end of call stack");
                        }
                        self.sp -= 1;
                        self.pc = self.stack[self.sp];
                    }
                    _ => {
                        // 0xNNN
                        // Call machine code routine at address NNN
                        unimplemented!();
                    }
                };
            }
            0x1 => {
                // 0x1NNN
                // Jump to address NNN (set pc to NNN)
                self.pc = instruction & 0x0FFF;
            }
            0x2 => {
                // 0x2NNN
                // Call subroutine at NNN
                if self.sp >= self.stack.len() {
                    panic!("Stack overflow");
                }
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = instruction & 0x0FFF;
            }
            0x3 => {
                // 0x3XNN
                // Skips the next instruction if VX == NN
                if self.v[x] == second_byte {
                    self.pc += 2;
                }
            }
            0x4 => {
                // 0x4XNN
                // Skips the next instruction if VX != NN
                if self.v[x] != second_byte {
                    self.pc += 2;
                }
            }
            0x5 => {
                // 0x5XY0
                // Skips the next instruction if VX == VY
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            0x6 => {
                // 0x6XNN
                // Sets register X to NN
                self.v[x] = second_byte;
            }
            0x7 => {
                // 0x7XNN
                // Adds NN to register X
                let result = self.v[x] as u16 + second_byte as u16;
                self.v[x] = (result & 0xFF) as u8;
            }
            0x8 => {
                match fourth_nibble {
                    0x0 => {
                        // 0x8XY0
                        // Sets VX to VY
                        self.v[x] = self.v[y];
                    }
                    0x1 => {
                        // 0x8XY1
                        // Sets VX to VX bitwise or VY
                        self.v[x] |= self.v[y];
                    }
                    0x2 => {
                        // 0x8XY2
                        // Sets VX to VX bitwise and VY
                        self.v[x] &= self.v[y];
                    }
                    0x3 => {
                        // 0x8XY3
                        // Sets VX to VX bitwise xor VY
                        self.v[x] ^= self.v[y];
                    }
                    0x4 => {
                        // 0x8XY4
                        // Sets VX to VX + VY
                        // If there is a carry, VF is set to 1, otherwise VF is set to 0
                        let result: u16 = self.v[x] as u16 + self.v[y] as u16;
                        self.v[0xF] = if result > 0xFF { 1 } else { 0 };
                        self.v[x] = (result & 0xFF) as u8;
                    }
                    0x5 => {
                        // 0x8XY5
                        // Sets VX to VX - VY
                        // VF set to 0 when there is a borrow, 1 if not
                        let result: i16 = self.v[x] as i16 - self.v[y] as i16;
                        self.v[0xF] = if result >= 0 { 1 } else { 0 };
                        self.v[x] = ((result + 256) & 0xFF) as u8;
                    }
                    0x6 => {
                        // 0x8XY6
                        // Stores LSB of VX in VF, then right shifts VX by one
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    }
                    0x7 => {
                        // 0x8XY7
                        // Sets VX to VY - VX
                        // VF set to 0 when there is a borrow, 1 if not
                        let result: i16 = self.v[y] as i16 - self.v[x] as i16;
                        self.v[0xF] = if result >= 0 { 1 } else { 0 };
                        self.v[x] = ((result + 256) & 0xFF) as u8;
                    }
                    0xE => {
                        // 0x8XYE
                        // Stores MSB of VX in VF, then right shifts VX by one
                        self.v[0xF] = self.v[x] & 0b10000000;
                        self.v[x] <<= 1;
                    }
                    _ => {
                        unimplemented!();
                    }
                }
            }
            0x9 => {
                // 0x9XY0
                // Skips the next instruction if VX != XY
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA => {
                // 0xANNN
                // Sets I to address NNN
                self.i = instruction & 0x0FFF;
            }
            0xB => {
                // 0xBNNN
                // Jumps to address NNN + V0
                self.pc = self.v[0x0] as u16 + (instruction & 0x0FFF);
            }
            0xC => {
                // 0xCXNN
                // Sets VX to the bitwise and of a 1 byte random number and NN
                let random_byte: u8 = rand::random();
                self.v[x] = random_byte & second_byte;
            }
            0xD => {
                // 0xDXYN
                // Draws a sprite at coordinate (VX, VY)
                //   - width: 8 px
                //   - height: (N+1) px
                // Read from address stored in I
                // VF set to 1 if any screen pixels are flipped from set to unset,
                // otherwise to 0
                self.display_updated = true;

                let x = (self.v[x] % 64) as u16;
                let y = (self.v[y] % 32) as u16;
                let n = fourth_nibble;

                self.v[0xF] = 0;
                for row in 0..=n {
                    let output_y = row + y;
                    if output_y >= 32 {
                        break;
                    }

                    let mut sprite_row = self.read(self.i + row);

                    if sprite_row == 0 {
                        continue;
                    }

                    let byte_position = x / 8;
                    let byte_shift = x % 8;

                    let output_address = DISPLAY_OFFSET + output_y * 8 + byte_position;

                    // check that the next byte is in bounds
                    if byte_shift != 0 && (byte_position + 1) < 8 {
                        let output_address = output_address + 1;

                        let overflow_byte = sprite_row << (8 - byte_shift);
                        let current_byte = self.read(output_address);
                        let result_byte = overflow_byte ^ current_byte;

                        // a bit has flipped
                        if self.v[0xF] == 0 && current_byte & result_byte != current_byte {
                            self.v[0xF] = 1;
                        }

                        self.write(output_address, result_byte);
                    }

                    sprite_row >>= byte_shift;

                    let current_byte = self.read(output_address);
                    let result_byte = sprite_row ^ current_byte;

                    // a bit has flipped
                    if self.v[0xF] == 0 && current_byte & result_byte != current_byte {
                        self.v[0xF] = 1;
                    }

                    self.write(output_address, result_byte);
                }
            }
            0xE => {
                match second_byte {
                    0x9E => {
                        // 0xEX9E
                        // Skips the next instruction if the keyboard key stored in VX is pressed.
                        if self.read_keypad(self.v[x]) {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // 0xEXA1
                        // Skips the next instruction if the keyboard key stored in VX is not pressed.
                        if !self.read_keypad(self.v[x]) {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        unimplemented!();
                    }
                }
            }
            0xF => {
                match second_byte {
                    0x07 => {
                        // 0xFX07
                        // Sets VX to the value of the delay timer.
                        self.v[x] = self.delay_timer;
                    }
                    0x0A => {
                        // 0xFX0A
                        // A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
                        let mut key_pos = 0;
                        let key_pressed = self.keypad.iter().enumerate().any(|(pos, &b)| {
                            if b {
                                key_pos = pos;
                            }
                            b
                        });
                        if key_pressed {
                            self.v[x] = key_pos as u8;
                        } else {
                            // loop
                            self.pc -= 2;
                        }
                    }
                    0x15 => {
                        // 0xFX15
                        // Sets the delay timer to VX.
                        self.delay_timer = self.v[x];
                    }
                    0x18 => {
                        // 0xFX18
                        // Sets the sound timer to VX.
                        self.sound_timer = self.v[x];
                    }
                    0x1E => {
                        // 0xFX1E
                        // Adds VX to I. VF is not affected
                        self.i += self.v[x] as u16;
                    }
                    0x29 => {
                        // 0xFX29
                        // Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
                        if self.v[x] >= 16 {
                            panic!("Invalid font character access");
                        }
                        self.i = 5 * (self.v[x] as u16);
                    }
                    0x33 => {
                        // 0xFX33
                        // Stores the binary-coded decimal representation of VX, with the most significant of three
                        // digits at the address in I, the middle digit at I plus 1,
                        // and the least significant digit at I plus 2.
                        self.write(self.i, self.v[x] / 100);
                        self.write(self.i + 1, (self.v[x] / 10) % 10);
                        self.write(self.i + 2, self.v[x] % 10);
                    }
                    0x55 => {
                        // 0xFX55
                        // Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
                        for addr_offset in 0..=x {
                            let addr = self.i + (addr_offset as u16);
                            self.write(addr, self.v[addr_offset as usize]);
                        }
                    }
                    0x65 => {
                        // 0xFX65
                        // Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
                        for addr_offset in 0..=x {
                            let addr = self.i + (addr_offset as u16);
                            self.v[addr_offset as usize] = self.read(addr);
                        }
                    }
                    _ => {
                        unimplemented!();
                    }
                }
            }
            _ => {
                unimplemented!();
            }
        };
    }

    pub fn display_to_string(&self) -> String {
        let mut string = String::new();
        for row in 0..32 {
            let offset = row * 8; // 8 bytes per row
            for &byte in &self.memory[0x0F00 + offset..0x0F08 + offset] {
                let mut mask = 0b1000_0000;
                while mask != 0 {
                    if byte & mask == 0 {
                        string.push('⬛');
                    } else {
                        string.push('⬜');
                    }
                    mask >>= 1;
                }
            }
            string.push('\n');
            string.push('\r');
        }
        string
    }
}
