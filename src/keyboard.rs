//! PS/2 keyboard driver — scancode → ASCII with lock-free ring buffer.
//!
//! No mutex — we disable interrupts around all access, and everything runs
//! on one CPU. The IRQ handler and the main code are the only producers/
//! consumers, so atomics are sufficient for synchronization.

use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use x86_64::instructions::port::Port;

const BUF_SIZE: usize = 256;
const BUF_MASK: usize = BUF_SIZE - 1;

static BUF: [AtomicU8; BUF_SIZE] = {
    const INIT: AtomicU8 = AtomicU8::new(0);
    [INIT; BUF_SIZE]
};
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);
static SHIFT: AtomicBool = AtomicBool::new(false);
static CAPS: AtomicBool = AtomicBool::new(false);

// US layout: [normal, shifted]
static LOWER: [u8; 58] = [
    0,    0x1B, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0',
    b'-', b'=', 0x08, b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i',
    b'o', b'p', b'[', b']', b'\n', 0,   b'a', b's', b'd', b'f', b'g', b'h',
    b'j', b'k', b'l', b';', b'\'', b'`', 0,   b'\\', b'z', b'x', b'c', b'v',
    b'b', b'n', b'm', b',', b'.', b'/', 0,   b'*', 0,   b' ',
];

static UPPER: [u8; 58] = [
    0,    0x1B, b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')',
    b'_', b'+', 0x08, b'\t', b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I',
    b'O', b'P', b'{', b'}', b'\n', 0,   b'A', b'S', b'D', b'F', b'G', b'H',
    b'J', b'K', b'L', b':', b'"', b'~', 0,   b'|', b'Z', b'X', b'C', b'V',
    b'B', b'N', b'M', b'<', b'>', b'?', 0,   b'*', 0,   b' ',
];

/// Push a byte into the ring buffer. Called from IRQ context.
fn push(byte: u8) {
    let tail = TAIL.load(Ordering::Relaxed);
    let next = (tail + 1) & BUF_MASK;
    let head = HEAD.load(Ordering::Relaxed);
    if next != head {
        BUF[tail].store(byte, Ordering::Relaxed);
        TAIL.store(next, Ordering::Release);
    }
}

/// Pop a byte from the ring buffer. Must be called with interrupts disabled.
pub fn pop() -> Option<u8> {
    let head = HEAD.load(Ordering::Relaxed);
    let tail = TAIL.load(Ordering::Acquire);
    if head == tail {
        return None;
    }
    let b = BUF[head].load(Ordering::Relaxed);
    HEAD.store((head + 1) & BUF_MASK, Ordering::Release);
    Some(b)
}

/// Check if the buffer is empty. Must be called with interrupts disabled.
pub fn is_empty() -> bool {
    let head = HEAD.load(Ordering::Relaxed);
    let tail = TAIL.load(Ordering::Acquire);
    head == tail
}

/// Called from the IRQ-1 interrupt handler. Reads one scancode and pushes
/// the decoded ASCII byte to the ring buffer.
pub fn handle_scancode() {
    let scancode: u8 = unsafe { Port::new(0x60).read() };

    match scancode {
        0x2A | 0x36 => { SHIFT.store(true, Ordering::Relaxed); }
        0xAA | 0xB6 => { SHIFT.store(false, Ordering::Relaxed); }
        0x3A => { CAPS.store(!CAPS.load(Ordering::Relaxed), Ordering::Relaxed); }
        sc if sc < 0x80 => {
            let idx = sc as usize;
            if idx < LOWER.len() {
                let shift = SHIFT.load(Ordering::Relaxed);
                let caps = CAPS.load(Ordering::Relaxed);
                let shifted = shift ^ (caps && idx >= 0x10 && idx <= 0x32);
                let ascii = if shifted { UPPER[idx] } else { LOWER[idx] };
                if ascii != 0 {
                    push(ascii);
                }
            }
        }
        _ => {}
    }
}
