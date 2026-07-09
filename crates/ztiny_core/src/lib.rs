//! Shared core utilities used across the `ztiny` workspace.
//!
//! This crate is the foundational dependency for other crates like
//! `ztiny_bus`, `ztiny_cpu`, and `ztiny_machine`.
//!
//! SECTION: Public reexports
#![allow(unused_imports)]

// NOTE: This crate currently exposes a broad set of helpers. As the
// project matures, only commonly shared utilities should remain here.
pub mod clock;
pub mod endian;
pub mod error;
pub mod numeric;
pub mod rgb;
pub mod types;
pub mod util;

pub mod prelude;

pub use clock::*;
pub use endian::*;
pub use error::*;
pub use rgb::*;
pub use types::*;
