#![allow(dead_code)]

const PALETTE: [(u8, u8, u8); 16] = [
    (0,   0,   0),   // 0: Transparent (render as black)
    (0,   0,   0),   // 1: Black
    (33,  200, 66),  // 2: Medium Green
    (94,  220, 120), // 3: Light Green
    (84,  85,  237), // 4: Dark Blue
    (125, 118, 252), // 5: Light Blue
    (212, 82,  77),  // 6: Dark Red
    (66,  235, 245), // 7: Cyan
    (252, 85,  84),  // 8: Medium Red
    (255, 121, 120), // 9: Light Red
    (212, 193, 84),  // 10: Dark Yellow
    (230, 206, 128), // 11: Light Yellow
    (33,  176, 59),  // 12: Dark Green
    (201, 91,  186), // 13: Magenta
    (204, 204, 204), // 14: Gray
    (255, 255, 255), // 15: White
];

pub struct Vdp {
    pub vram: Box<[u8; 16384]>,
    pub regs: [u8; 8],
    pub status: u8,
    pub addr: u16,
    pub addr_latch: bool,
    pub latch_byte: u8,
    pub read_buf: u8,
    pub int_pending: bool,
    pub line: u32,
    pub cycles: u32,
}

impl Vdp {
    pub fn new() -> Self {
        Vdp {
            vram: Box::new([0u8; 16384]),
            regs: [0u8; 8],
            status: 0,
            addr: 0,
            addr_latch: false,
            latch_byte: 0,
            read_buf: 0,
            int_pending: false,
            line: 0,
            cycles: 0,
        }
    }

    pub fn write_data(&mut self, val: u8) {
        self.vram[self.addr as usize & 0x3FFF] = val;
        self.read_buf = val;
        self.addr = (self.addr + 1) & 0x3FFF;
        self.addr_latch = false;
    }

    pub fn write_ctrl(&mut self, val: u8) {
        if !self.addr_latch {
            self.latch_byte = val;
            self.addr_latch = true;
        } else {
            self.addr_latch = false;
            if val & 0x80 == 0 {
                // address set
                self.addr = ((val as u16 & 0x3F) << 8) | self.latch_byte as u16;
                if val & 0x40 == 0 {
                    // read pre-fetch
                    self.read_buf = self.vram[self.addr as usize & 0x3FFF];
                    self.addr = (self.addr + 1) & 0x3FFF;
                }
            } else {
                // register write
                let reg = (val & 0x07) as usize;
                if reg < 8 {
                    self.regs[reg] = self.latch_byte;
                }
            }
        }
    }

    pub fn read_data(&mut self) -> u8 {
        let v = self.read_buf;
        self.read_buf = self.vram[self.addr as usize & 0x3FFF];
        self.addr = (self.addr + 1) & 0x3FFF;
        self.addr_latch = false;
        v
    }

    pub fn read_status(&mut self) -> u8 {
        let s = self.status;
        self.status &= 0x1F;
        self.int_pending = false;
        self.addr_latch = false;
        s
    }

    pub fn take_interrupt(&mut self) -> bool {
        if self.int_pending && (self.regs[1] & 0x20 != 0) {
            self.int_pending = false;
            true
        } else {
            false
        }
    }

    fn screen_mode(&self) -> u8 {
        let m1 = (self.regs[1] >> 4) & 1; // M1 = R1[4]
        let m2 = (self.regs[1] >> 3) & 1; // M2 = R1[3]  (actually M2=R1[3])
        let m3 = (self.regs[0] >> 1) & 1; // M3 = R0[1]
        // Screen 0 (Text)   : M1=1, M2=0, M3=0
        // Screen 1 (G1)     : M1=0, M2=0, M3=0
        // Screen 2 (G2)     : M1=0, M2=0, M3=1
        // Screen 3 (MC)     : M1=0, M2=1, M3=0
        if m1 == 1 { 0 } // text mode = 0
        else if m3 == 1 { 2 } // G2
        else if m2 == 1 { 3 } // Multicolor
        else { 1 } // G1 (default)
    }

    pub fn render_frame(&mut self, output: &mut [u8]) {
        if self.regs[1] & 0x40 == 0 {
            // Screen blank: fill with backdrop color
            let bg = (self.regs[7] & 0x0F) as usize;
            let (r, g, b) = PALETTE[bg];
            for i in 0..(256 * 192) {
                output[i * 4]     = r;
                output[i * 4 + 1] = g;
                output[i * 4 + 2] = b;
                output[i * 4 + 3] = 255;
            }
            return;
        }

        match self.screen_mode() {
            0 => self.render_text(output),
            1 => self.render_g1(output),
            2 => self.render_g2(output),
            3 => self.render_mc(output),
            _ => self.render_g1(output),
        }

        self.render_sprites(output);
    }

    /// Text mode (Screen 0): 40×24 chars, 6×8 pixels per char
    fn render_text(&mut self, output: &mut [u8]) {
        let name_base  = ((self.regs[2] & 0x0F) as usize) << 10;
        let pat_base   = ((self.regs[4] & 0x07) as usize) << 11;
        let fg_idx = ((self.regs[7] >> 4) & 0x0F) as usize;
        let bg_idx = ( self.regs[7]       & 0x0F) as usize;
        let (fg_r, fg_g, fg_b) = PALETTE[if fg_idx == 0 { 1 } else { fg_idx }];
        let (bg_r, bg_g, bg_b) = PALETTE[if bg_idx == 0 { 1 } else { bg_idx }];

        for row in 0..24usize {
            for col in 0..40usize {
                let name_addr = name_base + row * 40 + col;
                let ch = self.vram[name_addr & 0x3FFF] as usize;
                for line in 0..8usize {
                    let pat = self.vram[(pat_base + ch * 8 + line) & 0x3FFF];
                    let py = row * 8 + line;
                    for bit in 0..6usize {
                        let px = col * 6 + bit;
                        if px >= 256 || py >= 192 { continue; }
                        let pixel_set = pat & (0x80 >> bit) != 0;
                        let (r, g, b) = if pixel_set { (fg_r, fg_g, fg_b) } else { (bg_r, bg_g, bg_b) };
                        let off = (py * 256 + px) * 4;
                        output[off]     = r;
                        output[off + 1] = g;
                        output[off + 2] = b;
                        output[off + 3] = 255;
                    }
                }
            }
        }
        // Fill right margin (240..256) with bg
        for py in 0..192usize {
            for px in 240..256usize {
                let off = (py * 256 + px) * 4;
                output[off]     = bg_r;
                output[off + 1] = bg_g;
                output[off + 2] = bg_b;
                output[off + 3] = 255;
            }
        }
    }

    /// Graphic 1 (Screen 1): 32×24 chars, 8×8 pixels
    fn render_g1(&mut self, output: &mut [u8]) {
        let name_base  = ((self.regs[2] & 0x0F) as usize) << 10;
        let color_base = ((self.regs[3]) as usize) << 6;
        let pat_base   = ((self.regs[4] & 0x07) as usize) << 11;
        let bg_idx     = (self.regs[7] & 0x0F) as usize;
        let (bg_r, bg_g, bg_b) = PALETTE[if bg_idx == 0 { 1 } else { bg_idx }];

        for py in 0..192usize {
            for px in 0..256usize {
                let off = (py * 256 + px) * 4;
                output[off + 3] = 255;
                output[off]     = bg_r;
                output[off + 1] = bg_g;
                output[off + 2] = bg_b;
            }
        }

        for row in 0..24usize {
            for col in 0..32usize {
                let name_addr = name_base + row * 32 + col;
                let ch = self.vram[name_addr & 0x3FFF] as usize;
                let color_byte = self.vram[(color_base + (ch >> 3)) & 0x3FFF];
                let fg_idx = ((color_byte >> 4) & 0x0F) as usize;
                let bg2_idx = (color_byte & 0x0F) as usize;
                let (fg_r, fg_g, fg_b) = PALETTE[if fg_idx == 0 { 1 } else { fg_idx }];
                let (bg2_r, bg2_g, bg2_b) = PALETTE[if bg2_idx == 0 { 1 } else { bg2_idx }];

                for line in 0..8usize {
                    let pat = self.vram[(pat_base + ch * 8 + line) & 0x3FFF];
                    let py = row * 8 + line;
                    for bit in 0..8usize {
                        let px = col * 8 + bit;
                        let pixel_set = pat & (0x80 >> bit) != 0;
                        let (r, g, b) = if pixel_set { (fg_r, fg_g, fg_b) } else { (bg2_r, bg2_g, bg2_b) };
                        let off = (py * 256 + px) * 4;
                        output[off]     = r;
                        output[off + 1] = g;
                        output[off + 2] = b;
                        output[off + 3] = 255;
                    }
                }
            }
        }
    }

    /// Graphic 2 (Screen 2): VRAM divided into 3 sections of 256 patterns each
    fn render_g2(&mut self, output: &mut [u8]) {
        let name_base  = ((self.regs[2] & 0x0F) as usize) << 10;
        let color_base = ((self.regs[3] & 0x80) as usize) << 6; // upper bit selects bank
        let pat_base   = ((self.regs[4] & 0x04) as usize) << 11;
        let bg_idx = (self.regs[7] & 0x0F) as usize;
        let (bg_r, bg_g, bg_b) = PALETTE[if bg_idx == 0 { 1 } else { bg_idx }];

        for py in 0..192usize {
            for px in 0..256usize {
                let off = (py * 256 + px) * 4;
                output[off]     = bg_r;
                output[off + 1] = bg_g;
                output[off + 2] = bg_b;
                output[off + 3] = 255;
            }
        }

        for row in 0..24usize {
            let section = row / 8; // 0,1,2
            for col in 0..32usize {
                let name_addr = name_base + row * 32 + col;
                let ch = self.vram[name_addr & 0x3FFF] as usize;
                let base_off = section * 0x800;

                for line in 0..8usize {
                    let pat_addr  = (pat_base   + base_off + ch * 8 + line) & 0x3FFF;
                    let color_addr= (color_base + base_off + ch * 8 + line) & 0x3FFF;
                    let pat   = self.vram[pat_addr];
                    let color = self.vram[color_addr];
                    let fg_idx = ((color >> 4) & 0x0F) as usize;
                    let bg2_idx = (color & 0x0F) as usize;
                    let (fg_r, fg_g, fg_b) = PALETTE[if fg_idx == 0 { 1 } else { fg_idx }];
                    let (bg2_r, bg2_g, bg2_b) = PALETTE[if bg2_idx == 0 { 1 } else { bg2_idx }];

                    let py = row * 8 + line;
                    for bit in 0..8usize {
                        let px = col * 8 + bit;
                        let pixel_set = pat & (0x80 >> bit) != 0;
                        let (r, g, b) = if pixel_set { (fg_r, fg_g, fg_b) } else { (bg2_r, bg2_g, bg2_b) };
                        let off = (py * 256 + px) * 4;
                        output[off]     = r;
                        output[off + 1] = g;
                        output[off + 2] = b;
                        output[off + 3] = 255;
                    }
                }
            }
        }
    }

    /// Multicolor (Screen 3)
    fn render_mc(&mut self, output: &mut [u8]) {
        let name_base = ((self.regs[2] & 0x0F) as usize) << 10;
        let pat_base  = ((self.regs[4] & 0x07) as usize) << 11;

        for row in 0..24usize {
            for col in 0..32usize {
                let name_addr = name_base + row * 32 + col;
                let ch = self.vram[name_addr & 0x3FFF] as usize;
                for line in 0..8usize {
                    let pat_line = (row / 4) * 2 + line / 4; // which pattern byte (2 per char)
                    let pat = self.vram[(pat_base + ch * 8 + pat_line) & 0x3FFF];
                    // left nibble = left 4 pixels, right nibble = right 4 pixels
                    let left_col  = ((pat >> 4) & 0x0F) as usize;
                    let right_col = (pat & 0x0F) as usize;
                    let py = row * 8 + line;
                    for bit in 0..8usize {
                        let px = col * 8 + bit;
                        let ci = if bit < 4 { left_col } else { right_col };
                        let (r, g, b) = PALETTE[if ci == 0 { 1 } else { ci }];
                        let off = (py * 256 + px) * 4;
                        output[off]     = r;
                        output[off + 1] = g;
                        output[off + 2] = b;
                        output[off + 3] = 255;
                    }
                }
            }
        }
    }

    /// Sprite rendering (G1 / G2 / MC modes)
    fn render_sprites(&mut self, output: &mut [u8]) {
        // No sprites in text mode
        if self.screen_mode() == 0 { return; }

        let sat_base  = ((self.regs[5] & 0x7F) as usize) << 7;
        let spat_base = ((self.regs[6] & 0x07) as usize) << 11;
        let size16    = self.regs[1] & 0x02 != 0;
        let magnify   = self.regs[1] & 0x01 != 0;
        let sprite_size = if size16 { 16 } else { 8 };
        let display_size = if magnify { sprite_size * 2 } else { sprite_size };

        // Track sprites per line for 4-sprite limit
        let mut line_count = [0u8; 192];

        for spr in 0..32usize {
            let base = sat_base + spr * 4;
            let y_raw = self.vram[(base)     & 0x3FFF] as i32;
            let x     = self.vram[(base + 1) & 0x3FFF] as i32;
            let pattern = self.vram[(base + 2) & 0x3FFF] as usize;
            let attr  = self.vram[(base + 3) & 0x3FFF];

            // Y=208 (0xD0) signals end of sprite table
            if y_raw == 208 { break; }

            let y = (y_raw + 1) as i32;
            let color_idx = (attr & 0x0F) as usize;
            let ec = attr & 0x80 != 0; // early clock shift

            let pat_idx = if size16 { pattern & !3 } else { pattern };
            let spat_addr = spat_base + pat_idx * 8;

            for row in 0..sprite_size {
                let screen_y = y + if magnify { (row * 2) as i32 } else { row as i32 };
                let screen_y2 = if magnify { screen_y + 1 } else { screen_y };

                for dy in screen_y..=screen_y2 {
                    if dy < 0 || dy >= 192 { continue; }
                    if line_count[dy as usize] >= 4 { continue; }

                    // Get pattern byte
                    let (pat_byte, x_offset) = if size16 {
                        // 16x16: 4 bytes per row (2 bytes horizontal × 8 rows, then lower 8 rows)
                        let half = if row < 8 { 0 } else { 1 };
                        let byte_row = row % 8;
                        let lo = self.vram[(spat_addr + half * 8 + byte_row) & 0x3FFF];
                        let hi = self.vram[(spat_addr + half * 8 + byte_row + 16) & 0x3FFF];
                        let _ = hi;
                        (lo, 0)
                    } else {
                        (self.vram[(spat_addr + row) & 0x3FFF], 0)
                    };
                    let _ = x_offset;

                    // Draw 8 pixels
                    for bit in 0..8usize {
                        if pat_byte & (0x80 >> bit) == 0 { continue; }
                        if color_idx == 0 { continue; } // transparent
                        let sx = x - if ec { 32 } else { 0 } + if magnify { (bit * 2) as i32 } else { bit as i32 };
                        for ddx in 0..=(if magnify { 1 } else { 0 }) {
                            let screen_x = sx + ddx;
                            if screen_x < 0 || screen_x >= 256 { continue; }
                            let (r, g, b) = PALETTE[color_idx];
                            let off = (dy as usize * 256 + screen_x as usize) * 4;
                            output[off]     = r;
                            output[off + 1] = g;
                            output[off + 2] = b;
                            output[off + 3] = 255;
                        }
                    }
                    line_count[dy as usize] += 1;
                }
            }
        }
    }
}
