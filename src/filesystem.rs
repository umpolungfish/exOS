//! Sefirot filesystem — full implementation.
//!
//! Files are not flat objects but nodes in a Sefirot-like tree where:
//! - Each node encodes its relation to the root (Keter = the kernel)
//!   and its leaves (Malkuth = the user interface)
//! - The ten Sefirot correspond to ten filesystem abstraction layers,
//!   each with a defined transformation role (the letter-path between layers)
//! - Navigation is not by pathname alone but by **transformation** —
//!   you specify HOW you want to arrive at a file, not just WHERE it is
//!
//! The ten Sefirot layers (from root to manifestation):
//!
//! | # | Sefirah     | FS Role                        | Transformation                    |
//! |---|-------------|--------------------------------|-----------------------------------|
//! | 0 | Keter       | Kernel root / boot config      | Source — no transformation        |
//! | 1 | Chokhmah    | System binaries                | Wisdom → executable knowledge     |
//! | 2 | Binah       | System libraries               | Understanding → shared structure  |
//! | 3 | Da'at       | Device nodes                   | Knowledge → hardware interface    |
//! | 4 | Chesed      | User home directories          | Loving-kindness → user expansion  |
//! | 5 | Gevurah     | Permission/access control      | Severity → constraint             |
//! | 6 | Tiferet     | Shared memory / IPC endpoints  | Beauty → harmony between layers   |
//! | 7 | Netzach     | Network filesystem mounts      | Eternity → persistence across net |
//! | 8 | Hod         | Log files / audit trails       | Splendor → record of glory        |
//! | 9 | Yesod       | Temp files / caches            | Foundation — transient support    |
//! |10 | Malkuth     | User-facing data / UI configs  | Kingdom → manifestation           |
//!
//! Architecture:
//!   ALFS provides raw sector storage (ATA disk driver).
//!   SefirotFs layers a VFS on top: inodes, directories, paths.
//!   Each file lives in exactly one Sefirah level.
//!   The SefirotPath (chain of Sefirot traversed) is part of the file's identity.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::string::ToString;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;

use crate::alfs;
use crate::kernel_object::KernelObject;

// ── Sefirot levels ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Sefirah {
    Keter = 0,
    Chokhmah = 1,
    Binah = 2,
    Daat = 3,
    Chesed = 4,
    Gevurah = 5,
    Tiferet = 6,
    Netzach = 7,
    Hod = 8,
    Yesod = 9,
    Malkuth = 10,
}

impl Sefirah {
    pub fn name(&self) -> &'static str {
        match self {
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

    pub fn default_path(&self) -> &'static str {
        match self {
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

    /// The transformation this Sefirah applies to data passing through it.
    /// Returns a description string.
    pub fn transformation(&self) -> &'static str {
        match self {
            Self::Keter => "source (no transformation)",
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

    pub fn all() -> &'static [Sefirah] {
        &[
            Sefirah::Keter, Sefirah::Chokhmah, Sefirah::Binah, Sefirah::Daat,
            Sefirah::Chesed, Sefirah::Gevurah, Sefirah::Tiferet, Sefirah::Netzach,
            Sefirah::Hod, Sefirah::Yesod, Sefirah::Malkuth,
        ]
    }
}

// ── File types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,    // Normal file
    AlephProg,  // .aleph program (executable)
    Directory,  // Directory node
    Device,     // Device node
    IPC,        // IPC endpoint
    Log,        // Log file (append-only)
}

impl FileType {
    pub fn tag(&self) -> u8 {
        match self {
            FileType::Regular => 0,
            FileType::AlephProg => 1,
            FileType::Directory => 2,
            FileType::Device => 3,
            FileType::IPC => 4,
            FileType::Log => 5,
        }
    }

    pub fn from_tag(tag: u8) -> Self {
        match tag {
            0 => FileType::Regular,
            1 => FileType::AlephProg,
            2 => FileType::Directory,
            3 => FileType::Device,
            4 => FileType::IPC,
            5 => FileType::Log,
            _ => FileType::Regular,
        }
    }
}

// ── Inode ────────────────────────────────────────────────────────────────────

/// A file/directory in the Sefirot filesystem.
#[derive(Debug, Clone)]
pub struct Inode {
    pub ino: u32,              // Inode number (unique)
    pub sefirah: Sefirah,      // Which Sefirah level this belongs to
    pub name: String,          // Filename (no path)
    pub file_type: FileType,   // File type
    pub size: usize,           // File size in bytes
    pub content: Vec<u8>,      // File content (stored in memory for now)
    pub parent: Option<u32>,   // Parent directory inode number
}

impl Inode {
    /// Full SefirotPath description
    pub fn path_description(&self) -> String {
        if self.name.is_empty() {
            format!("[{}]{}", self.sefirah.name(), self.sefirah.default_path())
        } else {
            format!("[{}]{}{}", self.sefirah.name(), self.sefirah.default_path(), 
                format!("/{}", self.name))
        }
    }
}

// ── Sefirot filesystem state ────────────────────────────────────────────────

static NEXT_INO: AtomicU32 = AtomicU32::new(1);

// In-memory inode table. In a full implementation this would be backed by ALFS.
static INODES: Mutex<Vec<Inode>> = Mutex::new(Vec::new());

/// The Sefirot filesystem — manages the tree of abstraction layers.
pub struct SefirotFs {
    /// Current working Sefirah (default: Malkuth — user space)
    current: Sefirah,
    /// Current working directory (inode number, None = root of current Sefirah)
    cwd: Option<u32>,
}

impl SefirotFs {
    pub fn new() -> Self {
        Self {
            current: Sefirah::Malkuth,
            cwd: None,
        }
    }

    /// Navigate to a target Sefirah — the transformation event.
    /// Returns the list of Sefirot traversed (the transformation chain).
    pub fn navigate_to(&mut self, target: Sefirah) -> Vec<Sefirah> {
        // Build the transformation chain from current to target
        let current_idx = self.current as usize;
        let target_idx = target as usize;
        
        let chain = if target_idx >= current_idx {
            // Ascending: Keter -> Malkuth (manifestation)
            Sefirah::all()[current_idx..=target_idx].to_vec()
        } else {
            // Descending: Malkuth -> Keter (return to source)
            let mut chain: Vec<Sefirah> = Sefirah::all()[target_idx..=current_idx].to_vec();
            chain.reverse();
            chain
        };
        
        self.current = target;
        self.cwd = None; // Reset to root of new Sefirah
        chain
    }

    /// Navigate to a target Sefirah, gated by the object's Φ (criticality) level.
    ///
    /// Higher Sefirot (closer to Keter, the source) require higher criticality.
    /// This encodes the Kabbalistic principle that proximity to the divine source
    /// requires a self-modeling loop capable of sustaining that proximity.
    ///
    /// Φ requirements by Sefirot depth:
    ///
    /// | Sefirot          | Depth | Required Φ | Rationale                        |
    /// |------------------|-------|------------|----------------------------------|
    /// | Keter, Chokhmah, Binah | 0-2 | Φ_c (1)     | Supernal triad — self-modeling loop required |
    /// | Daat, Chesed, Gevurah  | 3-5 | Φ_c (1)     | Middle pillars — self-modeling loop required |
    /// | Tiferet → Malkuth      | 6-10| Φ_sub (0)   | Manifest layers — accessible to all |
    ///
    /// Note: Φ_c (ordinal 1) is the highest criticality value instantiated
    /// in the canonical 22-letter system. Φ_c_complex (2) is a meta-critical
    /// state in the type theory but is not used by any canonical Hebrew letter.
    /// Therefore, the supernal triad gates on Φ_c, not Φ_c_complex.
    pub fn navigate_to_type_safe(
        &mut self,
        target: Sefirah,
        obj: &KernelObject,
    ) -> Result<Vec<Sefirah>, &'static str> {
        let phi = obj.aleph_type.phi();
        let target_depth = target as u8;

        let required_phi = match target_depth {
            0..=5 => 1,  // Keter through Gevurah — Φ_c minimum
            _ => 0,      // Tiferet through Malkuth — any Φ
        };

        if phi < required_phi {
            return Err("insufficient Φ for this Sefirot depth");
        }

        Ok(self.navigate_to(target))
    }

    /// Get the current Sefirah
    pub fn current(&self) -> Sefirah {
        self.current
    }

    /// Get the current working directory
    pub fn cwd(&self) -> Option<u32> {
        self.cwd
    }

    /// Set the current working directory by inode number
    pub fn set_cwd(&mut self, ino: u32) -> bool {
        if INODES.lock().iter().any(|i| i.ino == ino && i.file_type == FileType::Directory) {
            self.cwd = Some(ino);
            true
        } else {
            false
        }
    }

    /// List all Sefirot layers and their paths
    pub fn tree(&self) -> &'static [(Sefirah, &'static str)] {
        &[
            (Sefirah::Keter, "/boot"),
            (Sefirah::Chokhmah, "/sys/bin"),
            (Sefirah::Binah, "/sys/lib"),
            (Sefirah::Daat, "/dev"),
            (Sefirah::Chesed, "/home"),
            (Sefirah::Gevurah, "/etc/permissions"),
            (Sefirah::Tiferet, "/ipc"),
            (Sefirah::Netzach, "/net/mount"),
            (Sefirah::Hod, "/var/log"),
            (Sefirah::Yesod, "/tmp"),
            (Sefirah::Malkuth, "/data"),
        ]
    }

    // ── File operations ──────────────────────────────────────────────────

    /// Create a new file in the current Sefirah.
    pub fn create(&mut self, name: &str, file_type: FileType, content: &[u8]) -> u32 {
        let ino = NEXT_INO.fetch_add(1, Ordering::SeqCst);
        let inode = Inode {
            ino,
            sefirah: self.current,
            name: name.to_string(),
            file_type,
            size: content.len(),
            content: content.to_vec(),
            parent: self.cwd,
        };
        INODES.lock().push(inode);
        ino
    }

    /// Create a directory in the current Sefirah.
    pub fn mkdir(&mut self, name: &str) -> u32 {
        self.create(name, FileType::Directory, &[])
    }

    /// Open a file by name in the current Sefirah.
    pub fn open(&self, name: &str) -> Option<Inode> {
        INODES.lock()
            .iter()
            .find(|i| i.name == name && i.sefirah == self.current)
            .cloned()
    }

    /// Open a file by inode number.
    pub fn open_by_ino(&self, ino: u32) -> Option<Inode> {
        INODES.lock().iter().find(|i| i.ino == ino).cloned()
    }

    /// Read a file's content as bytes.
    pub fn read(&self, name: &str) -> Option<Vec<u8>> {
        self.open(name).map(|i| i.content.clone())
    }

    /// Read a file's content as a string.
    pub fn read_string(&self, name: &str) -> Option<String> {
        let bytes = self.read(name)?;
        String::from_utf8(bytes).ok()
    }

    /// Write content to a file (creates if doesn't exist, overwrites if does).
    /// Persists to ALFS disk if mounted.
    pub fn write(&mut self, name: &str, content: &[u8]) -> u32 {
        let file_type = if name.ends_with(".aleph") {
            alfs::TYPE_ALEPH
        } else {
            alfs::TYPE_DATA
        };

        // Persist to ALFS disk
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

        // Update in-memory inode
        if let Some(existing) = self.open(name) {
            let ino = existing.ino;
            for inode in INODES.lock().iter_mut() {
                if inode.ino == ino {
                    inode.content = content.to_vec();
                    inode.size = content.len();
                    break;
                }
            }
            ino
        } else {
            self.create(name, FileType::Regular, content)
        }
    }

    /// Delete a file by name. Removes from both memory and disk.
    pub fn unlink(&mut self, name: &str) -> bool {
        // Remove from ALFS disk
        if alfs::is_mounted() {
            let _ = alfs::delete_file(name);
        }

        // Remove from memory
        if let Some(pos) = INODES.lock().iter().position(|i| {
            i.name == name && i.sefirah == self.current && i.file_type != FileType::Directory
        }) {
            INODES.lock().remove(pos);
            true
        } else {
            false
        }
    }

    /// List files in the current Sefirah (optionally filtered by parent directory).
    pub fn list(&self) -> Vec<Inode> {
        INODES.lock()
            .iter()
            .filter(|i| {
                i.sefirah == self.current &&
                (self.cwd.is_none() || i.parent == self.cwd)
            })
            .cloned()
            .collect()
    }

    /// List all files across all Sefirot levels.
    pub fn list_all(&self) -> Vec<Inode> {
        INODES.lock().iter().cloned().collect()
    }

    /// Find a file by Sefirot path: sefirah + name.
    pub fn find(&self, sefirah: Sefirah, name: &str) -> Option<Inode> {
        INODES.lock()
            .iter()
            .find(|i| i.sefirah == sefirah && i.name == name)
            .cloned()
    }

    /// Resolve a SefirotPath (chain of Sefirot + filename) to an inode.
    /// The chain specifies the transformation path taken to reach the file.
    pub fn resolve_path(&self, path: &SefirotPath) -> Option<Inode> {
        if path.chain.is_empty() {
            // Simple path: just look up in current Sefirah
            return self.open(&path.name);
        }
        
        // The last Sefirah in the chain determines where the file lives
        let target_sefirah = *path.chain.last().unwrap();
        
        INODES.lock()
            .iter()
            .find(|i| i.sefirah == target_sefirah && i.name == path.name)
            .cloned()
    }

    /// Get the full tree view: all Sefirot with their files.
    pub fn full_tree(&self) -> String {
        let mut out = String::new();
        for sefirah in Sefirah::all() {
            out += &format!("[{}] {}\n", sefirah.name(), sefirah.default_path());
            out += &format!("  transformation: {}\n", sefirah.transformation());
            
            let files: Vec<_> = INODES.lock()
                .iter()
                .filter(|i| i.sefirah == *sefirah)
                .cloned()
                .collect();
            
            if files.is_empty() {
                out += "  (empty)\n";
            } else {
                for f in &files {
                    let type_tag = match f.file_type {
                        FileType::Regular => "f",
                        FileType::AlephProg => "λ",
                        FileType::Directory => "d",
                        FileType::Device => "c",
                        FileType::IPC => "p",
                        FileType::Log => "l",
                    };
                    out += &format!("  {} {} ({} bytes)\n", type_tag, f.name, f.size);
                }
            }
            out += "\n";
        }
        out
    }
}

// ── SefirotPath ──────────────────────────────────────────────────────────────

/// A path through the Sefirot tree — specifies the transformation chain
/// to reach a file, not just a flat pathname.
#[derive(Debug, Clone)]
pub struct SefirotPath {
    /// The sequence of Sefirot traversed to reach the target
    pub chain: Vec<Sefirah>,
    /// The filename at the leaf
    pub name: String,
}

impl SefirotPath {
    pub fn new(chain: Vec<Sefirah>, name: &str) -> Self {
        Self {
            chain,
            name: String::from(name),
        }
    }

    /// The full resolved path
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

    /// Parse a path string into a SefirotPath.
    /// Format: "/boot/file" or "/sys/bin/prog" etc.
    pub fn parse(path_str: &str) -> Option<Self> {
        let path_str = path_str.trim_start_matches('/');
        if path_str.is_empty() {
            return Some(Self::new(Vec::new(), ""));
        }
        
        let parts: Vec<&str> = path_str.split('/').collect();
        if parts.is_empty() {
            return None;
        }
        
        // Determine which Sefirot this path belongs to
        let mut chain = Vec::new();
        let name = parts.last()?.to_string();
        
        for part in &parts[..parts.len() - 1] {
            let sefirah = Sefirah::all().iter()
                .find(|s| s.default_path().trim_start_matches('/') == *part)
                .copied()?;
            chain.push(sefirah);
        }
        
        Some(Self::new(chain, &name))
    }
}

// ── Boot-time initialization ────────────────────────────────────────────────

/// Initialize the Sefirot filesystem from ALFS disk.
/// Loads all files from ALFS into the Sefirot tree.
pub fn mount_from_alfs(fs: &mut SefirotFs) -> Result<usize, &'static str> {
    if !alfs::is_mounted() {
        return Err("ALFS not mounted");
    }
    
    let files = alfs::list();
    let mut count = 0;
    
    for file_info in &files {
        if let Some(content) = alfs::read_file(&file_info.name) {
            let file_type = match file_info.file_type {
                alfs::TYPE_ALEPH => FileType::AlephProg,
                _ => FileType::Regular,
            };
            
            // Determine which Sefirah this file belongs to
            // Default: Malkuth (user data) for .aleph files, others by convention
            let sefirah = infer_sefirah(&file_info.name);
            fs.navigate_to(sefirah);
            
            fs.create(&file_info.name, file_type, &content);
            count += 1;
        }
    }
    
    // Return to Malkuth
    fs.navigate_to(Sefirah::Malkuth);
    Ok(count)
}

/// Infer which Sefirah a file belongs to based on its name and type.
fn infer_sefirah(name: &str) -> Sefirah {
    if name.ends_with(".aleph") {
        // .aleph programs live in Chokhmah (system binaries / executable knowledge)
        Sefirah::Chokhmah
    } else if name.ends_with(".log") {
        Sefirah::Hod
    } else if name.starts_with("ipc_") || name.ends_with(".ipc") {
        Sefirah::Tiferet
    } else if name.starts_with("dev_") || name.starts_with("node_") {
        Sefirah::Daat
    } else {
        Sefirah::Malkuth
    }
}

// ── Global filesystem instance ──────────────────────────────────────────────

static GLOBAL_FS: Mutex<Option<SefirotFs>> = Mutex::new(None);

/// Get a guard to the global filesystem (mutex-protected).
pub fn fs() -> FsGuard {
    let mut g = GLOBAL_FS.lock();
    if g.is_none() {
        *g = Some(SefirotFs::new());
    }
    FsGuard(g)
}

/// Mutex guard wrapper for the global filesystem.
pub struct FsGuard(spin::MutexGuard<'static, Option<SefirotFs>>);

impl core::ops::Deref for FsGuard {
    type Target = SefirotFs;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}
impl core::ops::DerefMut for FsGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

/// Initialize the global filesystem from ALFS.
pub fn init() -> Result<usize, &'static str> {
    let mut filesystem = SefirotFs::new();
    let count = mount_from_alfs(&mut filesystem)?;
    *GLOBAL_FS.lock() = Some(filesystem);
    Ok(count)
}

/// Populate the global filesystem with built-in seed files.
/// Called at boot so `ls` and `cat` work immediately without a disk.
pub fn populate_defaults() {
    let mut fs = self::fs();

    // Keter — kernel root / boot config
    fs.navigate_to(Sefirah::Keter);
    fs.create("README", FileType::Regular,
        b"Keter: kernel root.\nThis Sefirah holds boot configuration and kernel identity.\n");

    // Chokhmah — system binaries / .aleph programs
    fs.navigate_to(Sefirah::Chokhmah);
    fs.create("hello.aleph", FileType::AlephProg,
        b"# Greeting: tensor aleph x shin (source ^ fire)\naleph x shin\n");
    fs.create("census.aleph", FileType::AlephProg,
        b"# Tier census of the 22-letter system\n:census\n");
    fs.create("distance.aleph", FileType::AlephProg,
        b"# Structural distance between mem and shin\nd(mem, shin)\n");

    // Binah — system libraries
    fs.navigate_to(Sefirah::Binah);
    fs.create("types.txt", FileType::Regular,
"12-primitive IG type lattice:\n  Ð_ω   Þ_O   Ř_=   Φ_±\n  ƒ_ℏ   Ç_mod Γ_aleph ɢ_seq\n  ⊙_c   Ħ_∞   Σ_1:1 Ω_Z\n".as_bytes());

    // Hod — logs
    fs.navigate_to(Sefirah::Hod);
    fs.create("boot.log", FileType::Log,
        b"exoterikOS boot log\n[OK] Serial\n[OK] Heap\n[OK] Framebuffer\n[OK] IDT\n[OK] Shell\n");

    // Malkuth — user data (default CWD)
    fs.navigate_to(Sefirah::Malkuth);
    fs.create("welcome.txt", FileType::Regular,
        b"Welcome to exoterikOS!\n\nCommands: help, ls, cd <sefirah>, cat <file>, write <file> <content>\n         aleph, type-check, history N, bench\n\nType 'history 50' to scroll back through output.\n");
    fs.create("notes.txt", FileType::Regular,
        b"Your notes go here.\nUse: write notes.txt <content>\n");
}
