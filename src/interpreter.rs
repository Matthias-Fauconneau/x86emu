use crate::instruction::{self, Argument, Instruction, Register, Flags, Repeat, ArgumentSize, get_register_size};
use crate::state::State;

impl State {
pub fn print(&self, instruction: &str) { if self.print_instructions { instruction::print(instruction); } }
pub fn print_no_size(&self, instruction: &str, op: &Instruction) { if self.print_instructions { instruction::print_no_size(instruction, op) } }
pub fn print_(&self, instruction: &str, op: &Instruction) { if self.print_instructions { instruction::print_(instruction, op) } }
}

fn jmp_iml(state: &mut State, op: &Instruction) {
    let first_argument = op.arg();
    let value = state.get_value(&first_argument, op.size());
    match *first_argument {
        Argument::Register { .. } => state.rip = value,
        Argument::Immediate { .. } => state.rip += value,
        Argument::EffectiveAddress { .. } => state.rip = value,
    }
}

fn mov_(state: &mut State, op: &Instruction) {
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let value = state.get_value(&first_argument, argument_size);
    state.set_value(value, second_argument, argument_size);
}

// different instructions with same opcode
pub fn arithmetic(state: &mut State, op: &Instruction) {
    let opcode = match op.opcode {
        Some(opcode) => opcode,
        None => panic!("Unsupported argument type for arithmetic"),
    };
    match opcode {
        0 => add(state, op),
        1 => or(state, op),
        2 => adc(state, op),
        3 => sbb(state, op),
        4 => and(state, op),
        5 => sub(state, op),
        6 => xor(state, op),
        7 => cmp(state, op),
        _ => unreachable!(),
    }
}

pub fn register_operation(state: &mut State, op: &Instruction) {
    let opcode = match op.opcode {
        Some(opcode) => opcode,
        None => panic!("Unsupported argument type for register_operation"),
    };
    match opcode {
        0 => inc(state, op),
        1 => dec(state, op),
        2 => call(state, op),
        3 => call(state, op), // far call
        4 => jmp(state, op),
        5 => jmp(state, op), // far jmp
        6 => push(state, op),
        _ => unreachable!(),
    }
}

pub fn compare_mul_operation(state: &mut State, op: &Instruction) {
    let opcode = match op.opcode {
        Some(opcode) => opcode,
        None => panic!("Unsupported argument type for compare_mul_operation"),
    };
    match opcode {
        0 => test(state, op),
        1 => test(state, op),
        2 => not(state, op),
        3 => neg(state, op),
        4 => mul(state, op),
        5 => imul(state, op),
        6 => div(state, op),
        7 => idiv(state, op),
        _ => unreachable!(),
    }
}

pub fn shift_rotate(state: &mut State, op: &Instruction) {
    let opcode = match op.opcode {
        Some(opcode) => opcode,
        None => panic!("Unsupported argument type for shift_rotate"),
    };
    match opcode {
        0 => rol(state, op),
        1 => ror(state, op),
        2 => rcl(state, op),
        3 => rcr(state, op),
        4 => shl(state, op),
        5 => shr(state, op),
        6 => shl(state, op), // sal and shl are the same
        7 => sar(state, op),
        _ => unreachable!(),
    }
}

pub fn stack_push<T>(state: &mut State, value: &T) {
    state.rsp -= std::mem::size_of::<T>() as i64;
    state.memory.write(state.rsp as u64, value);
}

pub fn stack_pop(state: &mut State) -> i64 {
    let rsp = state.rsp as u64;
    let value = state.memory.read_unaligned(rsp);
    state.rsp += 8;
    value
}

// all other instructions
pub fn push(state: &mut State, op: &Instruction) {
    state.print_("push", &op);
    let value = state.get_value(&op.arg(), op.size());
    match op.size() {
        ArgumentSize::Bit32 => { stack_push(state, &(value as i32)) }
        ArgumentSize::Bit64 => { stack_push(state, &value) }
        _ => panic!("Unsupported push value size"),
    };
}

pub fn pop(state: &mut State, op: &Instruction) {
    state.print_("pop", &op);
    let first_argument = op.arg();
    let value = stack_pop(state);
    state.set_value(value, &first_argument, op.size());
}

pub fn mov(state: &mut State, op: &Instruction) {
    state.print_("mov", &op);
    mov_(state, op);
}

pub fn movsx(state: &mut State, op: &Instruction) {
    state.print_no_size("movsx", &op);
    // normal mov already does the sign extension
    mov_(state, op);
}

pub fn movzx(state: &mut State, op: &Instruction) {
    state.print_no_size("movzx", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let value = state.get_value(&first_argument, argument_size);
    let first_argument_size = match *first_argument {
        Argument::Register {ref register} => {
            get_register_size(register)
        },
        Argument::EffectiveAddress {..} => {
            match op.explicit_size {
                Some(explicit_size) => explicit_size,
                None => panic!("movzx instruction needs explicit size when using an effective address"),
            }
        }
        _ => panic!("Invalid parameter for mov")
    };

    let value = match first_argument_size {
        ArgumentSize::Bit8 => value as u8 as u64,
        ArgumentSize::Bit16 => value as u16 as u64,
        ArgumentSize::Bit32 => value as u32 as u64,
        ArgumentSize::Bit64 => value as u64 as u64,
    };

    // ArgumentSize::Bit64 is not used because target is always a register
    state.set_value(value as i64, second_argument, ArgumentSize::Bit64);
}

fn add_(state: &mut State, value0: i64, value1: i64, argument_size: ArgumentSize) -> i64 {
    let (result, carry, overflow) = match argument_size {
        ArgumentSize::Bit8 => {
            let (result, carry) = (value1 as u8).overflowing_add(value0 as u8);
            let (_, overflow) = (value1 as i8).overflowing_add(value0 as i8);
            (result as i64, carry, overflow)
        }
        ArgumentSize::Bit16 => {
            let (result, carry) = (value1 as u16).overflowing_add(value0 as u16);
            let (_, overflow) = (value1 as i16).overflowing_add(value0 as i16);
            (result as i64, carry, overflow)
        }
        ArgumentSize::Bit32 => {
            let (result, carry) = (value1 as u32).overflowing_add(value0 as u32);
            let (_, overflow) = (value1 as i32).overflowing_add(value0 as i32);
            (result as i64, carry, overflow)
        }
        ArgumentSize::Bit64 => {
            let (result, carry) = (value1 as u64).overflowing_add(value0 as u64);
            let (_, overflow) = (value1 as i64).overflowing_add(value0 as i64);
            (result as i64, carry, overflow)
        }
    };
    state.set_flag(Flags::Carry, carry);
    state.set_flag(Flags::Overflow, overflow);

    state.compute_flags(result, argument_size);
    result
}

pub fn add(state: &mut State, op: &Instruction) {
    state.print_("add", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let value0 = state.get_value(&first_argument, argument_size);
    let value1 = state.get_value(&second_argument, argument_size);
    let result = add_(state, value0, value1, argument_size);
    state.set_value(result, &second_argument, argument_size);
}

pub fn or(state: &mut State, op: &Instruction) {
    state.print_("or", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let value0 = state.get_value(&first_argument, argument_size);
    let value1 = state.get_value(&second_argument, argument_size);
    let result = value0 | value1;
    state.compute_flags(result, argument_size);
    state.set_value(result, &second_argument, argument_size);
}

pub fn adc(state: &mut State, op: &Instruction) {
    state.print_("adc", &op);
    panic!("adc");
}

fn sub__(state: &mut State, value0: i64, value1: i64, argument_size: ArgumentSize) -> i64 {
    let (result, carry, overflow) = match argument_size {
        ArgumentSize::Bit8 => {
            let (result, carry) = (value1 as u8).overflowing_sub(value0 as u8);
            let (_, overflow) = (value1 as i8).overflowing_sub(value0 as i8);
            (result as i64, carry, overflow)
        }
        ArgumentSize::Bit16 => {
            let (result, carry) = (value1 as u16).overflowing_sub(value0 as u16);
            let (_, overflow) = (value1 as i16).overflowing_sub(value0 as i16);
            (result as i64, carry, overflow)
        }
        ArgumentSize::Bit32 => {
            let (result, carry) = (value1 as u32).overflowing_sub(value0 as u32);
            let (_, overflow) = (value1 as i32).overflowing_sub(value0 as i32);
            (result as i64, carry, overflow)
        }
        ArgumentSize::Bit64 => {
            let (result, carry) = (value1 as u64).overflowing_sub(value0 as u64);
            let (_, overflow) = (value1 as i64).overflowing_sub(value0 as i64);
            (result as i64, carry, overflow)
        }
    };
    state.set_flag(Flags::Carry, carry);
    state.set_flag(Flags::Overflow, overflow);
    state.compute_flags(result, argument_size);
    result
}

fn sub_(state: &mut State, op: &Instruction, set: bool) {
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let value0 = state.get_value(&first_argument, argument_size);
    let value1 = state.get_value(&second_argument, argument_size);
    let result = sub__(state, value0, value1, argument_size);
    if set {
        state.set_value(result, &second_argument, argument_size);
    }
}

pub fn sbb(state: &mut State, op: &Instruction) {
    state.print_("sbb", &op);
    sub_(state, op, true);
    // TODO: SBB without carry
}

fn and_(state: &mut State, op: &Instruction, set: bool) {
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let value0 = state.get_value(&first_argument, argument_size);
    let value1 = state.get_value(&second_argument, argument_size);
    let result = value0 & value1;
    state.compute_flags(result, argument_size);
    state.set_flag(Flags::Carry, false);
    state.set_flag(Flags::Overflow, false);
    if set {
        state.set_value(result, &second_argument, argument_size);
    }
}

pub fn and(state: &mut State, op: &Instruction) {
    state.print_("and", &op);
    and_(state, op, true);
}

pub fn sub(state: &mut State, op: &Instruction) {
    state.print_("sub", &op);
    sub_(state, op, true);
}

pub fn xor(state: &mut State, op: &Instruction) {
    state.print_("xor", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let value0 = state.get_value(&first_argument, argument_size);
    let value1 = state.get_value(&second_argument, argument_size);
    let result = value0 ^ value1;
    state.compute_flags(result, argument_size);
    state.set_value(result, &second_argument, argument_size);
}

pub fn cmp(state: &mut State, op: &Instruction) {
    state.print_("cmp", &op);
    sub_(state, op, false);
}

pub fn call(state: &mut State, op: &Instruction) {
    state.print_("call", &op);
    let value = state.rip;
    stack_push(state, &value);
    jmp_iml(state, op);
}

pub fn lea(state: &mut State, op: &Instruction) {
    state.print_("lea", &op);
    let (first_argument, second_argument) = op.args();
    let argument_size = op.size();
    match *first_argument {
        Argument::EffectiveAddress { .. } => {
            let value = state.calculate_effective_address(&first_argument) as i64;
            match *second_argument {
                Argument::Register { .. } => {
                    state.set_value(value, &second_argument, argument_size)
                }
                _ => panic!("Unsupported lea argument"),
            }
        }
        _ => panic!("Unsupported lea argument"),
    }
}

pub fn test(state: &mut State, op: &Instruction) {
    state.print_("test", &op);
    // TODO:  test not fully implemented
    and_(state, op, false);
}

pub fn cmovo(state: &mut State, op: &Instruction) {
    state.print_("cmovo", &op);
    if state.get_flag(Flags::Overflow) {
        mov_(state, op);
    }
}

pub fn cmovno(state: &mut State, op: &Instruction) {
    state.print_("cmovno", &op);
    if !state.get_flag(Flags::Overflow) {
        mov_(state, op);
    }
}

pub fn cmovb(state: &mut State, op: &Instruction) {
    state.print_("cmovb", &op);
    if state.get_flag(Flags::Carry) {
        mov_(state, op);
    }
}

pub fn cmovae(state: &mut State, op: &Instruction) {
    state.print_("cmovae", &op);
    if !state.get_flag(Flags::Carry) {
        mov_(state, op);
    }
}

pub fn cmove(state: &mut State, op: &Instruction) {
    state.print_("cmove", &op);
    if state.get_flag(Flags::Zero) {
        mov_(state, op);
    }
}

pub fn cmovne(state: &mut State, op: &Instruction) {
    state.print_("cmovne", &op);
    if !state.get_flag(Flags::Zero) {
        mov_(state, op);
    }
}

pub fn cmovbe(state: &mut State, op: &Instruction) {
    state.print_("cmovbe", &op);
    if state.get_flag(Flags::Carry) || state.get_flag(Flags::Zero) {
        mov_(state, op);
    }
}

pub fn cmova(state: &mut State, op: &Instruction) {
    state.print_("cmova", &op);
    if !state.get_flag(Flags::Carry) && !state.get_flag(Flags::Zero) {
        mov_(state, op);
    }
}

pub fn cmovs(state: &mut State, op: &Instruction) {
    state.print_("cmovs", &op);
    if state.get_flag(Flags::Sign) {
        mov_(state, op);
    }
}

pub fn cmovns(state: &mut State, op: &Instruction) {
    state.print_("cmovns", &op);
    if !state.get_flag(Flags::Sign) {
        mov_(state, op);
    }
}

pub fn cmovp(state: &mut State, op: &Instruction) {
    state.print_("cmovp", &op);
    if state.get_flag(Flags::Parity) {
        mov_(state, op);
    }
}

pub fn cmovnp(state: &mut State, op: &Instruction) {
    state.print_("cmovnp", &op);
    if !state.get_flag(Flags::Parity) {
        mov_(state, op);
    }
}

pub fn cmovl(state: &mut State, op: &Instruction) {
    state.print_("cmovl", &op);
    if state.get_flag(Flags::Sign) != state.get_flag(Flags::Overflow){
        mov_(state, op);
    }
}

pub fn cmovge(state: &mut State, op: &Instruction) {
    state.print_("cmovge", &op);
    if state.get_flag(Flags::Sign) == state.get_flag(Flags::Overflow){
        mov_(state, op);
    }
}

pub fn cmovle(state: &mut State, op: &Instruction) {
    state.print_("cmovle", &op);
    if state.get_flag(Flags::Zero) ||
            (state.get_flag(Flags::Sign) != state.get_flag(Flags::Overflow)) {
        mov_(state, op);
    }
}

pub fn cmovg(state: &mut State, op: &Instruction) {
    state.print_("cmovg", &op);
    if !state.get_flag(Flags::Zero) &&
            (state.get_flag(Flags::Sign) == state.get_flag(Flags::Overflow)) {
        mov_(state, op);
    }
}

pub fn rol(state: &mut State, op: &Instruction) {
    state.print_("rol", &op);
    panic!("rol");
}

pub fn ror(state: &mut State, op: &Instruction) {
    state.print_("rol", &op);
    panic!("rol");
}

pub fn rcl(state: &mut State, op: &Instruction) {
    state.print_("rcl", &op);
    panic!("rcl");
}

pub fn rcr(state: &mut State, op: &Instruction) {
    state.print_("rcr", &op);
    panic!("rcr");
}

pub fn shl(state: &mut State, op: &Instruction) {
    state.print_("shl", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let mut value0 = state.get_value(&first_argument, argument_size);
    let value1 = state.get_value(&second_argument, argument_size);

    let (result, carry, overflow) = match argument_size {
        ArgumentSize::Bit8 => {
            value0 = value0 % 0x20;
            if value0 > 8 {
                (0, false, false)
            } else if value0 == 8 {
                (0, value1 & 1 == 1, false)
            } else {
                let result = (value1 as u8) << (value0 as u32);
                let bit_position = 8 - value0;
                let (carry, _) = (value1 as u8).overflowing_shr(bit_position as u32);
                let carry = carry & 1 == 1;
                // overflow = most significant bit of result == carry
                let overflow = ((result & 0x80) >> 7 == 1) != carry;
                (result as i64, carry, overflow)
            }
        }
        ArgumentSize::Bit16 => {
            value0 = value0 % 0x20;
            if value0 > 16 {
                (0, false, false)
            } else if value0 == 16 {
                (0, value1 & 1 == 1, false)
            } else {
                let result = (value1 as u16) << (value0 as u32);
                let bit_position = 16 - value0;
                let (carry, _) = (value1 as u16).overflowing_shr(bit_position as u32);
                let carry = carry & 1 == 1;
                // overflow = most significant bit of result == carry
                let overflow = ((result & 0x8000) >> 15 == 1) != carry;
                (result as i64, carry, overflow)
            }
        }
        ArgumentSize::Bit32 => {
            value0 = value0 % 0x20;
            if value0 > 32 {
                (0, false, false)
            } else if value0 == 32 {
                (0, value1 & 1 == 1, false)
            } else {
                let result = (value1 as u32) << (value0 as u32);
                let bit_position = 32 - value0;
                let (carry, _) = (value1 as u32).overflowing_shr(bit_position as u32);
                let carry = carry & 1 == 1;
                // overflow = most significant bit of result == carry
                let overflow = ((result & 0x80000000) >> 31 == 1) != carry;
                (result as i64, carry, overflow)
            }
        }
        ArgumentSize::Bit64 => {
            if value0 > 64 {
                (0, false, false)
            } else if value0 == 64 {
                (0, value1 & 1 == 1, false)
            } else {
                let result = (value1 as u64) << (value0 as u32);
                let bit_position = 64 - value0;
                let (carry, _) = (value1 as u64).overflowing_shr(bit_position as u32);
                let carry = carry & 1 == 1;
                // overflow = most significant bit of result == carry
                let overflow = ((result & 0x8000000000000000) >> 63 == 1) != carry;
                (result as i64, carry, overflow)
            }
        }
    };

    if value0 == 1 {
        state.set_flag(Flags::Overflow, overflow);
    }
    if value0 != 0 {
        state.set_flag(Flags::Carry, carry);
        state.compute_flags(result, argument_size);
    }
    state.set_value(result, &second_argument, argument_size);
}

pub fn shr(state: &mut State, op: &Instruction) {
    state.print_("shr", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let mut value0 = state.get_value(&first_argument, argument_size);
    let value1 = state.get_value(&second_argument, argument_size);

    let (result, carry, overflow) = match argument_size {
        ArgumentSize::Bit8 => {
            value0 = value0 % 0x20;
            if value0 > 8 {
                (0, false, false)
            } else if value0 == 8 {
                (0, value1 & 0x80 == 0x80, false)
            } else {
                let result = (value1 as u8) >> (value0 as u32);
                let (carry, _) = (value1 as u8).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry, value1 & 0x80 == 0x80)
            }
        }
        ArgumentSize::Bit16 => {
            value0 = value0 % 0x20;
            if value0 > 16 {
                (0, false, false)
            } else if value0 == 16 {
                (0, value1 & 0x8000 == 0x8000, false)
            } else {
                let result = (value1 as u16) >> (value0 as u32);
                let (carry, _) = (value1 as u16).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry, value1 & 0x8000 == 0x8000)
            }
        }
        ArgumentSize::Bit32 => {
            value0 = value0 % 0x20;
            if value0 > 32 {
                (0, false, false)
            } else if value0 == 32 {
                (0, value1 & 0x80000000 == 0x80000000, false)
            } else {
                let result = (value1 as u32) >> (value0 as u32);
                let (carry, _) = (value1 as u32).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry, value1 & 0x80000000 == 0x80000000)
            }
        }
        ArgumentSize::Bit64 => {
            if value0 > 64 {
                (0, false, false)
            } else if value0 == 64 {
                (0, value1 as u64 & 0x8000000000000000 == 0x8000000000000000, false)
            } else {
                let result = (value1 as u64) >> (value0 as u32);
                let (carry, _) = (value1 as u64).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry, value1 as u64 & 0x8000000000000000 == 0x8000000000000000)
            }
        }
    };

    if value0 == 1 {
        state.set_flag(Flags::Overflow, overflow);
    }
    if value0 != 0 {
        state.set_flag(Flags::Carry, carry);
        state.compute_flags(result, argument_size);
    }
    state.set_value(result, &second_argument, argument_size);
}

pub fn sar(state: &mut State, op: &Instruction) {
    state.print_("sar", &op);
    let argument_size = op.size();
    let args = op.args();
    let mut value0 = state.get_value(args.0, argument_size);
    let value1 = state.get_value(args.1, argument_size);

    let (result, carry) = match argument_size {
        ArgumentSize::Bit8 => {
            value0 = value0 % 0x20;
            if value0 > 8 {
                (0, false)
            } else if value0 == 8 {
                (0, value1 & 0x80 == 0x80)
            } else {
                let result = (value1 as i8) >> (value0 as u32);
                let (carry, _) = (value1 as u8).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry)
            }
        }
        ArgumentSize::Bit16 => {
            value0 = value0 % 0x20;
            if value0 > 16 {
                (0, false)
            } else if value0 == 16 {
                (0, value1 & 0x8000 == 0x8000)
            } else {
                let result = (value1 as i16) >> (value0 as u32);
                let (carry, _) = (value1 as u16).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry)
            }
        }
        ArgumentSize::Bit32 => {
            value0 = value0 % 0x20;
            if value0 > 32 {
                (0, false)
            } else if value0 == 32 {
                (0, value1 & 0x80000000 == 0x80000000)
            } else {
                let result = (value1 as i32) >> (value0 as u32);
                let (carry, _) = (value1 as u32).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry,)
            }
        }
        ArgumentSize::Bit64 => {
            if value0 > 64 {
                (0, false)
            } else if value0 == 64 {
                (0, value1 as u64 & 0x8000000000000000 == 0x8000000000000000)
            } else {
                let result = (value1 as i64) >> (value0 as u32);
                let (carry, _) = (value1 as u64).overflowing_shr((value0 - 1) as u32);
                let carry = carry & 1 == 1;
                (result as i64, carry)
            }
        }
    };

    if value0 == 1 {
        state.set_flag(Flags::Overflow, false);
    }
    if value0 != 0 {
        state.set_flag(Flags::Carry, carry);
        state.compute_flags(result, argument_size);
    }
    state.set_value(result, &args.1, argument_size);
}

pub fn inc(state: &mut State, op: &Instruction) {
    state.print_("inc", &op);
    let first_argument = op.arg();
    let argument_size = op.size();
    let value = state.get_value(&first_argument, argument_size);
    let carry = state.get_flag(Flags::Carry);
    let result = add_(state, value, 1, argument_size);
    state.set_value(result, &first_argument, argument_size);
    state.set_flag(Flags::Carry, carry);
}

pub fn dec(state: &mut State, op: &Instruction) {
    state.print_("dec", &op);
    let first_argument = op.arg();
    let argument_size = op.size();
    let value = state.get_value(&first_argument, argument_size);
    let carry = state.get_flag(Flags::Carry);
    let result = sub__(state, 1, value, argument_size);
    state.set_value(result, &first_argument, argument_size);
    state.set_flag(Flags::Carry, carry);
}

pub fn div(state: &mut State, op: &Instruction) {
    state.print_("div", &op);
    let argument_size = op.size();
    let divisor = op.arg();
    let divisor = state.get_value(&divisor, argument_size);

    let (reg_lower, reg_upper) = match argument_size {
        ArgumentSize::Bit8 => (Register::AL, Register::AH),
        ArgumentSize::Bit16 => (Register::AX, Register::DX),
        ArgumentSize::Bit32 => (Register::EAX, Register::EDX),
        ArgumentSize::Bit64 => (Register::RAX, Register::RDX),
    };

    let dividend = ((state.get_register_value(&reg_upper) as u128) << 64) | (state.get_register_value(&reg_lower) as u128);
    let quotient = dividend / (divisor as u128);
    if quotient > (u64::MAX as u128) { panic!("floating point error"); }

    let reminder = dividend % (divisor as u128);

    state.set_register_value(&reg_lower, quotient as i64);
    state.set_register_value(&reg_upper, reminder as i64);

    // todo: set flags (including floating point error flags)
}

pub fn idiv(state: &mut State, op: &Instruction) {
    state.print_("idiv", &op);
    panic!("idiv");
}

pub fn mul(state: &mut State, op: &Instruction) {
    state.print_("mul", &op);
    println!("mul");
    let argument_size = op.size();
    let a = match argument_size {
        ArgumentSize::Bit8 => Register::AL,
        ArgumentSize::Bit16 => Register::AX,
        ArgumentSize::Bit32 => Register::EAX,
        ArgumentSize::Bit64 => Register::RAX,
    };
    let source0 = state.get_register_value(&a) as u64;
    let source1 = state.get_value(&op.arg(), argument_size) as u64;
    let result = source0 as u128 * source1 as u128;
    state.compute_flags(result as i64, argument_size);
    state.set_value(result as i64, &Argument::Register{ register: a }, argument_size);
    match argument_size {
        ArgumentSize::Bit8 => {},
        ArgumentSize::Bit16 => state.set_value((result>>16) as i64, &Argument::Register{ register: Register::DX }, argument_size),
        ArgumentSize::Bit32 => state.set_value((result>>32) as i64, &Argument::Register{ register: Register::EDX }, argument_size),
        ArgumentSize::Bit64 => state.set_value((result>>64) as i64, &Argument::Register{ register: Register::RDX }, argument_size),
    };
    // TODO: mul does not set carry/overflow flag
}

pub fn imul(state: &mut State, op: &Instruction) {
    state.print_("imul", &op);
    // TODO: implement one argument version
    let argument_size = op.size();
    let args = op.args();
    let source0 = state.get_value(&args.0, argument_size);
    let source1 = state.get_value(&args.1, argument_size);
    let result = source0 * source1;
    state.compute_flags(result, argument_size);
    match op.arguments.2 {
        Some(ref target) => state.set_value(result, target, argument_size),
        None => state.set_value(result, &args.1, argument_size),
    }
    // TODO:  imul does not set carry/overflow flag
}

pub fn not(state: &mut State, op: &Instruction) {
    state.print_("not", &op);
    let first_argument = op.arg();
    let argument_size = op.size();
    let value = state.get_value(&first_argument, argument_size);
    let result = !value;
    state.set_value(result, &first_argument, argument_size);
}

pub fn neg(state: &mut State, op: &Instruction) {
    state.print_("neg", &op);
    let first_argument = op.arg();
    let argument_size = op.size();
    let value = state.get_value(&first_argument, argument_size);
    let result = sub__(state, value, 0, argument_size);
    state.set_value(result, &first_argument, argument_size);
}

pub fn ret(state: &mut State) {
    state.print("ret");
    let value = stack_pop(state);
    state.rip = value;
}

pub fn lret(state: &mut State) {
    state.print("lret");
    let value = stack_pop(state);
    stack_pop(state); // Code segment
    state.rip = value;
}

pub fn leave(state: &mut State) {
    state.print("leave");
    let value = state.get_register_value(&Register::RBP);
    state.set_register_value(&Register::RSP, value);
    let value = stack_pop(state);
    state.set_register_value(&Register::RBP, value);
}

pub fn pushf(state: &mut State) { let value = state.rflags; stack_push(state, &value); }

pub fn popf(state: &mut State) {
    state.print("popf");
    let value = stack_pop(state);
    state.rflags = value;
}

pub fn std(state: &mut State) {
    state.print("std");
    state.set_flag(Flags::Direction, true);
}

pub fn cld(state: &mut State) {
    state.print("cld");
    state.set_flag(Flags::Direction, false);
}

fn repeat<F:Fn(&mut State)>(state: &mut State, op: &Instruction, f: F) {
    match op.repeat {
        Repeat::None => f(state),
        Repeat::Equal => loop {
            let rcx = state.get_value(&Argument::Register { register: Register::RCX }, ArgumentSize::Bit64);
            if rcx == 0 { break; }
            f(state);
            if state.get_flag(Flags::Zero) { break; }
            state.set_register_value(&Register::RCX, rcx - 1);
        },
        _ => unimplemented!(),
    }
}

pub fn stos(state: &mut State, op: &Instruction) {
    repeat(state, op, |state| {
        let to = state.get_value(&Argument::Register { register: Register::RDI }, ArgumentSize::Bit64);
        let length = match op.explicit_size.unwrap() {
            ArgumentSize::Bit8 => {
                state.print("rep stos %al,%es:(%rdi)");
                1
            },
            ArgumentSize::Bit16 => {
                state.print("rep stos %ax,%es:(%rdi)");
                2
            },
            ArgumentSize::Bit32 => {
                state.print("rep stos %eax,%es:(%rdi)");
                4
            },
            ArgumentSize::Bit64 => {
                state.print("rep stos %rax,%es:(%rdi)");
                8
            },
        };

        unimplemented!();
        /*let update = if state.get_flag(Flags::Direction) { -size } else { +size };
        state.set_register_value(&Register::RDI, (to + update) as i64);*/
    })
}

pub fn movs(state: &mut State, op: &Instruction) {
    //if op.repeat  { state.print("repe"); }
    state.print("movs %ds:(%rsi),%es:(%rdi)");
    let movs = |state:&mut State| {
        let from = state.get_value(&Argument::Register { register: Register::RSI }, ArgumentSize::Bit64) as u64;
        let to = state.get_value(&Argument::Register { register: Register::RDI }, ArgumentSize::Bit64) as u64;
        let size = match op.explicit_size.expect("movs need an explicit_size") {
            ArgumentSize::Bit64 => {state.memory.write(to, &state.memory.read::<u64>(from)); 8},
            ArgumentSize::Bit32 => {state.memory.write(to, &state.memory.read::<u32>(from)); 4},
            ArgumentSize::Bit16 => {state.memory.write(to, &state.memory.read::<u16>(from)); 2},
            ArgumentSize::Bit8 =>   {state.memory.write(to, &state.memory.read::<u8  >(from)); 1},
        };
        let update = if state.get_flag(Flags::Direction) { -size as i64 } else { size };
        state.set_register_value(&Register::RSI, from as i64 + update);
        state.set_register_value(&Register::RDI, to as i64 + update);
    };
}

pub fn scas(state: &mut State, op: &Instruction) {
    state.print_("scas", &op);
    repeat(state, op, |state: &mut State| {
        let argument_size = op.size();
        match argument_size {
            ArgumentSize::Bit8 => (),
            _ => panic!("scas: only 8bit values supported")
        }
        let (source_arg, needle) = op.args();
        let source = state.get_value(&source_arg, argument_size);
        let needle = state.get_value(&needle, argument_size);
        sub__(state, source, needle, argument_size);
        state.set_register_value(&Register::RDI, state.get_register_value(&Register::RDI) + if state.get_flag(Flags::Direction) { -1 } else { 1 } );
    })
}

pub fn jmp(state: &mut State, op: &Instruction) {
    state.print_("jmp", &op);
    jmp_iml(state, op);
}

pub fn jo(state: &mut State, op: &Instruction) {
    state.print_("jo", &op);
    if state.get_flag(Flags::Overflow) {
        jmp_iml(state, op);
    }
}

pub fn jno(state: &mut State, op: &Instruction) {
    state.print_("jno", &op);
    if !state.get_flag(Flags::Overflow) {
        jmp_iml(state, op);
    }
}

pub fn jb(state: &mut State, op: &Instruction) {
    state.print_("jb", &op);
    if state.get_flag(Flags::Carry) {
        jmp_iml(state, op);
    }
}

pub fn jae(state: &mut State, op: &Instruction) {
    state.print_("jae", &op);
    if !state.get_flag(Flags::Carry) {
        jmp_iml(state, op);
    }
}

pub fn je(state: &mut State, op: &Instruction) {
    state.print_("je", &op);
    if state.get_flag(Flags::Zero) {
        jmp_iml(state, op);
    }
}

pub fn jne(state: &mut State, op: &Instruction) {
    state.print_("jne", &op);
    if !state.get_flag(Flags::Zero) {
        jmp_iml(state, op);
    }
}

pub fn jbe(state: &mut State, op: &Instruction) {
    state.print_("jbe", &op);
    // CF=1 OR ZF=1
    if state.get_flag(Flags::Carry) || state.get_flag(Flags::Zero) {
        jmp_iml(state, op);
    }
}

pub fn ja(state: &mut State, op: &Instruction) {
    state.print_("ja", &op);
    // CF=0 AND ZF=0
    if !state.get_flag(Flags::Carry) && !state.get_flag(Flags::Zero) {
        jmp_iml(state, op);
    }
}

pub fn js(state: &mut State, op: &Instruction) {
    state.print_("js", &op);
    if state.get_flag(Flags::Sign) {
        jmp_iml(state, op);
    }
}

pub fn jns(state: &mut State, op: &Instruction) {
    state.print_("jns", &op);
    if !state.get_flag(Flags::Sign) {
        jmp_iml(state, op);
    }
}

pub fn jp(state: &mut State, op: &Instruction) {
    state.print_("jp", &op);
    if state.get_flag(Flags::Parity) {
        jmp_iml(state, op);
    }
}

pub fn jnp(state: &mut State, op: &Instruction) {
    state.print_("jnp", &op);
    if !state.get_flag(Flags::Parity) {
        jmp_iml(state, op);
    }
}

pub fn jl(state: &mut State, op: &Instruction) {
    // SF!=OF
    state.print_("jl", &op);
    if state.get_flag(Flags::Sign) != state.get_flag(Flags::Overflow){
        jmp_iml(state, op);
    }
}

pub fn jge(state: &mut State, op: &Instruction) {
    // SF=OF
    state.print_("jge", &op);
    if state.get_flag(Flags::Sign) == state.get_flag(Flags::Overflow){
        jmp_iml(state, op);
    }
}

pub fn jle(state: &mut State, op: &Instruction) {
    // (ZF=1) OR (SF!=OF)
    state.print_("jle", &op);
    if state.get_flag(Flags::Zero) ||
            (state.get_flag(Flags::Sign) != state.get_flag(Flags::Overflow)) {
        jmp_iml(state, op);
    }
}

pub fn jg(state: &mut State, op: &Instruction) {
    // (ZF=0) AND (SF=OF)
    state.print_("jg", &op);
    if !state.get_flag(Flags::Zero) &&
            (state.get_flag(Flags::Sign) == state.get_flag(Flags::Overflow)) {
        jmp_iml(state, op);
    }
}

fn set_byte(state: &mut State, op: &Instruction, set: bool) {
    let first_argument = op.arg();
    if set {
        state.set_value(1, &first_argument, ArgumentSize::Bit8);
    } else {
        state.set_value(0, &first_argument, ArgumentSize::Bit8);
    }
}

pub fn seto(state: &mut State, op: &Instruction) {
    state.print_("seto", &op);
    let set = state.get_flag(Flags::Overflow);
    set_byte(state, op, set);
}

pub fn setno(state: &mut State, op: &Instruction) {
    state.print_("setno", &op);
    let set = !state.get_flag(Flags::Overflow);
    set_byte(state, op, set);
}

pub fn setb(state: &mut State, op: &Instruction) {
    state.print_("setb", &op);
    let set = state.get_flag(Flags::Carry);
    set_byte(state, op, set);
}

pub fn setae(state: &mut State, op: &Instruction) {
    state.print_("setae", &op);
    let set = !state.get_flag(Flags::Carry);
    set_byte(state, op, set);
}

pub fn sete(state: &mut State, op: &Instruction) {
    state.print_("sete", &op);
    let set = state.get_flag(Flags::Zero);
    set_byte(state, op, set);
}

pub fn setne(state: &mut State, op: &Instruction) {
    state.print_("setne", &op);
    let set = !state.get_flag(Flags::Zero);
    set_byte(state, op, set);
}

pub fn setbe(state: &mut State, op: &Instruction) {
    state.print_("setbe", &op);
    let set = state.get_flag(Flags::Carry) || state.get_flag(Flags::Zero);
    set_byte(state, op, set);
}

pub fn seta(state: &mut State, op: &Instruction) {
    state.print_("seta", &op);
    let set = !state.get_flag(Flags::Carry) && !state.get_flag(Flags::Zero);
    set_byte(state, op, set);
}

pub fn sets(state: &mut State, op: &Instruction) {
    state.print_("sets", &op);
    let set = state.get_flag(Flags::Sign);
    set_byte(state, op, set);
}

pub fn setns(state: &mut State, op: &Instruction) {
    state.print_("setns", &op);
    let set = !state.get_flag(Flags::Sign);
    set_byte(state, op, set);
}

pub fn setp(state: &mut State, op: &Instruction) {
    state.print_("setp", &op);
    let set = state.get_flag(Flags::Parity);
    set_byte(state, op, set);
}

pub fn setnp(state: &mut State, op: &Instruction) {
    state.print_("setnp", &op);
    let set = !state.get_flag(Flags::Parity);
    set_byte(state, op, set);
}

pub fn setl(state: &mut State, op: &Instruction) {
    state.print_("setl", &op);
    let set = state.get_flag(Flags::Sign) != state.get_flag(Flags::Overflow);
    set_byte(state, op, set);
}

pub fn setge(state: &mut State, op: &Instruction) {
    state.print_("setge", &op);
    let set = state.get_flag(Flags::Sign) == state.get_flag(Flags::Overflow);
    set_byte(state, op, set);
}

pub fn setle(state: &mut State, op: &Instruction) {
    state.print_("setle", &op);
    let set = state.get_flag(Flags::Zero) ||
            (state.get_flag(Flags::Sign) != state.get_flag(Flags::Overflow));
    set_byte(state, op, set);
}

pub fn setg(state: &mut State, op: &Instruction) {
    state.print_("setg", &op);
    let set = !state.get_flag(Flags::Zero) &&
            (state.get_flag(Flags::Sign) == state.get_flag(Flags::Overflow));
    set_byte(state, op, set);
}

pub fn out(state: &mut State) {
    state.print("out   %al,(%dx)");
    let al = state.get_register_value(&Register::AL);
    let dx = state.get_register_value(&Register::DX);
    //println!("AL: {:x}, DX: {:x}", al as u8, dx);
}

pub fn wrmsr(state: &mut State) {
    state.print("wrmsr");
    // save_state(state, "state.bin");
    // panic!("machine state saved!");
    // todo: ement instruction
}

pub fn rdmsr(state: &mut State) {
    state.print("rdmsr");
    let ecx = state.get_register_value(&Register::RCX);
    match ecx {
        0xC0000080 => {
            state.set_register_value(&Register::RAX, 0x500);
            state.set_register_value(&Register::RDX, 0x0);
        }
        _ => {
            panic!("RDMSR: unsupported operand: {:x}", ecx);
        }
    }
}

pub fn bit_manipulation(state: &mut State, op: &Instruction) {
    let opcode = match op.opcode {
        Some(opcode) => opcode,
        None => panic!("Unsupported argument type for arithmetic"),
    };
    match opcode {
        4 => bt(state, op),
        5 => bts(state, op),
        6 => btr(state, op),
        7 => btc(state, op),
        _ => panic!("Invalid opcode for bt instructions"),
    }
}

/// normalize bit_position,
/// get current value of bit at bit_position (after normalization)
fn bt_prepare(bit_position: i64, op: i64, argument_size: ArgumentSize) -> (i64, bool) {
    let bit_position = match argument_size {
        ArgumentSize::Bit8 => bit_position % 8,
        ArgumentSize::Bit16 => bit_position % 16,
        ArgumentSize::Bit32 => bit_position % 32,
        ArgumentSize::Bit64 => bit_position % 64,
    };

    let bit = ((op >> bit_position) & 1) == 1;
    (bit_position, bit)
}

pub fn bt(state: &mut State, op: &Instruction) {
    state.print_("bt", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let bit_position = state.get_value(&first_argument, argument_size);
    let op = state.get_value(&second_argument, argument_size);
    let (_, bit) = bt_prepare(bit_position, op, argument_size);
    state.set_flag(Flags::Carry, bit);
}

// bit_manipulation: closure which takes the current bit value and modifies it depending on the instruciton
fn btx_<F>(state: &mut State, op: &Instruction, bit_manipulation: F)
    where F: FnOnce(bool) -> bool
{
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let bit_position = state.get_value(&first_argument, argument_size);
    let mut op = state.get_value(&second_argument, argument_size);

    let (bit_position, bit) = bt_prepare(bit_position, op, argument_size);

    state.set_flag(Flags::Carry, bit);

    let bit = bit_manipulation(bit);

    if bit {
        op |= 1 << bit_position;
    } else {
        op &= !(1 << bit_position);
    }
    state.set_value(op as i64, &second_argument, argument_size);
}

pub fn bts(state: &mut State, op: &Instruction) {
    state.print_("bts", &op);
    btx_(state, op, | _ | true);
}

pub fn btr(state: &mut State, op: &Instruction) {
    state.print_("btr", &op);
    btx_(state, op, | _ | false);
}

pub fn btc(state: &mut State, op: &Instruction) {
    state.print_("btc", &op);
    btx_(state, op, | b | !b);
}

pub fn cmpxchg(state: &mut State, op: &Instruction) {
    state.print_("cmpxchg", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let source = state.get_value(&first_argument, argument_size);
    let destination = state.get_value(&second_argument, argument_size);

    let accumulator_type = match argument_size {
        ArgumentSize::Bit8 => Register::AL,
        ArgumentSize::Bit16 => Register::AX,
        ArgumentSize::Bit32 => Register::EAX,
        ArgumentSize::Bit64 => Register::RAX,
    };
    let accumulator = state.get_register_value(&accumulator_type);

    if accumulator == destination {
        state.set_flag(Flags::Zero, true);
        state.set_value(source, &second_argument, argument_size);
    } else {
        state.set_flag(Flags::Zero, false);
        state.set_register_value(&accumulator_type, destination);
    }
}

pub fn xchg(state: &mut State, op: &Instruction) {
    state.print_("xchg", &op);
    let argument_size = op.size();
    let (first_argument, second_argument) = op.args();
    let arg1 = state.get_value(&first_argument, argument_size);
    let arg2 = state.get_value(&second_argument, argument_size);

    state.set_value(arg2, &first_argument, argument_size);
    state.set_value(arg1, &second_argument, argument_size);
}

pub fn syscall(state: &mut State) {
    let rax = state.get_register_value(&Register::RAX);

    let p1 = state.get_register_value(&Register::RDI) as u64;
    let p2 = state.get_register_value(&Register::RSI) as u64;
    let p3 = state.get_register_value(&Register::RDX) as u64;
    // let p4 = state.get_register_value(&Register::RCX) as u64;
    // let p5 = state.get_register_value(&Register::R8) as u64;
    // let p6 = state.get_register_value(&Register::R9) as u64;

    match rax {
        _ => panic!("unsupported syscall: {}", rax),
    }
}

pub fn lgdt(state: &mut State, op: &Instruction) {
    state.print_no_size("lgdt", &op);
    let first_argument = op.arg();
    state.gdt = state.get_value(first_argument, ArgumentSize::Bit64);
}

pub fn lidt(state: &mut State, op: &Instruction) {
    state.print_no_size("lidt", &op);
    let first_argument = op.arg();
    state.idt = state.get_value(first_argument, ArgumentSize::Bit64);
}

pub fn cpuid(state: &mut State) {
    state.print("cpuid");
    let value = state.get_register_value(&Register::RAX);
    match value {
        0 => {
            state.set_register_value(&Register::EAX, 1000);
            state.set_register_value(&Register::EBX, 0x756e6547);
            state.set_register_value(&Register::EDX, 0x49656e69);
            state.set_register_value(&Register::ECX, 0x6c65746e);
        },
        1 => {
            let edx = 1 << 0 | // Onboard x87 FPU
                        0 << 1 | // Virtual 8086 mode extensions (such as VIF, VIP, PIV)
                        0 << 2 | // Debugging extensions (CR4 bit 3)
                        1 << 3 | // Page Size Extension
                        0 << 4 | // Time Stamp Counter
                        1 << 5 | // Model-specific registers
                        1 << 6 | // Physical Address Extension
                        0 << 7 | //  Check Exception
                        1 << 8 | // CMPXCHG8 (compare-and-swap) instruction
                        1 << 9 | // Onboard Advanced Programmable Interrupt Controller
                        0 << 10 | // Reserved
                        0 << 11 | // SYSENTER and SYSEXIT instructions
                        0 << 12 | // Memory Type Range Registers
                        0 << 13 | // Page Global Enable bit in CR4
                        0 << 14 | //  check architecture
                        1 << 15 | // Conditional move and FCMOV instructions
                        0 << 16 | // Page Attribute Table
                        0 << 17 | // 36-bit page size extension
                        0 << 18 | // Processor Serial Number
                        0 << 19 | // CLFLUSH instruction (SSE2)
                        0 << 20 | // Reserved
                        0 << 21 | // Debug store: save trace of executed jumps
                        0 << 22 | // Onboard thermal control MSRs for ACPI
                        0 << 23 | // MMX instructions
                        1 << 24 | // FXSAVE, FXRESTOR instructions, CR4 bit 9
                        1 << 25 | // SSE instructions (a.k.a. Katmai New Instructions)
                        1 << 26 | // SSE2 instructions
                        0 << 27 | // CPU cache supports self-snoop
                        0 << 28 | // Hyper-threading
                        0 << 29 | // Thermal monitor automatically limits temperature
                        0 << 30 | // IA64 processor emulating x86
                        0 << 31; // Pending Break Enable (PBE# pin) wakeup support

            let ecx = 0 << 0 | // Prescott New Instructions-SSE3 (PNI)
                        0 << 1 | // PCLMULQDQ support
                        0 << 2 | // 64-bit debug store (edx bit 21)
                        0 << 3 | // MONITOR and MWAIT instructions (SSE3)
                        0 << 4 | // CPL qualified debug store
                        0 << 5 | // Virtual  eXtensions
                        0 << 6 | // Safer Mode Extensions (LaGrande)
                        0 << 7 | // Enhanced SpeedStep
                        0 << 8 | // Thermal Monitor 2
                        0 << 9 | // Supplemental SSE3 instructions
                        0 << 10 | // L1 Context ID
                        0 << 11 | // Silicon Debug interface
                        0 << 12 | // Fused multiply-add (FMA3)
                        0 << 13 | // CMPXCHG16B instruction
                        0 << 14 | // Can disable sending task priority messages
                        0 << 15 | // Perfmon & debug capability
                        0 << 16 | //
                        0 << 17 | // Process context identifiers (CR4 bit 17)
                        0 << 18 | // Direct cache access for DMA writes[12][13]
                        0 << 19 | // SSE4.1 instructions
                        0 << 20 | // SSE4.2 instructions
                        0 << 21 | // x2APIC support
                        0 << 22 | // MOVBE instruction (big-endian)
                        0 << 23 | // POPCNT instruction
                        0 << 24 | // APIC supports one-shot operation using a TSC deadline value
                        0 << 25 | // AES instruction set
                        0 << 26 | // XSAVE, XRESTOR, XSETBV, XGETBV
                        0 << 27 | // XSAVE enabled by OS
                        0 << 28 | // Advanced Vector Extensions
                        0 << 29 | // F16C (half-precision) FP support
                        0 << 30 | // RDRAND (on-chip random number generator) support
                        0 << 31; // Running on a hypervisor (always 0 on a real CPU, but also with some hypervisors)

            state.set_register_value(&Register::EAX, 0);
            state.set_register_value(&Register::EBX, 0);
            state.set_register_value(&Register::ECX, ecx);
            state.set_register_value(&Register::EDX, edx);
        },
        0x80000000 => {
            state.set_register_value(&Register::EAX, 0x80000001);
        },
        0x80000001 => {
            // let edx = 1 << 29 | // Long mode
            //           1 << 31;  // 3DNow!
            // state.set_register_value(&Register::EDX, edx);
            state.set_register_value(&Register::RAX, 0x663);
            state.set_register_value(&Register::RBX, 0x0);
            state.set_register_value(&Register::RCX, 0x5);
            state.set_register_value(&Register::RDX, 0x2193fbfd);
        }
        _ => panic!("CPUID: unsupported input: {:x}", value),
    }
}

pub fn int(state: &mut State, op: &Instruction) {
    state.print("int    $0x80");
    unimplemented!();
}
