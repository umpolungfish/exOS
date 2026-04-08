#!/usr/bin/env bash
#
# build_alfs.sh — creates an ALFS disk image from .aleph program files.
#
# Usage: ./build_alfs.sh programs/ target/alfs.img
#
# ALFS format:
#   Sector 0    : Superblock (magic "ALFS", version, file count, total sectors)
#   Sectors 1-16: Directory entries (8 per sector, 64 bytes each)
#   Sectors 17+ : File data
#

set -euo pipefail

if [ $# -lt 2 ]; then
    echo "Usage: $0 <programs-dir> <output.img>"
    exit 1
fi

PROG_DIR="$1"
OUTPUT="$2"

SECTOR_SIZE=512
DIR_START=1
DIR_SECTORS=16
DIR_ENTRIES=$((DIR_SECTORS * 8))  # 128
DATA_START=17

# Count files
FILE_COUNT=0
for f in "$PROG_DIR"/*.aleph; do
    [ -f "$f" ] && FILE_COUNT=$((FILE_COUNT + 1))
done

if [ "$FILE_COUNT" -eq 0 ]; then
    echo "No .aleph files found in $PROG_DIR"
    exit 1
fi

echo "Building ALFS disk: $FILE_COUNT files -> $OUTPUT"

# Calculate total size
DATA_SIZE=0
declare -a FILE_NAMES=()
declare -a FILE_SIZES=()
declare -a FILE_STARTS=()

NEXT_DATA_SECTOR=$DATA_START

for f in "$PROG_DIR"/*.aleph; do
    [ -f "$f" ] || continue
    BASENAME=$(basename "$f")
    SIZE=$(stat -c%s "$f")
    SECTORS=$(( (SIZE + SECTOR_SIZE - 1) / SECTOR_SIZE ))
    [ "$SECTORS" -eq 0 ] && SECTORS=1

    FILE_NAMES+=("$BASENAME")
    FILE_SIZES+=("$SECTORS")
    FILE_STARTS+=("$NEXT_DATA_SECTOR")
    DATA_SIZE=$((DATA_SIZE + SECTORS * SECTOR_SIZE))

    NEXT_DATA_SECTOR=$((NEXT_DATA_SECTOR + SECTORS))
done

TOTAL_SECTORS=$((NEXT_DATA_SECTOR))
TOTAL_BYTES=$((TOTAL_SECTORS * SECTOR_SIZE))

echo "  Total sectors: $TOTAL_SECTORS ($TOTAL_BYTES bytes)"

# Create the disk image (padded to total size)
dd if=/dev/zero of="$OUTPUT" bs=$SECTOR_SIZE count=$TOTAL_SECTORS 2>/dev/null

# ── Write superblock (sector 0) ──
# Magic: ALFS (4 bytes) + version (2 bytes LE) + file_count (2 bytes LE)
# + total_sectors (4 bytes LE) + reserved (502 bytes)
{
    printf 'ALFS'                                    # magic
    printf '\x01\x00'                                # version = 1 (LE)
    printf "\\x$(printf '%02x' $FILE_COUNT)\\x00"    # file_count (LE)
    printf "\\x$(printf '%02x' $((TOTAL_SECTORS & 0xFF)))"
    printf "\\x$(printf '%02x' $(((TOTAL_SECTORS >> 8) & 0xFF)))"
    printf "\\x$(printf '%02x' $(((TOTAL_SECTORS >> 16) & 0xFF)))"
    printf "\\x$(printf '%02x' $(((TOTAL_SECTORS >> 24) & 0xFF)))"
    # Pad to 512 bytes (502 bytes of zeros)
    dd if=/dev/zero bs=1 count=502 2>/dev/null
} | dd of="$OUTPUT" bs=1 seek=0 conv=notrunc 2>/dev/null

# ── Write directory entries ──
# Each entry: name (32 bytes, null-padded) + start_sector (4 LE)
#           + sector_count (4 LE) + file_type (1 byte) + reserved (23 bytes)
# 8 entries per sector

for i in $(seq 0 $((FILE_COUNT - 1))); do
    NAME="${FILE_NAMES[$i]}"
    START="${FILE_STARTS[$i]}"
    COUNT="${FILE_SIZES[$i]}"

    ENTRY_OFFSET=$(( (DIR_START * SECTOR_SIZE) + (i * 64) ))

    # Build entry
    {
        # Name: 32 bytes null-padded
        printf '%-32s' "$NAME" | head -c 32
        # start_sector: 4 bytes LE
        printf "\\x$(printf '%02x' $((START & 0xFF)))"
        printf "\\x$(printf '%02x' $(((START >> 8) & 0xFF)))"
        printf "\\x$(printf '%02x' $(((START >> 16) & 0xFF)))"
        printf "\\x$(printf '%02x' $(((START >> 24) & 0xFF)))"
        # sector_count: 4 bytes LE
        printf "\\x$(printf '%02x' $((COUNT & 0xFF)))"
        printf "\\x$(printf '%02x' $(((COUNT >> 8) & 0xFF)))"
        printf "\\x$(printf '%02x' $(((COUNT >> 16) & 0xFF)))"
        printf "\\x$(printf '%02x' $(((COUNT >> 24) & 0xFF)))"
        # file_type: 1 (aleph program)
        printf '\x01'
        # reserved: 23 bytes
        dd if=/dev/zero bs=1 count=23 2>/dev/null
    } | dd of="$OUTPUT" bs=1 seek=$ENTRY_OFFSET conv=notrunc 2>/dev/null
done

# ── Write file data ──
for i in $(seq 0 $((FILE_COUNT - 1))); do
    START="${FILE_STARTS[$i]}"
    FILEPATH="$PROG_DIR/${FILE_NAMES[$i]}"
    DATA_OFFSET=$((START * SECTOR_SIZE))

    dd if="$FILEPATH" of="$OUTPUT" bs=1 seek=$DATA_OFFSET conv=notrunc 2>/dev/null
done

echo "  Files:"
for i in $(seq 0 $((FILE_COUNT - 1))); do
    printf "    %-20s sector %-6d %d sectors\n" \
        "${FILE_NAMES[$i]}" "${FILE_STARTS[$i]}" "${FILE_SIZES[$i]}"
done
echo ""
echo "Done: $OUTPUT"
