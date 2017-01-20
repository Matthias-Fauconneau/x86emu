use instruction_set::{Register, InstructionArgument};
use cpu;

pub fn disassemble(code: &[u8]) {
    let mut instruction_pointer = 0;

    loop {
        let first_byte = code[instruction_pointer];

        let mut rex: Option<REX> = None;

        match first_byte {
            0xF0 | 0xF2 | 0xF3 => panic!("Lock and repeat prefixes/Bound prefix not supported"),
            0x2E | 0x3E | 0x36 | 0x26 | 0x64 | 0x65 => panic!("Segment override prefixes/branch hints not supported"),
            0x66 => panic!("Operand-size override prefix not supported"),
            0x67 => panic!("Address-size override prefix not supported"),
            0x40...0x4F => {  // 64bit REX prefix
                rex = Some(REX{ bits: first_byte });
                instruction_pointer += 1;
            },
            _ => ()
        }

        let first_byte = code[instruction_pointer];
        match first_byte {
            opcode @ 0x50...0x57 => cpu::push(InstructionArgument::OneRegister{ register: get_register(opcode - 0x50) }),
            0x89 => { /* mov */ 
                instruction_pointer += 1;
                cpu::mov(get_two_register_argument(rex, code[instruction_pointer]));
            },
            0x83 => {  /* arithmetic operation (64bit register target, 8bit immediate) */
                let modrm = code[instruction_pointer + 1];
                let register = get_register(modrm & 0b00000111);
                let immediate = code[instruction_pointer + 2];
                let argument = InstructionArgument::Immediate8BitRegister { immediate: immediate, register: register };
                assert!(modrm >> 6 == 0b11);
                match (modrm & 0b00111000) >> 3 {
                    0 => cpu::add(argument),
                    1 => cpu::or(argument),
                    2 => cpu::adc(argument),
                    3 => cpu::sbb(argument),
                    4 => cpu::and(argument),
                    5 => cpu::sub(argument),
                    6 => cpu::xor(argument),
                    7 => cpu::cmp(argument),
                    _ => unreachable!(),
                }
                instruction_pointer += 2;
            },
            0xC7 => {
                let modrm = code[instruction_pointer + 1];
                cpu::mov(get_two_register_argument(rex, modrm));
            }
            _ => panic!("Unknown instruction"),
        }
        instruction_pointer += 1;
    }
}


bitflags! {
    flags REX: u8 {
        const OPERAND_64_BIT = 0b00001000,
        const MOD_R_M_EXTENSION = 0b00000100,
        const SIB_EXTENSION = 0b00000010,
        const B = 0b00000001,
    }
}


fn get_two_register_argument(rex: Option<REX>, modrm: u8) -> InstructionArgument {
    match modrm >> 6 {
        /* effecive address */  0b00 => panic!("effective address not implemented"),
        /* effecive address + 8 bit deplacement */ 0b01 => {
            let register = get_register(modrm & 0b00000111);
            panic!("8bit deplacement not implemented");
        }
        /* effecive address + 32 bit displacement */ 0b10 => panic!("effective address 32bit displacement not implemented"),
        /* register */ 0b11 => {
            let register1 = get_register((modrm & 0b00111000) >> 3);
            let register2 = get_register(modrm & 0b00000111);
            InstructionArgument::TwoRegister{ register1: register1, register2: register2 }
        }
        _ => unreachable!(),
    }
}

fn get_register(num: u8) -> Register {
    match num {
        0 => Register::RAX,
        1 => Register::RBX,
        2 => Register::RCX,
        3 => Register::RDX,
        4 => Register::RSP,
        5 => Register::RBP,
        6 => Register::RSI,
        7 => Register::RDI,
        _ => panic!("Unknown instruction argument"),
    }
}