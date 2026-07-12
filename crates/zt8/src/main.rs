use ztiny_bus::{
    AddressMap, Attachment, Bus, BusAccess, Region, device::MemoryDevices,
};
use ztiny_cpu::Cpu;
use ztiny_machine::{Machine, MachineSpec};

const NOP: u8 = 0x00;
const LDA_IMM: u8 = 0x01;
const LDB_IMM: u8 = 0x02;
const STA_ABS: u8 = 0x03;
const MOV_AB: u8 = 0x04;
const ADD_AB: u8 = 0x05;
const HLT: u8 = 0xff;

#[derive(Default)]
struct Zt8Map {
    attachments: Vec<Attachment<u16>>,
}

impl AddressMap<u16> for Zt8Map {
    fn insert(&mut self, attachment: Attachment<u16>) {
        self.attachments.push(attachment);
    }

    fn lookup(&self, address: u16) -> Option<&Attachment<u16>> {
        self.attachments
            .iter()
            .find(|attachment| attachment.region.contains(address))
    }
}

struct Zt8MachineSpec;

impl MachineSpec for Zt8MachineSpec {
    type Address = u16;
    type Word = u8;
    type Bus = Bus<Self::Address, Self::Word, Zt8Map>;
    type Cpu = Zt8;
}

#[derive(Default)]
pub struct Zt8 {
    // SECTION: CPU state
    pc: u16,
    a: u8,
    b: u8,
    c: u8,
    t: u8,
    halted: bool,
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
        self.halted = false;
    }

    fn fetch<B>(&mut self, bus: &mut B) -> Option<Self::Word>
    where
        B: BusAccess<Address = Self::Address, Word = Self::Word>,
    {
        bus.read(self.pc)
    }

    fn decode<B>(
        &self,
        instruction: Self::Word,
        bus: &mut B,
    ) -> Option<Self::Word>
    where
        B: BusAccess<Address = Self::Address, Word = Self::Word>,
    {
        let _ = bus;
        match instruction {
            NOP | LDA_IMM | LDB_IMM | STA_ABS | MOV_AB | ADD_AB | HLT => {
                Some(instruction)
            }
            _ => None,
        }
    }

    fn execute<B>(&mut self, bus: &mut B, instruction: Self::Word)
    where
        B: BusAccess<Address = Self::Address, Word = Self::Word>,
    {
        match instruction {
            NOP => self.pc = self.pc.wrapping_add(1),
            LDA_IMM => {
                if let Some(value) =
                    self.read_operand(bus, self.pc.wrapping_add(1))
                {
                    self.a = value;
                    self.pc = self.pc.wrapping_add(2);
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            LDB_IMM => {
                if let Some(value) =
                    self.read_operand(bus, self.pc.wrapping_add(1))
                {
                    self.b = value;
                    self.pc = self.pc.wrapping_add(2);
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            STA_ABS => {
                let address = self.read_address(bus, self.pc.wrapping_add(1));
                if let Some(address) = address {
                    let _ = bus.write(address, self.a);
                }
                self.pc = self.pc.wrapping_add(3);
            }
            MOV_AB => {
                self.a = self.b;
                self.pc = self.pc.wrapping_add(1);
            }
            ADD_AB => {
                self.a = self.a.wrapping_add(self.b);
                self.pc = self.pc.wrapping_add(1);
            }
            HLT => {
                self.halted = true;
                self.pc = self.pc.wrapping_add(1);
            }
            _ => self.pc = self.pc.wrapping_add(1),
        }
    }

    fn halted(&mut self) -> bool {
        self.halted
    }
}

impl Zt8 {
    fn read_operand<B>(&self, bus: &mut B, address: u16) -> Option<u8>
    where
        B: BusAccess<Address = u16, Word = u8>,
    {
        bus.read(address)
    }

    fn read_address<B>(&self, bus: &mut B, address: u16) -> Option<u16>
    where
        B: BusAccess<Address = u16, Word = u8>,
    {
        let low = bus.read(address)?;
        let high = bus.read(address.wrapping_add(1))?;
        Some(u16::from_le_bytes([low, high]))
    }

    fn _peek<B>(&self, bus: &mut B) -> Option<u8>
    where
        B: BusAccess<Address = u16, Word = u8>,
    {
        bus.read(self.pc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ztiny_bus::BusAccess;

    struct TestBus {
        data: Vec<u8>,
    }

    impl TestBus {
        fn new(data: Vec<u8>) -> Self {
            Self { data }
        }
    }

    impl BusAccess for TestBus {
        type Address = u16;
        type Word = u8;

        fn read(&mut self, address: Self::Address) -> Option<Self::Word> {
            let index = usize::from(address);
            self.data.get(index).copied()
        }

        fn write(
            &mut self,
            address: Self::Address,
            value: Self::Word,
        ) -> Option<()> {
            let index = usize::from(address);
            if let Some(slot) = self.data.get_mut(index) {
                *slot = value;
                Some(())
            } else {
                None
            }
        }
    }

    #[test]
    fn lda_immediate_sets_register_a() {
        let mut cpu = Zt8::default();
        let mut bus = TestBus::new(vec![LDA_IMM, 0x2a, 0x00]);

        cpu.step(&mut bus);

        assert_eq!(cpu.a, 0x2a);
        assert_eq!(cpu.pc, 2);
    }

    #[test]
    fn add_registers_updates_a() {
        let mut cpu = Zt8::default();
        cpu.a = 0x01;
        cpu.b = 0x02;
        let mut bus = TestBus::new(vec![ADD_AB]);

        cpu.step(&mut bus);

        assert_eq!(cpu.a, 0x03);
        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn hlt_stops_the_cpu() {
        let mut cpu = Zt8::default();
        let mut bus = TestBus::new(vec![HLT]);

        cpu.step(&mut bus);

        assert!(cpu.halted());
    }

    #[test]
    fn machine_runs_program_through_bus_and_memory() {
        let mut bus = Bus::<u16, u8, Zt8Map>::new();
        let mut memory = MemoryDevices::<u16, u8>::default();
        memory.read_only = false;
        let region = Region::new(0x0000, 0x00ff);
        let _ = bus.attach(Box::new(memory), region);

        let mut machine = Machine::<Zt8MachineSpec>::new(Zt8::default(), bus);
        machine.bus.write(0x0000, LDA_IMM).unwrap();
        machine.bus.write(0x0001, 0x2a).unwrap();
        machine.bus.write(0x0002, HLT).unwrap();

        for _ in 0..3 {
            if machine.halted() {
                break;
            }
            machine.step();
        }

        assert!(machine.halted());
        assert_eq!(machine.cpu().a, 0x2a);
    }
}

fn main() {
    let mut bus = Bus::<u16, u8, Zt8Map>::new();
    let mut memory = MemoryDevices::<u16, u8>::default();
    memory.read_only = false;
    let region = Region::new(0x0000, 0x00ff);
    let _ = bus.attach(Box::new(memory), region);

    let mut machine = Machine::<Zt8MachineSpec>::new(Zt8::default(), bus);
    machine.bus.write(0x0000, LDA_IMM).unwrap();
    machine.bus.write(0x0001, 0x2a).unwrap();
    machine.bus.write(0x0002, HLT).unwrap();

    while !machine.halted() {
        machine.step();
    }

    println!("ZT8 halted")
}
