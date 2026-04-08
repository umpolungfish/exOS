#!/usr/bin/env bash
# run.sh — boot vOS with serial output filling the terminal (no curses box)
set -euo pipefail
cd "$(dirname "$0")"

IMG="target/x86_64-unknown-none/release/bootimage-exoterik-os.img"

if [ ! -f "$IMG" ]; then
    echo "No image found — run build_bootimage.sh first"
    exit 1
fi

# -nographic: serial → stdio, no separate window, fills the terminal
# Ctrl-A X to quit QEMU
echo "vOS booting... (Ctrl-A then X to quit)"
qemu-system-x86_64 \
    -drive format=raw,file="$(pwd)/$IMG" \
    -nographic \
    -no-reboot
