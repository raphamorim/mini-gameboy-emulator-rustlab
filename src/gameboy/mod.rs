mod cpu;
mod gpu;
mod mmu;

use crate::gameboy::cpu::Cpu;

pub struct Keypad {
    row0: u8,
    row1: u8,
    data: u8,
    pub interrupt: u8,
}

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

impl Default for Keypad {
    fn default() -> Self {
        Keypad {
            row0: 0x0F,
            row1: 0x0F,
            data: 0xFF,
            interrupt: 0,
        }
    }
}

impl Keypad {
    pub fn rb(&self) -> u8 {
        self.data
    }

    pub fn wb(&mut self, value: u8) {
        self.data = (self.data & 0xCF) | (value & 0x30);
        self.update();
    }

    fn update(&mut self) {
        let old_values = self.data & 0xF;
        let mut new_values = 0xF;

        if self.data & 0x10 == 0x00 {
            new_values &= self.row0;
        }
        if self.data & 0x20 == 0x00 {
            new_values &= self.row1;
        }

        if old_values == 0xF && new_values != 0xF {
            self.interrupt |= 0x10;
        }

        self.data = (self.data & 0xF0) | new_values;
    }

    pub fn keydown(&mut self, key: Button) {
        match key {
            Button::Right => self.row0 &= !(1 << 0),
            Button::Left => self.row0 &= !(1 << 1),
            Button::Up => self.row0 &= !(1 << 2),
            Button::Down => self.row0 &= !(1 << 3),
            Button::A => self.row1 &= !(1 << 0),
            Button::B => self.row1 &= !(1 << 1),
            Button::Select => self.row1 &= !(1 << 2),
            Button::Start => self.row1 &= !(1 << 3),
        }
        self.update();
    }

    pub fn keyup(&mut self, key: Button) {
        match key {
            Button::Right => self.row0 |= 1 << 0,
            Button::Left => self.row0 |= 1 << 1,
            Button::Up => self.row0 |= 1 << 2,
            Button::Down => self.row0 |= 1 << 3,
            Button::A => self.row1 |= 1 << 0,
            Button::B => self.row1 |= 1 << 1,
            Button::Select => self.row1 |= 1 << 2,
            Button::Start => self.row1 |= 1 << 3,
        }
        self.update();
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
        self.cpu.memory.keypad.keydown(key);
    }
    pub fn keyup(&mut self, key: Button) {
        self.cpu.memory.keypad.keyup(key);
    }
}
