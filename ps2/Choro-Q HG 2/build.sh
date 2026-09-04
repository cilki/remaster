#!/usr/bin/env bash
# Build shim -> splice into boot ELF -> repack ISO.
set -euo pipefail
SKILL=/root/.claude/skills/remaster-ps2
cd /workspace/shim
cargo build --release -Z build-std=core,alloc -Z json-target-spec \
    --target "$SKILL/mipsel-ps2-elf.json" 2>&1 | grep -Ev '^\s*(Compiling|Finished|Downloaded|Updating|Downloading)' || true
/usr/local/bin/resplice --inject-base 0x1c0000 \
    /workspace/orig/SLUS_203.98 \
    /workspace/shim/target/mipsel-ps2-elf/release/libshim.rlib \
    /workspace/extracted/SLUS_203.98
ls -la /workspace/extracted/SLUS_203.98
if [ "${1:-}" = "iso" ]; then
  repack_iso.py "/workspace/Road Trip.iso" /workspace/extracted /workspace/remaster.iso
fi
