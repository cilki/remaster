#!/usr/bin/env bash
# dis.sh <elf> <va> [count]  -- disassemble N instructions of a PS2 ELF at a VA
# The VA->offset mapping comes from the ELF's own program headers via elfmap.py.
f=$1; va=$((${2})); n=${3:-24}
read -r segva segoff < <("$(dirname "$0")/elfmap.py" "$f" --seg "$(printf 0x%x "$va")")
mipsel-objdump -D -b binary -m mips:5900 -EL \
  --adjust-vma=$((segva - segoff)) \
  --start-address=$((va)) --stop-address=$((va + n*4)) \
  "$f" 2>/dev/null | sed -n '/^ /p'
