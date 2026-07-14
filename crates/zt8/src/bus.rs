//! ZT8's concrete 64 KiB address space.
//!
//! The address space is split into four 16 KiB pages. The first page is the
//! fixed ROM, the second is a selectable ROM bank, the third is RAM, and the
//! final page contains memory-mapped devices and video memory.

use std::{error::Error, fmt};

use crate::devices::{Button, InputDevice, TimerDevice, VideoDevice};

pub const PAGE_SIZE: usize = 0x4000; // 16KB
pub const VIDEO_MEMORY_SIZE: usize = 0x3000;

pub const ROM_START: u16 = 0x0000;
pub const ROM_END: u16 = 0x3fff;
pub const BANK_START: u16 = 0x4000;
pub const BANK_END: u16 = 0x7fff;
pub const RAM_START: u16 = 0x8000;
pub const RAM_END: u16 = 0xbfff;
pub const IO_START: u16 = 0xc000;
pub const IO_END: u16 = 0xffff;

pub const REG_BANK_SELECT: u16 = 0xc000;
pub const REG_BANK_COUNT: u16 = 0xc001;
pub const REG_IRQ_ENABLE: u16 = 0xc002;
pub const REG_IRQ_PENDING: u16 = 0xc003;

pub const INPUT_REG_START: u16 = 0xc010;
pub const INPUT_REG_END: u16 = 0xc013;
pub const TIMER_REG_START: u16 = 0xc020;
pub const TIMER_REG_END: u16 = 0xc024;
pub const VIDEO_REG_START: u16 = 0xc030;
pub const VIDEO_REG_END: u16 = 0xc034;
pub const VIDEO_MEMORY_START: u16 = 0xd000;
pub const VIDEO_MEMORY_END: u16 = 0xffff;

pub const IRQ_TIMER: u8 = 1 << 0;
pub const IRQ_INPUT: u8 = 1 << 1;
pub const IRQ_MASK: u8 = IRQ_TIMER | IRQ_INPUT;

const UNMAPPED_VALUE: u8 = 0xff;

/// A failure to construct or load part of an address space.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoadError {
    InvalidBankCount { count: usize },
    InvalidBank { bank: usize, bank_count: usize },
    RomTooLarge { size: usize, capacity: usize },
    BankTooLarge { size: usize, capacity: usize },
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBankCount { count } => {
                write!(
                    formatter,
                    "bank count must be between 1 and 256, got {count}"
                )
            }
            Self::InvalidBank { bank, bank_count } => {
                write!(
                    formatter,
                    "bank {bank} is outside the {bank_count}-bank address space"
                )
            }
            Self::RomTooLarge { size, capacity } => {
                write!(
                    formatter,
                    "ROM image is {size} bytes; capacity is {capacity} bytes"
                )
            }
            Self::BankTooLarge { size, capacity } => {
                write!(
                    formatter,
                    "bank image is {size} bytes; capacity is {capacity} bytes"
                )
            }
        }
    }
}

impl Error for LoadError {}

/// The storage and memory-mapped devices visible to the ZT8 CPU.
pub struct AddressSpace {
    fixed_rom: Box<[u8; PAGE_SIZE]>,
    banks: Vec<Box<[u8; PAGE_SIZE]>>,
    ram: Box<[u8; PAGE_SIZE]>,
    input: InputDevice,
    timer: TimerDevice,
    video: VideoDevice,
    selected_bank: u8,
    irq_enable: u8,
    irq_pending: u8,
}

impl AddressSpace {
    /// Construct an address space with `bank_count` selectable ROM banks.
    ///
    /// Panics if the count is not in `1..=256`. Use [`Self::try_new`] when the
    /// bank count comes from untrusted input.
    pub fn new(bank_count: usize) -> Self {
        Self::try_new(bank_count).expect("ZT8 bank count must be in 1..=256")
    }

    pub fn try_new(bank_count: usize) -> Result<Self, LoadError> {
        if !(1..=256).contains(&bank_count) {
            return Err(LoadError::InvalidBankCount { count: bank_count });
        }

        let banks = (0..bank_count)
            .map(|_| Box::new([UNMAPPED_VALUE; PAGE_SIZE]))
            .collect();

        Ok(Self {
            fixed_rom: Box::new([UNMAPPED_VALUE; PAGE_SIZE]),
            banks,
            ram: Box::new([0; PAGE_SIZE]),
            input: InputDevice::default(),
            timer: TimerDevice::default(),
            video: VideoDevice::default(),
            selected_bank: 0,
            irq_enable: 0,
            irq_pending: 0,
        })
    }

    /// Replace the fixed ROM image, padding unused bytes with `0xff`.
    pub fn load_rom(&mut self, image: &[u8]) -> Result<(), LoadError> {
        if image.len() > PAGE_SIZE {
            return Err(LoadError::RomTooLarge {
                size: image.len(),
                capacity: PAGE_SIZE,
            });
        }

        self.fixed_rom.fill(UNMAPPED_VALUE);
        self.fixed_rom[..image.len()].copy_from_slice(image);
        Ok(())
    }

    /// Replace one selectable ROM bank, padding unused bytes with `0xff`.
    pub fn load_bank(
        &mut self,
        bank: usize,
        image: &[u8],
    ) -> Result<(), LoadError> {
        if image.len() > PAGE_SIZE {
            return Err(LoadError::BankTooLarge {
                size: image.len(),
                capacity: PAGE_SIZE,
            });
        }

        let bank_count = self.banks.len();
        let destination = self
            .banks
            .get_mut(bank)
            .ok_or(LoadError::InvalidBank { bank, bank_count })?;
        destination.fill(UNMAPPED_VALUE);
        destination[..image.len()].copy_from_slice(image);
        Ok(())
    }

    /// Read a byte as the CPU would. Device register reads are currently
    /// side-effect free; [`Self::peek`] is provided for debugger intent.
    pub fn read(&mut self, address: u16) -> u8 {
        self.peek(address)
    }

    /// Inspect a byte without changing address-space or device state.
    pub fn peek(&self, address: u16) -> u8 {
        match address {
            ROM_START..=ROM_END => {
                self.fixed_rom[usize::from(address - ROM_START)]
            }
            BANK_START..=BANK_END => {
                self.banks[usize::from(self.selected_bank)]
                    [usize::from(address - BANK_START)]
            }
            RAM_START..=RAM_END => self.ram[usize::from(address - RAM_START)],
            REG_BANK_SELECT => self.selected_bank,
            REG_BANK_COUNT => self.bank_count_register(),
            REG_IRQ_ENABLE => self.irq_enable,
            REG_IRQ_PENDING => self.irq_pending,
            INPUT_REG_START..=INPUT_REG_END => {
                self.input.read(address - INPUT_REG_START)
            }
            TIMER_REG_START..=TIMER_REG_END => {
                self.timer.read(address - TIMER_REG_START)
            }
            VIDEO_REG_START..=VIDEO_REG_END => {
                self.video.read(address - VIDEO_REG_START)
            }
            VIDEO_MEMORY_START..=VIDEO_MEMORY_END => self
                .video
                .read_vram(usize::from(address - VIDEO_MEMORY_START))
                .unwrap_or(UNMAPPED_VALUE),
            _ => UNMAPPED_VALUE,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            // Fixed and banked ROM are protected from guest writes.
            ROM_START..=BANK_END => {}
            RAM_START..=RAM_END => {
                self.ram[usize::from(address - RAM_START)] = value
            }
            REG_BANK_SELECT => {
                self.selected_bank =
                    (usize::from(value) % self.banks.len()) as u8;
            }
            REG_BANK_COUNT => {}
            REG_IRQ_ENABLE => self.irq_enable = value & IRQ_MASK,
            REG_IRQ_PENDING => self.clear_irq(value),
            INPUT_REG_START..=INPUT_REG_END => {
                self.input.write(address - INPUT_REG_START, value);
            }
            TIMER_REG_START..=TIMER_REG_END => {
                self.timer.write(address - TIMER_REG_START, value);
            }
            VIDEO_REG_START..=VIDEO_REG_END => {
                self.video.write(address - VIDEO_REG_START, value);
            }
            VIDEO_MEMORY_START..=VIDEO_MEMORY_END => {
                self.video.write_vram(
                    usize::from(address - VIDEO_MEMORY_START),
                    value,
                );
            }
            _ => {}
        }
    }

    /// Advance time-dependent devices and latch any generated interrupt.
    pub fn tick(&mut self, cycles: u32) {
        if self.timer.tick(cycles) {
            self.request_irq(IRQ_TIMER);
        }
    }

    /// Reset mutable machine state while preserving loaded ROM images.
    pub fn reset(&mut self) {
        self.ram.fill(0);
        self.selected_bank = 0;
        self.irq_enable = 0;
        self.irq_pending = 0;
        self.input.reset();
        self.timer.reset();
        self.video.reset();
    }

    pub fn irq_line(&self) -> bool {
        self.irq_enable & self.irq_pending != 0
    }

    pub fn request_irq(&mut self, sources: u8) {
        self.irq_pending |= sources & IRQ_MASK;
    }

    pub fn clear_irq(&mut self, sources: u8) {
        self.irq_pending &= !(sources & IRQ_MASK);
    }

    pub fn irq_enable(&self) -> u8 {
        self.irq_enable
    }

    pub fn irq_pending(&self) -> u8 {
        self.irq_pending
    }

    pub fn selected_bank(&self) -> u8 {
        self.selected_bank
    }

    pub fn bank_count(&self) -> usize {
        self.banks.len()
    }

    pub fn bank_count_register(&self) -> u8 {
        if self.banks.len() == 256 { 0 } else { self.banks.len() as u8 }
    }

    pub fn rom(&self) -> &[u8; PAGE_SIZE] {
        &self.fixed_rom
    }

    pub fn bank(&self, bank: usize) -> Option<&[u8; PAGE_SIZE]> {
        self.banks.get(bank).map(Box::as_ref)
    }

    pub fn bank_mut(&mut self, bank: usize) -> Option<&mut [u8; PAGE_SIZE]> {
        self.banks.get_mut(bank).map(Box::as_mut)
    }

    pub fn ram(&self) -> &[u8; PAGE_SIZE] {
        &self.ram
    }

    pub fn ram_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
        &mut self.ram
    }

    pub fn input(&self) -> &InputDevice {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut InputDevice {
        &mut self.input
    }

    /// Replaces host button state and raises the input IRQ for new presses.
    pub fn set_buttons(&mut self, state: u8) {
        let before = self.input.pressed();
        self.input.set_buttons(state);
        if self.input.pressed() & !before != 0 {
            self.request_irq(IRQ_INPUT);
        }
    }

    /// Presses one host button and raises the input IRQ on its rising edge.
    pub fn press_button(&mut self, button: Button) {
        self.set_buttons(self.input.state() | button.bit());
    }

    pub fn release_button(&mut self, button: Button) {
        self.set_buttons(self.input.state() & !button.bit());
    }

    pub fn set_pointer(&mut self, x: i8, y: i8) {
        self.input.set_pointer(x, y);
    }

    pub fn timer(&self) -> &TimerDevice {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut TimerDevice {
        &mut self.timer
    }

    pub fn video(&self) -> &VideoDevice {
        &self.video
    }

    pub fn video_mut(&mut self) -> &mut VideoDevice {
        &mut self.video
    }
}

impl Default for AddressSpace {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_have_the_expected_boundaries() {
        let mut memory = AddressSpace::new(1);
        memory.load_rom(&[0x11]).unwrap();
        memory.load_bank(0, &[0x22]).unwrap();
        memory.write(RAM_START, 0x33);

        assert_eq!(memory.peek(ROM_START), 0x11);
        assert_eq!(memory.peek(ROM_END), 0xff);
        assert_eq!(memory.peek(BANK_START), 0x22);
        assert_eq!(memory.peek(BANK_END), 0xff);
        assert_eq!(memory.peek(RAM_START), 0x33);
        assert_eq!(memory.peek(RAM_END), 0x00);
        assert_eq!(memory.peek(0xcfff), 0xff);
    }

    #[test]
    fn bank_select_wraps_to_an_available_bank() {
        let mut memory = AddressSpace::new(3);
        memory.load_bank(0, &[0xa0]).unwrap();
        memory.load_bank(1, &[0xa1]).unwrap();
        memory.load_bank(2, &[0xa2]).unwrap();

        memory.write(REG_BANK_SELECT, 2);
        assert_eq!(memory.read(BANK_START), 0xa2);
        assert_eq!(memory.read(REG_BANK_SELECT), 2);

        memory.write(REG_BANK_SELECT, 4);
        assert_eq!(memory.selected_bank(), 1);
        assert_eq!(memory.read(BANK_START), 0xa1);
    }

    #[test]
    fn guest_writes_cannot_change_rom() {
        let mut memory = AddressSpace::new(1);
        memory.load_rom(&[0x42]).unwrap();
        memory.load_bank(0, &[0x24]).unwrap();

        memory.write(ROM_START, 0x00);
        memory.write(BANK_START, 0x00);

        assert_eq!(memory.read(ROM_START), 0x42);
        assert_eq!(memory.read(BANK_START), 0x24);
    }

    #[test]
    fn loaders_reject_invalid_images_without_changing_memory() {
        let mut memory = AddressSpace::new(1);
        memory.load_rom(&[0x42]).unwrap();
        let oversized = vec![0; PAGE_SIZE + 1];

        assert!(matches!(
            memory.load_rom(&oversized),
            Err(LoadError::RomTooLarge { .. })
        ));
        assert_eq!(memory.peek(ROM_START), 0x42);
        assert!(matches!(
            memory.load_bank(1, &[]),
            Err(LoadError::InvalidBank { bank: 1, bank_count: 1 })
        ));
    }

    #[test]
    fn interrupt_registers_mask_and_clear_sources() {
        let mut memory = AddressSpace::new(1);
        memory.request_irq(IRQ_TIMER | IRQ_INPUT | 0x80);
        assert_eq!(memory.irq_pending(), IRQ_MASK);
        assert!(!memory.irq_line());

        memory.write(REG_IRQ_ENABLE, IRQ_TIMER | 0x80);
        assert!(memory.irq_line());
        assert_eq!(memory.irq_enable(), IRQ_TIMER);

        memory.write(REG_IRQ_PENDING, IRQ_TIMER);
        assert_eq!(memory.irq_pending(), IRQ_INPUT);
        assert!(!memory.irq_line());
    }

    #[test]
    fn bank_count_register_uses_zero_for_256_banks() {
        assert_eq!(AddressSpace::new(1).bank_count_register(), 1);
        assert_eq!(AddressSpace::new(256).bank_count_register(), 0);
        assert!(matches!(
            AddressSpace::try_new(0),
            Err(LoadError::InvalidBankCount { count: 0 })
        ));
    }
}
