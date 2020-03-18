use crate::{memory::Memory, instruction::{Register, RegisterSize, ArgumentSize, Opcode, Argument, Instruction, InstructionBuilder}};

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
    struct DecoderFlags: u64 {
        const REVERSED_REGISTER_DIRECTION = 1 << 0;
        const ADDRESS_SIZE_OVERRIDE = 1 << 2;
        const REPEAT_EQUAL = 1 << 3;
        const REPEAT_NOT_EQUAL = 1 << 4;
        const NEW_64BIT_REGISTER = 1 << 5;
        const NEW_8BIT_REGISTER = 1 << 6;
        const MOD_R_M_EXTENSION = 1 << 7;
        const SIB_EXTENSION = 1 << 8;
        const OPERAND_16_BIT = 1 << 9;
        const OPERAND_64_BIT = 1 << 10;
        const SIB_DISPLACEMENT_ONLY = 1 << 11;
    }
}

pub fn decode(rip : &mut i64, memory : &Memory) -> (Opcode, Instruction) {
    let mut decoder_flags = DecoderFlags { bits: 0 };
    loop {
        match memory.read_byte(*rip as u64) {
            0xF0 => {
                // todo: do not ignore lock/bound prefix
            }
            0xF2 => {
                decoder_flags |= DecoderFlags::REPEAT_NOT_EQUAL;
            }
            0xF3 => {
                decoder_flags |= DecoderFlags::REPEAT_EQUAL;
            }
            0x2E | 0x3E | 0x36 | 0x26 | 0x64 | 0x65 => {
                //TODO: do not ignore segment prefix (or probably we should?)
            }
            0x66 => {
                decoder_flags |= DecoderFlags::OPERAND_16_BIT;
            }
            0x67 => {
                decoder_flags |= DecoderFlags::ADDRESS_SIZE_OVERRIDE;
            }
            bits @ 0x40..=0x4F => { // 64bit REX prefix
                let rex = REX{bits};
                if rex.contains(REX::B) { decoder_flags |= DecoderFlags::NEW_64BIT_REGISTER; }
                if rex.contains(REX::R) { decoder_flags |= DecoderFlags::MOD_R_M_EXTENSION; }
                if rex.contains(REX::X) { decoder_flags |= DecoderFlags::SIB_EXTENSION; }
                if rex.contains(REX::W) { decoder_flags |= DecoderFlags::OPERAND_64_BIT;  }
                decoder_flags |= DecoderFlags::NEW_8BIT_REGISTER;
            }
            _ => break,
        }
        *rip += 1;
    }

    let register_size = if decoder_flags.contains(DecoderFlags::OPERAND_64_BIT) {
        RegisterSize::Bit64
    } else {
        if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
            RegisterSize::Bit16
        } else {
            RegisterSize::Bit32
        }
    };

    macro_rules! Opcode { ($($op:ident)+) => ( [$(Opcode::$op),+] ) }
    let jcc = Opcode!(Jo Jno Jb Jae Je Jne Jbe Ja Js Jns Jp Jnp Jl Jge Jle Jg);
    let scc = Opcode!(Seto Setno Setb Setae Sete Setne Setbe Seta Sets Setns Setp Setnp Setl Setge Setle Setg);
    match memory.read_byte(*rip as u64) {
        0x00 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::Add, argument)
        }
        0x01 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::Add, argument)
        }
        0x02 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Add, argument)
        }
        0x03 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Add, argument)
        }
        0x04 => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Add, argument)
        }
        0x05 => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Add, argument)
        }
        0x08 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::Or, argument)
        }
        0x09 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::Or, argument)
        }
        0x0A => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Or, argument)
        }
        0x0B => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Or, argument)
        }
        0x0C => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Or, argument)
        }
        0x0D => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Or, argument)
        }
        0x10 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::Adc, argument)
        }
        0x11 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::Adc, argument)
        }
        0x12 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Adc, argument)
        }
        0x13 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Adc, argument)
        }
        0x14 => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Adc, argument)
        }
        0x15 => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Adc, argument)
        }
        0x18 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::Sbb, argument)
        }
        0x19 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::Sbb, argument)
        }
        0x1A => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Sbb, argument)
        }
        0x1B => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Sbb, argument)
        }
        0x1C => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Sbb, argument)
        }
        0x1D => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Sbb, argument)
        }
        0x20 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::And, argument)
        }
        0x21 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::And, argument)
        }
        0x22 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::And, argument)
        }
        0x23 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::And, argument)
        }
        0x24 => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::And, argument)
        }
        0x25 => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::And, argument)
        }
        0x28 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::Sub, argument)
        }
        0x29 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::Sub, argument)
        }
        0x2A => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Sub, argument)
        }
        0x2B => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Sub, argument)
        }
        0x2C => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Sub, argument)
        }
        0x2D => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Sub, argument)
        }
        0x30 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::Xor, argument)
        }
        0x31 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::Xor, argument)
        }
        0x32 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Xor, argument)
        }
        0x33 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Xor, argument)
        }
        0x34 => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Xor, argument)
        }
        0x35 => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Xor, argument)
        }
        0x38 => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags);
            (Opcode::Cmp, argument)
        }
        0x39 => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
            (Opcode::Cmp, argument)
        }
        0x3A => {
            let argument = decode_8bit_reg_8bit_immediate(memory, rip, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Cmp, argument)
        }
        0x3B => {
            let argument = decode_reg_reg(memory, rip, register_size, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            (Opcode::Cmp, argument)
        }
        0x3C => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Cmp, argument)
        }
        0x3D => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Cmp, argument)
        }
        opcode @ 0x50..=0x57 => {
            *rip += 1;
            (Opcode::Push,
                        InstructionBuilder::new().first_argument(Argument::Register {
                            register: get_register(opcode - 0x50, RegisterSize::Bit64,
                                    decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                    decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                        }).finalize())
        }
        opcode @ 0x58..=0x5F => {
            let argument =
                InstructionBuilder::new().first_argument(Argument::Register {
                        register:
                            get_register(opcode - 0x58,
                                            RegisterSize::Bit64,
                                            decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                            decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                    })
                    .finalize();
            *rip += 1;
            (Opcode::Pop, argument)
        }
        0x63 => {
            let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            override_argument_size(&memory, *rip, &mut argument, ArgumentSize::Bit32, &decoder_flags);
            *rip += ip_offset;
            (Opcode::Movsx, argument)
        }
        0x68 => {
            let immediate = if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
                let immediate = memory.get_i16(*rip, 1) as i64;
                *rip += 3;
                immediate
            } else {
                let immediate = memory.get_i32(*rip, 1) as i64;
                *rip += 5;
                immediate
            };
            let argument = InstructionBuilder::new().first_argument(
                Argument::Immediate { immediate: immediate }
            ).finalize();
            (Opcode::Push, argument)
        }
        0x69 => {
            let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            *rip += ip_offset;
            let immediate = if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
                let immediate = memory.get_i16(*rip, 0) as i64;
                *rip += 2;
                immediate
            } else {
                let immediate = memory.get_i32(*rip, 0) as i64;
                *rip += 4;
                immediate
            };
            argument.third_argument = argument.second_argument;
            argument.second_argument = argument.first_argument;
            argument.first_argument = Some(Argument::Immediate { immediate: immediate });
            (Opcode::Imul, argument)
        }
        0x6A => (Opcode::Push, read_immediate_8bit(memory, rip)),
        0x6B => {
            let (mut argument, ip_offset) = get_argument(memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            *rip += ip_offset;
            let immediate = memory.get_i8(*rip, 0) as i64;
            argument.third_argument = argument.second_argument;
            argument.second_argument = argument.first_argument;
            argument.first_argument = Some(Argument::Immediate { immediate: immediate });
            *rip += 1;
            (Opcode::Imul, argument)
        }
        opcode @ 0x70..=0x7F => { (jcc[(opcode-0x70) as usize], read_immediate_8bit(memory, rip)) }
        0x80 => {
            // arithmetic operation (8bit register target, 8bit immediate)
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::Bit8,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Arithmetic, argument)
        }
        0x81 => {
            // arithmetic operation (32/64bit register target, 32bit immediate)
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::Bit32,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Arithmetic, argument)
        }
        0x83 => {
            // arithmetic operation (32/64bit register target, 8bit immediate)
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::Bit8,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Arithmetic, argument)
        }
        0x84 => {
            // test
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Test, argument)
        }
        0x85 => {
            // test
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Test, argument)
        }
        0x86 => {
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Xchg, argument)
        }
        0x87 => {
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Xchg, argument)
        }
        0x88 => {
            // mov
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0x89 => {
            // mov
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0x8A => {
            // mov
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0x8B => {
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Register,
                                                            ImmediateSize::None,
                                                            decoder_flags |
                                                            DecoderFlags::REVERSED_REGISTER_DIRECTION);
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0x8D => {
            let (argument, ip_offset) =
                get_argument(&memory, *rip, register_size,
                                    RegOrOpcode::Register,
                                    ImmediateSize::None,
                                    // TODO: REVERSED_REGISTER_DIRECTION correct?
                                    decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            *rip += ip_offset;
            (Opcode::Lea, argument)
        }
        0x8E => {
            // mov 16bit segment registers
            let (argument, ip_offset) =
                get_argument(&memory, *rip, RegisterSize::Segment,
                                    RegOrOpcode::Register,
                                    ImmediateSize::None,
                                    // TODO: REVERSED_REGISTER_DIRECTION correct?
                                    decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0x8F => {
            let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags |
                                                                DecoderFlags::REVERSED_REGISTER_DIRECTION);
            argument.second_argument = None;
            *rip += ip_offset;
            (Opcode::Pop, argument)
        }
        0x90 => {
            *rip += 1;
            (Opcode::Nop, Instruction::default())
        }
        opcode @ 0x91..=0x97 => {
            let argument = InstructionBuilder::new()
                .first_argument(Argument::Register {
                    register: get_register(0, register_size,
                                            decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                            decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                    })
                    .second_argument(Argument::Register {
                        register: get_register(opcode - 0x90,
                                                register_size,
                                                decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                                decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                    })
                    .finalize();
            *rip += 1;
            (Opcode::Xchg, argument)
        }
        0x98 => {
            let (register1, register2) = if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
                (Register::AL, Register::AX)
            } else if decoder_flags.contains(DecoderFlags::OPERAND_64_BIT) {
                (Register::EAX, Register::RAX)
            } else {
                (Register::AX, Register::EAX)
            };

            let argument = InstructionBuilder::new().first_argument(
                Argument::Register{register: register1}
            ).second_argument(Argument::Register{register: register2})
            .finalize();
            *rip += 1;
            (Opcode::Mov, argument)
        }
        0x99 => {
            let (register1, register2) = if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
                (Register::AX, Register::DX)
            } else if decoder_flags.contains(DecoderFlags::OPERAND_64_BIT) {
                (Register::RAX, Register::RDX)
            } else {
                (Register::EAX, Register::EDX)
            };

            let argument = InstructionBuilder::new().first_argument(
                Argument::Register{register: register1}
            ).second_argument(Argument::Register{register: register2})
            .finalize();
            *rip += 1;
            (Opcode::Mov, argument)
        }
        0x9C => {
            *rip += 1;
            (Opcode::Pushf, Instruction::default())
        }
        0x9D => {
            *rip += 1;
            (Opcode::Popf, Instruction::default())
        }
        0xA4 => {
            *rip += 1;
            (Opcode::Movs, InstructionBuilder::new()
                .repeat(decoder_flags.contains(DecoderFlags::REPEAT_EQUAL), decoder_flags.contains(DecoderFlags::REPEAT_NOT_EQUAL))
                .explicit_size(ArgumentSize::Bit8)
                .finalize())
        }
        0xA5 => {
            let argument_size = if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
                ArgumentSize::Bit16
            } else if decoder_flags.contains(DecoderFlags::OPERAND_64_BIT) {
                ArgumentSize::Bit64
            } else {
                ArgumentSize::Bit32
            };
            *rip += 1;
            (Opcode::Movs, InstructionBuilder::new()
                .repeat(decoder_flags.contains(DecoderFlags::REPEAT_EQUAL), decoder_flags.contains(DecoderFlags::REPEAT_NOT_EQUAL))
                .explicit_size(argument_size)
                .finalize())
        }
        0xA8 => {
            let argument = decode_al_immediate(memory, rip);
            (Opcode::Test, argument)
        }
        0xA9 => {
            let argument = decode_ax_immediate(memory, rip, register_size, decoder_flags);
            (Opcode::Test, argument)
        }
        0xAA => {
            *rip += 1;
            (Opcode::Stos, InstructionBuilder::new()
                .repeat(decoder_flags.contains(DecoderFlags::REPEAT_EQUAL), decoder_flags.contains(DecoderFlags::REPEAT_NOT_EQUAL))
                .explicit_size(ArgumentSize::Bit8)
                .finalize())
        }
        0xAB => {
            *rip += 1;
            let argument_size = match register_size {
                RegisterSize::Bit8 => ArgumentSize::Bit8,
                RegisterSize::Bit16 => ArgumentSize::Bit16,
                RegisterSize::Bit32 => ArgumentSize::Bit32,
                RegisterSize::Bit64 => ArgumentSize::Bit64,
                RegisterSize::Segment => panic!("Unsupported register size"),
            };
            (Opcode::Stos, InstructionBuilder::new()
                .repeat(decoder_flags.contains(DecoderFlags::REPEAT_EQUAL), decoder_flags.contains(DecoderFlags::REPEAT_NOT_EQUAL))
                .explicit_size(argument_size)
                .finalize())
        }
        0xAE => {
            *rip += 1;
            (Opcode::Scas, InstructionBuilder::new()
                .first_argument(Argument::EffectiveAddress{
                    base: Some(Register::RDI),
                    index: None,
                    scale: None,
                    displacement: 0,
                    })
                .second_argument(Argument::Register{ register: Register::AL })
                .repeat(decoder_flags.contains(DecoderFlags::REPEAT_EQUAL), decoder_flags.contains(DecoderFlags::REPEAT_NOT_EQUAL))
                .finalize())
        }
        opcode @ 0xB0..=0xB7 => {
            let immediate = memory.get_u8(*rip, 1) as i64;
            let argument =
                InstructionBuilder::new().first_argument(Argument::Immediate {
                        immediate: immediate as i64,
                    })
                    .second_argument(Argument::Register {
                        register:
                            get_register(opcode - 0xB0,
                                            RegisterSize::Bit8,
                                            decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                            decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                    })
                    .finalize();
            *rip += 2;
            (Opcode::Mov, argument)
        }
        opcode @ 0xB8..=0xBF => {
            let (immediate, ip_offset) = if decoder_flags.contains(DecoderFlags::OPERAND_64_BIT) {
                (memory.get_i64(*rip, 1) as i64, 9)
            } else {
                if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
                    (memory.get_i16(*rip, 1) as i64, 3)
                } else {
                    (memory.get_i32(*rip, 1) as i64, 5)
                }
            };
            let argument =
                InstructionBuilder::new().first_argument(Argument::Immediate {
                        immediate: immediate,
                    })
                    .second_argument(Argument::Register {
                        register:
                            get_register(opcode - 0xB8,
                                            register_size,
                                            decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                            decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                    })
                    .finalize();
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0xC6 => {
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::Bit8,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0xC7 => {
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::Bit32,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::Mov, argument)
        }
        0xC0 => {
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::Bit8,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::ShiftRotate, argument)
        }
        0xC1 => {
            let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::Bit8,
                                                            decoder_flags);
            *rip += ip_offset;
            (Opcode::ShiftRotate, argument)
        }
        0xC3 => {
            (Opcode::Ret, Instruction::default())
        }
        0xC9 => {
            *rip += 1;
            (Opcode::Leave, Instruction::default())
        }
        0xCB => {
            (Opcode::Lret, Instruction::default())
        }
        0xD1 => {
            let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            argument.second_argument = Some(argument.first_argument.unwrap());
            argument.first_argument = Some(Argument::Immediate{
                immediate: 1,
            });
            *rip += ip_offset;
            (Opcode::ShiftRotate, argument)
        }
        0xD2 => {
            let (mut argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                                RegOrOpcode::Opcode,
                                                                ImmediateSize::None,
                                                                decoder_flags);
            argument.second_argument = Some(argument.first_argument.unwrap());
            argument.first_argument = Some(Argument::Register{
                register: Register::CL
            });
            *rip += ip_offset;
            (Opcode::ShiftRotate, argument)
        }
        0xD3 => {
            let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                            RegOrOpcode::Opcode,
                                                            ImmediateSize::None,
                                                            decoder_flags);
            let size = argument.size();
            argument.second_argument = Some(argument.first_argument.unwrap());
            argument.first_argument = Some(Argument::Register{
                register: Register::CL
            });
            argument.explicit_size = Some(size);
            *rip += ip_offset;
            (Opcode::ShiftRotate, argument)
        }
        0xEB => { (Opcode::Jmp, read_immediate_8bit(memory, rip)) }
        0xE8 => {
            let immediate = memory.get_i32(*rip, 1);
            *rip += 5;
            (Opcode::Call,
                        InstructionBuilder::new().first_argument(Argument::Immediate {
                            immediate: immediate as i64,
                        }).finalize())
        }
        0xE9 => {
            let immediate = memory.get_i32(*rip, 1);
            *rip += 5;
            (Opcode::Jmp,
                        InstructionBuilder::new().first_argument(Argument::Immediate {
                            immediate: immediate as i64,
                        }).finalize())
        }
        0xEE => {
            *rip += 1;
            (Opcode::Out, Instruction::default())
        }
        0xF6 => {
            let modrm = memory.get_u8(*rip, 1);
            let opcode = (modrm & 0b00111000) >> 3;

            let (argument, ip_offset) = match opcode {
                0 | 1 => {
                    get_argument(&memory, *rip, RegisterSize::Bit8,
                                        RegOrOpcode::Opcode,
                                        ImmediateSize::Bit8,
                                        decoder_flags)
                },
                2 | 3 => {
                    get_argument(&memory, *rip, RegisterSize::Bit8,
                                        RegOrOpcode::Opcode,
                                        ImmediateSize::None,
                                        decoder_flags)
                }
                _ => panic!("no supported"),
            };
            *rip += ip_offset;
            (Opcode::CompareMulOperation, argument)
        }
        0xF7 => {
            let modrm = memory.get_u8(*rip, 1);
            let opcode = (modrm & 0b00111000) >> 3;

            let (argument, ip_offset) = match opcode {
                0 | 1 => {
                    // TODO: could also be 16 bit immediate
                    get_argument(&memory, *rip, register_size,
                                        RegOrOpcode::Opcode,
                                        ImmediateSize::Bit32,
                                        decoder_flags)
                },
                2 | 3 => {
                    get_argument(&memory, *rip, register_size,
                                        RegOrOpcode::Opcode,
                                        ImmediateSize::None,
                                        decoder_flags)
                },
                4 | 5 | 6 | 7 => {
                    /*let register = get_register(
                        0, register_size,decoder_flags.contains(NEW_64BIT_REGISTER), false);

                    (InstructionBuilder::new().first_argument(
                        Argument::Register{register: register})
                        .opcode(opcode)
                        .finalize(),
                    2)*/
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                        RegOrOpcode::Opcode,
                                                                        ImmediateSize::None,
                                                                        decoder_flags);
                    argument.second_argument = None;
                    argument.opcode = Some(opcode);
                    (argument, ip_offset)
                },
                _ => unreachable!()
            };
            *rip += ip_offset;
            (Opcode::CompareMulOperation, argument)
        }
        0xFA => {
            // todo: implement cli instruction
            *rip += 1;
            (Opcode::Nop, Instruction::default())
        }
        0xFB => {
            // todo: implement sti instruction
            *rip += 1;
            (Opcode::Nop, Instruction::default())
        }
        0xFC => {
            *rip += 1;
            (Opcode::Cld, Instruction::default())
        }
        0xFD => {
            *rip += 1;
            (Opcode::Std, Instruction::default())
        }
        0xFE => {
            let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                                RegOrOpcode::Opcode,
                                                                ImmediateSize::None,
                                                                decoder_flags);
            *rip += ip_offset;
            if argument.opcode.unwrap() > 1 {
                panic!("Invalid opcode");
            }
            (Opcode::RegisterOperation, argument)
        }
        0xFF => {
            // todo: cleanup code
            let modrm = memory.get_u8(*rip, 1);
            let opcode = (modrm & 0b00111000) >> 3;
            let register_size = if opcode == 2 || opcode == 4 {RegisterSize::Bit64} else {register_size}; // FF /2, 4 (Call/jmp near absolute indirect) implies REX.W
            let (mut argument, ip_offset) =
                get_argument(&memory, *rip, register_size, RegOrOpcode::Register, ImmediateSize::None, decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
            argument.second_argument = None;
            argument.opcode = Some(opcode);
            *rip += ip_offset;
            (Opcode::RegisterOperation, argument)
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
                            let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                                RegOrOpcode::Opcode,
                                                                                ImmediateSize::Bit32,
                                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                            argument.first_argument = Some(argument.second_argument.unwrap());
                            argument.second_argument = None;
                            *rip += ip_offset - 4;
                            if opcode == 2 {
                                (Opcode::Lgdt, argument)
                            } else {
                                (Opcode::Lidt, argument)
                            }
                        },
                        _ => panic!("0F 01 unsupported opcode: {:x}", opcode)
                    }
                }
                0x05 => {
                    *rip += 1;
                    (Opcode::Syscall, Instruction::default())
                }
                0x1F => {
                    // NOP with hint
                    let (_, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                    RegOrOpcode::Register,
                                                                    ImmediateSize::None,
                                                                    decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Nop, Instruction::default())
                }
                0x20 => {
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit64,
                                                                    RegOrOpcode::Register,
                                                                    ImmediateSize::None,
                                                                    decoder_flags);
                    let register = match argument.first_argument.unwrap() {
                        Argument::Register { register } => {
                            match register {
                                Register::R8 => Register::CR8,
                                Register::RAX => Register::CR0,
                                Register::RDX => Register::CR2,
                                Register::RBX => Register::CR3,
                                Register::RSP => Register::CR4,
                                _ => panic!("Invalid argument for mov r64, CRn instruciton"),
                            }
                        },
                        _ => panic!("Invalid argument for mov r64, CRn instruciton"),
                    };
                    argument.first_argument = Some(Argument::Register {register: register});
                    *rip += ip_offset;
                    (Opcode::Mov, argument)
                },
                0x22 => {
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit64,
                                                                    RegOrOpcode::Register,
                                                                    ImmediateSize::None,
                                                                    decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    let register = match argument.second_argument.unwrap() {
                        Argument::Register { register } => {
                            match register {
                                Register::R8 => Register::CR8,
                                Register::RAX => Register::CR0,
                                Register::RDX => Register::CR2,
                                Register::RBX => Register::CR3,
                                Register::RSP => Register::CR4,
                                _ => panic!("Invalid argument for mov r64, CRn instruciton"),
                            }
                        },
                        _ => panic!("Invalid argument for mov r64, CRn instruciton"),
                    };
                    argument.second_argument = Some(Argument::Register {register: register});
                    *rip += ip_offset;
                    (Opcode::Mov, argument)
                },
                0x30 => {
                    *rip += 1;
                    (Opcode::Wrmsr, Instruction::default())
                }
                0x32 => {
                    *rip += 1;
                    (Opcode::Rdmsr, Instruction::default())
                }
                0x40 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovo, argument)
                },
                0x41 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovno, argument)
                },
                0x42 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovb, argument)
                },
                0x43 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovae, argument)
                },
                0x44 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmove, argument)
                },
                0x45 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovne, argument)
                },
                0x46 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovbe, argument)
                },
                0x47 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmova, argument)
                },
                0x48 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovs, argument)
                },
                0x49 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovns, argument)
                },
                0x4a => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovp, argument)
                },
                0x4b => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovnp, argument)
                },
                0x4c => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovl, argument)
                },
                0x4d => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovge, argument)
                },
                0x4e => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovle, argument)
                },
                0x4f => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Cmovg, argument)
                },
                opcode @ 0x80..=0x8F => {
                    // TODO: could also be 16bit value
                    let immediate = memory.get_i32(*rip, 1) as i64;
                    *rip += 5;
                    (jcc[(opcode-0x80) as usize], InstructionBuilder::new().first_argument(Argument::Immediate { immediate }).finalize())
                },
                opcode @ 0x90..=0x9F => {
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                                    RegOrOpcode::Register,
                                                                    ImmediateSize::None,
                                                                    decoder_flags);
                    // TODO: change this hack to Something sane
                    argument.first_argument = Some(argument.second_argument.unwrap());
                    argument.second_argument = None;
                    *rip += ip_offset;
                    (scc[(opcode-0x90) as usize], argument)
                },
                0xA2 => {
                    *rip += 1;
                    (Opcode::Cpuid, Instruction::default())
                }
                0xA3 => {
                    let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
                    (Opcode::Bt, argument)
                }
                0xAB => {
                    let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
                    (Opcode::Bts, argument)
                }
                0xAF => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                RegOrOpcode::Register,
                                                                ImmediateSize::None,
                                                                decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    *rip += ip_offset;
                    (Opcode::Imul, argument)
                }
                0xB0 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                                    RegOrOpcode::Register,
                                                                    ImmediateSize::None,
                                                                    decoder_flags);
                    *rip += ip_offset;
                    (Opcode::Cmpxchg, argument)
                }
                0xB1 => {
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                    RegOrOpcode::Register,
                                                                    ImmediateSize::None,
                                                                    decoder_flags);
                    *rip += ip_offset;
                    (Opcode::Cmpxchg, argument)
                }
                0xB3 => {
                    let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
                    (Opcode::Btr, argument)
                }
                0xB6 => {
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                        RegOrOpcode::Register,
                                                                        ImmediateSize::None,
                                                                        decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);

                    override_argument_size(&memory, *rip, &mut argument, ArgumentSize::Bit8, &decoder_flags);
                    *rip += ip_offset;
                    (Opcode::Movzx, argument)
                }
                0xB7 => {
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                        RegOrOpcode::Register,
                                                                        ImmediateSize::None,
                                                                        decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    override_argument_size(&memory, *rip, &mut argument, ArgumentSize::Bit16, &decoder_flags);
                    *rip += ip_offset;
                    (Opcode::Movzx, argument)
                }
                0xBA => {
                    // bit manipulation
                    let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                    RegOrOpcode::Opcode,
                                                                    ImmediateSize::Bit8,
                                                                    decoder_flags);
                    *rip += ip_offset;
                    (Opcode::BitManipulation, argument)
                }
                0xBB => {
                    let argument = decode_reg_reg(memory, rip, register_size, decoder_flags);
                    (Opcode::Btc, argument)
                }
                0xBE => {
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                        RegOrOpcode::Register,
                                                                        ImmediateSize::None,
                                                                        decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    override_argument_size(&memory, *rip, &mut argument, ArgumentSize::Bit8, &decoder_flags);
                    *rip += ip_offset;
                    (Opcode::Movsx, argument)
                }
                0xBF => {
                    let (mut argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                                        RegOrOpcode::Register,
                                                                        ImmediateSize::None,
                                                                        decoder_flags | DecoderFlags::REVERSED_REGISTER_DIRECTION);
                    override_argument_size(&memory, *rip, &mut argument, ArgumentSize::Bit16, &decoder_flags);
                    *rip += ip_offset;
                    (Opcode::Movsx, argument)
                }
                unknown => panic!("Unknown instruction: 0F {:X}", unknown),
            }
        }
        0xCC => {
            // abuse int 3 instruction to signal failed test program
            panic!("int3 instruction");
        }
        0xCD => {
            // abuse int X instruction to signal passed test program
            (Opcode::Int, Instruction::default())
        }
        unknown => panic!("Unknown instruction: {:x}", unknown),
    }
}

    fn read_immediate_8bit(memory: &Memory, rip: &mut i64) -> Instruction {
        let immediate = memory.get_i8(*rip, 1) as i64;
        *rip += 2;
        InstructionBuilder::new().first_argument(Argument::Immediate { immediate }).finalize()
    }


    fn get_argument(memory : &Memory, rip: i64,
    register_size: RegisterSize, reg_or_opcode: RegOrOpcode, immediate_size: ImmediateSize, mut decoder_flags: DecoderFlags) -> (Instruction, i64) {
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
                                    decoder_flags |= DecoderFlags::SIB_DISPLACEMENT_ONLY;
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

                        let argument_size = match register_size {
                            RegisterSize::Bit8 => ArgumentSize::Bit8,
                            RegisterSize::Bit16 => ArgumentSize::Bit16,
                            RegisterSize::Bit32 => ArgumentSize::Bit32,
                            RegisterSize::Bit64 => ArgumentSize::Bit64,
                            RegisterSize::Segment => panic!("Unsupported register size"),
                        };
                        let register = if address_mod == 0b00 && rm == 0x5 {
                            Register::RIP
                        } else {
                            let register_size = if decoder_flags.contains(DecoderFlags::ADDRESS_SIZE_OVERRIDE) {
                                RegisterSize::Bit32
                            } else {
                                RegisterSize::Bit64
                            };
                            get_register(rm, register_size, decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                         decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER))
                        };

                        (InstructionBuilder::new().first_argument(Argument::Immediate {
                                 immediate: immediate as i64,
                             })
                             .second_argument(effective_address(sib, register, displacement, decoder_flags))
                             .opcode(register_or_opcode)
                             .explicit_size(argument_size)
                             .finalize(),
                         ip_offset + 1)
                    }
                    ImmediateSize::Bit32 => {
                        assert!(reg_or_opcode == RegOrOpcode::Opcode);
                        let immediate = if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
                            ip_offset += 2;
                            memory.get_i16(rip, ip_offset - 2) as i64
                        } else {
                            ip_offset += 4;
                            memory.get_i32(rip, ip_offset - 4) as i64
                        };

                        let argument_size = match register_size {
                            RegisterSize::Bit8 => ArgumentSize::Bit8,
                            RegisterSize::Bit16 => ArgumentSize::Bit16,
                            RegisterSize::Bit32 => ArgumentSize::Bit32,
                            RegisterSize::Bit64 => ArgumentSize::Bit64,
                            RegisterSize::Segment => panic!("Unsupported register size"),
                        };

                        let register = if address_mod == 0b00 && rm == 0x5 {
                            Register::RIP
                        } else {
                            let register_size = if decoder_flags.contains(DecoderFlags::ADDRESS_SIZE_OVERRIDE) {
                                RegisterSize::Bit32
                            } else {
                                RegisterSize::Bit64
                            };
                            get_register(rm, register_size, decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                         decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER))
                        };

                        (InstructionBuilder::new().first_argument(Argument::Immediate {
                                 immediate: immediate,
                             })
                             .second_argument(effective_address(sib, register, displacement, decoder_flags))
                             .opcode(register_or_opcode)
                             .explicit_size(argument_size)
                             .finalize(),
                         ip_offset)
                    }
                    ImmediateSize::None => {
                        let first_reg_size = if decoder_flags.contains(DecoderFlags::ADDRESS_SIZE_OVERRIDE) {
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
                                         decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                         decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER))
                        };

                        (match reg_or_opcode {
                            RegOrOpcode::Register => {
                                let register2 = get_register(register_or_opcode,
                                                            register_size,
                                                            decoder_flags.contains(DecoderFlags::MOD_R_M_EXTENSION),
                                                            decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER));

                                if decoder_flags.contains(DecoderFlags::REVERSED_REGISTER_DIRECTION) {
                                    InstructionBuilder::new().first_argument(effective_address(sib, register1, displacement, decoder_flags))
                                    .second_argument(
                                        Argument::Register {
                                            register: register2,
                                        }).finalize()
                                } else {
                                    InstructionBuilder::new().first_argument(Argument::Register {
                                            register: register2,
                                        })
                                        .second_argument(effective_address(sib, register1, displacement, decoder_flags))
                                        .finalize()
                                }
                            },
                            RegOrOpcode::Opcode => {
                                InstructionBuilder::new()
                                    .first_argument(effective_address(sib, register1, displacement, decoder_flags))
                                    .opcode(register_or_opcode)
                                    .explicit_size(ArgumentSize::Bit64)
                                    .finalize()
                            }
                        }, ip_offset)
                    }
                }
            }
            0b11 => {
                // register
                let register1 = get_register(modrm & 0b00000111,
                                             register_size,
                                             decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                             decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER));
                let value2 = (modrm & 0b00111000) >> 3;
                match reg_or_opcode {
                    RegOrOpcode::Register => {
                        (if decoder_flags.contains(DecoderFlags::REVERSED_REGISTER_DIRECTION) {
                             InstructionBuilder::new().first_argument(Argument::Register {
                                     register: register1,
                                 })
                                 .second_argument(Argument::Register {
                                     register:
                                         get_register(value2,
                                                      register_size,
                                                      decoder_flags.contains(DecoderFlags::MOD_R_M_EXTENSION), decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                                 })
                                 .finalize()
                         } else {
                             InstructionBuilder::new().first_argument(Argument::Register {
                                     register:
                                         get_register(value2,
                                                      register_size,
                                                      decoder_flags.contains(DecoderFlags::MOD_R_M_EXTENSION), decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER)),
                                 })
                                 .second_argument(Argument::Register {
                                     register: register1,
                                 })
                                 .finalize()
                         },
                         2)
                    }
                    RegOrOpcode::Opcode => {
                        match immediate_size {
                            ImmediateSize::Bit8 => {
                                let immediate = memory.get_i8(rip, 2);
                                (InstructionBuilder::new().first_argument(Argument::Immediate {
                                         immediate: immediate as i64,
                                     })
                                     .second_argument(Argument::Register {
                                         register: register1,
                                     })
                                     .opcode(value2)
                                     .finalize(),
                                 3)
                            }
                            ImmediateSize::Bit32 => {
                                let immediate = memory.get_i32(rip, 2);
                                (InstructionBuilder::new().first_argument(Argument::Immediate {
                                         immediate: immediate as i64,
                                     })
                                     .second_argument(Argument::Register {
                                         register: register1,
                                     })
                                     .opcode(value2)
                                     .finalize(),
                                 6)
                            }
                            ImmediateSize::None => {
                                (InstructionBuilder::new().first_argument(Argument::Register {
                                         register: register1,
                                     })
                                     .opcode(value2)
                                     .finalize(),
                                 2)
                            }
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn effective_address(sib: Option<u8>, register: Register, displacement: i32, decoder_flags: DecoderFlags) -> Argument {
        match sib {
            None => {
                Argument::EffectiveAddress {
                    base: Some(register),
                    index: None,
                    scale: None,
                    displacement: displacement,
                }
            }
            Some(sib) => {
                let base_num = sib & 0b00000111;
                let index = (sib & 0b00111000) >> 3;
                let scale = (sib & 0b11000000) >> 6;
                let scale = 2u8.pow(scale as u32) as u8;

                let register_size = if decoder_flags.contains(DecoderFlags::ADDRESS_SIZE_OVERRIDE) {
                    RegisterSize::Bit32
                } else {
                    RegisterSize::Bit64
                };

                let base = get_register(base_num, register_size,
                                       decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER), false);

                if index == 0x4 {
                    if base_num == 0x5 && decoder_flags.contains(DecoderFlags::SIB_DISPLACEMENT_ONLY) {
                        Argument::EffectiveAddress {
                            base: None,
                            displacement: displacement,
                            scale: None,
                            index: None,
                        }
                    } else {
                        Argument::EffectiveAddress {
                            base: Some(base),
                            displacement: displacement,
                            scale: None,
                            index: None,
                        }
                    }
                } else {
                    if base_num == 0x5 && decoder_flags.contains(DecoderFlags::SIB_DISPLACEMENT_ONLY) {
                        Argument::EffectiveAddress {
                            base: None,
                            displacement: displacement,
                            scale: Some(scale),
                            index: Some(get_register(index, register_size,
                                                    decoder_flags.contains(DecoderFlags::SIB_EXTENSION), false))
                        }
                    } else {
                        Argument::EffectiveAddress {
                            base: Some(base),
                            displacement: displacement,
                            scale: Some(scale),
                            index: Some(get_register(index, register_size,
                                                    decoder_flags.contains(DecoderFlags::SIB_EXTENSION), false))
                        }
                    }
                }
            }
        }
    }

    fn decode_8bit_reg_8bit_immediate(memory: &Memory, rip: &mut i64, decoder_flags: DecoderFlags) -> Instruction {
        let (argument, ip_offset) = get_argument(&memory, *rip, RegisterSize::Bit8,
                                                      RegOrOpcode::Register,
                                                      ImmediateSize::None,
                                                      decoder_flags);
        *rip += ip_offset;
        argument
    }

    fn decode_reg_reg(memory: &Memory, rip: &mut i64, register_size: RegisterSize, decoder_flags: DecoderFlags) -> Instruction {
        let (argument, ip_offset) = get_argument(&memory, *rip, register_size,
                                                      RegOrOpcode::Register,
                                                      ImmediateSize::None,
                                                      decoder_flags);
        *rip += ip_offset;
        argument
    }

    fn decode_al_immediate(memory: &Memory, rip: &mut i64) -> Instruction {
        let immediate = memory.get_i8(*rip, 1);
        let argument =
            InstructionBuilder::new().first_argument(Argument::Immediate {
                    immediate: immediate as i64,
                })
                .second_argument(Argument::Register {
                    register: Register::AL,
                })
                .finalize();
        *rip += 2;
        argument
    }

    fn decode_ax_immediate(memory: &Memory, rip: &mut i64, register_size: RegisterSize, decoder_flags: DecoderFlags) -> Instruction {
        let (immediate, ip_offset) = if decoder_flags.contains(DecoderFlags::OPERAND_16_BIT) {
            (memory.get_i16(*rip, 1) as i64, 3)
        } else {
            (memory.get_i32(*rip, 1) as i64, 5)
        };

        let register = get_register(0,
            register_size, decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
            false);

        let argument =
            InstructionBuilder::new().first_argument(Argument::Immediate {
                    immediate: immediate,
                })
                .second_argument(Argument::Register {
                    register: register,
                })
                .finalize();
        *rip += ip_offset;
        argument
    }

    fn override_argument_size(memory : &Memory, rip: i64, instruction: &mut Instruction, size: ArgumentSize, decoder_flags: &DecoderFlags) {
        let new_first_argument = match instruction.first_argument {
            Some(ref first_argument) => {
                match *first_argument {
                    Argument::Register {..}=> {
                        let register_size = match size {
                            ArgumentSize::Bit8 => RegisterSize::Bit8,
                            ArgumentSize::Bit16 => RegisterSize::Bit16,
                            ArgumentSize::Bit32 => RegisterSize::Bit32,
                            ArgumentSize::Bit64 => RegisterSize::Bit64,
                        };
                        let modrm = memory.get_u8(rip, 1);
                        let register = modrm & 0b00000111;
                        let register = get_register(register, register_size,
                                                    decoder_flags.contains(DecoderFlags::NEW_64BIT_REGISTER),
                                                    decoder_flags.contains(DecoderFlags::NEW_8BIT_REGISTER));
                        Some(Argument::Register {
                            register: register,
                        })
                    },
                    Argument::EffectiveAddress {..} => {
                        instruction.explicit_size = Some(size);
                        None
                    },
                    _ => panic!("Invalid instruction")
                }
            },
            None => panic!("Needs first_argument to override instruction size"),
        };
        match new_first_argument {
            Some(nfa) => instruction.first_argument = Some(nfa),
            None => (),
        }
    }

fn get_register(num: u8, size: RegisterSize, new_64bit_register: bool, new_8bit_register: bool) -> Register {
    match size {
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
                    _ => panic!("Unknown instruction argument"),
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
                    _ => panic!("Unknown instruction argument"),
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
                    _ => panic!("Unknown instruction argument"),
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
                    _ => panic!("Unknown instruction argument"),
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
                    _ => panic!("Unknown instruction argument"),
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
                    _ => panic!("Unknown instruction argument"),
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
                    _ => panic!("Unknown instruction argument"),
                }
            } else {
                if new_8bit_register {
                    match num {
                        0 => Register::AL,
                        1 => Register::CL,
                        2 => Register::DL,
                        3 => Register::BL,
                        4 => Register::SPL,
                        5 => Register::BPL,
                        6 => Register::SIL,
                        7 => Register::DIL,
                        _ => panic!("Unknown instruction argument"),
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
                        _ => panic!("Unknown instruction argument"),
                    }
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
                _ => panic!("Unknown instruction argument"),
            }
        }
    }
}
