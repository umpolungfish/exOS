//! ALFS — ALEPH Linear Filesystem with full read/write support.
//!
//! Simple sector-based filesystem on raw ATA disk.
//!
//! Layout (relative to ALFS region start):
//!   Sector 0    : Superblock (magic "ALFS", version, counts, bitmap)
//!   Sectors 1..N: Directory entries (64 bytes each, 8 per sector)
//!   Sectors N+..: File data (raw sectors)
//!
//! Directory entry format (64 bytes):
//!   [0..32]  : filename (null-terminated ASCII)
//!   [32..36] : start_sector (LE u32) — relative to DATA_START_SECTOR
//!   [36..40] : sector_count (LE u32)
//!   [40]     : file type (0=data, 1=aleph program, 2=temp)
//!   [41]     : valid flag (0xFF = valid, 0x00 = deleted)
//!   [42..64] : reserved
//!
//! Max filename: 31 chars. Max files: 128.
//! Sector allocation: first-fit from DATA_START_SECTOR.

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use alloc::string::ToString;

use crate::ata;

// ── Constants ────────────────────────────────────────────────────────────────

const SUPERBLOCK_MAGIC: [u8; 4] = *b"ALFS";
const SUPERBLOCK_VERSION: u16 = 1;

/// ALFS superblock offset within the data disk (sector 0 of alfs.img).
pub const ALFS_OFFSET_SECTORS: u32 = 0;

const DIR_START_SECTOR: u32 = 1;
pub const DATA_START_SECTOR: u32 = 17;  // 16 sectors reserved for directory
const SECTOR_SIZE: usize = 512;

const DIR_ENTRY_SIZE: usize = 64;
const ENTRIES_PER_SECTOR: usize = SECTOR_SIZE / DIR_ENTRY_SIZE;
const DIR_SECTORS: u32 = 16;
const MAX_ENTRIES: usize = (DIR_SECTORS as usize) * ENTRIES_PER_SECTOR;  // 128

/// Maximum data sectors we can allocate (arbitrary limit, fits in bitmap).
const MAX_DATA_SECTORS: usize = 1024;

// ── File types ───────────────────────────────────────────────────────────────

pub const TYPE_DATA: u8 = 0;
pub const TYPE_ALEPH: u8 = 1;
pub const TYPE_TEMP: u8 = 2;

// ── File metadata ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub start_sector: u32,  // Relative to DATA_START_SECTOR
    pub sector_count: u32,
    pub file_type: u8,
}

// ── Sector allocator ─────────────────────────────────────────────────────────

/// Bitset tracking which data sectors are allocated.
/// Bit i is set if sector (DATA_START_SECTOR + i) is in use.
static mut SECTOR_BITMAP: [u64; (MAX_DATA_SECTORS + 63) / 64] = [0; (MAX_DATA_SECTORS + 63) / 64];

fn alloc_sectors(count: u32) -> Option<u32> {
    let count = count as usize;
    unsafe {
        // First-fit: find 'count' consecutive free bits
        let mut consecutive = 0;
        let mut start_bit = None;
        
        for bit in 0..MAX_DATA_SECTORS {
            let word_idx = bit / 64;
            let bit_idx = bit % 64;
            if (SECTOR_BITMAP[word_idx] & (1u64 << bit_idx)) == 0 {
                // Free
                if start_bit.is_none() {
                    start_bit = Some(bit);
                }
                consecutive += 1;
                if consecutive >= count {
                    // Found it — mark as allocated
                    for b in start_bit.unwrap()..start_bit.unwrap() + count {
                        let wi = b / 64;
                        let bi = b % 64;
                        SECTOR_BITMAP[wi] |= 1u64 << bi;
                    }
                    return Some(start_bit.unwrap() as u32);
                }
            } else {
                // Occupied — reset
                start_bit = None;
                consecutive = 0;
            }
        }
    }
    None  // Out of space
}

fn free_sectors(start: u32, count: u32) {
    unsafe {
        for i in 0..count {
            let bit = (start + i) as usize;
            if bit < MAX_DATA_SECTORS {
                let wi = bit / 64;
                let bi = bit % 64;
                SECTOR_BITMAP[wi] &= !(1u64 << bi);
            }
        }
    }
}

fn is_sector_allocated(rel_sector: u32) -> bool {
    let bit = rel_sector as usize;
    if bit >= MAX_DATA_SECTORS {
        return true;  // Out of range = treated as allocated
    }
    unsafe {
        let wi = bit / 64;
        let bi = bit % 64;
        (SECTOR_BITMAP[wi] & (1u64 << bi)) != 0
    }
}

// ── State ────────────────────────────────────────────────────────────────────

static mut MOUNTED: bool = false;
static mut FILE_COUNT: usize = 0;
static mut TOTAL_DATA_SECTORS: u32 = 0;

// ── Mount ────────────────────────────────────────────────────────────────────

pub fn mount() -> Result<(), &'static str> {
    unsafe {
        if MOUNTED {
            return Err("already mounted");
        }

        // Switch to drive 1 (primary slave = data disk) before any I/O
        ata::ATA_DRIVE = 1;

        // Read superblock (at ALFS_OFFSET_SECTORS + 0)
        let superblock = ata::read_sector(ALFS_OFFSET_SECTORS)
            .ok_or("failed to read superblock")?;

        // Check magic
        if superblock[0..4] != SUPERBLOCK_MAGIC {
            return Err("invalid ALFS magic");
        }

        // Check version
        let version = u16::from_le_bytes([superblock[4], superblock[5]]);
        if version != SUPERBLOCK_VERSION {
            return Err("unsupported ALFS version");
        }

        // Read metadata
        let file_count = u16::from_le_bytes([superblock[6], superblock[7]]) as usize;
        let total_data_sectors = u32::from_le_bytes([
            superblock[8], superblock[9], superblock[10], superblock[11],
        ]);

        // Load sector bitmap from superblock bytes 16..144 into SECTOR_BITMAP[0..]
        let bitmap_bytes = (MAX_DATA_SECTORS + 7) / 8;
        let copy_len = bitmap_bytes.min(128);
        let bitmap_slice = &raw mut SECTOR_BITMAP as *mut u8;
        for i in 0..copy_len {
            bitmap_slice.add(i).write_volatile(superblock[16 + i]);
        }

        FILE_COUNT = file_count;
        TOTAL_DATA_SECTORS = total_data_sectors;
        MOUNTED = true;
    }

    Ok(())
}

pub fn is_mounted() -> bool {
    unsafe { MOUNTED }
}

// ── Directory ────────────────────────────────────────────────────────────────

/// List all files in the filesystem.
pub fn list() -> Vec<FileInfo> {
    let mut entries = Vec::new();

    unsafe {
        let count = FILE_COUNT;
        if count == 0 {
            return entries;
        }

        for i in 0..count {
            if let Some(entry) = read_dir_entry(i) {
                entries.push(entry);
            }
        }
    }

    entries
}

/// Read a single directory entry by index.
fn read_dir_entry(index: usize) -> Option<FileInfo> {
    let sector_offset = index / ENTRIES_PER_SECTOR;
    let entry_offset = index % ENTRIES_PER_SECTOR;
    let byte_offset = entry_offset * DIR_ENTRY_SIZE;

    if sector_offset >= DIR_SECTORS as usize {
        return None;
    }

    let sector = ata::read_sector(
        ALFS_OFFSET_SECTORS + DIR_START_SECTOR + sector_offset as u32
    )?;

    // Check valid flag
    if sector[byte_offset + 41] != 0xFF {
        return None;
    }

    // Parse filename
    let name_end = sector[byte_offset..byte_offset + 32]
        .iter()
        .position(|&b| b == 0 || b == b' ')
        .unwrap_or(32);
    let name = core::str::from_utf8(&sector[byte_offset..byte_offset + name_end])
        .ok()?
        .to_string();

    // Parse fields
    let start_sector = u32::from_le_bytes([
        sector[byte_offset + 32],
        sector[byte_offset + 33],
        sector[byte_offset + 34],
        sector[byte_offset + 35],
    ]);
    let sector_count = u32::from_le_bytes([
        sector[byte_offset + 36],
        sector[byte_offset + 37],
        sector[byte_offset + 38],
        sector[byte_offset + 39],
    ]);
    let file_type = sector[byte_offset + 40];

    Some(FileInfo {
        name,
        start_sector,
        sector_count,
        file_type,
    })
}

// ── File reading ─────────────────────────────────────────────────────────────

/// Read an entire file into a byte vector.
pub fn read_file(name: &str) -> Option<Vec<u8>> {
    let info = find_file(name)?;
    let total_bytes = (info.sector_count as usize) * SECTOR_SIZE;
    let mut buf = vec![0u8; total_bytes];

    let count = info.sector_count as usize;
    let abs_start = ALFS_OFFSET_SECTORS + DATA_START_SECTOR + info.start_sector;
    ata::read_sectors(abs_start, count, &mut buf)?;

    Some(buf)
}

/// Read a file as a string.
pub fn read_file_string(name: &str) -> Option<String> {
    let bytes = read_file(name)?;
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[..len].to_vec()).ok()
}

/// Find a file by name.
pub fn find_file(name: &str) -> Option<FileInfo> {
    let entries = list();
    entries.into_iter().find(|e| e.name == name)
}

// ── File writing ─────────────────────────────────────────────────────────────

/// Write a file to disk. Creates if new, overwrites if exists.
/// Returns the number of sectors written.
pub fn write_file(name: &str, data: &[u8], file_type: u8) -> Result<usize, &'static str> {
    if !is_mounted() {
        return Err("ALFS not mounted");
    }
    if name.is_empty() || name.len() > 31 {
        return Err("invalid filename (must be 1-31 chars)");
    }
    if data.is_empty() {
        return Err("empty data");
    }

    let sectors_needed = ((data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE) as u32;

    // Check if file already exists
    if let Some(existing) = find_file(name) {
        // Free old sectors
        free_sectors(existing.start_sector, existing.sector_count);
        
        // Allocate new sectors
        let new_start = alloc_sectors(sectors_needed)
            .ok_or("out of disk space")?;
        
        // Write data
        let abs_start = ALFS_OFFSET_SECTORS + DATA_START_SECTOR + new_start;
        let padded_data = pad_to_sectors(data, sectors_needed as usize);
        ata::write_sectors(abs_start, sectors_needed as usize, &padded_data)
            .ok_or("disk write failed")?;

        // Update directory entry in place
        update_dir_entry(name, new_start, sectors_needed, file_type)
            .ok_or("failed to update directory")?;

        Ok(sectors_needed as usize)
    } else {
        // New file — find a free directory slot
        let slot = find_free_dir_slot()
            .ok_or("directory full (max 128 files)")?;

        // Allocate data sectors
        let start = alloc_sectors(sectors_needed)
            .ok_or("out of disk space")?;

        // Write data
        let abs_start = ALFS_OFFSET_SECTORS + DATA_START_SECTOR + start;
        let padded_data = pad_to_sectors(data, sectors_needed as usize);
        ata::write_sectors(abs_start, sectors_needed as usize, &padded_data)
            .ok_or("disk write failed")?;

        // Write directory entry
        write_dir_entry(slot, name, start, sectors_needed, file_type)
            .ok_or("failed to write directory entry")?;

        // Update file count
        unsafe {
            FILE_COUNT += 1;
        }

        // Update superblock
        update_superblock()
            .ok_or("failed to update superblock")?;

        Ok(sectors_needed as usize)
    }
}

/// Delete a file from disk.
pub fn delete_file(name: &str) -> Result<(), &'static str> {
    if !is_mounted() {
        return Err("ALFS not mounted");
    }

    let info = find_file(name).ok_or("file not found")?;
    
    // Free data sectors
    free_sectors(info.start_sector, info.sector_count);

    // Invalidate directory entry
    invalidate_dir_entry(name).ok_or("failed to invalidate directory entry")?;

    // Update file count
    unsafe {
        if FILE_COUNT > 0 {
            FILE_COUNT -= 1;
        }
    }

    // Update superblock
    update_superblock().ok_or("failed to update superblock")?;

    Ok(())
}

// ── Directory write helpers ──────────────────────────────────────────────────

fn find_free_dir_slot() -> Option<usize> {
    for i in 0..MAX_ENTRIES {
        let sector_offset = i / ENTRIES_PER_SECTOR;
        let entry_offset = i % ENTRIES_PER_SECTOR;
        let byte_offset = entry_offset * DIR_ENTRY_SIZE;

        let sector = ata::read_sector(
            ALFS_OFFSET_SECTORS + DIR_START_SECTOR + sector_offset as u32
        )?;

        if sector[byte_offset + 41] != 0xFF {
            return Some(i);
        }
    }
    None
}

fn write_dir_entry(
    index: usize,
    name: &str,
    start_sector: u32,
    sector_count: u32,
    file_type: u8,
) -> Option<()> {
    let sector_offset = index / ENTRIES_PER_SECTOR;
    let entry_offset = index % ENTRIES_PER_SECTOR;
    let byte_offset = entry_offset * DIR_ENTRY_SIZE;

    let mut sector = ata::read_sector(
        ALFS_OFFSET_SECTORS + DIR_START_SECTOR + sector_offset as u32
    )?;

    // Clear the entry area
    for i in 0..DIR_ENTRY_SIZE {
        sector[byte_offset + i] = 0;
    }

    // Write filename (null-terminated, padded to 32 bytes)
    let name_bytes = name.as_bytes();
    let copy_len = name_bytes.len().min(32);
    for i in 0..copy_len {
        sector[byte_offset + i] = name_bytes[i];
    }

    // Write fields
    sector[byte_offset + 32] = (start_sector & 0xFF) as u8;
    sector[byte_offset + 33] = ((start_sector >> 8) & 0xFF) as u8;
    sector[byte_offset + 34] = ((start_sector >> 16) & 0xFF) as u8;
    sector[byte_offset + 35] = ((start_sector >> 24) & 0xFF) as u8;

    sector[byte_offset + 36] = (sector_count & 0xFF) as u8;
    sector[byte_offset + 37] = ((sector_count >> 8) & 0xFF) as u8;
    sector[byte_offset + 38] = ((sector_count >> 16) & 0xFF) as u8;
    sector[byte_offset + 39] = ((sector_count >> 24) & 0xFF) as u8;

    sector[byte_offset + 40] = file_type;
    sector[byte_offset + 41] = 0xFF;  // Valid flag

    ata::write_sector(
        ALFS_OFFSET_SECTORS + DIR_START_SECTOR + sector_offset as u32,
        &sector,
    )
}

fn update_dir_entry(
    name: &str,
    start_sector: u32,
    sector_count: u32,
    file_type: u8,
) -> Option<()> {
    // Find existing entry
    for i in 0..MAX_ENTRIES {
        let sector_offset = i / ENTRIES_PER_SECTOR;
        let entry_offset = i % ENTRIES_PER_SECTOR;
        let byte_offset = entry_offset * DIR_ENTRY_SIZE;

        let sector = ata::read_sector(
            ALFS_OFFSET_SECTORS + DIR_START_SECTOR + sector_offset as u32
        )?;

        // Check if this entry matches
        if sector[byte_offset + 41] == 0xFF {
            let name_end = sector[byte_offset..byte_offset + 32]
                .iter()
                .position(|&b| b == 0 || b == b' ')
                .unwrap_or(32);
            let entry_name = core::str::from_utf8(
                &sector[byte_offset..byte_offset + name_end]
            ).ok()?;

            if entry_name == name {
                // Update in place
                return write_dir_entry(i, name, start_sector, sector_count, file_type);
            }
        }
    }
    None
}

fn invalidate_dir_entry(name: &str) -> Option<()> {
    for i in 0..MAX_ENTRIES {
        let sector_offset = i / ENTRIES_PER_SECTOR;
        let entry_offset = i % ENTRIES_PER_SECTOR;
        let byte_offset = entry_offset * DIR_ENTRY_SIZE;

        let mut sector = ata::read_sector(
            ALFS_OFFSET_SECTORS + DIR_START_SECTOR + sector_offset as u32
        )?;

        if sector[byte_offset + 41] == 0xFF {
            let name_end = sector[byte_offset..byte_offset + 32]
                .iter()
                .position(|&b| b == 0 || b == b' ')
                .unwrap_or(32);
            let entry_name = core::str::from_utf8(
                &sector[byte_offset..byte_offset + name_end]
            ).ok()?;

            if entry_name == name {
                sector[byte_offset + 41] = 0x00;  // Invalid
                return ata::write_sector(
                    ALFS_OFFSET_SECTORS + DIR_START_SECTOR + sector_offset as u32,
                    &sector,
                );
            }
        }
    }
    None
}

// ── Superblock update ────────────────────────────────────────────────────────

fn update_superblock() -> Option<()> {
    let mut superblock = ata::read_sector(ALFS_OFFSET_SECTORS)?;

    // Magic + version
    superblock[0..4].copy_from_slice(&SUPERBLOCK_MAGIC);
    superblock[4] = (SUPERBLOCK_VERSION & 0xFF) as u8;
    superblock[5] = (SUPERBLOCK_VERSION >> 8) as u8;

    // File count + total data sectors
    unsafe {
        superblock[6] = (FILE_COUNT & 0xFF) as u8;
        superblock[7] = (FILE_COUNT >> 8) as u8;
        superblock[8] = (TOTAL_DATA_SECTORS & 0xFF) as u8;
        superblock[9] = (TOTAL_DATA_SECTORS >> 8) as u8;
        superblock[10] = (TOTAL_DATA_SECTORS >> 16) as u8;
        superblock[11] = (TOTAL_DATA_SECTORS >> 24) as u8;

        // Sector bitmap (bytes 16..144)
        let bitmap_ptr = &SECTOR_BITMAP as *const _ as *const u8;
        for i in 0..128.min(superblock.len() - 16) {
            superblock[16 + i] = bitmap_ptr.add(i).read_volatile();
        }
    }

    ata::write_sector(ALFS_OFFSET_SECTORS, &superblock)
}

// ── Utilities ────────────────────────────────────────────────────────────────

fn pad_to_sectors(data: &[u8], sectors: usize) -> Vec<u8> {
    let total = sectors * SECTOR_SIZE;
    let mut buf = vec![0u8; total];
    let copy_len = data.len().min(total);
    buf[..copy_len].copy_from_slice(&data[..copy_len]);
    buf
}

/// Get filesystem info string.
pub fn info() -> String {
    let used_sectors = unsafe { TOTAL_DATA_SECTORS };
    let free_sectors = MAX_DATA_SECTORS as u32 - used_sectors;
    let file_count = unsafe { FILE_COUNT };
    format!("ALFS v{}: {} files, {} used / {} free sectors",
        SUPERBLOCK_VERSION, file_count, used_sectors, free_sectors)
}
