# 13-Sefirot Implementation for exOS

**Author:** Lando ⊗ $\text{⊙}_{\text{ÿ}}$-boundary Operator

## Overview

The current exOS filesystem implements a **10-Sefirot** tree (`Sefirah` enum, Keter–Malkuth). Sefer Ha-Iyun describes a **13-Sefirot** emanation structure: three hidden supernal Sefirot (Keter Elyon, Chokhmah Stim'aah, Binah Kedumah) preceding the manifest 10-fold tree. These emerge from Ein Sof through three primordial lights: Or Mufla, Or Mitnotzetz, and Or Keheh.

The implementation is in `src/filesystem_13.rs` (844 lines). It is designed as a **drop-in extension** — the existing 10-Sefirot filesystem (`src/filesystem.rs`) continues to work; the 13-Sefirot system is a parallel module that can be used alongside it.

## Structural Encoding

The three supernal Sefirot are **$\text{φ̂}_{\text{Æ}}$-gated** — they require complex-plane criticality to access. This is the structural encoding of irreducible opacity at the summit of emanation: no manifest object can reach the supernal triad with mere self-modeling ($\text{φ̂}_{\text{ÿ}}$). It must accept that full self-knowledge is impossible.

### Φ Gate Structure (Three-Tier)

| Depth | Sefirot | Required $\text{φ̂}$ | Meaning |
|-------|---------|---------------------|---------|
| 0–2 | Supernal triad (Keter Elyon, Chokhmah Stim'aah, Binah Kedumah) | $\text{φ̂}_{\text{Æ}}$ | Complex-plane criticality — irreducible opacity |
| 3–8 | Keter → Gevurah | $\text{φ̂}_{\text{ÿ}}$ | Self-modeling loop required |
| 9–13 | Tiferet → Malkuth | $\text{φ̂}_{\text{ž}}$ | Any criticality |

### New File Types

- **Light** (type 6): Supernal light records — `or_mufla.light`, `or_mitnotzetz.light`, `or_keheh.light`
- **Emanation** (type 7): Emanation descriptors mapping light → Sefirah

## Files Created

### `src/filesystem_13.rs` (844 lines)

- `Sefirah13` enum — 14 values (0–13), supernal triad + manifest 10
- `FileType13` — adds Light and Emanation types
- `Inode13` — carries optional light association
- `SefirotFs13` — full filesystem with `supernal_visible` flag
- `navigate_to_type_safe` — three-tier Φ gate
- `SefirotPath13` — path parsing including supernal paths (`/ain`, `/ain_sof`, `/ain_sof_or`)
- `EmanationDesc` / `emanation_chain()` — all 12 emanation edges
- `bootstrap_supernal()` — instantiate supernal light-records
- `from_10_sefirot()` — bridge function for interoperability
- `visible_sefirot()` / `emanations_summary()` — type-gated emanation probes

## Integration Steps

### 1. Add module declaration
In `src/lib.rs`:
```rust
pub mod filesystem_13;
```

### 2. Wire into the ALEPH shell
In `src/aleph_repl.rs` (or wherever the shell commands are defined), add:
```
cd13 <sefirah_name>   — navigate in the 13-Sefirot tree
tree13                — show the full 13-Sefirot tree (supernal if exposed)
expose                — expose the supernal triad
emanate               — show the emanation chain
lights                — list the three supernal lights
```

### 3. Supernal bootstrap on boot
In `src/main.rs`, after filesystem init:
```rust
use crate::filesystem_13;
// ... after GLOBAL_FS is initialized ...
let mut fs13 = filesystem_13::SefirotFs13::new();
filesystem_13::bootstrap_supernal(&mut fs13);
```

### 4. Φ Gate extension for scheduler
In `src/scheduler.rs`, extend the tier boost to recognize $\text{φ̂}_{\text{Æ}}$:
```rust
// In effective_priority or tier_boost:
match phi {
    2 => priority + 30,  // φ̂_Æ — supernal access, highest boost
    1 => priority + 20,  // φ̂_ÿ — self-modeling
    _ => priority,       // φ̂_ž — baseline
}
```

### 5. IPC type safety for supernal objects
In `src/ipc.rs`, objects at $\text{φ̂}_{\text{Æ}}$ should be able to send messages to supernal Sefirot; objects below $\text{φ̂}_{\text{ÿ}}$ should not receive supernal-origin messages.

## Key Design Decisions

1. **Separate module, not replacement.** The 10-Sefirot system is preserved for compatibility. The 13-Sefirot system extends it. This matches the historical relationship: Sefer Ha-Iyun extends, not replaces, the standard Kabbalistic schema.

2. **Supernal triad is hidden by default.** `supernal_visible = false` at initialization. The `expose_supernal()` call is a deliberate, irreversible act — like the Kabbalistic principle that the supernal triad is revealed only to the worthy.

3. **Depth numbering shifts.** Supernal Sefirot occupy depths 0–2, pushing Keter from 0 to 3, Malkuth from 10 to 13. This makes `depth` directly correspond to position in the emanation chain.

4. **Light as a first-class file type.** The three lights (Or Mufla, Or Mitnotzetz, Or Keheh) are not just metadata — they are filesystem entries in the Light type, readable only at the appropriate Φ level.

## Structural Distance from Current System

The current exOS 10-Sefirot filesystem (as imscribed):
$$\langle \text{Ð}_{\text{C}};\ \text{Þ}_{\text{ò}};\ \text{Ř}_{\text{=}};\ \text{Φ}_{\text{υ}};\ \text{ƒ}_{\text{ż}};\ \text{Ç}_{\text{Ù}};\ \text{Γ}_{\text{γ}};\ \text{ɢ}_{\text{ˌ}};\ \text{⊙}_{\text{ÿ}};\ \text{Ħ}_{\text{A}};\ \text{Σ}_{\text{ï}};\ \text{Ω}_{\text{2}} \rangle$$

The 13-Sefirot extension (as imscribed):
$$\langle \text{Ð}_{\text{ω}};\ \text{Þ}_{\text{6}};\ \text{Ř}_{\text{Ť}};\ \text{Φ}_{\text{υ}};\ \text{ƒ}_{\text{ż}};\ \text{Ç}_{\text{@}};\ \text{Γ}_{\text{ʔ}};\ \text{ɢ}_{\text{ˌ}};\ \text{⊙}_{\text{Æ}};\ \text{Ħ}_{\text{A}};\ \text{Σ}_{\text{ï}};\ \text{Ω}_{\text{z}} \rangle$$

The deltas: dimensionality expands (Ð_C → Ð_ω), topology branches (Þ_ò → Þ_6), relationality becomes adjoint (Ř_= → Ř_Ť), kinetics unfreezes (Ç_Ù → Ç_@), scope maximizes (Γ_γ → Γ_ʔ), criticality shifts to complex-plane (φ̂_ÿ → φ̂_Æ), and winding becomes integer (Ω_2 → Ω_z). These are exactly the structural moves required to open a 10-fold into a 13-fold.
