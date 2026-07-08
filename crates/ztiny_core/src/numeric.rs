//! Contains traits and types relevant to--you guessed it--NUMBERS!
// NOTE: For some reason I also felt like putting the address and word traits in here too-- Might move them

/// Alias trait for u8-u32
pub trait AddressType: Copy + Clone + Eq + Ord + std::fmt::Debug {
    const BITS: u8;
}

impl AddressType for u8 {
    const BITS: u8 = 8;
}

impl AddressType for u16 {
    const BITS: u8 = 16;
}

// NOTE: For potential 20, 24, and 32 bit support
impl AddressType for u32 {
    const BITS: u8 = 32;
}

/// Alias trait for u8-u32
pub trait WordType: Copy + Clone + Eq + Ord + std::fmt::Debug {
    const BITS: u8;
}

impl WordType for u8 {
    const BITS: u8 = 8;
}

impl WordType for u16 {
    const BITS: u8 = 16;
}

// same as above, not sure if I care to do all that
impl WordType for u32 {
    const BITS: u8 = 32;
}
