
// See: https://en.wikipedia.org/wiki/CHIP-8#Virtual_machine_description 

// font sprites
const FONTS: [u8; 80] = [
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

pub struct Chip8 {
    // main memory
    // 
    // 0x000 -> 0x200 System reserved
    // 0x200 -> 0xF00 Programs
    // 0xF00 -> 0XFFF Display
    // 
    memory: [u8; 4096],

    // registers
    pc: u16, // PC: Program counter (current instruction)
    i: u16, // I: Index register
    v: [u8; 16], // V[N] where N: 0x0..=0xf (General purpose registers)

    // timers
    delay_timer: u8, // decremented at 60Hz
    sound_timer: u8, // play a sound while this is non-zero

    // stack pointer
    sp: usize, // current location on the stack
    stack: [u16; 16], // stores return addresses for subroutines

    // inputs
    keypad: [bool; 16], // whether each of the keys (0x0..=0xf) are pressed
}

impl Chip8 {
    fn fetch(&mut self) -> u16 {
        let result = 
            ((self.memory[self.pc as usize] as u16) << 8) |
             (self.memory[self.pc as usize] as u16);
        self.pc += 2;
        result
    }
    fn execute(&mut self, instruction: u16) {
        let first_nibble = (instruction & 0xF000) >> 12;
        let second_nibble = (instruction & 0x0F00) >> 8;
        let third_nibble = (instruction & 0x00F0) >> 4;
        let fourth_nibble = instruction & 0x000F;
        let second_byte: u8 = (instruction & 0x00FF) as u8;
        match first_nibble {
            0x0 => {
                match instruction {
                    0x00E0 => {
                        // 0x00E0
                        // Clear display
                        self.memory[0xF00..=0xFFF].fill(0);
                    },
                    0x00EE => {
                        // 0x00EE
                        // Return from subroutine
                        unimplemented!();
                    },
                    _ => {
                        // 0xNNN
                        // Call machine code routine at address NNN
                        unimplemented!();
                    }
                };
            },
            0x1 => {
                // 0x1NNN
                // Jump to address NNN (set pc to NNN)
                self.pc = instruction & 0x0FFF;
            },
            0x2 => {
                // 0x2NNN
                // Call subroutine at NNN
                unimplemented!();
            },
            0x3 => {
                // 0x3XNN
                // Skips the next instruction if VX == NN
                if self.v[second_nibble as usize] == second_byte {
                    self.pc += 2;
                }
            },
            0x4 => {
                // 0x4XNN
                // Skips the next instruction if VX != NN
                if self.v[second_nibble as usize] != second_byte {
                    self.pc += 2;
                }
            },
            0x5 => {
                // 0x5XY0
                // Skips the next instruction if VX == VY
                if self.v[second_nibble as usize] == self.v[third_nibble as usize] {
                    self.pc += 2;
                }
            },
            0x6 => {
                // 0x6XNN
                // Sets register X to NN
                self.v[second_nibble as usize] = second_byte;
            },
            0x7 => {
                // 0x7XNN
                // Adds NN to register X
                self.v[second_nibble as usize] += second_byte;
            },
            0x8 => {
                match fourth_nibble {
                    0x0 => {
                        // 0x8XY0
                        // Sets VX to VY
                        self.v[second_nibble as usize] = self.v[third_nibble as usize];
                    },
                    0x1 => {
                        // 0x8XY1
                        // Sets VX to VX bitwise or VY
                        self.v[second_nibble as usize] |= self.v[third_nibble as usize];
                    },
                    0x2 => {
                        // 0x8XY2
                        // Sets VX to VX bitwise and VY
                        self.v[second_nibble as usize] &= self.v[third_nibble as usize];
                    },
                    0x3 => {
                        // 0x8XY3
                        // Sets VX to VX bitwise xor VY
                        self.v[second_nibble as usize] ^= self.v[third_nibble as usize];
                    },
                    0x4 => {
                        // 0x8XY4
                        // Sets VX to VX + VY
                        // If there is a carry, VF is set to 1, otherwise VF is set to 0
                        unimplemented!();
                    },
                    0x5 => {
                        // 0x8XY5
                        // Sets VX to VX - VY
                        // VF set to 0 when there is a borrow, 1 if not
                        unimplemented!();
                    },
                    0x6 => {
                        // 0x8XY6
                        // Stores LSB of VX in VF, then right shifts VX by one
                        self.v[0xF] = self.v[second_nibble as usize] & 1;
                        self.v[second_nibble as usize] >>= 1;
                    },
                    0x7 => {
                        // 0x8XY7
                        // Sets VX to VY - VX
                        // VF set to 0 when there is a borrow, 1 if not
                        unimplemented!();
                    },
                    0xE => {
                        // 0x8XYE
                        // Stores MSB of VX in VF, then right shifts VX by one
                        self.v[0xF] = self.v[second_nibble as usize] & 0b10000000;
                        self.v[second_nibble as usize] <<= 1;
                    },
                    _ => {
                        unimplemented!();
                    }
                }
            },
            0x9 => {
                // 0x9XY0
                // Skips the next instruction if VX != XY
                if self.v[second_nibble as usize] != self.v[third_nibble as usize] {
                    self.pc += 2;
                }
            },
            0xA => {
                // 0xANNN
                // Sets I to address NNN
                self.i = instruction & 0x0FFF;
            },
            0xB => {
                // 0xBNNN
                // Jumps to address NNN + V0
                self.pc = self.v[0x0] as u16 + (instruction & 0x0FFF);
            },
            0xC => {
                // 0xCXNN
                // Sets VX to the bitwise and of a 1 byte random number and NN
                unimplemented!();
            },
            0xD => {
                // 0xDXYN
                // Draws a sprite at coordinate (VX, VY)
                //   - width: 8 px
                //   - height: (N+1) px
                // Read from address stored in I
                // VF set to 1 if any screen pixels are flipped from set to unset, otherwise to 0
                unimplemented!();
            },
            0xE => {
                match second_byte {
                    0x9E => {
                        // 0xEX9E
                        // Skips the next instruction if the keyboard key stored in VX is pressed.
                        unimplemented!();
                    },
                    0xA1 => {
                        // 0xEXA1
                        // Skips the next instruction if the keyboard key stored in VX is not pressed.
                        unimplemented!();
                    },
                    _ => {
                        unimplemented!();
                    }
                }
            },
            0xF => {
                match second_byte {
                    0x07 => {
                        // 0xFX07
                        // Sets VX to the value of the delay timer. 
                        unimplemented!();
                    },
                    0x0A => {
                        // 0xFX0A
                        // A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event) 
                        unimplemented!();
                    },
                    0x15 => {
                        // 0xFX15
                        // Sets the delay timer to VX. 
                        unimplemented!();
                    },
                    0x18 => {
                        // 0xFX18
                        // Sets the sound timer to VX.
                        unimplemented!();
                    },
                    0x1E => {
                        // 0xFX1E
                        // Adds VX to I. VF is not affected
                        unimplemented!();
                    },
                    0x29 => {
                        // 0xFX29
                        // Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font. 
                        unimplemented!();
                    },
                    0x33 => {
                        // 0xFX33
                        // Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.) 
                        unimplemented!();
                    },
                    0x55 => {
                        // 0xFX55
                        // Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
                        unimplemented!();
                    },
                    0x65 => {
                        // 0xFX65
                        // Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
                        unimplemented!();
                    },
                    _ => {
                        unimplemented!();
                    }
                }
            },
            _ => {
                unimplemented!();
            },
        };
    }
}
