#!/usr/bin/env bash
#
# build_bootimage.sh — builds exoterikOS UEFI boot image
#
set -euo pipefail
cd "$(dirname "$0")"

PROFILE="release"
TARGET="x86_64-unknown-none"
KERNEL_NAME="exoterik-os"
OUT_DIR="target/${TARGET}/${PROFILE}"
KERNEL_ELF="${OUT_DIR}/${KERNEL_NAME}"
BOOT_DIR="target/uefi-boot"
ESP_DIR="${BOOT_DIR}/esp"
EFI_DIR="${ESP_DIR}/EFI/BOOT"
BOOTX64="${EFI_DIR}/BOOTX64.EFI"

echo "═══ Φ_c exoterikOS UEFI Bootimage Builder ═══"
echo ""

# ── 1: Build kernel ELF ─────────────────────────────
echo "[1/3] Building kernel ELF..."
cargo build --profile "$PROFILE" --target "$TARGET" 2>&1 | grep -E 'Compiling|Finished|error' || true
[ ! -f "$KERNEL_ELF" ] && { echo "ERROR: $KERNEL_ELF not found"; exit 1; }
echo "  ✓ $(stat -c%s "$KERNEL_ELF") bytes"

# ── 2: Build bootloader binary with kernel embedded ──
echo "[2/3] Building UEFI bootloader (embeds kernel)..."

# Find the bootloader source
BL_SRC=""
for f in ~/.cargo/registry/src/*/bootloader-x86_64-uefi-0.11.*/Cargo.toml; do
    [ -f "$f" ] && { BL_SRC="$(dirname "$f")"; break; }
done
[ -z "$BL_SRC" ] && { echo "ERROR: bootloader-x86_64-uefi not found. Run: cargo build --release first."; exit 1; }

rm -rf "${BOOT_DIR}/bootloader-build"
mkdir -p "${BOOT_DIR}/bootloader-build"

# The bootloader must be built with x86_64-unknown-uefi target (not x86_64-unknown-none)
# because it uses #[cfg(target_os = "uefi")] for its panic handler and UEFI entry point.
KERNEL="$(pwd)/$KERNEL_ELF" \
KERNEL_MANIFEST="$(pwd)/Cargo.toml" \
KERNEL_DIRECTORY="$(pwd)" \
cargo build \
    --manifest-path "$BL_SRC/Cargo.toml" \
    --release \
    --target x86_64-unknown-uefi \
    --target-dir "$(pwd)/${BOOT_DIR}/bootloader-build" \
    2>&1 | grep -E 'Compiling|Finished|error' || true

BOOT_ELF="${BOOT_DIR}/bootloader-build/x86_64-unknown-uefi/release/bootloader-x86_64-uefi.efi"
[ ! -f "$BOOT_ELF" ] && { echo "ERROR: $BOOT_ELF not found"; exit 1; }
echo "  ✓ $(stat -c%s "$BOOT_ELF") bytes"

# ── 3: Create EFI system partition structure ─────────
echo "[3/3] Creating EFI boot image..."
mkdir -p "$EFI_DIR"

# Copy the .efi file directly (it's already a UEFI executable)
cp "$BOOT_ELF" "$BOOTX64"
echo "  ✓ EFI binary: $(stat -c%s "$BOOTX64") bytes"

# Copy the kernel ELF — bootloader looks for "kernel-x86_64" by default
cp "$KERNEL_ELF" "${ESP_DIR}/kernel-x86_64"
echo "  ✓ Kernel ELF: $(stat -c%s "${ESP_DIR}/kernel-x86_64") bytes (as kernel-x86_64)"

# Create a FAT32 disk image with the EFI system partition
IMG="${OUT_DIR}/bootimage-${KERNEL_NAME}.img"
dd if=/dev/zero of="$IMG" bs=1M count=64 2>/dev/null
mkfs.vfat -F 32 "$IMG" >/dev/null 2>&1 || {
    echo "WARNING: mkfs.vfat failed, creating raw image instead"
    cp "$BOOTX64" "${OUT_DIR}/bootimage-${KERNEL_NAME}.efi"
    echo "  ✓ EFI binary (standalone): ${OUT_DIR}/bootimage-${KERNEL_NAME}.efi"
}

# Copy files into the FAT image using mcopy
if command -v mcopy &>/dev/null; then
    mcopy -i "$IMG" -s "$ESP_DIR/EFI" "::EFI" 2>/dev/null || true
    mcopy -i "$IMG" -o "${ESP_DIR}/kernel-x86_64" "::" 2>/dev/null || true
    echo "  ✓ Files copied to disk image"
fi

echo ""
echo "═══ Build Complete ═══"
echo ""
echo "Kernel ELF:      $KERNEL_ELF"
echo "Bootloader ELF:  $BOOT_ELF"
echo "EFI binary:      $BOOTX64"
echo "Disk image:      $IMG"
echo ""
echo "Run:"
echo "  ./run.sh          # graphical mode with UEFI framebuffer"
echo "  ./run.sh --serial # serial-only mode"
echo ""
