//! Kernel performance benchmarks — RDTSC cycle counting + PIT calibration.
//!
//! Calibration: waits for two consecutive IRQ0 ticks (~54.9 ms apart at
//! PIT default divisor 65536 / 1.193182 MHz) and measures the TSC delta
//! to derive approximate CPU MHz. All bench results are then expressed in
//! both cycles/op and Mop/s.

extern crate alloc;
use alloc::vec::Vec;

use core::sync::atomic::Ordering;
use crate::interrupts::TICK_COUNT;

// ── RDTSC ────────────────────────────────────────────────────────────────────

#[inline(always)]
fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nostack, nomem, preserves_flags)
        );
    }
    ((hi as u64) << 32) | lo as u64
}

/// Estimate CPU frequency in MHz by measuring TSC ticks over one PIT period
/// (~54.9 ms). Spins (interrupts enabled) waiting for two consecutive ticks.
pub fn calibrate_mhz() -> u64 {
    // Wait for a tick edge
    let t0 = TICK_COUNT.load(Ordering::Relaxed);
    while TICK_COUNT.load(Ordering::Relaxed) == t0 {
        core::hint::spin_loop();
    }
    let tsc0 = rdtsc();
    let t1 = TICK_COUNT.load(Ordering::Relaxed);
    while TICK_COUNT.load(Ordering::Relaxed) == t1 {
        core::hint::spin_loop();
    }
    let tsc1 = rdtsc();

    // One PIT period = 65536 / 1_193_182 Hz = 54_925 μs
    // MHz = delta_cycles / 54_925  (cycles per μs = MHz)
    let delta = tsc1.saturating_sub(tsc0);
    delta / 54_926
}

// ── Benchmark result ─────────────────────────────────────────────────────────

pub struct BenchResult {
    pub name: &'static str,
    pub iters: u64,
    pub cycles: u64,
    pub mhz: u64,
}

impl BenchResult {
    pub fn cycles_per_op(&self) -> u64 {
        if self.iters == 0 { return 0; }
        self.cycles / self.iters
    }

    /// Millions of operations per second (0 if mhz unknown)
    pub fn mops(&self) -> u64 {
        let cpo = self.cycles_per_op();
        if cpo == 0 || self.mhz == 0 { return 0; }
        self.mhz / cpo
    }
}

// ── Individual benchmarks ─────────────────────────────────────────────────────

const ALEPH_ITERS: u64 = 100_000;
const HEAP_ITERS:  u64 = 10_000;
const NULL_ITERS:  u64 = 1_000_000;

/// Baseline: measure loop overhead (should be ~1-2 cy/op)
fn bench_null(mhz: u64) -> BenchResult {
    let iters = NULL_ITERS;
    let mut acc: u64 = 0;
    let t0 = rdtsc();
    for i in 0..iters {
        // volatile-ish: use the counter so the loop isn't elided
        unsafe { core::ptr::write_volatile(&mut acc, i); }
    }
    let _ = acc;
    BenchResult { name: "null loop", iters, cycles: rdtsc() - t0, mhz }
}

fn bench_tensor(mhz: u64) -> BenchResult {
    use crate::aleph::{LETTERS, tensor};
    let a = &LETTERS[12].t; // mem
    let b = &LETTERS[20].t; // shin
    let iters = ALEPH_ITERS;
    let mut r = [0u8; 12];
    let t0 = rdtsc();
    for _ in 0..iters {
        unsafe { core::ptr::write_volatile(&mut r, tensor(a, b)); }
    }
    let _ = unsafe { core::ptr::read_volatile(&r) };
    BenchResult { name: "aleph::tensor", iters, cycles: rdtsc() - t0, mhz }
}

fn bench_join(mhz: u64) -> BenchResult {
    use crate::aleph::{LETTERS, join};
    let a = &LETTERS[0].t;  // aleph
    let b = &LETTERS[12].t; // mem
    let iters = ALEPH_ITERS;
    let mut r = [0u8; 12];
    let t0 = rdtsc();
    for _ in 0..iters {
        unsafe { core::ptr::write_volatile(&mut r, join(a, b)); }
    }
    let _ = unsafe { core::ptr::read_volatile(&r) };
    BenchResult { name: "aleph::join", iters, cycles: rdtsc() - t0, mhz }
}

fn bench_meet(mhz: u64) -> BenchResult {
    use crate::aleph::{LETTERS, meet};
    let a = &LETTERS[4].t;  // hei
    let b = &LETTERS[20].t; // shin
    let iters = ALEPH_ITERS;
    let mut r = [0u8; 12];
    let t0 = rdtsc();
    for _ in 0..iters {
        unsafe { core::ptr::write_volatile(&mut r, meet(a, b)); }
    }
    let _ = unsafe { core::ptr::read_volatile(&r) };
    BenchResult { name: "aleph::meet", iters, cycles: rdtsc() - t0, mhz }
}

fn bench_distance(mhz: u64) -> BenchResult {
    use crate::aleph::{LETTERS, distance_scaled};
    let a = &LETTERS[0].t;  // aleph
    let b = &LETTERS[12].t; // mem
    let iters = ALEPH_ITERS;
    let mut r: u32 = 0;
    let t0 = rdtsc();
    for _ in 0..iters {
        unsafe { core::ptr::write_volatile(&mut r, distance_scaled(a, b)); }
    }
    let _ = unsafe { core::ptr::read_volatile(&r) };
    BenchResult { name: "aleph::distance", iters, cycles: rdtsc() - t0, mhz }
}

fn bench_tier(mhz: u64) -> BenchResult {
    use crate::aleph::{LETTERS, compute_tier, Tier};
    let iters = ALEPH_ITERS;
    let mut r: Tier = Tier::O0;
    let t0 = rdtsc();
    for i in 0..iters {
        let t = compute_tier(&LETTERS[(i % 22) as usize].t);
        unsafe { core::ptr::write_volatile(&mut r, t); }
    }
    let _ = unsafe { core::ptr::read_volatile(&r) };
    BenchResult { name: "aleph::tier", iters, cycles: rdtsc() - t0, mhz }
}

fn bench_system_language(mhz: u64) -> BenchResult {
    use crate::aleph::system_language;
    let iters = 10_000u64;
    let t0 = rdtsc();
    for _ in 0..iters {
        let _ = system_language();
    }
    BenchResult { name: "aleph::system_lang (JOIN 22)", iters, cycles: rdtsc() - t0, mhz }
}

fn bench_heap_alloc(mhz: u64) -> BenchResult {
    let iters = HEAP_ITERS;
    let layout = core::alloc::Layout::from_size_align(64, 8).unwrap();
    let mut ptrs: Vec<*mut u8> = Vec::with_capacity(iters as usize);
    let t0 = rdtsc();
    for _ in 0..iters {
        let p = unsafe { alloc::alloc::alloc(layout) };
        ptrs.push(p);
    }
    let cycles = rdtsc() - t0;
    // free all (not counted)
    for p in ptrs {
        unsafe { alloc::alloc::dealloc(p, layout); }
    }
    BenchResult { name: "heap alloc 64B", iters, cycles, mhz }
}

fn bench_heap_alloc_free(mhz: u64) -> BenchResult {
    let iters = HEAP_ITERS;
    let layout = core::alloc::Layout::from_size_align(64, 8).unwrap();
    let t0 = rdtsc();
    for _ in 0..iters {
        let p = unsafe { alloc::alloc::alloc(layout) };
        unsafe { alloc::alloc::dealloc(p, layout); }
    }
    BenchResult { name: "heap alloc+free 64B", iters, cycles: rdtsc() - t0, mhz }
}

// ── Run all ───────────────────────────────────────────────────────────────────

pub fn run_all() -> Vec<BenchResult> {
    let mhz = calibrate_mhz();

    let mut results = Vec::new();
    results.push(bench_null(mhz));
    results.push(bench_tensor(mhz));
    results.push(bench_join(mhz));
    results.push(bench_meet(mhz));
    results.push(bench_distance(mhz));
    results.push(bench_tier(mhz));
    results.push(bench_system_language(mhz));
    results.push(bench_heap_alloc(mhz));
    results.push(bench_heap_alloc_free(mhz));
    results
}
