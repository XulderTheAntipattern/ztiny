use ztiny_bus::Bus;
use ztiny_core::numeric::{AddressType, WordType};

// SECTION: CPU trait
// The CPU interface is intentionally small for early-stage machine design.
pub trait Cpu {
    type Address: AddressType;
    type Word: WordType;

    /// Reset CPU state to the initial condition.
    fn reset(&mut self);

    /// Execute one instruction or machine cycle.
    fn step(&mut self, bus: &mut Bus<Self::Address, Self::Word>);

    /// Return whether the CPU has halted.
    fn halted(&mut self) -> bool;
}
