#![allow(unused_imports)]
// NOTE: Anything that is found in more than 2-3 modules should be moved here.
// Attempt at avoiding spaghetti.
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
