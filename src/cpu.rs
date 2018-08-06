pub struct Cpu {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v_a: u8,
    v_b: u8,
    v_c: u8,
    v_d: u8,
    v_e: u8,
    v_f: u8,
    ///actually 12 bits
    i: u16,
    ///actually 12 bits, pointer into `memory`
    pc: u16, 
    sp: Vec<u16>,
    memory: [u8; 4096],
}

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
            0xA => self.v_a,
            0xB => self.v_b,
            0xC => self.v_c,
            0xD => self.v_d,
            0xE => self.v_e,
            0xF => self.v_f,
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
            0xA => &mut self.v_a,
            0xB => &mut self.v_b,
            0xC => &mut self.v_c,
            0xD => &mut self.v_d,
            0xE => &mut self.v_e,
            0xF => &mut self.v_f,
            _ => panic!("unexpected register id {}", register),
        }
    }

    ///new, initialized cpu
    pub fn new() -> Self {
        Default::default()
    }

    ///clear the screen
    ///0x00E0
    pub fn cls(&mut self) {
        unimplemented!()
    }

    ///return from subroutine
    ///0x00EE
    pub fn rts(&mut self) {
        if let Some(address) = self.sp.pop() {
            self.pc = address;
        } else {
            panic!("rts called with no return address");
        }
    }

    ///jump to address
    ///0x1NNN (NNN is the address)
    pub fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    ///jump to subroutine
    ///0x2NNN (NNN is the address)
    pub fn jsr(&mut self, address: u16) {
        self.sp.push(self.pc + 1);
        self.pc = address;
    }

    ///skeq - skip next instruction if register VX == constant RR
    ///0x3XRR
    pub fn skeq_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.id_to_reg(register_id);
        if reg == constant {
            self.pc += 1;
        }
    }

    ///skne - skip next intruction if register VX != constant RR
    ///0x4XRR
    pub fn skne(&mut self, register_id: u8, constant: u8) {
        let reg = self.id_to_reg(register_id);
        if reg != constant {
            self.pc += 1;
        }
    }

    ///skeq - skip next instruction if register VX == register VY
    ///0x5XY0
    pub fn skeq_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let x = self.id_to_reg(register_x_id);
        let y = self.id_to_reg(register_y_id);
        if x != y {
            self.pc += 1;
        }
    }

    ///mov - move constant RR to register VX
    ///0x6XRR
    pub fn mov_const(&mut self, register_x_id: u8, constant: u8) {
        let reg = self.id_to_reg_mut(register_x_id);
        *reg = constant;
    }

    ///add = add constant RR to register VX
    ///0x7XRR
    pub fn add_const(&mut self, register_id: u8, constant: u8) {
        let reg = self.id_to_reg_mut(register_id);
        *reg += constant;
    }

    ///mov_reg move register VY into VX
    ///0x8XY0
    pub fn mov_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x = y;
    }

    ///or register VY with register VX, store result into register VX
    ///0x8XY1
    pub fn or_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x = *x | y;
    }

    ///and register VY with register VX, store result into register VX
    ///0x8XY2
    pub fn and_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x = *x & y;
    }

    ///xor register VY with register VX, store result into register VX
    ///0x8XY3
    pub fn xor_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x = *x ^ y;
    }

    ///add_reg add register VY to VX, store result in register VX,
    ///carry stored in register VF
    ///0x8XY4
    pub fn add_reg(&mut self, register_x_id: u8, register_y_id: u8) {
        let y = self.id_to_reg(register_y_id);
        let x = self.id_to_reg_mut(register_x_id);
        *x = *x + y;
        //TODO catch carry to stick in v_f
        unimplemented!()
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
            v_a: 0,
            v_b: 0,
            v_c: 0,
            v_d: 0,
            v_e: 0,
            v_f: 0,
            i: 0,
            pc: 0x200,
            sp: Vec::new(),
            memory: [0; 4096],
        }
    }
}

