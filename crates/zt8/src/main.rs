use ztiny_bus::Bus;
use ztiny_cpu::Cpu;

pub struct Zt8 {
    // SECTION: CPU state
    pc: u16,
    a: u8,
    b: u8,
    c: u8,
    t: u8,
}

impl Cpu for Zt8 {
    type Address = u16;
    type Word = u8;

    fn reset(&mut self) {
        self.pc = 0;
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.t = 0;
    }

    fn step(&mut self, bus: &mut Bus<Self::Address, Self::Word>) {
        // NOTE: Placeholder step; the CPU does nothing yet.
        let _ = bus;

        // TODO: Decode instruction
        // TODO: Execute instruction
    }

    fn halted(&mut self) -> bool {
        false
    }
}

fn main() {
    println!("Sup")
}
