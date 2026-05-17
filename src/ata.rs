//! ATA/IDE PIO mode driver — reads sectors from the primary master disk.
//!
//! Uses LBA28 mode on the primary IDE channel (ports 0x1F0-0x1F7).
//! QEMU exposes the boot disk as primary master by default.

use core::hint::spin_loop;
use core::sync::atomic::{AtomicU8, Ordering};
use x86_64::instructions::port::Port;

const ATA_PRIMARY_DATA:       u16 = 0x1F0;  // Data port (16-bit)
#[allow(dead_code)]
const ATA_PRIMARY_ERROR:      u16 = 0x1F1;  // Error
const ATA_PRIMARY_SECCOUNT:   u16 = 0x1F2;  // Sector count
const ATA_PRIMARY_LBA_LOW:    u16 = 0x1F3;  // LBA bits 0-7
const ATA_PRIMARY_LBA_MID:    u16 = 0x1F4;  // LBA bits 8-15
const ATA_PRIMARY_LBA_HIGH:   u16 = 0x1F5;  // LBA bits 16-23
const ATA_PRIMARY_DRIVE:      u16 = 0x1F6;  // Drive select + LBA bits 24-27
const ATA_PRIMARY_STATUS:     u16 = 0x1F7;  // Status (read) / Command (write)
#[allow(dead_code)]
const ATA_PRIMARY_CONTROL:    u16 = 0x3F6;  // Control (for reset)

const ATA_STATUS_BSY: u8 = 0x80;
const ATA_STATUS_DRQ: u8 = 0x08;
const ATA_STATUS_ERR: u8 = 0x01;
const ATA_CMD_READ:  u8 = 0x20;
const ATA_CMD_WRITE: u8 = 0x30;

const SECTOR_SIZE: usize = 512;

/// Drive selector: 0 = primary master, 1 = primary slave.
pub static ATA_DRIVE: AtomicU8 = AtomicU8::new(0);

/// Base drive selector byte (0xE0 = drive 0, 0xF0 = drive 1).
#[inline]
fn drive_select(lba: u32) -> u8 {
    let drv = ATA_DRIVE.load(Ordering::Relaxed);
    0xE0 | ((drv & 1) << 4) | ((lba >> 24) & 0x0F) as u8
}

/// Read one 512-byte sector from the configured ATA drive.
/// Returns the sector contents, or None on error.
pub fn read_sector(lba: u32) -> Option<[u8; SECTOR_SIZE]> {
    let drive_byte = drive_select(lba);
    unsafe {
        Port::<u8>::new(ATA_PRIMARY_DRIVE).write(drive_byte);
        // Wait for drive to be ready
        if !poll_ready() { return None; }

        // Send parameters
        Port::<u8>::new(ATA_PRIMARY_SECCOUNT).write(1);  // 1 sector
        Port::<u8>::new(ATA_PRIMARY_LBA_LOW).write((lba & 0xFF) as u8);
        Port::<u8>::new(ATA_PRIMARY_LBA_MID).write(((lba >> 8) & 0xFF) as u8);
        Port::<u8>::new(ATA_PRIMARY_LBA_HIGH).write(((lba >> 16) & 0xFF) as u8);

        // Send read command
        Port::<u8>::new(ATA_PRIMARY_STATUS).write(ATA_CMD_READ);

        // Wait for data ready
        if !poll_ready() { return None; }

        // Wait for DRQ bit (data request)
        if !poll_drq() { return None; }

        // Read 256 16-bit words
        let mut buf = [0u8; SECTOR_SIZE];
        let mut data_port = Port::<u16>::new(ATA_PRIMARY_DATA);
        for i in 0..256 {
            let word = data_port.read();
            buf[i * 2] = (word & 0xFF) as u8;
            buf[i * 2 + 1] = (word >> 8) as u8;
        }

        Some(buf)
    }
}

/// Read multiple sectors into a buffer. Buffer must be sector_size-aligned.
pub fn read_sectors(start_lba: u32, count: usize, out: &mut [u8]) -> Option<()> {
    if out.len() < count * SECTOR_SIZE {
        return None;
    }

    let drive_byte = drive_select(start_lba);
    unsafe {
        Port::<u8>::new(ATA_PRIMARY_DRIVE).write(drive_byte);

        if !poll_ready() { return None; }

        for i in 0..count {
            let lba = start_lba + i as u32;
            Port::<u8>::new(ATA_PRIMARY_SECCOUNT).write(1);
            Port::<u8>::new(ATA_PRIMARY_LBA_LOW).write((lba & 0xFF) as u8);
            Port::<u8>::new(ATA_PRIMARY_LBA_MID).write(((lba >> 8) & 0xFF) as u8);
            Port::<u8>::new(ATA_PRIMARY_LBA_HIGH).write(((lba >> 16) & 0xFF) as u8);
            Port::<u8>::new(ATA_PRIMARY_STATUS).write(ATA_CMD_READ);

            if !poll_ready() { return None; }
            if !poll_drq() { return None; }

            let offset = i * SECTOR_SIZE;
            let mut data_port = Port::<u16>::new(ATA_PRIMARY_DATA);
            for j in 0..256 {
                let word = data_port.read();
                out[offset + j * 2] = (word & 0xFF) as u8;
                out[offset + j * 2 + 1] = (word >> 8) as u8;
            }
        }
    }

    Some(())
}
/// Write one 512-byte sector to the configured ATA drive.
/// Returns None on error.
pub fn write_sector(lba: u32, data: &[u8; SECTOR_SIZE]) -> Option<()> {
    let drive_byte = drive_select(lba);
    unsafe {
        Port::<u8>::new(ATA_PRIMARY_DRIVE).write(drive_byte);

        if !poll_ready() { return None; }

        // Send parameters
        Port::<u8>::new(ATA_PRIMARY_SECCOUNT).write(1);
        Port::<u8>::new(ATA_PRIMARY_LBA_LOW).write((lba & 0xFF) as u8);
        Port::<u8>::new(ATA_PRIMARY_LBA_MID).write(((lba >> 8) & 0xFF) as u8);
        Port::<u8>::new(ATA_PRIMARY_LBA_HIGH).write(((lba >> 16) & 0xFF) as u8);

        // Send write command
        Port::<u8>::new(ATA_PRIMARY_STATUS).write(ATA_CMD_WRITE);

        // Wait for DRQ (drive ready to accept data)
        if !poll_drq() { return None; }

        // Write 256 16-bit words
        let mut data_port = Port::<u16>::new(ATA_PRIMARY_DATA);
        for i in 0..256 {
            let word = (data[i * 2 + 1] as u16) << 8 | (data[i * 2] as u16);
            data_port.write(word);
        }

        // Wait for command completion
        if !poll_ready() { return None; }
        if poll_error() { return None; }

        Some(())
    }
}

/// Write multiple sectors to the disk.
pub fn write_sectors(start_lba: u32, count: usize, data: &[u8]) -> Option<()> {
    if data.len() < count * SECTOR_SIZE {
        return None;
    }

    let drive_byte = drive_select(start_lba);
    unsafe {
        Port::<u8>::new(ATA_PRIMARY_DRIVE).write(drive_byte);

        if !poll_ready() { return None; }

        for i in 0..count {
            let lba = start_lba + i as u32;
            Port::<u8>::new(ATA_PRIMARY_SECCOUNT).write(1);
            Port::<u8>::new(ATA_PRIMARY_LBA_LOW).write((lba & 0xFF) as u8);
            Port::<u8>::new(ATA_PRIMARY_LBA_MID).write(((lba >> 8) & 0xFF) as u8);
            Port::<u8>::new(ATA_PRIMARY_LBA_HIGH).write(((lba >> 16) & 0xFF) as u8);
            Port::<u8>::new(ATA_PRIMARY_STATUS).write(ATA_CMD_WRITE);

            if !poll_drq() { return None; }

            let offset = i * SECTOR_SIZE;
            let mut data_port = Port::<u16>::new(ATA_PRIMARY_DATA);
            for j in 0..256 {
                let word = (data[offset + j * 2 + 1] as u16) << 8
                         | (data[offset + j * 2] as u16);
                data_port.write(word);
            }

            if !poll_ready() { return None; }
            if poll_error() { return None; }
        }
    }

    Some(())
}

/// Poll until BSY clears and DRQ sets.
fn poll_drq() -> bool {
    let mut status_port = Port::<u8>::new(ATA_PRIMARY_STATUS);
    for _ in 0..100_000 {
        let status = unsafe { status_port.read() };
        if (status & ATA_STATUS_BSY) == 0 && (status & ATA_STATUS_DRQ) != 0 {
            return true;
        }
        if (status & ATA_STATUS_ERR) != 0 {
            return false;
        }
        spin_loop();
    }
    false
}

/// Poll until BSY clears.
fn poll_ready() -> bool {
    let mut status_port = Port::<u8>::new(ATA_PRIMARY_STATUS);
    for _ in 0..100_000 {
        let status = unsafe { status_port.read() };
        if (status & ATA_STATUS_BSY) == 0 {
            return true;
        }
        spin_loop();
    }
    false
}

/// Check if error bit is set.
fn poll_error() -> bool {
    let status = unsafe { Port::<u8>::new(ATA_PRIMARY_STATUS).read() };
    (status & ATA_STATUS_ERR) != 0
}
