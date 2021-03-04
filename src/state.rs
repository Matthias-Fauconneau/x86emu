use crate::{memory::Memory, instruction::{Operand, Register, Flags, OperandSize}};

/*#[derive(PartialEq)]
pub struct Location<'t> { pub file: &'t str, pub line: Option<u32> }
impl std::fmt::Display for Location<'_> { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    f.write_str(self.file)?;
    if let Some(line) = self.line { write!(f, ":{}", line)? }
    Ok(())
}}*/

pub struct State {
    pub rip: i64,
    pub rax: i64, pub rbx: i64, pub rcx: i64, pub rdx: i64, pub rsp: i64, pub rbp: i64, pub rsi: i64, pub rdi: i64,
    pub r8: i64, pub r9: i64, pub r10: i64, pub r11: i64, pub r12: i64, pub r13: i64, pub r14: i64, pub r15: i64,
    pub rflags: i64,
    pub cr0: i64, pub cr2: i64, /*cr3: memory.cr3,*/ pub cr4: i64, pub cr8: i64,
    pub gdt: i64, pub idt: i64,

    pub memory: Memory,
    // Kept in execution context to avoid passing to every instruction execution functions
    pub print_instructions: bool,
    //pub find_location: Box<dyn Fn(u64)->Option<Location<'static>>>
}

impl State {
    pub fn new(/*find_location: Box<dyn Fn(u64)->Option<Location<'static>>>*/) -> Self { Self{
        rip: 0,
        rax: 0, rbx: 0, rcx: 0, rdx: 0, rsp: 0, rbp: 0, rsi: 0, rdi: 0,
        r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0,
        rflags: 0,
        cr0: 0, cr2: 0, /*cr3: memory.cr3,*/ cr4: 0, cr8: 0,
        gdt: 0, idt: 0,
        memory: Default::default(),
        print_instructions: false,
        //find_location
    } }

    pub fn get_flag(&self, flag: Flags) -> bool {
        let f = flag as i64;
        self.rflags & f == f
    }

    pub fn set_flag(&mut self, flag: Flags, value: bool) {
        if value {
            self.rflags |= flag as i64;
        } else {
            self.rflags &= !(flag as i64);
        }
    }

    pub fn compute_flags(&mut self, result: i64, operand_size: OperandSize) {
        self.set_flag(Flags::Zero, result == 0);
        let sign = match operand_size {
            OperandSize::Bit8 => (result as u64) & 0x80 != 0,
            OperandSize::Bit16 => (result as u64) & 0x8000 != 0,
            OperandSize::Bit32 => (result as u64) & 0x80000000 != 0,
            OperandSize::Bit64 => (result as u64) & 0x8000000000000000 != 0,
        };
        self.set_flag(Flags::Sign, sign);


        let byte = result as u8;
        let mut parity = 0;
        for i in 0..8 {
            parity ^= (byte >> i) & 0b1
        }
        self.set_flag(Flags::Parity, parity != 0b1)
    }

    pub fn get_value(&self, arg: &Operand, operand_size: OperandSize) -> i64 {
        match *arg {
            Operand::Register { register } => self.get_register_value(register),
            Operand::Immediate { immediate } => immediate,
            Operand::EffectiveAddress { .. } => {
                let address = self.calculate_effective_address(arg);
                match operand_size {
                    OperandSize::Bit8 => self.memory.read_byte(address) as i64,
                    OperandSize::Bit16 => {
                        let value: i16 = self.memory.read_unaligned(address);
                        value as i64
                    }
                    OperandSize::Bit32 => {
                        let value: i32 = self.memory.read_unaligned(address);
                        value as i64
                    }
                    OperandSize::Bit64 => {
                        let value: i64 = self.memory.read_unaligned(address);
                        value
                    }
                }
            }
        }
    }

    pub fn get_register_value(&self, register: Register) -> i64 {
        match register {
            Register::RAX => self.rax,
            Register::RBX => self.rbx,
            Register::RCX => self.rcx,
            Register::RDX => self.rdx,
            Register::RSP => self.rsp,
            Register::RBP => self.rbp,
            Register::RSI => self.rsi,
            Register::RDI => self.rdi,

            Register::R8 => self.r8,
            Register::R9 => self.r9,
            Register::R10 => self.r10,
            Register::R11 => self.r11,
            Register::R12 => self.r12,
            Register::R13 => self.r13,
            Register::R14 => self.r14,
            Register::R15 => self.r15,

            Register::CR0 => self.cr0,
            Register::CR2 => self.cr2,
            Register::CR3 => unimplemented!(), //self.memory.cr3,
            Register::CR4 => self.cr4,
            Register::CR8 => self.cr8,

            Register::RIP => self.rip as i64,

            // 32 Bit
            Register::EAX => self.rax as i32 as i64,
            Register::EBX => self.rbx as i32 as i64,
            Register::ECX => self.rcx as i32 as i64,
            Register::EDX => self.rdx as i32 as i64,
            Register::ESP => self.rsp as i32 as i64,
            Register::EBP => self.rbp as i32 as i64,
            Register::ESI => self.rsi as i32 as i64,
            Register::EDI => self.rdi as i32 as i64,

            Register::R8D => self.r8 as i32 as i64,
            Register::R9D => self.r9 as i32 as i64,
            Register::R10D => self.r10 as i32 as i64,
            Register::R11D => self.r11 as i32 as i64,
            Register::R12D => self.r12 as i32 as i64,
            Register::R13D => self.r13 as i32 as i64,
            Register::R14D => self.r14 as i32 as i64,
            Register::R15D => self.r15 as i32 as i64,

            // 16 Bit
            Register::AX => self.rax as i16 as i64,
            Register::BX => self.rbx as i16 as i64,
            Register::CX => self.rcx as i16 as i64,
            Register::DX => self.rdx as i16 as i64,
            Register::SP => self.rsp as i16 as i64,
            Register::BP => self.rbp as i16 as i64,
            Register::SI => self.rsi as i16 as i64,
            Register::DI => self.rdi as i16 as i64,

            Register::R8W => self.r8 as i16 as i64,
            Register::R9W => self.r9 as i16 as i64,
            Register::R10W => self.r10 as i16 as i64,
            Register::R11W => self.r11 as i16 as i64,
            Register::R12W => self.r12 as i16 as i64,
            Register::R13W => self.r13 as i16 as i64,
            Register::R14W => self.r14 as i16 as i64,
            Register::R15W => self.r15 as i16 as i64,

            // 8 Bit
            Register::AL => self.rax as i8 as i64,
            Register::CL => self.rcx as i8 as i64,
            Register::DL => self.rdx as i8 as i64,
            Register::BL => self.rbx as i8 as i64,
            Register::AH => (self.rax as i16 >> 8) as i64,
            Register::CH => (self.rcx as i16 >> 8) as i64,
            Register::DH => (self.rdx as i16 >> 8) as i64,
            Register::BH => (self.rbx as i16 >> 8) as i64,

            Register::R8B => self.r8 as i8 as i64,
            Register::R9B => self.r9 as i8 as i64,
            Register::R10B => self.r10 as i8 as i64,
            Register::R11B => self.r11 as i8 as i64,
            Register::R12B => self.r12 as i8 as i64,
            Register::R13B => self.r13 as i8 as i64,
            Register::R14B => self.r14 as i8 as i64,
            Register::R15B => self.r15 as i8 as i64,

            Register::SPL => self.rsp as i8 as i64,
            Register::BPL => self.rbp as i8 as i64,
            Register::SIL => self.rsi as i8 as i64,
            Register::DIL => self.rdi as i8 as i64,

            Register::ES => 0,
            Register::CS => 0,
            Register::SS => 0,
            Register::DS => 0,
            Register::FS => 0,
            Register::GS => 0,
        }
    }

    pub fn set_register_value(&mut self, register: Register, value: i64) {
        match register {
            // 64 Bit
            Register::RAX => self.rax = value,
            Register::RBX => self.rbx = value,
            Register::RCX => self.rcx = value,
            Register::RDX => self.rdx = value,
            Register::RSP => self.rsp = value,
            Register::RBP => self.rbp = value,
            Register::RSI => self.rsi = value,
            Register::RDI => self.rdi = value,

            Register::R8 => self.r8 = value,
            Register::R9 => self.r9 = value,
            Register::R10 => self.r10 = value,
            Register::R11 => self.r11 = value,
            Register::R12 => self.r12 = value,
            Register::R13 => self.r13 = value,
            Register::R14 => self.r14 = value,
            Register::R15 => self.r15 = value,

            Register::CR0 => {
                println!("CR0: {:x}", value);
                self.cr0 = value
            },
            Register::CR2 => {
                println!("CR2: {:x}", value);
                self.cr2 = value
            },
            Register::CR3 => {
                println!("CR3: {:x}", value);
                unimplemented!(); //self.memory.cr3 = value
            },
            Register::CR4 => {
                println!("CR4: {:x}", value);
                self.cr4 = value
            },
            Register::CR8 => {
                println!("CR5: {:x}", value);
                self.cr8 = value
            },

            Register::RIP => self.rip = value,

            // 32 Bit
            Register::EAX => self.rax = value as u32 as u64 as i64,
            Register::EBX => self.rbx = value as u32 as u64 as i64,
            Register::ECX => self.rcx = value as u32 as u64 as i64,
            Register::EDX => self.rdx = value as u32 as u64 as i64,
            Register::ESP => self.rsp = value as u32 as u64 as i64,
            Register::EBP => self.rbp = value as u32 as u64 as i64,
            Register::ESI => self.rsi = value as u32 as u64 as i64,
            Register::EDI => self.rdi = value as u32 as u64 as i64,

            Register::R8D => self.r8 = value as u32 as u64 as i64,
            Register::R9D => self.r9 = value as u32 as u64 as i64,
            Register::R10D => self.r10 = value as u32 as u64 as i64,
            Register::R11D => self.r11 = value as u32 as u64 as i64,
            Register::R12D => self.r12 = value as u32 as u64 as i64,
            Register::R13D => self.r13 = value as u32 as u64 as i64,
            Register::R14D => self.r14 = value as u32 as u64 as i64,
            Register::R15D => self.r15 = value as u32 as u64 as i64,

            // 16 Bit
            Register::AX => self.rax = ((self.rax as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::BX => self.rbx = ((self.rbx as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::CX => self.rcx = ((self.rcx as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::DX => self.rdx = ((self.rdx as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::SP => self.rsp = ((self.rsp as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::BP => self.rbp = ((self.rbp as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::SI => self.rsi = ((self.rsi as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::DI => self.rdi = ((self.rdi as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,

            Register::R8W => self.r8 = ((self.r8 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::R9W => self.r9 = ((self.r9 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::R10W => self.r10 = ((self.r10 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::R11W => self.r11 = ((self.r11 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::R12W => self.r12 = ((self.r12 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::R13W => self.r13 = ((self.r13 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::R14W => self.r14 = ((self.r14 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,
            Register::R15W => self.r15 = ((self.r15 as u64 & 0xFFFFFFFFFFFF0000) | (value as u16 as u64)) as i64,

            // 8 Bit
            Register::AL => self.rax = ((self.rax as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::CL => self.rcx = ((self.rcx as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::DL => self.rdx = ((self.rdx as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::BL => self.rbx = ((self.rbx as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::AH => self.rax = ((self.rax as u64 & 0xFFFFFFFFFFFF00FF) |
                            ((value as u8 as u64) << 8)) as i64,
            Register::CH => self.rcx = ((self.rcx as u64 & 0xFFFFFFFFFFFF00FF) |
                            ((value as u8 as u64) << 8)) as i64,
            Register::DH => self.rdx = ((self.rdx as u64 & 0xFFFFFFFFFFFF00FF) |
                            ((value as u8 as u64) << 8)) as i64,
            Register::BH => self.rbx = ((self.rbx as u64 & 0xFFFFFFFFFFFF00FF) |
                            ((value as u8 as u64) << 8)) as i64,

            Register::R8B => self.r8 = ((self.r8 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::R9B => self.r9 = ((self.r9 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::R10B => self.r10 = ((self.r10 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::R11B => self.r11 = ((self.r11 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::R12B => self.r12 = ((self.r12 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::R13B => self.r13 = ((self.r13 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::R14B => self.r14 = ((self.r14 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::R15B => self.r15 = ((self.r15 as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,

            Register::SPL => self.rsp = ((self.rsp as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::BPL => self.rbp = ((self.rbp as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::SIL => self.rsi = ((self.rsi as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,
            Register::DIL => self.rdi = ((self.rdi as u64 & 0xFFFFFFFFFFFFFF00) | (value as u8 as u64)) as i64,

            Register::ES => (),
            Register::CS => (),
            Register::SS => (),
            Register::DS => (),
            Register::FS => (),
            Register::GS => (),
        }
    }

    pub fn set_value(&mut self, value: i64, arg: &Operand, operand_size: OperandSize) {
        match *arg {
            Operand::Register { register } => { self.set_register_value(register, value) },
            Operand::EffectiveAddress { .. } => {
                let address = self.calculate_effective_address(arg);
                match operand_size {
                    OperandSize::Bit8   => self.memory.write(address, &(value as i8)),
                    OperandSize::Bit16 => self.memory.write_unaligned(address, &(value as i16)),
                    OperandSize::Bit32 => self.memory.write_unaligned(address, &(value as i32)),
                    OperandSize::Bit64 => self.memory.write_unaligned(address, &(value as i64)),
                }
            },
            Operand::Immediate { .. } => panic!("Cannot set value on immediate value"),
        }
    }

    pub fn calculate_effective_address(&self, arg: &Operand) -> u64 {
        match *arg {
            Operand::EffectiveAddress { ref base, ref index, scale, displacement} => {
                let mut address = match *base {
                    Some(base) => self.get_register_value(base),
                    None => 0,
                };
                address += match *index {
                    None => 0,
                    Some(index) => self.get_register_value(index) * scale.unwrap() as i64,
                };
                address += displacement as i64;
                address as u64
            }
            _ => unreachable!(),
        }
    }
}

