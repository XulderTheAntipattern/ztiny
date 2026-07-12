use ztiny_bus::BusAccess;
pub use ztiny_core::numeric::{AddressType, WordType};
use ztiny_cpu::Cpu;

// SECTION: Machine wrapper
pub struct Machine<S: MachineSpec> {
    cpu: S::Cpu,
    pub bus: S::Bus,
}

impl<S: MachineSpec> Machine<S> {
    pub fn new(cpu: S::Cpu, bus: S::Bus) -> Self {
        Self { cpu, bus }
    }

    pub fn cpu(&self) -> &S::Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut S::Cpu {
        &mut self.cpu
    }

    pub fn step(&mut self) {
        // TODO: Remove this in favor of running each step within the cpu itself.
        self.cpu.step(&mut self.bus);
    }

    // NOTE: Bus-level reset is not implemented yet.
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn halted(&mut self) -> bool {
        self.cpu.halted()
    }
}

pub trait MachineSpec {
    type Address: AddressType;
    type Word: WordType;
    type Bus: BusAccess<Address = Self::Address, Word = Self::Word>;
    type Cpu: Cpu<Address = Self::Address, Word = Self::Word, Bus = Self::Bus>;
}
