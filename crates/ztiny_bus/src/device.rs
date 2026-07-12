// TODO: This module should probably end up broken up. Honestly I may just move it into it's

use std::marker::PhantomData;

use ztiny_core::numeric::{AddressType, WordType};

// SECTION: Device trait
pub trait Device {
    type Address: AddressType;
    type Word: WordType;

    /// The size of the device's addressable region.
    fn len(&self) -> Self::Address;

    /// Convenience helper to detect an empty device region.
    /// NOTE: This helper is kept for now to avoid warnings.
    fn is_empty(&self) -> bool;

    /// Read a word at a device-local offset.
    ///
    /// The bus is responsible for translating from global addresses.
    fn read(&mut self, offset: Self::Address) -> Self::Word;

    /// Write a word at a device-local offset.
    fn write(&mut self, offset: Self::Address, value: Self::Word);
}

// TODO: Definitely break this up from this module.
pub struct MemoryDevices<A: AddressType, W: WordType> {
    pub data: Box<[W]>,
    pub read_only: bool,
    _address: PhantomData<A>,
}

impl<A, W> MemoryDevices<A, W>
where
    A: AddressType,
    W: WordType + Default,
{
    pub fn new() -> Self {
        // REVIEW: defaulting to usize::MAX is fine and all, but size should be considered
        // NOTE: ^ because I want this capable of spinning many devices.
        let capacity = 1usize.checked_shl(A::BITS as u32).unwrap_or(usize::MAX);
        let mut data = Vec::with_capacity(capacity);
        data.resize_with(capacity, W::default);

        Self {
            data: data.into_boxed_slice(),
            read_only: true,
            _address: PhantomData,
        }
    }
}

impl<A, W> Default for MemoryDevices<A, W>
where
    A: AddressType,
    W: WordType + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A, W> Device for MemoryDevices<A, W>
where
    A: AddressType,
    W: WordType + Default,
{
    type Address = A;
    type Word = W;

    fn len(&self) -> Self::Address {
        A::from_usize(self.data.len()).expect("address type overflow")
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn read(&mut self, offset: Self::Address) -> Self::Word {
        let idx = offset.into_usize();
        self.data.get(idx).copied().unwrap_or_default()
    }

    fn write(&mut self, offset: Self::Address, value: Self::Word) {
        // TODO: Error result should be returned here when error system implemented.
        // NOTE: Yes I know I should do that soon.
        if self.read_only {
            return;
        }

        let idx = offset.into_usize();
        if let Some(slot) = self.data.get_mut(idx) {
            *slot = value;
        }
    }
}
