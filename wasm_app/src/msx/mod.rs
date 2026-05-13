#![allow(dead_code)]

pub mod z80;
pub mod memory;
pub mod vdp;
pub mod psg;
pub mod ppi;
pub mod keyboard;

use z80::Z80;
use memory::Bus;

pub struct MsxState {
    pub cpu: Z80,
    pub bus: Bus,
    pub frame_buffer: Vec<u8>,
    pub audio_buffer: Vec<f32>,
}

impl MsxState {
    pub fn new() -> Self {
        let mut state = MsxState {
            cpu: Z80::new(),
            bus: Bus::new(),
            frame_buffer: vec![0u8; 256 * 192 * 4],
            audio_buffer: Vec::new(),
        };
        state.cpu.reset();
        state
    }

    pub fn tick_frame(&mut self) {
        const CYCLES_PER_FRAME: u32 = 59667;
        let mut cycles_run: u32 = 0;

        // VBLANK status at start of frame
        self.bus.vdp.int_pending = true;
        self.bus.vdp.status |= 0x80;  // VBLANK flag (bit 7)

        while cycles_run < CYCLES_PER_FRAME {
            let c = self.cpu.step(&mut self.bus);
            cycles_run += c;
            self.bus.psg.tick(c);
            // Update keyboard state in PPI
            self.bus.ppi.update_keyboard(&self.bus.keyboard);
        }

        // Trigger VBL interrupt at END of frame (real MSX timing: VBL occurs at frame end)
        // This ensures HALT (which waits for INT) is properly woken up
        if self.bus.vdp.take_interrupt() {
            let data = match self.cpu.im {
                2 => 0xFF,
                _ => 0xFF,
            };
            self.cpu.int(&mut self.bus, data);
        }

        self.bus.vdp.render_frame(&mut self.frame_buffer);
        self.audio_buffer = self.bus.psg.take_samples();

        // Update JIFFY counter at $FC9E in RAM (MSX BIOS work area)
        let jiffy = self.bus.ram[0xFC9E].wrapping_add(1);
        self.bus.ram[0xFC9E] = jiffy;
    }

    pub fn frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    pub fn audio_samples(&mut self) -> Vec<f32> {
        let s = self.audio_buffer.clone();
        self.audio_buffer.clear();
        s
    }
}
