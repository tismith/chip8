use std;

pub struct Cpu {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    va: u8,
    vb: u8,
    vc: u8,
    vd: u8,
    ve: u8,
    vf: u8,
    ///actually 12 bits
    i: u16,
    ///actually 12 bits, pointer into `memory`
    pc: u16,
    sp: Vec<u16>,
    memory: [u8; 4096],
}

const INSTRUCTION_WIDTH: u16 = 2;

impl Cpu {
    ///convert an id to a register reference
    fn id_to_reg(&self, register: u8) -> u8 {
        match register {
            0x0 => self.v0,
            0x1 => self.v1,
            0x2 => self.v2,
            0x3 => self.v3,
            0x4 => self.v4,
            0x5 => self.v5,
            0x6 => self.v6,
            0x7 => self.v7,
            0xA => self.va,
            0xB => self.vb,
            0xC => self.vc,
            0xD => self.vd,
            0xE => self.ve,
            0xF => self.vf,
            _ => panic!("unexpected register id {}", register),
        }
    }

    ///convert an id to a mutable register reference
    fn id_to_reg_mut(&mut self, register: u8) -> &mut u8 {
        match register {
            0x0 => &mut self.v0,
            0x1 => &mut self.v1,
            0x2 => &mut self.v2,
            0x3 => &mut self.v3,
            0x4 => &mut self.v4,
            0x5 => &mut self.v5,
            0x6 => &mut self.v6,
            0x7 => &mut self.v7,
            0xA => &mut self.va,
            0xB => &mut self.vb,
            0xC => &mut self.vc,
            0xD => &mut self.vd,
            0xE => &mut self.ve,
            0xF => &mut self.vf,
            _ => panic!("unexpected register id {}", register),
        }
    }

    ///new, initialized cpu
    pub fn new() -> Self {
        Default::default()
    }

    ///0x00E0
    ///clear the screen
    pub fn cls(&mut self) {
        unimplemented!()
    }

    ///0x00EE
    ///return from subroutine
    pub fn rts(&mut self) {
        if let Some(address) = self.sp.pop() {
            self.pc = address;
        } else {
            error!("rts called with no return address");
        }
    }

    ///0x1NNN (NNN is the address)
    ///jump to address
    pub fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    ///0x2NNN (NNN is the address)
    ///jump to subroutine
    pub fn jsr(&mut self, address: u16) {
        self.sp.push(self.pc + INSTRUCTION_WIDTH);
        self.pc = address;
    }

    ///0x3XRR
    ///skeq - skip next instruction if register VX == constant RR
    pub fn skeq_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.id_to_reg(register_id);
        if reg == constant {
            self.pc += INSTRUCTION_WIDTH;
        }
    }

    ///0x4XRR
    ///skne - skip next intruction if register VX != constant RR
    pub fn skne(&mut self, register_id: u8, constant: u8) {
        let reg = self.id_to_reg(register_id);
        if reg != constant {
            self.pc += INSTRUCTION_WIDTH;
        }
    }

    ///0x5XY0
    ///skeq - skip next instruction if register VX == register VY
    pub fn skeq_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let x = self.id_to_reg(register_x_id);
        let y = self.id_to_reg(register_y_id);
        if x != y {
            self.pc += INSTRUCTION_WIDTH;
        }
    }

    ///0x6XRR
    ///mov - move constant RR to register VX
    pub fn mov_const(&mut self, register_x_id: u8, constant: u8) {
        let reg = self.id_to_reg_mut(register_x_id);
        *reg = constant;
    }

    ///0x7XRR
    ///add = add constant RR to register VX
    pub fn add_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.id_to_reg_mut(register_id);
        *reg += constant;
    }

    ///0x8XY0
    ///mov_reg move register VY into VX
    pub fn mov_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x = y;
    }

    ///0x8XY1
    ///or register VY with register VX, store result into register VX
    pub fn or_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x |= y;
    }

    ///0x8XY2
    ///and register VY with register VX, store result into register VX
    pub fn and_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x &= y;
    }

    ///0x8XY3
    ///xor register VY with register VX, store result into register VX
    pub fn xor_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x ^= y;
    }

    ///0x8XY4
    ///add_reg add register VY to VX, store result in register VX,
    ///carry stored in register VF
    pub fn add_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg(register_x_id);
        if let None = x.checked_add(y) {
            self.vf = 0x01;
        }
        *self.id_to_reg_mut(register_x_id) = x.wrapping_add(y);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            va: 0,
            vb: 0,
            vc: 0,
            vd: 0,
            ve: 0,
            vf: 0,
            i: 0,
            pc: 0x200,
            sp: Vec::new(),
            memory: [0; 4096],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_or_reg() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0xA) = 0x01;
        *cpu.id_to_reg_mut(3) = 0x10;
        cpu.or_reg(0xA, 3);
        assert_eq!(cpu.id_to_reg(0xA), 0x01 | 0x10);
        assert_eq!(cpu.id_to_reg(3), 0x10);
    }

    #[test]
    fn test_xor_reg() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0xA) = 0x01;
        *cpu.id_to_reg_mut(3) = 0x10;
        cpu.xor_reg(0xA, 3);
        assert_eq!(cpu.id_to_reg(0xA), 0x01 ^ 0x10);
        assert_eq!(cpu.id_to_reg(3), 0x10);
    }

    #[test]
    fn test_and_reg() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(2) = 0x01;
        *cpu.id_to_reg_mut(3) = 0x10;
        cpu.and_reg(2, 3);
        assert_eq!(cpu.id_to_reg(2), 0x01 & 0x10);
        assert_eq!(cpu.id_to_reg(3), 0x10);
    }

    #[test]
    fn test_add_reg() {
        let mut cpu = Cpu::new();
        cpu.v2 = 0x01;
        cpu.v3 = 0x10;
        cpu.add_reg(2, 3);
        assert_eq!(cpu.v2, 0x01 + 0x10);
        assert_eq!(cpu.v3, 0x10);
    }

    #[test]
    fn test_add_reg_overflow() {
        let mut cpu = Cpu::new();
        cpu.v2 = 0xFF;
        cpu.v3 = 0x01;
        cpu.add_reg(2, 3);
        assert_eq!(cpu.v2, 0x00);
        assert_eq!(cpu.vf, 0x01);
    }

    #[test]
    fn test_add_reg_overflow2() {
        let mut cpu = Cpu::new();
        cpu.v2 = 0xFF;
        cpu.v3 = 0xFF;
        cpu.add_reg(2, 3);
        assert_eq!(cpu.v2, 0xFE);
        assert_eq!(cpu.vf, 0x01);
    }
}
