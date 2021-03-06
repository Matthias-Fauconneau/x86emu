use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Register {
    // 128 Bit
    XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
    XMM8, XMM9, XMM10, XMM11, XMM12, XMM13, XMM14, XMM15,
    // 64 Bit
    RAX, RBX, RCX, RDX, RSP, RBP, RSI, RDI,
    R8, R9, R10, R11, R12, R13, R14, R15,
    RIP,
    CR0, CR2, CR3, CR4, CR8,
    // 32 Bit
    EAX, EBX, ECX, EDX, ESP, EBP, ESI, EDI,
    R8D, R9D, R10D, R11D, R12D, R13D, R14D, R15D,
    // 16 Bit
    AX, CX, DX, BX, SP, BP, SI, DI,
    R8W, R9W, R10W, R11W, R12W, R13W, R14W, R15W,
    // 8 Bit
    AL, CL, DL, BL, AH, CH, DH, BH,
    SPL, BPL, SIL, DIL,
    R8B, R9B, R10B, R11B, R12B, R13B, R14B, R15B,
    ES, CS, SS, DS, FS, GS,
}

pub enum Flags {
    Carry = 1 /*<< 0*/,
    Parity = 1 << 2,
    Zero = 1 << 6,
    Sign = 1 << 7,
    Direction = 1 << 10,
    Overflow = 1 << 11,
}

#[derive(Debug)] pub enum Repeat { None, Equal, NotEqual }
impl Default for Repeat { fn default() -> Repeat { Repeat::None } }

#[derive(Clone, Copy, Debug)] pub enum RegisterSize { Bit8, Bit16, Bit32, Bit64, Bit128, Segment }
#[derive(Debug, Copy, Clone)] pub enum OperandSize { Bit128, Bit64, Bit32, Bit16, Bit8 }

pub fn get_register_size(reg: Register) -> OperandSize {
	match reg {
		Register::XMM0 | Register::XMM1 |Register::XMM2 |Register::XMM3 |
		Register::XMM4 | Register::XMM5 |Register::XMM6 |Register::XMM7 |
		Register::XMM8 | Register::XMM9 |Register::XMM10 |Register::XMM11 |
		Register::XMM12 | Register::XMM13 |Register::XMM14 |Register::XMM15 => OperandSize::Bit128,

		Register::RAX | Register::RBX | Register::RCX | Register::RDX | Register::RSP |
		Register::RBP | Register::RSI | Register::RDI | Register::RIP | Register::R8 |
		Register::R9 | Register::R10 | Register::R11 | Register::R12 | Register::R13 |
		Register::R14 | Register::R15 | Register::CR0 | Register::CR2 | Register::CR3 |
		Register::CR4 | Register::CR8 => OperandSize::Bit64,

		Register::EAX | Register::EBX | Register::ECX | Register::EDX | Register::ESP |
		Register::EBP | Register::ESI | Register::EDI | Register::R8D | Register::R9D |
		Register::R10D | Register::R11D | Register::R12D | Register::R13D | Register::R14D |
		Register::R15D => OperandSize::Bit32,

		Register::AX | Register::CX | Register::DX | Register::BX | Register::SP |
		Register::BP | Register::SI | Register::DI | Register::R8W | Register::R9W |
		Register::R10W | Register::R11W | Register::R12W | Register::R13W | Register::R14W |
		Register::R15W | Register::ES | Register::CS | Register::SS | Register::DS |
		Register::FS | Register::GS => OperandSize::Bit16,

		Register::AL | Register::CL | Register::DL | Register::BL | Register::AH |
		Register::CH | Register::DH | Register::BH | Register::SPL | Register::BPL |
		Register::SIL | Register::DIL | Register::R8B | Register::R9B |
		Register::R10B | Register::R11B | Register::R12B | Register::R13B | Register::R14B |
		Register::R15B => OperandSize::Bit8,
	}
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug)]
pub enum Operand {
    Immediate(i64),
    Register(Register),
    EffectiveAddress {
        base: Option<Register>,
        index: Option<Register>,
        scale: Option<u8>,
        displacement: i32,
    },
}

impl Operand {
    pub fn format(&self, size: OperandSize) -> String {
        match *self {
            Operand::Register(_) | Operand::EffectiveAddress {..} => format!("{}", self),
            Operand::Immediate(immediate) => {
                format!("$0x{:x}", match size {
                    OperandSize::Bit8 => immediate as u8 as u64,
                    OperandSize::Bit16 => immediate as u16 as u64,
                    OperandSize::Bit32 => immediate as u32 as u64,
                    OperandSize::Bit64 => immediate as u64,
                    _ => unreachable!(),
                })
            }
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operand::Register(ref register) => write!(f, "{}", register),
            Operand::Immediate(immediate) => write!(f, "$0x{:x}", immediate),
            Operand::EffectiveAddress { displacement, .. } => match displacement.cmp(&0) {
                std::cmp::Ordering::Less => write!(f, "-{:#x}{}", displacement.abs(), format_effective_address(self)),
                std::cmp::Ordering::Greater => write!(f, "{:#x}{}", displacement, format_effective_address(self)),
                std::cmp::Ordering::Equal => write!(f, "0x0{}", format_effective_address(self)),
            }
        }
    }
}

#[derive(Default,Debug)]
pub struct Operands {
    pub operands: [Option<Operand>; 3],
    pub opcode: Option<u8>, // modifier (actual instruction is (Opcode, Operands))
    pub explicit_size: Option<OperandSize>,
    pub repeat: Repeat,
}

impl Operands {
    pub fn op(&self) -> &Operand { assert!(self.operands[1].is_none()); self.operands[0].as_ref().unwrap() }
    pub fn operands(&self) -> (&Operand, &Operand) { (self.operands[0].as_ref().unwrap(), self.operands[1].as_ref().unwrap()) }

    pub fn size(&self) -> OperandSize {
        if let Some(explicit_size) = self.explicit_size { return explicit_size; }
        match *self.operands[0].as_ref().unwrap() {
            Operand::Register(register) => { get_register_size(register) }
            Operand::Immediate(_) | Operand::EffectiveAddress { .. } => {
                if let Some(Operand::Register(register)) = self.operands[1] { return get_register_size(register); }
                OperandSize::Bit64
            }
        }
    }
}

impl Operands {
    pub fn fmt(&self, rip : i64) -> String {
        let mut f = String::new();
        use std::fmt::Write;
        if let Some(op0) = &self.operands[0] {
            write!(f, "{}", op0.format(self.size())).unwrap();
            if let Some(op1) = &self.operands[1] { write!(f, ",{}", op1.format(self.size())).unwrap(); }
            if rip != 0 { if let Operand::Immediate(immediate) = op0 { write!(f, " {:x}", rip+immediate).unwrap(); } }
        }
        f
    }
}
impl fmt::Display for Operands {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*if let Some(op0) = &self.operands.0 {
            write!(f, "{}", op0.format(self.size()))?;
            if let Some(op1) = &self.operands.1 { write!(f, ",{}", op1.format(self.size()))?; }
        }
        Ok(())*/
        write!(f, "{}", self.fmt(0))
    }
}

fn format_effective_address(op: &Operand) -> String {
    match *op {
        Operand::EffectiveAddress { ref base, ref index, scale, .. } => {
            match *index {
                None => {
                    match *base {
                        Some(ref base) => format!("({})", base),
                        None => format!(""),
                    }
                }
                Some(ref index) => {
                    match *base {
                        Some(ref base) => format!("({},{},{})", base, index, scale.unwrap()),
                        None => format!("(,{},{})", index, scale.unwrap()),
                    }
                }
            }
        },
        _ => unreachable!()
    }
}

#[derive(Clone,Copy)]
pub enum Opcode {
    Adc,
    Add,
    And,
    Arithmetic,
    BitManipulation,
    Bt,
    Bts,
    Btr,
    Btc,
    Call,
    Cld,
    Cmova,
    Cmovae,
    Cmovb,
    Cmovbe,
    Cmove,
    Cmovg,
    Cmovge,
    Cmovl,
    Cmovle,
    Cmovne,
    Cmovno,
    Cmovnp,
    Cmovns,
    Cmovo,
    Cmovp,
    Cmovs,
    Cmp,
    CompareMulOperation,
    Cpuid,
    Cvtpi2ps,
    Cvttps2pi,
    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    Imul,
    Int,
    Ja,
    Jae,
    Jb,
    Jbe,
    Je,
    Jg,
    Jge,
    Jl,
    Jle,
    Jmp,
    Jne,
    Jno,
    Jnp,
    Jns,
    Jo,
    Jp,
    Js,
    Lea,
    Leave,
    Lidt,
    Lgdt,
    Mov,
    Movs,
    Movd,
    Movss,
    Movsx,
    Movzx,
    Nop,
    Or,
    Out,
    Pop,
    Popf,
    Push,
    Pushf,
    Rdmsr,
    RegisterOperation,
    Ret,
    Lret,
    Sbb,
    ShiftRotate,
    Std,
    Stos,
    Sub,
    Test,
    Wrmsr,
    Xor,
    Scas,
    Cmpxchg,
    Xchg,
    Syscall,
    Seto,
    Setno,
    Setb,
    Setae,
    Sete,
    Setne,
    Setbe,
    Seta,
    Sets,
    Setns,
    Setp,
    Setnp,
    Setl,
    Setge,
    Setle,
    Setg,
    Ud2,
}
