
pub struct SerialPort {
    port: u16,
}

#[allow(dead_code)]
impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn init(&self) {
        unsafe {
            outb(self.port + 1, 0x00); // Disable all interrupts
            outb(self.port + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(self.port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            outb(self.port + 1, 0x00); //                  (hi byte)
            outb(self.port + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(self.port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 4, 0x0B); // IRQs enabled, RTS/DSR set
        }
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe {
            while inb(self.port + 5) & 0x20 == 0 {} // Wait for the transmit buffer to be empty
            outb(self.port, byte);
        }
    }
}

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port);
    val
}

impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s1: &str) -> Result<(), core::fmt::Error> {
        for &byte in s1.as_bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _sprint(args: core::fmt::Arguments) {
    use core::fmt::Write;
    crate::SERIAL.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_sprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
