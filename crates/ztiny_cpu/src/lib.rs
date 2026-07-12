use ztiny_bus::BusAccess;
use ztiny_core::numeric::{AddressType, WordType};

// TODO: I'd like to clean up these signatures, it's pretty gross
// SECTION: CPU trait
pub trait Cpu {
    type Address: AddressType;
    type Word: WordType;
    type Bus: BusAccess<Address = Self::Address, Word = Self::Word>;

    fn reset(&mut self);

    fn fetch(&mut self, bus: &mut Self::Bus) -> Option<Self::Word>;

    fn decode(
        &self,
        instruction: Self::Word,
        bus: &mut Self::Bus,
    ) -> Option<Self::Word>;

    fn execute(&mut self, bus: &mut Self::Bus, instruction: Self::Word);

    // REVIEW: Likely to be depreciated in favor of the Machine orchestrating.
    /// Execute one instruction or machine cycle using the shared fetch/decode/execute pipeline.
    fn step(&mut self, bus: &mut Self::Bus) {
        if let Some(instruction) = self.fetch(bus) {
            let _ = self.decode(instruction, bus);
            self.execute(bus, instruction);
        }
    }

    /// Return whether the CPU has halted.
    fn halted(&self) -> bool;
}
// !SECTION
