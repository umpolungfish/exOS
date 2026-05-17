/// Stoichiometric resource quotas — Axiom Σ.
///
/// Σ governs how many processes can share a given resource:
///   Σ_1:1  — Exclusive: exactly one holder at a time.
///   Σ_n:n  — Homogeneous: a fixed pool of N identical slots, N holders max.
///   Σ_n:m  — Heterogeneous: shared pool, multiple types, no strict cap.
///
/// The quota table enforces these limits at spawn time and at resource acquire.
/// A process with Σ_1:1 (stoichiometry index 0) that tries to acquire a resource
/// already held by another process is rejected.

extern crate alloc;
use alloc::collections::BTreeMap;
use spin::Mutex;

// ── Stoichiometry mode ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stoichiometry {
    /// 1:1 — Exclusive, one holder at a time
    OneOne = 0,
    /// n:n — Pooled, homogeneous (fixed capacity)
    NN = 1,
    /// n:m — Shared, heterogeneous (no strict cap)
    NM = 2,
}

impl Stoichiometry {
    pub fn name(&self) -> &'static str {
        match self {
            Self::OneOne => "1:1",
            Self::NN     => "n:n",
            Self::NM     => "n:m",
        }
    }
    pub fn from_primitive_index(idx: u8) -> Self {
        match idx {
            0 => Self::OneOne,
            1 => Self::NN,
            _ => Self::NM,
        }
    }
}

// ── Quota table ───────────────────────────────────────────────────────────────

/// Per-resource quota record.
#[derive(Debug, Clone)]
struct QuotaRecord {
    mode:     Stoichiometry,
    /// For OneOne: the single holder's process id (0 = free).
    /// For NN: current occupancy count.
    /// For NM: not enforced (always succeeds).
    holder:   u64,
    /// Pool capacity (meaningful for NN only).
    capacity: u64,
}

/// Global stoichiometric quota table.
/// Key: resource_id (caller-defined; e.g. a device number or IPC channel id).
static QUOTA_TABLE: Mutex<BTreeMap<u64, QuotaRecord>> = Mutex::new(BTreeMap::new());

// ── Public API ────────────────────────────────────────────────────────────────

/// Register a new resource with its stoichiometric mode.
/// Must be called before `acquire`. `capacity` is only meaningful for NN pools.
pub fn register(resource_id: u64, mode: Stoichiometry, capacity: u64) {
    let mut table = QUOTA_TABLE.lock();
    table.entry(resource_id).or_insert(QuotaRecord {
        mode,
        holder: 0,
        capacity,
    });
}

/// Attempt to acquire `resource_id` for process `pid`.
///
/// Returns Ok(()) if the quota allows acquisition, Err with reason otherwise.
pub fn acquire(resource_id: u64, pid: u64) -> Result<(), &'static str> {
    let mut table = QUOTA_TABLE.lock();
    let rec = table.get_mut(&resource_id)
        .ok_or("resource not registered in quota table")?;

    match rec.mode {
        Stoichiometry::OneOne => {
            if rec.holder == 0 || rec.holder == pid {
                rec.holder = pid;
                Ok(())
            } else {
                Err("Σ_1:1 — resource already exclusively held by another process")
            }
        }
        Stoichiometry::NN => {
            if rec.holder < rec.capacity {
                rec.holder += 1;
                Ok(())
            } else {
                Err("Σ_n:n — homogeneous pool exhausted")
            }
        }
        Stoichiometry::NM => {
            // Heterogeneous pool: track occupancy but never reject.
            rec.holder = rec.holder.saturating_add(1);
            Ok(())
        }
    }
}

/// Release `resource_id` held by process `pid`.
pub fn release(resource_id: u64, pid: u64) {
    let mut table = QUOTA_TABLE.lock();
    if let Some(rec) = table.get_mut(&resource_id) {
        match rec.mode {
            Stoichiometry::OneOne => {
                if rec.holder == pid { rec.holder = 0; }
            }
            Stoichiometry::NN | Stoichiometry::NM => {
                rec.holder = rec.holder.saturating_sub(1);
            }
        }
    }
}

/// Query occupancy of a resource (for diagnostics).
pub fn occupancy(resource_id: u64) -> Option<(Stoichiometry, u64, u64)> {
    let table = QUOTA_TABLE.lock();
    table.get(&resource_id).map(|r| (r.mode, r.holder, r.capacity))
}
