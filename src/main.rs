extern crate minifb;
extern crate rand;
use std::thread;
use std::time::{Duration, Instant};


use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};



const SCALE_FACTOR: usize = 10; // Scale factor to enlarge the display
const WINDOW_WIDTH: usize = WIDTH * SCALE_FACTOR;
const WINDOW_HEIGHT: usize = HEIGHT * SCALE_FACTOR;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const CYCLES_PER_FRAME: usize = 16;

struct Chip8 {
    pc: u16,
    index: u16,
    sp: u16,
    memory: [u8; MEMORY_SIZE],
    v: [u8; NUM_REGISTERS],
    stack: [u16; STACK_SIZE],
    screen: [u32; WIDTH * HEIGHT],
    delay_timer: u8,
    sound_timer: u8,
    keys: [bool; NUM_KEYS],
}

const FONT_SET: [u8; 80] = [
    // Font data for 0-F (each character is 5 bytes)
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

const KEY_MAP: [Key; NUM_KEYS] = [
    Key::X,    // 0
    Key::Key1, // 1
    Key::Key2, // 2
    Key::Key3, // 3
    Key::Q,    // 4
    Key::W,    // 5
    Key::E,    // 6
    Key::A,    // 7
    Key::S,    // 8
    Key::D,    // 9
    Key::Z,    // A
    Key::C,    // B
    Key::Key4, // C
    Key::R,    // D
    Key::F,    // E
    Key::V     // F
];

impl Chip8 {
    fn new() -> Self {
        let mut chip = Chip8 {
            pc: 0x200,
            index: 0,
            sp: 0,
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            stack: [0; STACK_SIZE],
            screen: [0; WIDTH * HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; NUM_KEYS],
        };

        for (i, &byte) in FONT_SET.iter().enumerate() {
            chip.memory[0x50 + i] = byte;
        }

        chip
    }

    fn emulate_cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);

        //println!("V0: {}, V1: {}, I: {}", self.v[0], self.v[1], self.index);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn fetch_opcode(&self) -> u16 {
        ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16)
    }

    fn execute_opcode(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.clear_screen(),
                0x00EE => {
                    if self.sp == 0 {
                        panic!("Stack underflow!");
                    }
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                },
                _ => println!("unknown opcode: {:#04x}", opcode),
            },
            0x1000 => {
                self.pc = opcode & 0x0FFF;
            },
            0x2000 => {
                if self.sp >= STACK_SIZE as u16 {
                    panic!("Stack overflow!");
                }
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            },
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                if self.v[x] == value {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x0FF) as u8;
                if self.v[x] != value {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0x6000 => {
                let x= ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.v[x] = value;
                self.pc += 2;
            },
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.v[x] = self.v[x].wrapping_add(value);
                self.pc += 2;
            },
            0x8000 => match opcode & 0x000F {
                0x0000 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.v[x] |= self.v[y];
                    self.pc += 2;
                },
                0x0001 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.v[x] |= self.v[y];
                    self.pc += 2;
                },
                0x0002 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.v[x] &= self.v[y];
                    self.pc += 2;
                },
                0x0003 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.v[x] ^= self.v[y];
                    self.pc += 2;
                },
                0x0004 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[0xF] = if carry { 1 } else { 0 };
                    self.v[x] = sum;
                    self.pc += 2;
                },
                0x0005 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    self.pc += 2;
                },
                0x0006 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.v[0xF] = self.v[x] & 0x1;
                    self.v[x] >>= 1;
                    self.pc += 2;
                },
                0x0007 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    self.pc += 2;
                },
                0x000E => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.v[0xF] = (self.v[x] & 0x80) >> 7;
                    self.v[x] <<= 1;
                    self.pc += 2;
                },
                _ => println!("Unknown opcode: {:#04x}", opcode),
            },
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0xA000 => {
                self.index = opcode & 0x0FFF;
                self.pc += 2;
            },
            0xB000 => {
                self.pc = (opcode & 0x0FFF) + self.v[0] as u16;
            },
            0xC000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.v[x] = rand::random::<u8>() & value;
                self.pc += 2;
            },
            0xD000 => {
                let x = self.v[((opcode & 0x0F00) >> 8) as usize] as usize;
                let y = self.v[((opcode & 0x00F0) >> 4) as usize] as usize;
                let height = (opcode & 0x000F) as usize;

                self.v[0xF] = 0;
                for byte in 0..height {
                    let pixel = self.memory[(self.index as usize) + byte ];
                    for bit in 0..8 {
                        let pos_x = (x + bit) % WIDTH;
                        let pos_y = (y + byte) % HEIGHT;
                        let pixel_value = (pixel >> (7 - bit)) & 1;

                        let idx = pos_x + pos_y * WIDTH;
                        let prev_pixel = self.screen[idx];

                        let color = if pixel_value == 1 {
                            0xFFFFA500
                        }
                        else {
                            0x00000000
                        };

                        if prev_pixel != 0x00000000 && color != 0x00000000  {
                            self.v[0xF] = 1;
                        }

                        self.screen[idx] ^= color;
                    }
                }

                self.pc += 2;
            },
            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    if self.keys[self.v[x] as usize] {
                        self.pc += 2;
                    }
                    self.pc += 2;
                },
                0x00A1 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    if !self.keys[self.v[x] as usize] {
                        self.pc += 2;
                    }
                    self.pc += 2;
                },
                _ => println!("Unknown opcode: {:#04x}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let mut key_pressed = false;
                    for i in 0..NUM_KEYS {
                        if self.keys[i] {
                            self.v[x] = i as u8;
                            key_pressed = true;
                            break;
                        }
                    }
                    if key_pressed {
                        self.pc += 2;
                    }
                },
                0x0015 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.delay_timer = self.v[x];
                    self.pc += 2;
                },
                0x0018 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.sound_timer = self.v[x];
                    self.pc += 2;
                },
                0x001E => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.index = 0x50 + self.v[x] as u16 * 5; // Each digit sprite is 5 bytes
                    self.pc += 2;
                },
                0x0029 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.index = (self.v[x] as u16 * 5) + 0x50; // Each digit is 5 bytes long
                    self.pc += 2;
                },
                0x0033 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.memory[self.index as usize] = self.v[x] / 100;
                    self.memory[self.index as usize + 1] = (self.v[x] % 100) / 10;
                    self.memory[self.index as usize + 2] = self.v[x] % 10;
                    self.pc += 2;
                },
                0x0055 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.memory[self.index as usize + i] = self.v[i];
                    }
                    self.pc += 2;
                },
                0x0065 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.v[i] = self.memory[self.index as usize + i];
                    }
                    self.pc += 2;
                },
                _ => println!("Unknown opcode: {:#04x}", opcode),
            },
            _ => println!("unknown opcode: {:#04x}", opcode),
        }
    }

    fn clear_screen(&mut self) {
        for pixel in self.screen.iter_mut() {
            *pixel = 0;
        }
    }

    fn load_rom(&mut self, filename: &str) {
        let rom = std::fs::read(filename).expect("Unable to read ROM");
        for (i, &byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }
    }
}

fn main () {
    let mut chip = Chip8::new();

    chip.load_rom("flightrunner.ch8");

    let mut window = Window::new(
        "Chip-8 Emulator",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut last_timer_update = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let keys_pressed = window.get_keys_pressed(minifb::KeyRepeat::No);
        chip.keys = [false; NUM_KEYS];

        if let Some(keys) = keys_pressed {
            for key in keys {
                if let Some(idx) = KEY_MAP.iter().position(|&k| k == key) {
                    chip.keys[idx] = true;
                }
            }
        }

        let frame_start = Instant::now();

        if last_timer_update.elapsed() >= Duration::from_millis(16) {
            if chip.delay_timer > 0 {
                chip.delay_timer -= 1;
            }
            if chip.sound_timer > 0 {
                chip.sound_timer -= 1;

                // Implement sound here
            }
            last_timer_update = Instant::now();
        }

        for _ in 0..CYCLES_PER_FRAME {
            chip.emulate_cycle();
        }

        window.update_with_buffer(&chip.screen, WIDTH, HEIGHT).unwrap();

        let elapsed = frame_start.elapsed();
        if elapsed < Duration::from_millis(16) {
            thread::sleep(Duration::from_millis(16) - elapsed);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*; // Import everything from the main module

    #[test]
    fn test_opcode_6xnn() {
        let mut chip = Chip8::new();
        chip.execute_opcode(0x6012);
        assert_eq!(chip.v[0], 0x12);
    }

    #[test]
    fn test_opcode_7xnn() {
        let mut chip = Chip8::new();
        chip.v[0] = 5;
        chip.execute_opcode(0x7003);
        assert_eq!(chip.v[0], 8);
    }

    #[test]
    fn test_opcode_a000() {
        let mut chip = Chip8::new();
        // Emulate opcode 0xA123 (set I to 0x123)
        chip.execute_opcode(0xA123);
        assert_eq!(chip.index, 0x123);
    }
}
