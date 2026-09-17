# mata

A bare-metal kernel for ARM Cortex-M, written from scratch in Rust. No
`cortex-m-rt`, no HAL crates, no `std`: the linker script, vector table, reset
path and every driver are in this repo, because the point is to understand what
those crates normally hide.

Runs on QEMU's `lm3s6965evb` (Stellaris LM3S6965, Cortex-M3, 256K flash / 64K
SRAM). Real hardware support comes later, behind a board abstraction.

## Run it

```sh
cargo run              # builds, boots under QEMU, prints, exits
cargo run --release
```

`cargo run` exits with the kernel's own exit status, so it doubles as the test
harness.

Debugging:

```sh
qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic \
  -semihosting-config enable=on,target=native \
  -kernel target/thumbv7m-none-eabi/debug/mata -S -gdb tcp::3333
arm-none-eabi-gdb -ex 'target remote :3333' target/thumbv7m-none-eabi/debug/mata
```

## Layout

```
.cargo/config.toml   target triple + QEMU runner
kernel/link.x        memory map, vector table placement, section layout
kernel/build.rs      hands link.x to the linker
kernel/src/boot.rs   vector table, reset handler
kernel/src/fault.rs  exception entry and fault reporting
kernel/src/debug.rs  semihosting output and exit
kernel/src/demo.rs   deliberate faults, behind --features fault-demo
```

## Roadmap

Tracked as issues, one per stage. Each lands as its own PR.
