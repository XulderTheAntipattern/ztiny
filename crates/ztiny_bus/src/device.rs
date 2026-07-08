use ztiny_core::numeric::{AddressType, WordType};

// TODO: This needs to be moved elsewhere. Maybe an `id` module.

pub trait Device {
    type Address: AddressType;
    type Word: WordType;

    fn len(&self) -> Self::Address;

    // We call it an offset because we're essentially never accessing all of vram at once
    fn read(&mut self, offset: Self::Address) -> Self::Word;
    fn write(&mut self, offset: Self::Address, value: Self::Word);
}
