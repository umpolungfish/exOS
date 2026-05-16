#!/usr/bin/env bash
# run.sh — boot exOS with UEFI framebuffer + serial output
set -euo pipefail
cd "$(dirname "$0")"

# build_bootimage.sh produces a FAT32 disk image
IMG="target/x86_64-unknown-none/release/bootimage-exoterik-os.img"

if [ ! -f "$IMG" ]; then
    echo "No bootimage found — run: bash build_bootimage.sh"
    exit 1
fi

# OVMF firmware paths (common locations)
OVMF_CODE="/usr/share/OVMF/OVMF_CODE.fd"
OVMF_VARS="/usr/share/OVMF/OVMF_VARS.fd"

# Try alternate paths if the default doesn't exist
if [ ! -f "$OVMF_CODE" ]; then
    OVMF_CODE="/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"
    OVMF_VARS="/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"
fi
if [ ! -f "$OVMF_CODE" ]; then
    OVMF_CODE="/usr/share/qemu/edk2-x86_64-code.fd"
    OVMF_VARS="/usr/share/qemu/edk2-x86_64-vars.fd"
fi
if [ ! -f "$OVMF_CODE" ]; then
    # Try to find it anywhere
    OVMF_CODE=$(find /usr/share -name "OVMF_CODE.fd" -o -name "OVMF_CODE_*.fd" -o -name "edk2-x86_64-code.fd" 2>/dev/null | head -1)
fi

if [ -z "$OVMF_CODE" ] || [ ! -f "$OVMF_CODE" ]; then
    echo "ERROR: OVMF firmware not found. Install it with:"
    echo "  Ubuntu/Debian: sudo apt install ovmf"
    echo "  Arch:          sudo pacman -S edk2-ovmf"
    echo "  Fedora:        sudo dnf install edk2-ovmf"
    exit 1
fi

# OVMF_VARS.fd must be writable — copy it locally if we can't write the original.
LOCAL_VARS="$(pwd)/.ovmf_vars.fd"
if [ ! -w "$OVMF_VARS" ]; then
    cp -f "$OVMF_VARS" "$LOCAL_VARS" 2>/dev/null || true
    OVMF_VARS="$LOCAL_VARS"
fi

# ALFS data disk (ATA drive 1, primary slave).
# ALFS superblock lives at sector 0 of this image.
ALFS_IMG="$(pwd)/alfs.img"
if [ ! -f "$ALFS_IMG" ]; then
    echo "Creating fresh ALFS data disk (32 MB)..."
    dd if=/dev/zero of="$ALFS_IMG" bs=1M count=32 2>/dev/null
    # Write ALFS superblock at sector 0: magic "ALFS", version=1, counts=0, bitmap=0
    printf 'ALFS\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00' | \
        dd of="$ALFS_IMG" bs=1 seek=0 count=16 conv=notrunc 2>/dev/null
    echo "  ✓ ALFS disk: $ALFS_IMG"
fi

# Use serial mode (nographic) or graphical mode
if [ "${1:-}" = "--serial" ]; then
    echo "3xOS booting in SERIAL mode... (Ctrl-A then X to quit)"
    qemu-system-x86_64 \
        -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
        -drive if=pflash,format=raw,file="$OVMF_VARS" \
        -drive format=raw,file="$(pwd)/$IMG" \
        -drive format=raw,file="$ALFS_IMG",if=ide,index=1,media=disk \
        -m 128M \
        -display none \
        -no-reboot \
        -serial stdio
else
    echo "3xOS booting in GRAPHICAL mode with UEFI framebuffer..."
    qemu-system-x86_64 \
        -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
        -drive if=pflash,format=raw,file="$OVMF_VARS" \
        -drive format=raw,file="$(pwd)/$IMG" \
        -drive format=raw,file="$ALFS_IMG",if=ide,index=1,media=disk \
        -m 128M \
        -no-reboot \
        -vga virtio \
        -serial pty
fi
