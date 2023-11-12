mod cpu;
mod gpu;
mod mmu;

use crate::gameboy::cpu::Cpu;

#[derive(Copy, Clone)]
pub enum Button {
    A,
    B,
    Left,
    Right,
    Up,
    Down,
    Start,
    Select,
}

pub struct Input {
    current: u8,
    buttons: u8,
    directions: u8,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            current: 0x10,
            buttons: 0xf,
            directions: 0xf,
        }
    }
}

impl Input {
    pub fn read_byte(&self) -> u8 {
        match self.current {
            0x20 => self.buttons,
            0x10 => self.directions,
            _ => 0xf,
        }
    }

    pub fn write_byte(&mut self, value: u8) {
        match !value & 0x30 {
            0x20 => self.current = 0x20,
            0x10 => self.current = 0x10,
            _ => {}
        }
    }

    // The eight gameboy buttons/direction keys are arranged in form of a 2x4 matrix.
    // Select either button or direction keys by writing to this register, then read-out bit 0-3.
    // Bit 7 - Not used
    // Bit 6 - Not used
    // Bit 5 - P15 Select Button Keys      (0=Select)
    // Bit 4 - P14 Select Direction Keys   (0=Select)
    // Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
    // Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
    // Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
    // Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
    //
    // Example
    // Bit 3 - P13 Input Down or Start (0=Pressed) 0111 = 0x7
    // Bit 2 - P12 Input Up or Select (0=Pressed) 1011 = 0xb
    // Bit 1 - P11 Input Left or Button B (0=Pressed) 1101 = 0xd
    // Bit 0 - P10 Input Right or Button A (0=Pressed) 1110 = 0xe
    pub fn keydown(&mut self, key: Button) {
        match key {
            Button::A => {
                self.buttons &= 0xe;
            }
            Button::B => {
                self.buttons &= 0xd;
            }
            Button::Start => {
                self.buttons &= 0x7;
            }
            Button::Select => {
                self.buttons &= 0xb;
            }
            Button::Left => {
                self.directions &= 0xd;
            }
            Button::Up => {
                self.directions &= 0xb;
            }
            Button::Down => {
                self.directions &= 0x7;
            }
            Button::Right => {
                self.directions &= 0xe;
            }
        }
    }

    pub fn keyup(&mut self, key: Button) {
        match key {
            Button::A => {
                self.buttons |= !0xe;
            }
            Button::B => {
                self.buttons |= !0xd;
            }
            Button::Start => {
                self.buttons |= !0x7;
            }
            Button::Select => {
                self.buttons |= !0xb;
            }
            Button::Left => {
                self.directions |= !0xd;
            }
            Button::Up => {
                self.directions |= !0xb;
            }
            Button::Down => {
                self.directions |= !0x7;
            }
            Button::Right => {
                self.directions |= !0xe;
            }
        }
    }
}

pub struct MemoryBankController {
    rom: Vec<u8>,
    rombank: usize,
    rombanks: usize,
}

impl MemoryBankController {
    pub fn new(data: Vec<u8>) -> MemoryBankController {
        MemoryBankController {
            rom: data,
            rombank: 1,
            rombanks: 8,
        }
    }
    pub fn readrom(&self, a: u16) -> u8 {
        let bank = if a < 0x4000 { 0 } else { self.rombank };
        let idx = (bank * 0x4000) | ((a as usize) & 0x3FFF);
        *self.rom.get(idx).unwrap_or(&0xFF)
    }
    pub fn writerom(&mut self, a: u16, v: u8) {
        if let 0x2000..=0x3FFF = a {
            let lower = match (v as usize) & 0x1F {
                0 => 1,
                n => n,
            };
            self.rombank = ((self.rombank & 0x60) | lower) % self.rombanks;
        }
    }
}

pub struct GameBoy {
    width: u32,
    height: u32,
    cpu: Cpu,
}

impl GameBoy {
    pub fn new(rom_data: &[u8]) -> Self {
        Self {
            cpu: Cpu::new(rom_data.to_vec()),
            width: 160,
            height: 144,
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn frame(&mut self) {
        let waitticks = 70224;
        let mut ticks = 0;

        'frame: loop {
            while ticks < waitticks {
                ticks += self.cpu.do_cycle();
                if self.cpu.memory.gpu.updated {
                    self.cpu.memory.gpu.updated = false;
                    break 'frame;
                }
            }

            ticks -= waitticks;
        }
    }
    pub fn data(&self) -> &[u8] {
        &*self.cpu.memory.gpu.data
    }
    pub fn keydown(&mut self, key: Button) {
        self.cpu.memory.input.keydown(key);
    }
    pub fn keyup(&mut self, key: Button) {
        self.cpu.memory.input.keyup(key);
    }
}
