//! Built-in host-facing devices for the ZT8 machine.
//!
//! Register offsets in this module are local to their device. The machine bus
//! is responsible for assigning each device a location in the global address
//! space.

/// Current button state. Each bit corresponds to a [`Button`].
pub const INPUT_STATE: u16 = 0x00;
/// Rising-edge button latch. Guest writes clear the bits that are set (W1C).
pub const INPUT_PRESSED: u16 = 0x01;
/// Signed pointer X coordinate, encoded as an `i8` in one byte.
pub const INPUT_POINTER_X: u16 = 0x02;
/// Signed pointer Y coordinate, encoded as an `i8` in one byte.
pub const INPUT_POINTER_Y: u16 = 0x03;

/// Low byte of the timer reload value.
pub const TIMER_RELOAD_LO: u16 = 0x00;
/// High byte of the timer reload value.
pub const TIMER_RELOAD_HI: u16 = 0x01;
/// Timer control register.
pub const TIMER_CTRL: u16 = 0x02;
/// Low byte of the timer's current count.
pub const TIMER_COUNT_LO: u16 = 0x03;
/// High byte of the timer's current count.
pub const TIMER_COUNT_HI: u16 = 0x04;
/// Enables timer counting when set in [`TIMER_CTRL`].
pub const TIMER_CTRL_ENABLE: u8 = 1 << 0;
/// Reloads the timer after expiry when set in [`TIMER_CTRL`].
pub const TIMER_CTRL_PERIODIC: u8 = 1 << 1;
/// Write-only strobe which copies the reload value into the current count.
pub const TIMER_CTRL_RESTART: u8 = 1 << 7;

/// Video control register.
pub const VIDEO_CTRL: u16 = 0x00;
/// Write-only register which requests that the current frame be presented.
pub const VIDEO_PRESENT: u16 = 0x01;
/// Read-only framebuffer width.
pub const VIDEO_WIDTH: u16 = 0x02;
/// Read-only framebuffer height.
pub const VIDEO_HEIGHT: u16 = 0x03;
/// Read-only pixel format (`1` means RGB332).
pub const VIDEO_FORMAT: u16 = 0x04;
/// Enables presentation when set in [`VIDEO_CTRL`].
pub const VIDEO_CTRL_ENABLE: u8 = 1 << 0;

/// Width of the video framebuffer in pixels.
pub const WIDTH: usize = 128;
/// Height of the video framebuffer in pixels.
pub const HEIGHT: usize = 96;
/// Number of RGB332 bytes in the video framebuffer.
pub const VRAM_SIZE: usize = WIDTH * HEIGHT;

/// Expands one RGB332 framebuffer byte into eight-bit RGB channels.
pub fn rgb332_to_rgb888(pixel: u8) -> [u8; 3] {
    let red = ((pixel >> 5) & 0x07) as u16;
    let green = ((pixel >> 2) & 0x07) as u16;
    let blue = (pixel & 0x03) as u16;
    [(red * 255 / 7) as u8, (green * 255 / 7) as u8, (blue * 255 / 3) as u8]
}

/// Buttons exposed by [`InputDevice`], one per bit in the input registers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Button {
    Up = 1 << 0,
    Down = 1 << 1,
    Left = 1 << 2,
    Right = 1 << 3,
    A = 1 << 4,
    B = 1 << 5,
    Start = 1 << 6,
    Select = 1 << 7,
}

impl Button {
    /// Returns this button's bit in [`INPUT_STATE`] and [`INPUT_PRESSED`].
    pub const fn bit(self) -> u8 {
        self as u8
    }
}

/// Host-populated button and pointer state.
///
/// `state` reflects buttons held now. `pressed` latches rising edges until the
/// guest acknowledges them by writing ones to [`INPUT_PRESSED`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputDevice {
    state: u8,
    pressed: u8,
    pointer_x: i8,
    pointer_y: i8,
}

impl InputDevice {
    pub const fn new() -> Self {
        Self { state: 0, pressed: 0, pointer_x: 0, pointer_y: 0 }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub const fn state(&self) -> u8 {
        self.state
    }

    pub const fn pressed(&self) -> u8 {
        self.pressed
    }

    pub const fn pointer_x(&self) -> i8 {
        self.pointer_x
    }

    pub const fn pointer_y(&self) -> i8 {
        self.pointer_y
    }

    pub const fn pointer(&self) -> (i8, i8) {
        (self.pointer_x, self.pointer_y)
    }

    pub const fn is_down(&self, button: Button) -> bool {
        self.state & button.bit() != 0
    }

    /// Replaces the host button state and latches newly pressed buttons.
    pub fn set_buttons(&mut self, state: u8) {
        self.pressed |= state & !self.state;
        self.state = state;
    }

    pub fn press(&mut self, button: Button) {
        self.set_buttons(self.state | button.bit());
    }

    pub fn release(&mut self, button: Button) {
        self.set_buttons(self.state & !button.bit());
    }

    pub fn set_pointer(&mut self, x: i8, y: i8) {
        self.pointer_x = x;
        self.pointer_y = y;
    }

    /// Reads a local input register. Unknown offsets read as `0xff`.
    pub const fn read(&self, offset: u16) -> u8 {
        match offset {
            INPUT_STATE => self.state,
            INPUT_PRESSED => self.pressed,
            INPUT_POINTER_X => self.pointer_x as u8,
            INPUT_POINTER_Y => self.pointer_y as u8,
            _ => 0xff,
        }
    }

    /// Writes a local input register.
    ///
    /// Only [`INPUT_PRESSED`] is guest-writable. Its bits use write-one-to-clear
    /// semantics; host-owned state and pointer registers ignore guest writes.
    pub fn write(&mut self, offset: u16, value: u8) {
        if offset == INPUT_PRESSED {
            self.pressed &= !value;
        }
    }
}

/// A cycle-counting 16-bit timer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimerDevice {
    reload: u16,
    count: u16,
    control: u8,
}

impl TimerDevice {
    pub const fn new() -> Self {
        Self { reload: 0, count: 0, control: 0 }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub const fn reload(&self) -> u16 {
        self.reload
    }

    pub const fn count(&self) -> u16 {
        self.count
    }

    pub const fn control(&self) -> u8 {
        self.control
    }

    pub const fn enabled(&self) -> bool {
        self.control & TIMER_CTRL_ENABLE != 0
    }

    pub const fn periodic(&self) -> bool {
        self.control & TIMER_CTRL_PERIODIC != 0
    }

    pub fn set_reload(&mut self, reload: u16) {
        self.reload = reload;
    }

    pub fn set_count(&mut self, count: u16) {
        self.count = count;
    }

    pub fn restart(&mut self) {
        self.count = self.reload;
    }

    /// Advances the timer and reports whether it reached zero at least once.
    ///
    /// A periodic timer reloads immediately on expiry and consumes any cycles
    /// left in this tick. A zero reload value remains stopped at zero. A
    /// one-shot timer retains its enable bit after expiry but cannot raise a
    /// second interrupt until its count is restarted or written again.
    pub fn tick(&mut self, cycles: u32) -> bool {
        if cycles == 0 || !self.enabled() || self.count == 0 {
            return false;
        }

        let count = u32::from(self.count);
        if cycles < count {
            self.count -= cycles as u16;
            return false;
        }

        self.count = 0;
        if self.periodic() && self.reload != 0 {
            let remaining = cycles - count;
            let period = u32::from(self.reload);
            let remainder = remaining % period;
            self.count = if remainder == 0 {
                self.reload
            } else {
                (period - remainder) as u16
            };
        }

        true
    }

    /// Reads a local timer register. Multi-byte values are little-endian.
    /// Unknown offsets read as `0xff`.
    pub const fn read(&self, offset: u16) -> u8 {
        match offset {
            TIMER_RELOAD_LO => self.reload as u8,
            TIMER_RELOAD_HI => (self.reload >> 8) as u8,
            TIMER_COUNT_LO => self.count as u8,
            TIMER_COUNT_HI => (self.count >> 8) as u8,
            TIMER_CTRL => self.control,
            _ => 0xff,
        }
    }

    /// Writes a local timer register. Multi-byte values are little-endian.
    ///
    /// The restart bit in [`TIMER_CTRL`] acts as a strobe and is not retained
    /// when the control register is read back.
    pub fn write(&mut self, offset: u16, value: u8) {
        match offset {
            TIMER_RELOAD_LO => {
                self.reload = (self.reload & 0xff00) | u16::from(value)
            }
            TIMER_RELOAD_HI => {
                self.reload = (self.reload & 0x00ff) | (u16::from(value) << 8);
            }
            TIMER_COUNT_LO => {
                self.count = (self.count & 0xff00) | u16::from(value)
            }
            TIMER_COUNT_HI => {
                self.count = (self.count & 0x00ff) | (u16::from(value) << 8);
            }
            TIMER_CTRL => {
                self.control =
                    value & (TIMER_CTRL_ENABLE | TIMER_CTRL_PERIODIC);
                if value & TIMER_CTRL_RESTART != 0 {
                    self.restart();
                }
            }
            _ => {}
        }
    }
}

/// Host-side output target for the ZT8 RGB332 framebuffer.
///
/// Backends can convert the supplied one-byte-per-pixel RGB332 data into a
/// window, image, terminal representation, or any other presentation format.
pub trait VideoBackend {
    fn present(&mut self, width: usize, height: usize, pixels: &[u8]);
}

/// A 128x96 RGB332 framebuffer and its presentation registers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoDevice {
    pixels: [u8; VRAM_SIZE],
    enabled: bool,
    dirty: bool,
    frame_pending: bool,
    frame_sequence: u64,
}

impl Default for VideoDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoDevice {
    pub const fn new() -> Self {
        Self {
            pixels: [0; VRAM_SIZE],
            enabled: false,
            dirty: false,
            frame_pending: false,
            frame_sequence: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub const fn frame_pending(&self) -> bool {
        self.frame_pending
    }

    pub const fn frame_sequence(&self) -> u64 {
        self.frame_sequence
    }

    /// Returns the complete RGB332 framebuffer in row-major order.
    pub fn frame(&self) -> &[u8] {
        &self.pixels
    }

    /// Alias for [`VideoDevice::frame`].
    pub fn pixels(&self) -> &[u8] {
        self.frame()
    }

    /// Converts the current frame to RGBA8888 in caller-owned storage.
    /// Returns `false` without writing if the destination is too short.
    pub fn copy_rgba(&self, destination: &mut [u8]) -> bool {
        if destination.len() < VRAM_SIZE * 4 {
            return false;
        }
        for (pixel, rgba) in
            self.pixels.iter().zip(destination.chunks_exact_mut(4))
        {
            let [red, green, blue] = rgb332_to_rgb888(*pixel);
            rgba.copy_from_slice(&[red, green, blue, 0xff]);
        }
        true
    }

    pub fn pixel(&self, x: usize, y: usize) -> Option<u8> {
        let index = Self::pixel_index(x, y)?;
        Some(self.pixels[index])
    }

    /// Sets one pixel and returns whether the coordinates were in bounds.
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) -> bool {
        let Some(index) = Self::pixel_index(x, y) else {
            return false;
        };
        self.pixels[index] = value;
        self.dirty = true;
        true
    }

    pub fn read_vram(&self, offset: usize) -> Option<u8> {
        self.pixels.get(offset).copied()
    }

    /// Writes a framebuffer byte and returns whether the offset was in bounds.
    pub fn write_vram(&mut self, offset: usize, value: u8) -> bool {
        let Some(pixel) = self.pixels.get_mut(offset) else {
            return false;
        };
        *pixel = value;
        self.dirty = true;
        true
    }

    /// Reads a local video register. Unknown offsets read as `0xff`.
    pub const fn read(&self, offset: u16) -> u8 {
        match offset {
            VIDEO_CTRL => {
                if self.enabled {
                    VIDEO_CTRL_ENABLE
                } else {
                    0
                }
            }
            VIDEO_PRESENT => self.frame_sequence as u8,
            VIDEO_WIDTH => WIDTH as u8,
            VIDEO_HEIGHT => HEIGHT as u8,
            VIDEO_FORMAT => 1,
            _ => 0xff,
        }
    }

    /// Writes a local video register.
    pub fn write(&mut self, offset: u16, value: u8) {
        match offset {
            VIDEO_CTRL => self.enabled = value & VIDEO_CTRL_ENABLE != 0,
            VIDEO_PRESENT => self.request_present(),
            _ => {}
        }
    }

    /// Queues the current framebuffer for presentation.
    pub fn request_present(&mut self) {
        self.frame_sequence = self.frame_sequence.wrapping_add(1);
        self.frame_pending = true;
    }

    /// Consumes a pending frame and presents it when video output is enabled.
    ///
    /// Returns `true` only when the backend was called. A pending request is
    /// consumed even while disabled; pixel dirtiness is cleared only after a
    /// successful presentation.
    pub fn render_if_pending(
        &mut self,
        backend: &mut dyn VideoBackend,
    ) -> bool {
        if !self.frame_pending {
            return false;
        }

        self.frame_pending = false;
        if !self.enabled {
            return false;
        }

        backend.present(WIDTH, HEIGHT, &self.pixels);
        self.dirty = false;
        true
    }

    fn pixel_index(x: usize, y: usize) -> Option<usize> {
        if x < WIDTH && y < HEIGHT { Some(y * WIDTH + x) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_latches_rising_edges_and_guest_clears_them() {
        let mut input = InputDevice::new();
        input.press(Button::A);
        input.press(Button::Right);

        assert_eq!(
            input.read(INPUT_STATE),
            Button::A.bit() | Button::Right.bit()
        );
        assert_eq!(
            input.read(INPUT_PRESSED),
            Button::A.bit() | Button::Right.bit()
        );

        input.write(INPUT_PRESSED, Button::A.bit());
        assert_eq!(input.read(INPUT_PRESSED), Button::Right.bit());

        input.set_buttons(Button::Right.bit());
        input.set_buttons(Button::Right.bit() | Button::A.bit());
        assert_eq!(
            input.read(INPUT_PRESSED),
            Button::Right.bit() | Button::A.bit()
        );
    }

    #[test]
    fn input_pointer_registers_preserve_signed_bytes() {
        let mut input = InputDevice::new();
        input.set_pointer(-12, 101);

        assert_eq!(input.pointer(), (-12, 101));
        assert_eq!(input.read(INPUT_POINTER_X) as i8, -12);
        assert_eq!(input.read(INPUT_POINTER_Y) as i8, 101);
    }

    #[test]
    fn timer_registers_are_little_endian_and_restart_is_a_strobe() {
        let mut timer = TimerDevice::new();
        timer.write(TIMER_RELOAD_LO, 0x34);
        timer.write(TIMER_RELOAD_HI, 0x12);
        timer.write(
            TIMER_CTRL,
            TIMER_CTRL_ENABLE | TIMER_CTRL_PERIODIC | TIMER_CTRL_RESTART,
        );

        assert_eq!(timer.reload(), 0x1234);
        assert_eq!(timer.count(), 0x1234);
        assert_eq!(timer.read(TIMER_COUNT_LO), 0x34);
        assert_eq!(timer.read(TIMER_COUNT_HI), 0x12);
        assert_eq!(
            timer.read(TIMER_CTRL),
            TIMER_CTRL_ENABLE | TIMER_CTRL_PERIODIC
        );
    }

    #[test]
    fn timer_irqs_only_on_zero_transitions() {
        let mut timer = TimerDevice::new();
        timer.set_reload(5);
        timer.write(
            TIMER_CTRL,
            TIMER_CTRL_ENABLE | TIMER_CTRL_PERIODIC | TIMER_CTRL_RESTART,
        );

        assert!(!timer.tick(4));
        assert_eq!(timer.count(), 1);
        assert!(timer.tick(1));
        assert_eq!(timer.count(), 5);
        assert!(timer.tick(7));
        assert_eq!(timer.count(), 3);

        timer.write(TIMER_CTRL, TIMER_CTRL_ENABLE);
        timer.set_count(2);
        assert!(timer.tick(2));
        assert_eq!(timer.count(), 0);
        assert!(!timer.tick(1));
    }

    #[derive(Default)]
    struct RecordingBackend {
        width: usize,
        height: usize,
        frames: Vec<Vec<u8>>,
    }

    impl VideoBackend for RecordingBackend {
        fn present(&mut self, width: usize, height: usize, pixels: &[u8]) {
            self.width = width;
            self.height = height;
            self.frames.push(pixels.to_vec());
        }
    }

    #[test]
    fn video_tracks_dirty_pixels_and_presents_pending_frames() {
        let mut video = VideoDevice::new();
        let mut backend = RecordingBackend::default();

        assert!(video.set_pixel(3, 2, 0xe3));
        assert_eq!(video.pixel(3, 2), Some(0xe3));
        assert!(video.dirty());
        assert!(!video.set_pixel(WIDTH, 0, 1));

        video.write(VIDEO_CTRL, VIDEO_CTRL_ENABLE);
        video.write(VIDEO_PRESENT, 0);
        assert_eq!(video.frame_sequence(), 1);
        assert!(video.frame_pending());
        assert!(video.render_if_pending(&mut backend));

        assert!(!video.frame_pending());
        assert!(!video.dirty());
        assert_eq!((backend.width, backend.height), (WIDTH, HEIGHT));
        assert_eq!(backend.frames.len(), 1);
        assert_eq!(backend.frames[0][2 * WIDTH + 3], 0xe3);
        assert!(!video.render_if_pending(&mut backend));
    }

    #[test]
    fn disabled_video_consumes_present_without_calling_backend() {
        let mut video = VideoDevice::new();
        let mut backend = RecordingBackend::default();
        video.write_vram(0, 0xff);
        video.request_present();

        assert!(!video.render_if_pending(&mut backend));
        assert!(!video.frame_pending());
        assert!(video.dirty());
        assert!(backend.frames.is_empty());
    }

    #[test]
    fn rgb332_conversion_uses_the_full_channel_ranges() {
        assert_eq!(rgb332_to_rgb888(0x00), [0, 0, 0]);
        assert_eq!(rgb332_to_rgb888(0xff), [255, 255, 255]);
        assert_eq!(rgb332_to_rgb888(0xe0), [255, 0, 0]);
    }
}
