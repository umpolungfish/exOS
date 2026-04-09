<div align="center">
  <h1>USAGE — exoterik_OS Φ_c Kernel</h1>
  <p><b>Build, run, REPL reference, ALFS, programs, subsystem API, and extension guide</b></p>
  <img src="exOS.png" alt="exoterik_OS banner" width="400">
</div>

<div align="center">
  <img src="https://img.shields.io/badge/LANGUAGE-Rust%20Nightly-blue" alt="Language">
  <img src="https://img.shields.io/badge/TARGET-x86__64--unknown--none-orange" alt="Target">
  <img src="https://img.shields.io/badge/BOOT-UEFI%20OVMF-red" alt="Boot">
  <img src="https://img.shields.io/badge/ENGINE-SynthOmnicon%20v0.4.27-purple" alt="Engine">
  <img src="https://img.shields.io/badge/ALEPH-v0.5.0%20Native-green" alt="ALEPH">
</div>

<p align="center">
  <a href="#1-build-and-installation">Build</a> •
  <a href="#2-running-the-kernel">Run</a> •
  <a href="#3-boot-sequence">Boot Sequence</a> •
  <a href="#4-aleph-repl-reference">ALEPH REPL</a> •
  <a href="#5-alfs--programs">ALFS & Programs</a> •
  <a href="#6-subsystem-api-reference">API Reference</a> •
  <a href="#7-extending-the-kernel">Extending</a> •
  <a href="#8-troubleshooting">Troubleshooting</a>
</p>

<hr>

## 1. Build and Installation

### Prerequisites

```bash
# Rust nightly toolchain
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src --toolchain nightly
rustup target add x86_64-unknown-none --toolchain nightly

# QEMU
sudo apt install qemu-system-x86   # Ubuntu/Debian
sudo pacman -S qemu-full           # Arch

# UEFI firmware (OVMF)
sudo apt install ovmf              # Ubuntu/Debian
sudo pacman -S edk2-ovmf           # Arch

# mtools (for mcopy in build_bootimage.sh)
sudo apt install mtools
```

### Building

```bash
# Kernel ELF only (fast — for checking compilation)
cargo build --release

# Full UEFI bootable disk image (kernel + bootloader + FAT32 ESP)
./build_bootimage.sh
```

`build_bootimage.sh` does three things:
1. Builds the kernel ELF at `target/x86_64-unknown-none/release/exoterik-os`
2. Compiles the UEFI bootloader with the kernel embedded
3. Creates a 64 MB FAT32 disk image with the EFI system partition

### Adding Programs

Drop a `.aleph` file into `programs/`, then register it in `src/programs.rs`:

```rust
BuiltinProgram { name: "my_program.aleph", data: include_bytes!("../programs/my_program.aleph") },
```

Rebuild. On next boot the kernel seeds it to ALFS automatically.

<hr>

## 2. Running the Kernel

### Graphical Mode (default)

```bash
./run.sh
```

Boots with the UEFI GOP framebuffer. Hebrew letters are rendered as hand-drawn 8×16 bitmap glyphs. Text output goes to both screen and a serial PTY (printed to terminal: `char device redirected to /dev/pts/N`).

### Serial Mode

```bash
./run.sh --serial
```

No graphics window. All output is piped to stdio. Hebrew letters display as ASCII transliterations (A, B, G, D...). Press `Ctrl-A` then `X` to quit QEMU.

### ALFS Disk

`run.sh` automatically creates `alfs.img` (32 MB) if it doesn't exist, writing the ALFS superblock. The kernel mounts this as ATA primary slave on boot and seeds all built-in programs.

To start with a clean disk:
```bash
rm alfs.img && ./run.sh
```

<hr>

## 3. Boot Sequence

```
[BOOT] Heap: initialized (4MB)
[BOOT] UEFI GOP Framebuffer: 1024x768 @ 32bpp
[exoterikOS] Phi_c Kernel booting...
P_pm_sym -> P_asym symmetry break initiated.
[INIT] Three-layer objects: all structural/operational/determinative variants exercised
[SCHED] Ergative scheduler online, symmetry broken
[MEM] Phonological allocator: Velar -> Bilabial gradient online
[FS] Sefirot tree: Keter -> Malkuth, 10 layers mapped
[IPC] Three-layer message: well_formed=true len=5
[CMD] Generative command: gematria=356 pratyahara=356
[ALEPH] 22-letter type system online. O_inf: 3, O_2: 6, O_1: 1, O_0: 12
[TYPE] IPC gate (close): accepted=true
[TYPE] IPC gate (remote): accepted=false
[TYPE] Omega gate (Velar+Kernel): allowed=true
[TYPE] Omega gate (Velar+User): allowed=false
[TYPE] Tier gate (O_inf ergative): ok=true
[TYPE] Tier gate (O_0 ergative): ok=false
[TYPE] Phi gate (Keter+Kernel): ok=true
[TYPE] Phi gate (Keter+Driver): ok=false
[TYPE] C scores: kernel=0.873 user=0.324 os_synthon=0.873
[FS] ALFS v1: 14 files, 0 used / 1024 free sectors - files: 14
[exoterikOS] Phi_c Kernel fully online. Type 'help' for commands.
exOS>
```

Each phase is structurally mandatory — not arbitrary initialization order. The symmetry-breaking interrupt init must precede any process scheduling; the heap must precede any allocation; ALFS must mount before programs are available.

<hr>

## 4. ALEPH REPL Reference

### Entering

```
exOS> aleph
```

### Language Syntax

```
expr ::= letter_name               aleph, bet, gimel, ... tav
       | var_name                  previously bound with let
       | expr x expr               tensor  (P,F,K: min; rest: max)
       | expr v expr               join    (component-wise max)
       | expr ^ expr               meet    (component-wise min)
       | expr ::> expr             vav-cast
       | mediate(w, a, b)          triadic: w v (a x b)
       | palace(n) expr            tier barrier gate (n = 1..7)
       | probe_Phi(expr)           criticality primitive report
       | probe_Omega(expr)         protection primitive report
       | tier(expr)                ouroboricity tier report
       | d(expr, expr)             structural distance + conflict set
       | system()                  JOIN of all 22 letters
       | census()                  tier distribution table
       | let name = expr           bind result in session
       | match expr { pat => expr, ... }   tier pattern match
```

Tier patterns: `O_0`, `O_1`, `O_2`, `O_2d`, `O_inf`, `_` (wildcard).

### Letter Names

Letters can be referenced by full name or single-letter alias:

| Name | Alias | Glyph | Tier |
|:-----|:------|:------|:-----|
| aleph | A | א | O_2 |
| bet | B | ב | O_0 |
| gimel | G | ג | O_0 |
| dalet | D | ד | O_0 |
| hei | H | ה | O_2 |
| vav | V | ו | **O_inf** |
| zayin | Z | ז | O_0 |
| chet | C | ח | O_0 |
| tet | T | ט | O_0 |
| yod | Y | י | O_0 |
| kaf | K | כ | O_0 |
| lamed | L | ל | O_1 |
| mem | M | מ | **O_inf** |
| nun | N | נ | O_0 |
| samech | S | ס | O_0 |
| ayin | E | ע | O_2 |
| pei | P | פ | O_0 |
| tzadi | Q | צ | O_0 |
| kuf | U | ק | O_2 |
| resh | R | ר | O_0 |
| shin | X | ש | **O_inf** |
| tav | O | ת | O_2 |

### REPL Commands

| Command | Description |
|:--------|:------------|
| `:help` | Full syntax reference |
| `:tips` | Quick start examples |
| `:quit` / `:q` | Return to shell |
| `:ls` | List session bindings (name, tier, Φ, Ω, glyph) |
| `:tuple name` | Visual 12-primitive bar chart |
| `:explain name` | Detailed type breakdown + C score |
| `:tier name` | Ouroboricity tier of one letter |
| `:census` | Tier distribution across all 22 letters |
| `:system` | Compute JOIN of all 22 letters |
| `:orbit N letter pole` | Live convergence orbit (see below) |
| `:files` | List files on ALFS |
| `:save name` | Save last result to ALFS as `name.aleph` |
| `:save name expr` | Save expression text to ALFS as `name.aleph` |
| `:load name` | Load and bind an `.aleph` file from ALFS |
| `:run name` | Run an `.aleph` file from ALFS, print results |
| `:history` | Show command history for this session |
| `:scroll [N]` | Replay last N lines of output (default 40) |
| `:clear` | Clear screen |

### :orbit — Live Frobenius Convergence

```
:orbit N letter pole
```

Applies `state = state ⊗ pole` N times from `letter`. At each step prints:
- Step number
- Nearest canonical letter (by weighted distance)
- Ouroboricity tier
- Distance to pole
- Convergence delta (color-coded)

Exits early if distance drops below 0.01 (fully converged).

```
A> :orbit 6 lamed vav
  Orbit of L under V (6 steps)
  step  nearest        tier     d(state,pole)  delta
  --------------------------------------------------------
     0  L (lamed)      O_1      2.3875
     1  V (vav)        O_inf    0.0000  (fixed)
  -- converged at step 1 --
```

The three O_inf poles (vav, mem, shin) are **Frobenius attractors** — most letters converge in 1–2 tensor steps. Mediation chains and palace-guarded states may take longer.

<hr>

## 5. ALFS & Programs

### Filesystem Layout

ALFS lives on `alfs.img` (32 MB), mounted as ATA primary slave (drive 1):

```
Sector 0      : Superblock (magic "ALFS", version, file count, sector bitmap)
Sectors 1–16  : Directory (128 entries × 64 bytes; 8 per sector)
Sectors 17+   : File data (contiguous sector runs, first-fit allocation)
```

Max 128 files. Max 512 KB per file (1024 data sectors). Filenames up to 31 chars.

### Program Seeding

On first boot (or after `rm alfs.img`), the kernel writes all built-in programs from `programs/` to ALFS. These are compiled in via `include_bytes!` in `src/programs.rs` — no host-side tool needed.

### REPL File Operations

```
A> :files                        # list all files on ALFS
A> :run frobenius_orbits         # run frobenius_orbits.aleph
A> :save experiment              # save last computed result
A> :save tikkun aleph x mem      # save expression text
A> :load tikkun                  # load and bind tikkun.aleph result
```

Saved files persist across reboots in `alfs.img`.

### Built-in Programs

Run any of these with `:run name` (omit `.aleph`):

| File | What it demonstrates |
|:-----|:--------------------|
| `frobenius.aleph` | Self-idempotency + cross-distances of vav, mem, shin |
| `frobenius_orbits.aleph` | 4-step unrolled convergence orbits + mediation stability |
| `creation.aleph` | Structural genesis from first light |
| `creation_liturgy.aleph` | Full liturgical sequence through all tiers |
| `meditation.aleph` | Deep mediation chains |
| `selfreplicating_light.aleph` | Self-replicating structure via mediate |
| `light_stability.aleph` | Stability analysis under perturbation |
| `light_replication_kernel.aleph` | Kernel-level light replication with palace barriers |
| `tikkun_construction_full.aleph` | Healing anomalous objects via palace + mediate |
| `tikkun_palace_verification.aleph` | Palace-gate verification across all Sefirot levels |
| `exploration_primitives.aleph` | Primitive-by-primitive exploration |
| `distance_probes_indistinguishable.aleph` | Distance + conflict-set across all 22 letters |
| `pratyahara.aleph` | Varnamala compression via tensor chains |

<hr>

## 6. Subsystem API Reference

### 6.1 VGA / Framebuffer (`vga.rs`, `framebuffer.rs`, `font_renderer.rs`)

The writer is mode-aware. In graphical mode (UEFI GOP), `write_byte` calls `render_char` which:
- Bytes `0x20–0x7E`: renders from the built-in 8×16 PC font
- Bytes `0xE0–0xF5`: renders from `HEBREW_FONT` (22 hand-drawn Hebrew bitmaps)

`display_glyph(letter)` returns the appropriate code point for the current mode:
- Framebuffer: `0xE0 + letter_index` → Hebrew bitmap
- VGA text: ASCII transliteration (`A`, `B`, `G`...)

Scale is 1× (native 8×16 pixels). To change: edit `SCALE` in `font_renderer.rs`.

### 6.2 ATA Driver (`ata.rs`)

```rust
pub static mut ATA_DRIVE: u8;  // 0 = primary master (boot), 1 = primary slave (ALFS)

pub fn read_sector(lba: u32) -> Option<[u8; 512]>
pub fn read_sectors(start_lba: u32, count: usize, out: &mut [u8]) -> Option<()>
pub fn write_sector(lba: u32, data: &[u8; 512]) -> Option<()>
pub fn write_sectors(start_lba: u32, count: usize, data: &[u8]) -> Option<()>
```

Drive selection is global. ALFS sets `ATA_DRIVE = 1` at mount time and holds it there. All ALFS I/O targets the data disk.

### 6.3 ALFS (`alfs.rs`)

```rust
pub fn mount() -> Result<(), &'static str>
pub fn is_mounted() -> bool
pub fn list() -> Vec<FileInfo>
pub fn find_file(name: &str) -> Option<FileInfo>
pub fn read_file(name: &str) -> Option<Vec<u8>>
pub fn read_file_string(name: &str) -> Option<String>
pub fn write_file(name: &str, data: &[u8], file_type: u8) -> Result<usize, &'static str>
pub fn delete_file(name: &str) -> Result<(), &'static str>
pub fn info() -> String

pub const TYPE_DATA:  u8 = 0;
pub const TYPE_ALEPH: u8 = 1;
pub const TYPE_TEMP:  u8 = 2;
```

### 6.4 ALEPH Type System (`aleph.rs`)

```rust
pub type Tuple = [u8; 12];   // [D, T, R, P, F, K, G, Γ, Φ, H, S, Ω]

// Core lattice operations
pub fn tensor(a: &Tuple, b: &Tuple) -> Tuple   // P,F,K: min; rest: max
pub fn join(a: &Tuple, b: &Tuple) -> Tuple      // component-wise max
pub fn meet(a: &Tuple, b: &Tuple) -> Tuple      // component-wise min
pub fn mediate(w: &Tuple, a: &Tuple, b: &Tuple) -> Tuple  // w v (a x b)

// Distance
pub fn distance(a: &Tuple, b: &Tuple) -> f64
pub fn distance_scaled(a: &Tuple, b: &Tuple) -> u32   // × 100
pub fn conflict_set(a: &Tuple, b: &Tuple) -> Vec<usize>

// Lookup
pub fn resolve_letter(input: &str) -> Option<&'static LetterDef>
pub fn nearest_letter(t: &Tuple) -> &'static LetterDef

// Tier
pub fn compute_tier(t: &Tuple) -> Tier
pub fn tier_name(t: Tier) -> &'static str

// Display
pub fn display_glyph(l: &LetterDef) -> &'static str   // mode-aware
pub fn format_letter(l: &LetterDef) -> String
pub fn format_tuple(l: &LetterDef) -> String
pub fn format_explain(l: &LetterDef) -> String

// Aggregates
pub fn system_language() -> Tuple    // JOIN of all 22
pub fn tier_census() -> [usize; 5]
```

### 6.5 Kernel Object (`kernel_object.rs`)

```rust
pub enum StructuralType  { Process, File, Socket, Semaphore, MemoryRegion }
pub enum OperationalMode { Compute, IO, Network, MemoryManage, Schedule, Idle }
pub enum Determinative   { Kernel, Service, User, Driver, Init }

pub struct KernelObject {
    pub structural:   StructuralType,
    pub operational:  OperationalMode,
    pub determinative: Determinative,
    pub aleph_type:   AlephKernelType,
    pub id: u64,
}

impl KernelObject {
    pub fn new(structural, operational, determinative, id) -> Self;
    pub fn is_well_formed(&self) -> bool;
}
```

### 6.6 Ergative Scheduler (`scheduler.rs`)

```rust
pub enum GrammaticalRole { Ergative, Absolutive }

pub struct ProcessControlBlock {
    pub id: u64,
    pub obj: KernelObject,
    pub role: GrammaticalRole,
    pub priority: u8,
    pub stack_pointer: u64,
    pub targets: Vec<u64>,
}

impl ProcessControlBlock {
    pub fn determine_role(&mut self);        // Ergative if targets non-empty
    pub fn effective_priority(&self) -> u8;  // +10 for Ergative
}

impl ErgativeScheduler {
    pub fn new() -> Self;
    pub fn break_symmetry(&mut self);
    pub fn is_symmetric(&self) -> bool;
    pub fn spawn(&mut self, pcb: ProcessControlBlock);
    pub fn schedule_next(&mut self) -> Option<&ProcessControlBlock>;
}
```

### 6.7 Phonological Memory (`memory.rs`)

```rust
pub enum ArticulationDepth {
    Velar = 0,    // Ω_Z,  maximum protection, validated
    Palatal = 1,  // Ω_Z,  high protection, validated
    Retroflex = 2,// Ω_Z₂, medium, validated
    Dental = 3,   // Ω_0,  low, unchecked
    Bilabial = 4, // Ω_0,  none, fastest
}

impl PhonologicalAllocator {
    pub fn new() -> Self;
    pub fn set_depth(&mut self, depth: ArticulationDepth);
    pub fn allocate(&self, layout: Layout) -> Option<*mut u8>;
    pub fn deallocate(&self, ptr: *mut u8, layout: Layout);
}
```

### 6.8 Sefirot Filesystem (`filesystem.rs`)

In-memory tree with 10 Sefirot layers. Not persistent — use ALFS for persistence.

```rust
pub enum Sefirah {
    Keter, Chokhmah, Binah, Daat,
    Chesed, Gevurah, Tiferet, Netzach, Hod, Yesod, Malkuth,
}

impl SefirotFs {
    pub fn new() -> Self;
    pub fn navigate_to(&mut self, target: Sefirah);
    pub fn current(&self) -> Sefirah;
    pub fn write(&self, name: &str, data: &[u8]);
    pub fn read(&self, name: &str) -> Option<Vec<u8>>;
    pub fn tree(&self) -> &'static [(Sefirah, &'static str)];
}
```

### 6.9 IPC (`ipc.rs`)

```rust
pub struct IpcMessage { /* structural signature, payload, determinative */ }

impl IpcMessage {
    pub fn new(sig: StructuralSignature, payload: &'static [u8], det: MessageDeterminative) -> Self;
    pub fn is_well_formed(&self) -> bool;
    pub fn len(&self) -> usize;
}
```

<hr>

## 7. Extending the Kernel

### Adding a new ALEPH built-in function

1. Add a new `Expr` variant in `aleph_parser.rs`
2. Add tokenization in `tokenize()` and parsing in `parse_primary()`
3. Add evaluation in `aleph_eval.rs` `Evaluator::eval()`
4. Optionally add a `:command` alias in `aleph_repl.rs` `handle_line()`

### Adding a new REPL command

In `aleph_repl.rs`, add a branch in `handle_line()`:

```rust
if src.starts_with(":mycommand ") {
    let args = src[12..].trim();
    self.my_command(args);
    return;
}
```

Add `fn my_command(&self, args: &str)` to `impl AlephRepl`. Update `:help` text.

### Adding a new Hebrew glyph

The Hebrew font is in `src/vga_font_data.rs` as `HEBREW_FONT: [[u8; 16]; 22]`. Each entry is 16 bytes (one per scanline, MSB = leftmost pixel). The index matches the letter order in `aleph.rs::LETTERS` and maps to code point `0xE0 + index`.

Edit the relevant `[u8; 16]` entry. The font editor at `tools/vga_font_editor.html` generates these byte arrays.

### Adding a new ATA disk

Add a second drive to QEMU with `-drive format=raw,file=disk2.img,if=ide,index=2,media=disk`. Set `ata::ATA_DRIVE = 2` before I/O (noting the ATA driver currently supports only one channel; secondary channel support requires adding port constants `0x170–0x177`).

<hr>

## 8. Troubleshooting

### `[FS] ALFS: invalid ALFS magic`

The kernel read the boot disk instead of the data disk. This was a bug fixed in the current codebase (drive selection now happens before the superblock read). If you see this, rebuild from the latest source.

### `[FS] ALFS: failed to read superblock`

The `alfs.img` was not passed to QEMU or the ATA driver failed to poll ready. Ensure `run.sh` is used (not a manual QEMU invocation missing the `-drive ... alfs.img` argument).

### `[ERROR] Filesystem not mounted`

ALFS failed to mount (check boot log for reason). Until mounted, `:files`, `:save`, `:load`, `:run` all return this error. Fix the mount issue and reboot.

### Hebrew characters not showing (graphical mode)

Confirm you're running `./run.sh` without `--serial`. The UEFI framebuffer must be active (`[BOOT] UEFI GOP Framebuffer:` line in boot log). In serial mode, Hebrew always displays as ASCII.

### Screen text is very large

Edit `SCALE` in `src/font_renderer.rs`. Set to `1` for native 8×16 (current default). `2` gives 16×32 (very large on 1024×768).

### `poll_ready` timeout / ATA hangs

The ATA PIO driver polls up to 100,000 iterations. Under heavy QEMU load this may timeout. Increase the loop bound in `ata.rs` `poll_ready()` or add a `port_delay()` after drive selection.

### Build fails with `abi_x86_interrupt` error

Ensure you're on nightly: `rustup default nightly`. The `abi_x86_interrupt` feature is nightly-only.

### `mcopy` not found

Install mtools: `sudo apt install mtools`. Without it, `build_bootimage.sh` creates the FAT image but cannot copy files into it (the UEFI bootloader will then fail to find the kernel).

<hr>

> *"The boundary encoding determines the bulk."*
