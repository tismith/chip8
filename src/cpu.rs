//! The CHIP-8 CPU emulation and instruction set

use rand;
use std;

///The core CPU registers and memory
pub struct Cpu {
    register: [u8; 16],
    delay: u8,
    sound: u8,
    ///actually 12 bits
    i: u16,
    ///actually 12 bits, pointer into `memory`
    pc: u16,
    sp: Vec<u16>,
    key: [bool; 16],
    unknown_key: bool,
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    memory: [u8; 4096],
}

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const TIMER_FREQUENCY: usize = 60;

const INITIAL_PC: u16 = 0x200;
const INSTRUCTION_WIDTH: u16 = 2;
const FONTSET_ADDRESS: u16 = 0x50;
const FONTSET: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Cpu {
    ///lookup a mutable key register
    pub fn key_mut(&mut self, keycode: u8) -> &mut bool {
        if keycode < 0x0F {
            &mut self.key[usize::from(keycode)]
        } else {
            warn!("unexpected keycode {}", keycode);
            &mut self.unknown_key
        }
    }

    ///new, initialized cpu
    pub fn new() -> Self {
        Default::default()
    }

    ///copies the rom into memory
    pub fn load_rom(&mut self, rom: &[u8]) {
        for (i, byte) in rom.iter().enumerate() {
            *self.mem_mut(INITIAL_PC + i as u16) = *byte;
        }
    }

    ///decrements timers, returns true if the buzzer needs to sound
    pub fn tick_timers(&mut self) -> bool {
        let mut make_sound = false;
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound == 1 {
            make_sound = true;
        }
        if self.sound > 0 {
            self.sound -= 1;
        }
        make_sound
    }

    ///runs a single instruction, from PC
    pub fn tick(&mut self) {
        let opcode = (u16::from(self.mem(self.pc)) << 8) + u16::from(self.mem(self.pc + 1));
        let address = opcode & 0x0FFF;
        let value = (opcode & 0x00FF) as u8;
        let reg = ((opcode >> 8) & 0x000F) as u8;
        let x = ((opcode >> 8) & 0x000F) as u8;
        let y = ((opcode >> 4) & 0x000F) as u8;
        let n = (opcode & 0x000F) as u8;
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.cls(),
                0x00EE => self.rts(),
                _ => error!("unmatched opcode! {}", opcode),
            },
            0x1000 => self.jmp(address),
            0x2000 => self.jsr(address),
            0x3000 => self.skeq_const(reg, value),
            0x4000 => self.skne_const(reg, value),
            0x5000 => self.skeq_reg(x, y),
            0x6000 => self.mov_const(reg, value),
            0x7000 => self.add_const(reg, value),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.mov_reg(x, y),
                0x0001 => self.or_reg(x, y),
                0x0002 => self.and_reg(x, y),
                0x0003 => self.xor_reg(x, y),
                0x0004 => self.add_reg(x, y),
                0x0005 => self.sub_reg(x, y),
                0x0006 => self.shr(x, y),
                0x0007 => self.rsb(x, y),
                0x000E => self.shl(x, y),
                _ => error!("unmatched opcode! {}", opcode),
            },
            0x9000 => self.skne_reg(x, y),
            0xA000 => self.mvi(address),
            0xB000 => self.jmi(address),
            0xC000 => self.rand(reg, value),
            0xD000 => self.sprite(x, y, n),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.skpr(x),
                0x00A1 => self.skup(x),
                _ => error!("unmatched opcode! {}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.gdelay(x),
                0x000A => self.key(x),
                0x0015 => self.sdelay(x),
                0x0018 => self.ssound(x),
                0x001E => self.adi(x),
                0x0029 => self.font(x),
                0x0033 => self.bcd(x),
                0x0055 => self.str(x),
                0x0065 => self.ldr(x),
                _ => error!("unmatched opcode! {}", opcode),
            },
            _ => error!("unmatched opcode! {}", opcode),
        }
    }

    ///returns a slice of the screen
    pub fn screen(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.screen
    }

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

    ///0x00E0
    ///clear the screen
    fn cls(&mut self) {
        for x in self.screen.iter_mut() {
            *x = false;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x00EE
    ///return from subroutine
    fn rts(&mut self) {
        if let Some(address) = self.sp.pop() {
            self.pc = address;
        } else {
            error!("rts called with no return address");
        }
    }

    ///0x1NNN (NNN is the address)
    ///jump to address
    fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    ///0x2NNN (NNN is the address)
    ///jump to subroutine
    fn jsr(&mut self, address: u16) {
        self.sp.push(self.pc.wrapping_add(INSTRUCTION_WIDTH));
        self.pc = address;
    }

    ///0x3XRR
    ///skeq - skip next instruction if register VX == constant RR
    fn skeq_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.reg(register_id);
        if reg == constant {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x4XRR
    ///skne - skip next intruction if register VX != constant RR
    fn skne_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.reg(register_id);
        if reg != constant {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x5XY0
    ///skeq - skip next instruction if register VX == register VY
    fn skeq_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let x = self.reg(register_x_id);
        let y = self.reg(register_y_id);
        if x == y {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x6XRR
    ///mov - move constant RR to register VX
    fn mov_const(&mut self, register_x_id: u8, constant: u8) {
        {
            let reg = self.reg_mut(register_x_id);
            *reg = constant;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x7XRR
    ///add = add constant RR to register VX
    ///No carry generated
    fn add_const(&mut self, register_id: u8, constant: u8) {
        {
            let reg = self.reg_mut(register_id);
            *reg = reg.wrapping_add(constant);
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x8XY0
    ///mov_reg move register VY into VX
    fn mov_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        {
            let y = self.reg(register_y_id);
            let x = self.reg_mut(register_x_id);
            *x = y;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x8XY1
    ///or register VY with register VX, store result into register VX
    fn or_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        {
            let y = self.reg(register_y_id);
            let x = self.reg_mut(register_x_id);
            *x |= y;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x8XY2
    ///and register VY with register VX, store result into register VX
    fn and_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        {
            let y = self.reg(register_y_id);
            let x = self.reg_mut(register_x_id);
            *x &= y;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x8XY3
    ///xor register VY with register VX, store result into register VX
    fn xor_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        {
            let y = self.reg(register_y_id);
            let x = self.reg_mut(register_x_id);
            *x ^= y;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///0x8XY4
    ///add_reg add register VY to VX, store result in register VX,
    ///carry stored in register VF
    fn add_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg(register_x_id);
        let (result, overflow) = x.overflowing_add(y);
        if overflow {
            self.register[0x0F] = 0x01;
        }
        *self.reg_mut(register_x_id) = result;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///8XY5
    ///sub vx,vy subtract register VY from VX, borrow stored in register VF
    ///register VF set to 1 if borrows
    fn sub_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg(register_x_id);
        let (result, borrow) = x.overflowing_sub(y);
        if !borrow {
            self.register[0x0F] = 0x01;
        }
        *self.reg_mut(register_x_id) = result;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///8XY6 shr vx  shift register VX right, bit 0 goes into register VF
    fn shr(&mut self, register_x_id: u8, _register_y_id: u8) {
        let x = self.reg(register_x_id);
        self.register[0x0F] = x & 0x01;
        //*self.reg_mut(register_y_id) = x >> 1;
        *self.reg_mut(register_x_id) = x >> 1;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///8XY7 rsb vx,vy   subtract register VX from register VY
    ///result stored in register VX
    ///register F set to 1 if borrows
    fn rsb(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.reg(register_y_id);
        let x = self.reg(register_x_id);
        let (result, borrow) = y.overflowing_sub(x);
        if !borrow {
            self.register[0x0F] = 0x01;
        }
        *self.reg_mut(register_x_id) = result;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///8XYE shl vx  shift register VX left, bit 7 stored into register VF
    fn shl(&mut self, register_x_id: u8, _register_y_id: u8) {
        let x = self.reg(register_x_id);
        if x & 0x80 != 0 {
            self.register[0x0F] = 0x01;
        }
        //*self.reg_mut(register_y_id) = x << 1;
        *self.reg_mut(register_x_id) = x << 1;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///9XY0 skne vx,vy  skip next instruction
    ///if register VX != register VY
    fn skne_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let x = self.reg(register_x_id);
        let y = self.reg(register_y_id);
        if x != y {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///ANNN mvi nnn Load index register (I) with constant NNN
    fn mvi(&mut self, value: u16) {
        self.i = value & 0xFFF;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///BNNN jmi nnn Jump to address NNN + register V0
    fn jmi(&mut self, value: u16) {
        self.pc = u16::from(self.reg(0)).wrapping_add(value & 0xFFF);
    }

    ///CXKK rand vx,kk register VX = random number AND KK
    fn rand(&mut self, register_x_id: u8, value: u8) {
        *self.reg_mut(register_x_id) = rand::random::<u8>() & value;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///DXYN sprite vx,vy,n  Draw sprite at screen location
    ///(register VX,register VY) height N
    ///Sprites stored in memory at location in index register (I),
    ///maximum 8bits wide. Wraps around
    ///the screen. If when drawn, clears a pixel,
    ///register VF is set to 1 otherwise it is zero. All
    ///drawing is XOR drawing (e.g. it toggles the screen pixels)
    fn sprite(&mut self, register_x_id: u8, register_y_id: u8, num_lines: u8) {
        let x = usize::from(self.reg(register_x_id));
        let y = usize::from(self.reg(register_y_id));
        let mut index = 0;
        for line in 0..num_lines {
            let sprite_row = self.mem(self.i + u16::from(line));
            for i in 0..8 {
                let sprite_pixel = (sprite_row << i) & 0x80;
                if sprite_pixel != 0 {
                    let sprite_x = (x + (index % 8)) % SCREEN_WIDTH;
                    let sprite_y = (y + (index / 8)) % SCREEN_HEIGHT;
                    let pixel_address = sprite_y * SCREEN_WIDTH + sprite_x;
                    let current_pixel = self.screen[pixel_address];
                    if current_pixel {
                        *self.reg_mut(0xf) = 0x01;
                    }
                    self.screen[pixel_address] = !current_pixel;
                }
                index += 1;
            }
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///ek9e skpr k  skip if key (register rk) pressed
    ///The key is a key number, see the chip-8
    ///documentation
    fn skpr(&mut self, key_id: u8) {
        let key = self.reg(key_id);
        if key > 0x0F {
            error!("invalid key id {}", key);
        } else if self.key[usize::from(key)] {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///eka1 skup k  skip if key (register rk) not pressed
    fn skup(&mut self, key_id: u8) {
        let key = self.reg(key_id);
        if key > 0x0F {
            error!("invalid key id {}", key);
        } else if !self.key[usize::from(key)] {
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fr07 gdelay vr   get delay timer into vr
    fn gdelay(&mut self, register_x_id: u8) {
        *self.reg_mut(register_x_id) = self.delay;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fr0a key vr  wait for for keypress,put key in register vr
    fn key(&mut self, register_x_id: u8) {
        if let Some((key, _)) = self.key.iter().enumerate().find(|(_, &p)| p) {
            *self.reg_mut(register_x_id) = key as u8;
            self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
        }
    }

    ///fr15 sdelay vr   set the delay timer to vr
    fn sdelay(&mut self, register_x_id: u8) {
        self.delay = self.reg(register_x_id);
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fr18 ssound vr   set the sound timer to vr
    fn ssound(&mut self, register_x_id: u8) {
        self.sound = self.reg(register_x_id);
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fr1e adi vr  add register vr to the index register
    fn adi(&mut self, register_x_id: u8) {
        self.i = self.i.wrapping_add(u16::from(self.reg(register_x_id)));
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fr29 font vr point I to the sprite for hexadecimal
    ///character in vr   Sprite is 5 bytes high
    fn font(&mut self, register_x_id: u8) {
        self.i = FONTSET_ADDRESS + u16::from(self.reg(register_x_id)) * 5;
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fr33 bcd vr  store the bcd representation of register vr
    ///at location I,I+1,I+2
    ///Doesn't change I
    fn bcd(&mut self, register_x_id: u8) {
        if self.i >= (4096 - 3) {
            error!("bcd called with I too large: {}", self.i);
        } else {
            let x = self.reg(register_x_id);
            let x100 = x / 100;
            let x10 = (x - (x100 * 100)) / 10;
            let x1 = x - (x100 * 100) - (x10 * 10);
            let i = self.i;
            *self.mem_mut(i) = x100;
            *self.mem_mut(i + 1) = x10;
            *self.mem_mut(i + 2) = x1;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fr55 str v0-vr   store registers v0-vr at location I onwards
    ///I is incremented to point to
    ///the next location on. e.g. I = I + r + 1
    fn str(&mut self, register_x_id: u8) {
        let r = self.reg(register_x_id);
        let bound = std::cmp::min(r, 0x0F);
        for i in 0..=bound {
            self.memory[usize::from(self.i)] = self.reg(i);
            self.i += 1;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }

    ///fx65 ldr v0-vr   load registers v0-vr from location I onwards
    ///as above.
    fn ldr(&mut self, register_x_id: u8) {
        let r = self.reg(register_x_id);
        let bound = std::cmp::min(r, 0x0F);
        for i in 0..=bound {
            *self.reg_mut(i) = self.memory[usize::from(self.i)];
            self.i += 1;
        }
        self.pc = self.pc.wrapping_add(INSTRUCTION_WIDTH);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        let mut cpu = Cpu {
            register: [0; 16],
            delay: 0,
            sound: 0,
            i: 0,
            pc: INITIAL_PC,
            sp: Vec::new(),
            key: [false; 16],
            unknown_key: false,
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            memory: [0; 4096],
        };

        cpu.memory[usize::from(FONTSET_ADDRESS)..(usize::from(FONTSET_ADDRESS) + FONTSET.len())]
            .copy_from_slice(&FONTSET);
        cpu
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cls() {
        let mut cpu = Cpu::new();
        cpu.screen[0] = true;
        cpu.screen[SCREEN_WIDTH * SCREEN_HEIGHT - 1] = true;
        cpu.cls();
        assert_eq!(cpu.screen[0], false);
        assert_eq!(cpu.screen[SCREEN_WIDTH * SCREEN_HEIGHT - 1], false);
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
        assert_eq!(cpu.pc, 0x204);

        cpu.skeq_const(0, 0x02);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_skne_const() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0) = 0x01;
        cpu.skne_const(0, 0x01);
        assert_eq!(cpu.pc, 0x202);

        cpu.skne_const(0, 0x02);
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_skeq_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0) = 0x01;
        *cpu.reg_mut(1) = 0x01;
        cpu.skeq_reg(0, 1);
        assert_eq!(cpu.pc, 0x204);

        *cpu.reg_mut(2) = 0x02;
        cpu.skeq_reg(1, 2);
        assert_eq!(cpu.pc, 0x206);
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
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_sub_reg_underflow() {
        let mut cpu = Cpu::new();
        cpu.register[0x05] = 0x00;
        cpu.register[0x06] = 0x01;
        cpu.sub_reg(5, 6);
        assert_eq!(cpu.register[0x05], 0xFF);
        assert_eq!(cpu.register[0x0F], 0x00);
    }

    #[test]
    fn test_shr() {
        let mut cpu = Cpu::new();
        cpu.register[0x07] = 0xF1;
        cpu.shr(7, 8);
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
        assert_eq!(cpu.register[0x0F], 0x00);
    }

    #[test]
    fn test_shl() {
        let mut cpu = Cpu::new();
        cpu.register[0x07] = 0xF1;
        cpu.shl(7, 8);
        assert_eq!(cpu.register[0x07], 0xE2);
        assert_eq!(cpu.register[0x0F], 0x01);
    }

    #[test]
    fn test_skne_reg() {
        let mut cpu = Cpu::new();
        *cpu.reg_mut(0) = 0x01;
        *cpu.reg_mut(1) = 0x01;
        cpu.skne_reg(0, 1);
        assert_eq!(cpu.pc, 0x202);

        *cpu.reg_mut(2) = 0x02;
        cpu.skne_reg(1, 2);
        assert_eq!(cpu.pc, 0x206);
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
