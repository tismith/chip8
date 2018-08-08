//! The CHIP-8 CPU emulation and instruction set

use rand;

///The core CPU registers and memory
pub struct Cpu {
    register: [u8; 16],
    ///actually 12 bits
    i: u16,
    ///actually 12 bits, pointer into `memory`
    pc: u16,
    sp: Vec<u16>,
    screen: [u8; 64 * 32 / 8],
    memory: [u8; 4096],
}

const INSTRUCTION_WIDTH: u16 = 2;

impl Cpu {
    ///convert an id to a register reference
    fn reg(&self, register: u8) -> u8 {
        if register <= 0x0F {
            return self.register[usize::from(register)];
        }
        panic!("unexpected register id {}", register)
    }

    ///convert an id to a mutable register reference
    fn reg_mut(&mut self, register: u8) -> &mut u8 {
        if register <= 0x0F {
            return &mut self.register[usize::from(register)];
        }
        panic!("unexpected register id {}", register)
    }

    ///lookup a memory address
    fn mem(&self, address: u16) -> u8 {
        if address < 4096 {
            return self.memory[usize::from(address)];
        }
        panic!("unexpected memory access at {}", address)
    }

    ///lookup a mutable memory address
    fn mem_mut(&mut self, address: u16) -> &mut u8 {
        if address < 4096 {
            return &mut self.memory[usize::from(address)];
        }
        panic!("unexpected memory access at {}", address)
    }

    ///new, initialized cpu
    pub fn new() -> Self {
        Default::default()
    }

    ///returns a slice of the screen
    pub fn screen(&self) -> &[u8; 64 * 32 / 8] {
        &self.screen
    }

    ///0x00E0
    ///clear the screen
    pub fn cls(&mut self) {
        for x in self.screen.iter_mut() {
            *x = 0;
        }
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
        self.sp.push(self.pc.wrapping_add(INSTRUCTION_WIDTH));
        self.pc = address;
    }

    ///0x3XRR
    ///skeq - skip next instruction if register VX == constant RR
    pub fn skeq_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.reg(register_id);
        if reg == constant {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
    }

    ///0x4XRR
    ///skne - skip next intruction if register VX != constant RR
    pub fn skne_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.reg(register_id);
        if reg != constant {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
    }

    ///0x5XY0
    ///skeq - skip next instruction if register VX == register VY
    pub fn skeq_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let x = self.reg(register_x_id);
        let y = self.reg(register_y_id);
        if x == y {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
    }

    ///0x6XRR
    ///mov - move constant RR to register VX
    pub fn mov_const(&mut self, register_x_id: u8, constant: u8) {
        let reg = self.reg_mut(register_x_id);
        *reg = constant;
    }

    ///0x7XRR
    ///add = add constant RR to register VX
    ///No carry generated
    pub fn add_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.reg_mut(register_id);
        *reg = reg.wrapping_add(constant);
    }

    ///0x8XY0
    ///mov_reg move register VY into VX
    pub fn mov_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg_mut(register_x_id);
        *x = y;
    }

    ///0x8XY1
    ///or register VY with register VX, store result into register VX
    pub fn or_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg_mut(register_x_id);
        *x |= y;
    }

    ///0x8XY2
    ///and register VY with register VX, store result into register VX
    pub fn and_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg_mut(register_x_id);
        *x &= y;
    }

    ///0x8XY3
    ///xor register VY with register VX, store result into register VX
    pub fn xor_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg_mut(register_x_id);
        *x ^= y;
    }

    ///0x8XY4
    ///add_reg add register VY to VX, store result in register VX,
    ///carry stored in register VF
    pub fn add_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg(register_x_id);
        let (result, overflow) = x.overflowing_add(y);
        if overflow {
            self.register[0x0F] = 0x01;
        }
        *self.reg_mut(register_x_id) = result;
    }

    ///8XY5
    ///sub vx,vy subtract register VY from VX, borrow stored in register VF
    ///register VF set to 1 if borrows
    pub fn sub_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg(register_x_id);
        let (result, borrow) = x.overflowing_sub(y);
        if borrow {
            self.register[0x0F] = 0x01;
        }
        *self.reg_mut(register_x_id) = result;
    }

    ///8X06 shr vx  shift register VX right, bit 0 goes into register VF
    pub fn shr(&mut self, register_x_id: u8) {
        let x = self.reg(register_x_id);
        self.register[0x0F] = x & 0x01;
        *self.reg_mut(register_x_id) = x >> 1;
    }

    ///8XY7 rsb vx,vy   subtract register VX from register VY
    ///result stored in register VX
    ///register F set to 1 if borrows
    pub fn rsb(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg(register_x_id);
        let (result, borrow) = y.overflowing_sub(x);
        if borrow {
            self.register[0x0F] = 0x01;
        }
        *self.reg_mut(register_x_id) = result;
    }

    ///8X0E shl vx  shift register VX left, bit 7 stored into register VF
    pub fn shl(&mut self, register_x_id: u8) {
        let x = self.reg(register_x_id);
        if x & 0x80 != 0 {
            self.register[0x0F] = 0x01;
        }
        *self.reg_mut(register_x_id) = x << 1;
    }

    ///9XY0 skne vx,vy  skip next instruction
    ///if register VX != register VY
    pub fn skne_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let x = self.reg(register_x_id);
        let y = self.reg(register_y_id);
        if x != y {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
    }

    ///ANNN mvi nnn Load index register (I) with constant NNN
    pub fn mvi(&mut self, value: u16) {
        self.i = value & 0xFFF;
    }

    ///BNNN jmi nnn Jump to address NNN + register V0
    pub fn jmi(&mut self, value: u16) {
        self.pc = u16::from(self.reg(0)).wrapping_add(value & 0xFFF);
    }

    ///CXKK rand vx,kk register VX = random number AND KK
    pub fn rand(&mut self, register_x_id: u8, value: u8) {
        *self.reg_mut(register_x_id) = rand::random::<u8>() & value;
    }

    ///DXYN sprite vx,vy,n  Draw sprite at screen location
    ///(register VX,register VY) height N
    ///Sprites stored in memory at location in index register (I),
    ///maximum 8bits wide. Wraps around
    ///the screen. If when drawn, clears a pixel,
    ///register VF is set to 1 otherwise it is zero. All
    ///drawing is XOR drawing (e.g. it toggles the screen pixels)
    pub fn sprite(&mut self, register_x_id: u8, register_y_id: u8) {
        unimplemented!()
    }

    ///ek9e skpr k  skip if key (register rk) pressed
    ///The key is a key number, see the chip-8
    ///documentation
    pub fn skpr(&mut self, key_id: u8) {
        unimplemented!()
    }

    ///eka1 skup k  skip if key (register rk) not pressed
    pub fn skup(&mut self, key_id: u8) {
        unimplemented!()
    }

    ///fr07 gdelay vr   get delay timer into vr
    pub fn gdelay(&mut self, register_x_id: u8) {
        unimplemented!()
    }

    ///fr0a key vr  wait for for keypress,put key in register vr
    pub fn key(&mut self, register_x_id: u8) {
        unimplemented!()
    }

    ///fr15 sdelay vr   set the delay timer to vr
    pub fn sdelay(&mut self, register_x_id: u8) {
        unimplemented!()
    }

    ///fr18 ssound vr   set the sound timer to vr
    pub fn ssound(&mut self, register_x_id: u8) {
        unimplemented!()
    }

    ///fr1e adi vr  add register vr to the index register
    pub fn adi(&mut self, register_x_id: u8) {
        self.i = self
            .i
            .wrapping_add(u16::from(self.reg(register_x_id)));
    }

    ///fr29 font vr point I to the sprite for hexadecimal
    ///character in vr   Sprite is 5 bytes high
    pub fn font(&mut self, register_x_id: u8) {
        unimplemented!()
    }

    ///fr33 bcd vr  store the bcd representation of register vr
    ///at location I,I+1,I+2
    ///Doesn't change I
    pub fn bcd(&mut self, register_x_id: u8) {
        if self.i >= (4096 - 3) {
            error!("bcd called with I too large: {}", self.i);
            return;
        }
        let x = self.reg(register_x_id);
        let x100 = x / 100;
        let x10 = (x - (x100 * 100)) / 10;
        let x1 = x - (x100 * 100) - (x10 * 10);
        let i = self.i;
        *self.mem_mut(i) = x100;
        *self.mem_mut(i + 1) = x10;
        *self.mem_mut(i + 2) = x1;
    }

    ///fr55 str v0-vr   store registers v0-vr at location I onwards
    ///I is incremented to point to
    ///the next location on. e.g. I = I + r + 1
    pub fn str(&mut self, register_x_id: u8) {
        unimplemented!()
    }

    ///fx65 ldr v0-vr   load registers v0-vr from location I onwards
    ///as above.
    pub fn ldr(&mut self, register_x_id: u8) {
        unimplemented!()
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            register: [0; 16],
            i: 0,
            pc: 0x200,
            sp: Vec::new(),
            screen: [0; 64 * 32 / 8],
            memory: [0; 4096],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cls() {
        let mut cpu = Cpu::new();
        cpu.screen[0] = 0xFF;
        cpu.screen[255] = 0xFF;
        cpu.cls();
        assert_eq!(cpu.screen[0], 0x00);
        assert_eq!(cpu.screen[255], 0x00);
    }

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
        *cpu.reg_mut(0) = 0x01;
        cpu.skeq_const(0, 0x01);
        assert_eq!(cpu.pc, 0x202);

        cpu.skeq_const(0, 0x02);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skne_const() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0) = 0x01;
        cpu.skne_const(0, 0x01);
        assert_eq!(cpu.pc, 0x200);

        cpu.skne_const(0, 0x02);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skeq_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0) = 0x01;
        *cpu.reg_mut(1) = 0x01;
        cpu.skeq_reg(0, 1);
        assert_eq!(cpu.pc, 0x202);

        *cpu.reg_mut(2) = 0x02;
        cpu.skeq_reg(1, 2);
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_mov_const() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(1) = 0x01;
        cpu.mov_const(1, 0x10);
        assert_eq!(cpu.reg(1), 0x10);
    }

    #[test]
    fn test_add_const() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(1) = 0x01;
        cpu.add_const(1, 0x10);
        assert_eq!(cpu.reg(1), 0x01 + 0x10);
    }

    #[test]
    fn test_mov_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0xA) = 0x01;
        *cpu.reg_mut(3) = 0x10;
        cpu.mov_reg(0xA, 3);
        assert_eq!(cpu.reg(0xA), 0x10);
        assert_eq!(cpu.reg(3), 0x10);
    }

    #[test]
    fn test_or_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0xA) = 0x01;
        *cpu.reg_mut(3) = 0x10;
        cpu.or_reg(0xA, 3);
        assert_eq!(cpu.reg(0xA), 0x01 | 0x10);
        assert_eq!(cpu.reg(3), 0x10);
    }

    #[test]
    fn test_xor_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0xA) = 0x01;
        *cpu.reg_mut(3) = 0x10;
        cpu.xor_reg(0xA, 3);
        assert_eq!(cpu.reg(0xA), 0x01 ^ 0x10);
        assert_eq!(cpu.reg(3), 0x10);
    }

    #[test]
    fn test_and_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(2) = 0x01;
        *cpu.reg_mut(3) = 0x10;
        cpu.and_reg(2, 3);
        assert_eq!(cpu.reg(2), 0x01 & 0x10);
        assert_eq!(cpu.reg(3), 0x10);
    }

    #[test]
    fn test_add_reg() {
        let mut cpu = Cpu::new();
        cpu.register[0x0D] = 0x01;
        cpu.register[0x0E] = 0x10;
        cpu.add_reg(0xd, 0xe);
        assert_eq!(cpu.register[0x0D], 0x01 + 0x10);
        assert_eq!(cpu.register[0x0E], 0x10);
    }

    #[test]
    fn test_add_reg_overflow() {
        let mut cpu = Cpu::new();
        cpu.register[0x0C] = 0xFF;
        cpu.register[0x0D] = 0x01;
        cpu.add_reg(0xc, 0xd);
        assert_eq!(cpu.register[0x0C], 0x00);
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_add_reg_overflow2() {
        let mut cpu = Cpu::new();
        cpu.register[0x0A] = 0xFF;
        cpu.register[0x0B] = 0xFF;
        cpu.add_reg(0xa, 0xb);
        assert_eq!(cpu.register[0x0A], 0xFE);
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_sub_reg() {
        let mut cpu = Cpu::new();
        cpu.register[0x08] = 0x10;
        cpu.register[0x09] = 0x01;
        cpu.sub_reg(8, 9);
        assert_eq!(cpu.register[0x08], 0x10 - 0x01);
        assert_eq!(cpu.register[0x09], 0x01);
    }

    #[test]
    fn test_sub_reg_underflow() {
        let mut cpu = Cpu::new();
        cpu.register[0x05] = 0x00;
        cpu.register[0x06] = 0x01;
        cpu.sub_reg(5, 6);
        assert_eq!(cpu.register[0x05], 0xFF);
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_shr() {
        let mut cpu = Cpu::new();
        cpu.register[0x07] = 0xF1;
        cpu.shr(7);
        assert_eq!(cpu.register[0x07], 0x78);
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_rsb() {
        let mut cpu = Cpu::new();
        cpu.register[0x08] = 0x01;
        cpu.register[0x09] = 0x10;
        cpu.rsb(8, 9);
        assert_eq!(cpu.register[0x08], 0x10 - 0x01);
        assert_eq!(cpu.register[0x09], 0x10);
    }

    #[test]
    fn test_rsb_underflow() {
        let mut cpu = Cpu::new();
        cpu.register[0x05] = 0x01;
        cpu.register[0x06] = 0x00;
        cpu.rsb(5, 6);
        assert_eq!(cpu.register[0x05], 0xFF);
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_shl() {
        let mut cpu = Cpu::new();
        cpu.register[0x07] = 0xF1;
        cpu.shl(7);
        assert_eq!(cpu.register[0x07], 0xE2);
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_skne_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0) = 0x01;
        *cpu.reg_mut(1) = 0x01;
        cpu.skne_reg(0, 1);
        assert_eq!(cpu.pc, 0x200);

        *cpu.reg_mut(2) = 0x02;
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
        *cpu.reg_mut(0) = 0x10;
        cpu.jmi(0xF00);
        assert_eq!(cpu.pc, 0xF10);
    }

    #[test]
    fn test_rand() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0xB) = 0x10;
        cpu.rand(0xB, 0x0F);
        assert_ne!(cpu.reg(0xB), 0x10);
        assert_eq!(cpu.reg(0xB) & 0xF0, 0x00);
    }

    #[test]
    fn test_bcd() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0xB) = 123;
        cpu.i = 0x300;
        cpu.bcd(0xB);
        assert_eq!(cpu.mem(0x300), 1);
        assert_eq!(cpu.mem(0x301), 2);
        assert_eq!(cpu.mem(0x302), 3);
    }

    #[test]
    fn test_adi() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(7) = 0x10;
        cpu.i = 0x01;
        cpu.adi(7);
        assert_eq!(cpu.i, 0x10 + 0x01);
    }
}
