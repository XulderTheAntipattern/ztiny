//! ZT8 instruction-set constants and encoding helpers.
//!
//! ZT8 is an eight-bit, little-endian CPU with four byte registers
//! ([`Register::A`] through [`Register::D`]), two 16-bit address registers
//! (`X` and `Y`), a 16-bit descending stack pointer, and a 16-bit program
//! counter. Multi-byte instruction operands are stored least-significant byte
//! first.
//!
//! The address space is split into four 16 KiB pages: fixed ROM at
//! `0x0000..=0x3fff`, a banked ROM window at `0x4000..=0x7fff`, work RAM at
//! `0x8000..=0xbfff`, and devices at `0xc000..=0xffff`. The device page
//! contains MMIO at `0xc000..=0xc0ff` and a 128 by 96 RGB332 framebuffer at
//! `0xd000..=0xffff`.
//!
//! Opcode families put a register number in the low two bits. [`mov`] uses
//! bits 3..2 for the destination and bits 1..0 for the source. Singleton
//! instructions are represented by [`Opcode`]; the family base constants and
//! const encoding functions cover the remaining assigned opcodes. Values
//! `0xbf` and `0xc0..=0xff` are reserved.
//!
//! The stack grows downward from its reset value of `0xc000`: a byte push
//! decrements SP before writing and a pop reads before incrementing. A word is
//! pushed high byte first and then low byte, making the low byte the first one
//! popped. Calls push the PC after all operands. IRQ and SWI push PC followed
//! by flags, set the interrupt-disable flag, and load their respective vector;
//! RTI pops flags and then PC. Reset sets only the interrupt-disable flag and
//! loads PC through [`VECTOR_RESET`].
//!
//! Arithmetic uses conventional `N`, `Z`, `C`, and `V` meanings. In SUB, SBC,
//! and CMP, carry means *no borrow*; SBC subtracts `value + !carry`. Relative
//! branches add their signed operand to the PC after the operand fetch, with
//! 16-bit wrapping. Loads and register moves set N/Z, stores do not, and
//! logical operations clear V while preserving C.
//!
//! Cycle counts model byte transfers and have no page-crossing penalty:
//! register/control instructions take 1 cycle, immediate-byte instructions 2,
//! immediate-word instructions 3, absolute byte loads/stores 4, indirect
//! loads/stores 2, offset-indirect operations 3, and absolute word
//! loads/stores 5. A branch takes 2 cycles plus 1 when taken; absolute
//! JMP/CALL take 3/5, indirect JMP/CALL take 1/3, RET takes 3, RTI takes 4,
//! and interrupt entry takes 6. Executing or idling in HALT takes 1 cycle.

use core::fmt;

/// Carry flag. For subtraction, set means that no borrow occurred.
pub const FLAG_CARRY: u8 = 0x01;
/// Zero-result flag.
pub const FLAG_ZERO: u8 = 0x02;
/// Maskable-interrupt disable flag.
pub const FLAG_INTERRUPT_DISABLE: u8 = 0x04;
/// Signed arithmetic overflow flag.
pub const FLAG_OVERFLOW: u8 = 0x40;
/// Negative-result flag (the most-significant result bit).
pub const FLAG_NEGATIVE: u8 = 0x80;
/// Mask of all defined status bits. Undefined bits always read as zero.
pub const FLAG_MASK: u8 =
    FLAG_CARRY | FLAG_ZERO | FLAG_INTERRUPT_DISABLE | FLAG_OVERFLOW | FLAG_NEGATIVE;

/// Little-endian software-interrupt vector in fixed ROM.
pub const VECTOR_SWI: u16 = 0x3ffa;
/// Little-endian maskable-interrupt vector in fixed ROM.
pub const VECTOR_IRQ: u16 = 0x3ffc;
/// Little-endian reset vector in fixed ROM.
pub const VECTOR_RESET: u16 = 0x3ffe;

/// Size of one of the four address-space pages.
pub const PAGE_SIZE: usize = 0x4000;
pub const FIXED_ROM_START: u16 = 0x0000;
pub const FIXED_ROM_END: u16 = 0x3fff;
pub const BANK_WINDOW_START: u16 = 0x4000;
pub const BANK_WINDOW_END: u16 = 0x7fff;
pub const RAM_START: u16 = 0x8000;
pub const RAM_END: u16 = 0xbfff;
pub const DEVICE_PAGE_START: u16 = 0xc000;
pub const DEVICE_PAGE_END: u16 = 0xffff;
pub const MMIO_START: u16 = 0xc000;
pub const MMIO_END: u16 = 0xc0ff;
pub const FRAMEBUFFER_START: u16 = 0xd000;
pub const FRAMEBUFFER_END: u16 = 0xffff;
pub const FRAMEBUFFER_WIDTH: usize = 128;
pub const FRAMEBUFFER_HEIGHT: usize = 96;
pub const FRAMEBUFFER_LEN: usize = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT;

/// An eight-bit general-purpose register.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Register {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

impl Register {
    /// The register's two-bit instruction encoding.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// The register's index in a four-element register file.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Decodes a complete register number. Values with any higher bits set are
    /// rejected rather than silently masked.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::C),
            3 => Some(Self::D),
            _ => None,
        }
    }
}

/// Error returned when a byte is not a valid ZT8 register number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidRegister(pub u8);

impl fmt::Display for InvalidRegister {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid ZT8 register number: {}", self.0)
    }
}

impl std::error::Error for InvalidRegister {}

impl TryFrom<u8> for Register {
    type Error = InvalidRegister;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(InvalidRegister(value))
    }
}

impl From<Register> for u8 {
    fn from(register: Register) -> Self {
        register.code()
    }
}

/// Opcodes that do not encode an eight-bit register in their opcode byte.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0x00,
    Halt = 0x01,
    Swi = 0x02,
    Ret = 0x03,
    Rti = 0x04,
    Clc = 0x05,
    Sec = 0x06,
    Cli = 0x07,
    Sei = 0x08,
    Clv = 0x09,
    PushFlags = 0x0a,
    PopFlags = 0x0b,
    Tsx = 0x0c,
    Txs = 0x0d,
    JumpX = 0x0e,
    CallX = 0x0f,

    LoadXImmediate = 0x34,
    LoadYImmediate = 0x35,
    LoadSpImmediate = 0x36,
    LoadXAbsolute = 0x37,
    LoadYAbsolute = 0x38,
    StoreXAbsolute = 0x39,
    StoreYAbsolute = 0x3a,
    IncrementX = 0x3b,
    DecrementX = 0x3c,
    IncrementY = 0x3d,
    DecrementY = 0x3e,
    SwapXy = 0x3f,

    AddImmediate = 0x80,
    AddCarryImmediate = 0x81,
    SubtractImmediate = 0x82,
    SubtractCarryImmediate = 0x83,
    AndImmediate = 0x84,
    OrImmediate = 0x85,
    XorImmediate = 0x86,
    CompareImmediate = 0x87,

    AddXOffset = 0xa0,
    AddYOffset = 0xa1,
    CompareXImmediate = 0xa2,
    CompareYImmediate = 0xa3,
    JumpAbsolute = 0xa4,
    CallAbsolute = 0xa5,
    LoadXLowA = 0xa6,
    LoadXHighA = 0xa7,
    LoadALowX = 0xa8,
    LoadAHighX = 0xa9,
    LoadYLowA = 0xaa,
    LoadYHighA = 0xab,
    LoadALowY = 0xac,
    LoadAHighY = 0xad,
    AddXA = 0xae,
    AddYA = 0xaf,

    BranchAlways = 0xb0,
    BranchEqual = 0xb1,
    BranchNotEqual = 0xb2,
    BranchCarrySet = 0xb3,
    BranchCarryClear = 0xb4,
    BranchMinus = 0xb5,
    BranchPlus = 0xb6,
    BranchOverflowSet = 0xb7,
    BranchOverflowClear = 0xb8,
    BranchHigher = 0xb9,
    BranchLowerOrSame = 0xba,
    BranchGreaterOrEqual = 0xbb,
    BranchLessThan = 0xbc,
    BranchGreaterThan = 0xbd,
    BranchLessOrEqual = 0xbe,
}

impl Opcode {
    #[must_use]
    pub const fn byte(self) -> u8 {
        self as u8
    }
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> Self {
        opcode.byte()
    }
}

pub const REGISTER_MASK: u8 = 0x03;

pub const LDI_BASE: u8 = 0x10;
pub const LD_ABS_BASE: u8 = 0x14;
pub const ST_ABS_BASE: u8 = 0x18;
pub const LD_X_BASE: u8 = 0x1c;
pub const ST_X_BASE: u8 = 0x20;
pub const LD_Y_BASE: u8 = 0x24;
pub const ST_Y_BASE: u8 = 0x28;
pub const PUSH_BASE: u8 = 0x2c;
pub const POP_BASE: u8 = 0x30;
pub const MOV_BASE: u8 = 0x40;
pub const LD_X_OFFSET_BASE: u8 = 0x50;
pub const ST_X_OFFSET_BASE: u8 = 0x54;
pub const LD_Y_OFFSET_BASE: u8 = 0x58;
pub const ST_Y_OFFSET_BASE: u8 = 0x5c;
pub const ADD_BASE: u8 = 0x60;
pub const ADC_BASE: u8 = 0x64;
pub const SUB_BASE: u8 = 0x68;
pub const SBC_BASE: u8 = 0x6c;
pub const AND_BASE: u8 = 0x70;
pub const OR_BASE: u8 = 0x74;
pub const XOR_BASE: u8 = 0x78;
pub const CMP_BASE: u8 = 0x7c;
pub const INC_BASE: u8 = 0x88;
pub const DEC_BASE: u8 = 0x8c;
pub const SHL_BASE: u8 = 0x90;
pub const SHR_BASE: u8 = 0x94;
pub const ROL_BASE: u8 = 0x98;
pub const ROR_BASE: u8 = 0x9c;

const fn register_opcode(base: u8, register: Register) -> u8 {
    base | register.code()
}

macro_rules! register_encoders {
    ($(#[$meta:meta])* $name:ident, $base:ident; $($(#[$rest_meta:meta])* $rest_name:ident, $rest_base:ident;)*) => {
        $(#[$meta])*
        #[must_use]
        pub const fn $name(register: Register) -> u8 {
            register_opcode($base, register)
        }

        register_encoders!($($(#[$rest_meta])* $rest_name, $rest_base;)*);
    };
    () => {};
}

register_encoders! {
    /// Loads an immediate byte into `register`.
    ldi, LDI_BASE;
    /// Loads `register` from a 16-bit absolute address.
    ld_abs, LD_ABS_BASE;
    /// Stores `register` at a 16-bit absolute address.
    st_abs, ST_ABS_BASE;
    /// Loads `register` through X.
    ld_x, LD_X_BASE;
    /// Stores `register` through X.
    st_x, ST_X_BASE;
    /// Loads `register` through Y.
    ld_y, LD_Y_BASE;
    /// Stores `register` through Y.
    st_y, ST_Y_BASE;
    push, PUSH_BASE;
    pop, POP_BASE;
    /// Loads through X plus a following signed byte offset.
    ld_x_offset, LD_X_OFFSET_BASE;
    /// Stores through X plus a following signed byte offset.
    st_x_offset, ST_X_OFFSET_BASE;
    /// Loads through Y plus a following signed byte offset.
    ld_y_offset, LD_Y_OFFSET_BASE;
    /// Stores through Y plus a following signed byte offset.
    st_y_offset, ST_Y_OFFSET_BASE;
    add, ADD_BASE;
    adc, ADC_BASE;
    sub, SUB_BASE;
    sbc, SBC_BASE;
    and, AND_BASE;
    or, OR_BASE;
    xor, XOR_BASE;
    cmp, CMP_BASE;
    inc, INC_BASE;
    dec, DEC_BASE;
    shl, SHL_BASE;
    shr, SHR_BASE;
    rol, ROL_BASE;
    ror, ROR_BASE;
}

/// Encodes `MOV destination, source`.
#[must_use]
pub const fn mov(destination: Register, source: Register) -> u8 {
    MOV_BASE | (destination.code() << 2) | source.code()
}

/// Extracts the register encoded in the low two bits of a family opcode.
#[must_use]
pub const fn family_register(opcode: u8) -> Register {
    // Masking makes every input valid.
    match opcode & REGISTER_MASK {
        0 => Register::A,
        1 => Register::B,
        2 => Register::C,
        _ => Register::D,
    }
}

/// Extracts the destination register from a MOV opcode.
#[must_use]
pub const fn mov_destination(opcode: u8) -> Register {
    family_register(opcode >> 2)
}

/// Extracts the source register from a MOV opcode.
#[must_use]
pub const fn mov_source(opcode: u8) -> Register {
    family_register(opcode)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTERS: [Register; 4] = [Register::A, Register::B, Register::C, Register::D];

    #[test]
    fn register_numbers_round_trip() {
        for (index, register) in REGISTERS.into_iter().enumerate() {
            assert_eq!(register.index(), index);
            assert_eq!(Register::try_from(index as u8), Ok(register));
        }
        assert_eq!(Register::try_from(4), Err(InvalidRegister(4)));
        assert_eq!(Register::try_from(u8::MAX), Err(InvalidRegister(u8::MAX)));
    }

    #[test]
    fn family_encodings_use_low_two_bits() {
        type RegisterEncoder = fn(Register) -> u8;
        let families: &[(u8, RegisterEncoder)] = &[
            (LDI_BASE, ldi),
            (LD_ABS_BASE, ld_abs),
            (ST_ABS_BASE, st_abs),
            (LD_X_BASE, ld_x),
            (ST_X_BASE, st_x),
            (LD_Y_BASE, ld_y),
            (ST_Y_BASE, st_y),
            (PUSH_BASE, push),
            (POP_BASE, pop),
            (LD_X_OFFSET_BASE, ld_x_offset),
            (ST_X_OFFSET_BASE, st_x_offset),
            (LD_Y_OFFSET_BASE, ld_y_offset),
            (ST_Y_OFFSET_BASE, st_y_offset),
            (ADD_BASE, add),
            (ADC_BASE, adc),
            (SUB_BASE, sub),
            (SBC_BASE, sbc),
            (AND_BASE, and),
            (OR_BASE, or),
            (XOR_BASE, xor),
            (CMP_BASE, cmp),
            (INC_BASE, inc),
            (DEC_BASE, dec),
            (SHL_BASE, shl),
            (SHR_BASE, shr),
            (ROL_BASE, rol),
            (ROR_BASE, ror),
        ];

        for &(base, encode) in families {
            for register in REGISTERS {
                let encoded = encode(register);
                assert_eq!(encoded, base + register.code());
                assert_eq!(family_register(encoded), register);
            }
        }
    }

    #[test]
    fn mov_encodes_both_registers_without_collisions() {
        let mut seen = [false; 16];
        for destination in REGISTERS {
            for source in REGISTERS {
                let encoded = mov(destination, source);
                assert!((MOV_BASE..=MOV_BASE + 0x0f).contains(&encoded));
                assert_eq!(mov_destination(encoded), destination);
                assert_eq!(mov_source(encoded), source);
                let slot = usize::from(encoded - MOV_BASE);
                assert!(!seen[slot]);
                seen[slot] = true;
            }
        }
        assert!(seen.into_iter().all(core::convert::identity));
    }

    #[test]
    fn singleton_encodings_anchor_the_opcode_ranges() {
        assert_eq!(Opcode::Nop.byte(), 0x00);
        assert_eq!(Opcode::CallX.byte(), 0x0f);
        assert_eq!(Opcode::LoadXImmediate.byte(), 0x34);
        assert_eq!(Opcode::SwapXy.byte(), 0x3f);
        assert_eq!(Opcode::AddImmediate.byte(), 0x80);
        assert_eq!(Opcode::CompareImmediate.byte(), 0x87);
        assert_eq!(Opcode::AddXOffset.byte(), 0xa0);
        assert_eq!(Opcode::AddYA.byte(), 0xaf);
        assert_eq!(Opcode::BranchAlways.byte(), 0xb0);
        assert_eq!(Opcode::BranchLessOrEqual.byte(), 0xbe);
    }

    #[test]
    fn flags_vectors_and_framebuffer_layout_are_stable() {
        assert_eq!(FLAG_MASK, 0xc7);
        assert_eq!(VECTOR_IRQ, VECTOR_SWI + 2);
        assert_eq!(VECTOR_RESET, VECTOR_IRQ + 2);
        assert_eq!(usize::from(FRAMEBUFFER_END - FRAMEBUFFER_START) + 1, FRAMEBUFFER_LEN);
        assert_eq!(FRAMEBUFFER_LEN, 12_288);
    }
}
