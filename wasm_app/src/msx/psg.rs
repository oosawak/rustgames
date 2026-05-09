#![allow(dead_code)]

pub struct Psg {
    pub regs: [u8; 16],
    pub addr: u8,
    tone_counter: [u32; 3],
    tone_out: [bool; 3],
    noise_counter: u32,
    noise_lfsr: u32,
    noise_out: bool,
    env_counter: u32,
    env_step: u8,
    env_hold: bool,
    env_dir: i8,
    env_volume: u8,
    sample_counter: u32,
    pub samples: Vec<f32>,
}

// AY-3-8910: MSX clock 3.58MHz, PSG clock = CPU clock / 2 = ~1.79MHz
// Tone period in PSG clocks: period = reg * 8 (fine + coarse * 256)
// We accumulate at CPU cycles, divide by 16 for PSG clock ticks

impl Psg {
    pub fn new() -> Self {
        Psg {
            regs: [0; 16],
            addr: 0,
            tone_counter: [0; 3],
            tone_out: [false; 3],
            noise_counter: 0,
            noise_lfsr: 1,
            noise_out: false,
            env_counter: 0,
            env_step: 0,
            env_hold: false,
            env_dir: 1,
            env_volume: 15,
            sample_counter: 0,
            samples: Vec::new(),
        }
    }

    pub fn write_addr(&mut self, addr: u8) {
        self.addr = addr & 0x0F;
    }

    pub fn write_data(&mut self, val: u8) {
        let reg = self.addr as usize;
        // Mask certain registers
        let v = match reg {
            0 | 2 | 4 | 11 | 12 => val,
            1 | 3 | 5 => val & 0x0F,
            6 => val & 0x1F,
            7 => val,
            8 | 9 | 10 => val & 0x1F,
            13 => {
                // Envelope shape: reset envelope
                self.env_step = 0;
                self.env_hold = false;
                let attack = val & 0x04 != 0;
                self.env_dir = if attack { 1 } else { -1 };
                self.env_volume = if attack { 0 } else { 15 };
                val & 0x0F
            }
            _ => val,
        };
        self.regs[reg] = v;
    }

    pub fn read_data(&self) -> u8 {
        self.regs[self.addr as usize]
    }

    pub fn port_a(&self) -> u8 {
        self.regs[14]
    }

    /// Advance PSG by `cycles` Z80 CPU cycles.
    /// CPU clock = 3.58 MHz, PSG runs at CPU/16 for tone (each period = period_reg * 8 PSG clocks)
    /// Sample rate = 44100 Hz, CPU cycles per sample = 3579545 / 44100 ≈ 81.17
    pub fn tick(&mut self, cycles: u32) {
        const SAMPLE_RATE: u32 = 44100;
        const CPU_CLOCK: u32 = 3_579_545;
        // We accumulate cycles * SAMPLE_RATE, when >= CPU_CLOCK we emit a sample
        // For tone: period in cycles = tone_period_reg * 16 (8 PSG cycles * 2 CPU/PSG)
        
        for _ in 0..cycles {
            // Tick tones (every 16 CPU cycles)
            // We'll approximate by running per CPU cycle with period_reg * 16
            self.tick_generators(1);
        }

        // Sample generation: accumulate
        self.sample_counter += cycles;
        // cycles per sample ≈ 81
        let cycles_per_sample = CPU_CLOCK / SAMPLE_RATE; // 81
        while self.sample_counter >= cycles_per_sample {
            self.sample_counter -= cycles_per_sample;
            self.samples.push(self.mix_sample());
        }
    }

    fn tone_period(&self, ch: usize) -> u32 {
        let lo = self.regs[ch * 2] as u32;
        let hi = (self.regs[ch * 2 + 1] & 0x0F) as u32;
        let p = (hi << 8) | lo;
        if p == 0 { 1 } else { p }
    }

    fn noise_period(&self) -> u32 {
        let p = (self.regs[6] & 0x1F) as u32;
        if p == 0 { 1 } else { p }
    }

    fn env_period(&self) -> u32 {
        let lo = self.regs[11] as u32;
        let hi = self.regs[12] as u32;
        let p = (hi << 8) | lo;
        if p == 0 { 1 } else { p }
    }

    fn tick_generators(&mut self, cycles: u32) {
        let enable = self.regs[7];
        
        // Tone channels
        for ch in 0..3 {
            let period = self.tone_period(ch) * 16; // period in CPU cycles
            self.tone_counter[ch] += cycles;
            if self.tone_counter[ch] >= period {
                self.tone_counter[ch] -= period;
                self.tone_out[ch] = !self.tone_out[ch];
            }
        }

        // Noise
        let noise_period = self.noise_period() * 16;
        self.noise_counter += cycles;
        if self.noise_counter >= noise_period {
            self.noise_counter -= noise_period;
            // 17-bit LFSR: feedback = bit0 XOR bit3
            let feedback = (self.noise_lfsr ^ (self.noise_lfsr >> 3)) & 1;
            self.noise_lfsr = (self.noise_lfsr >> 1) | (feedback << 16);
            self.noise_out = self.noise_lfsr & 1 != 0;
        }

        // Envelope
        let env_period = self.env_period() * 256;
        self.env_counter += cycles;
        if self.env_counter >= env_period && !self.env_hold {
            self.env_counter -= env_period;
            let shape = self.regs[13];
            let hold = shape & 0x01 != 0;
            let alt = shape & 0x02 != 0;
            let attack = shape & 0x04 != 0;
            let cont = shape & 0x08 != 0;

            if self.env_dir > 0 {
                if self.env_volume < 15 {
                    self.env_volume += 1;
                } else {
                    // end of attack
                    if !cont {
                        self.env_hold = true; self.env_volume = 0;
                    } else if hold {
                        self.env_hold = true;
                    } else if alt {
                        self.env_dir = -1;
                    } else {
                        self.env_volume = 0; // restart
                    }
                }
            } else {
                if self.env_volume > 0 {
                    self.env_volume -= 1;
                } else {
                    if !cont {
                        self.env_hold = true; self.env_volume = 0;
                    } else if hold {
                        self.env_hold = true;
                    } else if alt {
                        self.env_dir = 1;
                    } else {
                        self.env_volume = 15; // restart
                    }
                }
            }
            let _ = attack; // used in initial direction setting on reg write
        }
    }

    fn channel_level(&self, ch: usize) -> f32 {
        let vol_reg = self.regs[8 + ch];
        let use_env = vol_reg & 0x10 != 0;
        let level = if use_env { self.env_volume } else { vol_reg & 0x0F };
        // Convert 0-15 to amplitude (non-linear, approximate dB scale)
        if level == 0 { return 0.0; }
        let db = (level as f32) / 15.0;
        db * db // approximate square law
    }

    fn mix_sample(&self) -> f32 {
        let enable = self.regs[7];
        let mut out = 0.0f32;

        for ch in 0..3 {
            let tone_en  = enable & (1 << ch) == 0;
            let noise_en = enable & (1 << (ch + 3)) == 0;
            let tone_v  = if !tone_en  { 1.0 } else if self.tone_out[ch]  { 1.0 } else { -1.0 };
            let noise_v = if !noise_en { 1.0 } else if self.noise_out      { 1.0 } else { -1.0 };
            // if both disabled → silence; if both enabled → AND
            let active = match (tone_en, noise_en) {
                (false, false) => false,
                (true,  false) => self.tone_out[ch],
                (false, true)  => self.noise_out,
                (true,  true)  => self.tone_out[ch] && self.noise_out,
            };
            let _ = (tone_v, noise_v);
            let amp = if active { self.channel_level(ch) } else { 0.0 };
            out += amp;
        }
        // normalize 3 channels to -1..1
        (out / 3.0).clamp(-1.0, 1.0)
    }

    pub fn take_samples(&mut self) -> Vec<f32> {
        let s = self.samples.clone();
        self.samples.clear();
        s
    }
}
