#!/usr/bin/env python3
"""Find MIPS lui/addiu(+lw/sw/etc) pairs that materialise a given address.

  xref.py <elf> <lo> [hi]      # addresses in hex; hi defaults to lo+4

Scans every executable segment of the ELF.  Register tracking is crude (a lui
is remembered until the next lui to the same register), so expect the odd
false positive -- confirm hits with dis.sh.
"""
import struct, sys
from elfmap import Img

def scan(im):
    """Yields (pc, formed_addr, op, lui_pc, rs, rt) for each lui+imm pair."""
    for seg_va, seg_len in im.exec_ranges():
        code = im.read(seg_va, seg_len)
        n = len(code) // 4
        words = struct.unpack(f"<{n}I", code[:n*4])
        # track last lui per register
        lui = {}
        for i, w in enumerate(words):
            pc = seg_va + i*4
            op = w >> 26
            rs = (w >> 21) & 31
            rt = (w >> 16) & 31
            imm = w & 0xffff
            simm = imm - 0x10000 if imm & 0x8000 else imm
            if op == 0x0f:  # LUI
                lui[rt] = (imm << 16, pc)
                continue
            # addiu / ori / lw / sw / lh / lb / lbu / sb / sh / lwc1 / swc1
            if op in (0x09, 0x0d, 0x23, 0x2b, 0x21, 0x20, 0x24, 0x28, 0x29,
                      0x31, 0x39, 0x25, 0x37, 0x3f):
                if rs in lui:
                    base, lpc = lui[rs]
                    addr = (base + simm) & 0xffffffff if op != 0x0d else (base | imm)
                    yield pc, addr, op, lpc, rs, rt

if __name__ == "__main__":
    if len(sys.argv) < 3:
        sys.exit(__doc__.strip())
    im = Img(sys.argv[1])
    lo = int(sys.argv[2], 16)
    hi = int(sys.argv[3], 16) if len(sys.argv) > 3 else lo + 4
    for pc, addr, op, lpc, rs, rt in scan(im):
        if lo <= addr < hi:
            print(f"pc=0x{pc:08x} lui@0x{lpc:08x} -> 0x{addr:08x} "
                  f"op=0x{op:02x} rs=${rs} rt=${rt}")
