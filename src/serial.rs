//! 16550A UART driver — COM1 (0x3F8), 115200 baud, 8N1, polling TX.
//!
//! All VGA output is mirrored here so `-nographic` gives a clean terminal feed.

use x86_64::instructions::port::Port;

const COM1: u16 = 0x3F8;

pub fn init() {
    unsafe {
        // Disable interrupts
        Port::<u8>::new(COM1 + 1).write(0x00);
        // Enable DLAB (set baud divisor)
        Port::<u8>::new(COM1 + 3).write(0x80);
        // Baud divisor low byte = 1 → 115200
        Port::<u8>::new(COM1 + 0).write(0x01);
        Port::<u8>::new(COM1 + 1).write(0x00);
        // 8 bits, no parity, one stop bit (DLAB cleared)
        Port::<u8>::new(COM1 + 3).write(0x03);
        // Enable FIFO, clear, 14-byte threshold
        Port::<u8>::new(COM1 + 2).write(0xC7);
        // RTS/DSR set
        Port::<u8>::new(COM1 + 4).write(0x0B);
    }
}

#[inline]
fn tx_ready() -> bool {
    unsafe { Port::<u8>::new(COM1 + 5).read() & 0x20 != 0 }
}

pub fn write_byte(b: u8) {
    // Spin until transmit buffer empty (THRE bit)
    while !tx_ready() {}
    unsafe { Port::<u8>::new(COM1).write(b); }
}

pub fn write_str(s: &str) {
    for b in s.bytes() {
        if b == b'\n' { write_byte(b'\r'); }
        write_byte(b);
    }
}

/// True if the UART has a byte waiting in the RX FIFO.
#[inline]
pub fn rx_ready() -> bool {
    unsafe { Port::<u8>::new(COM1 + 5).read() & 0x01 != 0 }
}

/// Read one byte from the UART if available.
pub fn try_read() -> Option<u8> {
    if rx_ready() {
        Some(unsafe { Port::<u8>::new(COM1).read() })
    } else {
        None
    }
}
