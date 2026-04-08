#!/usr/bin/env bash
#
# build_bootimage.sh — creates a bootable disk image for exoterikOS
#
# cargo bootimage is broken on rustc >= 1.90 (the -Z json-target-spec
# flag was removed). This script replicates what bootimage does manually.
#
set -euo pipefail
cd "$(dirname "$0")"

KERNEL_NAME="exoterik-os"
PROFILE="release"
TARGET="x86_64-unknown-none"
OUT_DIR="target/${TARGET}/${PROFILE}"
KERNEL="${OUT_DIR}/${KERNEL_NAME}"
BOOTIMAGE="${OUT_DIR}/bootimage-${KERNEL_NAME}.bin"
BOOTLOADER_TARGET="x86_64-bootloader.json"
BOOT_BUILD_DIR="target/bootloader-build"

echo "═══ Φ_c exoterikOS Bootimage Builder ═══"
echo ""

# ── 1: Build kernel ─────────────────────────────
echo "[1/4] Building kernel ELF..."
cargo build --profile "$PROFILE" 2>&1 | grep -E 'Compiling|Finished|error' || true
[ ! -f "$KERNEL" ] && { echo "ERROR: $KERNEL not found"; exit 1; }
echo "  ✓ $(stat -c%s "$KERNEL") bytes"

# ── 2: Create JSON target ───────────────────────
echo "[2/4] Creating bootloader target spec..."
cat > "$BOOTLOADER_TARGET" << 'SPEC'
{
    "llvm-target": "x86_64-unknown-none-gnu",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
    "linker-flavor": "ld.lld",
    "linker": "rust-lld",
    "pre-link-args": {"ld.lld": ["--script=linker.ld","--gc-sections"]},
    "target-endian": "little",
    "target-pointer-width": 64,
    "target-c-int-width": 32,
    "arch": "x86_64",
    "os": "none",
    "features": "-mmx,-sse,+soft-float",
    "disable-redzone": true,
    "panic-strategy": "abort",
    "executables": true,
    "relocation-model": "static",
    "rustc-abi": "x86-softfloat"
}
SPEC
echo "  ✓ $BOOTLOADER_TARGET"

# ── 3: Build bootloader binary (embeds kernel) ──
echo "[3/4] Building bootloader (embeds kernel)..."

BL_SRC=""
for f in ~/.cargo/registry/src/*/bootloader-0.9.*/Cargo.toml; do
    [ -f "$f" ] && { BL_SRC="$(dirname "$f")"; break; }
done
[ -z "$BL_SRC" ] && { echo "ERROR: bootloader 0.9.x not found"; exit 1; }

rm -rf "$BOOT_BUILD_DIR"

KERNEL="$(pwd)/$KERNEL" \
KERNEL_MANIFEST="$(pwd)/Cargo.toml" \
KERNEL_DIRECTORY="$(pwd)" \
CARGO_TARGET_DIR="$(pwd)/$BOOT_BUILD_DIR" \
cargo build \
    --manifest-path "$BL_SRC/Cargo.toml" \
    --features "binary,map_physical_memory" \
    --target "$(pwd)/$BOOTLOADER_TARGET" \
    --target-dir "$(pwd)/$BOOT_BUILD_DIR" \
    --release 2>&1 | grep -E 'Compiling|Finished|error' || true

# Cargo strips the .json extension when naming the output directory
BOOT_BIN="$BOOT_BUILD_DIR/${BOOTLOADER_TARGET%.json}/${PROFILE}/bootloader"
[ ! -f "$BOOT_BIN" ] && { echo "ERROR: $BOOT_BIN not found"; exit 1; }
echo "  ✓ $(stat -c%s "$BOOT_BIN") bytes"

cp "$BOOT_BIN" "$BOOTIMAGE"

# ── 4: Convert ELF to raw disk image ──────────────
echo "[4/5] Converting ELF to bootable disk image..."

DISKIMAGE="${OUT_DIR}/bootimage-${KERNEL_NAME}.img"

# Convert ELF to flat binary — objcopy extracts all LOAD segments in order,
# producing the raw MBR+stage2+kernel image that QEMU boots directly.
LLVM_OBJCOPY="$(rustup which llvm-objcopy 2>/dev/null || find ~/.rustup/toolchains/nightly-*/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-objcopy 2>/dev/null | head -1)"
"$LLVM_OBJCOPY" -I elf64-x86-64 -O binary "$BOOT_BIN" "$DISKIMAGE"
# Pad to 1MB for QEMU compatibility
dd if=/dev/zero bs=1 count=0 seek=$((1024*1024)) of="$DISKIMAGE" 2>/dev/null
echo "  ✓ $(stat -c%s "$DISKIMAGE") bytes (flat binary disk image)"

# ── 5: Build ALFS data disk ──────────────────────
echo "[5/6] Building ALFS data disk..."
ALFS_IMG="${OUT_DIR}/alfs.img"
if [ -d "programs" ] && ls programs/*.aleph 1>/dev/null 2>&1; then
    bash build_alfs.sh programs "$ALFS_IMG"
    # Append ALFS image to the end of the boot disk
    cat "$ALFS_IMG" >> "$DISKIMAGE"
    ALFS_SECTORS=$(( $(stat -c%s "$ALFS_IMG") / 512 ))
    DISK_SECTORS=$(( $(stat -c%s "$DISKIMAGE") / 512 ))
    echo "  ✓ ALFS appended at sector $((DISK_SECTORS - ALFS_SECTORS)) (${ALFS_SECTORS} sectors)"
else
    echo "  (skipped — no programs/ directory or no .aleph files)"
    # Create empty ALFS image so the kernel knows there's no filesystem
    rm -f "$ALFS_IMG"
fi

# ── 6: Report ────────────────────────────────────
echo "[6/6] Done."
echo ""
echo "Bootable image: $DISKIMAGE"
echo ""
echo "Run:"
echo "  qemu-system-x86_64 -drive format=raw,file=$(pwd)/$DISKIMAGE -display curses -no-reboot"
echo ""
