#![allow(dead_code)]
use super::keyboard::Keyboard;

pub struct Ppi {
    pub port_a: u8,
    pub port_b: u8,
    pub port_c: u8,
    pub control: u8,
}

impl Ppi {
    pub fn new() -> Self {
        Ppi { port_a: 0, port_b: 0xFF, port_c: 0, control: 0x9B }
    }

    pub fn read(&self, port: u8) -> u8 {
        match port {
            0xA8 => self.port_a,
            0xA9 => self.port_b,
            0xAA => self.port_c,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, port: u8, val: u8, keyboard: &Keyboard) {
        match port {
            0xA8 => self.port_a = val,
            0xAA => {
                self.port_c = val;
                self.update_keyboard(keyboard);
            }
            0xAB => self.control = val,
            _ => {}
        }
    }

    pub fn update_keyboard(&mut self, keyboard: &Keyboard) {
        let col = (self.port_c & 0x0F) as usize;
        self.port_b = keyboard.read_row(col);
    }
}
