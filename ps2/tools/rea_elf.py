#!/usr/bin/env python3
"""Make a copy of a PS2 ELF that REA will accept, for Ghidra analysis.

REA screens MIPS ELFs by e_flags before it ever starts Ghidra and only admits
MIPS32/MIPS32R2 o32-shaped headers. PS2 boot ELFs declare MIPS-III, the 5900
machine variant and EABI64 (e.g. 0x20924001), so they are rejected outright.
Nothing but the flags word is changed: segments, addresses and bytes are
identical, so every address REA reports applies 1:1 to the original ELF.
Ghidra itself ignores the flags because REA_GHIDRA_LANGUAGE_ID pins the
Emotion Engine language (r5900:LE:32:default) from ghidra-emotionengine.

Only ever feed the copy to REA. Splice (resplice) and repack the original.

CLI:
  rea_elf.py <elf> [out]       # writes <elf>.rea.elf by default, prints path
"""
import struct, sys

# EF_MIPS_ARCH_32 | EF_MIPS_NOREORDER: the "recordless legacy" shape REA admits
# (no ABI bits, no .MIPS.abiflags - PS2 ELFs predate that section anyway)
REA_FLAGS = 0x50000001

def normalise(src, dst):
    b = bytearray(open(src, "rb").read())
    if b[:4] != b"\x7fELF" or b[4] != 1 or b[5] != 1:
        sys.exit(f"{src}: not a 32-bit little-endian ELF")
    e_machine = struct.unpack_from("<H", b, 0x12)[0]
    if e_machine != 8:
        sys.exit(f"{src}: e_machine {e_machine} is not MIPS")
    old = struct.unpack_from("<I", b, 0x24)[0]
    struct.pack_into("<I", b, 0x24, REA_FLAGS)
    open(dst, "wb").write(b)
    return old

if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__.strip())
    src = sys.argv[1]
    dst = sys.argv[2] if len(sys.argv) > 2 else src + ".rea.elf"
    old = normalise(src, dst)
    print(f"{dst}", file=sys.stdout)
    print(f"e_flags 0x{old:08x} -> 0x{REA_FLAGS:08x}", file=sys.stderr)
