//! The CHIP-8 CPU emulation and instruction set

use rand;

///The core CPU registers and memory
pub struct Cpu {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
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
            0x8 => self.v8,
            0x9 => self.v9,
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
            0x8 => &mut self.v8,
            0x9 => &mut self.v9,
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
    pub fn skne_const(&mut self, register_id: u8, constant: u8) {
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
        if x == y {
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
    ///No carry generated
    pub fn add_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.id_to_reg_mut(register_id);
        *reg = reg.wrapping_add(constant);
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
        let (result, overflow) = x.overflowing_add(y);
        if overflow {
            self.vf = 0x01;
        }
        *self.id_to_reg_mut(register_x_id) = result;
    }

    ///8XY5
    ///sub vx,vy subtract register VY from VX, borrow stored in register VF
    ///register VF set to 1 if borrows
    pub fn sub_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg(register_x_id);
        let (result, borrow) = x.overflowing_sub(y);
        if borrow {
            self.vf = 0x01;
        }
        *self.id_to_reg_mut(register_x_id) = result;
    }

    ///8X06 shr vx  shift register VX right, bit 0 goes into register VF
    pub fn shr(&mut self, register_x_id: u8) {
        let x = self.id_to_reg(register_x_id);
        self.vf = x & 0x01;
        *self.id_to_reg_mut(register_x_id) = x >> 1;
    }

    ///8XY7 rsb vx,vy   subtract register VX from register VY
    ///result stored in register VX
    ///register F set to 1 if borrows
    pub fn rsb(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg(register_x_id);
        let (result, borrow) = y.overflowing_sub(x);
        if borrow {
            self.vf = 0x01;
        }
        *self.id_to_reg_mut(register_x_id) = result;
    }

    ///8X0E shl vx  shift register VX left, bit 7 stored into register VF
    pub fn shl(&mut self, register_x_id: u8) {
        let x = self.id_to_reg(register_x_id);
        if x & 0x80 != 0 {
            self.vf = 0x01;
        }
        *self.id_to_reg_mut(register_x_id) = x << 1;
    }

    ///9XY0 skne vx,vy  skip next instruction
    ///if register VX != register VY
    pub fn skne_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let x = self.id_to_reg(register_x_id);
        let y = self.id_to_reg(register_y_id);
        if x != y {
            self.pc += INSTRUCTION_WIDTH;
        }
    }

    ///ANNN mvi nnn Load index register (I) with constant NNN
    pub fn mvi(&mut self, value: u16) {
        self.i = value & 0xFFF;
    }

    ///BNNN jmi nnn Jump to address NNN + register V0
    pub fn jmi(&mut self, value: u16) {
        self.pc = u16::from(self.id_to_reg(0)).wrapping_add(value & 0xFFF);
    }

    ///CXKK rand vx,kk register VX = random number AND KK
    pub fn rand(&mut self, register_x_id: u8, value: u8) {
        *self.id_to_reg_mut(register_x_id) = rand::random::<u8>() & value;
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
            v8: 0,
            v9: 0,
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
    fn test_jmp() {
        let mut cpu = Cpu::new();
        cpu.jmp(0x400);
        assert_eq!(cpu.pc, 0x400);

        cpu.jmp(0x600);
        assert_eq!(cpu.pc, 0x600);
    }

    #[test]
    fn test_jsr_rts() {
        let mut cpu = Cpu::new();
        cpu.jsr(0x400);
        assert_eq!(cpu.pc, 0x400);

        cpu.rts();
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_jsr_rts_nested() {
        let mut cpu = Cpu::new();
        cpu.jsr(0x400);
        assert_eq!(cpu.pc, 0x400);
        cpu.jsr(0x430);
        assert_eq!(cpu.pc, 0x430);
        cpu.jsr(0x440);
        assert_eq!(cpu.pc, 0x440);

        cpu.rts();
        assert_eq!(cpu.pc, 0x432);
        cpu.rts();
        assert_eq!(cpu.pc, 0x402);
        cpu.rts();
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skeq_const() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0) = 0x01;
        cpu.skeq_const(0, 0x01);
        assert_eq!(cpu.pc, 0x202);

        cpu.skeq_const(0, 0x02);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skne_const() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0) = 0x01;
        cpu.skne_const(0, 0x01);
        assert_eq!(cpu.pc, 0x200);

        cpu.skne_const(0, 0x02);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skeq_reg() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0) = 0x01;
        *cpu.id_to_reg_mut(1) = 0x01;
        cpu.skeq_reg(0, 1);
        assert_eq!(cpu.pc, 0x202);

        *cpu.id_to_reg_mut(2) = 0x02;
        cpu.skeq_reg(1, 2);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_mov_const() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(1) = 0x01;
        cpu.mov_const(1, 0x10);
        assert_eq!(cpu.id_to_reg(1), 0x10);
    }

    #[test]
    fn test_add_const() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(1) = 0x01;
        cpu.add_const(1, 0x10);
        assert_eq!(cpu.id_to_reg(1), 0x01 + 0x10);
    }

    #[test]
    fn test_mov_reg() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0xA) = 0x01;
        *cpu.id_to_reg_mut(3) = 0x10;
        cpu.mov_reg(0xA, 3);
        assert_eq!(cpu.id_to_reg(0xA), 0x10);
        assert_eq!(cpu.id_to_reg(3), 0x10);
    }

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
        cpu.vd = 0x01;
        cpu.ve = 0x10;
        cpu.add_reg(0xd, 0xe);
        assert_eq!(cpu.vd, 0x01 + 0x10);
        assert_eq!(cpu.ve, 0x10);
    }

    #[test]
    fn test_add_reg_overflow() {
        let mut cpu = Cpu::new();
        cpu.vc = 0xFF;
        cpu.vd = 0x01;
        cpu.add_reg(0xc, 0xd);
        assert_eq!(cpu.vc, 0x00);
        assert_eq!(cpu.vf, 0x01);
    }

    #[test]
    fn test_add_reg_overflow2() {
        let mut cpu = Cpu::new();
        cpu.va = 0xFF;
        cpu.vb = 0xFF;
        cpu.add_reg(0xa, 0xb);
        assert_eq!(cpu.va, 0xFE);
        assert_eq!(cpu.vf, 0x01);
    }

    #[test]
    fn test_sub_reg() {
        let mut cpu = Cpu::new();
        cpu.v8 = 0x10;
        cpu.v9 = 0x01;
        cpu.sub_reg(8, 9);
        assert_eq!(cpu.v8, 0x10 - 0x01);
        assert_eq!(cpu.v9, 0x01);
    }

    #[test]
    fn test_sub_reg_underflow() {
        let mut cpu = Cpu::new();
        cpu.v5 = 0x00;
        cpu.v6 = 0x01;
        cpu.sub_reg(5, 6);
        assert_eq!(cpu.v5, 0xFF);
        assert_eq!(cpu.vf, 0x01);
    }

    #[test]
    fn test_shr() {
        let mut cpu = Cpu::new();
        cpu.v7 = 0xF1;
        cpu.shr(7);
        assert_eq!(cpu.v7, 0x78);
        assert_eq!(cpu.vf, 0x01);
    }

    #[test]
    fn test_rsb() {
        let mut cpu = Cpu::new();
        cpu.v8 = 0x01;
        cpu.v9 = 0x10;
        cpu.rsb(8, 9);
        assert_eq!(cpu.v8, 0x10 - 0x01);
        assert_eq!(cpu.v9, 0x10);
    }

    #[test]
    fn test_rsb_underflow() {
        let mut cpu = Cpu::new();
        cpu.v5 = 0x01;
        cpu.v6 = 0x00;
        cpu.rsb(5, 6);
        assert_eq!(cpu.v5, 0xFF);
        assert_eq!(cpu.vf, 0x01);
    }

    #[test]
    fn test_shl() {
        let mut cpu = Cpu::new();
        cpu.v7 = 0xF1;
        cpu.shl(7);
        assert_eq!(cpu.v7, 0xE2);
        assert_eq!(cpu.vf, 0x01);
    }

    #[test]
    fn test_skne_reg() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0) = 0x01;
        *cpu.id_to_reg_mut(1) = 0x01;
        cpu.skne_reg(0, 1);
        assert_eq!(cpu.pc, 0x200);

        *cpu.id_to_reg_mut(2) = 0x02;
        cpu.skne_reg(1, 2);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_mvi() {
        let mut cpu = Cpu::new();
        cpu.mvi(0x123);
        assert_eq!(cpu.i, 0x123);

        cpu.mvi(0xF321);
        assert_eq!(cpu.i, 0x321);
    }

    #[test]
    fn test_jmi() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0) = 0x10;
        cpu.jmi(0xF00);
        assert_eq!(cpu.pc, 0xF10);
    }

    #[test]
    fn test_rand() {
        let mut cpu = Cpu::new();
        *cpu.id_to_reg_mut(0xB) = 0x10;
        cpu.rand(0xB, 0x0F);
        assert_ne!(cpu.id_to_reg(0xB), 0x10);
        assert_eq!(cpu.id_to_reg(0xB) & 0xF0, 0x00);
    }

}
