use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering};
use spin::Mutex;
use lazy_static::lazy_static;

use crate::framebuffer;
use crate::font_renderer;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Display mode: VGA text mode or VESA framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisplayMode {
    /// Traditional VGA text mode (80x25, 0xB8000).
    VgaText = 0,
    /// VESA/UEFI framebuffer (pixel-based rendering).
    Framebuffer = 1,
}

/// Global display mode — set during init.
static DISPLAY_MODE: AtomicU8 = AtomicU8::new(DisplayMode::VgaText as u8);

/// Get the current display mode.
pub fn get_display_mode() -> DisplayMode {
    match DISPLAY_MODE.load(Ordering::Relaxed) {
        0 => DisplayMode::VgaText,
        1 => DisplayMode::Framebuffer,
        _ => unreachable!(),
    }
}

/// Set the display mode.
pub fn set_display_mode(mode: DisplayMode) {
    DISPLAY_MODE.store(mode as u8, Ordering::Relaxed);
}

/// Convert VGA Color to framebuffer Color.
fn vga_color_to_fb_color(color: Color) -> framebuffer::Color {
    match color {
        Color::Black => framebuffer::Color::BLACK,
        Color::Blue => framebuffer::Color::BLUE,
        Color::Green => framebuffer::Color::GREEN,
        Color::Cyan => framebuffer::Color::CYAN,
        Color::Red => framebuffer::Color::RED,
        Color::Magenta => framebuffer::Color::MAGENTA,
        Color::Brown => framebuffer::Color::ORANGE,
        Color::LightGray => framebuffer::Color::LIGHT_GRAY,
        Color::DarkGray => framebuffer::Color::DARK_GRAY,
        Color::LightBlue => framebuffer::Color::new(173, 216, 230),
        Color::LightGreen => framebuffer::Color::new(144, 238, 144),
        Color::LightCyan => framebuffer::Color::new(224, 255, 255),
        Color::LightRed => framebuffer::Color::new(255, 182, 193),
        Color::LightMagenta => framebuffer::Color::new(255, 105, 180),
        Color::Yellow => framebuffer::Color::YELLOW,
        Color::White => framebuffer::Color::WHITE,
    }
}

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
unsafe impl Send for ColorCode {}
unsafe impl Sync for ColorCode {}

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
unsafe impl Send for ScreenChar {}
unsafe impl Sync for ScreenChar {}

struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct VgaWriter {
    pub column_position: usize,
    pub row_position: usize,
    pub color_code: ColorCode,
    buffer: *mut Buffer,
    /// Framebuffer cursor position (in pixels).
    fb_cursor_x: u64,
    fb_cursor_y: u64,
}

unsafe impl Send for VgaWriter {}
unsafe impl Sync for VgaWriter {}

impl VgaWriter {
    pub fn write_byte(&mut self, byte: u8) {
        // Write to serial always.
        crate::serial::write_byte_to_serial(byte);

        match get_display_mode() {
            DisplayMode::VgaText => self.write_byte_vga(byte),
            DisplayMode::Framebuffer => self.write_byte_fb(byte),
        }
    }

    /// Write a byte in VGA text mode.
    fn write_byte_vga(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line_vga(),
            0x08 => self.backspace_vga(),  // backspace
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line_vga();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                unsafe {
                    (*self.buffer).chars[row][col] = ScreenChar {
                        ascii_character: byte,
                        color_code: self.color_code,
                    };
                }
                self.column_position += 1;
            }
        }
        self.update_cursor();
    }

    /// Write a byte in framebuffer mode.
    fn write_byte_fb(&mut self, byte: u8) {
        framebuffer::with_fb(|fb| {
            let fg = vga_color_to_fb_color(self.get_fg_color());
            let bg = vga_color_to_fb_color(self.get_bg_color());
            let char_h = font_renderer::SCALED_CHAR_HEIGHT;

            match byte {
                b'\n' => {
                    self.fb_cursor_x = 0;
                    self.fb_cursor_y += char_h;
                    if self.fb_cursor_y + char_h > fb.height() {
                        fb.scroll(char_h);
                        self.fb_cursor_y = fb.height() - char_h;
                    }
                }
                0x08 => {
                    // Backspace.
                    if self.fb_cursor_x >= font_renderer::SCALED_CHAR_WIDTH {
                        self.fb_cursor_x -= font_renderer::SCALED_CHAR_WIDTH;
                        font_renderer::render_char(fb, b' ', self.fb_cursor_x, self.fb_cursor_y, fg, bg);
                    }
                }
                byte => {
                    if self.fb_cursor_x + font_renderer::SCALED_CHAR_WIDTH > fb.width() {
                        self.fb_cursor_x = 0;
                        self.fb_cursor_y += char_h;
                        if self.fb_cursor_y + char_h > fb.height() {
                            fb.scroll(char_h);
                            self.fb_cursor_y = fb.height() - char_h;
                        }
                    }
                    font_renderer::render_char(fb, byte, self.fb_cursor_x, self.fb_cursor_y, fg, bg);
                    self.fb_cursor_x += font_renderer::SCALED_CHAR_WIDTH;
                }
            }
        });
    }

    pub fn backspace(&mut self) {
        match get_display_mode() {
            DisplayMode::VgaText => self.backspace_vga(),
            DisplayMode::Framebuffer => {
                if self.fb_cursor_x >= font_renderer::SCALED_CHAR_WIDTH {
                    self.fb_cursor_x -= font_renderer::SCALED_CHAR_WIDTH;
                    framebuffer::with_fb(|fb| {
                        let fg = vga_color_to_fb_color(self.get_fg_color());
                        let bg = vga_color_to_fb_color(self.get_bg_color());
                        font_renderer::render_char(fb, b' ', self.fb_cursor_x, self.fb_cursor_y, fg, bg);
                    });
                }
            }
        }
    }

    fn get_fg_color(&self) -> Color {
        let cc = self.color_code.0;
        let fg_idx = cc & 0xF;
        match fg_idx {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::LightMagenta,
            14 => Color::Yellow,
            15 => Color::White,
            _ => Color::White,
        }
    }

    fn get_bg_color(&self) -> Color {
        let cc = self.color_code.0;
        let bg_idx = (cc >> 4) & 0xF;
        match bg_idx {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::LightMagenta,
            14 => Color::Yellow,
            15 => Color::White,
            _ => Color::Black,
        }
    }
    fn update_cursor(&self) {
        // Only update VGA cursor in VGA mode.
        if get_display_mode() == DisplayMode::VgaText {
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
    }

    /// Write a Unicode char — handles Hebrew (U+05D0–U+05EA) in framebuffer mode.
    pub fn write_char_unicode(&mut self, c: char) {
        if c.is_ascii() {
            self.write_byte(c as u8);
            return;
        }
        // Non-ASCII: serial as raw UTF-8 bytes, VGA text as '?', FB as Unicode glyph.
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        for b in s.bytes() {
            crate::serial::write_byte_to_serial(b);
        }
        match get_display_mode() {
            DisplayMode::VgaText => self.write_byte_vga(b'?'),
            DisplayMode::Framebuffer => self.write_char_unicode_fb(c),
        }
    }

    fn write_char_unicode_fb(&mut self, c: char) {
        framebuffer::with_fb(|fb| {
            let fg = vga_color_to_fb_color(self.get_fg_color());
            let bg = vga_color_to_fb_color(self.get_bg_color());
            let char_h = font_renderer::SCALED_CHAR_HEIGHT;
            if self.fb_cursor_x + font_renderer::SCALED_CHAR_WIDTH > fb.width() {
                self.fb_cursor_x = 0;
                self.fb_cursor_y += char_h;
                if self.fb_cursor_y + char_h > fb.height() {
                    fb.scroll(char_h);
                    self.fb_cursor_y = fb.height() - char_h;
                }
            }
            font_renderer::render_char_unicode(fb, c, self.fb_cursor_x, self.fb_cursor_y, fg, bg);
            self.fb_cursor_x += font_renderer::SCALED_CHAR_WIDTH;
        });
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char_unicode(c);
        }
    }

    #[allow(dead_code)]
    fn new_line(&mut self) {
        match get_display_mode() {
            DisplayMode::VgaText => self.new_line_vga(),
            DisplayMode::Framebuffer => {
                framebuffer::with_fb(|fb| {
                    let char_h = font_renderer::SCALED_CHAR_HEIGHT;
                    self.fb_cursor_x = 0;
                    self.fb_cursor_y += char_h;
                    if self.fb_cursor_y + char_h > fb.height() {
                        fb.scroll(char_h);
                        self.fb_cursor_y = fb.height() - char_h;
                    }
                });
            }
        }
    }

    fn new_line_vga(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    (*self.buffer).chars[row - 1][col] = (*self.buffer).chars[row][col];
                }
            }
        }
        self.clear_row_vga(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    #[allow(dead_code)]
    fn clear_row(&mut self, row: usize) {
        match get_display_mode() {
            DisplayMode::VgaText => self.clear_row_vga(row),
            DisplayMode::Framebuffer => {
                framebuffer::with_fb(|fb| {
                    let fg = vga_color_to_fb_color(self.get_fg_color());
                    let bg = vga_color_to_fb_color(self.get_bg_color());
                    let y = self.fb_cursor_y;
                    for x in (0..fb.width()).step_by(font_renderer::SCALED_CHAR_WIDTH as usize) {
                        font_renderer::render_char(fb, b' ', x, y, fg, bg);
                    }
                });
            }
        }
    }

    fn clear_row_vga(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            unsafe {
                (*self.buffer).chars[row][col] = ScreenChar {
                    ascii_character: b' ',
                    color_code: self.color_code,
                };
            }
        }
    }

    fn backspace_vga(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
            unsafe {
                (*self.buffer).chars[row][col] = ScreenChar {
                    ascii_character: b' ',
                    color_code: self.color_code,
                };
            }
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char_unicode(c);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: 0xb8000 as *mut Buffer,
        fb_cursor_x: 0,
        fb_cursor_y: 0,
    });
}

pub fn init() {}

/// Initialize framebuffer mode.
/// Call this after boot if VESA mode is available.
pub fn init_framebuffer() {
    set_display_mode(DisplayMode::Framebuffer);
    framebuffer::with_fb(|fb| {
        // Clear to black.
        let fg = framebuffer::Color::WHITE;
        let bg = framebuffer::Color::BLACK;
        for y in (0..fb.height()).step_by(font_renderer::SCALED_CHAR_HEIGHT as usize) {
            for x in (0..fb.width()).step_by(font_renderer::SCALED_CHAR_WIDTH as usize) {
                font_renderer::render_char(fb, b' ', x, y, fg, bg);
            }
        }
    });
}

/// Test function that exercises all Color variants and write_string
pub fn write_colored_test() {
    match get_display_mode() {
        DisplayMode::VgaText => write_colored_test_vga(),
        DisplayMode::Framebuffer => write_colored_test_fb(),
    }
}

fn write_colored_test_vga() {
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

fn write_colored_test_fb() {
    framebuffer::with_fb(|fb| {
        font_renderer::render_hebrew_test(fb);
    });
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
    // Feed history without any heap allocation (fmt::Arguments is Copy).
    crate::history::HISTORY.lock().write_fmt(args).ok();
    WRITER.lock().write_fmt(args).unwrap();  // write_fmt → write_str → serial + VGA
}
