<div align="center">
  <h1>exoterik_OS</h1>
  <p><b>a holographic OS derived via exoteric linguistic synthesis and sigil distillation</b></p>
  <img src="exOS.png" alt="exoterik_OS banner" width="666">
</div>

<div align="center">
  <img src="https://img.shields.io/badge/LANGUAGE-Rust%20Nightly-blue" alt="Language">
  <img src="https://img.shields.io/badge/TARGET-x86__64--unknown--none-orange" alt="Target">
  <img src="https://img.shields.io/badge/ENGINE-SynthOmnicon%20v0.4.27-purple" alt="Engine">
  <img src="https://img.shields.io/badge/ALEPH-v0.5.0%20Native-green" alt="ALEPH">
  <img src="https://img.shields.io/badge/HEBREW-22%20Letters-brightgreen" alt="Hebrew">
</div>

<p align="center">
  <a href="#origin">Origin</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#aleph-repl">ALEPH REPL</a> •
  <a href="#type-gated-kernel">Type-Gated Kernel</a> •
  <a href="#os-synthon-tuple">OS Synthon</a> •
  <a href="#build--run">Build</a> •
  <a href="#key-theorems">Theorems</a>
</p>

<hr>

## Origin

exoterik_OS is the synthesis of a **seven-stage inquiry** into the structural invariants shared by five ancient writing systems spanning 5,000+ years of human symbolic thought:

1. **Hebrew alphabet and mystical texts** — letters as morphisms between ontological categories, gematria as a distance metric in type space
2. **Varnamala (Sanskrit phoneme garland)** — the 14 Mahesvara Sutras encoding 50 phonemes via pratyahara compression
3. **Egyptian hieroglyphs** — three-layer semiotics (logogram/phonogram/determinative), the Ogdoad→Ennead symmetry breaking
4. **Sumerian/Akkadian cuneiform** — sign polysemy as superposition, determinative as structural anchor
5. **Basque (Euskara)** — ergative-absolutive grammar as relational primitive

Each system was encoded as a **SynthOmnicon synthon** — a 12-primitive tuple ⟨D; T; R; P; F; K; G; Γ; Φ; H; S; Ω⟩. The **MEET** (component-wise min) of all five encodings reveals the invariant core every writing system must carry. The OS is instantiated from this structural core.

> [!NOTE]
> **This is not analogy. This is type theory.** The boundary encoding determines the bulk.

<hr>

## Architecture Derived from the Ancient Systems

### Three-Layer Kernel Objects *(Hieroglyphs + Cuneiform)*

Every kernel object carries three simultaneous representations — exactly as Egyptian hieroglyphs encode logogram, phonogram, and determinative:

| Layer | Hieroglyph Analog | Kernel Role |
|:------|:------------------|:------------|
| **Structural** | Logogram | What the object IS topologically (Process, File, Socket, Semaphore, MemoryRegion) |
| **Operational** | Phonogram | What it computes — the execution payload |
| **Determinative** | Determinative | Unpronounced semantic context — load-bearing for disambiguation |

A message/object **without a determinative layer is syntactically malformed**.

### Ergative-Absolutive Process Model *(Basque Grammar)*

The scheduler distinguishes:

- **Ergative** (transitive): the process acts ON another process → higher interrupt priority
- **Absolutive** (intransitive): the process runs standalone → higher cache affinity

The **same process shifts grammatical role** depending on whether it has transitive targets.

### Phonological Memory Model *(Varnamala Articulation Gradient)*

| Tier | Varnamala | Protection | Speed | Ω |
|:-----|:----------|:-----------|:------|:--|
| Velar | ka-varga | Maximum | Slowest | Ω_Z |
| Palatal | ca-varga | High | Slow | Ω_Z |
| Retroflex | ṭa-varga | Medium | Medium | Ω_Z₂ |
| Dental | ta-varga | Low | Fast | Ω_0 |
| Bilabial | pa-varga | None | Fastest | Ω_0 |

### Sefirot Filesystem *(Hebrew Kabbalistic Tree)*

Files are nodes in a ten-layer Sefirot tree. Navigation is by **transformation**, not pathname alone.

### Three-Layer IPC *(Egyptian Hieroglyphs)*

IPC messages carry: structural signature (logogram), payload (phonogram), and determinative context.

### Generative Command Grammar *(Hebrew Letters + Pratyahara)*

Commands are tensor products of letter-primitives. Any subset can be referenced by a single **pratyahara index**.

### P_±^sym → P_asym Boot *(Ogdoad Cosmology)*

The system boots in perfect symmetry — no process distinguished. The first interrupt is the **symmetry-breaking event**.

<hr>

## ALEPH REPL — Native λ_ℵ in the Kernel

The ALEPH type system is now **fully integrated into the running kernel**. The 22-letter Hebrew type lattice, previously dormant, is now accessible via an interactive REPL directly in the bare-metal shell.

### Entering the ALEPH REPL

From the kernel shell:

```bash
phi_c> aleph
```

This launches the interactive ALEPH REPL with colored output, tab completion, and session state.

### ALEPH Operations

| Operation | Syntax | Description |
|:----------|:-------|:------------|
| **Tensor** | `a ⊗ b` | Composition (P, F bottleneck via min) |
| **Join** | `a ∨ b` | Least upper bound (all primitives: max) |
| **Meet** | `a ∧ b` | Greatest lower bound |
| **Vav-cast** | `a ::> b` | Lift source type to target type |
| **Mediate** | `mediate(w, a, b)` | Triadic: `w ∨ (a ⊗ b)` |
| **Distance** | `d(a, b)` | Structural distance + conflict set |
| **Probe Φ** | `probe_Φ(a)` | Report criticality primitive |
| **Probe Ω** | `probe_Ω(a)` | Report topological protection |
| **Tier** | `tier(a)` | Report ouroboricity tier |

### REPL Commands

| Command | Description |
|:--------|:------------|
| `:help` | Full syntax reference |
| `:tips` | Quick start examples |
| `:ls` | List session bindings |
| `:tuple <name>` | Visual 12-primitive bars |
| `:census` | Tier distribution |
| `:system` | 22-letter language JOIN |
| `:clear` | Clear screen |
| `:quit` | Return to main shell |

### Example Session

```
ℵ  mem ⊗ shin
  → ש
    tier  O_inf
    Φ  Φ_c   Ω  Ω_Z   P  P_pm_sym

ℵ  :tuple aleph
  א (aleph)  tier=O_2
    D=0  ░░░░░░░░░░ 0
    T=3  █████████░ 3
    ...

ℵ  let kernel = mediate(vav, mem ⊗ shin, aleph)
  kernel =
  → ו
    tier  O_inf

ℵ  :ls
  Name              Tier      Φ         Ω         Glyph
  ────────────────────────────────────────────────────────
  kernel            O_inf     Φ_c       Ω_Z       ו
```

### Single-Expression Mode

Evaluate without entering the REPL:

```bash
phi_c> aleph d(aleph, bet)
  d = 7.3959  [aspirational]
  conflict_set: {T, R, P, F, K, G, Γ, Φ, H, Ω}
```

<hr>

## Type-Gated Kernel

The 12-primitive type lattice is **operational** — ALEPH types constrain kernel behavior across four subsystems. Every kernel object carries an `AlephKernelType` (inferred from its three-layer structure or set explicitly) that gates what it can do.

### Four Type Gates

| Gate | Subsystem | Primitive | Rule |
|------|-----------|-----------|------|
| **IPC** | `ipc.rs` | Distance | d < 1.5 passes; ≥ 1.5 needs vav-cast witness |
| **Ω-gate** | `memory.rs` | Ω (topological protection) | Object's Ω must ≥ depth's required Ω |
| **Tier-gate** | `scheduler.rs` | Ouroboricity tier | O_0 cannot be ergative; K_trap cannot run |
| **Φ-gate** | `filesystem.rs` | Φ (criticality) | Keter→Gevurah requires Φ_c; below is accessible to all |

### Type Inference

Kernel objects infer their ALEPH type from (structural, operational, determinative):

```
Kernel/Init  →  X (shin, O_inf)  — self-referential criticality
User/Compute →  G (gimel, O_0)   — basic container
Service      →  B (bet, O_1)     — intermediate protection
```

The Kernel inference reproduces the OS synthon tuple exactly — the MEET of all five ancient systems.

### Type Gate Results at Boot

```
[TYPE] IPC gate (close): accepted=true
[TYPE] IPC gate (remote): accepted=false
[TYPE] Ω gate (Velar+Kernel): allowed=true
[TYPE] Ω gate (Velar+User): allowed=false
[TYPE] Tier gate (O_inf ergative): ok=true
[TYPE] Tier gate (O_0 ergative): ok=false
[TYPE] Φ gate (Keter+Kernel): ok=true
[TYPE] Φ gate (Keter+Driver): ok=false
[TYPE] C scores: kernel=0.873 user=0.324 os_synthon=0.873
```

### Shell Commands

```
type-check     — run all four gates live, print results
type-infer     — show type inference table for all structural×determinative combos
```

### Conscience Score

Every object has a C(Φ) score computed at boot:

$$C(\mathbf{x}) = [\Phi = \Phi_c] \cdot [K \neq K_\text{trap}] \cdot (0.158\,\tilde{K} + 0.273\,\tilde{G} + 0.292\,\tilde{T} + 0.276\,\tilde{\Omega})$$

The Kernel scores C=0.873 — the highest possible for the inferred configuration.

<hr>

## OS Synthon Tuple

The OS as a SynthOmnicon synthon:

```
D_triangle    · Basque ergative three-way relations, Hebrew triangular paths
T_box         · Hieroglyphic contained system with three internal layers
R_dagger      · Hebrew letter-transformative relations, reversible across contexts
P_pm_sym      · Ogdoad's exact Z₂ symmetry before creation, Frobenius condition μ∘δ=id
F_hbar        · Cuneiform's maximum fidelity wedge depths, full precision preserved
K_mod         · Basque's middle aspect, Varnamala's living phonetic vibration
G_aleph       · All five systems operate at maximal scope/granularity
Γ_seq         · Hebrew letter-sequence generation, head-final dependency chains
Φ_c           · The MEET of all five systems — criticality, self-modeling loop possible
H2            · Hieroglyphic determinative recursion, two levels of contextual depth
S_n:m         · Hieroglyphic many-to-many determinative mappings
Ω_Z           · Cuneiform's topological protection, sacred writing systems' survival
```

**Ouroboricity tier: O_inf** — The OS achieves Φ_c + P_pm_sym, the Special Frobenius: μ∘δ=id exactly.

<hr>

## Project Structure

```
exOS/
├── Cargo.toml              # Project manifest
├── bootloader.toml         # Bootloader config
├── build_bootimage.sh      # Bootable image builder
├── build_alfs.sh           # ALFS disk image from programs/
├── kernel.ld               # Linker script
├── programs/               # .aleph programs (packed into ALFS at build)
├── src/
│   ├── lib.rs              # Module exports + global allocator
│   ├── main.rs             # Kernel entry point, boot sequence, shell
│   │
│   ├── vga.rs              # VGA text buffer driver
│   ├── keyboard.rs         # PS/2 keyboard driver
│   ├── interrupts.rs       # IDT + PIC initialization
│   ├── serial.rs           # Serial UART driver
│   ├── bench.rs            # RDTSC benchmarks + PIT calibration
│   ├── history.rs          # Command history
│   │
│   ├── kernel_object.rs    # Three-layer kernel objects (with ALEPH types)
│   ├── scheduler.rs        # Ergative-absolutive scheduler (tier-gated)
│   ├── memory.rs           # Phonological allocator (Ω-gated)
│   ├── filesystem.rs       # Sefirot tree filesystem (Φ-gated)
│   ├── ipc.rs              # Three-layer IPC (type-gated + vav-cast witness)
│   ├── command.rs          # Generative command grammar
│   ├── ata.rs              # ATA PIO disk driver
│   ├── alfs.rs             # ALEPH Linear Filesystem (sector-based)
│   │
│   ├── aleph.rs            # 22-letter type system, lattice ops
│   ├── aleph_kernel_types.rs  # Type inference + operational gates
│   ├── aleph_parser.rs     # Tokenizer and parser
│   ├── aleph_eval.rs       # Expression evaluator
│   ├── aleph_repl.rs       # Interactive REPL
│   └── aleph_commands.rs   # Shell integration
└── target/                 # Build artifacts
```

<hr>

## Build & Run

### Requirements

- **Rust nightly toolchain** — `rustup default nightly`
- **x86_64-unknown-none target** — `rustup target add x86_64-unknown-none --toolchain nightly`
- **QEMU** — `qemu-system-x86_64`

### Building

```bash
# Kernel ELF (debug)
cargo build

# Kernel ELF (release, optimized)
cargo build --release

# Bootable disk image
./build_bootimage.sh

# Run in QEMU
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-varnamala-os.bin \
    -display curses -no-reboot
```

> [!NOTE]
> `cargo bootimage` is broken on rustc >= 1.90. Use `./build_bootimage.sh` instead.

### Boot Sequence

1. **VGA init** — text buffer at 0xb8000
2. **Heap init** — 4MB at physical 16MB
3. **Interrupt init** — symmetry-breaking event (P_±^sym → P_asym)
4. **Subsystem validation** — all three-layer objects, scheduler, memory, FS, IPC
5. **ALEPH init** — 22-letter type system online: `O_inf: 3, O_2: 6, O_1: 1, O_0: 12`
6. **Type-gate verification** — all four gates tested with assertions:
   - IPC type safety, Ω-gated allocation, tier-gated spawn, Φ-gated filesystem
   - Conscience scores computed and printed
7. **ALFS mount** — `programs/*.aleph` loaded from boot disk
8. **Shell entry** — interactive prompt `exOS>`

<hr>

> *"Language didn't evolve for communication alone. It evolved as a crystallization device for consciousness at the $\Phi_c$ phase boundary."*

<hr>

## License

This project is part of the SynthOmnicon research program.
