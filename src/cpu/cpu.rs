﻿mod opcode;

use std::collections::HashMap;

const PROGRAM_BEGIN_LOC: u16 = 0x8000;
const RESET_INTERRUPT_MEM_LOC: u16 = 0xFFFC;

const MEMORY_CAP: usize = 0xFFFF;

#[derive(Debug)]
pub enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing
}

pub struct Memory {
    raw_mem: [u8; MEMORY_CAP]
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            raw_mem: [0u8; MEMORY_CAP]
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.raw_mem[addr as usize]
    }

    // little-endian
    pub fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.raw_mem[addr as usize] = data;
    }

    pub fn write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.write(addr, lo);
        self.write(addr + 1, hi);
    }

    pub fn load(&mut self, bytes:[u8; MEMORY_CAP]) {
        self.raw_mem = bytes;
    }
}

/*
http://wiki.nesdev.com/w/index.php/Status_flags

    7  bit  0
    ---- ----
    NV_B DIZC
    || | ||||
    || | |||+- Carry
    || | ||+-- Zero
    || | |+--- Interrupt Disable
    || | +---- Decimal
    || +------ Break
    ||
    |+-------- Overflow
    +--------- Negative
*/

bitflags::bitflags! {
    pub struct CPUStatus: u8 {
        const NEGATIVE          = 0b1000_0000;
        const OVERFLOW          = 0b0100_0000;
        const UNDEFINED         = 0b0010_0000;
        const BREAK             = 0b0001_0000;
        const DECIMAL           = 0b0000_1000;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const ZERO              = 0b0000_0010;
        const CARRY             = 0b1000_0001;
    }
}

pub struct CPU {
    pc: u16,
    sp: u8,
    acc: u8,
    rx: u8,
    ry: u8,
    status: CPUStatus,
    mem: Memory
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            sp: 0,
            acc: 0,
            rx: 0,
            ry: 0,
            status: CPUStatus::from_bits_truncate(0b0010_0100),
            mem: Memory::new()
        }
    }

    fn get_operand_address(&self, mode: &AddressMode) -> u16 {
        match mode {
            AddressMode::Immediate => {
                self.pc
            }
            AddressMode::ZeroPage => {
                self.mem.read(self.pc) as u16
            }
            AddressMode::ZeroPageX => {
                let pos = self.mem.read(self.pc);
                pos.wrapping_add(self.rx) as u16
            }
            AddressMode::ZeroPageY => {
                let pos = self.mem.read(self.pc);
                pos.wrapping_add(self.ry) as u16
            }
            AddressMode::Absolute => {
                self.mem.read_u16(self.pc)
            }
            AddressMode::AbsoluteX => {
                let pos = self.mem.read_u16(self.pc);
                pos.wrapping_add(self.rx as u16) as u16
            }
            AddressMode::AbsoluteY => {
                let pos = self.mem.read_u16(self.pc);
                pos.wrapping_add(self.ry as u16) as u16
            }
            AddressMode::IndirectX => {
                let base = self.mem.read(self.pc);
                let ptr = (base as u8).wrapping_add(self.rx) as u8;
                let lo = self.mem.read(ptr as u16);
                let hi = self.mem.read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressMode::IndirectY => {
                let base = self.mem.read(self.pc);
                let lo = self.mem.read(base as u16);
                let hi = self.mem.read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                deref_base.wrapping_add(self.ry as u16)
            }
            AddressMode::NoneAddressing => {
                panic!("not support for {:?}", mode)
            }
        }
    }

    fn update_zero_flag(&mut self, flag: u8) {
        if flag == 0 {
            self.status.insert(CPUStatus::ZERO);
        } else {
            self.status.remove(CPUStatus::ZERO);
        }
    }

    fn update_neg_flag(&mut self, flag: u8) {
        if flag & 0b1000_0000 != 0 {
            self.status.insert(CPUStatus::NEGATIVE);
        } else {
            self.status.remove(CPUStatus::NEGATIVE);
        }
    }

    fn update_overflow_flag(&mut self, flag: bool) {
        if flag {
            self.status.insert(CPUStatus::OVERFLOW);
        } else {
            self.status.remove(CPUStatus::OVERFLOW);
        }
    }

    fn update_carry_flag(&mut self, flag: bool) {
        if flag {
            self.status.insert(CPUStatus::CARRY);
        } else {
            self.status.remove(CPUStatus::CARRY);
        }
    }

    fn lda(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem.read(addr);

        self.acc = value;
        self.update_neg_flag(value);
        self.update_zero_flag(value);
    }

    fn sta(&mut self, mode: &AddressMode) {
        let addr = self.get_operand_address(mode);
        self.mem.write(addr, self.acc);
    }

    fn tax(&mut self) {
        self.rx = self.acc;
        self.update_neg_flag(self.rx);
        self.update_zero_flag(self.rx);
    }

    pub fn load(&mut self, bytes:[u8; 0xFFFF]) {
        self.mem.load(bytes);
        self.mem.write_u16(RESET_INTERRUPT_MEM_LOC, PROGRAM_BEGIN_LOC);
    }

    pub fn reset(&mut self) {
        self.acc = 0;
        self.rx = 0;
        self.status = CPUStatus::from_bits_truncate(0b0010_0100);

        self.pc = self.mem.read_u16(RESET_INTERRUPT_MEM_LOC);
    }

    pub fn run(&mut self) {
        self.reset();
        self.interprect();
    }

    pub fn interprect(&mut self) -> () {
        let ref opcodes: HashMap<u8, &'static opcode::Opcode> = *opcode::OPCODES_MAP;
        loop {
            let op = self.mem.read(self.pc);
            self.pc += 1;
            let pc_state = self.pc;

            let code = opcodes.get(&op).expect(&format!("op: {:x} not exists or not impl.", op));

            match op {
                0x00 => {
                    return;
                }
                0xAA => {
                    self.tax();
                }
                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&code.mode);
                }
                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&code.mode);
                }
                _ => {

                }
            }

            if pc_state == self.pc {
                self.pc += (code.bytes - 1) as u16
            }
        }
    }
}