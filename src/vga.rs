use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

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
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

impl Clone for ColorCode { fn clone(&self) -> Self { *self } }
impl Copy for ColorCode {}

#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl Clone for ScreenChar {
    fn clone(&self) -> Self {
        ScreenChar { ascii_character: self.ascii_character, color_code: self.color_code }
    }
}
impl Copy for ScreenChar {}

struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct VgaWriter {
    pub column_position: usize,
    pub color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl VgaWriter {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => self.backspace(),  // backspace
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;
            }
        }
        self.update_cursor();
    }

    pub fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
            self.buffer.chars[row][col] = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
        }
    }

    fn update_cursor(&self) {
        let pos = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH + self.column_position;
        unsafe {
            use x86_64::instructions::port::Port;
            let mut idx: Port<u8> = Port::new(0x3D4);
            let mut dat: Port<u8> = Port::new(0x3D5);
            idx.write(0x0F);
            dat.write((pos & 0xFF) as u8);
            idx.write(0x0E);
            dat.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        crate::serial::write_str(s);
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row - 1][col] = self.buffer.chars[row][col];
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col] = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        crate::serial::write_str(s);
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn init() {}

/// Test function that exercises all Color variants and write_string
pub fn write_colored_test() {
    let mut writer = WRITER.lock();
    let colors = [
        Color::Black, Color::Blue, Color::Green, Color::Cyan,
        Color::Red, Color::Magenta, Color::Brown, Color::LightGray,
        Color::DarkGray, Color::LightBlue, Color::LightGreen, Color::LightCyan,
        Color::LightRed, Color::LightMagenta, Color::Yellow, Color::White,
    ];
    for (i, &color) in colors.iter().enumerate() {
        let byte = b'A' + i as u8;
        let cc = ColorCode::new(color, Color::Black);
        writer.write_byte(byte);
        // Consume the ColorCode to avoid dead code
        let _ = cc;
    }
    writer.write_byte(b' ');
    writer.write_string("[COLOR TEST]");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::vga::_print(format_args!("{}\n", format_args!($($arg)*)))
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use alloc::string::ToString;
    crate::history::feed(&args.to_string());
    WRITER.lock().write_fmt(args).unwrap();  // write_fmt → write_str → serial + VGA
}
