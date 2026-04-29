#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

#[macro_use]
mod vga;
#[macro_use]
mod serial;
mod gdt;
mod interrupts;

use crate::serial::SerialPort;
use crate::vga::{Color, ColorCode};
use core::fmt::Write;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    // Normal usage
    pub static ref SERIAL: Mutex<SerialPort> = {
        let sp = SerialPort::new(0x3F8); // COM1
        sp.init();
        Mutex::new(sp) };
    pub static ref VGA: Mutex<vga::Writer> = Mutex::new(
        vga::Writer::new(ColorCode::new(Color::White, Color::Black))
    );

    // IRQ handlers
    pub static ref SERIAL_IRQ: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3F8));
    pub static ref VGA_IRQ: Mutex<vga::Writer> = Mutex::new(
        vga::Writer::new(ColorCode::new(Color::Red, Color::Black))
    );
}

// =============================================================================
// _START
// =============================================================================

#[no_mangle]
pub extern "C" fn _start() -> ! {
    interrupts::init();
    writeln!(SERIAL.lock(), "Hello, serial port!").unwrap();

    writeln!(VGA.lock(), "Hello, world!").unwrap();
    writeln!(VGA.lock(), "This is a VGA text mode example.").unwrap();

    // x86_64::instructions::interrupts::int3();

    writeln!(SERIAL.lock(), "Logs sent!").unwrap();


    loop {}
}

// =============================================================================
// END OF _START
// =============================================================================

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn halt_loop() -> ! {
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}
