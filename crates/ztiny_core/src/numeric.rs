//! Core numeric abstractions for the emulator framework.
//!
//! SECTION: Core traits

/// Address width types supported by the system.
///
/// This trait is implemented for basic integer widths and is used by the
/// bus, devices, and CPU interfaces.
///
// REVIEW: Extend the implementation list if wider address spaces are needed.
pub trait AddressType: Copy + Clone + Eq + Ord + std::fmt::Debug + Default {
    const BITS: u8;

    fn into_usize(self) -> usize;
    fn from_usize(value: usize) -> Option<Self>;
}

impl AddressType for u8 {
    const BITS: u8 = 8;

    fn into_usize(self) -> usize {
        self as usize
    }

    fn from_usize(value: usize) -> Option<Self> {
        Self::try_from(value).ok()
    }
}

impl AddressType for u16 {
    const BITS: u8 = 16;

    fn into_usize(self) -> usize {
        self as usize
    }

    fn from_usize(value: usize) -> Option<Self> {
        Self::try_from(value).ok()
    }
}

// ANCHOR: potential 32-bit support
impl AddressType for u32 {
    const BITS: u8 = 32;

    fn into_usize(self) -> usize {
        self as usize
    }

    fn from_usize(value: usize) -> Option<Self> {
        Self::try_from(value).ok()
    }
}

/// Word width types supported by the system.
///
/// This trait is the element type used for memory reads and writes.
/// It is intentionally similar to `AddressType`.
pub trait WordType: Copy + Clone + Eq + Ord + std::fmt::Debug + Default {
    const BITS: u8;
}

impl WordType for u8 {
    const BITS: u8 = 8;
}

impl WordType for u16 {
    const BITS: u8 = 16;
}

// ANCHOR: support for 32-bit words is implemented here
impl WordType for u32 {
    const BITS: u8 = 32;
}
