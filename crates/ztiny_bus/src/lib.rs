//! Bus-related abstractions for the emulator.
//!
//! This crate defines the device, region, attachment, and bus model used by
//! the machine implementation.

pub mod attachment;
pub mod bus;
pub mod device;
pub mod id;
pub mod map;
pub mod region;

pub use attachment::Attachment;
pub use bus::{Bus, BusAccess};
pub use device::Device;
pub use id::*;
pub use map::AddressMap;
pub use region::Region;
