use ztiny_bus::Bus;
use ztiny_core::numeric::{AddressType, WordType};

pub trait Cpu<A: AddressType, W: WordType> {
    fn reset(&mut self);
    fn step(&mut self, bus: &mut Bus<A, W>);
    fn halted(&mut self) -> bool;
}
