use ztiny_bus::Bus;
pub use ztiny_core::numeric::{AddressType, WordType};
use ztiny_cpu::Cpu;

// SECTION: Machine wrapper
pub struct Machine<S: MachineSpec> {
    cpu: S::Cpu,
    pub bus: Bus<S::Address, S::Word>,
}

impl<S: MachineSpec> Machine<S> {
    /// Execute one machine step by driving the CPU with the bus.
    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    /// Reset the machine by resetting only the CPU for now.
    // NOTE: Bus-level reset is not implemented yet.
    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}

pub trait MachineSpec {
    type Address: AddressType;
    type Word: WordType;
    type Cpu: Cpu<Address = Self::Address, Word = Self::Word>;
    // type Video: VideoDevice;
    // type Audio: AudioDevice;
    // type MainRam: Memory;
    // type Rom: Memory;
}
