use ztiny_bus::Bus;
use ztiny_core::numeric::{AddressType, WordType};
use ztiny_cpu::Cpu;

pub struct Machine<A, W, C>
where
    A: AddressType,
    W: WordType,
    C: Cpu<A, W>,
{
    cpu: C,
    bus: Bus<A, W>,
}

impl<A, W, C> Machine<A, W, C>
where
    A: AddressType,
    W: WordType,
    C: Cpu<A, W>,
{
    pub fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}
