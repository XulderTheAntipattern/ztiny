use zt8::{
    Machine, Opcode, Register, StopReason, VideoBackend,
    bus::PAGE_SIZE,
    devices::{VIDEO_CTRL_ENABLE, rgb332_to_rgb888},
    isa::{VECTOR_RESET, ldi, st_abs},
};

#[derive(Default)]
struct ConsoleBackend;

impl VideoBackend for ConsoleBackend {
    fn present(&mut self, width: usize, height: usize, pixels: &[u8]) {
        let [red, green, blue] = rgb332_to_rgb888(pixels[0]);
        println!("frame {width}x{height}, first pixel rgb({red}, {green}, {blue})");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = 0x0100u16;
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

    let mut rom = vec![0xff; PAGE_SIZE];
    rom[usize::from(start)..usize::from(start) + program.len()].copy_from_slice(&program);
    rom[usize::from(VECTOR_RESET)..usize::from(VECTOR_RESET) + 2]
        .copy_from_slice(&start.to_le_bytes());

    let mut machine = Machine::default();
    machine.load_rom(&rom)?;
    machine.reset();
    let result = machine.run(1_000)?;
    if result.reason != StopReason::Halted {
        return Err("demo program exceeded its step limit".into());
    }

    machine.render_if_pending(&mut ConsoleBackend);
    println!("ZT8 halted after {} steps / {} cycles", result.steps, result.cycles);
    Ok(())
}
