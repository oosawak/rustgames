#![allow(dead_code)]
use super::z80::BusAccess;
use super::vdp::Vdp;
use super::psg::Psg;
use super::ppi::Ppi;
use super::keyboard::Keyboard;

pub struct Bus {
    pub ram: Box<[u8; 65536]>,
    pub bios: Vec<u8>,
    pub sub_rom: Vec<u8>,
    pub cart: Vec<u8>,
    pub slot_select: u8,
    pub sub_slot_select: [u8; 4],
    pub vdp: Vdp,
    pub psg: Psg,
    pub ppi: Ppi,
    pub keyboard: Keyboard,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Box::new([0u8; 65536]),
            bios: Vec::new(),
            sub_rom: Vec::new(),
            cart: Vec::new(),
            slot_select: 0,
            sub_slot_select: [0; 4],
            vdp: Vdp::new(),
            psg: Psg::new(),
            ppi: Ppi::new(),
            keyboard: Keyboard::new(),
        }
    }

    pub fn load_bios(&mut self, data: &[u8]) {
        self.bios = data.to_vec();
    }

    pub fn load_sub_rom(&mut self, data: &[u8]) {
        self.sub_rom = data.to_vec();
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.cart = data.to_vec();
    }

    fn read_slot(&self, slot: u8, addr: u16) -> u8 {
        match slot {
            0 => {
                // BIOS / sub ROM
                let off = addr as usize;
                if off < self.bios.len() {
                    self.bios[off]
                } else {
                    // sub ROM at 0x4000 - 0x7FFF
                    let sub_off = addr.wrapping_sub(0x4000) as usize;
                    if addr >= 0x4000 && sub_off < self.sub_rom.len() {
                        self.sub_rom[sub_off]
                    } else {
                        0xFF
                    }
                }
            }
            1 => {
                // Cartridge: mapped to $4000-$7FFF (16KB) or $4000-$BFFF (32KB)
                if !self.cart.is_empty() && addr >= 0x4000 {
                    let off = (addr - 0x4000) as usize;
                    if off < self.cart.len() {
                        self.cart[off]
                    } else {
                        0xFF
                    }
                } else {
                    0xFF
                }
            }
            2 => 0xFF,
            3 => self.ram[addr as usize],
            _ => 0xFF,
        }
    }
}

impl BusAccess for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        let page = (addr >> 14) as usize;
        let slot = (self.slot_select >> (page * 2)) & 3;

        // Special: 0xFFFF reads sub-slot register of the selected slot for page 3
        if addr == 0xFFFF && slot == 3 {
            // Return inverted sub-slot select for slot 3
            return !self.sub_slot_select[3];
        }

        self.read_slot(slot, addr)
    }

    fn mem_write(&mut self, addr: u16, val: u8) {
        let page = (addr >> 14) as usize;
        let slot = (self.slot_select >> (page * 2)) & 3;

        if addr == 0xFFFF {
            // Sub-slot select for the slot mapped to page 3
            self.sub_slot_select[slot as usize] = val;
            return;
        }

        if slot == 3 {
            self.ram[addr as usize] = val;
        }
        // ROM slots are read-only
    }

    fn io_read(&mut self, port: u16) -> u8 {
        match (port & 0xFF) as u8 {
            0x98 => self.vdp.read_data(),
            0x99 => self.vdp.read_status(),
            0xA2 => self.psg.read_data(),
            0xA8 => self.slot_select,
            0xA9 => {
                self.ppi.update_keyboard(&self.keyboard);
                self.ppi.port_b
            }
            0xAA => self.ppi.port_c,
            0xAB => self.ppi.control,
            _ => 0xFF,
        }
    }

    fn io_write(&mut self, port: u16, val: u8) {
        match (port & 0xFF) as u8 {
            0x98 => self.vdp.write_data(val),
            0x99 => self.vdp.write_ctrl(val),
            0xA0 => self.psg.write_addr(val),
            0xA1 => self.psg.write_data(val),
            0xA8 => {
                self.slot_select = val;
                self.ppi.port_a = val;
            }
            0xAA => {
                self.ppi.port_c = val;
                self.ppi.update_keyboard(&self.keyboard);
            }
            0xAB => self.ppi.control = val,
            _ => {}
        }
    }
}
