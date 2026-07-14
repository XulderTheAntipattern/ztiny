//! CPU, address-space, timing, input, and video assembly for a ZT8 machine.

use crate::{
    bus::{AddressSpace, LoadError},
    cpu::{Cpu, CpuFault, Step},
    devices::{Button, VideoBackend},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopReason {
    Halted,
    StepLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunResult {
    pub reason: StopReason,
    pub steps: usize,
    pub cycles: u64,
}

/// A complete, deterministic ZT8 virtual machine.
pub struct Machine {
    cpu: Cpu,
    bus: AddressSpace,
}

impl Machine {
    pub fn new(bank_count: usize) -> Self {
        Self { cpu: Cpu::new(), bus: AddressSpace::new(bank_count) }
    }

    pub fn try_new(bank_count: usize) -> Result<Self, LoadError> {
        Ok(Self { cpu: Cpu::new(), bus: AddressSpace::try_new(bank_count)? })
    }

    pub fn with_address_space(bus: AddressSpace) -> Self {
        Self { cpu: Cpu::new(), bus }
    }

    pub fn load_rom(&mut self, image: &[u8]) -> Result<(), LoadError> {
        self.bus.load_rom(image)
    }

    pub fn load_bank(&mut self, bank: usize, image: &[u8]) -> Result<(), LoadError> {
        self.bus.load_bank(bank, image)
    }

    /// Resets RAM/devices and then loads the CPU reset vector from ROM.
    pub fn reset(&mut self) {
        self.bus.reset();
        self.cpu.reset(&mut self.bus);
    }

    /// Runs one CPU quantum and advances devices by its exact cycle count.
    pub fn step(&mut self) -> Result<Step, CpuFault> {
        let irq = self.bus.irq_line();
        let step = self.cpu.step(&mut self.bus, irq)?;
        self.bus.tick(u32::from(step.cycles));
        Ok(step)
    }

    /// Runs until HLT or until `step_limit` scheduling quanta have completed.
    pub fn run(&mut self, step_limit: usize) -> Result<RunResult, CpuFault> {
        let starting_cycles = self.cpu.cycles();
        let mut steps = 0;
        while steps < step_limit && !self.cpu.halted() {
            self.step()?;
            steps += 1;
        }

        Ok(RunResult {
            reason: if self.cpu.halted() { StopReason::Halted } else { StopReason::StepLimit },
            steps,
            cycles: self.cpu.cycles().wrapping_sub(starting_cycles),
        })
    }

    pub const fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub const fn bus(&self) -> &AddressSpace {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut AddressSpace {
        &mut self.bus
    }

    /// Supplies the complete host button bitfield and latches rising edges.
    pub fn set_buttons(&mut self, state: u8) {
        self.bus.set_buttons(state);
    }

    pub fn press_button(&mut self, button: Button) {
        self.bus.press_button(button);
    }

    pub fn release_button(&mut self, button: Button) {
        self.bus.release_button(button);
    }

    pub fn set_pointer(&mut self, x: i8, y: i8) {
        self.bus.set_pointer(x, y);
    }

    /// Presents one guest-requested frame, if any, through a host backend.
    pub fn render_if_pending(&mut self, backend: &mut dyn VideoBackend) -> bool {
        self.bus.video_mut().render_if_pending(backend)
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bus::{IRQ_INPUT, IRQ_TIMER, PAGE_SIZE, REG_IRQ_ENABLE},
        devices::{
            TIMER_CTRL, TIMER_CTRL_ENABLE, TIMER_CTRL_RESTART, VIDEO_CTRL_ENABLE, VideoDevice,
        },
        isa::{Opcode, Register, VECTOR_RESET, ldi, st_abs},
    };

    fn machine_with_program(program: &[u8]) -> Machine {
        let start = 0x0100u16;
        let mut rom = vec![0xff; PAGE_SIZE];
        rom[usize::from(start)..usize::from(start) + program.len()].copy_from_slice(program);
        rom[usize::from(VECTOR_RESET)..usize::from(VECTOR_RESET) + 2]
            .copy_from_slice(&start.to_le_bytes());
        let mut machine = Machine::new(2);
        machine.load_rom(&rom).unwrap();
        machine.reset();
        machine
    }

    #[test]
    fn machine_runs_a_program_and_ticks_the_timer() {
        let mut machine = machine_with_program(&[Opcode::Nop.byte(), Opcode::Halt.byte()]);
        machine.bus_mut().timer_mut().set_count(1);
        machine.bus_mut().timer_mut().write(TIMER_CTRL, TIMER_CTRL_ENABLE);

        let result = machine.run(10).unwrap();

        assert_eq!(result.reason, StopReason::Halted);
        assert_eq!(result.steps, 2);
        assert_ne!(machine.bus().irq_pending() & IRQ_TIMER, 0);
    }

    #[test]
    fn host_button_edges_reach_mmio_and_raise_an_irq() {
        let mut machine = Machine::default();
        machine.bus_mut().write(REG_IRQ_ENABLE, IRQ_INPUT);
        machine.press_button(Button::A);

        assert!(machine.bus().irq_line());
        assert!(machine.bus().input().is_down(Button::A));
        assert_eq!(machine.bus().input().pressed(), Button::A.bit());
    }

    #[derive(Default)]
    struct RecordingBackend {
        frames: usize,
        first_pixel: u8,
    }

    impl VideoBackend for RecordingBackend {
        fn present(&mut self, width: usize, height: usize, pixels: &[u8]) {
            assert_eq!((width, height), (128, 96));
            self.frames += 1;
            self.first_pixel = pixels[0];
        }
    }

    #[test]
    fn guest_program_can_draw_and_request_a_frame() {
        let program = [
            ldi(Register::A),
            VIDEO_CTRL_ENABLE,
            st_abs(Register::A),
            0x30,
            0xc0,
            ldi(Register::A),
            0xe3,
            st_abs(Register::A),
            0x00,
            0xd0,
            st_abs(Register::A),
            0x31,
            0xc0,
            Opcode::Halt.byte(),
        ];
        let mut machine = machine_with_program(&program);
        machine.run(20).unwrap();
        let mut backend = RecordingBackend::default();

        assert!(machine.render_if_pending(&mut backend));
        assert_eq!(backend.frames, 1);
        assert_eq!(backend.first_pixel, 0xe3);
        assert_eq!(machine.bus().video().frame_sequence(), 1);
    }

    #[test]
    fn reset_preserves_rom_but_clears_mutable_state() {
        let mut machine = machine_with_program(&[Opcode::Halt.byte()]);
        machine.bus_mut().ram_mut()[0] = 0xaa;
        machine.bus_mut().video_mut().write_vram(0, 0xff);
        machine.bus_mut().timer_mut().write(TIMER_CTRL, TIMER_CTRL_ENABLE | TIMER_CTRL_RESTART);
        let original_opcode = machine.bus().rom()[0x0100];

        machine.reset();

        assert_eq!(machine.bus().ram()[0], 0);
        assert_eq!(machine.bus().video(), &VideoDevice::new());
        assert_eq!(machine.bus().rom()[0x0100], original_opcode);
    }
}
