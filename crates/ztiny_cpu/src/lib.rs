use ztiny_bus::BusAccess;
use ztiny_core::numeric::{AddressType, WordType};

// SECTION: CPU trait
pub trait Cpu {
    type Address: AddressType;
    type Word: WordType;

    /// Reset CPU state to the initial condition.
    fn reset(&mut self);

    /// Fetch the next word from an addressable bus implementation.
    fn fetch<B>(&mut self, bus: &mut B) -> Option<Self::Word>
    where
        B: BusAccess<Address = Self::Address, Word = Self::Word>;

    /// Decode a fetched word using the supplied bus abstraction.
    fn decode<B>(
        &self,
        instruction: Self::Word,
        bus: &mut B,
    ) -> Option<Self::Word>
    where
        B: BusAccess<Address = Self::Address, Word = Self::Word>;

    /// Execute a decoded instruction against the supplied bus abstraction.
    fn execute<B>(&mut self, bus: &mut B, instruction: Self::Word)
    where
        B: BusAccess<Address = Self::Address, Word = Self::Word>;

    // REVIEW: Likely to be depreciated in favor of the Machine orchestrating.
    /// Execute one instruction or machine cycle using the shared fetch/decode/execute pipeline.
    fn step<B>(&mut self, bus: &mut B)
    where
        B: BusAccess<Address = Self::Address, Word = Self::Word>,
    {
        if let Some(instruction) = self.fetch(bus) {
            let _ = self.decode(instruction, bus);
            self.execute(bus, instruction);
        }
    }

    /// Return whether the CPU has halted.
    fn halted(&mut self) -> bool;
}
// !SECTION
