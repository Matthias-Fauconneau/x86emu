use bitflags::bitflags;
use crate::{memory::Memory, instruction::{Register, RegisterSize, OperandSize, Opcode, Repeat, Operand, Operands}};

#[derive(PartialEq)] enum RegOrOpcode { Register, Opcode, }
#[derive(PartialEq)] enum ImmediateSize { None, Bit8, Bit32, }
bitflags! {
	struct REX: u8 {
		const B = 0b00000001;
		const X = 0b00000010;
		const R = 0b00000100;
		const W = 0b00001000;
	}
}
bitflags! {
	struct Flags: u64 {
		const REVERSED_REGISTER_DIRECTION = 1 /*<< 0*/;
		const ADDRESS_SIZE_OVERRIDE = 1 << 2;
		const NEW_64BIT_REGISTER = 1 << 5;
		const NEW_8BIT_REGISTER = 1 << 6;
		const MOD_R_M_EXTENSION = 1 << 7;
		const SIB_EXTENSION = 1 << 8;
		const OPERAND_16_BIT = 1 << 9;
		const OPERAND_64_BIT = 1 << 10;
		const SIB_DISPLACEMENT_ONLY = 1 << 11;
		const OP1_XMM = 1 << 12;
		const OP2_XMM = 1 << 13;
	}
}

pub fn decode(rip : &mut i64, memory : &Memory) -> (Opcode, Operands) {
	let mut flags = Flags { bits: 0 };
	let mut repeat = Repeat::None;
	loop {
		match memory.read_byte(*rip as u64) {
			0xF0 => { /* todo: do not ignore lock/bound prefix */ }
			0xF2 => { repeat = Repeat::NotEqual }
			0xF3 => { repeat = Repeat::Equal; }
			0x2E | 0x3E | 0x36 | 0x26 | 0x64 | 0x65 => { /* TODO: do not ignore segment prefix (or probably we should?) */ }
			0x66 => { flags |= Flags::OPERAND_16_BIT; }
			0x67 => { flags |= Flags::ADDRESS_SIZE_OVERRIDE; }
			bits @ 0x40..=0x4F => { // 64bit REX prefix
				let rex = REX{bits};
				if rex.contains(REX::B) { flags |= Flags::NEW_64BIT_REGISTER; }
				if rex.contains(REX::R) { flags |= Flags::MOD_R_M_EXTENSION; }
				if rex.contains(REX::X) { flags |= Flags::SIB_EXTENSION; }
				if rex.contains(REX::W) { flags |= Flags::OPERAND_64_BIT;  }
				flags |= Flags::NEW_8BIT_REGISTER;
			}
			_ => break,
		}
		*rip += 1;
	}

	let register_size = if flags.contains(Flags::OPERAND_64_BIT) {
			RegisterSize::Bit64
	} else if flags.contains(Flags::OPERAND_16_BIT) {
			RegisterSize::Bit16
	} else {
			RegisterSize::Bit32
	};

	macro_rules! Opcode { ($($op:ident)+) => ( [$(Opcode::$op),+] ) }
	let jcc = Opcode!(Jo Jno Jb Jae Je Jne Jbe Ja Js Jns Jp Jnp Jl Jge Jle Jg);
	let scc = Opcode!(Seto Setno Setb Setae Sete Setne Setbe Seta Sets Setns Setp Setnp Setl Setge Setle Setg);
	match memory.read_byte(*rip as u64) {
			0x00 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::Add, op)
			}
			0x01 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::Add, op)
			}
			0x02 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Add, op)
			}
			0x03 => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Add, op)
			}
			0x04 => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Add, op)
			}
			0x05 => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Add, op)
			}
			0x08 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::Or, op)
			}
			0x09 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::Or, op)
			}
			0x0A => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Or, op)
			}
			0x0B => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Or, op)
			}
			0x0C => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Or, op)
			}
			0x0D => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Or, op)
			}
			0x10 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::Adc, op)
			}
			0x11 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::Adc, op)
			}
			0x12 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Adc, op)
			}
			0x13 => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Adc, op)
			}
			0x14 => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Adc, op)
			}
			0x15 => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Adc, op)
			}
			0x18 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::Sbb, op)
			}
			0x19 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::Sbb, op)
			}
			0x1A => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Sbb, op)
			}
			0x1B => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Sbb, op)
			}
			0x1C => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Sbb, op)
			}
			0x1D => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Sbb, op)
			}
			0x20 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::And, op)
			}
			0x21 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::And, op)
			}
			0x22 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::And, op)
			}
			0x23 => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::And, op)
			}
			0x24 => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::And, op)
			}
			0x25 => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::And, op)
			}
			0x28 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::Sub, op)
			}
			0x29 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::Sub, op)
			}
			0x2A => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Sub, op)
			}
			0x2B => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Sub, op)
			}
			0x2C => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Sub, op)
			}
			0x2D => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Sub, op)
			}
			0x30 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::Xor, op)
			}
			0x31 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::Xor, op)
			}
			0x32 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Xor, op)
			}
			0x33 => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Xor, op)
			}
			0x34 => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Xor, op)
			}
			0x35 => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Xor, op)
			}
			0x38 => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags);
					(Opcode::Cmp, op)
			}
			0x39 => {
					let op = decode_reg_reg(memory, rip, register_size, flags);
					(Opcode::Cmp, op)
			}
			0x3A => {
					let op = decode_8bit_reg_8bit_immediate(memory, rip, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Cmp, op)
			}
			0x3B => {
					let op = decode_reg_reg(memory, rip, register_size, flags | Flags::REVERSED_REGISTER_DIRECTION);
					(Opcode::Cmp, op)
			}
			0x3C => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Cmp, op)
			}
			0x3D => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Cmp, op)
			}
			opcode @ 0x50..=0x57 => {
					*rip += 1;
					(Opcode::Push, Operands{ operands: (Some(Operand::Register{ register: get_register(opcode - 0x50, RegisterSize::Bit64,
																																																																													flags.contains(Flags::NEW_64BIT_REGISTER),
																																																																													flags.contains(Flags::NEW_8BIT_REGISTER)) }),
																																						None, None), ..Default::default() })
			}
			opcode @ 0x58..=0x5F => {
					*rip += 1;
					(Opcode::Pop, Operands{ operands: (Some(Operand::Register{ register: get_register(opcode - 0x58, RegisterSize::Bit64,
																																																																													flags.contains(Flags::NEW_64BIT_REGISTER),
																																																																													flags.contains(Flags::NEW_8BIT_REGISTER)) }),
																																						None, None), ..Default::default() })
			}
			0x63 => {
					let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
					override_operand_size(&memory, *rip, &mut op, OperandSize::Bit32, &flags);
					*rip += ip_offset;
					(Opcode::Movsx, op)
			}
			0x68 => {
					let immediate = if flags.contains(Flags::OPERAND_16_BIT) {
							let immediate = memory.get_i16(*rip, 1) as i64;
							*rip += 3;
							immediate
					} else {
							let immediate = memory.get_i32(*rip, 1) as i64;
							*rip += 5;
							immediate
					};
					(Opcode::Push, Operands{ operands: (Some(Operand::Immediate{ immediate }), None, None), ..Default::default() })
			}
			0x69 => {
					let (mut op, ip_offset) = get_operands(&memory, *rip, register_size, RegOrOpcode::Register, ImmediateSize::None,
																																					flags | Flags::REVERSED_REGISTER_DIRECTION);
					*rip += ip_offset;
					let immediate = if flags.contains(Flags::OPERAND_16_BIT) {
							let immediate = memory.get_i16(*rip, 0) as i64;
							*rip += 2;
							immediate
					} else {
							let immediate = memory.get_i32(*rip, 0) as i64;
							*rip += 4;
							immediate
					};
					op.operands.2 = op.operands.1;
					op.operands.1 = op.operands.0;
					op.operands.0 = Some(Operand::Immediate{ immediate });
					(Opcode::Imul, op)
			}
			0x6A => (Opcode::Push, read_immediate_8bit(memory, rip)),
			0x6B => {
					let (mut op, ip_offset) = get_operands(memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
					*rip += ip_offset;
					let immediate = memory.get_i8(*rip, 0) as i64;
					op.operands.2 = op.operands.1;
					op.operands.1 = op.operands.0;
					op.operands.0 = Some(Operand::Immediate{ immediate });
					*rip += 1;
					(Opcode::Imul, op)
			}
			opcode @ 0x70..=0x7F => { (jcc[(opcode-0x70) as usize], read_immediate_8bit(memory, rip)) }
			0x80 => {
					// arithmetic operation (8bit register target, 8bit immediate)
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																													RegOrOpcode::Opcode,
																													ImmediateSize::Bit8,
																													flags);
					*rip += ip_offset;
					(Opcode::Arithmetic, op)
			}
			0x81 => {
					// arithmetic operation (32/64bit register target, 32bit immediate)
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Opcode,
																													ImmediateSize::Bit32,
																													flags);
					*rip += ip_offset;
					(Opcode::Arithmetic, op)
			}
			0x83 => {
					// arithmetic operation (32/64bit register target, 8bit immediate)
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Opcode,
																													ImmediateSize::Bit8,
																													flags);
					*rip += ip_offset;
					(Opcode::Arithmetic, op)
			}
			0x84 => {
					// test
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags);
					*rip += ip_offset;
					(Opcode::Test, op)
			}
			0x85 => {
					// test
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags);
					*rip += ip_offset;
					(Opcode::Test, op)
			}
			0x86 => {
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags);
					*rip += ip_offset;
					(Opcode::Xchg, op)
			}
			0x87 => {
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags);
					*rip += ip_offset;
					(Opcode::Xchg, op)
			}
			0x88 => {
					// mov
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags);
					*rip += ip_offset;
					(Opcode::Mov, op)
			}
			0x89 => {
					// mov
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags);
					*rip += ip_offset;
					(Opcode::Mov, op)
			}
			0x8A => {
					// mov
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags | Flags::REVERSED_REGISTER_DIRECTION);
					*rip += ip_offset;
					(Opcode::Mov, op)
			}
			0x8B => {
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Register,
																													ImmediateSize::None,
																													flags |
																													Flags::REVERSED_REGISTER_DIRECTION);
					*rip += ip_offset;
					(Opcode::Mov, op)
			}
			0x8D => {
					let (op, ip_offset) =
							get_operands(&memory, *rip, register_size,
																	RegOrOpcode::Register,
																	ImmediateSize::None,
																	// TODO: REVERSED_REGISTER_DIRECTION correct?
																	flags | Flags::REVERSED_REGISTER_DIRECTION);
					*rip += ip_offset;
					(Opcode::Lea, op)
			}
			0x8E => {
					// mov 16bit segment registers
					let (op, ip_offset) =
							get_operands(&memory, *rip, RegisterSize::Segment,
																	RegOrOpcode::Register,
																	ImmediateSize::None,
																	// TODO: REVERSED_REGISTER_DIRECTION correct?
																	flags | Flags::REVERSED_REGISTER_DIRECTION);
					*rip += ip_offset;
					(Opcode::Mov, op)
			}
			0x8F => {
					let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags |
																															Flags::REVERSED_REGISTER_DIRECTION);
					op.operands.1 = None;
					*rip += ip_offset;
					(Opcode::Pop, op)
			}
			0x90 => {
					*rip += 1;
					(Opcode::Nop, Operands::default())
			}
			opcode @ 0x91..=0x97 => {
					let register1 = get_register(0, register_size, flags.contains(Flags::NEW_64BIT_REGISTER), flags.contains(Flags::NEW_8BIT_REGISTER));
					let register2 = get_register(opcode - 0x90, register_size, flags.contains(Flags::NEW_64BIT_REGISTER), flags.contains(Flags::NEW_8BIT_REGISTER));
					*rip += 1;
					(Opcode::Xchg, Operands{ operands: (Some(Operand::Register{ register: register1 }), Some(Operand::Register{ register: register2 }), None), ..Default::default() })
			}
			0x98 => {
					let (register1, register2) = if flags.contains(Flags::OPERAND_16_BIT) {
							(Register::AL, Register::AX)
					} else if flags.contains(Flags::OPERAND_64_BIT) {
							(Register::EAX, Register::RAX)
					} else {
							(Register::AX, Register::EAX)
					};
					*rip += 1;
					(Opcode::Mov, Operands{ operands: (Some(Operand::Register{register: register1}), Some(Operand::Register{register: register2}), None), ..Default::default() })
			}
			0x99 => {
					let (register1, register2) = if flags.contains(Flags::OPERAND_16_BIT) {
							(Register::AX, Register::DX)
					} else if flags.contains(Flags::OPERAND_64_BIT) {
							(Register::RAX, Register::RDX)
					} else {
							(Register::EAX, Register::EDX)
					};

					*rip += 1;
					(Opcode::Mov, Operands{ operands: (Some(Operand::Register{register: register1}), Some(Operand::Register{register: register2}), None), ..Default::default() })
			}
			0x9C => {
					*rip += 1;
					(Opcode::Pushf, Operands::default())
			}
			0x9D => {
					*rip += 1;
					(Opcode::Popf, Operands::default())
			}
			0xA4 => {
					*rip += 1;
					(Opcode::Movs, Operands{ repeat, explicit_size: Some(OperandSize::Bit8), ..Default::default()})
			}
			0xA5 => {
					let operand_size = if flags.contains(Flags::OPERAND_16_BIT) {
							OperandSize::Bit16
					} else if flags.contains(Flags::OPERAND_64_BIT) {
							OperandSize::Bit64
					} else {
							OperandSize::Bit32
					};
					*rip += 1;
					(Opcode::Movs, Operands{ repeat, explicit_size: Some(operand_size), ..Default::default() })
			}
			0xA8 => {
					let op = decode_al_immediate(memory, rip);
					(Opcode::Test, op)
			}
			0xA9 => {
					let op = decode_ax_immediate(memory, rip, register_size, flags);
					(Opcode::Test, op)
			}
			0xAA => {
					*rip += 1;
					(Opcode::Stos, Operands{ repeat, explicit_size: Some(OperandSize::Bit8), ..Default::default() })
			}
			0xAB => {
					*rip += 1;
					let operand_size = match register_size {
							RegisterSize::Bit8 => OperandSize::Bit8,
							RegisterSize::Bit16 => OperandSize::Bit16,
							RegisterSize::Bit32 => OperandSize::Bit32,
							RegisterSize::Bit64 => OperandSize::Bit64,
							_ => panic!("Unsupported register size"),
					};
					(Opcode::Stos, Operands{ repeat, explicit_size: Some(operand_size), ..Default::default() })
			}
			0xAE => {
					*rip += 1;
					(Opcode::Scas, Operands{ operands: (Some(Operand::EffectiveAddress{ base: Some(Register::RDI), index: None, scale: None, displacement: 0 }),
																																						Some(Operand::Register{ register: Register::AL }),
																																						None), repeat, ..Default::default() })
			}
			opcode @ 0xB0..=0xB7 => {
					let immediate = memory.get_u8(*rip, 1) as i64;
					*rip += 2;
					(Opcode::Mov, Operands{ operands: (Some(Operand::Immediate{ immediate: immediate as i64 }),
																																					Some(Operand::Register{ register: get_register(opcode - 0xB0, RegisterSize::Bit8,
																																																																													flags.contains(Flags::NEW_64BIT_REGISTER),
																																																																													flags.contains(Flags::NEW_8BIT_REGISTER)) }),
																																					None), ..Default::default() })
			}
			opcode @ 0xB8..=0xBF => {
					let (immediate, ip_offset) = if flags.contains(Flags::OPERAND_64_BIT) {
							(memory.get_i64(*rip, 1) as i64, 9)
					} else if flags.contains(Flags::OPERAND_16_BIT) {
							(memory.get_i16(*rip, 1) as i64, 3)
					} else {
							(memory.get_i32(*rip, 1) as i64, 5)
					};
					*rip += ip_offset;
					(Opcode::Mov, Operands{ operands: (Some(Operand::Immediate{ immediate }),
																																					Some(Operand::Register{ register: get_register(opcode - 0xB8, register_size,
																																																																													flags.contains(Flags::NEW_64BIT_REGISTER),
																																																																													flags.contains(Flags::NEW_8BIT_REGISTER)) }),
																																					None), ..Default::default() })
			}
			0xC6 => {
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																													RegOrOpcode::Opcode,
																													ImmediateSize::Bit8,
																													flags);
					*rip += ip_offset;
					(Opcode::Mov, op)
			}
			0xC7 => {
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Opcode,
																													ImmediateSize::Bit32,
																													flags);
					*rip += ip_offset;
					(Opcode::Mov, op)
			}
			0xC0 => {
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																													RegOrOpcode::Opcode,
																													ImmediateSize::Bit8,
																													flags);
					*rip += ip_offset;
					(Opcode::ShiftRotate, op)
			}
			0xC1 => {
					let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Opcode,
																													ImmediateSize::Bit8,
																													flags);
					*rip += ip_offset;
					(Opcode::ShiftRotate, op)
			}
			0xC3 => {
					(Opcode::Ret, Operands::default())
			}
			0xC9 => {
					*rip += 1;
					(Opcode::Leave, Operands::default())
			}
			0xCB => {
					(Opcode::Lret, Operands::default())
			}
			0xD1 => {
					let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Opcode,
																													ImmediateSize::None,
																													flags);
					op.operands.1 = Some(op.operands.0.unwrap());
					op.operands.0 = Some(Operand::Immediate{
							immediate: 1,
					});
					*rip += ip_offset;
					(Opcode::ShiftRotate, op)
			}
			0xD2 => {
					let (mut op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																															RegOrOpcode::Opcode,
																															ImmediateSize::None,
																															flags);
					op.operands.1 = Some(op.operands.0.unwrap());
					op.operands.0 = Some(Operand::Register{
							register: Register::CL
					});
					*rip += ip_offset;
					(Opcode::ShiftRotate, op)
			}
			0xD3 => {
					let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																													RegOrOpcode::Opcode,
																													ImmediateSize::None,
																													flags);
					let size = op.size();
					op.operands.1 = Some(op.operands.0.unwrap());
					op.operands.0 = Some(Operand::Register{
							register: Register::CL
					});
					op.explicit_size = Some(size);
					*rip += ip_offset;
					(Opcode::ShiftRotate, op)
			}
			0xEB => { (Opcode::Jmp, read_immediate_8bit(memory, rip)) }
			0xE8 => {
					let immediate = memory.get_i32(*rip, 1);
					*rip += 5;
					(Opcode::Call, Operands{ operands: (Some(Operand::Immediate{ immediate: immediate as i64 }), None, None), ..Default::default() } )
			}
			0xE9 => {
					let immediate = memory.get_i32(*rip, 1);
					*rip += 5;
					(Opcode::Jmp, Operands{ operands: (Some(Operand::Immediate{ immediate: immediate as i64 }), None, None), ..Default::default() } )
			}
			0xEE => {
					*rip += 1;
					(Opcode::Out, Operands::default())
			}
			0xF6 => {
					let modrm = memory.get_u8(*rip, 1);
					let opcode = (modrm & 0b00111000) >> 3;

					let (op, ip_offset) = match opcode {
							0 | 1 => {
									get_operands(&memory, *rip, RegisterSize::Bit8,
																			RegOrOpcode::Opcode,
																			ImmediateSize::Bit8,
																			flags)
							},
							2 | 3 => {
									get_operands(&memory, *rip, RegisterSize::Bit8,
																			RegOrOpcode::Opcode,
																			ImmediateSize::None,
																			flags)
							}
							_ => panic!("no supported"),
					};
					*rip += ip_offset;
					(Opcode::CompareMulOperation, op)
			}
			0xF7 => {
					let modrm = memory.get_u8(*rip, 1);
					let opcode = (modrm & 0b00111000) >> 3;

					let (op, ip_offset) = match opcode {
							0 | 1 => {
									// TODO: could also be 16 bit immediate
									get_operands(&memory, *rip, register_size,
																			RegOrOpcode::Opcode,
																			ImmediateSize::Bit32,
																			flags)
							},
							2 | 3 => {
									get_operands(&memory, *rip, register_size,
																			RegOrOpcode::Opcode,
																			ImmediateSize::None,
																			flags)
							},
							4 | 5 | 6 | 7 => {
									/*let register = get_register(
											0, register_size,flags.contains(NEW_64BIT_REGISTER), false);

									(OperandsBuilder::new(). (
											Operand::Register{register: register})
											.opcode(opcode)
											.finalize(),
									2)*/
									let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																																			RegOrOpcode::Opcode,
																																			ImmediateSize::None,
																																			flags);
									op.operands.1 = None;
									op.opcode = Some(opcode);
									(op, ip_offset)
							},
							_ => unreachable!()
					};
					*rip += ip_offset;
					(Opcode::CompareMulOperation, op)
			}
			0xFA => {
					// todo: implement cli instruction
					*rip += 1;
					(Opcode::Nop, Operands::default())
			}
			0xFB => {
					// todo: implement sti instruction
					*rip += 1;
					(Opcode::Nop, Operands::default())
			}
			0xFC => {
					*rip += 1;
					(Opcode::Cld, Operands::default())
			}
			0xFD => {
					*rip += 1;
					(Opcode::Std, Operands::default())
			}
			0xFE => {
					let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																															RegOrOpcode::Opcode,
																															ImmediateSize::None,
																															flags);
					*rip += ip_offset;
					if op.opcode.unwrap() > 1 { panic!("Invalid opcode"); }
					(Opcode::RegisterOperation, op)
			}
			0xFF => {
					// todo: cleanup code
					let modrm = memory.get_u8(*rip, 1);
					let opcode = (modrm & 0b00111000) >> 3;
					let register_size = if opcode == 2 || opcode == 4 {RegisterSize::Bit64} else {register_size}; // FF /2, 4 (Call/jmp near absolute indirect) implies REX.W
					let (mut op, ip_offset) =
							get_operands(&memory, *rip, register_size, RegOrOpcode::Register, ImmediateSize::None, flags | Flags::REVERSED_REGISTER_DIRECTION);
					op.operands.1 = None;
					op.opcode = Some(opcode);
					*rip += ip_offset;
					(Opcode::RegisterOperation, op)
			}
			0x0F => {
					// two byte instructions
					*rip += 1;
					match memory.get_u8(*rip, 0) {
							0x01 => {
									let modrm = memory.get_u8(*rip, 1);
									let opcode = (modrm & 0b00111000) >> 3;
									match opcode {
											2  | 3 => {
													let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																																							RegOrOpcode::Opcode,
																																							ImmediateSize::Bit32,
																																							flags | Flags::REVERSED_REGISTER_DIRECTION);
													op.operands.0 = Some(op.operands.1.unwrap());
													op.operands.1 = None;
													*rip += ip_offset - 4;
													if opcode == 2 {
															(Opcode::Lgdt, op)
													} else {
															(Opcode::Lidt, op)
													}
											},
											_ => panic!("0F 01 unsupported opcode: {:x}", opcode)
									}
							}
							0x05 => {
									*rip += 1;
									(Opcode::Syscall, Operands::default())
							}
							0x0B => {
									*rip += 1;
									(Opcode::Ud2, Operands::default())
							}
							0x10|0x28 => {
								//assert!(flags==Flags::empty() || flags==Flags::NEW_8BIT_REGISTER, "{:?}", flags);
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM); // Checkme
								*rip += ip_offset;
								(Opcode::Movps, op)
							}
							0x11|0x29 => {
								assert!(flags==Flags::empty() || flags==Flags::NEW_8BIT_REGISTER);
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM | Flags::REVERSED_REGISTER_DIRECTION); // Checkme
								*rip += ip_offset;
								(Opcode::Movps, op)
							}
							0x1F => {
									// NOP with hint
									let (_, ip_offset) = get_operands(&memory, *rip, register_size,
																																	RegOrOpcode::Register,
																																	ImmediateSize::None,
																																	flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Nop, Operands::default())
							}
							0x20 => {
									let (mut op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit64,
																																	RegOrOpcode::Register,
																																	ImmediateSize::None,
																																	flags);
									let register = match op.operands.0.unwrap() {
											Operand::Register { register } => {
													match register {
															Register::R8 => Register::CR8,
															Register::RAX => Register::CR0,
															Register::RDX => Register::CR2,
															Register::RBX => Register::CR3,
															Register::RSP => Register::CR4,
															_ => panic!("Invalid operand for mov r64, CRn instruciton"),
													}
											},
											_ => panic!("Invalid operand for mov r64, CRn instruciton"),
									};
									op.operands.0 = Some(Operand::Register{ register });
									*rip += ip_offset;
									(Opcode::Mov, op)
							},
							0x22 => {
									let (mut op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit64,
																																	RegOrOpcode::Register,
																																	ImmediateSize::None,
																																	flags | Flags::REVERSED_REGISTER_DIRECTION);
									let register = match op.operands.1.unwrap() {
											Operand::Register { register } => {
													match register {
															Register::R8 => Register::CR8,
															Register::RAX => Register::CR0,
															Register::RDX => Register::CR2,
															Register::RBX => Register::CR3,
															Register::RSP => Register::CR4,
															_ => panic!("Invalid operand for mov r64, CRn instruciton"),
													}
											},
											_ => panic!("Invalid operand for mov r64, CRn instruciton"),
									};
									op.operands.1 = Some(Operand::Register { register });
									*rip += ip_offset;
									(Opcode::Mov, op)
							},
							0x2A => {
								let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM); // checkme
								*rip += ip_offset;
								(Opcode::Cvtpi2ps, op)
							}
							0x2C => {
								let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::Cvttps2pi, op)
							}
							0x30 => {
									*rip += 1;
									(Opcode::Wrmsr, Operands::default())
							}
							0x32 => {
									*rip += 1;
									(Opcode::Rdmsr, Operands::default())
							}
							0x40 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovo, op)
							},
							0x41 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovno, op)
							},
							0x42 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovb, op)
							},
							0x43 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovae, op)
							},
							0x44 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmove, op)
							},
							0x45 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovne, op)
							},
							0x46 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovbe, op)
							},
							0x47 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmova, op)
							},
							0x48 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovs, op)
							},
							0x49 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovns, op)
							},
							0x4a => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovp, op)
							},
							0x4b => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovnp, op)
							},
							0x4c => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovl, op)
							},
							0x4d => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovge, op)
							},
							0x4e => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovle, op)
							},
							0x4f => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Cmovg, op)
							},
							0x55 => {
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::And, op)
							}
							0x56 => {
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::Or, op)
							},
							0x57 => {
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::Xor, op)
							},
							0x58 => {
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::Fadd, op)
							},
							0x59 => {
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::Fmul, op)
							},
							0x5C => {
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::Fsub, op)
							},
							0x5E => {
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::OP2_XMM);
								*rip += ip_offset;
								(Opcode::Fdiv, op)
							}
							0x6E => {
								assert!(flags.contains(Flags::OPERAND_16_BIT));
								//let register_size = if register_size == RegisterSize::Bit16 { RegisterSize::Bit128 }
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,//register_size,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM); // Checkme
								*rip += ip_offset;
								(Opcode::Movd, op)
							}
							0x7E => {
								assert!(flags.contains(Flags::OPERAND_16_BIT));
								let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit128,//register_size,
																																RegOrOpcode::Register,
																																ImmediateSize::None,
																																flags | Flags::OP1_XMM | Flags::REVERSED_REGISTER_DIRECTION); // Checkme
								*rip += ip_offset;
								(Opcode::Movd, op)
							}
							opcode @ 0x80..=0x8F => {
									// TODO: could also be 16bit value
									let immediate = memory.get_i32(*rip, 1) as i64;
									*rip += 5;
									(jcc[(opcode-0x80) as usize], Operands{ operands: (Some(Operand::Immediate{ immediate }), None, None), ..Default::default() })
							},
							opcode @ 0x90..=0x9F => {
									let (mut op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																																	RegOrOpcode::Register,
																																	ImmediateSize::None,
																																	flags);
									// TODO: change this hack to Something sane
									op.operands.0 = Some(op.operands.1.unwrap());
									op.operands.1 = None;
									*rip += ip_offset;
									(scc[(opcode-0x90) as usize], op)
							},
							0xA2 => {
									*rip += 1;
									(Opcode::Cpuid, Operands::default())
							}
							0xA3 => {
									let op = decode_reg_reg(memory, rip, register_size, flags);
									(Opcode::Bt, op)
							}
							0xAB => {
									let op = decode_reg_reg(memory, rip, register_size, flags);
									(Opcode::Bts, op)
							}
							0xAF => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																															RegOrOpcode::Register,
																															ImmediateSize::None,
																															flags | Flags::REVERSED_REGISTER_DIRECTION);
									*rip += ip_offset;
									(Opcode::Imul, op)
							}
							0xB0 => {
									let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																																	RegOrOpcode::Register,
																																	ImmediateSize::None,
																																	flags);
									*rip += ip_offset;
									(Opcode::Cmpxchg, op)
							}
							0xB1 => {
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																																	RegOrOpcode::Register,
																																	ImmediateSize::None,
																																	flags);
									*rip += ip_offset;
									(Opcode::Cmpxchg, op)
							}
							0xB3 => {
									let op = decode_reg_reg(memory, rip, register_size, flags);
									(Opcode::Btr, op)
							}
							0xB6 => {
									let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																																			RegOrOpcode::Register,
																																			ImmediateSize::None,
																																			flags | Flags::REVERSED_REGISTER_DIRECTION);

									override_operand_size(&memory, *rip, &mut op, OperandSize::Bit8, &flags);
									*rip += ip_offset;
									(Opcode::Movzx, op)
							}
							0xB7 => {
									let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																																			RegOrOpcode::Register,
																																			ImmediateSize::None,
																																			flags | Flags::REVERSED_REGISTER_DIRECTION);
									override_operand_size(&memory, *rip, &mut op, OperandSize::Bit16, &flags);
									*rip += ip_offset;
									(Opcode::Movzx, op)
							}
							0xBA => {
									// bit manipulation
									let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																																	RegOrOpcode::Opcode,
																																	ImmediateSize::Bit8,
																																	flags);
									*rip += ip_offset;
									(Opcode::BitManipulation, op)
							}
							0xBB => {
									let op = decode_reg_reg(memory, rip, register_size, flags);
									(Opcode::Btc, op)
							}
							0xBE => {
									let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																																			RegOrOpcode::Register,
																																			ImmediateSize::None,
																																			flags | Flags::REVERSED_REGISTER_DIRECTION);
									override_operand_size(&memory, *rip, &mut op, OperandSize::Bit8, &flags);
									*rip += ip_offset;
									(Opcode::Movsx, op)
							}
							0xBF => {
									let (mut op, ip_offset) = get_operands(&memory, *rip, register_size,
																																			RegOrOpcode::Register,
																																			ImmediateSize::None,
																																			flags | Flags::REVERSED_REGISTER_DIRECTION);
									override_operand_size(&memory, *rip, &mut op, OperandSize::Bit16, &flags);
									*rip += ip_offset;
									(Opcode::Movsx, op)
							}
							unknown => panic!("Unknown instruction: {:?} 0F {:X}", flags, unknown),
					}
			}
			0xCC => {
					// abuse int 3 instruction to signal failed test program
					panic!("int3 instruction");
			}
			0xCD => {
					// abuse int X instruction to signal passed test program
					(Opcode::Int, Operands::default())
			}
			unknown => panic!("Unknown instruction: {:x}", unknown),
	}
}

fn read_immediate_8bit(memory: &Memory, rip: &mut i64) -> Operands {
	let immediate = memory.get_i8(*rip, 1) as i64;
	*rip += 2;
	Operands{ operands: (Some(Operand::Immediate{ immediate }), None, None), ..Default::default()}
}

fn get_operands(memory : &Memory, rip: i64, register_size: RegisterSize, reg_or_opcode: RegOrOpcode, immediate_size: ImmediateSize, mut flags: Flags) -> (Operands, i64) {
	let modrm = memory.get_u8(rip, 1);
	let mut address_mod = modrm >> 6;
	match address_mod {
			0b00 | 0b01 | 0b10 => {
					// effective address / effecive address + 8 bit deplacement /
					// effecive address + 32 bit deplacement
					let rm = modrm & 0b00000111;

					// special case: RIP relative adressing. We fake a 32bit displacement instruction.
					if address_mod == 0b00 && rm == 0x5 {
							address_mod = 0b100;
					}

					// sib byte
					let (sib, offset) = if rm == 0b100 {
							(Some(memory.get_u8(rip, 2)), 3)
					} else {
							(None, 2)
					};

					let (displacement, mut ip_offset) = match address_mod {
							0b00 => {
									match sib {
											Some(sib) => {
													let base = sib & 0b00000111;
													if base == 0x5 {
															let displacement = memory.get_i32(rip, offset);
															flags |= Flags::SIB_DISPLACEMENT_ONLY;
															(displacement, 4)
													} else {
															(0, 0)
													}
											},
											None => (0, 0)
									}
							}
							0b01 => {
									(memory.get_i8(rip, offset) as i8 as i32, 1)
							}
							0b10 | 0b100 => {
									let displacement = memory.get_i32(rip, offset);
									// change RIP relative addressing mode back to 0b00
									if address_mod == 0b100 {
											address_mod = 0b00;
									}

									(displacement, 4)
							}
							_ => unreachable!(),
					};
					ip_offset += offset; // skip instruction + modrm byte

					let register_or_opcode = (modrm & 0b00111000) >> 3;
					// TODO: based on REX, this could be a 64bit value
					match immediate_size {
							ImmediateSize::Bit8 => {
									assert!(reg_or_opcode == RegOrOpcode::Opcode);
									let immediate = memory.get_u8(rip, ip_offset);

									let operand_size = match register_size {
											RegisterSize::Bit8 => OperandSize::Bit8,
											RegisterSize::Bit16 => OperandSize::Bit16,
											RegisterSize::Bit32 => OperandSize::Bit32,
											RegisterSize::Bit64 => OperandSize::Bit64,
											_ => panic!("Unsupported register size"),
									};
									let register = if address_mod == 0b00 && rm == 0x5 {
											Register::RIP
									} else {
											let register_size = if flags.contains(Flags::ADDRESS_SIZE_OVERRIDE) {
													RegisterSize::Bit32
											} else {
													RegisterSize::Bit64
											};
											get_register(rm, register_size, flags.contains(Flags::NEW_64BIT_REGISTER),
																		flags.contains(Flags::NEW_8BIT_REGISTER))
									};

									(Operands{ operands: (Some(Operand::Immediate{ immediate: immediate as i64 }),
																													Some(effective_address(sib, register, displacement, flags)), None),
																			opcode: Some(register_or_opcode), explicit_size: Some(operand_size), ..Default::default() },
										ip_offset + 1)
							}
							ImmediateSize::Bit32 => {
									assert!(reg_or_opcode == RegOrOpcode::Opcode);
									let immediate = if flags.contains(Flags::OPERAND_16_BIT) {
											let value : i16 = memory.get_i16(rip, ip_offset);
											ip_offset += 2;
											value as i64
									} else {
											let value : i32 = memory.get_i32(rip, ip_offset);
											ip_offset += 4;
											value as i64
									};

									let operand_size = match register_size {
											RegisterSize::Bit8 => OperandSize::Bit8,
											RegisterSize::Bit16 => OperandSize::Bit16,
											RegisterSize::Bit32 => OperandSize::Bit32,
											RegisterSize::Bit64 => OperandSize::Bit64,
											_ => panic!("Unsupported register size"),
									};

									let register = if address_mod == 0b00 && rm == 0x5 {
											Register::RIP
									} else {
											let register_size = if flags.contains(Flags::ADDRESS_SIZE_OVERRIDE) {
													RegisterSize::Bit32
											} else {
													RegisterSize::Bit64
											};
											get_register(rm, register_size, flags.contains(Flags::NEW_64BIT_REGISTER),
																		flags.contains(Flags::NEW_8BIT_REGISTER))
									};

									(Operands{ operands: (Some(Operand::Immediate{ immediate }), Some(effective_address(sib, register, displacement, flags)), None),
																			opcode: Some(register_or_opcode), explicit_size: Some(operand_size), ..Default::default() },
										ip_offset)
							}
							ImmediateSize::None => {
									let first_reg_size = if flags.contains(Flags::ADDRESS_SIZE_OVERRIDE) {
											RegisterSize::Bit32
									} else {
											RegisterSize::Bit64
									};

									// special case: RIP relative adressing.
									let register1 = if address_mod == 0b00 && rm == 0x5 {
											Register::RIP
									} else {
											get_register(rm,
																		first_reg_size,
																		flags.contains(Flags::NEW_64BIT_REGISTER),
																		flags.contains(Flags::NEW_8BIT_REGISTER))
									};

									(match reg_or_opcode {
											RegOrOpcode::Register => {
													let register2 = get_register_or_xmm(register_or_opcode,
																											register_size,
																											flags.contains(Flags::MOD_R_M_EXTENSION),
																											flags.contains(Flags::NEW_8BIT_REGISTER),
																											flags.contains(Flags::OP2_XMM));

													if flags.contains(Flags::REVERSED_REGISTER_DIRECTION) {
															Operands{ operands: (Some(effective_address(sib, register1, displacement, flags)), Some(Operand::Register{ register: register2 }), None),
																									..Default::default()}
													} else {
															Operands{ operands: (Some(Operand::Register{ register: register2 }),
																																			Some(effective_address(sib, register1, displacement, flags)), None), ..Default::default() }
													}
											},
											RegOrOpcode::Opcode => {
													Operands{ operands: (Some(effective_address(sib, register1, displacement, flags)), None, None),
																						opcode: Some(register_or_opcode), explicit_size: Some(OperandSize::Bit64), ..Default::default() }
											}
									}, ip_offset)
							}
					}
			}
			0b11 => {
					// register
					let register = get_register_or_xmm(modrm & 0b00000111,
																				register_size,
																				flags.contains(Flags::NEW_64BIT_REGISTER),
																				flags.contains(Flags::NEW_8BIT_REGISTER),
																				flags.contains(Flags::OP1_XMM));
					let value2 = (modrm & 0b00111000) >> 3;
					match reg_or_opcode {
							RegOrOpcode::Register => {
									(if flags.contains(Flags::REVERSED_REGISTER_DIRECTION) {
											Operands{ operands: (Some(Operand::Register{ register }),
																															Some(Operand::Register{ register: get_register_or_xmm(value2, register_size,
																																																															flags.contains(Flags::MOD_R_M_EXTENSION),
																																																															flags.contains(Flags::NEW_8BIT_REGISTER),
																																																															flags.contains(Flags::OP2_XMM)) }),
																															None), ..Default::default()}
										} else {
											Operands{ operands: (Some(Operand::Register{ register: get_register_or_xmm(value2, register_size,
																																																															flags.contains(Flags::MOD_R_M_EXTENSION),
																																																															flags.contains(Flags::NEW_8BIT_REGISTER),
																																																															flags.contains(Flags::OP2_XMM)) }),
																															Some(Operand::Register{ register }),
																															None), ..Default::default()}
										},
										2)
							}
							RegOrOpcode::Opcode => {
									match immediate_size {
											ImmediateSize::Bit8 => {
													let immediate = memory.get_i8(rip, 2);
													(Operands{ operands: (Some(Operand::Immediate{ immediate: immediate as i64 }), Some(Operand::Register{ register }), None),
																							opcode: Some(value2), ..Default::default()}, 3)
											}
											ImmediateSize::Bit32 => {
													let immediate = memory.get_i32(rip, 2);
													(Operands{ operands: (Some(Operand::Immediate{ immediate: immediate as i64 }), Some(Operand::Register{ register }), None),
																							opcode: Some(value2), ..Default::default()}, 6)
											}
											ImmediateSize::None => { (Operands{ operands: (Some(Operand::Register{ register }), None, None), opcode: Some(value2), ..Default::default()}, 2) }
									}
							}
					}
			}
			_ => unreachable!(),
	}
}

fn effective_address(sib: Option<u8>, register: Register, displacement: i32, flags: Flags) -> Operand {
	match sib {
		None => {
			Operand::EffectiveAddress {
					base: Some(register),
					index: None,
					scale: None,
					displacement,
			}
		}
		Some(sib) => {
			let base_num = sib & 0b00000111;
			let index = (sib & 0b00111000) >> 3;
			let scale = (sib & 0b11000000) >> 6;
			let scale = 2u8.pow(scale as u32) as u8;

			let register_size = if flags.contains(Flags::ADDRESS_SIZE_OVERRIDE) {
				RegisterSize::Bit32
			} else {
				RegisterSize::Bit64
			};

			let base = get_register(base_num, register_size,
															flags.contains(Flags::NEW_64BIT_REGISTER), false);

			if index == 0x4 {
				if base_num == 0x5 && flags.contains(Flags::SIB_DISPLACEMENT_ONLY) {
					Operand::EffectiveAddress {
							base: None,
							displacement,
							scale: None,
							index: None,
					}
				} else {
					Operand::EffectiveAddress {
							base: Some(base),
							displacement,
							scale: None,
							index: None,
					}
				}
			} else if base_num == 0x5 && flags.contains(Flags::SIB_DISPLACEMENT_ONLY) {
				Operand::EffectiveAddress {
						base: None,
						displacement,
						scale: Some(scale),
						index: Some(get_register(index, register_size,
																		flags.contains(Flags::SIB_EXTENSION), false))
				}
			} else {
				Operand::EffectiveAddress {
						base: Some(base),
						displacement,
						scale: Some(scale),
						index: Some(get_register(index, register_size,
																		flags.contains(Flags::SIB_EXTENSION), false))
				}
			}
		}
	}
}

fn decode_8bit_reg_8bit_immediate(memory: &Memory, rip: &mut i64, flags: Flags) -> Operands {
	let (op, ip_offset) = get_operands(&memory, *rip, RegisterSize::Bit8,
																								RegOrOpcode::Register,
																								ImmediateSize::None,
																								flags);
	*rip += ip_offset;
	op
}

fn decode_reg_reg(memory: &Memory, rip: &mut i64, register_size: RegisterSize, flags: Flags) -> Operands {
	let (op, ip_offset) = get_operands(&memory, *rip, register_size,
																								RegOrOpcode::Register,
																								ImmediateSize::None,
																								flags);
	*rip += ip_offset;
	op
}

fn decode_al_immediate(memory: &Memory, rip: &mut i64) -> Operands {
	let immediate = memory.get_i8(*rip, 1);
	let op = Operands{ operands: (Some(Operand::Immediate{ immediate: immediate as i64 }), Some(Operand::Register { register: Register::AL }), None), ..Default::default() };
	*rip += 2;
	op
}

fn decode_ax_immediate(memory: &Memory, rip: &mut i64, register_size: RegisterSize, flags: Flags) -> Operands {
	let (immediate, ip_offset) = if flags.contains(Flags::OPERAND_16_BIT) {
			(memory.get_i16(*rip, 1) as i64, 3)
	} else {
			(memory.get_i32(*rip, 1) as i64, 5)
	};

	let register = get_register(0,
			register_size, flags.contains(Flags::NEW_64BIT_REGISTER),
			false);

	let op = Operands{ operands: (Some(Operand::Immediate{ immediate }), Some(Operand::Register { register }), None), ..Default::default()};
	*rip += ip_offset;
	op
}

fn override_operand_size(memory : &Memory, rip: i64, op: &mut Operands, size: OperandSize, flags: &Flags) {
	match op.operands.0 {
		Some(Operand::Register{..}) => {
			let register_size = match size {
					OperandSize::Bit8 => RegisterSize::Bit8,
					OperandSize::Bit16 => RegisterSize::Bit16,
					OperandSize::Bit32 => RegisterSize::Bit32,
					OperandSize::Bit64 => RegisterSize::Bit64,
					_ => unreachable!(),
			};
			let modrm = memory.get_u8(rip, 1);
			let register = modrm & 0b00000111;
			let register = get_register(register, register_size,
																	flags.contains(Flags::NEW_64BIT_REGISTER),
																	flags.contains(Flags::NEW_8BIT_REGISTER));
			op.operands.0 = Some(Operand::Register{ register })
		},
		Some(Operand::EffectiveAddress{..}) => { op.explicit_size = Some(size); },
			_ => panic!("Invalid instruction")
}
}

fn get_register(num: u8, size: RegisterSize, new_64bit_register: bool, new_8bit_register: bool) -> Register {
	match size {
		RegisterSize::Bit128 => {
			if new_64bit_register {
				match num {
					0 => Register::XMM8,
					1 => Register::XMM9,
					2 => Register::XMM10,
					3 => Register::XMM11,
					4 => Register::XMM12,
					5 => Register::XMM13,
					6 => Register::XMM14,
					7 => Register::XMM15,
					_ => panic!("Unknown instruction operand"),
				}
			} else {
				match num {
					0 => Register::XMM0,
					1 => Register::XMM1,
					2 => Register::XMM2,
					3 => Register::XMM3,
					4 => Register::XMM4,
					5 => Register::XMM5,
					6 => Register::XMM6,
					7 => Register::XMM7,
					_ => panic!("Unknown instruction operand"),
				}
			}
		}
		RegisterSize::Bit64 => {
			if new_64bit_register {
				match num {
					0 => Register::R8,
					1 => Register::R9,
					2 => Register::R10,
					3 => Register::R11,
					4 => Register::R12,
					5 => Register::R13,
					6 => Register::R14,
					7 => Register::R15,
					_ => panic!("Unknown instruction operand"),
				}
			} else {
				match num {
					0 => Register::RAX,
					1 => Register::RCX,
					2 => Register::RDX,
					3 => Register::RBX,
					4 => Register::RSP,
					5 => Register::RBP,
					6 => Register::RSI,
					7 => Register::RDI,
					_ => panic!("Unknown instruction operand"),
				}
			}
		}
		RegisterSize::Bit32 => {
			if new_64bit_register {
				match num {
					0 => Register::R8D,
					1 => Register::R9D,
					2 => Register::R10D,
					3 => Register::R11D,
					4 => Register::R12D,
					5 => Register::R13D,
					6 => Register::R14D,
					7 => Register::R15D,
					_ => panic!("Unknown instruction operand"),
				}
			} else {
				match num {
					0 => Register::EAX,
					1 => Register::ECX,
					2 => Register::EDX,
					3 => Register::EBX,
					4 => Register::ESP,
					5 => Register::EBP,
					6 => Register::ESI,
					7 => Register::EDI,
					_ => panic!("Unknown instruction operand"),
				}
			}
		}
		RegisterSize::Bit16 => {
			if new_64bit_register {
				match num {
					0 => Register::R8W,
					1 => Register::R9W,
					2 => Register::R10W,
					3 => Register::R11W,
					4 => Register::R12W,
					5 => Register::R13W,
					6 => Register::R14W,
					7 => Register::R15W,
					_ => panic!("Unknown instruction operand"),
				}
			} else {
				match num {
					0 => Register::AX,
					1 => Register::CX,
					2 => Register::DX,
					3 => Register::BX,
					4 => Register::SP,
					5 => Register::BP,
					6 => Register::SI,
					7 => Register::DI,
					_ => panic!("Unknown instruction operand"),
				}
			}
		}
		RegisterSize::Bit8 => {
			if new_64bit_register {
				match num {
					0 => Register::R8B,
					1 => Register::R9B,
					2 => Register::R10B,
					3 => Register::R11B,
					4 => Register::R12B,
					5 => Register::R13B,
					6 => Register::R14B,
					7 => Register::R15B,
					_ => panic!("Unknown instruction operand"),
				}
			} else if new_8bit_register {
				match num {
					0 => Register::AL,
					1 => Register::CL,
					2 => Register::DL,
					3 => Register::BL,
					4 => Register::SPL,
					5 => Register::BPL,
					6 => Register::SIL,
					7 => Register::DIL,
					_ => panic!("Unknown instruction operand"),
				}
			} else {
				match num {
					0 => Register::AL,
					1 => Register::CL,
					2 => Register::DL,
					3 => Register::BL,
					4 => Register::AH,
					5 => Register::CH,
					6 => Register::DH,
					7 => Register::BH,
					_ => panic!("Unknown instruction operand"),
				}
			}
		}
		RegisterSize::Segment => {
			match num {
				0 => Register::ES,
				1 => Register::CS,
				2 => Register::SS,
				3 => Register::DS,
				4 => Register::FS,
				5 => Register::GS,
				_ => panic!("Unknown instruction operand"),
			}
		}
	}
}

fn get_xmm(num: u8, size: RegisterSize, new_64bit_register: bool) -> Register {
	match size {
		/*RegisterSize::Bit128*/_ => {
			if new_64bit_register {
				match num {
					0 => Register::XMM8,
					1 => Register::XMM9,
					2 => Register::XMM10,
					3 => Register::XMM11,
					4 => Register::XMM12,
					5 => Register::XMM13,
					6 => Register::XMM14,
					7 => Register::XMM15,
					_ => panic!("Unknown instruction operand"),
				}
			} else {
				match num {
					0 => Register::XMM0,
					1 => Register::XMM1,
					2 => Register::XMM2,
					3 => Register::XMM3,
					4 => Register::XMM4,
					5 => Register::XMM5,
					6 => Register::XMM6,
					7 => Register::XMM7,
					_ => panic!("Unknown instruction operand"),
				}
			}
		}
	}
}


fn get_register_or_xmm(num: u8, size: RegisterSize, new_64bit_register: bool, new_8bit_register: bool, xmm: bool) -> Register {
	if xmm { get_xmm(num, size, new_64bit_register) }
	else { get_register(num, size, new_64bit_register, new_8bit_register) }
}
