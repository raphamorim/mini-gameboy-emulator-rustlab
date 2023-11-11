// Originally ported from https://github.com/mvdnes/rboy
// Licensed under MIT License https://github.com/mvdnes/rboy/blob/master/LICENSE
//
// Also for inspiration:
// https://github.com/alexcrichton/jba/blob/rust/src/gpu.rs

pub struct Gpu {
    mode: u8,
    clock: u32,
    line: u8,
    lyc: u8,
    lcd_on: bool,
    win_tilemap: u16,
    tilebase: u16,
    bg_tilemap: u16,
    sprite_size: u32,
    sprite_on: bool,
    ly: bool,
    scy: u8,
    scx: u8,
    winy: u8,
    winx: u8,
    wy_trigger: bool,
    wy_pos: i32,
    palbr: u8,
    pal0r: u8,
    palb: [u8; 4],
    pal0: [u8; 4],
    pub vram: [u8; 8 << 10],
    pub voam: [u8; 0xA0],
    vrambank: usize,
    pub data: Box<[u8; 92160]>,
    pub updated: bool,
    pub interrupt: u8,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            mode: 0,
            clock: 0,
            line: 0,
            lyc: 0,
            lcd_on: false,
            win_tilemap: 0x9C00,
            tilebase: 0x8000,
            bg_tilemap: 0x9C00,
            sprite_size: 8,
            sprite_on: false,
            ly: false,
            scy: 0,
            scx: 0,
            winy: 0,
            winx: 0,
            wy_trigger: false,
            wy_pos: -1,
            palbr: 0,
            pal0r: 0,
            palb: [0; 4],
            pal0: [0; 4],
            vram: [0; 8 << 10],
            voam: [0; 0xA0],
            data: Box::new([0; 92160]),
            updated: false,
            interrupt: 0,
            vrambank: 0,
        }
    }

    pub fn do_cycle(&mut self, ticks: u32) {
        if !self.lcd_on {
            return;
        }

        let mut ticksleft = ticks;

        while ticksleft > 0 {
            let curticks = if ticksleft >= 80 { 80 } else { ticksleft };
            self.clock += curticks;
            ticksleft -= curticks;

            // If clock >= 456, then we've completed an entire line. This line might
            // have been part of a vblank or part of a scanline.
            if self.clock >= 456 {
                self.clock -= 456;
                self.line = (self.line + 1) % 154;
                if self.ly && self.line == self.lyc {
                    self.interrupt |= 0x02;
                }

                if self.line >= 144 && self.mode != 1 {
                    self.change_mode(1);
                }
            }

            if self.line < 144 {
                // RDOAM takes 80 cycles
                if self.clock <= 80 {
                    if self.mode != 2 {
                        self.change_mode(2);
                    }
                // RDVRAM takes 172 cycles
                } else if self.clock <= (80 + 172) {
                    if self.mode != 3 {
                        self.change_mode(3);
                    }
                // HBLANK takes rest of time before line rendered
                } else if self.mode != 0 {
                    self.change_mode(0);
                }
            }
        }
    }

    fn change_mode(&mut self, mode: u8) {
        self.mode = mode;

        if match self.mode {
            0 => {
                for x in 0..160 {
                    self.data[self.line as usize * 160 * 4 + x * 4] = 255;
                    self.data[self.line as usize * 160 * 4 + x * 4 + 1] = 255;
                    self.data[self.line as usize * 160 * 4 + x * 4 + 2] = 255;
                }
                self.draw_background();
                self.draw_sprites();
                false
            }
            1 => {
                self.wy_trigger = false;
                self.interrupt |= 0x01;
                self.updated = true;
                false
            }
            3 => {
                if !self.wy_trigger && self.line == self.winy {
                    self.wy_trigger = true;
                    self.wy_pos = -1;
                }
                false
            }
            _ => false,
        } {
            self.interrupt |= 0x02;
        }
    }

    pub fn read_byte(&self, a: u16) -> u8 {
        match a {
            0x8000..=0x9FFF => {
                self.vram[(self.vrambank * 0x2000) | (a as usize & 0x1FFF)]
            }
            0xFE00..=0xFE9F => self.voam[a as usize - 0xFE00],
            0xFF40 => {
                (if self.lcd_on { 0x80 } else { 0 })
                    | (if self.win_tilemap == 0x9C00 { 0x40 } else { 0 })
                    | (if self.tilebase == 0x8000 { 0x10 } else { 0 })
                    | (if self.bg_tilemap == 0x9C00 { 0x08 } else { 0 })
                    | (if self.sprite_size == 16 { 0x04 } else { 0 })
                    | (if self.sprite_on { 0x02 } else { 0 })
            }
            0xFF41 => {
                0x80 | (if self.ly { 0x40 } else { 0 })
                    | (if self.line == self.lyc { 0x04 } else { 0 })
                    | self.mode
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.line,
            0xFF45 => self.lyc,
            0xFF47 => self.palbr,
            0xFF4A => self.winy,
            0xFF4B => self.winx,
            0xFF4F => self.vrambank as u8 | 0xFE,
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, a: u16, v: u8) {
        match a {
            0x8000..=0x9FFF => {
                self.vram[(self.vrambank * 0x2000) | (a as usize & 0x1FFF)] = v
            }
            0xFE00..=0xFE9F => self.voam[a as usize - 0xFE00] = v,
            0xFF40 => {
                self.lcd_on = v & 0x80 == 0x80;
                self.win_tilemap = if v & 0x40 == 0x40 { 0x9C00 } else { 0x9800 };
                self.tilebase = if v & 0x10 == 0x10 { 0x8000 } else { 0x8800 };
                self.bg_tilemap = if v & 0x08 == 0x08 { 0x9C00 } else { 0x9800 };
                self.sprite_size = if v & 0x04 == 0x04 { 16 } else { 8 };
                self.sprite_on = v & 0x02 == 0x02;
                if !self.lcd_on {
                    self.clock = 0;
                    self.line = 0;
                    self.mode = 0;
                    self.wy_trigger = false;
                    for v in self.data.iter_mut() {
                        *v = 255;
                    }
                    self.updated = true;
                }
            }
            0xFF41 => {
                self.ly = v & 0x40 == 0x40;
            }
            0xFF42 => self.scy = v,
            0xFF43 => self.scx = v,
            0xFF45 => self.lyc = v,
            0xFF47 => {
                self.palbr = v;
                for index in 0..4 {
                    self.palb[index] = {
                        match (self.palbr >> (2 * index)) & 0x03 {
                            0 => 255,
                            1 => 192,
                            2 => 96,
                            _ => 0,
                        }
                    };
                    self.pal0[index] = {
                        match (self.pal0r >> (2 * index)) & 0x03 {
                            0 => 255,
                            1 => 192,
                            2 => 96,
                            _ => 0,
                        }
                    };
                }
            }
            0xFF48 => {
                self.pal0r = v;
                for index in 0..4 {
                    self.palb[index] = {
                        match (self.palbr >> (2 * index)) & 0x03 {
                            0 => 255,
                            1 => 192,
                            2 => 96,
                            _ => 0,
                        }
                    };
                    self.pal0[index] = {
                        match (self.pal0r >> (2 * index)) & 0x03 {
                            0 => 255,
                            1 => 192,
                            2 => 96,
                            _ => 0,
                        }
                    };
                }
            }
            0xFF49 => {
                for index in 0..4 {
                    self.palb[index] = {
                        match (self.palbr >> (2 * index)) & 0x03 {
                            0 => 255,
                            1 => 192,
                            2 => 96,
                            _ => 0,
                        }
                    };
                    self.pal0[index] = {
                        match (self.pal0r >> (2 * index)) & 0x03 {
                            0 => 255,
                            1 => 192,
                            2 => 96,
                            _ => 0,
                        }
                    };
                }
            }
            0xFF4A => self.winy = v,
            0xFF4B => self.winx = v,
            0xFF4F => self.vrambank = (v & 0x01) as usize,
            _ => {}
        }
    }

    fn draw_background(&mut self) {
        let drawbg = true;

        let wx_trigger = self.winx <= 166;
        let winy = if self.wy_trigger && wx_trigger {
            self.wy_pos += 1;
            self.wy_pos
        } else {
            -1
        };

        if winy < 0 && !drawbg {
            return;
        }

        let wintiley = (winy as u16 >> 3) & 31;

        let bgy = self.scy.wrapping_add(self.line);
        let bgtiley = (bgy as u16 >> 3) & 31;

        for x in 0..160 {
            let winx = -((self.winx as i32) - 7) + (x as i32);
            let bgx = self.scx as u32 + x as u32;

            let (tilemapbase, tiley, tilex, pixely, pixelx) = if winy >= 0 && winx >= 0 {
                (
                    self.win_tilemap,
                    wintiley,
                    (winx as u16 >> 3),
                    winy as u16 & 0x07,
                    winx as u8 & 0x07,
                )
            } else if drawbg {
                (
                    self.bg_tilemap,
                    bgtiley,
                    (bgx as u16 >> 3) & 31,
                    bgy as u16 & 0x07,
                    bgx as u8 & 0x07,
                )
            } else {
                continue;
            };

            let tilenr: u8 =
                self.vram[(tilemapbase + tiley * 32 + tilex) as usize & 0x1FFF];

            let tileaddress = self.tilebase
                + (if self.tilebase == 0x8000 {
                    tilenr as u16
                } else {
                    (tilenr as i8 as i16 + 128) as u16
                }) * 16;

            let a0 = tileaddress + (pixely * 2);

            let (b1, b2) = (
                self.vram[a0 as usize & 0x1FFF],
                self.vram[(a0 + 1) as usize & 0x1FFF],
            );

            let xbit = 7 - pixelx as u32;
            let colnr = if b1 & (1 << xbit) != 0 { 1 } else { 0 }
                | if b2 & (1 << xbit) != 0 { 2 } else { 0 };

            let color = self.palb[colnr];
            self.data[self.line as usize * 160 * 4 + x * 4] = color;
            self.data[self.line as usize * 160 * 4 + x * 4 + 1] = color;
            self.data[self.line as usize * 160 * 4 + x * 4 + 2] = color;
        }
    }

    fn draw_sprites(&mut self) {
        if !self.sprite_on {
            return;
        }

        let line = self.line as i32;
        let sprite_size = self.sprite_size as i32;

        let mut sprites_to_draw = [(0, 0, 0); 10];
        let mut sidx = 0;
        for index in 0..40 {
            let spriteaddr = 0xFE00 + (index as u16) * 4;
            let spritey = self.read_byte(spriteaddr) as u16 as i32 - 16;
            if line < spritey || line >= spritey + sprite_size {
                continue;
            }
            let spritex = self.read_byte(spriteaddr + 1) as u16 as i32 - 8;
            sprites_to_draw[sidx] = (spritex, spritey, index);
            sidx += 1;
            if sidx >= 10 {
                break;
            }
        }

        sprites_to_draw[..sidx].sort_unstable_by(|a, b| {
            if a.0 != b.0 {
                return b.0.cmp(&a.0);
            }
            b.2.cmp(&a.2)
        });

        for &(spritex, spritey, i) in &sprites_to_draw[..sidx] {
            if !(-7..160).contains(&spritex) {
                continue;
            }

            let spriteaddr = 0xFE00 + (i as u16) * 4;
            let tilenum = (self.read_byte(spriteaddr + 2)
                & (if self.sprite_size == 16 { 0xFE } else { 0xFF }))
                as u16;
            let flags = self.read_byte(spriteaddr + 3) as usize;
            let xflip: bool = flags & (1 << 5) != 0;
            let yflip: bool = flags & (1 << 6) != 0;

            let tiley: u16 = if yflip {
                (sprite_size - 1 - (line - spritey)) as u16
            } else {
                (line - spritey) as u16
            };

            let tileaddress = 0x8000u16 + tilenum * 16 + tiley * 2;
            let (b1, b2) = (
                self.vram[tileaddress as usize & 0x1FFF],
                self.vram[(tileaddress + 1) as usize & 0x1FFF],
            );

            for x in 0..8 {
                if spritex + x < 0 || spritex + x >= 160 {
                    continue;
                }

                let xbit = 1 << (if xflip { x } else { 7 - x } as u32);
                let colnr = (if b1 & xbit != 0 { 1 } else { 0 })
                    | (if b2 & xbit != 0 { 2 } else { 0 });
                if colnr == 0 {
                    continue;
                }
                let color = self.pal0[colnr];
                self.data[self.line as usize * 160 * 4 + ((spritex + x) as usize) * 4] =
                    color;
                self.data
                    [self.line as usize * 160 * 4 + ((spritex + x) as usize) * 4 + 1] =
                    color;
                self.data
                    [self.line as usize * 160 * 4 + ((spritex + x) as usize) * 4 + 2] =
                    color;
            }
        }
    }
}
