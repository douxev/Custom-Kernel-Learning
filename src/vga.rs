use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new(color_code: ColorCode) -> Self {
        Writer {
            column_position: 0,
            color_code,
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }

    pub fn change_color(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col].write(blank);
        }
        self.column_position = 0;
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s1: &str) -> Result<(), core::fmt::Error> {
        for &byte in s1.as_bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
