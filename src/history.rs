//! Kernel line-history ring buffer.
//!
//! All text passing through `_print` is accumulated here line by line.
//! The `history` shell command replays the last N lines directly to VGA+serial,
//! bypassing `_print` to avoid recursion.

use spin::Mutex;
use lazy_static::lazy_static;

pub const HIST_LINES: usize = 500;
pub const HIST_WIDTH: usize = 80;

pub struct History {
    /// Circular buffer of complete lines (no trailing \n stored)
    lines: [[u8; HIST_WIDTH]; HIST_LINES],
    lens:  [u8; HIST_LINES],
    head:  usize,   // next write slot
    count: usize,   // valid entries (saturates at HIST_LINES)
    /// Accumulator for the current incomplete line
    cur:     [u8; HIST_WIDTH],
    cur_len: usize,
}

impl History {
    const fn new() -> Self {
        History {
            lines: [[0u8; HIST_WIDTH]; HIST_LINES],
            lens:  [0u8; HIST_LINES],
            head:  0,
            count: 0,
            cur:     [0u8; HIST_WIDTH],
            cur_len: 0,
        }
    }

    fn commit_cur(&mut self) {
        let n = self.cur_len.min(HIST_WIDTH);
        self.lines[self.head][..n].copy_from_slice(&self.cur[..n]);
        self.lens[self.head] = n as u8;
        self.head  = (self.head + 1) % HIST_LINES;
        if self.count < HIST_LINES { self.count += 1; }
        self.cur_len = 0;
    }

    /// Feed a raw string (may contain \n).
    pub fn feed(&mut self, s: &str) {
        for b in s.bytes() {
            if b == b'\n' {
                self.commit_cur();
            } else if self.cur_len < HIST_WIDTH {
                self.cur[self.cur_len] = b;
                self.cur_len += 1;
            }
            // silently drop chars past HIST_WIDTH on one line
        }
    }

    /// Iterate over stored lines from oldest to newest.
    pub fn for_each_line(&self, mut f: impl FnMut(&str)) {
        let start = if self.count == HIST_LINES { self.head } else { 0 };
        for i in 0..self.count {
            let idx = (start + i) % HIST_LINES;
            let n   = self.lens[idx] as usize;
            if let Ok(s) = core::str::from_utf8(&self.lines[idx][..n]) {
                f(s);
            }
        }
    }

    pub fn line_count(&self) -> usize { self.count }
}

impl core::fmt::Write for History {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.feed(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref HISTORY: Mutex<History> = Mutex::new(History::new());
}

/// Called from `_print`; feeds text into the ring buffer.
pub fn feed(s: &str) {
    HISTORY.lock().feed(s);
}

/// Replay stored lines directly to VGA + serial, bypassing `_print`
/// so we don't push history-of-history.
pub fn replay(last_n: usize) {
    use crate::vga::WRITER;
    use core::fmt::Write;

    let hist = HISTORY.lock();
    let total = hist.line_count();
    let skip  = if total > last_n { total - last_n } else { 0 };
    let mut lineno = 0usize;

    hist.for_each_line(|line| {
        if lineno >= skip {
            // Write directly — no `println!` to avoid re-entrancy
            crate::serial::write_str(line);
            crate::serial::write_str("\n");
            let mut w = WRITER.lock();
            let _ = w.write_str(line);
            let _ = w.write_str("\n");
        }
        lineno += 1;
    });
}
