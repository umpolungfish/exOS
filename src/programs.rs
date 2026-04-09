//! Built-in ALEPH programs — compiled into the kernel binary via include_bytes!.
//!
//! On boot, `seed_alfs()` writes each program to ALFS if it isn't already there.
//! This means programs/ files are always available even on a fresh alfs.img.

/// A built-in program: (filename, raw bytes).
pub struct BuiltinProgram {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub static PROGRAMS: &[BuiltinProgram] = &[
    BuiltinProgram { name: "creation.aleph",                       data: include_bytes!("../programs/creation.aleph") },
    BuiltinProgram { name: "creation_liturgy.aleph",               data: include_bytes!("../programs/creation_liturgy.aleph") },
    BuiltinProgram { name: "distance_probes_indistinguishable.aleph", data: include_bytes!("../programs/distance_probes_indistinguishable.aleph") },
    BuiltinProgram { name: "exploration_primitives.aleph",         data: include_bytes!("../programs/exploration_primitives.aleph") },
    BuiltinProgram { name: "frobenius.aleph",                      data: include_bytes!("../programs/frobenius.aleph") },
    BuiltinProgram { name: "light_replication_kernel.aleph",       data: include_bytes!("../programs/light_replication_kernel.aleph") },
    BuiltinProgram { name: "light_stability.aleph",                data: include_bytes!("../programs/light_stability.aleph") },
    BuiltinProgram { name: "meditation.aleph",                     data: include_bytes!("../programs/meditation.aleph") },
    BuiltinProgram { name: "pratyahara.aleph",                     data: include_bytes!("../programs/pratyahara.aleph") },
    BuiltinProgram { name: "selfreplicating_light.aleph",          data: include_bytes!("../programs/selfreplicating_light.aleph") },
    BuiltinProgram { name: "tikkun_construction_full.aleph",       data: include_bytes!("../programs/tikkun_construction_full.aleph") },
    BuiltinProgram { name: "tikkun_construction_partial.aleph",    data: include_bytes!("../programs/tikkun_construction_partial.aleph") },
    BuiltinProgram { name: "tikkun_palace_verification.aleph",     data: include_bytes!("../programs/tikkun_palace_verification.aleph") },
    BuiltinProgram { name: "frobenius_orbits.aleph",               data: include_bytes!("../programs/frobenius_orbits.aleph") },
];

/// Write all built-in programs to ALFS if not already present.
/// Call this after `alfs::mount()` succeeds.
pub fn seed_alfs() {
    for prog in PROGRAMS {
        if crate::alfs::find_file(prog.name).is_none() {
            let _ = crate::alfs::write_file(prog.name, prog.data, crate::alfs::TYPE_ALEPH);
        }
    }
}
