use crate::{memory::Memory, instruction::{Operand, Register, Flags, OperandSize}};

pub enum Value {
	I64(i64),
	I32(i32),
	XMM(u128),
	U8(u8),
}
use Value::*;

impl From<f32> for Value {
	fn from(value: f32) -> Value { XMM(value.to_bits() as u128) }
}

impl From<Value> for u128 {
	fn from(value: Value) -> u128 { match value {
		Value::I64(value) => value as u64 as u128, // /!\ zero extension
		Value::I32(value) => value as u64 as u128, // /!\ zero extension
		Value::XMM(value) => value,
		_ => unreachable!(),
	}}
}

impl From<Value> for i64 {
	fn from(value: Value) -> i64 { match value {
		Value::I64(value) => value as i64,
		Value::I32(value) => value as i64, // /!\ sign extend
		Value::XMM(value) => value as i64, // /!\ truncate
		_ => unreachable!(),
	}}
}

impl From<Value> for i32 {
	fn from(value: Value) -> i32 { match value {
		//Value::I64(value) => value,
		Value::I32(value) => value,
		Value::XMM(value) => value as i32, // /!\ truncate
		_ => unreachable!("{:?}", value),
	}}
}

impl From<Value> for f32 {
	fn from(value: Value) -> f32 { match value {
		Value::XMM(value) => f32::from_bits(value as u32),
		_ => unreachable!(),
	}}
}

impl From<Value> for u16 {
	fn from(value: Value) -> u16 { match value {
		//Value::I64(value) => value,
		//Value::I32(value) => value as i64, // /!\ sign extend
		//Value::XMM(value) => value as i64, // /!\ truncate
		_ => unreachable!(),
	}}
}


impl From<Value> for u8 {
	fn from(value: Value) -> u8 { match value {
		//Value::I64(value) => value,
		//Value::I32(value) => value as i64, // /!\ sign extend
		//Value::XMM(value) => value as i64, // /!\ truncate
		_ => unreachable!(),
	}}
}
/*impl From<Value> for u128 {
	fn from(value: Value) { match value {
		Value::XMM(value) => value,
		_ => unreachable!(),
	}}
}*/

impl std::fmt::Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Value::I64(value) => write!(f, "{}", value),
			Value::I32(value) => write!(f, "{}", value),
			Value::XMM(value) => write!(f, "{}", f32::from_bits(*value as u32)),
			Value::U8(value) => write!(f, "{}", value),
		}
	}
}

pub struct State {
	pub rip: i64,
	pub rax: i64, pub rbx: i64, pub rcx: i64, pub rdx: i64, pub rsp: i64, pub rbp: i64, pub rsi: i64, pub rdi: i64,
	pub r8: i64, pub r9: i64, pub r10: i64, pub r11: i64, pub r12: i64, pub r13: i64, pub r14: i64, pub r15: i64,
	pub rflags: i64,
	pub cr0: i64, pub cr2: i64, pub cr4: i64, pub cr8: i64,
	pub gdt: i64, pub idt: i64,
	pub xmm: [u128; 16],

	pub memory: Memory,
	pub print_instructions: bool,
}

impl State {
    pub fn new() -> Self { Self{
        rip: 0,
        rax: 0, rbx: 0, rcx: 0, rdx: 0, rsp: 0, rbp: 0, rsi: 0, rdi: 0,
        r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0,
        rflags: 0,
        cr0: 0, cr2: 0, cr4: 0, cr8: 0,
        gdt: 0, idt: 0,
        xmm: [0; 16],
        memory: Default::default(),
        print_instructions: false,
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
            OperandSize::Bit128 => (result as u64) & 0x8000000000000000 != 0, // fixme
        };
        self.set_flag(Flags::Sign, sign);

        let byte = result as u8;
        let mut parity = 0;
        for i in 0..8 {
            parity ^= (byte >> i) & 0b1
        }
        self.set_flag(Flags::Parity, parity != 0b1)
    }

    #[track_caller] pub fn get_value(&self, arg: &Operand, operand_size: OperandSize) -> i64 {
        match *arg {
            Operand::Register(register) => self.get_register_value(register),
            Operand::Immediate(immediate) => immediate,
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
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn get_xmm(&self, arg: &Operand, operand_size: OperandSize) -> u128 {
        match *arg {
            Operand::Register(register) => self.get_register_xmm(register),
            Operand::Immediate(_) => unimplemented!(),//immediate,
            Operand::EffectiveAddress { .. } => {
                let address = self.calculate_effective_address(arg);
                match operand_size {
                    /*OperandSize::Bit8 => self.memory.read_byte(address) as u64,
                    OperandSize::Bit16 => {
                        let value: u16 = self.memory.read_unaligned(address);
                        value as u128
                    }
                    OperandSize::Bit32 => {
                        let value: u32 = self.memory.read_unaligned(address);
                        value as u128
                    }
                    OperandSize::Bit64 => {
                        let value: u64 = self.memory.read_unaligned(address);
                        value as u128
                    }*/
                    OperandSize::Bit128 => {
                        let value: u128 = self.memory.read_unaligned(address);
                        value
                    },
                    _ => unimplemented!(),
                }
            }
        }
    }

    /*#[track_caller] pub fn get_value_or_xmm(&self, arg: &Operand, operand_size: OperandSize) -> u128 {
			match *arg {
				Operand::Register { register } => self.get_register_value_or_xmm(register),
				//Operand::Immediate { immediate } => immediate,
				Operand::EffectiveAddress { .. } => {
						let address = self.calculate_effective_address(arg);
						match operand_size {
								/*OperandSize::Bit8 => self.memory.read_byte(address) as i64,
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
								_ => unreachable!(),*/
								OperandSize::Bit128 => {
										let value: u128 = self.memory.read_unaligned(address);
										value
								},
								_ => unimplemented!(),
						}
				}
				_ => unreachable!(),
			}
    }*/

    #[track_caller] pub fn get_register_value(&self, register: Register) -> i64 {
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

            _ => panic!("Expected integer register"),
        }
    }

    pub fn get_register_xmm(&self, register: Register) -> u128 {
			match register {
				Register::XMM0 => self.xmm[0],
				Register::XMM1 => self.xmm[1],
				Register::XMM2 => self.xmm[2],
				Register::XMM3 => self.xmm[3],
				Register::XMM4 => self.xmm[4],
				Register::XMM5 => self.xmm[5],
				Register::XMM6 => self.xmm[6],
				Register::XMM7 => self.xmm[7],
				Register::XMM8 => self.xmm[8],
				Register::XMM9 => self.xmm[9],
				Register::XMM10 => self.xmm[10],
				Register::XMM11 => self.xmm[11],
				Register::XMM12 => self.xmm[12],
				Register::XMM13 => self.xmm[13],
				Register::XMM14 => self.xmm[14],
				Register::XMM15 => self.xmm[15],
				_ => panic!("Expected XMM register"),
			}
		}

		/*#[track_caller] pub fn get_register_value_or_xmm(&self, register: Register) -> u128 {
        match register {
            Register::RAX => self.rax as u128,
            Register::RBX => self.rbx as u128,
            Register::RCX => self.rcx as u128,
            Register::RDX => self.rdx as u128,
            Register::RSP => self.rsp as u128,
            Register::RBP => self.rbp as u128,
            Register::RSI => self.rsi as u128,
            Register::RDI => self.rdi as u128,

            Register::R8 => self.r8 as u128,
            Register::R9 => self.r9 as u128,
            Register::R10 => self.r10 as u128,
            Register::R11 => self.r11 as u128,
            Register::R12 => self.r12 as u128,
            Register::R13 => self.r13 as u128,
            Register::R14 => self.r14 as u128,
            Register::R15 => self.r15 as u128,

            // 32 Bit
            Register::EAX => self.rax as i32 as u128,
            Register::EBX => self.rbx as i32 as u128,
            Register::ECX => self.rcx as i32 as u128,
            Register::EDX => self.rdx as i32 as u128,
            Register::ESP => self.rsp as i32 as u128,
            Register::EBP => self.rbp as i32 as u128,
            Register::ESI => self.rsi as i32 as u128,
            Register::EDI => self.rdi as i32 as u128,

            Register::R8D => self.r8 as i32 as u128,
            Register::R9D => self.r9 as i32 as u128,
            Register::R10D => self.r10 as i32 as u128,
            Register::R11D => self.r11 as i32 as u128,
            Register::R12D => self.r12 as i32 as u128,
            Register::R13D => self.r13 as i32 as u128,
            Register::R14D => self.r14 as i32 as u128,
            Register::R15D => self.r15 as i32 as u128,

						Register::XMM0 => self.xmm[0],
						Register::XMM1 => self.xmm[1],
						Register::XMM2 => self.xmm[2],
						Register::XMM3 => self.xmm[3],
						Register::XMM4 => self.xmm[4],
						Register::XMM5 => self.xmm[5],
						Register::XMM6 => self.xmm[6],
						Register::XMM7 => self.xmm[7],
						Register::XMM8 => self.xmm[8],
						Register::XMM9 => self.xmm[9],
						Register::XMM10 => self.xmm[10],
						Register::XMM11 => self.xmm[11],
						Register::XMM12 => self.xmm[12],
						Register::XMM13 => self.xmm[13],
						Register::XMM14 => self.xmm[14],
						Register::XMM15 => self.xmm[15],
						_ => unreachable!("{}", register),
				}
		}*/

		#[track_caller] pub fn register(&self, register: Register) -> Value {
			use Value::*;
        match register {
            Register::RAX => I64(self.rax),
            Register::RBX => I64(self.rbx),
            Register::RCX => I64(self.rcx),
            Register::RDX => I64(self.rdx),
            Register::RSP => I64(self.rsp),
            Register::RBP => I64(self.rbp),
            Register::RSI => I64(self.rsi),
            Register::RDI => I64(self.rdi),
            Register::R8 => I64(self.r8),
            Register::R9 => I64(self.r9),
            Register::R10 => I64(self.r10),
            Register::R11 => I64(self.r11),
            Register::R12 => I64(self.r12),
            Register::R13 => I64(self.r13),
            Register::R14 => I64(self.r14),
            Register::R15 => I64(self.r15),

            Register::EAX => I32(self.rax as i32),
            Register::EBX => I32(self.rbx as i32),
            Register::ECX => I32(self.rcx as i32),
            Register::EDX => I32(self.rdx as i32),
            Register::ESP => I32(self.rsp as i32),
            Register::EBP => I32(self.rbp as i32),
            Register::ESI => I32(self.rsi as i32),
            Register::EDI => I32(self.rdi as i32),
            Register::R8D => I32(self.r8 as i32),
            Register::R9D => I32(self.r9 as i32),
            Register::R10D => I32(self.r10 as i32),
            Register::R11D => I32(self.r11 as i32),
            Register::R12D => I32(self.r12 as i32),
            Register::R13D => I32(self.r13 as i32),
            Register::R14D => I32(self.r14 as i32),
            Register::R15D => I32(self.r15 as i32),

						Register::XMM0 => XMM(self.xmm[0]),
						Register::XMM1 => XMM(self.xmm[1]),
						Register::XMM2 => XMM(self.xmm[2]),
						Register::XMM3 => XMM(self.xmm[3]),
						Register::XMM4 => XMM(self.xmm[4]),
						Register::XMM5 => XMM(self.xmm[5]),
						Register::XMM6 => XMM(self.xmm[6]),
						Register::XMM7 => XMM(self.xmm[7]),
						Register::XMM8 => XMM(self.xmm[8]),
						Register::XMM9 => XMM(self.xmm[9]),
						Register::XMM10 => XMM(self.xmm[10]),
						Register::XMM11 => XMM(self.xmm[11]),
						Register::XMM12 => XMM(self.xmm[12]),
						Register::XMM13 => XMM(self.xmm[13]),
						Register::XMM14 => XMM(self.xmm[14]),
						Register::XMM15 => XMM(self.xmm[15]),

						// low 8bit
						Register::AL => U8(self.rax as u8),
            Register::CL => U8(self.rcx as u8),
            Register::DL => U8(self.rdx as u8),
            Register::BL => U8(self.rbx as u8),
            Register::SPL => U8(self.rsp as u8),
            Register::BPL => U8(self.rbp as u8),
            Register::SIL => U8(self.rsi as u8),
            Register::DIL => U8(self.rdi as u8),
            Register::R8B => U8(self.r8 as u8),
            Register::R9B => U8(self.r9 as u8),
            Register::R10B => U8(self.r10 as u8),
            Register::R11B => U8(self.r11 as u8),
            Register::R12B => U8(self.r12 as u8),
            Register::R13B => U8(self.r13 as u8),
            Register::R14B => U8(self.r14 as u8),
            Register::R15B => U8(self.r15 as u8),
						_ => unreachable!("{}", register),
				}
		}

		#[track_caller] pub fn get(&self, arg: &Operand, operand_size: OperandSize) -> Value {
			use Value::*;
			match *arg {
				Operand::Register(register) => self.register(register),
				Operand::Immediate(immediate) => I64(immediate),
				Operand::EffectiveAddress { .. } => {
						let address = self.calculate_effective_address(arg);
						match operand_size {
								/*OperandSize::Bit8 => self.memory.read_byte(address) as i64,
								OperandSize::Bit16 => {
										let value: i16 = self.memory.read_unaligned(address);
										value as i64
								}*/
								OperandSize::Bit32 => I32(self.memory.read_unaligned(address)),
								OperandSize::Bit64 => I64(self.memory.read_unaligned(address)), // fixme
								OperandSize::Bit128 => XMM(self.memory.read_unaligned(address)),
								_ => panic!("{:?}", operand_size),
						}
				}
				//_ => unreachable!(),
			}
    }


    #[track_caller] pub fn set_register_value(&mut self, register: Register, value: i64) {
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

            _ => panic!("Expected integer register"),
        }
    }

    #[track_caller] pub fn set_register_xmm(&mut self, register: Register, value: u128) {
			match register {
				Register::XMM0 => self.xmm[0] = value,
				Register::XMM1 => self.xmm[1] = value,
				Register::XMM2 => self.xmm[2] = value,
				Register::XMM3 => self.xmm[3] = value,
				Register::XMM4 => self.xmm[4] = value,
				Register::XMM5 => self.xmm[5] = value,
				Register::XMM6 => self.xmm[6] = value,
				Register::XMM7 => self.xmm[7] = value,
				Register::XMM8 => self.xmm[8] = value,
				Register::XMM9 => self.xmm[9] = value,
				Register::XMM10 => self.xmm[10] = value,
				Register::XMM11 => self.xmm[11] = value,
				Register::XMM12 => self.xmm[12] = value,
				Register::XMM13 => self.xmm[13] = value,
				Register::XMM14 => self.xmm[14] = value,
				Register::XMM15 => self.xmm[15] = value,

				_ => panic!("set_register_xmm: Expected xmm register"),
			}
		}

		pub fn set_register_value_or_xmm(&mut self, register: Register, value: u128) {
        match register {
            // 64 Bit
            Register::RAX => self.rax = value as u64 as i64,
            Register::RBX => self.rbx = value as u64 as i64,
            Register::RCX => self.rcx = value as u64 as i64,
            Register::RDX => self.rdx = value as u64 as i64,
            Register::RSP => self.rsp = value as u64 as i64,
            Register::RBP => self.rbp = value as u64 as i64,
            Register::RSI => self.rsi = value as u64 as i64,
            Register::RDI => self.rdi = value as u64 as i64,

            Register::R8 => self.r8 = value as u64 as i64,
            Register::R9 => self.r9 = value as u64 as i64,
            Register::R10 => self.r10 = value as u64 as i64,
            Register::R11 => self.r11 = value as u64 as i64,
            Register::R12 => self.r12 = value as u64 as i64,
            Register::R13 => self.r13 = value as u64 as i64,
            Register::R14 => self.r14 = value as u64 as i64,
            Register::R15 => self.r15 = value as u64 as i64,

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

						Register::XMM0 => self.xmm[0] = value,
						Register::XMM1 => self.xmm[1] = value,
						Register::XMM2 => self.xmm[2] = value,
						Register::XMM3 => self.xmm[3] = value,
						Register::XMM4 => self.xmm[4] = value,
						Register::XMM5 => self.xmm[5] = value,
						Register::XMM6 => self.xmm[6] = value,
						Register::XMM7 => self.xmm[7] = value,
						Register::XMM8 => self.xmm[8] = value,
						Register::XMM9 => self.xmm[9] = value,
						Register::XMM10 => self.xmm[10] = value,
						Register::XMM11 => self.xmm[11] = value,
						Register::XMM12 => self.xmm[12] = value,
						Register::XMM13 => self.xmm[13] = value,
						Register::XMM14 => self.xmm[14] = value,
						Register::XMM15 => self.xmm[15] = value,

						_ => panic!("set_register_value_or_xmm: Expected xmm or integer register"),
				}
		}

		pub fn set_register(&mut self, register: Register, value: Value) {
        match register {
            // 64 Bit
            Register::RAX => self.rax = value.into(),
            Register::RBX => self.rbx = value.into(),
            Register::RCX => self.rcx = value.into(),
            Register::RDX => self.rdx = value.into(),
            Register::RSP => self.rsp = value.into(),
            Register::RBP => self.rbp = value.into(),
            Register::RSI => self.rsi = value.into(),
            Register::RDI => self.rdi = value.into(),

            Register::R8 => self.r8 = value.into(),
            Register::R9 => self.r9 = value.into(),
            Register::R10 => self.r10 = value.into(),
            Register::R11 => self.r11 = value.into(),
            Register::R12 => self.r12 = value.into(),
            Register::R13 => self.r13 = value.into(),
            Register::R14 => self.r14 = value.into(),
            Register::R15 => self.r15 = value.into(),

            // 32 Bit
            Register::EAX => self.rax = value.into(),
            Register::EBX => self.rbx = value.into(),
            Register::ECX => self.rcx = value.into(),
            Register::EDX => self.rdx = value.into(),
            Register::ESP => self.rsp = value.into(),
            Register::EBP => self.rbp = value.into(),
            Register::ESI => self.rsi = value.into(),
            Register::EDI => self.rdi = value.into(),

            Register::R8D => self.r8 = value.into(),
            Register::R9D => self.r9 = value.into(),
            Register::R10D => self.r10 = value.into(),
            Register::R11D => self.r11 = value.into(),
            Register::R12D => self.r12 = value.into(),
            Register::R13D => self.r13 = value.into(),
            Register::R14D => self.r14 = value.into(),
            Register::R15D => self.r15 = value.into(),

						Register::XMM0 => self.xmm[0] = value.into(),
						Register::XMM1 => self.xmm[1] = value.into(),
						Register::XMM2 => self.xmm[2] = value.into(),
						Register::XMM3 => self.xmm[3] = value.into(),
						Register::XMM4 => self.xmm[4] = value.into(),
						Register::XMM5 => self.xmm[5] = value.into(),
						Register::XMM6 => self.xmm[6] = value.into(),
						Register::XMM7 => self.xmm[7] = value.into(),
						Register::XMM8 => self.xmm[8] = value.into(),
						Register::XMM9 => self.xmm[9] = value.into(),
						Register::XMM10 => self.xmm[10] = value.into(),
						Register::XMM11 => self.xmm[11] = value.into(),
						Register::XMM12 => self.xmm[12] = value.into(),
						Register::XMM13 => self.xmm[13] = value.into(),
						Register::XMM14 => self.xmm[14] = value.into(),
						Register::XMM15 => self.xmm[15] = value.into(),

						_ => panic!("set_register_value_or_xmm: Expected xmm or integer register"),
				}
		}

		#[track_caller] pub fn set_value(&mut self, value: i64, arg: &Operand, operand_size: OperandSize) {
        match *arg {
            Operand::Register(register) => { self.set_register_value(register, value) },
            Operand::EffectiveAddress { .. } => {
                let address = self.calculate_effective_address(arg);
                match operand_size {
                    OperandSize::Bit8   => self.memory.write(address, &(value as i8)),
                    OperandSize::Bit16 => self.memory.write_unaligned(address, &(value as i16)),
                    OperandSize::Bit32 => self.memory.write_unaligned(address, &(value as i32)),
                    OperandSize::Bit64 => self.memory.write_unaligned(address, &(value as i64)),
                    _ => unreachable!(),
                }
            },
            Operand::Immediate { .. } => panic!("Cannot set value on immediate value"),
        }
    }

    /*pub fn set_xmm(&mut self, value: u128, arg: &Operand, operand_size: OperandSize) {
        match *arg {
            Operand::Register { register } => { self.set_register_xmm(register, value) },
            Operand::EffectiveAddress { .. } => {
                let address = self.calculate_effective_address(arg);
                match operand_size {
                    OperandSize::Bit8   => self.memory.write(address, &(value as u8)),
                    OperandSize::Bit16 => self.memory.write_unaligned(address, &(value as u16)),
                    OperandSize::Bit32 => self.memory.write_unaligned(address, &(value as u32)),
                    OperandSize::Bit64 => self.memory.write_unaligned(address, &(value as u64)),
                    OperandSize::Bit128 => self.memory.write_unaligned(address, &value),
                }
            },
            Operand::Immediate { .. } => panic!("Cannot set value on immediate value"),
        }
    }*/
    pub fn set_xmm(&mut self, value: Value, arg: &Operand, operand_size: OperandSize) {
			self.set(value, arg, operand_size);
    }

    /*pub fn set_value_or_xmm(&mut self, value: u128, arg: &Operand, operand_size: OperandSize) {
        match *arg {
            Operand::Register { register } => { self.set_register_value_or_xmm(register, value) },
            Operand::EffectiveAddress { .. } => {
                let address = self.calculate_effective_address(arg);
                match operand_size {
                    OperandSize::Bit8   => self.memory.write(address, &(value as u8)),
                    OperandSize::Bit16 => self.memory.write_unaligned(address, &(value as u16)),
                    OperandSize::Bit32 => self.memory.write_unaligned(address, &(value as u32)),
                    OperandSize::Bit64 => self.memory.write_unaligned(address, &(value as u64)),
                    OperandSize::Bit128 => self.memory.write_unaligned(address, &value),
                }
            },
            Operand::Immediate { .. } => panic!("Cannot set value on immediate value"),
        }
    }*/

    pub fn set(&mut self, value: Value, arg: &Operand, operand_size: OperandSize) {
        match *arg {
            Operand::Register(register) => { self.set_register(register, value) },
            Operand::EffectiveAddress { .. } => {
                let address = self.calculate_effective_address(arg);
                match operand_size {
                    OperandSize::Bit8   => self.memory.write(address, &(value.into():u8)),
                    OperandSize::Bit16 => self.memory.write_unaligned(address, &(value.into():u16)),
                    OperandSize::Bit32 => self.memory.write_unaligned(address, &(value.into():i32)),
                    OperandSize::Bit64 => self.memory.write_unaligned(address, &(value.into():i64)),
                    //OperandSize::Bit128 => self.memory.write_unaligned(address, &(value.into():u128)),
                    _ => panic!("{:?}", operand_size),
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

