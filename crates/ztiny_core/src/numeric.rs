//! Core numeric abstractions for the emulator framework.
//!
//! This crate provides the narrow set of traits used to represent valid
//! address and word widths for the machine model.
//!
// SECTION: Core traits

/// Address width types supported by the system.
///
/// This trait is implemented for basic integer widths and is used by the
/// bus, devices, and CPU interfaces.
///
// REVIEW: If the project later needs 20- or 24-bit address spaces, the
// trait implementation list can be extended here.
pub trait AddressType: Copy + Clone + Eq + Ord + std::fmt::Debug {
    const BITS: u8;
}

impl AddressType for u8 {
    const BITS: u8 = 8;
}

impl AddressType for u16 {
    const BITS: u8 = 16;
}

// ANCHOR: potential 32-bit support
impl AddressType for u32 {
    const BITS: u8 = 32;
}

/// Word width types supported by the system.
///
/// This trait is the element type used for memory reads and writes.
/// It is intentionally similar to `AddressType`.
pub trait WordType: Copy + Clone + Eq + Ord + std::fmt::Debug {
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
