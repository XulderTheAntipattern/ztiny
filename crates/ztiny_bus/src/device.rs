use ztiny_core::numeric::{AddressType, WordType};

// SECTION: Device trait
// A minimal device interface for bus-attached components.
pub trait Device {
    type Address: AddressType;
    type Word: WordType;

    /// The size of the device's addressable region.
    fn len(&self) -> Self::Address;

    /// Convenience helper to detect an empty device region.
    // NOTE: This exists to silence warnings; it may be removed later.
    fn is_empty(&self) -> bool;

    /// Read a word at a device-local offset.
    ///
    /// The bus is responsible for translating from global addresses.
    fn read(&mut self, offset: Self::Address) -> Self::Word;

    /// Write a word at a device-local offset.
    fn write(&mut self, offset: Self::Address, value: Self::Word);
}
