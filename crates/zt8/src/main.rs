use ztiny_bus::Bus;
use ztiny_cpu::Cpu;

pub struct Zt8 {
    // Math registers
    a: u8,
    b: u8,
    c: u8,
    // Test/conditional/branching result
    t: u8,
}

impl Cpu for Zt8 {
    type Address = u16;
    type Word = u8;

    fn reset(&mut self) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.t = 0;
    }

    fn step(&mut self, bus: &mut Bus<Self::Address, Self::Word>) {
        // minimal placeholder: no-op step
        let _ = bus;
    }

    fn halted(&mut self) -> bool {
        false
    }
}

fn main() {
    println!("Sup")
}
