# k3_ft_interrupt

A small educational kernel written in Rust, exploring the basics of bare-metal
programming on x86_64: VGA text mode output, serial port logging, and the CPU
exception handling path (IDT, breakpoint, page fault, double fault on a
dedicated IST stack).

This project exists purely as a learning exercise. It is not intended to be a
usable operating system.

## Features

- `no_std` Rust kernel targeting a custom `x86_64-ft_kernel.json` triple
- VGA text mode writer (color support, scrolling)
- 16550 UART serial driver (COM1) for logging through QEMU
- Interrupt Descriptor Table with handlers for:
  - `#BP` breakpoint (`int3`)
  - `#PF` page fault, with decoded error code (cause / access type / mode / faulting address)
  - `#DF` double fault, dispatched onto its own IST stack to survive kernel stack overflows
- Separate global writers for normal code and for interrupt context, to avoid
  any deadlock between the main execution path and IRQ handlers

## Roadmap

Planned next steps, roughly in order:

### Finishing the interrupts milestone
- `#GP` general protection fault handler
- GDT + TSS setup (currently only an IST entry exists for `#DF`)
- 8259 PIC remap (offsets `0x20` / `0x28`) so hardware IRQs no longer collide
  with CPU exceptions
- Hardware timer interrupt (PIT / IRQ 0) with proper EOI handling
- Keyboard interrupt (IRQ 1): raw scancode read from port `0x60`, then
  decoding to ASCII via `pc-keyboard`

### Memory management
- Read and walk the active 4-level page tables (translate virtual → physical)
- `BootInfoFrameAllocator` driven by the bootloader's memory map
- Generic `map_page(page, frame, flags)` that creates intermediate tables on
  demand
- Kernel heap region mapped into virtual memory
- A `GlobalAlloc` implementation (starting with a bump allocator, then a
  linked-list or fixed-size-block allocator) so `Box`, `Vec`, `String` work

### Async runtime
- Lock-free scancode queue fed from the keyboard IRQ handler
- `ScancodeStream` implementing `Stream`, with a `Waker` driven by the IRQ
- A minimal cooperative executor that polls only woken tasks and `hlt`s when
  idle
- Demo `async fn print_keypresses()` running concurrently with other tasks

### Multitasking
- Cooperative scheduler: `Task` struct with saved context, `switch_to` in
  inline assembly, round-robin `yield_now()`
- Preemptive scheduler driven by the timer IRQ, with full register
  save/restore
- Kernel synchronization primitives: `Mutex<T>` (interrupt-safe),
  `Semaphore`, `sleep(ticks)`

## Building and running

This kernel uses the [`bootloader`](https://crates.io/crates/bootloader) crate
and is meant to be run under QEMU. A nightly Rust toolchain is required (pinned
in `rust-toolchain.toml`).

```sh
cargo build
cargo run
```

## Acknowledgements

This project follows along closely with Philipp Oppermann's excellent
**Writing an OS in Rust** series and its companion repository:

- Blog: <https://os.phil-opp.com/>
- Repository: <https://github.com/phil-opp/blog_os>

Most of the architectural choices (custom target, bootloader integration, IDT
setup, double fault handling via IST, VGA / serial writers behind a spinlock)
are directly inspired by that work. Huge thanks to Phil-Opp for making
bare-metal Rust approachable.
