//! A dependency-free implementation of the ZT8 8-bit virtual machine.
//!
//! The CPU has a 16-bit address bus, an 8-bit data bus, a practical integer
//! instruction set, banked ROM, RAM, MMIO, host input, a cycle timer, and a
//! 128x96 RGB332 framebuffer with a backend-neutral presentation interface.

pub mod bus;
pub mod cpu;
pub mod devices;
pub mod isa;
pub mod machine;

pub use bus::{AddressSpace, LoadError};
pub use cpu::{Cpu, CpuFault, Step, StepKind};
pub use devices::{Button, InputDevice, TimerDevice, VideoBackend, VideoDevice};
pub use isa::{Opcode, Register};
pub use machine::{Machine, RunResult, StopReason};

pub type Address = u16;
pub type Byte = u8;
