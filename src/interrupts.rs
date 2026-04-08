//! Interrupt descriptor table + 8259 PIC initialization.
//!
//! IRQ remapping: PIC1 → vectors 0x20-0x27, PIC2 → 0x28-0x2F.
//! Only IRQ1 (keyboard, vector 0x21) is unmasked.

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;
use core::sync::atomic::{AtomicU64, Ordering};

/// Incremented on every IRQ0 (PIT timer tick, ~18.2 Hz).
/// Used by bench.rs for TSC calibration.
pub static TICK_COUNT: AtomicU64 = AtomicU64::new(0);

// ── PIC constants ─────────────────────────────────────────────────────────────
const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const EOI:        u8 = 0x20;

pub const INT_KEYBOARD: u8 = 0x21; // IRQ1 remapped to vector 0x21

/// Initialize the 8259 PIC: remap IRQs to 0x20-0x2F, mask everything except
/// IRQ1 (keyboard).
pub fn init_pic() {
    unsafe {
        let mut p1c: Port<u8> = Port::new(PIC1_CMD);
        let mut p1d: Port<u8> = Port::new(PIC1_DATA);
        let mut p2c: Port<u8> = Port::new(PIC2_CMD);
        let mut p2d: Port<u8> = Port::new(PIC2_DATA);

        // ICW1 — start init sequence
        p1c.write(0x11);
        p2c.write(0x11);

        // ICW2 — vector offsets
        p1d.write(0x20);  // PIC1 starts at IRQ vector 0x20
        p2d.write(0x28);  // PIC2 starts at IRQ vector 0x28

        // ICW3 — cascade
        p1d.write(0x04);  // IRQ2 line connects to PIC2
        p2d.write(0x02);  // PIC2 cascade identity = 2

        // ICW4 — 8086 mode
        p1d.write(0x01);
        p2d.write(0x01);

        // OCW1 — masks: enable IRQ0 (timer) + IRQ1 (keyboard), mask rest
        // IRQ0 needed to wake CPU from HLT so the serial UART is polled
        p1d.write(0b1111_1100);  // bit 0 = IRQ0 = 0 → enabled, bit 1 = IRQ1 = 0 → enabled
        p2d.write(0xFF);
    }
}

/// Send end-of-interrupt to PIC1 (and PIC2 if the IRQ came from it).
#[inline]
pub fn eoi(irq_vector: u8) {
    unsafe {
        if irq_vector >= 0x28 {
            Port::<u8>::new(PIC2_CMD).write(EOI);
        }
        Port::<u8>::new(PIC1_CMD).write(EOI);
    }
}

// ── IDT ───────────────────────────────────────────────────────────────────────
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
    unsafe {
        let idt = &mut *core::ptr::addr_of_mut!(IDT);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // IRQ0 timer — needed to avoid spurious interrupt panic
        idt[0x20].set_handler_fn(timer_handler);
        // IRQ1 keyboard
        idt[0x21].set_handler_fn(keyboard_handler);
        let idt = &*core::ptr::addr_of!(IDT);
        idt.load();
    }
    init_pic();
    // Enable hardware interrupts
    x86_64::instructions::interrupts::enable();
}

// ── Handlers ──────────────────────────────────────────────────────────────────
extern "x86-interrupt" fn breakpoint_handler(_sf: InterruptStackFrame) {
    crate::println!("[INT] Breakpoint");
}

extern "x86-interrupt" fn timer_handler(_sf: InterruptStackFrame) {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    eoi(0x20);
}

extern "x86-interrupt" fn keyboard_handler(_sf: InterruptStackFrame) {
    crate::keyboard::handle_scancode();
    eoi(0x21);
}
