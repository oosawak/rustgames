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
    pub debug_log: Vec<String>,
    last_slot_select: u8,
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
            debug_log: Vec::new(),
            last_slot_select: 0,
        }
    }

    pub fn load_bios(&mut self, data: &[u8]) {
        let mut bios = data.to_vec();
        // BIOS boot: $0DD3=LD B,$78 (120フレームHALT待機) → $01 (1フレームに短縮)
        if bios.len() > 0x0DD4 && bios[0x0DD3] == 0x06 && bios[0x0DD4] == 0x78 {
            bios[0x0DD4] = 0x01;
        }
        self.bios = bios;
    }

    pub fn load_sub_rom(&mut self, data: &[u8]) {
        self.sub_rom = data.to_vec();
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.cart = data.to_vec();
    }

    fn read_slot(&mut self, slot: u8, addr: u16) -> u8 {
        match slot {
            0 => {
                // Sub ROM at 0x4000 - 0x7FFF takes priority over main BIOS
                if addr >= 0x4000 && !self.sub_rom.is_empty() {
                    let sub_off = (addr - 0x4000) as usize;
                    if sub_off < self.sub_rom.len() {
                        return self.sub_rom[sub_off];
                    }
                }
                let off = addr as usize;
                if off < self.bios.len() {
                    self.bios[off]
                } else {
                    0xFF
                }
            }
            1 => {
                if self.cart.is_empty() {
                    if addr >= 0x4000 && addr < 0x5000 {
                        self.add_log(format!("[CART] EMPTY at ${:04X}", addr));
                    }
                    return 0xFF;
                }
                let page = (addr >> 14) as usize;
                let sub_slot = (self.sub_slot_select[1] >> (page * 2)) & 3;
                if sub_slot == 0 && addr >= 0x4000 {
                    let off = (addr - 0x4000) as usize;
                    if off < self.cart.len() {
                        if addr >= 0x4000 && addr < 0x4010 {
                            self.add_log(format!("[CART] READ ${:04X} (off=${:04X}) = ${:02X}", addr, off, self.cart[off]));
                        }
                        return self.cart[off];
                    } else {
                        if addr >= 0x4000 && addr < 0x4010 {
                            self.add_log(format!("[CART] OOB READ ${:04X} (off=${:04X} >= ${:04X})", addr, off, self.cart.len()));
                        }
                    }
                } else {
                    if addr >= 0x4000 && addr < 0x4010 {
                        self.add_log(format!("[CART] SUBSLOT MISMATCH ${:04X} page={} sub_slot={}", addr, page, sub_slot));
                    }
                }
                0xFF
            }
            2 => 0xFF,
            3 => self.ram[addr as usize],
            _ => 0xFF,
        }
    }

    /// Check if a main slot has sub-slots (expandable)
    fn slot_has_sub_slots(&self, slot: u8) -> bool {
        match slot {
            1 => !self.cart.is_empty(),
            _ => false,
        }
    }

    pub fn add_log(&mut self, msg: String) {
        if self.debug_log.len() >= 1000 {
            self.debug_log.remove(0);
        }
        self.debug_log.push(msg);
    }
}

impl BusAccess for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        let page = (addr >> 14) as usize;
        let slot = (self.slot_select >> (page * 2)) & 3;

        // Log slot select changes
        if slot == 1 && addr >= 0x4000 && addr < 0x5000 && self.slot_select != self.last_slot_select {
            self.add_log(format!("[READ] addr=${:04X} page={} slot={} slot_select=${:02X}", addr, page, slot, self.slot_select));
            self.last_slot_select = self.slot_select;
        }

        // $FFFF: sub-slot select register for the slot mapped to page 3
        if addr == 0xFFFF && self.slot_has_sub_slots(slot) {
            return !self.sub_slot_select[slot as usize];
        }

        self.read_slot(slot, addr)
    }

    fn mem_write(&mut self, addr: u16, val: u8) {
        let page = (addr >> 14) as usize;
        let slot = (self.slot_select >> (page * 2)) & 3;

        // $FFFF: write sub-slot select register
        if addr == 0xFFFF {
            self.sub_slot_select[slot as usize] = val;
            return;
        }

        // Only RAM (slot 3) is writable
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
                if val != self.slot_select {
                    self.add_log(format!("[SLOT] $A8 write: ${:02X} → ${:02X}", self.slot_select, val));
                }
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
