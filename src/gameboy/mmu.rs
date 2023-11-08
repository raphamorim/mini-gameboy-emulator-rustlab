use crate::gameboy::gpu::Gpu;
use crate::gameboy::Keypad;

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

const WRAM_SIZE: usize = 0x8000;
const ZRAM_SIZE: usize = 0x7F;

pub struct MemoryManagementUnit {
    wram: [u8; WRAM_SIZE],
    zram: [u8; ZRAM_SIZE],
    pub inte: u8,
    pub intf: u8,
    pub keypad: Keypad,
    pub gpu: Gpu,
    wrambank: usize,
    pub mbc: MemoryBankController,
}

impl MemoryManagementUnit {
    pub fn new(data: Vec<u8>) -> MemoryManagementUnit {
        let mmu_mbc = MemoryBankController::new(data);

        let mut res = MemoryManagementUnit {
            wram: [0; WRAM_SIZE],
            zram: [0; ZRAM_SIZE],
            wrambank: 1,
            inte: 0,
            intf: 0,
            keypad: Keypad::default(),
            gpu: Gpu::new(),
            mbc: mmu_mbc,
        };
        res.set_initial();
        res
    }

    fn set_initial(&mut self) {
        self.write_byte(0xFF05, 0);
        self.write_byte(0xFF06, 0);
        self.write_byte(0xFF07, 0);
        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF14, 0xBF);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0);
        self.write_byte(0xFF19, 0xBF);
        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1E, 0xFF);
        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0);
        self.write_byte(0xFF22, 0);
        self.write_byte(0xFF23, 0xBF);
        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
        self.write_byte(0xFF40, 0x91);
        self.write_byte(0xFF42, 0);
        self.write_byte(0xFF43, 0);
        self.write_byte(0xFF45, 0);
        self.write_byte(0xFF47, 0xFC);
        self.write_byte(0xFF48, 0xFF);
        self.write_byte(0xFF49, 0xFF);
        self.write_byte(0xFF4A, 0);
        self.write_byte(0xFF4B, 0);
    }

    pub fn do_cycle(&mut self, ticks: u32) -> u32 {
        self.intf |= self.keypad.interrupt;
        self.keypad.interrupt = 0;

        self.gpu.do_cycle(ticks);
        self.intf |= self.gpu.interrupt;
        self.gpu.interrupt = 0;

        ticks
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, (value & 0xFF) as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        (self.read_byte(address) as u16) | ((self.read_byte(address + 1) as u16) << 8)
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.mbc.readrom(address),
            0x8000..=0x9FFF => self.gpu.read_byte(address),
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram[address as usize & 0x0FFF],
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.wram[(self.wrambank * 0x1000) | address as usize & 0x0FFF]
            }
            0xFE00..=0xFE9F => self.gpu.read_byte(address),
            0xFF00 => self.keypad.rb(),
            0xFF0F => self.intf | 0b11100000,
            0xFF40..=0xFF4F => self.gpu.read_byte(address),
            0xFF68..=0xFF6B => self.gpu.read_byte(address),
            0xFF70 => self.wrambank as u8,
            0xFF80..=0xFFFE => self.zram[address as usize & 0x007F],
            0xFFFF => self.inte,
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.mbc.writerom(address, value),
            0x8000..=0x9FFF => self.gpu.write_byte(address, value),
            0xC000..=0xCFFF | 0xE000..=0xEFFF => {
                self.wram[address as usize & 0x0FFF] = value
            }
            0xD000..=0xDFFF | 0xF000..=0xFDFF => {
                self.wram[(self.wrambank * 0x1000) | (address as usize & 0x0FFF)] = value
            }
            0xFE00..=0xFE9F => self.gpu.write_byte(address, value),
            0xFF00 => self.keypad.wb(value),
            0xFF46 => {
                let base = (value as u16) << 8;
                for i in 0..0xA0 {
                    let b = self.read_byte(base + i);
                    self.write_byte(0xFE00 + i, b);
                }
            }
            0xFF40..=0xFF4F => self.gpu.write_byte(address, value),
            0xFF68..=0xFF6B => self.gpu.write_byte(address, value),
            0xFF0F => self.intf = value,
            0xFF70 => {
                self.wrambank = match value & 0x7 {
                    0 => 1,
                    n => n as usize,
                };
            }
            0xFF80..=0xFFFE => self.zram[address as usize & 0x007F] = value,
            0xFFFF => self.inte = value,
            _ => {}
        };
    }
}
