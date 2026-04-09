//! Linear framebuffer driver for VESA/UEFI graphics mode.
//!
//! This module provides a software framebuffer that renders pixels to a
//! linear memory buffer. When booted via VESA/UEFI with GOP, the bootloader
//! provides the actual framebuffer physical address. Until then, this module
//! provides a fallback buffer that can be displayed via other means.
//!
//! The framebuffer uses 32-bit pixels (8 bits per channel, BGRA or RGBx format).

/// Color representation for 32-bit framebuffer pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0 };
    pub const DARK_GRAY: Color = Color { r: 64, g: 64, b: 64 };
    pub const LIGHT_GRAY: Color = Color { r: 192, g: 192, b: 192 };
    pub const ORANGE: Color = Color { r: 255, g: 165, b: 0 };

    /// Create a new color from RGB values.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    /// Convert to a 32-bit pixel value (RGBx format, little-endian: B, G, R, _).
    pub fn to_pixel(&self) -> u32 {
        u32::from(*self)
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> u32 {
        // Little-endian: B, G, R, _ (unused alpha)
        (color.r as u32) << 16 | (color.g as u32) << 8 | (color.b as u32)
    }
}

/// Framebuffer information from bootloader.
#[derive(Debug, Clone, Copy)]
pub struct FrameBufferInfo {
    /// Physical address of the framebuffer.
    pub addr: u64,
    /// Width in pixels.
    pub width: u64,
    /// Height in pixels.
    pub height: u64,
    /// Pitch (bytes per scanline).
    pub pitch: u64,
    /// Bits per pixel.
    pub bpp: u8,
}

/// A linear framebuffer.
///
/// When booted with VESA/UEFI GOP, `buffer` points to the actual hardware framebuffer.
/// Otherwise, `buffer` points to a software buffer that can be rendered and inspected.
pub struct FrameBuffer {
    /// Pointer to the framebuffer memory.
    buffer: *mut u8,
    /// Framebuffer dimensions and layout.
    info: FrameBufferInfo,
    /// Current cursor position (in pixels, top-left of character cell).
    cursor_x: u64,
    cursor_y: u64,
    /// Current foreground color.
    fg_color: Color,
    /// Current background color.
    bg_color: Color,
    /// Whether this framebuffer is backed by real hardware.
    pub is_hardware: bool,
}

/// Software framebuffer size (used when no VESA mode is active).
const SW_FB_WIDTH: u64 = 1024;
const SW_FB_HEIGHT: u64 = 768;

/// Static software buffer (allocated in BSS).
static mut SW_BUFFER: [u8; (SW_FB_WIDTH * SW_FB_HEIGHT * 4) as usize] = [0; (SW_FB_WIDTH * SW_FB_HEIGHT * 4) as usize];

impl FrameBuffer {
    /// Create a hardware-backed framebuffer from bootloader info.
    ///
    /// # Safety
    /// The physical address must be valid and mapped.
    pub unsafe fn new_hardware(info: FrameBufferInfo) -> Self {
        let ptr = info.addr as *mut u8;
        // Clear the framebuffer to black.
        let size = (info.pitch * info.height) as usize;
        core::ptr::write_bytes(ptr, 0, size);

        FrameBuffer {
            buffer: ptr,
            info,
            cursor_x: 0,
            cursor_y: 0,
            fg_color: Color::WHITE,
            bg_color: Color::BLACK,
            is_hardware: true,
        }
    }

    /// Create a software framebuffer (fallback when no VESA mode).
    ///
    /// # Safety
    /// Must be called only once during boot.
    pub unsafe fn new_software() -> Self {
        // Initialize the software buffer to zeros.
        let buf_ptr = SW_BUFFER.as_mut_ptr();
        let size = (SW_FB_WIDTH * SW_FB_HEIGHT * 4) as usize;
        core::ptr::write_bytes(buf_ptr, 0, size);

        let info = FrameBufferInfo {
            addr: buf_ptr as u64,
            width: SW_FB_WIDTH,
            height: SW_FB_HEIGHT,
            pitch: SW_FB_WIDTH * 4,
            bpp: 32,
        };

        FrameBuffer {
            buffer: buf_ptr,
            info,
            cursor_x: 0,
            cursor_y: 0,
            fg_color: Color::WHITE,
            bg_color: Color::BLACK,
            is_hardware: false,
        }
    }

    /// Get framebuffer width in pixels.
    pub fn width(&self) -> u64 {
        self.info.width
    }

    /// Get framebuffer height in pixels.
    pub fn height(&self) -> u64 {
        self.info.height
    }

    /// Get framebuffer info.
    pub fn info(&self) -> FrameBufferInfo {
        self.info
    }

    /// Set a single pixel at (x, y).
    pub fn set_pixel(&mut self, x: u64, y: u64, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let pixel: u32 = color.into();
        let offset = (y * self.info.pitch + x * 4) as isize;
        unsafe {
            let ptr = self.buffer.offset(offset) as *mut u32;
            ptr.write_volatile(pixel);
        }
    }

    /// Get the pixel color at (x, y).
    pub fn get_pixel(&self, x: u64, y: u64) -> Color {
        if x >= self.info.width || y >= self.info.height {
            return Color::BLACK;
        }

        let offset = (y * self.info.pitch + x * 4) as isize;
        unsafe {
            let ptr = self.buffer.offset(offset) as *const u32;
            let val = ptr.read_volatile();
            Color {
                r: ((val >> 16) & 0xFF) as u8,
                g: ((val >> 8) & 0xFF) as u8,
                b: (val & 0xFF) as u8,
            }
        }
    }

    /// Fill a rectangle with the given color.
    pub fn fill_rect(&mut self, x: u64, y: u64, width: u64, height: u64, color: Color) {
        for cy in y..y + height {
            for cx in x..x + width {
                self.set_pixel(cx, cy, color);
            }
        }
    }

    /// Draw a horizontal line.
    pub fn draw_hline(&mut self, x: u64, y: u64, length: u64, color: Color) {
        self.fill_rect(x, y, length, 1, color);
    }

    /// Draw a vertical line.
    pub fn draw_vline(&mut self, x: u64, y: u64, length: u64, color: Color) {
        self.fill_rect(x, y, 1, length, color);
    }

    /// Draw a rectangle outline.
    pub fn draw_rect(&mut self, x: u64, y: u64, width: u64, height: u64, color: Color) {
        self.draw_hline(x, y, width, color);
        self.draw_hline(x, y + height - 1, width, color);
        self.draw_vline(x, y, height, color);
        self.draw_vline(x + width - 1, y, height, color);
    }

    /// Clear the entire framebuffer.
    pub fn clear(&mut self) {
        self.fill_rect(0, 0, self.info.width, self.info.height, self.bg_color);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Set the foreground color.
    pub fn set_fg_color(&mut self, color: Color) {
        self.fg_color = color;
    }

    /// Set the background color.
    pub fn set_bg_color(&mut self, color: Color) {
        self.bg_color = color;
    }

    /// Get the current foreground color.
    pub fn fg_color(&self) -> Color {
        self.fg_color
    }

    /// Get the current background color.
    pub fn bg_color(&self) -> Color {
        self.bg_color
    }

    /// Advance the cursor by one character cell.
    pub fn advance_cursor(&mut self, char_width: u64, char_height: u64) {
        self.cursor_x += char_width;
        if self.cursor_x + char_width > self.info.width {
            self.cursor_x = 0;
            self.cursor_y += char_height;
            if self.cursor_y + char_height > self.info.height {
                self.scroll(char_height);
            }
        }
    }

    /// Scroll the framebuffer up by the given number of pixels.
    pub fn scroll(&mut self, lines: u64) {
        if lines >= self.info.height {
            self.clear();
            return;
        }

        let bytes_per_line = self.info.pitch as usize;
        let total_bytes = bytes_per_line * self.info.height as usize;
        let scroll_bytes = bytes_per_line * lines as usize;

        unsafe {
            // Move everything up by 'lines' rows.
            let src = self.buffer.add(scroll_bytes);
            let dst = self.buffer;
            core::ptr::copy(src, dst, total_bytes - scroll_bytes);

            // Clear the bottom 'lines' rows.
            let clear_start = self.buffer.add(total_bytes - scroll_bytes);
            core::ptr::write_bytes(clear_start, 0, scroll_bytes);
        }

        self.cursor_y -= lines;
    }

    /// Get the current cursor position in pixels.
    pub fn cursor_position(&self) -> (u64, u64) {
        (self.cursor_x, self.cursor_y)
    }

    /// Set the cursor position in pixels.
    pub fn set_cursor_position(&mut self, x: u64, y: u64) {
        self.cursor_x = x;
        self.cursor_y = y;
    }
}

/// Global framebuffer instance.
pub static mut FRAMEBUFFER: Option<FrameBuffer> = None;

/// Initialize the framebuffer (hardware if available, otherwise software fallback).
///
/// # Safety
/// Must be called once during boot.
pub unsafe fn init(hw_info: Option<FrameBufferInfo>) {
    match hw_info {
        Some(info) => {
            FRAMEBUFFER = Some(FrameBuffer::new_hardware(info));
        }
        None => {
            FRAMEBUFFER = Some(FrameBuffer::new_software());
        }
    }
}

/// Initialize hardware framebuffer directly from UEFI GOP parameters.
///
/// This is the preferred path when using bootloader-x86_64-uefi.
///
/// # Safety
/// `addr` must be a valid, mapped physical framebuffer address from UEFI GOP.
pub unsafe fn init_hw(addr: u64, width: u64, height: u64, pitch: u64, bpp: u8) {
    let info = FrameBufferInfo {
        addr,
        width,
        height,
        pitch,
        bpp,
    };
    FRAMEBUFFER = Some(FrameBuffer::new_hardware(info));
}

/// Get a mutable reference to the global framebuffer.
pub fn get_fb() -> Option<&'static mut FrameBuffer> {
    unsafe { FRAMEBUFFER.as_mut() }
}

/// Test pattern: render a color gradient and text to verify framebuffer works.
pub fn test_pattern() {
    if let Some(fb) = get_fb() {
        // Draw a color bar at the top.
        let bar_height = 20;
        let colors = [
            Color::RED, Color::GREEN, Color::BLUE,
            Color::CYAN, Color::MAGENTA, Color::YELLOW,
        ];
        let bar_width = fb.width() / colors.len() as u64;
        for (i, &color) in colors.iter().enumerate() {
            fb.fill_rect(
                (i as u64) * bar_width,
                0,
                bar_width,
                bar_height,
                color,
            );
        }

        // Draw a border.
        fb.draw_rect(0, 0, fb.width() - 1, fb.height() - 1, Color::WHITE);
    }
}
