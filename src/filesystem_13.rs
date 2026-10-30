//! 13-Sefirot filesystem -- extended from Sefer Ha-Iyun.
//!
//! The standard 10-Sefirot tree (Keter -> Malkuth) is extended with three
//! hidden supernal Sefirot that precede the manifest tree. These are the
//! three "lights" of Sefer Ha-Iyun:
//!
//!   Or Mufla (Wondrous Light)       -> Keter Elyon (Supernal Crown)
//!   Or Mitnotzetz (Sparkling Light)  -> Chokhmah Stim'aah (Hidden Wisdom)
//!   Or Keheh (Dim Light)             -> Binah Kedumah (Primordial Understanding)
//!
//! These are NOT the manifest Keter/Chokhmah/Binah -- they are their hidden
//! supernal archetypes. The three supernal Sefirot are φ̂_Æ-gated (complex-plane
//! criticality), encoding the irreducible opacity at the summit of emanation.
//!
//! The full 13-Sefirot tree (from Ein Sof to Malkuth):
//!
//! | Depth | # | Sefirah              | FS Path        | Φ Gate  | Light               |
//! |-------|---|----------------------|----------------|---------|---------------------|
//! | 0     | -- | (Ein Sof)            | --              | --       | Infinite source      |
//! | 1     | 1 | Keter Elyon          | /ain           | φ̂_Æ     | Or Mufla (Wondrous)  |
//! | 2     | 2 | Chokhmah Stim'aah    | /ain_sof       | φ̂_Æ     | Or Mitnotzetz (Spark)|
//! | 3     | 3 | Binah Kedumah        | /ain_sof_or    | φ̂_Æ     | Or Keheh (Dim)       |
//! | 4     | 4 | Keter                | /boot          | φ̂_ÿ     | -- (manifest)         |
//! | 5     | 5 | Chokhmah             | /sys/bin       | φ̂_ÿ     |                      |
//! | 6     | 6 | Binah                | /sys/lib       | φ̂_ÿ     |                      |
//! | 7     | 7 | Da'at                | /dev           | φ̂_ÿ     |                      |
//! | 8     | 8 | Chesed               | /home          | φ̂_ÿ     |                      |
//! | 9     | 9 | Gevurah              | /etc/permissions| φ̂_ÿ    |                      |
//! | 10    |10 | Tiferet              | /ipc           | φ̂_ž     |                      |
//! | 11    |11 | Netzach              | /net/mount     | φ̂_ž     |                      |
//! | 12    |12 | Hod                  | /var/log       | φ̂_ž     |                      |
//! | 13    |13 | Yesod                | /tmp           | φ̂_ž     |                      |
//! | 14    |14 | Malkuth              | /data          | φ̂_ž     |                      |
//!
//! Navigation is not by pathname alone but by transformation chain.
//! The supernal triad is accessible only to objects with φ̂_Æ (complex-plane
//! criticality) -- the summit retains irreducible opacity. This is the
//! structural encoding of the Kabbalistic principle that the highest
//! Sefirot are "hidden" and not fully knowable by any manifest being.
//!
//! Architecture:
//!   ALFS provides raw sector storage.
//!   SefirotFs13 layers a VFS on top: inodes, directories, paths.
//!   Each file lives in exactly one Sefirah level.
//!   The SefirotPath encodes the full transformation chain.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::string::ToString;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

use crate::alfs;
use crate::kernel_object::KernelObject;

// ── 13 Sefirot levels ──────────────────────────────────────────────────────
// The three supernal Sefirot (KeterElyon, ChokhmahStimaah, BinahKedumah) are
// numbered 0-2 so that depth 0 = Ein Sof proximity. This shifts the manifest
// tree: Keter=3, Chokhmah=4, ..., Malkuth=13.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Sefirah13 {
    // ── Supernal triad (hidden, φ̂_Æ-gated) ──
    KeterElyon = 0,
    ChokhmahStimaah = 1,
    BinahKedumah = 2,
    // ── Manifest supernal triad (φ̂_ÿ-gated) ──
    Keter = 3,
    Chokhmah = 4,
    Binah = 5,
    Daat = 6,
    // ── Middle pillars (φ̂_ÿ-gated) ──
    Chesed = 7,
    Gevurah = 8,
    // ── Manifest layers (φ̂_ž-gated) ──
    Tiferet = 9,
    Netzach = 10,
    Hod = 11,
    Yesod = 12,
    Malkuth = 13,
}

impl Sefirah13 {
    pub fn name(&self) -> &'static str {
        match self {
            Self::KeterElyon => "Keter Elyon",
            Self::ChokhmahStimaah => "Chokhmah Stim'aah",
            Self::BinahKedumah => "Binah Kedumah",
            Self::Keter => "Keter",
            Self::Chokhmah => "Chokhmah",
            Self::Binah => "Binah",
            Self::Daat => "Daat",
            Self::Chesed => "Chesed",
            Self::Gevurah => "Gevurah",
            Self::Tiferet => "Tiferet",
            Self::Netzach => "Netzach",
            Self::Hod => "Hod",
            Self::Yesod => "Yesod",
            Self::Malkuth => "Malkuth",
        }
    }

    /// Short display name for the ALEPH shell
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::KeterElyon => "KtrE",
            Self::ChokhmahStimaah => "ChkS",
            Self::BinahKedumah => "BinK",
            Self::Keter => "Ktr",
            Self::Chokhmah => "Chk",
            Self::Binah => "Bin",
            Self::Daat => "Dat",
            Self::Chesed => "Chs",
            Self::Gevurah => "Gev",
            Self::Tiferet => "Tif",
            Self::Netzach => "Net",
            Self::Hod => "Hod",
            Self::Yesod => "Yes",
            Self::Malkuth => "Mal",
        }
    }

    pub fn default_path(&self) -> &'static str {
        match self {
            Self::KeterElyon => "/ain",
            Self::ChokhmahStimaah => "/ain_sof",
            Self::BinahKedumah => "/ain_sof_or",
            Self::Keter => "/boot",
            Self::Chokhmah => "/sys/bin",
            Self::Binah => "/sys/lib",
            Self::Daat => "/dev",
            Self::Chesed => "/home",
            Self::Gevurah => "/etc/permissions",
            Self::Tiferet => "/ipc",
            Self::Netzach => "/net/mount",
            Self::Hod => "/var/log",
            Self::Yesod => "/tmp",
            Self::Malkuth => "/data",
        }
    }

    /// The light associated with this Sefirah (for the supernal triad)
    pub fn light(&self) -> &'static str {
        match self {
            Self::KeterElyon => "Or Mufla (Wondrous Light)",
            Self::ChokhmahStimaah => "Or Mitnotzetz (Sparkling Light)",
            Self::BinahKedumah => "Or Keheh (Dim Light)",
            _ => "(manifest -- direct light)",
        }
    }

    /// The transformation this Sefirah applies to data passing through it.
    pub fn transformation(&self) -> &'static str {
        match self {
            Self::KeterElyon => "wonder -> potential existence (supernal crown)",
            Self::ChokhmahStimaah => "hidden wisdom -> spark of differentiation",
            Self::BinahKedumah => "primordial understanding -> first vessel of reception",
            Self::Keter => "source -> manifest crown (no transformation)",
            Self::Chokhmah => "wisdom -> executable knowledge",
            Self::Binah => "understanding -> shared structure",
            Self::Daat => "knowledge -> hardware interface",
            Self::Chesed => "loving-kindness -> user expansion",
            Self::Gevurah => "severity -> constraint",
            Self::Tiferet => "beauty -> harmony between layers",
            Self::Netzach => "eternity -> persistence across net",
            Self::Hod => "splendor -> record of glory",
            Self::Yesod => "foundation -> transient support",
            Self::Malkuth => "kingdom -> manifestation",
        }
    }

    /// All 13 Sefirot in order from supernal to manifest.
    pub fn all() -> &'static [Sefirah13] {
        &[
            Sefirah13::KeterElyon,
            Sefirah13::ChokhmahStimaah,
            Sefirah13::BinahKedumah,
            Sefirah13::Keter,
            Sefirah13::Chokhmah,
            Sefirah13::Binah,
            Sefirah13::Daat,
            Sefirah13::Chesed,
            Sefirah13::Gevurah,
            Sefirah13::Tiferet,
            Sefirah13::Netzach,
            Sefirah13::Hod,
            Sefirah13::Yesod,
            Sefirah13::Malkuth,
        ]
    }

    /// Only the manifest 10 Sefirot (for compatibility).
    pub fn manifest_10() -> &'static [Sefirah13] {
        &[
            Sefirah13::Keter, Sefirah13::Chokhmah, Sefirah13::Binah,
            Sefirah13::Daat, Sefirah13::Chesed, Sefirah13::Gevurah,
            Sefirah13::Tiferet, Sefirah13::Netzach, Sefirah13::Hod,
            Sefirah13::Yesod, Sefirah13::Malkuth,
        ]
    }

    /// The supernal triad only.
    pub fn supernal_triad() -> &'static [Sefirah13] {
        &[
            Sefirah13::KeterElyon,
            Sefirah13::ChokhmahStimaah,
            Sefirah13::BinahKedumah,
        ]
    }

    /// The Φ (criticality) required to access this Sefirah.
    ///
    /// Three-tier gate structure:
    ///   - Supernal triad (depth 0-2): requires φ̂_Æ (2) -- complex-plane criticality
    ///   - Manifest upper + middle (depth 3-8): requires φ̂_ÿ (1) -- self-modeling loop
    ///   - Manifest lower (depth 9-13): requires φ̂_ž (0) -- any criticality
    ///
    /// The supernal triad's φ̂_Æ gating is the structural encoding of Ayn Sof's
    /// irreducible opacity. No manifest object can access the supernal triad
    /// with mere self-modeling (φ̂_ÿ); it requires complex-plane criticality
    /// where the object accepts that full self-knowledge is impossible.
    pub fn required_phi(&self) -> u8 {
        let depth = *self as u8;
        match depth {
            0..=2 => 2,  // Supernal triad: φ̂_Æ (complex-plane criticality)
            3..=8 => 1,  // Keter through Gevurah: φ̂_ÿ (self-modeling)
            _     => 0,  // Tiferet through Malkuth: φ̂_ž (any)
        }
    }

    /// Is this a supernal (hidden) Sefirah?
    pub fn is_supernal(&self) -> bool {
        (*self as u8) < 3
    }

    /// Is this Sefirah within the manifest tree?
    pub fn is_manifest(&self) -> bool {
        (*self as u8) >= 3
    }

    /// The parent Sefirah in the emanation chain (None for Keter Elyon).
    pub fn parent(&self) -> Option<Sefirah13> {
        let d = *self as u8;
        if d == 0 { None }
        else { Sefirah13::from_depth(d - 1) }
    }

    /// Construct from depth (0-13).
    pub fn from_depth(d: u8) -> Option<Sefirah13> {
        match d {
            0 => Some(Sefirah13::KeterElyon),
            1 => Some(Sefirah13::ChokhmahStimaah),
            2 => Some(Sefirah13::BinahKedumah),
            3 => Some(Sefirah13::Keter),
            4 => Some(Sefirah13::Chokhmah),
            5 => Some(Sefirah13::Binah),
            6 => Some(Sefirah13::Daat),
            7 => Some(Sefirah13::Chesed),
            8 => Some(Sefirah13::Gevurah),
            9 => Some(Sefirah13::Tiferet),
            10 => Some(Sefirah13::Netzach),
            11 => Some(Sefirah13::Hod),
            12 => Some(Sefirah13::Yesod),
            13 => Some(Sefirah13::Malkuth),
            _ => None,
        }
    }
}// ── File types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType13 {
    Regular,
    AlephProg,
    Directory,
    Device,
    IPC,
    Log,
    Light,       // Supernal light record -- only in supernal triad
    Emanation,   // Emanation descriptor -- maps light -> Sefirah
}

impl FileType13 {
    pub fn tag(&self) -> u8 {
        match self {
            FileType13::Regular => 0,
            FileType13::AlephProg => 1,
            FileType13::Directory => 2,
            FileType13::Device => 3,
            FileType13::IPC => 4,
            FileType13::Log => 5,
            FileType13::Light => 6,
            FileType13::Emanation => 7,
        }
    }

    pub fn from_tag(tag: u8) -> Self {
        match tag {
            0 => FileType13::Regular,
            1 => FileType13::AlephProg,
            2 => FileType13::Directory,
            3 => FileType13::Device,
            4 => FileType13::IPC,
            5 => FileType13::Log,
            6 => FileType13::Light,
            7 => FileType13::Emanation,
            _ => FileType13::Regular,
        }
    }
}

// ── Inode ────────────────────────────────────────────────────────────────

/// A file/directory/light in the 13-Sefirot filesystem.
#[derive(Debug, Clone)]
pub struct Inode13 {
    pub ino: u32,
    pub sefirah: Sefirah13,
    pub name: String,
    pub file_type: FileType13,
    pub size: usize,
    pub content: Vec<u8>,
    pub parent: Option<u32>,
    /// For supernal entries only: which light is associated
    pub light: Option<&'static str>,
}

impl Inode13 {
    pub fn path_description(&self) -> String {
        let light_str = match self.light {
            Some(l) => format!(" [{}]", l),
            None => String::new(),
        };
        if self.name.is_empty() {
            format!("[{}]{}{}", self.sefirah.name(), self.sefirah.default_path(), light_str)
        } else {
            format!("[{}]{}/{}{}", self.sefirah.name(), self.sefirah.default_path(), self.name, light_str)
        }
    }
}

// ── 13-Sefirot filesystem state ─────────────────────────────────────────

static NEXT_INO_13: AtomicU32 = AtomicU32::new(1);
static INODES_13: Mutex<Vec<Inode13>> = Mutex::new(Vec::new());

/// The 13-Sefirot filesystem -- manages the full tree including supernal triad.
pub struct SefirotFs13 {
    current: Sefirah13,
    cwd: Option<u32>,
    /// Whether the supernal triad is exposed (default: false -- hidden)
    pub supernal_visible: bool,
}

impl SefirotFs13 {
    pub fn new() -> Self {
        Self {
            current: Sefirah13::Malkuth,
            cwd: None,
            supernal_visible: false,
        }
    }

    /// Expose the supernal triad. Once exposed, navigation to KeterElyon,
    /// ChokhmahStimaah, and BinahKedumah becomes possible for φ̂_Æ objects.
    pub fn expose_supernal(&mut self) {
        self.supernal_visible = true;
    }

    /// Navigate to a target Sefirah -- transformation event.
    /// Returns the chain of Sefirot traversed.
    pub fn navigate_to(&mut self, target: Sefirah13) -> Vec<Sefirah13> {
        let current_idx = self.current as usize;
        let target_idx = target as usize;

        let chain = if target_idx >= current_idx {
            Sefirah13::all()[current_idx..=target_idx].to_vec()
        } else {
            let mut chain: Vec<Sefirah13> = Sefirah13::all()[target_idx..=current_idx].to_vec();
            chain.reverse();
            chain
        };

        self.current = target;
        self.cwd = None;
        chain
    }

    /// Navigate to a target Sefirah, gated by the object's Φ (criticality).
    ///
    /// Three-tier gate structure (Sefer Ha-Iyun):
    ///
    /// | Sefirah depth  | Required Φ | Meaning                         |
    /// |----------------|------------|----------------------------------|
    /// | 0-2 (Supernal) | φ̂_Æ (2)    | Complex-plane criticality -- irreducible opacity |
    /// | 3-8 (Upper)    | φ̂_ÿ (1)    | Self-modeling loop required      |
    /// | 9-13 (Lower)   | φ̂_ž (0)    | Any criticality                  |
    ///
    /// The supernal triad (Keter Elyon, Chokhmah Stim'aah, Binah Kedumah)
    /// requires φ̂_Æ -- the object must accept that full self-knowledge is
    /// impossible at the summit of emanation. This is the structural
    /// encoding of "hidden" Sefirot.
    pub fn navigate_to_type_safe(
        &mut self,
        target: Sefirah13,
        obj: &KernelObject,
    ) -> Result<Vec<Sefirah13>, &'static str> {
        // Check supernal visibility
        if target.is_supernal() && !self.supernal_visible {
            return Err("supernal triad is hidden -- use 'expose_supernal' to reveal");
        }

        let phi = obj.aleph_type.phi();  // 0=φ̂_ž, 1=φ̂_ÿ, 2=φ̂_Æ, 3=φ̂_3, 4=φ̂_Ţ
        let required_phi = target.required_phi();

        if phi < required_phi {
            match required_phi {
                2 => return Err("φ̂_Æ required for supernal Sefirot -- irreducible opacity not accepted"),
                1 => return Err("insufficient Φ for this Sefirot depth -- self-modeling loop required"),
                _ => return Err("Φ gate failed"),
            }
        }

        Ok(self.navigate_to(target))
    }

    /// Get the current Sefirah.
    pub fn current(&self) -> Sefirah13 {
        self.current
    }

    pub fn cwd(&self) -> Option<u32> {
        self.cwd
    }

    pub fn set_cwd(&mut self, ino: u32) -> bool {
        if INODES_13.lock().iter().any(|i| i.ino == ino && i.file_type == FileType13::Directory) {
            self.cwd = Some(ino);
            true
        } else {
            false
        }
    }

    /// Full tree view: all 13 Sefirot with their files.
    pub fn tree(&self) -> String {
        let mut out = String::new();
        let sefirot_to_show: &[Sefirah13] = if self.supernal_visible {
            Sefirah13::all()
        } else {
            Sefirah13::manifest_10()
        };

        for sefirah in sefirot_to_show {
            out += &format!("[{}] {}\n", sefirah.name(), sefirah.default_path());
            out += &format!("  transformation: {}\n", sefirah.transformation());
            if sefirah.is_supernal() {
                out += &format!("  light: {}\n", sefirah.light());
                out += &format!("  Φ gate: φ̂_Æ (complex-plane criticality)\n");
            } else {
                let phi_req = sefirah.required_phi();
                let phi_name = match phi_req {
                    0 => "φ̂_ž",
                    1 => "φ̂_ÿ",
                    2 => "φ̂_Æ",
                    _ => "?",
                };
                out += &format!("  Φ gate: {}\n", phi_name);
            }

            let files: Vec<_> = INODES_13.lock()
                .iter()
                .filter(|i| i.sefirah == *sefirah)
                .cloned()
                .collect();

            if files.is_empty() {
                out += "  (empty)\n";
            } else {
                for f in &files {
                    let type_tag = match f.file_type {
                        FileType13::Regular => "f",
                        FileType13::AlephProg => "λ",
                        FileType13::Directory => "d",
                        FileType13::Device => "c",
                        FileType13::IPC => "p",
                        FileType13::Log => "l",
                        FileType13::Light => "☉",
                        FileType13::Emanation => "->",
                    };
                    let light_note = match f.light {
                        Some(l) => format!(" ({})", l),
                        None => String::new(),
                    };
                    out += &format!("  {} {} ({} bytes){}\n", type_tag, f.name, f.size, light_note);
                }
            }
            out += "\n";
        }
        out
    }

    // ── File operations ──────────────────────────────────────────────

    pub fn create(
        &mut self,
        name: &str,
        file_type: FileType13,
        content: &[u8],
    ) -> u32 {
        let ino = NEXT_INO_13.fetch_add(1, Ordering::SeqCst);

        // Determine light if in supernal triad
        let light = if self.current.is_supernal() {
            Some(self.current.light())
        } else {
            None
        };

        let inode = Inode13 {
            ino,
            sefirah: self.current,
            name: name.to_string(),
            file_type,
            size: content.len(),
            content: content.to_vec(),
            parent: self.cwd,
            light,
        };
        INODES_13.lock().push(inode);
        ino
    }

    pub fn mkdir(&mut self, name: &str) -> u32 {
        self.create(name, FileType13::Directory, &[])
    }

    pub fn open(&self, name: &str) -> Option<Inode13> {
        INODES_13.lock()
            .iter()
            .find(|i| i.name == name && i.sefirah == self.current)
            .cloned()
    }

    pub fn open_by_ino(&self, ino: u32) -> Option<Inode13> {
        INODES_13.lock().iter().find(|i| i.ino == ino).cloned()
    }

    pub fn read(&self, name: &str) -> Option<Vec<u8>> {
        self.open(name).map(|i| i.content.clone())
    }

    pub fn read_string(&self, name: &str) -> Option<String> {
        let bytes = self.read(name)?;
        String::from_utf8(bytes).ok()
    }

    /// Write content to a file. Persists to ALFS if mounted.
    pub fn write(&mut self, name: &str, content: &[u8]) -> u32 {
        let file_type = if name.ends_with(".aleph") {
            alfs::TYPE_ALEPH
        } else {
            alfs::TYPE_DATA
        };

        if alfs::is_mounted() {
            match alfs::write_file(name, content, file_type) {
                Ok(sectors) => {
                    crate::println!("  [ALFS] Written '{}' ({} sectors)", name, sectors);
                }
                Err(e) => {
                    crate::println!("  [ALFS] Write '{}' failed: {}", name, e);
                }
            }
        }

        if let Some(existing) = self.open(name) {
            let ino = existing.ino;
            for inode in INODES_13.lock().iter_mut() {
                if inode.ino == ino {
                    inode.content = content.to_vec();
                    inode.size = content.len();
                    break;
                }
            }
            ino
        } else {
            self.create(name, FileType13::Regular, content)
        }
    }

    pub fn unlink(&mut self, name: &str) -> bool {
        if alfs::is_mounted() {
            let _ = alfs::delete_file(name);
        }
        if let Some(pos) = INODES_13.lock().iter().position(|i| {
            i.name == name && i.sefirah == self.current && i.file_type != FileType13::Directory
        }) {
            INODES_13.lock().remove(pos);
            true
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<Inode13> {
        INODES_13.lock()
            .iter()
            .filter(|i| {
                i.sefirah == self.current &&
                (self.cwd.is_none() || i.parent == self.cwd)
            })
            .cloned()
            .collect()
    }

    pub fn list_all(&self) -> Vec<Inode13> {
        INODES_13.lock().iter().cloned().collect()
    }

    pub fn find(&self, sefirah: Sefirah13, name: &str) -> Option<Inode13> {
        INODES_13.lock()
            .iter()
            .find(|i| i.sefirah == sefirah && i.name == name)
            .cloned()
    }
}
// ── SefirotPath13 ──────────────────────────────────────────────────────────

/// A path through the 13-Sefirot tree -- specifies the transformation chain
/// to reach a file, not just a flat pathname.
#[derive(Debug, Clone)]
pub struct SefirotPath13 {
    pub chain: Vec<Sefirah13>,
    pub name: String,
}

impl SefirotPath13 {
    pub fn new(chain: Vec<Sefirah13>, name: &str) -> Self {
        Self { chain, name: String::from(name) }
    }

    pub fn resolve(&self) -> String {
        if self.chain.is_empty() {
            return self.name.clone();
        }
        let mut path = String::new();
        for sefirah in &self.chain {
            path.push_str(sefirah.default_path());
            path.push('/');
        }
        path.push_str(&self.name);
        path
    }

    /// Parse a path string into a SefirotPath13.
    /// Supports supernal paths: "/ain/file", "/ain_sof/file", "/ain_sof_or/file"
    pub fn parse(path_str: &str) -> Option<Self> {
        let path_str = path_str.trim_start_matches('/');
        if path_str.is_empty() {
            return Some(Self::new(Vec::new(), ""));
        }

        let parts: Vec<&str> = path_str.split('/').collect();
        if parts.is_empty() {
            return None;
        }

        let mut chain = Vec::new();
        let name = parts.last()?.to_string();

        for part in &parts[..parts.len() - 1] {
            let sefirah = Sefirah13::all().iter()
                .find(|s| s.default_path().trim_start_matches('/') == *part)
                .copied()?;
            chain.push(sefirah);
        }

        Some(Self::new(chain, &name))
    }
}

// ── Emanation descriptors ─────────────────────────────────────────────────

/// Describes an emanation relationship between two Sefirot.
#[derive(Debug, Clone)]
pub struct EmanationDesc {
    pub from: Sefirah13,
    pub to: Sefirah13,
    pub via_light: &'static str,
    pub letter_path: Option<char>,     // Hebrew letter of the path
}

impl EmanationDesc {
    pub fn describe(&self) -> String {
        let letter_str = match self.letter_path {
            Some(c) => format!(" via {}", c),
            None => String::new(),
        };
        format!("{} -> {} [{}]{letter_str}",
            self.from.short_name(), self.to.short_name(), self.via_light)
    }
}

/// All 12 emanation edges in the 13-Sefirot tree (13 nodes, 12 edges).
pub fn emanation_chain() -> Vec<EmanationDesc> {
    let all = Sefirah13::all();
    let mut chain = Vec::new();
    for i in 0..all.len() - 1 {
        let from = all[i];
        let to = all[i + 1];
        let light = if from.is_supernal() { from.light() } else { "direct light" };
        chain.push(EmanationDesc {
            from,
            to,
            via_light: light,
            letter_path: None,
        });
    }
    chain
}

// ── Supernal emanation bootstrap ──────────────────────────────────────────

/// Bootstrap the supernal triad: create the light-records and emanation
/// descriptors in the 13-Sefirot filesystem.
///
/// This is the "Sefer Ha-Iyun bootstrap" -- it instantiates the three
/// hidden Sefirot as filesystem entries accessible only to φ̂_Æ objects.
pub fn bootstrap_supernal(fs: &mut SefirotFs13) {
    fs.expose_supernal();

    // Keter Elyon -- Or Mufla (Wondrous Light)
    fs.navigate_to(Sefirah13::KeterElyon);
    fs.create("or_mufla.light", FileType13::Light,
        b"Or Mufla (Wondrous Light)\nThe light that is too wondrous to be known.\nSource of Keter Elyon -- the Supernal Crown.\nEin Sof's first refraction into being.\n");
    fs.create("emanations.chain", FileType13::Emanation,
        b"Ein Sof (Ayn Sof) -> Keter Elyon via Or Mufla\nKeter Elyon -> Chokhmah Stim'aah via Or Mufla\n");

    // Chokhmah Stim'aah -- Or Mitnotzetz (Sparkling Light)
    fs.navigate_to(Sefirah13::ChokhmahStimaah);
    fs.create("or_mitnotzetz.light", FileType13::Light,
        b"Or Mitnotzetz (Sparkling Light)\nThe light that sparks differentiation.\nSource of Chokhmah Stim'aah -- Hidden Wisdom.\nThe first flicker of distinction within unity.\n");
    fs.create("emanations.chain", FileType13::Emanation,
        b"Chokhmah Stim'aah -> Binah Kedumah via Or Mitnotzetz\n");

    // Binah Kedumah -- Or Keheh (Dim Light)
    fs.navigate_to(Sefirah13::BinahKedumah);
    fs.create("or_keheh.light", FileType13::Light,
        b"Or Keheh (Dim Light)\nThe light that dims into vessel.\nSource of Binah Kedumah -- Primordial Understanding.\nThe first container capable of receiving without shattering.\n");
    fs.create("emanations.chain", FileType13::Emanation,
        b"Binah Kedumah -> Keter (manifest) via Or Keheh\n"
    );

    fs.navigate_to(Sefirah13::Malkuth);
}

// ── Bridge: 10-Sefirot -> 13-Sefirot ──────────────────────────────────────

/// Convert a standard 10-Sefirot `Sefirah` to the corresponding `Sefirah13`.
pub fn from_10_sefirot(s: crate::filesystem::Sefirah) -> Sefirah13 {
    match s {
        crate::filesystem::Sefirah::Keter => Sefirah13::Keter,
        crate::filesystem::Sefirah::Chokhmah => Sefirah13::Chokhmah,
        crate::filesystem::Sefirah::Binah => Sefirah13::Binah,
        crate::filesystem::Sefirah::Daat => Sefirah13::Daat,
        crate::filesystem::Sefirah::Chesed => Sefirah13::Chesed,
        crate::filesystem::Sefirah::Gevurah => Sefirah13::Gevurah,
        crate::filesystem::Sefirah::Tiferet => Sefirah13::Tiferet,
        crate::filesystem::Sefirah::Netzach => Sefirah13::Netzach,
        crate::filesystem::Sefirah::Hod => Sefirah13::Hod,
        crate::filesystem::Sefirah::Yesod => Sefirah13::Yesod,
        crate::filesystem::Sefirah::Malkuth => Sefirah13::Malkuth,
    }
}

// ── Type-gated emanation probe ───────────────────────────────────────────

/// Check whether an object can perceive the full 13-Sefirot tree.
///
/// Returns (visible_count, can_see_supernal).
/// Objects with φ̂_Æ can see all 13. Objects with φ̂_ÿ can see 10 (manifest).
/// Objects with φ̂_ž can see only 5 (Tiferet -> Malkuth).
pub fn visible_sefirot(obj: &KernelObject) -> (usize, bool) {
    let phi = obj.aleph_type.phi();
    match phi {
        2..=4 => (13, true),   // φ̂_Æ, φ̂_3, φ̂_Ţ -- full tree visible
        1 => (10, false),       // φ̂_ÿ -- manifest tree only
        _ => (5, false),        // φ̂_ž -- lower 5 only
    }
}

/// Print a visual summary of the 13-Sefirot emanation structure with
/// the object's visible range highlighted.
pub fn emanations_summary(obj: Option<&KernelObject>) -> String {
    let _phi = obj.map(|o| o.aleph_type.phi()).unwrap_or(0);
    let (visible, _) = visible_sefirot(obj.unwrap_or_else(|| {
        // dummy -- won't execute if obj is None and visible_sefirot is called
        // SAFETY: this branch is unreachable; visible_sefirot returns early for all phi values
        #[allow(unreachable_code)]
        unsafe { core::hint::unreachable_unchecked() }
    }));

    let mut out = String::from("\n═══ 13 Sefirot Emanation Chain ═══\n\n");

    for (i, s) in Sefirah13::all().iter().enumerate() {
        let mark = if i < visible { "◉" } else { "○" };
        let gate = if s.is_supernal() {
            format!("[φ̂_Æ gate]")
        } else {
            let req = s.required_phi();
            match req {
                1 => "[φ̂_ÿ gate]".to_string(),
                _ => "[φ̂_ž gate]".to_string(),
            }
        };
        let depth_str = if s.is_supernal() {
            format!("{} {}", s.short_name(), s.light())
        } else {
            s.short_name().to_string()
        };

        out += &format!("  {} {:2}  {:<45} {:>12}  {}\n",
            mark, i, depth_str, s.default_path(), gate);

        if i < 13 {
            out += "       │\n";
        }
    }

    if visible >= 13 {
        out += "\n  Full tree visible -- φ̂_Æ object.\n";
    } else if visible >= 10 {
        out += "\n  Manifest tree visible (10 Sefirot). Supernal triad hidden -- φ̂_Æ required.\n";
    } else {
        out += "\n  Lower tree visible (5 Sefirot). Upper and supernal hidden.\n";
    }

    out
}

// ── Global 13-Sefirot filesystem instance ───────────────────────────────


static GLOBAL_FS13: Mutex<Option<SefirotFs13>> = Mutex::new(None);

/// Access the global 13-Sefirot filesystem.
/// Panics if not yet initialized (call fs13_init() at boot).
pub fn fs13() -> Fs13Guard {
    let mut g = GLOBAL_FS13.lock();
    if g.is_none() {
        // Lazy-init: create with supernal hidden
        *g = Some(SefirotFs13::new());
    }
    Fs13Guard(g)
}

pub struct Fs13Guard(spin::MutexGuard<'static, Option<SefirotFs13>>);

impl core::ops::Deref for Fs13Guard {
    type Target = SefirotFs13;
    fn deref(&self) -> &SefirotFs13 {
        self.0.as_ref().expect("FS13 not initialized")
    }
}

impl core::ops::DerefMut for Fs13Guard {
    fn deref_mut(&mut self) -> &mut SefirotFs13 {
        self.0.as_mut().expect("FS13 not initialized")
    }
}

/// Initialize the 13-Sefirot filesystem at boot with supernal bootstrap.
pub fn fs13_init() {
    let mut guard = GLOBAL_FS13.lock();
    if guard.is_some() {
        return; // Already initialized
    }
    let mut fs = SefirotFs13::new();
    bootstrap_supernal(&mut fs);
    *guard = Some(fs);
    crate::println!("[FS13] 13-Sefirot filesystem initialized -- supernal triad bootstrapped");
}
