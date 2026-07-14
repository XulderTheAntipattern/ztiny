# ZT8

ZT8 is a dependency-free 8-bit virtual machine implemented in Rust. It has a
16-bit address bus, byte-wide data, banked ROM, work RAM, memory-mapped input
and timer devices, and a `128x96` RGB332 framebuffer.

The implementation lives in one reusable crate. The binary is a small program
and video-backend demonstration.

## Machine layout

The 64 KiB address space is always divided into four 16 KiB pages:

| Address range | Purpose |
| --- | --- |
| `0000-3FFF` | Fixed read-only program ROM |
| `4000-7FFF` | Read-only window into one of 1-256 selectable ROM banks |
| `8000-BFFF` | 16 KiB zero-initialized work RAM and descending stack |
| `C000-FFFF` | MMIO, reserved device space, and video RAM |

ROM images shorter than 16 KiB are padded with `FF`. Guest writes to either ROM
page are ignored. Register `C000` selects the visible bank modulo the installed
bank count, and `C001` reports the count (`00` means 256 banks).

The CPU loads little-endian vectors from fixed ROM:

| Vector | Address |
| --- | --- |
| Software interrupt | `3FFA` |
| Maskable IRQ | `3FFC` |
| Reset | `3FFE` |

## CPU

The programmer-visible state is four byte registers (`A`, `B`, `C`, `D`),
16-bit index/address registers (`X`, `Y`), a 16-bit stack pointer and program
counter, and `C`, `Z`, `I`, `V`, `N` flags. The reset stack pointer is `C000`;
the first pre-decrement push lands at `BFFF` in RAM.

Operands and stored words are little-endian. Arithmetic wraps. Relative branch
offsets are signed bytes relative to the end of the instruction. Illegal or
reserved opcodes return a typed `CpuFault` instead of panicking or silently
executing.

The instruction set includes:

- Immediate, absolute, `X`/`Y` indirect, and signed indexed loads/stores
- Four-register moves, push/pop, and 16-bit `X`/`Y` loads/stores/transfers
- `ADD`, `ADC`, `SUB`, `SBC`, `AND`, `OR`, `XOR`, and compare forms
- Increment, decrement, shift, rotate, and 16-bit index arithmetic
- Absolute/indirect jump and call, return, and signed/unsigned branches
- Flag control, software interrupt, maskable IRQ, interrupt return, and halt

Opcode constants and const encoding helpers are in
[`isa.rs`](crates/zt8/src/isa.rs). That module also documents flag effects,
stack/interrupt ordering, and cycle counts.

## Devices

| Address | Register |
| --- | --- |
| `C000` | Bank select (read/write) |
| `C001` | Bank count (read-only) |
| `C002` | IRQ enable: bit 0 timer, bit 1 input |
| `C003` | IRQ pending (read, write-one-to-clear) |
| `C010` | Current buttons: Up, Down, Left, Right, A, B, Start, Select |
| `C011` | Rising-edge button latch (read, write-one-to-clear) |
| `C012-C013` | Signed host pointer X/Y |
| `C020-C021` | Timer reload value, little-endian |
| `C022` | Timer control: enable, periodic, restart strobe |
| `C023-C024` | Current timer count, little-endian |
| `C030` | Video enable |
| `C031` | Write to request presentation; read low frame-sequence byte |
| `C032-C034` | Width (`128`), height (`96`), format (`1` = RGB332) |
| `D000-FFFF` | Row-major, one-byte-per-pixel RGB332 framebuffer |

The host supplies buttons and pointer values through typed `Machine` methods.
New button edges latch an input IRQ. The cycle timer is advanced by each
instruction's declared cycle count, so device behavior remains deterministic.

Video output is deliberately backend-neutral. A guest writes pixels, enables
video at `C030`, and writes `C031` when a frame is complete. The host owns a
backend implementing:

```rust
use zt8::VideoBackend;

struct MyBackend;

impl VideoBackend for MyBackend {
    fn present(&mut self, width: usize, height: usize, rgb332: &[u8]) {
        // Upload, convert, print, or stream the borrowed frame here.
    }
}
```

Then it calls `machine.render_if_pending(&mut backend)`. The core also provides
`rgb332_to_rgb888` and `VideoDevice::copy_rgba` for common backend formats.

## Host usage

```rust
use zt8::{Button, Machine};

let mut machine = Machine::new(4);
machine.load_rom(&fixed_rom)?;
machine.load_bank(0, &asset_bank)?;
machine.reset();

machine.press_button(Button::Start);
let result = machine.run(100_000)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Machine::step` is available for frame loops and debuggers. It executes one
instruction or interrupt quantum and ticks all devices. `Machine::run` stops on
`HLT`, a step limit, or a typed CPU fault.

## Build and test

```bash
cargo run -p zt8
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
