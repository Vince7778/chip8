// Contains the structure of the Chip8.

use rand::RngCore;
use rand_pcg::Lcg64Xsh32;
use std::{error::Error, fmt};
use crate::hexes::HEXES_FLAT;

pub const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const REG_V0: usize = 0x00;
const REG_VF: usize = 0x0F;

const MEMORY_OFFSET: usize = 0x0200;

const INSTRUCTION_SIZE: u16 = 2;

#[derive(Debug)]
pub struct Chip8 {
    pub memory: [u8; MEMORY_SIZE],
    registers: [u8; REGISTER_COUNT],
    stack: [u16; STACK_SIZE],
    pub frame_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT / 8],
    sp: u8,     // stack pointer
    ir: u16,    // index register
    pub dt: u8,     // delay timer
    st: u8,     // sound timer
    pub pc: u16,    // program counter
    rng: Lcg64Xsh32,
    pub keypad_waiting: bool,
    keypad_reg: u8,
    pub display_changed: bool,
    pub sound_playing: bool
}

#[derive(Debug)]
pub enum ChipError {
    // Operation doesn't exist
    BadOperationError(u8, u8),
    // Stack is empty, can't pop off
    EmptyStackError,
    // Stack is full, can't push
    FullStackError,
    // Invalid program counter (out of bounds)
    ProgramCounterError(u16),
    // Can't read this much into memory
    MemoryOverflowError,
}

impl Error for ChipError {}

impl fmt::Display for ChipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_str = match self {
            ChipError::BadOperationError(b1, b2) => format!("Bad operation with bytes 0x{:02X}{:02X}", b1, b2),
            ChipError::EmptyStackError => "Tried to pop off empty stack".to_string(),
            ChipError::FullStackError => "Tried to push onto full stack".to_string(),
            ChipError::ProgramCounterError(pc) => format!("Program counter out of bounds: 0x{:02X}", pc),
            ChipError::MemoryOverflowError => format!("Memory overflowed"),
        };
        write!(f, "PROCESSOR ERROR: {}", err_str)
    }
}

enum ProgramCounterControl {
    Next,
    Skip,
    Jump(u16)
}

impl Chip8 {
    pub fn new(rng: Lcg64Xsh32) -> Self {
        let mut memory = [0; MEMORY_SIZE];
        // load hex into memory
        for (ind, val) in HEXES_FLAT.iter().enumerate() {
            memory[ind] = *val;
        }

        Chip8 {
            memory,
            registers: [0; REGISTER_COUNT],
            stack: [0; STACK_SIZE],
            frame_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT / 8],
            sp: 0,     // stack pointer
            ir: 0,    // index register
            dt: 0,     // delay timer
            st: 0,     // sound timer
            pc: MEMORY_OFFSET as u16,    // program counter
            rng,
            keypad_waiting: false,
            keypad_reg: 0,
            display_changed: false,
            sound_playing: false
        }
    }

    pub fn load(&mut self, bytes: &Vec<u8>) -> Result<(), ChipError> {
        for (i, &byte) in bytes.iter().enumerate() {
            let mem_addr = MEMORY_OFFSET + i;
            if mem_addr >= MEMORY_SIZE {
                return Err(ChipError::MemoryOverflowError);
            } else {
                self.memory[mem_addr] = byte;
            }
        }
        Ok(())
    }

    // Pop from top of stack
    fn stack_pop(&mut self) -> Result<u16, ChipError> {
        if self.sp == 0xFF {
            Err(ChipError::EmptyStackError)
        } else {
            let val = self.stack[self.sp as usize];
            self.sp -= 1;
            Ok(val)
        }
    }

    // Push to top of stack
    fn stack_push(&mut self, val: u16) -> Result<(), ChipError> {
        if self.sp == 0xFF {
            Err(ChipError::FullStackError)
        } else {
            self.sp += 1;
            self.stack[self.sp as usize] = val;
            Ok(())
        }
    }

    // Clear display
    fn op_cls(&mut self) -> ProgramCounterControl {
        for i in 0..self.frame_buffer.len() {
            self.frame_buffer[i] = 0;
        }
        ProgramCounterControl::Next
    }

    // Return from subroutine
    fn op_ret(&mut self) -> Result<ProgramCounterControl, ChipError> {
        let new_addr = self.stack_pop()?;
        Ok(ProgramCounterControl::Jump(new_addr))
    }

    // Jump
    fn op_jp(&mut self, addr: u16) -> ProgramCounterControl {
        ProgramCounterControl::Jump(addr)
    }

    // Call subroutine
    fn op_call(&mut self, addr: u16) -> Result<ProgramCounterControl, ChipError> {
        self.stack_push(self.pc + INSTRUCTION_SIZE)?;
        Ok(ProgramCounterControl::Jump(addr))
    }

    // Skip if equal
    fn op_se(&mut self, reg: u8, val: u8) -> ProgramCounterControl {
        if self.registers[reg as usize] == val {
            ProgramCounterControl::Skip
        } else {
            ProgramCounterControl::Next
        }
    }

    // Skip if not equal
    fn op_sne(&mut self, reg: u8, val: u8) -> ProgramCounterControl {
        if self.registers[reg as usize] != val {
            ProgramCounterControl::Skip
        } else {
            ProgramCounterControl::Next
        }
    }

    // Skip if equal, compare registers
    fn op_se_reg(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        if self.registers[reg1 as usize] == self.registers[reg2 as usize] {
            ProgramCounterControl::Skip
        } else {
            ProgramCounterControl::Next
        }
    }

    // Load value into register
    fn op_ld(&mut self, reg: u8, val: u8) -> ProgramCounterControl {
        self.registers[reg as usize] = val;
        ProgramCounterControl::Next
    }

    fn op_add(&mut self, reg: u8, val: u8) -> ProgramCounterControl {
        self.registers[reg as usize] = self.registers[reg as usize].overflowing_add(val).0;
        ProgramCounterControl::Next
    }

    // Load register 2 into register 1
    fn op_ld_reg(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        self.registers[reg1 as usize] = self.registers[reg2 as usize];
        ProgramCounterControl::Next
    }

    // Or registers
    fn op_or(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        self.registers[reg1 as usize] |= self.registers[reg2 as usize];
        ProgramCounterControl::Next
    }

    // And registers
    fn op_and(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        self.registers[reg1 as usize] &= self.registers[reg2 as usize];
        ProgramCounterControl::Next
    }

    // Xor registers
    fn op_xor(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        self.registers[reg1 as usize] ^= self.registers[reg2 as usize];
        ProgramCounterControl::Next
    }

    // Add registers, setting VF if overflow
    fn op_add_reg(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        let val1 = self.registers[reg1 as usize];
        let val2 = self.registers[reg2 as usize];
        let result = val1.overflowing_add(val2);
        self.registers[reg1 as usize] = result.0;
        self.registers[REG_VF] = result.1 as u8;
        ProgramCounterControl::Next
    }

    // Subtract registers, setting VF if NO underflow
    fn op_sub_reg(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        let val1 = self.registers[reg1 as usize];
        let val2 = self.registers[reg2 as usize];
        let result = val1.overflowing_sub(val2);
        self.registers[reg1 as usize] = result.0;
        self.registers[REG_VF] = (!result.1) as u8;
        ProgramCounterControl::Next
    }

    // Shift right, setting VF if LSB == 1
    fn op_shr(&mut self, reg: u8) -> ProgramCounterControl {
        let val = self.registers[reg as usize];
        self.registers[reg as usize] = val >> 1;
        self.registers[REG_VF] = val & 1;
        ProgramCounterControl::Next
    }

    // Subtract registers (but reg2-reg1 this time), setting VF if NO underflow
    fn op_subn_reg(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        let val1 = self.registers[reg1 as usize];
        let val2 = self.registers[reg2 as usize];
        let result = val2.overflowing_sub(val1);
        self.registers[reg1 as usize] = result.0;
        self.registers[REG_VF] = (!result.1) as u8;
        ProgramCounterControl::Next
    }

    // Shift left, setting VF if MSB == 1
    fn op_shl(&mut self, reg: u8) -> ProgramCounterControl {
        let val = self.registers[reg as usize];
        self.registers[reg as usize] = val << 1;
        self.registers[REG_VF] = (val >> 7) & 1;
        ProgramCounterControl::Next
    }

    // Skip if not equal, compare registers
    fn op_sne_reg(&mut self, reg1: u8, reg2: u8) -> ProgramCounterControl {
        if self.registers[reg1 as usize] != self.registers[reg2 as usize] {
            ProgramCounterControl::Skip
        } else {
            ProgramCounterControl::Next
        }
    }

    // Load into index register
    fn op_ld_i(&mut self, val: u16) -> ProgramCounterControl {
        self.ir = val;
        ProgramCounterControl::Next
    }

    // Jump to V0 + addr
    fn op_jp_v0(&mut self, addr: u16) -> ProgramCounterControl {
        ProgramCounterControl::Jump(self.registers[REG_V0] as u16 + addr)
    }

    // Get random number
    fn op_rnd(&mut self, reg: u8, val: u8) -> ProgramCounterControl {
        self.registers[reg as usize] = self.rng.next_u32() as u8 & val;
        ProgramCounterControl::Next
    }

    // Draw to screen
    fn op_drw(&mut self, regx: u8, regy: u8, byte_count: u8) -> ProgramCounterControl {
        let xpos = self.registers[regx as usize] as usize;
        let ypos = self.registers[regy as usize] as usize;
        let mut overlap = false;

        for byte in 0..byte_count {
            let memory_byte = self.memory[self.ir as usize + byte as usize];
            let cur_y = (ypos + byte as usize) % SCREEN_HEIGHT;

            for bit in 0..8 {
                if (memory_byte & (1 << (7 - bit))) > 0 {
                    let cur_x = (xpos + bit as usize) % SCREEN_WIDTH;
                    let display_byte_index = (cur_y * SCREEN_WIDTH + cur_x) / 8;
                    let display_byte = &mut self.frame_buffer[display_byte_index];
                    let display_bit_index = 7 - (cur_x % 8);
                    let display_mask = 1 << display_bit_index;
                    // check overlap
                    if (*display_byte & display_mask) > 0 {
                        overlap = true;
                    }
                    *display_byte ^= display_mask;
                }
            }
        }

        self.registers[REG_VF] = overlap as u8;
        self.display_changed = true;
        ProgramCounterControl::Next
    }

    // skip if key pressed
    fn op_skp(&mut self, reg: u8, key_input: u16) -> ProgramCounterControl {
        let val = self.registers[reg as usize];
        if (key_input & (1 << val)) > 0 {
            ProgramCounterControl::Skip
        } else {
            ProgramCounterControl::Next
        }
    }

    // skip if key not pressed
    fn op_sknp(&mut self, reg: u8, key_input: u16) -> ProgramCounterControl {
        let val = self.registers[reg as usize];
        if (key_input & (1u16 << val)) == 0 {
            ProgramCounterControl::Skip
        } else {
            ProgramCounterControl::Next
        }
    }

    // load delay timer into reg
    fn op_ld_vx_dt(&mut self, reg: u8) -> ProgramCounterControl {
        self.registers[reg as usize] = self.dt;
        ProgramCounterControl::Next
    }

    // wait until key press
    fn op_key_wait(&mut self, reg: u8) -> ProgramCounterControl {
        self.keypad_waiting = true;
        self.keypad_reg = reg;
        ProgramCounterControl::Next
    }

    // load reg into delay timer
    fn op_ld_dt_vx(&mut self, reg: u8) -> ProgramCounterControl {
        self.dt = self.registers[reg as usize];
        ProgramCounterControl::Next
    }

    // load reg into sound timer
    fn op_ld_st_vx(&mut self, reg: u8) -> ProgramCounterControl {
        self.st = self.registers[reg as usize];
        ProgramCounterControl::Next
    }

    fn op_add_i_vx(&mut self, reg: u8) -> ProgramCounterControl {
        self.ir += self.registers[reg as usize] as u16;
        ProgramCounterControl::Next
    }

    fn op_ld_f_vx(&mut self, reg: u8) -> ProgramCounterControl {
        let digit = self.registers[reg as usize];
        self.ir = ((digit & 0xF) * 5) as u16;
        ProgramCounterControl::Next
    }

    fn op_ld_b_vx(&mut self, reg: u8) -> ProgramCounterControl {
        let val = self.registers[reg as usize];
        self.memory[self.ir as usize] = (val / 100) % 10;
        self.memory[self.ir as usize+1] = (val / 10) % 10;
        self.memory[self.ir as usize+2] = val % 10;
        ProgramCounterControl::Next
    }

    // Stores registers v0 through vx into memory.
    fn op_ld_i_vx(&mut self, reg: u8) -> ProgramCounterControl {
        for ind in 0..(reg as usize+1) {
            self.memory[self.ir as usize + ind] = self.registers[ind];
        } 
        ProgramCounterControl::Next
    }

    // Reads registers v0 through vx from memory.
    fn op_ld_vx_i(&mut self, reg: u8) -> ProgramCounterControl {
        for ind in 0..(reg as usize+1) {
            self.registers[ind] = self.memory[self.ir as usize + ind];
        }
        ProgramCounterControl::Next
    }

    fn run(&mut self, b1: u8, b2: u8, key_input: u16) -> Result<ProgramCounterControl, ChipError> {
        //println!("{:03x} {}", self.pc, translator::translate(b1, b2));
        let top_b1 = b1 >> 4;
        let bottom_b1 = b1 & 0x0F;
        let top_b2 = b2 >> 4;
        let bottom_b2 = b2 & 0x0F;
        match top_b1 {
            0x0 => match b2 {
                0xE0 => Ok(self.op_cls()),
                0xEE => self.op_ret(),
                _ => Err(ChipError::BadOperationError(b1, b2))
            }
            0x1 => Ok(self.op_jp(get_nnn(b1, b2))),
            0x2 => self.op_call(get_nnn(b1, b2)),
            0x3 => Ok(self.op_se(bottom_b1, b2)),
            0x4 => Ok(self.op_sne(bottom_b1, b2)),
            0x5 => match bottom_b2 {
                0x0 => Ok(self.op_se_reg(bottom_b1, top_b2)),
                _ => Err(ChipError::BadOperationError(b1, b2)),
            },
            0x6 => Ok(self.op_ld(bottom_b1, b2)),
            0x7 => Ok(self.op_add(bottom_b1, b2)),
            0x8 => match bottom_b2 {
                0x0 => Ok(self.op_ld_reg(bottom_b1, top_b2)),
                0x1 => Ok(self.op_or(bottom_b1, top_b2)),
                0x2 => Ok(self.op_and(bottom_b1, top_b2)),
                0x3 => Ok(self.op_xor(bottom_b1, top_b2)),
                0x4 => Ok(self.op_add_reg(bottom_b1, top_b2)),
                0x5 => Ok(self.op_sub_reg(bottom_b1, top_b2)),
                0x6 => Ok(self.op_shr(bottom_b1)),
                0x7 => Ok(self.op_subn_reg(bottom_b1, top_b2)),
                0xE => Ok(self.op_shl(bottom_b1)),
                _ => Err(ChipError::BadOperationError(b1, b2))
            },
            0x9 => match bottom_b2 {
                0x0 => Ok(self.op_sne_reg(bottom_b1, top_b2)),
                _ => Err(ChipError::BadOperationError(b1, b2))
            },
            0xA => Ok(self.op_ld_i(get_nnn(b1, b2))),
            0xB => Ok(self.op_jp_v0(get_nnn(b1, b2))),
            0xC => Ok(self.op_rnd(bottom_b1, b2)),
            0xD => Ok(self.op_drw(bottom_b1, top_b2, bottom_b2)),
            0xE => match b2 {
                0x9E => Ok(self.op_skp(bottom_b1, key_input)),
                0xA1 => Ok(self.op_sknp(bottom_b1, key_input)),
                _ => Err(ChipError::BadOperationError(b1, b2)),
            },
            0xF => match b2 {
                0x07 => Ok(self.op_ld_vx_dt(bottom_b1)),
                0x0A => Ok(self.op_key_wait(bottom_b1)),
                0x15 => Ok(self.op_ld_dt_vx(bottom_b1)),
                0x18 => Ok(self.op_ld_st_vx(bottom_b1)),
                0x1E => Ok(self.op_add_i_vx(bottom_b1)),
                0x29 => Ok(self.op_ld_f_vx(bottom_b1)),
                0x33 => Ok(self.op_ld_b_vx(bottom_b1)),
                0x55 => Ok(self.op_ld_i_vx(bottom_b1)),
                0x65 => Ok(self.op_ld_vx_i(bottom_b1)),
                _ => Err(ChipError::BadOperationError(b1, b2))
            }
            _ => Err(ChipError::BadOperationError(b1, b2))
        }
    }

    pub fn tick(&mut self, key_input: u16) -> Result<(), ChipError> {
        if self.pc as usize > MEMORY_SIZE-INSTRUCTION_SIZE as usize {
            return Err(ChipError::ProgramCounterError(self.pc));
        }
        let instruction = (self.memory[self.pc as usize], self.memory[self.pc as usize+1]);

        let res = self.run(instruction.0, instruction.1, key_input)?;
        match res {
            ProgramCounterControl::Next => self.pc += INSTRUCTION_SIZE,
            ProgramCounterControl::Skip => self.pc += 2*INSTRUCTION_SIZE,
            ProgramCounterControl::Jump(addr) => self.pc = addr,
        };

        Ok(())
    }

    // Runs after 1/60 sec has elapsed and timers should be ticked down.
    pub fn frame(&mut self) {
        self.display_changed = false;
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
        self.sound_playing = self.st > 0;
    }

    pub fn keypad_press(&mut self, key_press: u16) {
        if self.keypad_waiting {
            if let Some(bit) = get_lowest_bit_pos(key_press) {
                self.keypad_waiting = false;

                self.registers[self.keypad_reg as usize] = bit as u8;
            }
        }
    }
}

// Gets the last three bits from an instruction.
pub fn get_nnn(b1: u8, b2: u8) -> u16 {
    let x1 = b1 as u16;
    let x2 = b2 as u16;
    (x1 * 0x0100 + x2) & 0x0FFF
}

// Get lowest set bit index
fn get_lowest_bit_pos(num: u16) -> Option<u8> {
    for i in 0..16 {
        if (num & (1 << i)) > 0 {
            return Some(i);
        }
    }
    return None;
}
