//! ZT8 processor state and instruction execution.

use core::fmt;

use crate::{
    bus::AddressSpace,
    isa::{
        FLAG_CARRY, FLAG_INTERRUPT_DISABLE, FLAG_MASK, FLAG_NEGATIVE, FLAG_OVERFLOW, FLAG_ZERO,
        Register, VECTOR_IRQ, VECTOR_RESET, VECTOR_SWI,
    },
};

/// Address used by the first push after reset. The stack grows into RAM.
pub const RESET_STACK_POINTER: u16 = 0xc000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpuFault {
    IllegalOpcode { pc: u16, opcode: u8 },
}

impl fmt::Display for CpuFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllegalOpcode { pc, opcode } => {
                write!(formatter, "illegal opcode {opcode:#04x} at {pc:#06x}")
            }
        }
    }
}

impl std::error::Error for CpuFault {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepKind {
    Instruction(u8),
    Interrupt,
    Halted,
}

/// Result of one CPU scheduling quantum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Step {
    pub pc: u16,
    pub cycles: u8,
    pub kind: StepKind,
}

/// Complete programmer-visible state for the ZT8 processor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cpu {
    registers: [u8; 4],
    x: u16,
    y: u16,
    sp: u16,
    pc: u16,
    flags: u8,
    halted: bool,
    cycles: u64,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: [0; 4],
            x: 0,
            y: 0,
            sp: RESET_STACK_POINTER,
            pc: 0,
            flags: FLAG_INTERRUPT_DISABLE,
            halted: false,
            cycles: 0,
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets processor state and loads PC from the fixed-ROM reset vector.
    pub fn reset(&mut self, bus: &mut AddressSpace) {
        *self = Self::default();
        self.pc = Self::read_word(bus, VECTOR_RESET);
    }

    pub fn register(&self, register: Register) -> u8 {
        self.registers[register.index()]
    }

    pub fn set_register(&mut self, register: Register, value: u8) {
        self.registers[register.index()] = value;
    }

    pub fn a(&self) -> u8 {
        self.register(Register::A)
    }

    pub fn b(&self) -> u8 {
        self.register(Register::B)
    }

    pub fn c(&self) -> u8 {
        self.register(Register::C)
    }

    pub fn d(&self) -> u8 {
        self.register(Register::D)
    }

    pub const fn x(&self) -> u16 {
        self.x
    }

    pub fn set_x(&mut self, value: u16) {
        self.x = value;
    }

    pub const fn y(&self) -> u16 {
        self.y
    }

    pub fn set_y(&mut self, value: u16) {
        self.y = value;
    }

    pub const fn sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    pub const fn pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub const fn flags(&self) -> u8 {
        self.flags
    }

    pub fn set_flags(&mut self, value: u8) {
        self.flags = value & FLAG_MASK;
    }

    pub const fn flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    pub const fn halted(&self) -> bool {
        self.halted
    }

    pub const fn cycles(&self) -> u64 {
        self.cycles
    }

    /// Executes one instruction, services an IRQ, or idles for one cycle.
    pub fn step(&mut self, bus: &mut AddressSpace, irq: bool) -> Result<Step, CpuFault> {
        if irq && !self.flag(FLAG_INTERRUPT_DISABLE) {
            let pc = self.pc;
            self.halted = false;
            self.enter_interrupt(bus, VECTOR_IRQ);
            return Ok(self.finish_step(pc, 6, StepKind::Interrupt));
        }

        if self.halted {
            return Ok(self.finish_step(self.pc, 1, StepKind::Halted));
        }

        let instruction_pc = self.pc;
        let opcode = self.fetch_byte(bus);
        let cycles = self.execute(bus, instruction_pc, opcode)?;
        Ok(self.finish_step(instruction_pc, cycles, StepKind::Instruction(opcode)))
    }

    fn execute(
        &mut self,
        bus: &mut AddressSpace,
        instruction_pc: u16,
        opcode: u8,
    ) -> Result<u8, CpuFault> {
        let register = usize::from(opcode & 0x03);
        let cycles = match opcode {
            0x00 => 1,
            0x01 => {
                self.halted = true;
                1
            }
            0x02 => {
                self.enter_interrupt(bus, VECTOR_SWI);
                6
            }
            0x03 => {
                self.pc = self.pop_word(bus);
                3
            }
            0x04 => {
                let flags = self.pop_byte(bus);
                self.set_flags(flags);
                self.pc = self.pop_word(bus);
                self.halted = false;
                4
            }
            0x05 => {
                self.set_flag(FLAG_CARRY, false);
                1
            }
            0x06 => {
                self.set_flag(FLAG_CARRY, true);
                1
            }
            0x07 => {
                self.set_flag(FLAG_INTERRUPT_DISABLE, false);
                1
            }
            0x08 => {
                self.set_flag(FLAG_INTERRUPT_DISABLE, true);
                1
            }
            0x09 => {
                self.set_flag(FLAG_OVERFLOW, false);
                1
            }
            0x0a => {
                self.push_byte(bus, self.flags);
                2
            }
            0x0b => {
                let flags = self.pop_byte(bus);
                self.set_flags(flags);
                2
            }
            0x0c => {
                self.x = self.sp;
                self.set_nz_word(self.x);
                1
            }
            0x0d => {
                self.sp = self.x;
                1
            }
            0x0e => {
                self.pc = self.x;
                1
            }
            0x0f => {
                let target = self.x;
                self.push_word(bus, self.pc);
                self.pc = target;
                3
            }
            0x10..=0x13 => {
                let value = self.fetch_byte(bus);
                self.store_register(register, value);
                2
            }
            0x14..=0x17 => {
                let address = self.fetch_word(bus);
                let value = bus.read(address);
                self.store_register(register, value);
                4
            }
            0x18..=0x1b => {
                let address = self.fetch_word(bus);
                bus.write(address, self.registers[register]);
                4
            }
            0x1c..=0x1f => {
                let value = bus.read(self.x);
                self.store_register(register, value);
                2
            }
            0x20..=0x23 => {
                bus.write(self.x, self.registers[register]);
                2
            }
            0x24..=0x27 => {
                let value = bus.read(self.y);
                self.store_register(register, value);
                2
            }
            0x28..=0x2b => {
                bus.write(self.y, self.registers[register]);
                2
            }
            0x2c..=0x2f => {
                self.push_byte(bus, self.registers[register]);
                2
            }
            0x30..=0x33 => {
                let value = self.pop_byte(bus);
                self.store_register(register, value);
                2
            }
            0x34 => {
                self.x = self.fetch_word(bus);
                self.set_nz_word(self.x);
                3
            }
            0x35 => {
                self.y = self.fetch_word(bus);
                self.set_nz_word(self.y);
                3
            }
            0x36 => {
                self.sp = self.fetch_word(bus);
                3
            }
            0x37 => {
                let address = self.fetch_word(bus);
                self.x = Self::read_word(bus, address);
                self.set_nz_word(self.x);
                5
            }
            0x38 => {
                let address = self.fetch_word(bus);
                self.y = Self::read_word(bus, address);
                self.set_nz_word(self.y);
                5
            }
            0x39 => {
                let address = self.fetch_word(bus);
                Self::write_word(bus, address, self.x);
                5
            }
            0x3a => {
                let address = self.fetch_word(bus);
                Self::write_word(bus, address, self.y);
                5
            }
            0x3b => {
                self.x = self.x.wrapping_add(1);
                self.set_nz_word(self.x);
                1
            }
            0x3c => {
                self.x = self.x.wrapping_sub(1);
                self.set_nz_word(self.x);
                1
            }
            0x3d => {
                self.y = self.y.wrapping_add(1);
                self.set_nz_word(self.y);
                1
            }
            0x3e => {
                self.y = self.y.wrapping_sub(1);
                self.set_nz_word(self.y);
                1
            }
            0x3f => {
                core::mem::swap(&mut self.x, &mut self.y);
                self.set_nz_word(self.x);
                1
            }
            0x40..=0x4f => {
                let destination = usize::from((opcode >> 2) & 0x03);
                let source = usize::from(opcode & 0x03);
                let value = self.registers[source];
                self.store_register(destination, value);
                1
            }
            0x50..=0x53 => {
                let address = Self::offset_address(self.x, self.fetch_byte(bus));
                let value = bus.read(address);
                self.store_register(register, value);
                3
            }
            0x54..=0x57 => {
                let address = Self::offset_address(self.x, self.fetch_byte(bus));
                bus.write(address, self.registers[register]);
                3
            }
            0x58..=0x5b => {
                let address = Self::offset_address(self.y, self.fetch_byte(bus));
                let value = bus.read(address);
                self.store_register(register, value);
                3
            }
            0x5c..=0x5f => {
                let address = Self::offset_address(self.y, self.fetch_byte(bus));
                bus.write(address, self.registers[register]);
                3
            }
            0x60..=0x63 => {
                self.add_to_a(self.registers[register], false);
                1
            }
            0x64..=0x67 => {
                self.add_to_a(self.registers[register], self.flag(FLAG_CARRY));
                1
            }
            0x68..=0x6b => {
                self.subtract_from_a(self.registers[register], false, true);
                1
            }
            0x6c..=0x6f => {
                self.subtract_from_a(self.registers[register], !self.flag(FLAG_CARRY), true);
                1
            }
            0x70..=0x73 => {
                self.logic_to_a(self.a() & self.registers[register]);
                1
            }
            0x74..=0x77 => {
                self.logic_to_a(self.a() | self.registers[register]);
                1
            }
            0x78..=0x7b => {
                self.logic_to_a(self.a() ^ self.registers[register]);
                1
            }
            0x7c..=0x7f => {
                self.subtract_from_a(self.registers[register], false, false);
                1
            }
            0x80 => {
                let value = self.fetch_byte(bus);
                self.add_to_a(value, false);
                2
            }
            0x81 => {
                let value = self.fetch_byte(bus);
                self.add_to_a(value, self.flag(FLAG_CARRY));
                2
            }
            0x82 => {
                let value = self.fetch_byte(bus);
                self.subtract_from_a(value, false, true);
                2
            }
            0x83 => {
                let value = self.fetch_byte(bus);
                self.subtract_from_a(value, !self.flag(FLAG_CARRY), true);
                2
            }
            0x84 => {
                let value = self.fetch_byte(bus);
                self.logic_to_a(self.a() & value);
                2
            }
            0x85 => {
                let value = self.fetch_byte(bus);
                self.logic_to_a(self.a() | value);
                2
            }
            0x86 => {
                let value = self.fetch_byte(bus);
                self.logic_to_a(self.a() ^ value);
                2
            }
            0x87 => {
                let value = self.fetch_byte(bus);
                self.subtract_from_a(value, false, false);
                2
            }
            0x88..=0x8b => {
                let value = self.registers[register].wrapping_add(1);
                self.store_register(register, value);
                1
            }
            0x8c..=0x8f => {
                let value = self.registers[register].wrapping_sub(1);
                self.store_register(register, value);
                1
            }
            0x90..=0x93 => {
                let value = self.registers[register];
                self.set_flag(FLAG_CARRY, value & 0x80 != 0);
                self.set_flag(FLAG_OVERFLOW, false);
                self.store_register(register, value << 1);
                1
            }
            0x94..=0x97 => {
                let value = self.registers[register];
                self.set_flag(FLAG_CARRY, value & 0x01 != 0);
                self.set_flag(FLAG_OVERFLOW, false);
                self.store_register(register, value >> 1);
                1
            }
            0x98..=0x9b => {
                let value = self.registers[register];
                let carry_in = u8::from(self.flag(FLAG_CARRY));
                self.set_flag(FLAG_CARRY, value & 0x80 != 0);
                self.set_flag(FLAG_OVERFLOW, false);
                self.store_register(register, (value << 1) | carry_in);
                1
            }
            0x9c..=0x9f => {
                let value = self.registers[register];
                let carry_in = u8::from(self.flag(FLAG_CARRY)) << 7;
                self.set_flag(FLAG_CARRY, value & 0x01 != 0);
                self.set_flag(FLAG_OVERFLOW, false);
                self.store_register(register, (value >> 1) | carry_in);
                1
            }
            0xa0 => {
                self.x = Self::offset_address(self.x, self.fetch_byte(bus));
                self.set_nz_word(self.x);
                2
            }
            0xa1 => {
                self.y = Self::offset_address(self.y, self.fetch_byte(bus));
                self.set_nz_word(self.y);
                2
            }
            0xa2 => {
                let value = self.fetch_word(bus);
                self.compare_word(self.x, value);
                3
            }
            0xa3 => {
                let value = self.fetch_word(bus);
                self.compare_word(self.y, value);
                3
            }
            0xa4 => {
                self.pc = self.fetch_word(bus);
                3
            }
            0xa5 => {
                let target = self.fetch_word(bus);
                self.push_word(bus, self.pc);
                self.pc = target;
                5
            }
            0xa6 => {
                self.x = (self.x & 0xff00) | u16::from(self.a());
                self.set_nz_word(self.x);
                1
            }
            0xa7 => {
                self.x = (self.x & 0x00ff) | (u16::from(self.a()) << 8);
                self.set_nz_word(self.x);
                1
            }
            0xa8 => {
                self.store_register(Register::A.index(), self.x as u8);
                1
            }
            0xa9 => {
                self.store_register(Register::A.index(), (self.x >> 8) as u8);
                1
            }
            0xaa => {
                self.y = (self.y & 0xff00) | u16::from(self.a());
                self.set_nz_word(self.y);
                1
            }
            0xab => {
                self.y = (self.y & 0x00ff) | (u16::from(self.a()) << 8);
                self.set_nz_word(self.y);
                1
            }
            0xac => {
                self.store_register(Register::A.index(), self.y as u8);
                1
            }
            0xad => {
                self.store_register(Register::A.index(), (self.y >> 8) as u8);
                1
            }
            0xae => {
                self.x = self.x.wrapping_add(u16::from(self.a()));
                self.set_nz_word(self.x);
                1
            }
            0xaf => {
                self.y = self.y.wrapping_add(u16::from(self.a()));
                self.set_nz_word(self.y);
                1
            }
            0xb0..=0xbe => {
                let offset = self.fetch_byte(bus);
                let take = match opcode {
                    0xb0 => true,
                    0xb1 => self.flag(FLAG_ZERO),
                    0xb2 => !self.flag(FLAG_ZERO),
                    0xb3 => self.flag(FLAG_CARRY),
                    0xb4 => !self.flag(FLAG_CARRY),
                    0xb5 => self.flag(FLAG_NEGATIVE),
                    0xb6 => !self.flag(FLAG_NEGATIVE),
                    0xb7 => self.flag(FLAG_OVERFLOW),
                    0xb8 => !self.flag(FLAG_OVERFLOW),
                    0xb9 => self.flag(FLAG_CARRY) && !self.flag(FLAG_ZERO),
                    0xba => !self.flag(FLAG_CARRY) || self.flag(FLAG_ZERO),
                    0xbb => self.flag(FLAG_NEGATIVE) == self.flag(FLAG_OVERFLOW),
                    0xbc => self.flag(FLAG_NEGATIVE) != self.flag(FLAG_OVERFLOW),
                    0xbd => {
                        !self.flag(FLAG_ZERO)
                            && self.flag(FLAG_NEGATIVE) == self.flag(FLAG_OVERFLOW)
                    }
                    0xbe => {
                        self.flag(FLAG_ZERO) || self.flag(FLAG_NEGATIVE) != self.flag(FLAG_OVERFLOW)
                    }
                    _ => unreachable!(),
                };
                if take {
                    self.pc = Self::offset_address(self.pc, offset);
                    3
                } else {
                    2
                }
            }
            _ => return Err(CpuFault::IllegalOpcode { pc: instruction_pc, opcode }),
        };
        Ok(cycles)
    }

    fn finish_step(&mut self, pc: u16, cycles: u8, kind: StepKind) -> Step {
        self.cycles = self.cycles.wrapping_add(u64::from(cycles));
        Step { pc, cycles, kind }
    }

    fn fetch_byte(&mut self, bus: &mut AddressSpace) -> u8 {
        let value = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn fetch_word(&mut self, bus: &mut AddressSpace) -> u16 {
        let low = self.fetch_byte(bus);
        let high = self.fetch_byte(bus);
        u16::from_le_bytes([low, high])
    }

    fn read_word(bus: &mut AddressSpace, address: u16) -> u16 {
        let low = bus.read(address);
        let high = bus.read(address.wrapping_add(1));
        u16::from_le_bytes([low, high])
    }

    fn write_word(bus: &mut AddressSpace, address: u16, value: u16) {
        let [low, high] = value.to_le_bytes();
        bus.write(address, low);
        bus.write(address.wrapping_add(1), high);
    }

    fn push_byte(&mut self, bus: &mut AddressSpace, value: u8) {
        self.sp = self.sp.wrapping_sub(1);
        bus.write(self.sp, value);
    }

    fn pop_byte(&mut self, bus: &mut AddressSpace) -> u8 {
        let value = bus.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        value
    }

    fn push_word(&mut self, bus: &mut AddressSpace, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.push_byte(bus, high);
        self.push_byte(bus, low);
    }

    fn pop_word(&mut self, bus: &mut AddressSpace) -> u16 {
        let low = self.pop_byte(bus);
        let high = self.pop_byte(bus);
        u16::from_le_bytes([low, high])
    }

    fn enter_interrupt(&mut self, bus: &mut AddressSpace, vector: u16) {
        self.push_word(bus, self.pc);
        self.push_byte(bus, self.flags);
        self.set_flag(FLAG_INTERRUPT_DISABLE, true);
        self.pc = Self::read_word(bus, vector);
    }

    fn store_register(&mut self, register: usize, value: u8) {
        self.registers[register] = value;
        self.set_nz_byte(value);
    }

    fn set_flag(&mut self, flag: u8, set: bool) {
        if set {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
        self.flags &= FLAG_MASK;
    }

    fn set_nz_byte(&mut self, value: u8) {
        self.set_flag(FLAG_ZERO, value == 0);
        self.set_flag(FLAG_NEGATIVE, value & 0x80 != 0);
    }

    fn set_nz_word(&mut self, value: u16) {
        self.set_flag(FLAG_ZERO, value == 0);
        self.set_flag(FLAG_NEGATIVE, value & 0x8000 != 0);
    }

    fn add_to_a(&mut self, value: u8, carry: bool) {
        let left = self.a();
        let sum = u16::from(left) + u16::from(value) + u16::from(carry);
        let result = sum as u8;
        self.registers[Register::A.index()] = result;
        self.set_nz_byte(result);
        self.set_flag(FLAG_CARRY, sum > 0xff);
        self.set_flag(FLAG_OVERFLOW, (!(left ^ value) & (left ^ result) & 0x80) != 0);
    }

    fn subtract_from_a(&mut self, value: u8, borrow: bool, store: bool) {
        let left = self.a();
        let subtrahend = u16::from(value) + u16::from(borrow);
        let result = left.wrapping_sub(value).wrapping_sub(u8::from(borrow));
        self.set_nz_byte(result);
        self.set_flag(FLAG_CARRY, u16::from(left) >= subtrahend);
        self.set_flag(FLAG_OVERFLOW, ((left ^ value) & (left ^ result) & 0x80) != 0);
        if store {
            self.registers[Register::A.index()] = result;
        }
    }

    fn logic_to_a(&mut self, result: u8) {
        self.registers[Register::A.index()] = result;
        self.set_nz_byte(result);
        self.set_flag(FLAG_OVERFLOW, false);
    }

    fn compare_word(&mut self, left: u16, right: u16) {
        let result = left.wrapping_sub(right);
        self.set_nz_word(result);
        self.set_flag(FLAG_CARRY, left >= right);
        self.set_flag(FLAG_OVERFLOW, ((left ^ right) & (left ^ result) & 0x8000) != 0);
    }

    fn offset_address(base: u16, encoded_offset: u8) -> u16 {
        base.wrapping_add_signed(i16::from(encoded_offset as i8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bus::{IRQ_TIMER, PAGE_SIZE, REG_IRQ_ENABLE, REG_IRQ_PENDING},
        isa::FLAG_CARRY,
    };

    fn program_bus(program: &[u8], start: u16) -> AddressSpace {
        let mut image = vec![0xff; PAGE_SIZE];
        let start_index = usize::from(start);
        image[start_index..start_index + program.len()].copy_from_slice(program);
        image[usize::from(VECTOR_RESET)..usize::from(VECTOR_RESET) + 2]
            .copy_from_slice(&start.to_le_bytes());
        let mut bus = AddressSpace::new(1);
        bus.load_rom(&image).unwrap();
        bus
    }

    fn run_to_halt(cpu: &mut Cpu, bus: &mut AddressSpace) {
        for _ in 0..100 {
            if cpu.halted() {
                return;
            }
            cpu.step(bus, false).unwrap();
        }
        panic!("program did not halt");
    }

    #[test]
    fn executes_load_arithmetic_store_and_halt() {
        let mut bus = program_bus(&[0x10, 0x7f, 0x80, 0x01, 0x18, 0x00, 0x80, 0x01], 0x0100);
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);
        run_to_halt(&mut cpu, &mut bus);

        assert_eq!(cpu.a(), 0x80);
        assert!(cpu.flag(FLAG_NEGATIVE));
        assert!(cpu.flag(FLAG_OVERFLOW));
        assert_eq!(bus.read(0x8000), 0x80);
    }

    #[test]
    fn call_and_return_use_a_little_endian_descending_stack() {
        let mut bus = program_bus(
            &[0xa5, 0x08, 0x01, 0x10, 0x42, 0x01, 0x00, 0x00, 0x10, 0x99, 0x03],
            0x0100,
        );
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);
        run_to_halt(&mut cpu, &mut bus);

        assert_eq!(cpu.a(), 0x42);
        assert_eq!(cpu.sp(), RESET_STACK_POINTER);
        assert_eq!(bus.read(0xbffe), 0x03);
        assert_eq!(bus.read(0xbfff), 0x01);
    }

    #[test]
    fn branches_use_signed_offsets_from_the_end_of_the_instruction() {
        let mut bus = program_bus(&[0x10, 3, 0x8c, 0xb2, 0xfd, 0x01], 0x0100);
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);
        run_to_halt(&mut cpu, &mut bus);

        assert_eq!(cpu.a(), 0);
        assert!(cpu.flag(FLAG_ZERO));
    }

    #[test]
    fn subtraction_sets_carry_when_no_borrow_occurs() {
        let mut bus = program_bus(&[0x10, 5, 0x82, 5, 0x01], 0x0100);
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);
        run_to_halt(&mut cpu, &mut bus);

        assert_eq!(cpu.a(), 0);
        assert!(cpu.flag(FLAG_CARRY));
        assert!(cpu.flag(FLAG_ZERO));
    }

    #[test]
    fn reserved_opcode_reports_a_deterministic_fault() {
        let mut bus = program_bus(&[0xbf], 0x0100);
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);

        assert_eq!(
            cpu.step(&mut bus, false),
            Err(CpuFault::IllegalOpcode { pc: 0x0100, opcode: 0xbf })
        );
    }

    #[test]
    fn irq_and_rti_restore_pc_flags_and_stack() {
        let mut image = vec![0xff; PAGE_SIZE];
        image[0x0100..0x0102].copy_from_slice(&[0x07, 0x01]); // CLI; HLT
        image[0x0200..0x0208].copy_from_slice(&[
            0x10,
            IRQ_TIMER,
            0x18,
            REG_IRQ_PENDING as u8,
            0xc0,
            0x10,
            0x42,
            0x04,
        ]);
        image[usize::from(VECTOR_RESET)..usize::from(VECTOR_RESET) + 2]
            .copy_from_slice(&0x0100u16.to_le_bytes());
        image[usize::from(VECTOR_IRQ)..usize::from(VECTOR_IRQ) + 2]
            .copy_from_slice(&0x0200u16.to_le_bytes());
        let mut bus = AddressSpace::new(1);
        bus.load_rom(&image).unwrap();
        let mut cpu = Cpu::new();
        cpu.reset(&mut bus);

        cpu.step(&mut bus, false).unwrap(); // CLI
        bus.write(REG_IRQ_ENABLE, IRQ_TIMER);
        bus.request_irq(IRQ_TIMER);
        let irq = bus.irq_line();
        assert_eq!(cpu.step(&mut bus, irq).unwrap().kind, StepKind::Interrupt);
        assert_eq!(cpu.pc(), 0x0200);
        assert_eq!(cpu.sp(), RESET_STACK_POINTER - 3);
        cpu.step(&mut bus, false).unwrap(); // LDI A, IRQ_TIMER
        cpu.step(&mut bus, false).unwrap(); // clear pending
        cpu.step(&mut bus, false).unwrap(); // LDI A, 0x42
        cpu.step(&mut bus, false).unwrap(); // RTI

        assert_eq!(cpu.a(), 0x42);
        assert_eq!(cpu.pc(), 0x0101);
        assert_eq!(cpu.sp(), RESET_STACK_POINTER);
        assert!(!cpu.flag(FLAG_INTERRUPT_DISABLE));
        assert!(!bus.irq_line());
        cpu.step(&mut bus, false).unwrap();
        assert!(cpu.halted());
    }
}
