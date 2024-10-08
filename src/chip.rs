use std::time::{Duration, Instant};

use rand::Rng;

use crate::{config::*, screen::Interface};

#[allow(dead_code)]
pub struct Register {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
    va: u8,
    vb: u8,
    vc: u8,
    vd: u8,
    ve: u8,
    vf: u8,
    i: u16,
}

#[allow(dead_code)]
impl Register {
    pub fn set_reg_v(&mut self, reg: u8, val: u8) {
        match reg {
            0x0 => self.v0 = val,
            0x1 => self.v1 = val,
            0x2 => self.v2 = val,
            0x3 => self.v3 = val,
            0x4 => self.v4 = val,
            0x5 => self.v5 = val,
            0x6 => self.v6 = val,
            0x7 => self.v7 = val,
            0x8 => self.v8 = val,
            0x9 => self.v9 = val,
            0xa => self.va = val,
            0xb => self.vb = val,
            0xc => self.vc = val,
            0xd => self.vd = val,
            0xe => self.ve = val,
            0xf => self.vf = val,
            _ => panic!("Invalid register set access, reg: v{reg:x}"),
        };
    }

    pub fn get_reg_v(&self, reg: u8) -> u8 {
        match reg {
            0x0 => self.v0,
            0x1 => self.v1,
            0x2 => self.v2,
            0x3 => self.v3,
            0x4 => self.v4,
            0x5 => self.v5,
            0x6 => self.v6,
            0x7 => self.v7,
            0x8 => self.v8,
            0x9 => self.v9,
            0xA => self.va,
            0xB => self.vb,
            0xC => self.vc,
            0xD => self.vd,
            0xE => self.ve,
            0xF => self.vf,
            _ => panic!("Invalid register get access, reg: v {reg:02x}"),
        }
    }
}

pub struct Chip<T>
where
    T: Interface,
{
    pub running: bool,
    pub memory: [u8; MEMSIZE],
    pub pc: u16,
    pub registers: Register,
    pub stack: [u16; 16],
    pub stackpointer: u8,
    pub interface: T,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keyboard: Option<u8>,
    release_key_wait: Option<u8>,
}

#[allow(dead_code)]
impl<T: Interface> Chip<T> {
    pub fn new(prog_counter: u16, interface: T) -> Self {
        Chip {
            running: false,
            memory: [0; MEMSIZE],
            pc: prog_counter,
            registers: Register {
                v0: 0,
                v1: 0,
                v2: 0,
                v3: 0,
                v4: 0,
                v5: 0,
                v6: 0,
                v7: 0,
                v8: 0,
                v9: 0,
                va: 0,
                vb: 0,
                vc: 0,
                vd: 0,
                ve: 0,
                vf: 0,
                i: 0,
            },
            stack: [0; 16],
            stackpointer: 0,
            interface,
            delay_timer: 0,
            sound_timer: 0,
            keyboard: None,
            release_key_wait: None,
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        let target_frame_time = Duration::from_millis(1 / SCREEN_REFRESH_RATE as u64);

        while self.running {
            let frame_start = Instant::now();

            // Execute next instructions for frame
            for _ in 0..(INSTRUCTION_FREQUENCY / SCREEN_REFRESH_RATE) {
                self.execute_inst();
            }

            // Update Screen
            self.interface.update_screen();

            // Update Sound and Delay timer
            self.delay_timer -= 1;
            self.sound_timer -= 1;
            let frame_duration = Instant::now() - frame_start;
            if frame_duration > target_frame_time {
                std::thread::sleep(target_frame_time - frame_duration);
            }
        }
    }

    pub fn init_interface(&self) {
        self.interface.init();
    }

    pub fn stop_interface(&self) {
        self.interface.stop();
    }

    pub fn load_prog(&mut self, prog: Vec<u8>) {
        for (i, inst_part) in prog.iter().enumerate() {
            self.memory[PROG_POS_START as usize + i] = *inst_part;
        }
    }

    pub fn execute_inst(&mut self) {
        let val: u16 = 0
            | (((self.memory[self.pc as usize] as u16) << 8)
                | self.memory[(self.pc + 1) as usize] as u16);
        let a: u16 = (val & 0xF000) >> 12;
        let b: u16 = (val & 0x0F00) >> 8;
        let c: u16 = (val & 0x00F0) >> 4;
        let d: u16 = val & 0x000F;
        match a {
            0x0 => match (c << 4) | d {
                // Cls, Clear the screen
                0xE0 => {
                    self.interface.clear_screen();
                    self.pc += 2;
                }
                // Ret, Return from subroutine
                0xEE => {
                    self.stackpointer -= 1;
                    self.pc = self.stack[self.stackpointer as usize];
                    self.pc += 2;
                }
                // Empty, Does nothing
                _ => self.pc += 2,
            },
            // Jmp, Jump to addr
            0x1 => self.pc = b | c | d,
            // Call, Call subroutine at addr
            0x2 => {
                self.stack[self.stackpointer as usize] = self.pc;
                self.stackpointer += 1;
                self.pc = b | c | d;
            }
            // Skip next inst if value in vx == byte
            0x3 => {
                if self.registers.get_reg_v(b as u8) == (c | d) as u8 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Skip next inst if value in vx != byte
            0x4 => {
                if self.registers.get_reg_v(b as u8) != (c | d) as u8 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Skip if val in vx == val in vy
            0x5 => {
                if self.registers.get_reg_v(b as u8) == self.registers.get_reg_v(c as u8) {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Loads byte into vx
            0x6 => {
                self.registers.set_reg_v(b as u8, (c | d) as u8);
                self.pc += 2;
            }
            // Adds byte to vx
            0x7 => {
                let (res, _) = self
                    .registers
                    .get_reg_v((b >> 8) as u8)
                    .overflowing_add((c | d) as u8);
                self.registers.set_reg_v(b as u8, res);
                self.pc += 2;
            }
            0x8 => match d {
                // Loads val of vy in vx
                0x0 => {
                    self.registers
                        .set_reg_v(b as u8, self.registers.get_reg_v(c as u8));
                    self.pc += 2;
                }
                // Bitwise or of vx and vy, stores result in vx
                0x1 => {
                    let val = self.registers.get_reg_v(b as u8) | self.registers.get_reg_v(c as u8);
                    self.registers.set_reg_v(b as u8, val);
                    self.registers.set_reg_v(0xF, 0);
                    self.pc += 2;
                }
                // Bitwise and of vx and vy, stores result in vx
                0x2 => {
                    let val = self.registers.get_reg_v(b as u8) & self.registers.get_reg_v(c as u8);
                    self.registers.set_reg_v(b as u8, val);
                    self.registers.set_reg_v(0xF, 0);
                    self.pc += 2;
                }
                // Bitwise xor of vx and vy, stores result in vx
                0x3 => {
                    let val = self.registers.get_reg_v(b as u8) ^ self.registers.get_reg_v(c as u8);
                    self.registers.set_reg_v(b as u8, val);
                    self.registers.set_reg_v(0xF, 0);
                    self.pc += 2;
                }
                // Add vx and vy, result stored in vx, if overflow (vx + vy >= 255) VF set to 1
                0x4 => {
                    let (res, vf) = self
                        .registers
                        .get_reg_v(b as u8)
                        .overflowing_add(self.registers.get_reg_v(c as u8));
                    self.registers.set_reg_v(b as u8, res);
                    self.registers.set_reg_v(0xF, vf as u8);
                    self.pc += 2;
                }
                // Subtract vy from vx, result stored in vx, if vx > vy VF set to 1, otherwise 0
                0x5 => {
                    let (res, vf) = self
                        .registers
                        .get_reg_v(b as u8)
                        .overflowing_sub(self.registers.get_reg_v(c as u8));
                    self.registers.set_reg_v(b as u8, res);
                    self.registers.set_reg_v(0xF, !vf as u8);
                    self.pc += 2;
                }
                // Shift vx right, VF set to least significant bit of vx
                0x6 => {
                    let val = self.registers.get_reg_v(c as u8);
                    self.registers.set_reg_v(b as u8, val >> 1);
                    self.registers.set_reg_v(0xF, val & 0x1);
                    self.pc += 2;
                }
                // Subtract vx from vy, result stored in vx, if vy > vx VF set to 1, otherwise 0
                0x7 => {
                    let (res, vf) = self
                        .registers
                        .get_reg_v(c as u8)
                        .overflowing_sub(self.registers.get_reg_v(b as u8));
                    self.registers.set_reg_v(b as u8, res);
                    self.registers.set_reg_v(0xF, !vf as u8);
                    self.pc += 2;
                }
                // Shift vx left, VF set to most significant bit of vx
                0xE => {
                    let val = self.registers.get_reg_v(c as u8);
                    self.registers.set_reg_v(b as u8, val << 1);
                    self.registers.set_reg_v(0xF, (val >> 7) & 0x1);
                    self.pc += 2;
                }
                _ => panic!("Illegal instruction {val}"),
            },
            // Skip if val in vx != val in vy
            0x9 => {
                self.pc += 2;
                if self.registers.get_reg_v(b as u8) != self.registers.get_reg_v(c as u8) {
                    self.pc += 2;
                }
            }
            // Load addr into register I
            0xA => {
                self.registers.i = b | c | d;
                self.pc += 2;
            }
            // Jumps to addr + V0
            0xB => {
                self.pc = b | c | d + self.registers.get_reg_v(0x0) as u16;
            }
            // Moves rnd value (0-255) & byte into vx
            0xC => {
                let val = (rand::thread_rng().gen_range(0..=255) as u8) & (c | d) as u8;
                self.registers.set_reg_v(b as u8, val);
                self.pc += 2;
            }
            // Display n-byte sprite starting at memory location I at (vx, vy), set VF = collision
            0xD => {
                let mut sprite_buffer: Vec<u8> = Vec::new();
                for i in 0..(d as u8) {
                    sprite_buffer.push(self.memory[(self.registers.i + i as u16) as usize]);
                }
                if self.interface.draw_sprite(
                    self.registers.get_reg_v(b as u8),
                    self.registers.get_reg_v(c as u8),
                    sprite_buffer,
                ) {
                    self.registers.set_reg_v(0xF, 1);
                } else {
                    self.registers.set_reg_v(0xF, 0);
                }
                self.pc += 2;
            }

            0xE => match (c >> 4) | d {
                // Skip next instruction if key with the value of vx is pressed
                0x9E => {
                    let target = self.registers.get_reg_v(b as u8);
                    if self.interface.get_key(target) {
                        self.pc += 2;
                    }
                    self.pc += 2;
                }
                // Skip next instruction if key with the value of vx is not pressed
                0xA1 => {
                    let target = self.registers.get_reg_v(b as u8);
                    if !self.interface.get_key(target) {
                        self.pc += 2;
                    }
                    self.pc += 2;
                }
                _ => panic!("Illegal instruction {val}"),
            },

            0xF => match (c >> 4) | d {
                // Set vx to delay timer val
                0x07 => {
                    self.registers.set_reg_v(b as u8, self.delay_timer);
                    self.pc += 2;
                }
                // Wait for a key press, store the value of the key in vx
                0x0A => {
                    // TODO:
                    if let Some(key) = self.release_key_wait {
                        if !self.interface.get_key(key) {
                            self.release_key_wait = None;
                            self.pc += 2;
                        }
                    } else {
                        if let Some(key) = self.keyboard {
                            self.registers.set_reg_v(b as u8, key);
                            self.release_key_wait = Some(key);
                        }
                    }
                }
                // Set delay timer value to vx
                0x15 => {
                    self.delay_timer = self.registers.get_reg_v(b as u8);
                    self.pc += 2;
                }
                // Set sound timer value to vx
                0x18 => {
                    self.sound_timer = self.registers.get_reg_v(b as u8);
                    self.pc += 2;
                }
                // Add vx to I
                0x1E => {
                    let val = self.registers.i + self.registers.get_reg_v(b as u8) as u16;
                    self.registers.i = val;
                    self.pc += 2;
                }
                // Set I = location of font char for val of vx
                0x29 => {
                    let x = self.registers.get_reg_v(b as u8);
                    self.registers.i = (FONT_POS_START + 5 * x as usize) as u16;
                    self.pc += 2;
                }
                // Store BCD representation of vx in memory locations pointed to by I, I+1, and I+2
                0x33 => {
                    let val = self.registers.get_reg_v(b as u8);
                    self.memory[self.registers.i as usize] = val / 100;
                    self.memory[(self.registers.i + 1) as usize] = (val % 100) / 10;
                    self.memory[(self.registers.i + 2) as usize] = (val % 100) % 10;
                    self.pc += 2;
                }
                // Store registers v0 through vx in memory starting at location I
                0x55 => {
                    let i = self.registers.i;
                    for x in 0..=b as u8 {
                        self.memory[(i + x as u16) as usize] = self.registers.get_reg_v(x);
                    }
                    self.registers.i = i + b as u8 as u16 + 1;
                    self.pc += 2;
                }
                // Read registers V0 through Vx from memory starting at location I
                0x65 => {
                    let i = self.registers.i;
                    for x in 0..=b as u8 {
                        self.registers
                            .set_reg_v(x, self.memory[(i + x as u16) as usize]);
                    }
                    self.registers.i = i + b as u8 as u16 + 1;
                    self.pc += 2;
                }
                _ => panic!("Illegal instruction {val}"),
            },
            _ => panic!("Illegal instruction {val}"),
        }
    }

    fn load_font(&mut self) {
        let font: [u8; 80] = [
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
            0xF0, 0x90, 0xF0, 0x90, 0x90, // a
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // b
            0xF0, 0x80, 0x80, 0x80, 0xF0, // c
            0xE0, 0x90, 0x90, 0x90, 0xE0, // d
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // e
            0xF0, 0x80, 0xF0, 0x80, 0x80, // f
        ];
        for (i, byte) in font.iter().enumerate() {
            self.memory[FONT_POS_START + i] = *byte;
        }
    }

    fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    fn set_byte(&mut self, addr: u16, val: u8) {
        if (addr as usize) < MEMSIZE {
            self.memory[addr as usize] = val;
        } else {
            panic!("Tried to access memory out of bounds, Memsize: {MEMSIZE}, addr: {addr}");
        }
    }

    fn get_addr(&self, addr: u16) -> u8 {
        if (addr as usize) < MEMSIZE {
            self.memory[addr as usize]
        } else {
            panic!("Tried to access memory out of bounds, Memsize: {MEMSIZE}, addr: {addr}");
        }
    }
}
