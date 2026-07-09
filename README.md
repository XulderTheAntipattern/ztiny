# ztiny

`ztiny` is a Rust workspace for a tiny machine/emulation framework. The repository is organized as a workspace with multiple crates that define shared core types, a bus/device model, CPU and machine abstractions, and placeholder audio/video/CLI crates.

## Workspace Crates

- `ztiny_core`
  - Shared core primitives and utilities.
  - Includes numeric traits for address and word widths, clock/endian utilities, error handling, RGB types, and common constants.

- `ztiny_bus`
  - Bus and device abstractions.
  - Defines `Bus`, `Device`, `Attachment`, `Region`, and address mapping support.

- `ztiny_cpu`
  - CPU trait definition.
  - Defines a generic `Cpu<A, W>` interface with `reset`, `step`, and `halted`.

- `ztiny_machine`
  - Machine assembly layer.
  - Connects a CPU and a bus into a simple machine that can reset and step.

- `ztiny_audio`
  - Audio crate placeholder.
  - Currently contains a minimal test helper function.

- `ztiny_video`
  - Video crate placeholder.
  - Currently contains a minimal test helper function.

- `ztiny_cli`
  - CLI executable scaffold.
  - Current entrypoint prints a simple `Hello, world!` message.

## Build & Test

From the repository root:

```bash
cargo build
cargo test
```

## Usage

This workspace is currently an early-stage skeleton. The core structure is in place for building a modular emulator or virtual machine, but most functionality is still under development.

## Notes

- The workspace uses Cargo workspace resolver 2.
- The current CLI does not yet expose a runtime or emulator entrypoint beyond the placeholder main.
- The `ztiny_bus` crate defines the bus and device abstractions, but device/address routing is not fully implemented yet.

## Next Steps

Potential development goals include:

- Implement concrete CPU cores and instruction sets.
- Build device implementations for memory, I/O, video, and audio.
- Expand the CLI to load and run guest code.
- Add examples and documentation for target machine configurations.
