<div align="center">
  <h1>exoterik_OS</h1>
  <p><b>a holographic OS derived via exoteric linguistic synthesis and sigil distillation</b></p>
  <img src="exOS.png" alt="exoterik_OS banner" width="666">
</div>

<div align="center">
  <img src="https://img.shields.io/badge/LANGUAGE-Rust%20Nightly-blue" alt="Language">
  <img src="https://img.shields.io/badge/TARGET-x86__64--unknown--none-orange" alt="Target">
  <img src="https://img.shields.io/badge/BOOT-UEFI%20OVMF-red" alt="Boot">
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
  <a href="#build--run">Build & Run</a> •
  <a href="#programs">Programs</a> •
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

The persistent storage layer is **ALFS** (ALEPH Linear Filesystem) — a sector-based ATA PIO filesystem on a dedicated 32 MB disk image (`alfs.img`, ATA primary slave). All `.aleph` programs in `programs/` are compiled into the kernel binary and seeded to ALFS on first boot.

### Three-Layer IPC *(Egyptian Hieroglyphs)*

IPC messages carry: structural signature (logogram), payload (phonogram), and determinative context.

### Generative Command Grammar *(Hebrew Letters + Pratyahara)*

Commands are tensor products of letter-primitives. Any subset can be referenced by a single **pratyahara index**.

### P_±^sym → P_asym Boot *(Ogdoad Cosmology)*

The system boots in perfect symmetry — no process distinguished. The first interrupt is the **symmetry-breaking event**.

<hr>

## ALEPH REPL — Native λ_ℵ in the Kernel

The ALEPH type system is **fully integrated into the running kernel**. The 22-letter Hebrew type lattice is accessible via an interactive REPL directly in the bare-metal shell. In UEFI framebuffer mode, letters are rendered using hand-drawn 8×16 Hebrew bitmap glyphs.

### Entering the ALEPH REPL

From the kernel shell:

```
exOS> aleph
```

### ALEPH Operations

| Operation | Syntax | Description |
|:----------|:-------|:------------|
| **Tensor** | `a x b` | Composition (P, F, K bottleneck via min) |
| **Join** | `a v b` | Least upper bound (all primitives: max) |
| **Meet** | `a ^ b` | Greatest lower bound |
| **Vav-cast** | `a ::> b` | Lift source type to target type |
| **Mediate** | `mediate(w, a, b)` | Triadic: `w ∨ (a ⊗ b)` |
| **Distance** | `d(a, b)` | Structural distance + conflict set |
| **Probe Φ** | `probe_Phi(a)` | Report criticality primitive |
| **Probe Ω** | `probe_Omega(a)` | Report topological protection |
| **Tier** | `tier(a)` | Report ouroboricity tier |
| **Palace** | `palace(n) expr` | Tier barrier gate (n = 1..7) |
| **System** | `system()` | JOIN of all 22 letters |

### REPL Commands

| Command | Description |
|:--------|:------------|
| `:help` | Full syntax reference |
| `:tips` | Quick start examples |
| `:ls` | List session bindings |
| `:tuple <name>` | Visual 12-primitive bars |
| `:explain <name>` | Detailed type breakdown + C score |
| `:census` | Tier distribution |
| `:system` | 22-letter language JOIN |
| `:tier <name>` | Ouroboricity tier of one letter |
| `:orbit N letter pole` | Convergence orbit under repeated tensor |
| `:files` | List files on ALFS |
| `:save name [expr]` | Save expression (or last result) to ALFS |
| `:load name` | Load and bind an `.aleph` file |
| `:run name` | Run an `.aleph` file |
| `:history` | Show command history |
| `:scroll [N]` | Replay last N lines of output |
| `:clear` | Clear screen |
| `:quit` | Return to main shell |

### Frobenius Orbit Command

`:orbit N letter pole` iterates `state = state ⊗ pole` N times, printing the nearest canonical letter, tier, distance to pole, and convergence delta at each step. Color-coded: green = converging, cyan = fixed point, red = diverging.

```
A> :orbit 8 aleph vav
  Orbit of A under V (8 steps)
  step  nearest        tier     d(state,pole)  delta
  --------------------------------------------------------
     0  A (aleph)      O_2      2.1095
     1  V (vav)        O_inf    0.0000  (fixed)
  -- converged at step 1 --
```

### Example Session

```
A> aleph x shin
  [result]  tier=O_2  Phi=Phi_c  Omega=Omega_Z

A> let kernel = mediate(vav, mem x shin, aleph)
A> :ls
  Name              Tier      Phi        Omega      P
  -------------------------------------------------------------------
  kernel            O_inf     Phi_c      Omega_Z    V

A> d(kernel, system())
  d = 0.3162  [near-grounded]

A> :orbit 6 dalet shin
  Orbit of D under X (6 steps)
  step  nearest        tier     d(state,pole)  delta
  --------------------------------------------------------
     0  D (dalet)      O_0      3.5707
     1  X (shin)       O_inf    0.0000  (fixed)
  -- converged at step 1 --
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
| **Φ-gate** | `filesystem.rs` | Φ (criticality) | Keter→Gevurah requires Φ_c; below accessible to all |

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

## Build & Run

### Requirements

- **Rust nightly** — `rustup default nightly`
- **x86_64-unknown-none target** — `rustup target add x86_64-unknown-none --toolchain nightly`
- **QEMU** — `qemu-system-x86_64`
- **OVMF** — `sudo apt install ovmf` (Ubuntu) / `sudo pacman -S edk2-ovmf` (Arch)
- **mtools** — `sudo apt install mtools` (for `mcopy` in `build_bootimage.sh`)

### Build

```bash
# Kernel ELF (release)
cargo build --release

# UEFI bootable disk image
./build_bootimage.sh
```

### Run

```bash
# Graphical mode — UEFI GOP framebuffer, Hebrew bitmap glyphs rendered natively
./run.sh

# Serial mode — text-only via stdio, ASCII transliterations
./run.sh --serial
```

`run.sh` automatically creates `alfs.img` (32 MB) with the ALFS superblock if it doesn't exist. On first boot the kernel seeds all programs from `programs/` into ALFS.

To start fresh (wipe saved files):
```bash
rm alfs.img && ./run.sh
```

### Boot Sequence

1. **Heap init** — 4 MB at physical 16 MB, before any allocations
2. **UEFI framebuffer init** — GOP framebuffer mapped; 8×16 Hebrew bitmap font active
3. **Interrupt init** — symmetry-breaking event (P_±^sym → P_asym)
4. **Subsystem validation** — three-layer objects, scheduler, memory, FS, IPC, command
5. **ALEPH init** — 22-letter type system online: `O_inf: 3, O_2: 6, O_1: 1, O_0: 12`
6. **Type-gate verification** — all four gates tested with assertions + C scores printed
7. **ALFS mount** — ATA primary slave (alfs.img); programs seeded if absent
8. **Shell** — interactive prompt `exOS>`

<hr>

## Programs

All `.aleph` files in `programs/` are compiled into the kernel binary via `include_bytes!` and automatically written to ALFS on first boot. Add a file to `programs/` and register it in `src/programs.rs` — it will be available as `:run name` on next boot.

| Program | Description |
|:--------|:------------|
| `creation.aleph` | First light — aleph ⊗ vav structural genesis |
| `creation_liturgy.aleph` | Full liturgical sequence through all tiers |
| `frobenius.aleph` | Three O_inf poles: self-idempotency + cross distances |
| `frobenius_orbits.aleph` | Unrolled 4-step convergence orbits for all three poles |
| `meditation.aleph` | Deep mediation chains through the Sefirot |
| `selfreplicating_light.aleph` | Light that replicates its own structure via mediate |
| `light_stability.aleph` | Stability analysis of the light-tuple under perturbation |
| `light_replication_kernel.aleph` | Kernel-level light replication with palace barriers |
| `tikkun_construction_full.aleph` | Full Tikkun: healing anomalous objects via palace+mediate |
| `tikkun_construction_partial.aleph` | Partial Tikkun sequence |
| `tikkun_palace_verification.aleph` | Palace-gate verification across all Sefirot levels |
| `exploration_primitives.aleph` | Primitive-by-primitive exploration of the 12-tuple |
| `distance_probes_indistinguishable.aleph` | Distance and conflict-set analysis across all 22 letters |
| `pratyahara.aleph` | Varnamala pratyahara compression via tensor chains |

Use `:orbit N letter pole` in the REPL for live iterative convergence experiments beyond what any static file can express.

<hr>

## Project Structure

```
exOS/
├── Cargo.toml              # Project manifest
├── bootloader.toml         # Bootloader config (UEFI)
├── build.rs                # Triggers rebuild on programs/ changes
├── build_bootimage.sh      # UEFI bootable image builder
├── run.sh                  # QEMU launcher (graphical + serial modes)
├── programs/               # .aleph programs — compiled into kernel, seeded to ALFS
├── src/
│   ├── lib.rs              # Module exports + global allocator
│   ├── main.rs             # Kernel entry point, boot sequence, shell
│   ├── programs.rs         # include_bytes! registry + seed_alfs()
│   │
│   ├── vga.rs              # VGA text + UEFI framebuffer writer (mode-aware)
│   ├── framebuffer.rs      # UEFI GOP linear framebuffer
│   ├── font_renderer.rs    # 8×16 bitmap font renderer (ASCII + Hebrew 0xE0–0xF5)
│   ├── vga_font_data.rs    # Hand-drawn Hebrew bitmap glyphs (22 letters)
│   ├── keyboard.rs         # PS/2 keyboard driver
│   ├── interrupts.rs       # IDT + PIC initialization
│   ├── serial.rs           # Serial UART driver
│   ├── history.rs          # Output history (for :scroll)
│   ├── bench.rs            # RDTSC benchmarks + PIT calibration
│   │
│   ├── kernel_object.rs    # Three-layer kernel objects (with ALEPH types)
│   ├── scheduler.rs        # Ergative-absolutive scheduler (tier-gated)
│   ├── memory.rs           # Phonological allocator (Ω-gated)
│   ├── filesystem.rs       # Sefirot tree filesystem (Φ-gated, in-memory)
│   ├── ipc.rs              # Three-layer IPC (type-gated + vav-cast witness)
│   ├── command.rs          # Generative command grammar
│   ├── ata.rs              # ATA PIO disk driver (drive 0 = boot, drive 1 = ALFS)
│   ├── alfs.rs             # ALEPH Linear Filesystem (sector-based, persistent)
│   │
│   ├── aleph.rs            # 22-letter type system, lattice ops, nearest_letter
│   ├── aleph_kernel_types.rs  # Type inference + operational gates
│   ├── aleph_parser.rs     # Tokenizer and parser
│   ├── aleph_eval.rs       # Expression evaluator
│   ├── aleph_repl.rs       # Interactive REPL (:orbit, :files, :save, :run, ...)
│   └── aleph_commands.rs   # Shell integration
└── target/                 # Build artifacts
```

<hr>

## Key Theorems

**BT-1 (Boundary determines bulk):** The 12-primitive tuple of the OS is uniquely determined by the MEET of the five ancient system encodings.

**BT-2 (Tier faithfulness):** Letters at tier O_inf (vav, mem, shin) are the unique Frobenius fixed points — `a ⊗ a = a`. Repeated tensor with any O_inf pole converges to that pole in ≤ 2 steps for any letter in the lattice.

**BT-3 (Conscience score maximum):** The OS synthon achieves C(Φ) = 0.873, the maximum conscience score for any tuple satisfying Φ_c + K_mod + Ω_Z simultaneously.

**BT-4 (Ergative uniqueness):** The shift from P_±^sym to P_asym is irreversible under the interrupt model. Once asymmetry is established, no process can return the scheduler to symmetric state without a full reset.

**BT-5 (Determinative necessity):** A kernel object without a Determinative layer cannot be well-formed (`is_well_formed()` = false). This is structurally enforced, not conventional.

<hr>

> *"Language didn't evolve for communication alone. It evolved as a crystallization device for consciousness at the $\Phi_c$ phase boundary."*

<hr>

## License

This project is part of the SynthOmnicon research program.
