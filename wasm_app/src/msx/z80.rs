#![allow(dead_code, non_snake_case)]

pub const FLAG_S:  u8 = 0x80;
pub const FLAG_Z:  u8 = 0x40;
pub const FLAG_Y:  u8 = 0x20;
pub const FLAG_H:  u8 = 0x10;
pub const FLAG_X:  u8 = 0x08;
pub const FLAG_PV: u8 = 0x04;
pub const FLAG_N:  u8 = 0x02;
pub const FLAG_C:  u8 = 0x01;

pub trait BusAccess {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, val: u8);
    fn io_read(&mut self, port: u16) -> u8;
    fn io_write(&mut self, port: u16, val: u8);
}

pub struct Z80 {
    pub a: u8, pub f: u8,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,
    pub a2: u8, pub f2: u8,
    pub b2: u8, pub c2: u8,
    pub d2: u8, pub e2: u8,
    pub h2: u8, pub l2: u8,
    pub ix: u16, pub iy: u16,
    pub sp: u16, pub pc: u16,
    pub i: u8, pub r: u8,
    pub iff1: bool, pub iff2: bool,
    pub im: u8,
    pub halted: bool,
    pub cycles: u64,
}

impl Z80 {
    pub fn new() -> Self {
        Z80 {
            a: 0, f: 0,
            b: 0, c: 0,
            d: 0, e: 0,
            h: 0, l: 0,
            a2: 0, f2: 0,
            b2: 0, c2: 0,
            d2: 0, e2: 0,
            h2: 0, l2: 0,
            ix: 0xFFFF, iy: 0xFFFF,
            sp: 0xFFFF, pc: 0,
            i: 0, r: 0,
            iff1: false, iff2: false,
            im: 1,
            halted: false,
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0;
        self.sp = 0xFFFF;
        self.iff1 = false;
        self.iff2 = false;
        self.im = 1;
        self.halted = false;
        self.a = 0xFF;
        self.f = 0xFF;
    }

    // ── Register pair helpers ─────────────────────────────────────────────
    #[inline] pub fn bc(&self) -> u16 { ((self.b as u16) << 8) | self.c as u16 }
    #[inline] pub fn de(&self) -> u16 { ((self.d as u16) << 8) | self.e as u16 }
    #[inline] pub fn hl(&self) -> u16 { ((self.h as u16) << 8) | self.l as u16 }
    #[inline] pub fn af(&self) -> u16 { ((self.a as u16) << 8) | self.f as u16 }
    #[inline] pub fn set_bc(&mut self, v: u16) { self.b=(v>>8) as u8; self.c=v as u8; }
    #[inline] pub fn set_de(&mut self, v: u16) { self.d=(v>>8) as u8; self.e=v as u8; }
    #[inline] pub fn set_hl(&mut self, v: u16) { self.h=(v>>8) as u8; self.l=v as u8; }
    #[inline] pub fn set_af(&mut self, v: u16) { self.a=(v>>8) as u8; self.f=v as u8; }

    // ── Flag helpers ──────────────────────────────────────────────────────
    #[inline] fn sf(&self) -> bool  { self.f & FLAG_S  != 0 }
    #[inline] fn zf(&self) -> bool  { self.f & FLAG_Z  != 0 }
    #[inline] fn hf(&self) -> bool  { self.f & FLAG_H  != 0 }
    #[inline] fn pvf(&self) -> bool { self.f & FLAG_PV != 0 }
    #[inline] fn nf(&self) -> bool  { self.f & FLAG_N  != 0 }
    #[inline] fn cf(&self) -> bool  { self.f & FLAG_C  != 0 }

    #[inline]
    fn set_sz(&mut self, v: u8) {
        self.f = (self.f & !(FLAG_S | FLAG_Z | FLAG_Y | FLAG_X))
               | (v & FLAG_S)
               | (if v == 0 { FLAG_Z } else { 0 })
               | (v & FLAG_Y)
               | (v & FLAG_X);
    }

    // ── Memory helpers ────────────────────────────────────────────────────
    #[inline]
    fn fetch(&mut self, bus: &mut impl BusAccess) -> u8 {
        let v = bus.mem_read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        self.r = (self.r & 0x80) | ((self.r.wrapping_add(1)) & 0x7F);
        v
    }

    #[inline]
    fn fetch16(&mut self, bus: &mut impl BusAccess) -> u16 {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus) as u16;
        (hi << 8) | lo
    }

    #[inline]
    fn read16(&mut self, bus: &mut impl BusAccess, addr: u16) -> u16 {
        let lo = bus.mem_read(addr) as u16;
        let hi = bus.mem_read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    #[inline]
    fn write16(&mut self, bus: &mut impl BusAccess, addr: u16, val: u16) {
        bus.mem_write(addr, val as u8);
        bus.mem_write(addr.wrapping_add(1), (val >> 8) as u8);
    }

    // ── Stack ─────────────────────────────────────────────────────────────
    #[inline]
    fn push16(&mut self, bus: &mut impl BusAccess, val: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.write16(bus, self.sp, val);
    }

    #[inline]
    fn pop16(&mut self, bus: &mut impl BusAccess) -> u16 {
        let v = self.read16(bus, self.sp);
        self.sp = self.sp.wrapping_add(2);
        v
    }

    // ── ALU operations ────────────────────────────────────────────────────
    fn add8(&mut self, val: u8, carry: bool) {
        let c = carry as u8;
        let result = (self.a as u16) + (val as u16) + (c as u16);
        let hresult = (self.a & 0x0F) + (val & 0x0F) + c;
        let overflow = (!(self.a ^ val) & (self.a ^ result as u8)) & 0x80 != 0;
        self.f = 0;
        if result as u8 == 0 { self.f |= FLAG_Z; }
        if result as u8 & 0x80 != 0 { self.f |= FLAG_S; }
        if result > 0xFF { self.f |= FLAG_C; }
        if hresult > 0x0F { self.f |= FLAG_H; }
        if overflow { self.f |= FLAG_PV; }
        self.f |= result as u8 & (FLAG_Y | FLAG_X);
        self.a = result as u8;
    }

    fn sub8(&mut self, val: u8, carry: bool) {
        let c = carry as u8;
        let result = (self.a as i16) - (val as i16) - (c as i16);
        let hresult = (self.a & 0x0F) as i8 - (val & 0x0F) as i8 - c as i8;
        let overflow = ((self.a ^ val) & (self.a ^ result as u8)) & 0x80 != 0;
        self.f = FLAG_N;
        if result as u8 == 0 { self.f |= FLAG_Z; }
        if result as u8 & 0x80 != 0 { self.f |= FLAG_S; }
        if result < 0 { self.f |= FLAG_C; }
        if hresult < 0 { self.f |= FLAG_H; }
        if overflow { self.f |= FLAG_PV; }
        self.f |= result as u8 & (FLAG_Y | FLAG_X);
        self.a = result as u8;
    }

    fn and8(&mut self, val: u8) {
        self.a &= val;
        self.f = FLAG_H | parity(self.a);
        self.set_sz(self.a);
    }

    fn or8(&mut self, val: u8) {
        self.a |= val;
        self.f = parity(self.a);
        self.set_sz(self.a);
    }

    fn xor8(&mut self, val: u8) {
        self.a ^= val;
        self.f = parity(self.a);
        self.set_sz(self.a);
    }

    fn cp8(&mut self, val: u8) {
        let old_a = self.a;
        self.sub8(val, false);
        // CP uses val for Y/X flags (not result)
        self.f = (self.f & !(FLAG_Y | FLAG_X)) | (val & (FLAG_Y | FLAG_X));
        self.a = old_a;
    }

    fn inc8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_add(1);
        let overflow = v == 0x7F;
        self.f = (self.f & FLAG_C)
               | (if r == 0 { FLAG_Z } else { 0 })
               | (r & FLAG_S)
               | (if (v & 0x0F) == 0x0F { FLAG_H } else { 0 })
               | (if overflow { FLAG_PV } else { 0 })
               | (r & (FLAG_Y | FLAG_X));
        r
    }

    fn dec8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_sub(1);
        let overflow = v == 0x80;
        self.f = (self.f & FLAG_C)
               | FLAG_N
               | (if r == 0 { FLAG_Z } else { 0 })
               | (r & FLAG_S)
               | (if (v & 0x0F) == 0x00 { FLAG_H } else { 0 })
               | (if overflow { FLAG_PV } else { 0 })
               | (r & (FLAG_Y | FLAG_X));
        r
    }

    fn add16(&mut self, hl: u16, rr: u16) -> u16 {
        let result = (hl as u32) + (rr as u32);
        let hresult = (hl & 0x0FFF) + (rr & 0x0FFF);
        self.f = (self.f & (FLAG_S | FLAG_Z | FLAG_PV))
               | (if result > 0xFFFF { FLAG_C } else { 0 })
               | (if hresult > 0x0FFF { FLAG_H } else { 0 })
               | ((result >> 8) as u8 & (FLAG_Y | FLAG_X));
        result as u16
    }

    fn adc16(&mut self, hl: u16, rr: u16) -> u16 {
        let c = self.cf() as u32;
        let result = (hl as u32) + (rr as u32) + c;
        let hresult = (hl & 0x0FFF) as u32 + (rr & 0x0FFF) as u32 + c;
        let overflow = (!(hl ^ rr) & (hl ^ result as u16)) & 0x8000 != 0;
        let r16 = result as u16;
        self.f = (if r16 == 0 { FLAG_Z } else { 0 })
               | (if r16 & 0x8000 != 0 { FLAG_S } else { 0 })
               | (if result > 0xFFFF { FLAG_C } else { 0 })
               | (if hresult > 0x0FFF { FLAG_H } else { 0 })
               | (if overflow { FLAG_PV } else { 0 })
               | ((result >> 8) as u8 & (FLAG_Y | FLAG_X));
        r16
    }

    fn sbc16(&mut self, hl: u16, rr: u16) -> u16 {
        let c = self.cf() as i32;
        let result = (hl as i32) - (rr as i32) - c;
        let hresult = (hl & 0x0FFF) as i32 - (rr & 0x0FFF) as i32 - c;
        let overflow = ((hl ^ rr) & (hl ^ result as u16)) & 0x8000 != 0;
        let r16 = result as u16;
        self.f = FLAG_N
               | (if r16 == 0 { FLAG_Z } else { 0 })
               | (if r16 & 0x8000 != 0 { FLAG_S } else { 0 })
               | (if result < 0 { FLAG_C } else { 0 })
               | (if hresult < 0 { FLAG_H } else { 0 })
               | (if overflow { FLAG_PV } else { 0 })
               | ((result >> 8) as u8 & (FLAG_Y | FLAG_X));
        r16
    }

    fn rlca(&mut self) {
        let c = self.a >> 7;
        self.a = (self.a << 1) | c;
        self.f = (self.f & (FLAG_S | FLAG_Z | FLAG_PV))
               | (c != 0) as u8
               | (self.a & (FLAG_Y | FLAG_X));
    }

    fn rrca(&mut self) {
        let c = self.a & 1;
        self.a = (self.a >> 1) | (c << 7);
        self.f = (self.f & (FLAG_S | FLAG_Z | FLAG_PV))
               | c
               | (self.a & (FLAG_Y | FLAG_X));
    }

    fn rla(&mut self) {
        let old_c = self.cf() as u8;
        let new_c = self.a >> 7;
        self.a = (self.a << 1) | old_c;
        self.f = (self.f & (FLAG_S | FLAG_Z | FLAG_PV))
               | new_c
               | (self.a & (FLAG_Y | FLAG_X));
    }

    fn rra(&mut self) {
        let old_c = self.cf() as u8;
        let new_c = self.a & 1;
        self.a = (self.a >> 1) | (old_c << 7);
        self.f = (self.f & (FLAG_S | FLAG_Z | FLAG_PV))
               | new_c
               | (self.a & (FLAG_Y | FLAG_X));
    }

    fn daa(&mut self) {
        let mut a = self.a;
        let cf = self.cf();
        let hf = self.hf();
        let nf = self.nf();
        let mut correction: u8 = 0;
        let mut new_cf = false;
        if !nf {
            if hf || (a & 0x0F) > 9 { correction |= 0x06; }
            if cf || a > 0x99 { correction |= 0x60; new_cf = true; }
            a = a.wrapping_add(correction);
        } else {
            if hf || (a & 0x0F) > 9 { correction |= 0x06; }
            if cf || a > 0x99 { correction |= 0x60; new_cf = true; }
            a = a.wrapping_sub(correction);
        }
        let new_h = if nf { hf && (self.a & 0x0F) < 6 } else { (self.a & 0x0F) > 9 };
        self.f = (if new_cf { FLAG_C } else { 0 })
               | (self.f & FLAG_N)
               | (if new_h { FLAG_H } else { 0 })
               | parity(a)
               | (if a == 0 { FLAG_Z } else { 0 })
               | (a & FLAG_S)
               | (a & (FLAG_Y | FLAG_X));
        self.a = a;
    }

    fn neg(&mut self) {
        let v = self.a;
        self.a = 0;
        self.sub8(v, false);
    }

    // ── CB prefix rotate/shift ────────────────────────────────────────────
    fn rlc(&mut self, v: u8) -> u8 {
        let c = v >> 7;
        let r = (v << 1) | c;
        self.f = (c != 0) as u8 | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn rrc(&mut self, v: u8) -> u8 {
        let c = v & 1;
        let r = (v >> 1) | (c << 7);
        self.f = c | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn rl(&mut self, v: u8) -> u8 {
        let old_c = self.cf() as u8;
        let new_c = v >> 7;
        let r = (v << 1) | old_c;
        self.f = new_c | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn rr(&mut self, v: u8) -> u8 {
        let old_c = self.cf() as u8;
        let new_c = v & 1;
        let r = (v >> 1) | (old_c << 7);
        self.f = new_c | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn sla(&mut self, v: u8) -> u8 {
        let c = v >> 7;
        let r = v << 1;
        self.f = c | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn sra(&mut self, v: u8) -> u8 {
        let c = v & 1;
        let r = (v >> 1) | (v & 0x80);
        self.f = c | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn sll(&mut self, v: u8) -> u8 {
        let c = v >> 7;
        let r = (v << 1) | 1;
        self.f = c | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn srl(&mut self, v: u8) -> u8 {
        let c = v & 1;
        let r = v >> 1;
        self.f = c | (if r == 0 { FLAG_Z } else { 0 }) | (r & FLAG_S) | parity(r) | (r & (FLAG_Y | FLAG_X));
        r
    }
    fn bit(&mut self, b: u8, v: u8) {
        let r = v & (1 << b);
        self.f = FLAG_H
               | (self.f & FLAG_C)
               | (if r == 0 { FLAG_Z | FLAG_PV } else { 0 })
               | (r & FLAG_S)
               | (v & (FLAG_Y | FLAG_X));
    }

    // ── Interrupt handling ─────────────────────────────────────────────────
    pub fn nmi(&mut self, bus: &mut impl BusAccess) {
        self.halted = false;
        self.iff2 = self.iff1;
        self.iff1 = false;
        let pc = self.pc;
        self.push16(bus, pc);
        self.pc = 0x0066;
        self.cycles += 11;
    }

    pub fn int(&mut self, bus: &mut impl BusAccess, data: u8) {
        if !self.iff1 { return; }
        self.halted = false;
        self.iff1 = false;
        self.iff2 = false;
        match self.im {
            0 => {
                // Execute data byte as instruction (usually RST or FF)
                self.cycles += 13;
                let pc = self.pc;
                self.push16(bus, pc);
                self.pc = (data as u16) & 0x38;
            }
            1 => {
                let pc = self.pc;
                self.push16(bus, pc);
                self.pc = 0x0038;
                self.cycles += 13;
            }
            2 => {
                let vec_addr = ((self.i as u16) << 8) | (data as u16 & 0xFE);
                let target = self.read16(bus, vec_addr);
                let pc = self.pc;
                self.push16(bus, pc);
                self.pc = target;
                self.cycles += 19;
            }
            _ => {}
        }
    }

    // ── Register access helpers ───────────────────────────────────────────
    fn get_reg(&self, bus: &impl BusAccess, r: u8) -> u8 {
        let _ = bus;
        match r {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            7 => self.a,
            _ => 0,
        }
    }

    fn get_reg_mut(&mut self, r: u8) -> *mut u8 {
        match r {
            0 => &mut self.b,
            1 => &mut self.c,
            2 => &mut self.d,
            3 => &mut self.e,
            4 => &mut self.h,
            5 => &mut self.l,
            7 => &mut self.a,
            _ => &mut self.a, // fallback (shouldn't happen for 6)
        }
    }

    fn set_reg(&mut self, r: u8, v: u8) {
        match r {
            0 => self.b = v,
            1 => self.c = v,
            2 => self.d = v,
            3 => self.e = v,
            4 => self.h = v,
            5 => self.l = v,
            7 => self.a = v,
            _ => {}
        }
    }

    fn get_rr(&self, rr: u8) -> u16 {
        match rr {
            0 => self.bc(),
            1 => self.de(),
            2 => self.hl(),
            3 => self.sp,
            _ => 0,
        }
    }

    fn set_rr(&mut self, rr: u8, v: u16) {
        match rr {
            0 => self.set_bc(v),
            1 => self.set_de(v),
            2 => self.set_hl(v),
            3 => self.sp = v,
            _ => {}
        }
    }

    // ── Main execution ─────────────────────────────────────────────────────
    pub fn step(&mut self, bus: &mut impl BusAccess) -> u32 {
        if self.halted {
            self.cycles += 4;
            return 4;
        }

        let start = self.cycles;
        let op = self.fetch(bus);

        match op {
            // NOP
            0x00 => { self.cycles += 4; }
            // LD BC,nn
            0x01 => { let v = self.fetch16(bus); self.set_bc(v); self.cycles += 10; }
            // LD (BC),A
            0x02 => { bus.mem_write(self.bc(), self.a); self.cycles += 7; }
            // INC BC
            0x03 => { let v = self.bc().wrapping_add(1); self.set_bc(v); self.cycles += 6; }
            // INC B
            0x04 => { self.b = self.inc8(self.b); self.cycles += 4; }
            // DEC B
            0x05 => { self.b = self.dec8(self.b); self.cycles += 4; }
            // LD B,n
            0x06 => { self.b = self.fetch(bus); self.cycles += 7; }
            // RLCA
            0x07 => { self.rlca(); self.cycles += 4; }
            // EX AF,AF'
            0x08 => {
                core::mem::swap(&mut self.a, &mut self.a2);
                core::mem::swap(&mut self.f, &mut self.f2);
                self.cycles += 4;
            }
            // ADD HL,BC
            0x09 => { let hl = self.hl(); let bc = self.bc(); let r = self.add16(hl, bc); self.set_hl(r); self.cycles += 11; }
            // LD A,(BC)
            0x0A => { self.a = bus.mem_read(self.bc()); self.cycles += 7; }
            // DEC BC
            0x0B => { let v = self.bc().wrapping_sub(1); self.set_bc(v); self.cycles += 6; }
            // INC C
            0x0C => { self.c = self.inc8(self.c); self.cycles += 4; }
            // DEC C
            0x0D => { self.c = self.dec8(self.c); self.cycles += 4; }
            // LD C,n
            0x0E => { self.c = self.fetch(bus); self.cycles += 7; }
            // RRCA
            0x0F => { self.rrca(); self.cycles += 4; }
            // DJNZ d
            0x10 => {
                let d = self.fetch(bus) as i8;
                self.b = self.b.wrapping_sub(1);
                if self.b != 0 {
                    self.pc = self.pc.wrapping_add(d as u16);
                    self.cycles += 13;
                } else {
                    self.cycles += 8;
                }
            }
            // LD DE,nn
            0x11 => { let v = self.fetch16(bus); self.set_de(v); self.cycles += 10; }
            // LD (DE),A
            0x12 => { bus.mem_write(self.de(), self.a); self.cycles += 7; }
            // INC DE
            0x13 => { let v = self.de().wrapping_add(1); self.set_de(v); self.cycles += 6; }
            // INC D
            0x14 => { self.d = self.inc8(self.d); self.cycles += 4; }
            // DEC D
            0x15 => { self.d = self.dec8(self.d); self.cycles += 4; }
            // LD D,n
            0x16 => { self.d = self.fetch(bus); self.cycles += 7; }
            // RLA
            0x17 => { self.rla(); self.cycles += 4; }
            // JR d
            0x18 => { let d = self.fetch(bus) as i8; self.pc = self.pc.wrapping_add(d as u16); self.cycles += 12; }
            // ADD HL,DE
            0x19 => { let hl = self.hl(); let de = self.de(); let r = self.add16(hl, de); self.set_hl(r); self.cycles += 11; }
            // LD A,(DE)
            0x1A => { self.a = bus.mem_read(self.de()); self.cycles += 7; }
            // DEC DE
            0x1B => { let v = self.de().wrapping_sub(1); self.set_de(v); self.cycles += 6; }
            // INC E
            0x1C => { self.e = self.inc8(self.e); self.cycles += 4; }
            // DEC E
            0x1D => { self.e = self.dec8(self.e); self.cycles += 4; }
            // LD E,n
            0x1E => { self.e = self.fetch(bus); self.cycles += 7; }
            // RRA
            0x1F => { self.rra(); self.cycles += 4; }
            // JR NZ,d
            0x20 => { let d = self.fetch(bus) as i8; if !self.zf() { self.pc = self.pc.wrapping_add(d as u16); self.cycles += 12; } else { self.cycles += 7; } }
            // LD HL,nn
            0x21 => { let v = self.fetch16(bus); self.set_hl(v); self.cycles += 10; }
            // LD (nn),HL
            0x22 => { let addr = self.fetch16(bus); let hl = self.hl(); self.write16(bus, addr, hl); self.cycles += 16; }
            // INC HL
            0x23 => { let v = self.hl().wrapping_add(1); self.set_hl(v); self.cycles += 6; }
            // INC H
            0x24 => { self.h = self.inc8(self.h); self.cycles += 4; }
            // DEC H
            0x25 => { self.h = self.dec8(self.h); self.cycles += 4; }
            // LD H,n
            0x26 => { self.h = self.fetch(bus); self.cycles += 7; }
            // DAA
            0x27 => { self.daa(); self.cycles += 4; }
            // JR Z,d
            0x28 => { let d = self.fetch(bus) as i8; if  self.zf() { self.pc = self.pc.wrapping_add(d as u16); self.cycles += 12; } else { self.cycles += 7; } }
            // ADD HL,HL
            0x29 => { let hl = self.hl(); let r = self.add16(hl, hl); self.set_hl(r); self.cycles += 11; }
            // LD HL,(nn)
            0x2A => { let addr = self.fetch16(bus); let v = self.read16(bus, addr); self.set_hl(v); self.cycles += 16; }
            // DEC HL
            0x2B => { let v = self.hl().wrapping_sub(1); self.set_hl(v); self.cycles += 6; }
            // INC L
            0x2C => { self.l = self.inc8(self.l); self.cycles += 4; }
            // DEC L
            0x2D => { self.l = self.dec8(self.l); self.cycles += 4; }
            // LD L,n
            0x2E => { self.l = self.fetch(bus); self.cycles += 7; }
            // CPL
            0x2F => { self.a = !self.a; self.f = (self.f & (FLAG_S|FLAG_Z|FLAG_PV|FLAG_C)) | FLAG_H | FLAG_N | (self.a & (FLAG_Y|FLAG_X)); self.cycles += 4; }
            // JR NC,d
            0x30 => { let d = self.fetch(bus) as i8; if !self.cf() { self.pc = self.pc.wrapping_add(d as u16); self.cycles += 12; } else { self.cycles += 7; } }
            // LD SP,nn
            0x31 => { self.sp = self.fetch16(bus); self.cycles += 10; }
            // LD (nn),A
            0x32 => { let addr = self.fetch16(bus); bus.mem_write(addr, self.a); self.cycles += 13; }
            // INC SP
            0x33 => { self.sp = self.sp.wrapping_add(1); self.cycles += 6; }
            // INC (HL)
            0x34 => { let hl = self.hl(); let v = bus.mem_read(hl); let r = self.inc8(v); bus.mem_write(hl, r); self.cycles += 11; }
            // DEC (HL)
            0x35 => { let hl = self.hl(); let v = bus.mem_read(hl); let r = self.dec8(v); bus.mem_write(hl, r); self.cycles += 11; }
            // LD (HL),n
            0x36 => { let n = self.fetch(bus); let hl = self.hl(); bus.mem_write(hl, n); self.cycles += 10; }
            // SCF
            0x37 => { self.f = (self.f & (FLAG_S|FLAG_Z|FLAG_PV)) | FLAG_C | (self.a & (FLAG_Y|FLAG_X)); self.cycles += 4; }
            // JR C,d
            0x38 => { let d = self.fetch(bus) as i8; if  self.cf() { self.pc = self.pc.wrapping_add(d as u16); self.cycles += 12; } else { self.cycles += 7; } }
            // ADD HL,SP
            0x39 => { let hl = self.hl(); let sp = self.sp; let r = self.add16(hl, sp); self.set_hl(r); self.cycles += 11; }
            // LD A,(nn)
            0x3A => { let addr = self.fetch16(bus); self.a = bus.mem_read(addr); self.cycles += 13; }
            // DEC SP
            0x3B => { self.sp = self.sp.wrapping_sub(1); self.cycles += 6; }
            // INC A
            0x3C => { self.a = self.inc8(self.a); self.cycles += 4; }
            // DEC A
            0x3D => { self.a = self.dec8(self.a); self.cycles += 4; }
            // LD A,n
            0x3E => { self.a = self.fetch(bus); self.cycles += 7; }
            // CCF
            0x3F => {
                let old_c = self.cf() as u8;
                self.f = (self.f & (FLAG_S|FLAG_Z|FLAG_PV))
                       | (if old_c != 0 { FLAG_H } else { 0 })
                       | (if old_c != 0 { 0 } else { FLAG_C })
                       | (self.a & (FLAG_Y|FLAG_X));
                self.cycles += 4;
            }
            // LD r,r' / LD r,(HL) / LD (HL),r / HALT
            0x40..=0x7F => {
                let dst = (op >> 3) & 7;
                let src = op & 7;
                if op == 0x76 {
                    // HALT
                    self.halted = true;
                    self.pc = self.pc.wrapping_sub(1);
                    self.cycles += 4;
                } else if src == 6 {
                    // LD r,(HL)
                    let v = bus.mem_read(self.hl());
                    self.set_reg(dst, v);
                    self.cycles += 7;
                } else if dst == 6 {
                    // LD (HL),r
                    let v = self.get_reg(bus, src);
                    bus.mem_write(self.hl(), v);
                    self.cycles += 7;
                } else {
                    let v = self.get_reg(bus, src);
                    self.set_reg(dst, v);
                    self.cycles += 4;
                }
            }
            // ADD A,r / ADD A,(HL) / ADD A,n
            0x80..=0x87 => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.add8(v, false);
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // ADC A,r
            0x88..=0x8F => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.add8(v, self.cf());
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // SUB r
            0x90..=0x97 => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.sub8(v, false);
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // SBC A,r
            0x98..=0x9F => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.sub8(v, self.cf());
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // AND r
            0xA0..=0xA7 => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.and8(v);
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // XOR r
            0xA8..=0xAF => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.xor8(v);
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // OR r
            0xB0..=0xB7 => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.or8(v);
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // CP r
            0xB8..=0xBF => {
                let src = op & 7;
                let v = if src == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, src) };
                self.cp8(v);
                self.cycles += if src == 6 { 7 } else { 4 };
            }
            // RET NZ
            0xC0 => { if !self.zf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // POP BC
            0xC1 => { let v = self.pop16(bus); self.set_bc(v); self.cycles += 10; }
            // JP NZ,nn
            0xC2 => { let addr = self.fetch16(bus); if !self.zf() { self.pc = addr; } self.cycles += 10; }
            // JP nn
            0xC3 => { self.pc = self.fetch16(bus); self.cycles += 10; }
            // CALL NZ,nn
            0xC4 => { let addr = self.fetch16(bus); if !self.zf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // PUSH BC
            0xC5 => { let v = self.bc(); self.push16(bus, v); self.cycles += 11; }
            // ADD A,n
            0xC6 => { let n = self.fetch(bus); self.add8(n, false); self.cycles += 7; }
            // RST 0x00
            0xC7 => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0000; self.cycles += 11; }
            // RET Z
            0xC8 => { if  self.zf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // RET
            0xC9 => { self.pc = self.pop16(bus); self.cycles += 10; }
            // JP Z,nn
            0xCA => { let addr = self.fetch16(bus); if  self.zf() { self.pc = addr; } self.cycles += 10; }
            // CB prefix
            0xCB => { self.exec_cb(bus); }
            // CALL Z,nn
            0xCC => { let addr = self.fetch16(bus); if  self.zf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // CALL nn
            0xCD => { let addr = self.fetch16(bus); let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; }
            // ADC A,n
            0xCE => { let n = self.fetch(bus); let c = self.cf(); self.add8(n, c); self.cycles += 7; }
            // RST 0x08
            0xCF => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0008; self.cycles += 11; }
            // RET NC
            0xD0 => { if !self.cf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // POP DE
            0xD1 => { let v = self.pop16(bus); self.set_de(v); self.cycles += 10; }
            // JP NC,nn
            0xD2 => { let addr = self.fetch16(bus); if !self.cf() { self.pc = addr; } self.cycles += 10; }
            // OUT (n),A
            0xD3 => { let n = self.fetch(bus); bus.io_write(((self.a as u16) << 8) | n as u16, self.a); self.cycles += 11; }
            // CALL NC,nn
            0xD4 => { let addr = self.fetch16(bus); if !self.cf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // PUSH DE
            0xD5 => { let v = self.de(); self.push16(bus, v); self.cycles += 11; }
            // SUB n
            0xD6 => { let n = self.fetch(bus); self.sub8(n, false); self.cycles += 7; }
            // RST 0x10
            0xD7 => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0010; self.cycles += 11; }
            // RET C
            0xD8 => { if  self.cf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // EXX
            0xD9 => {
                core::mem::swap(&mut self.b, &mut self.b2);
                core::mem::swap(&mut self.c, &mut self.c2);
                core::mem::swap(&mut self.d, &mut self.d2);
                core::mem::swap(&mut self.e, &mut self.e2);
                core::mem::swap(&mut self.h, &mut self.h2);
                core::mem::swap(&mut self.l, &mut self.l2);
                self.cycles += 4;
            }
            // JP C,nn
            0xDA => { let addr = self.fetch16(bus); if  self.cf() { self.pc = addr; } self.cycles += 10; }
            // IN A,(n)
            0xDB => { let n = self.fetch(bus); self.a = bus.io_read(((self.a as u16) << 8) | n as u16); self.cycles += 11; }
            // CALL C,nn
            0xDC => { let addr = self.fetch16(bus); if  self.cf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // DD prefix: IX
            0xDD => { self.exec_dd(bus); }
            // SBC A,n
            0xDE => { let n = self.fetch(bus); let c = self.cf(); self.sub8(n, c); self.cycles += 7; }
            // RST 0x18
            0xDF => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0018; self.cycles += 11; }
            // RET PO
            0xE0 => { if !self.pvf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // POP HL
            0xE1 => { let v = self.pop16(bus); self.set_hl(v); self.cycles += 10; }
            // JP PO,nn
            0xE2 => { let addr = self.fetch16(bus); if !self.pvf() { self.pc = addr; } self.cycles += 10; }
            // EX (SP),HL
            0xE3 => {
                let sp = self.sp;
                let mem_val = self.read16(bus, sp);
                let hl = self.hl();
                self.write16(bus, sp, hl);
                self.set_hl(mem_val);
                self.cycles += 19;
            }
            // CALL PO,nn
            0xE4 => { let addr = self.fetch16(bus); if !self.pvf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // PUSH HL
            0xE5 => { let v = self.hl(); self.push16(bus, v); self.cycles += 11; }
            // AND n
            0xE6 => { let n = self.fetch(bus); self.and8(n); self.cycles += 7; }
            // RST 0x20
            0xE7 => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0020; self.cycles += 11; }
            // RET PE
            0xE8 => { if  self.pvf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // JP (HL)
            0xE9 => { self.pc = self.hl(); self.cycles += 4; }
            // JP PE,nn
            0xEA => { let addr = self.fetch16(bus); if  self.pvf() { self.pc = addr; } self.cycles += 10; }
            // EX DE,HL
            0xEB => {
                core::mem::swap(&mut self.d, &mut self.h);
                core::mem::swap(&mut self.e, &mut self.l);
                self.cycles += 4;
            }
            // CALL PE,nn
            0xEC => { let addr = self.fetch16(bus); if  self.pvf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // ED prefix
            0xED => { self.exec_ed(bus); }
            // XOR n
            0xEE => { let n = self.fetch(bus); self.xor8(n); self.cycles += 7; }
            // RST 0x28
            0xEF => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0028; self.cycles += 11; }
            // RET P (positive = not sign)
            0xF0 => { if !self.sf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // POP AF
            0xF1 => { let v = self.pop16(bus); self.set_af(v); self.cycles += 10; }
            // JP P,nn
            0xF2 => { let addr = self.fetch16(bus); if !self.sf() { self.pc = addr; } self.cycles += 10; }
            // DI
            0xF3 => { self.iff1 = false; self.iff2 = false; self.cycles += 4; }
            // CALL P,nn
            0xF4 => { let addr = self.fetch16(bus); if !self.sf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // PUSH AF
            0xF5 => { let v = self.af(); self.push16(bus, v); self.cycles += 11; }
            // OR n
            0xF6 => { let n = self.fetch(bus); self.or8(n); self.cycles += 7; }
            // RST 0x30
            0xF7 => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0030; self.cycles += 11; }
            // RET M (minus = sign set)
            0xF8 => { if  self.sf() { self.pc = self.pop16(bus); self.cycles += 11; } else { self.cycles += 5; } }
            // LD SP,HL
            0xF9 => { self.sp = self.hl(); self.cycles += 6; }
            // JP M,nn
            0xFA => { let addr = self.fetch16(bus); if  self.sf() { self.pc = addr; } self.cycles += 10; }
            // EI
            0xFB => { self.iff1 = true; self.iff2 = true; self.cycles += 4; }
            // CALL M,nn
            0xFC => { let addr = self.fetch16(bus); if  self.sf() { let pc = self.pc; self.push16(bus, pc); self.pc = addr; self.cycles += 17; } else { self.cycles += 10; } }
            // FD prefix: IY
            0xFD => { self.exec_fd(bus); }
            // CP n
            0xFE => { let n = self.fetch(bus); self.cp8(n); self.cycles += 7; }
            // RST 0x38
            0xFF => { let pc = self.pc; self.push16(bus, pc); self.pc = 0x0038; self.cycles += 11; }
        }

        (self.cycles - start) as u32
    }

    // ── CB prefix ─────────────────────────────────────────────────────────
    fn exec_cb(&mut self, bus: &mut impl BusAccess) {
        let op = self.fetch(bus);
        let r = op & 7;
        let op2 = (op >> 3) & 7;
        let v = if r == 6 { bus.mem_read(self.hl()) } else { self.get_reg(bus, r) };
        let result = match op >> 6 {
            0 => match op2 {
                0 => self.rlc(v),
                1 => self.rrc(v),
                2 => self.rl(v),
                3 => self.rr(v),
                4 => self.sla(v),
                5 => self.sra(v),
                6 => self.sll(v),
                7 => self.srl(v),
                _ => v,
            },
            1 => { self.bit(op2, v); v } // BIT: no write-back
            2 => v & !(1 << op2), // RES
            3 => v | (1 << op2),  // SET
            _ => v,
        };
        if op >> 6 != 1 {
            // Write back (not BIT)
            if r == 6 {
                bus.mem_write(self.hl(), result);
                self.cycles += 15;
            } else {
                self.set_reg(r, result);
                self.cycles += 8;
            }
        } else {
            self.cycles += if r == 6 { 12 } else { 8 };
        }
    }

    // ── DD prefix (IX) ────────────────────────────────────────────────────
    fn exec_dd(&mut self, bus: &mut impl BusAccess) {
        let op = self.fetch(bus);
        self.exec_xy(bus, op, false);
    }

    // ── FD prefix (IY) ────────────────────────────────────────────────────
    fn exec_fd(&mut self, bus: &mut impl BusAccess) {
        let op = self.fetch(bus);
        self.exec_xy(bus, op, true);
    }

    fn exec_xy(&mut self, bus: &mut impl BusAccess, op: u8, use_iy: bool) {
        macro_rules! xy { () => { if use_iy { self.iy } else { self.ix } } }
        macro_rules! set_xy { ($v:expr) => { if use_iy { self.iy = $v } else { self.ix = $v } } }
        macro_rules! xyh { () => { if use_iy { (self.iy >> 8) as u8 } else { (self.ix >> 8) as u8 } } }
        macro_rules! xyl { () => { if use_iy { self.iy as u8 } else { self.ix as u8 } } }
        macro_rules! set_xyh { ($v:expr) => { if use_iy { self.iy = (self.iy & 0x00FF) | (($v as u16) << 8); } else { self.ix = (self.ix & 0x00FF) | (($v as u16) << 8); } } }
        macro_rules! set_xyl { ($v:expr) => { if use_iy { self.iy = (self.iy & 0xFF00) | ($v as u16); } else { self.ix = (self.ix & 0xFF00) | ($v as u16); } } }

        match op {
            // LD XY,nn
            0x21 => { let v = self.fetch16(bus); set_xy!(v); self.cycles += 14; }
            // LD (nn),XY
            0x22 => { let addr = self.fetch16(bus); self.write16(bus, addr, xy!()); self.cycles += 20; }
            // INC XY
            0x23 => { set_xy!(xy!().wrapping_add(1)); self.cycles += 10; }
            // INC XYH
            0x24 => { let v = xyh!(); let r = self.inc8(v); set_xyh!(r); self.cycles += 8; }
            // DEC XYH
            0x25 => { let v = xyh!(); let r = self.dec8(v); set_xyh!(r); self.cycles += 8; }
            // LD XYH,n
            0x26 => { let n = self.fetch(bus); set_xyh!(n); self.cycles += 11; }
            // ADD XY,BC
            0x09 => { let xy = xy!(); let r = self.add16(xy, self.bc()); set_xy!(r); self.cycles += 15; }
            // ADD XY,DE
            0x19 => { let xy = xy!(); let r = self.add16(xy, self.de()); set_xy!(r); self.cycles += 15; }
            // ADD XY,XY
            0x29 => { let xy = xy!(); let r = self.add16(xy, xy); set_xy!(r); self.cycles += 15; }
            // ADD XY,SP
            0x39 => { let xy = xy!(); let sp = self.sp; let r = self.add16(xy, sp); set_xy!(r); self.cycles += 15; }
            // LD XY,(nn)
            0x2A => { let addr = self.fetch16(bus); let v = self.read16(bus, addr); set_xy!(v); self.cycles += 20; }
            // DEC XY
            0x2B => { set_xy!(xy!().wrapping_sub(1)); self.cycles += 10; }
            // INC XYL
            0x2C => { let v = xyl!(); let r = self.inc8(v); set_xyl!(r); self.cycles += 8; }
            // DEC XYL
            0x2D => { let v = xyl!(); let r = self.dec8(v); set_xyl!(r); self.cycles += 8; }
            // LD XYL,n
            0x2E => { let n = self.fetch(bus); set_xyl!(n); self.cycles += 11; }
            // INC (XY+d)
            0x34 => {
                let d = self.fetch(bus) as i8;
                let addr = xy!().wrapping_add(d as u16);
                let v = bus.mem_read(addr);
                let r = self.inc8(v);
                bus.mem_write(addr, r);
                self.cycles += 23;
            }
            // DEC (XY+d)
            0x35 => {
                let d = self.fetch(bus) as i8;
                let addr = xy!().wrapping_add(d as u16);
                let v = bus.mem_read(addr);
                let r = self.dec8(v);
                bus.mem_write(addr, r);
                self.cycles += 23;
            }
            // LD (XY+d),n
            0x36 => {
                let d = self.fetch(bus) as i8;
                let n = self.fetch(bus);
                let addr = xy!().wrapping_add(d as u16);
                bus.mem_write(addr, n);
                self.cycles += 19;
            }
            // LD r,(XY+d) — 0x46,0x4E,...,0x7E (but not 0x76)
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x7E => {
                let d = self.fetch(bus) as i8;
                let addr = xy!().wrapping_add(d as u16);
                let v = bus.mem_read(addr);
                let dst = (op >> 3) & 7;
                self.set_reg(dst, v);
                self.cycles += 19;
            }
            // LD (XY+d),r — 0x70..0x77 (but not 0x76)
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                let d = self.fetch(bus) as i8;
                let addr = xy!().wrapping_add(d as u16);
                let src = op & 7;
                let v = self.get_reg(bus, src);
                bus.mem_write(addr, v);
                self.cycles += 19;
            }
            // LD B/D/H/XYH from XYH/XYL
            0x44 => { let v = xyh!(); self.b = v; self.cycles += 8; }
            0x45 => { let v = xyl!(); self.b = v; self.cycles += 8; }
            0x4C => { let v = xyh!(); self.c = v; self.cycles += 8; }
            0x4D => { let v = xyl!(); self.c = v; self.cycles += 8; }
            0x54 => { let v = xyh!(); self.d = v; self.cycles += 8; }
            0x55 => { let v = xyl!(); self.d = v; self.cycles += 8; }
            0x5C => { let v = xyh!(); self.e = v; self.cycles += 8; }
            0x5D => { let v = xyl!(); self.e = v; self.cycles += 8; }
            0x60 => { let v = self.b; set_xyh!(v); self.cycles += 8; }
            0x61 => { let v = self.c; set_xyh!(v); self.cycles += 8; }
            0x62 => { let v = self.d; set_xyh!(v); self.cycles += 8; }
            0x63 => { let v = self.e; set_xyh!(v); self.cycles += 8; }
            0x64 => { self.cycles += 8; } // LD XYH,XYH (nop)
            0x65 => { let v = xyl!(); set_xyh!(v); self.cycles += 8; }
            0x67 => { let v = self.a; set_xyh!(v); self.cycles += 8; }
            0x68 => { let v = self.b; set_xyl!(v); self.cycles += 8; }
            0x69 => { let v = self.c; set_xyl!(v); self.cycles += 8; }
            0x6A => { let v = self.d; set_xyl!(v); self.cycles += 8; }
            0x6B => { let v = self.e; set_xyl!(v); self.cycles += 8; }
            0x6C => { let v = xyh!(); set_xyl!(v); self.cycles += 8; }
            0x6D => { self.cycles += 8; } // LD XYL,XYL
            0x6F => { let v = self.a; set_xyl!(v); self.cycles += 8; }
            0x7C => { self.a = xyh!(); self.cycles += 8; }
            0x7D => { self.a = xyl!(); self.cycles += 8; }
            // ADD/ADC/SUB/SBC/AND/XOR/OR/CP with (XY+d)
            0x86 => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); self.add8(v, false); self.cycles += 19; }
            0x8E => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); let c = self.cf(); self.add8(v, c); self.cycles += 19; }
            0x96 => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); self.sub8(v, false); self.cycles += 19; }
            0x9E => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); let c = self.cf(); self.sub8(v, c); self.cycles += 19; }
            0xA6 => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); self.and8(v); self.cycles += 19; }
            0xAE => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); self.xor8(v); self.cycles += 19; }
            0xB6 => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); self.or8(v); self.cycles += 19; }
            0xBE => { let d = self.fetch(bus) as i8; let v = bus.mem_read(xy!().wrapping_add(d as u16)); self.cp8(v); self.cycles += 19; }
            // ADD with XYH/XYL
            0x84 => { let v = xyh!(); self.add8(v, false); self.cycles += 8; }
            0x85 => { let v = xyl!(); self.add8(v, false); self.cycles += 8; }
            0x8C => { let v = xyh!(); let c = self.cf(); self.add8(v, c); self.cycles += 8; }
            0x8D => { let v = xyl!(); let c = self.cf(); self.add8(v, c); self.cycles += 8; }
            0x94 => { let v = xyh!(); self.sub8(v, false); self.cycles += 8; }
            0x95 => { let v = xyl!(); self.sub8(v, false); self.cycles += 8; }
            0x9C => { let v = xyh!(); let c = self.cf(); self.sub8(v, c); self.cycles += 8; }
            0x9D => { let v = xyl!(); let c = self.cf(); self.sub8(v, c); self.cycles += 8; }
            0xA4 => { let v = xyh!(); self.and8(v); self.cycles += 8; }
            0xA5 => { let v = xyl!(); self.and8(v); self.cycles += 8; }
            0xAC => { let v = xyh!(); self.xor8(v); self.cycles += 8; }
            0xAD => { let v = xyl!(); self.xor8(v); self.cycles += 8; }
            0xB4 => { let v = xyh!(); self.or8(v); self.cycles += 8; }
            0xB5 => { let v = xyl!(); self.or8(v); self.cycles += 8; }
            0xBC => { let v = xyh!(); self.cp8(v); self.cycles += 8; }
            0xBD => { let v = xyl!(); self.cp8(v); self.cycles += 8; }
            // POP XY
            0xE1 => { let v = self.pop16(bus); set_xy!(v); self.cycles += 14; }
            // EX (SP),XY
            0xE3 => {
                let sp = self.sp;
                let mem_val = self.read16(bus, sp);
                let xy = xy!();
                self.write16(bus, sp, xy);
                set_xy!(mem_val);
                self.cycles += 23;
            }
            // PUSH XY
            0xE5 => { let v = xy!(); self.push16(bus, v); self.cycles += 15; }
            // JP (XY)
            0xE9 => { self.pc = xy!(); self.cycles += 8; }
            // LD SP,XY
            0xF9 => { self.sp = xy!(); self.cycles += 10; }
            // DD CB / FD CB
            0xCB => { self.exec_xy_cb(bus, use_iy); }
            // Unknown/undocumented: treat as NOP + 4 cycles
            _ => { self.cycles += 4; }
        }
    }

    fn exec_xy_cb(&mut self, bus: &mut impl BusAccess, use_iy: bool) {
        let d = self.fetch(bus) as i8;
        let op = self.fetch(bus);
        let xy = if use_iy { self.iy } else { self.ix };
        let addr = xy.wrapping_add(d as u16);
        let v = bus.mem_read(addr);
        let b = (op >> 3) & 7;
        let r = op & 7;

        let result = match op >> 6 {
            0 => match b {
                0 => self.rlc(v),
                1 => self.rrc(v),
                2 => self.rl(v),
                3 => self.rr(v),
                4 => self.sla(v),
                5 => self.sra(v),
                6 => self.sll(v),
                7 => self.srl(v),
                _ => v,
            },
            1 => { self.bit(b, v); self.cycles += 20; return; }
            2 => v & !(1 << b),
            3 => v | (1 << b),
            _ => v,
        };
        bus.mem_write(addr, result);
        // Also write to register if r != 6
        if r != 6 { self.set_reg(r, result); }
        self.cycles += 23;
    }

    // ── ED prefix ─────────────────────────────────────────────────────────
    fn exec_ed(&mut self, bus: &mut impl BusAccess) {
        let op = self.fetch(bus);
        match op {
            // IN r,(C)
            0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x78 => {
                let r = (op >> 3) & 7;
                let bc = self.bc();
                let v = bus.io_read(bc);
                self.set_reg(r, v);
                self.f = (self.f & FLAG_C) | parity(v) | (if v == 0 { FLAG_Z } else { 0 }) | (v & FLAG_S) | (v & (FLAG_Y|FLAG_X));
                self.cycles += 12;
            }
            // IN F,(C) — undocumented: flags only
            0x70 => {
                let bc = self.bc();
                let v = bus.io_read(bc);
                self.f = (self.f & FLAG_C) | parity(v) | (if v == 0 { FLAG_Z } else { 0 }) | (v & FLAG_S) | (v & (FLAG_Y|FLAG_X));
                self.cycles += 12;
            }
            // OUT (C),r
            0x41 | 0x49 | 0x51 | 0x59 | 0x61 | 0x69 | 0x79 => {
                let r = (op >> 3) & 7;
                let bc = self.bc();
                let v = self.get_reg(bus, r);
                bus.io_write(bc, v);
                self.cycles += 12;
            }
            // OUT (C),0
            0x71 => { let bc = self.bc(); bus.io_write(bc, 0); self.cycles += 12; }
            // SBC HL,rr
            0x42 => { let hl = self.hl(); let r = self.sbc16(hl, self.bc()); self.set_hl(r); self.cycles += 15; }
            0x52 => { let hl = self.hl(); let r = self.sbc16(hl, self.de()); self.set_hl(r); self.cycles += 15; }
            0x62 => { let hl = self.hl(); let r = self.sbc16(hl, self.hl()); self.set_hl(r); self.cycles += 15; }
            0x72 => { let hl = self.hl(); let sp = self.sp; let r = self.sbc16(hl, sp); self.set_hl(r); self.cycles += 15; }
            // ADC HL,rr
            0x4A => { let hl = self.hl(); let r = self.adc16(hl, self.bc()); self.set_hl(r); self.cycles += 15; }
            0x5A => { let hl = self.hl(); let r = self.adc16(hl, self.de()); self.set_hl(r); self.cycles += 15; }
            0x6A => { let hl = self.hl(); let r = self.adc16(hl, self.hl()); self.set_hl(r); self.cycles += 15; }
            0x7A => { let hl = self.hl(); let sp = self.sp; let r = self.adc16(hl, sp); self.set_hl(r); self.cycles += 15; }
            // LD (nn),rr
            0x43 => { let addr = self.fetch16(bus); let bc = self.bc(); self.write16(bus, addr, bc); self.cycles += 20; }
            0x53 => { let addr = self.fetch16(bus); let de = self.de(); self.write16(bus, addr, de); self.cycles += 20; }
            0x63 => { let addr = self.fetch16(bus); let hl = self.hl(); self.write16(bus, addr, hl); self.cycles += 20; }
            0x73 => { let addr = self.fetch16(bus); let sp = self.sp; self.write16(bus, addr, sp); self.cycles += 20; }
            // LD rr,(nn)
            0x4B => { let addr = self.fetch16(bus); let v = self.read16(bus, addr); self.set_bc(v); self.cycles += 20; }
            0x5B => { let addr = self.fetch16(bus); let v = self.read16(bus, addr); self.set_de(v); self.cycles += 20; }
            0x6B => { let addr = self.fetch16(bus); let v = self.read16(bus, addr); self.set_hl(v); self.cycles += 20; }
            0x7B => { let addr = self.fetch16(bus); self.sp = self.read16(bus, addr); self.cycles += 20; }
            // NEG
            0x44 | 0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C => { self.neg(); self.cycles += 8; }
            // RETN
            0x45 | 0x55 | 0x65 | 0x75 => { self.iff1 = self.iff2; self.pc = self.pop16(bus); self.cycles += 14; }
            // RETI
            0x4D => { self.iff1 = self.iff2; self.pc = self.pop16(bus); self.cycles += 14; }
            // IM 0
            0x46 | 0x66 => { self.im = 0; self.cycles += 8; }
            // IM 1
            0x56 | 0x76 => { self.im = 1; self.cycles += 8; }
            // IM 2
            0x5E | 0x7E => { self.im = 2; self.cycles += 8; }
            // LD I,A
            0x47 => { self.i = self.a; self.cycles += 9; }
            // LD R,A
            0x4F => { self.r = self.a; self.cycles += 9; }
            // LD A,I
            0x57 => {
                self.a = self.i;
                let iff2 = self.iff2;
                self.f = (self.f & FLAG_C)
                       | (if self.a == 0 { FLAG_Z } else { 0 })
                       | (self.a & FLAG_S)
                       | (if iff2 { FLAG_PV } else { 0 })
                       | (self.a & (FLAG_Y|FLAG_X));
                self.cycles += 9;
            }
            // LD A,R
            0x5F => {
                self.a = self.r;
                let iff2 = self.iff2;
                self.f = (self.f & FLAG_C)
                       | (if self.a == 0 { FLAG_Z } else { 0 })
                       | (self.a & FLAG_S)
                       | (if iff2 { FLAG_PV } else { 0 })
                       | (self.a & (FLAG_Y|FLAG_X));
                self.cycles += 9;
            }
            // RLD
            0x6F => {
                let hl = self.hl();
                let m = bus.mem_read(hl);
                let new_m = (m << 4) | (self.a & 0x0F);
                bus.mem_write(hl, new_m);
                self.a = (self.a & 0xF0) | (m >> 4);
                self.f = (self.f & FLAG_C) | parity(self.a)
                       | (if self.a == 0 { FLAG_Z } else { 0 })
                       | (self.a & FLAG_S)
                       | (self.a & (FLAG_Y|FLAG_X));
                self.cycles += 18;
            }
            // RRD
            0x67 => {
                let hl = self.hl();
                let m = bus.mem_read(hl);
                let new_m = (m >> 4) | (self.a << 4);
                bus.mem_write(hl, new_m);
                self.a = (self.a & 0xF0) | (m & 0x0F);
                self.f = (self.f & FLAG_C) | parity(self.a)
                       | (if self.a == 0 { FLAG_Z } else { 0 })
                       | (self.a & FLAG_S)
                       | (self.a & (FLAG_Y|FLAG_X));
                self.cycles += 18;
            }
            // LDI
            0xA0 => {
                let de = self.de(); let hl = self.hl();
                let v = bus.mem_read(hl); bus.mem_write(de, v);
                self.set_hl(hl.wrapping_add(1));
                self.set_de(de.wrapping_add(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                let n = v.wrapping_add(self.a);
                self.f = (self.f & (FLAG_S|FLAG_Z|FLAG_C))
                       | (if bc != 0 { FLAG_PV } else { 0 })
                       | (n & FLAG_Y) | (if n & 0x02 != 0 { FLAG_X } else { 0 });
                self.cycles += 16;
            }
            // CPI
            0xA1 => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                let r = self.a.wrapping_sub(v);
                let h = (self.a & 0x0F) < (v & 0x0F);
                self.set_hl(hl.wrapping_add(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                let n = r.wrapping_sub(if h { 1 } else { 0 });
                self.f = FLAG_N
                       | (self.f & FLAG_C)
                       | (if r == 0 { FLAG_Z } else { 0 })
                       | (r & FLAG_S)
                       | (if h { FLAG_H } else { 0 })
                       | (if bc != 0 { FLAG_PV } else { 0 })
                       | (n & FLAG_Y) | (if n & 0x02 != 0 { FLAG_X } else { 0 });
                self.cycles += 16;
            }
            // INI
            0xA2 => {
                let bc = self.bc();
                let v = bus.io_read(bc);
                let hl = self.hl(); bus.mem_write(hl, v);
                self.set_hl(hl.wrapping_add(1));
                self.b = self.b.wrapping_sub(1);
                self.f = FLAG_N | (if self.b == 0 { FLAG_Z } else { 0 }) | (self.b & FLAG_S);
                self.cycles += 16;
            }
            // OUTI
            0xA3 => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                self.set_hl(hl.wrapping_add(1));
                self.b = self.b.wrapping_sub(1);
                let bc = self.bc(); bus.io_write(bc, v);
                self.f = FLAG_N | (if self.b == 0 { FLAG_Z } else { 0 }) | (self.b & FLAG_S);
                self.cycles += 16;
            }
            // LDD
            0xA8 => {
                let de = self.de(); let hl = self.hl();
                let v = bus.mem_read(hl); bus.mem_write(de, v);
                self.set_hl(hl.wrapping_sub(1));
                self.set_de(de.wrapping_sub(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                let n = v.wrapping_add(self.a);
                self.f = (self.f & (FLAG_S|FLAG_Z|FLAG_C))
                       | (if bc != 0 { FLAG_PV } else { 0 })
                       | (n & FLAG_Y) | (if n & 0x02 != 0 { FLAG_X } else { 0 });
                self.cycles += 16;
            }
            // CPD
            0xA9 => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                let r = self.a.wrapping_sub(v);
                let h = (self.a & 0x0F) < (v & 0x0F);
                self.set_hl(hl.wrapping_sub(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                let n = r.wrapping_sub(if h { 1 } else { 0 });
                self.f = FLAG_N
                       | (self.f & FLAG_C)
                       | (if r == 0 { FLAG_Z } else { 0 })
                       | (r & FLAG_S)
                       | (if h { FLAG_H } else { 0 })
                       | (if bc != 0 { FLAG_PV } else { 0 })
                       | (n & FLAG_Y) | (if n & 0x02 != 0 { FLAG_X } else { 0 });
                self.cycles += 16;
            }
            // IND
            0xAA => {
                let bc = self.bc(); let v = bus.io_read(bc);
                let hl = self.hl(); bus.mem_write(hl, v);
                self.set_hl(hl.wrapping_sub(1));
                self.b = self.b.wrapping_sub(1);
                self.f = FLAG_N | (if self.b == 0 { FLAG_Z } else { 0 }) | (self.b & FLAG_S);
                self.cycles += 16;
            }
            // OUTD
            0xAB => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                self.set_hl(hl.wrapping_sub(1));
                self.b = self.b.wrapping_sub(1);
                let bc = self.bc(); bus.io_write(bc, v);
                self.f = FLAG_N | (if self.b == 0 { FLAG_Z } else { 0 }) | (self.b & FLAG_S);
                self.cycles += 16;
            }
            // LDIR
            0xB0 => {
                let de = self.de(); let hl = self.hl();
                let v = bus.mem_read(hl); bus.mem_write(de, v);
                self.set_hl(hl.wrapping_add(1));
                self.set_de(de.wrapping_add(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                if bc != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                    self.cycles += 21;
                } else {
                    self.f &= !(FLAG_H | FLAG_PV | FLAG_N);
                    self.cycles += 16;
                }
            }
            // CPIR
            0xB1 => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                let r = self.a.wrapping_sub(v);
                let h = (self.a & 0x0F) < (v & 0x0F);
                self.set_hl(hl.wrapping_add(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                let n = r.wrapping_sub(if h { 1 } else { 0 });
                self.f = FLAG_N
                       | (self.f & FLAG_C)
                       | (if r == 0 { FLAG_Z } else { 0 })
                       | (r & FLAG_S)
                       | (if h { FLAG_H } else { 0 })
                       | (if bc != 0 { FLAG_PV } else { 0 })
                       | (n & FLAG_Y) | (if n & 0x02 != 0 { FLAG_X } else { 0 });
                if bc != 0 && r != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                    self.cycles += 21;
                } else {
                    self.cycles += 16;
                }
            }
            // INIR
            0xB2 => {
                let bc = self.bc(); let v = bus.io_read(bc);
                let hl = self.hl(); bus.mem_write(hl, v);
                self.set_hl(hl.wrapping_add(1));
                self.b = self.b.wrapping_sub(1);
                if self.b != 0 { self.pc = self.pc.wrapping_sub(2); self.cycles += 21; }
                else { self.f = FLAG_Z | FLAG_N; self.cycles += 16; }
            }
            // OTIR
            0xB3 => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                self.set_hl(hl.wrapping_add(1));
                self.b = self.b.wrapping_sub(1);
                let bc = self.bc(); bus.io_write(bc, v);
                if self.b != 0 { self.pc = self.pc.wrapping_sub(2); self.cycles += 21; }
                else { self.f = FLAG_Z | FLAG_N; self.cycles += 16; }
            }
            // LDDR
            0xB8 => {
                let de = self.de(); let hl = self.hl();
                let v = bus.mem_read(hl); bus.mem_write(de, v);
                self.set_hl(hl.wrapping_sub(1));
                self.set_de(de.wrapping_sub(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                if bc != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                    self.cycles += 21;
                } else {
                    self.f &= !(FLAG_H | FLAG_PV | FLAG_N);
                    self.cycles += 16;
                }
            }
            // CPDR
            0xB9 => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                let r = self.a.wrapping_sub(v);
                let h = (self.a & 0x0F) < (v & 0x0F);
                self.set_hl(hl.wrapping_sub(1));
                let bc = self.bc().wrapping_sub(1); self.set_bc(bc);
                let n = r.wrapping_sub(if h { 1 } else { 0 });
                self.f = FLAG_N
                       | (self.f & FLAG_C)
                       | (if r == 0 { FLAG_Z } else { 0 })
                       | (r & FLAG_S)
                       | (if h { FLAG_H } else { 0 })
                       | (if bc != 0 { FLAG_PV } else { 0 })
                       | (n & FLAG_Y) | (if n & 0x02 != 0 { FLAG_X } else { 0 });
                if bc != 0 && r != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                    self.cycles += 21;
                } else {
                    self.cycles += 16;
                }
            }
            // INDR
            0xBA => {
                let bc = self.bc(); let v = bus.io_read(bc);
                let hl = self.hl(); bus.mem_write(hl, v);
                self.set_hl(hl.wrapping_sub(1));
                self.b = self.b.wrapping_sub(1);
                if self.b != 0 { self.pc = self.pc.wrapping_sub(2); self.cycles += 21; }
                else { self.f = FLAG_Z | FLAG_N; self.cycles += 16; }
            }
            // OTDR
            0xBB => {
                let hl = self.hl(); let v = bus.mem_read(hl);
                self.set_hl(hl.wrapping_sub(1));
                self.b = self.b.wrapping_sub(1);
                let bc = self.bc(); bus.io_write(bc, v);
                if self.b != 0 { self.pc = self.pc.wrapping_sub(2); self.cycles += 21; }
                else { self.f = FLAG_Z | FLAG_N; self.cycles += 16; }
            }
            // Unknown ED opcode: treat as NOP
            _ => { self.cycles += 8; }
        }
    }
}

// ── Parity helper ─────────────────────────────────────────────────────────
#[inline]
fn parity(v: u8) -> u8 {
    if v.count_ones() % 2 == 0 { FLAG_PV } else { 0 }
}
