#!/usr/bin/env python3
"""Map PS2 virtual addresses <-> file offsets in a boot ELF.

Library:  Img("/path/to/SLUS_XXX.XX")  -- .off()/.va_of_off()/.read()/.cstr()

CLI:
  elfmap.py <elf>              # list PT_LOAD segments
  elfmap.py <elf> <va>         # file offset of a VA
  elfmap.py <elf> --seg <va>   # "0x<seg_va> 0x<seg_off>" of the containing
                               # segment (consumed by dis.sh)
"""
import struct, sys

class Img:
    def __init__(self, path):
        self.path = path
        self.b = open(path, "rb").read()
        # parse program headers
        phoff, phentsize, phnum = struct.unpack_from("<I", self.b, 0x1c)[0], \
            struct.unpack_from("<H", self.b, 0x2a)[0], struct.unpack_from("<H", self.b, 0x2c)[0]
        self.segs = []
        for i in range(phnum):
            o = phoff + i*phentsize
            p_type, p_off, p_va, p_pa, p_filesz, p_memsz, p_flags = \
                struct.unpack_from("<7I", self.b, o)
            if p_type == 1:
                self.segs.append((p_va, p_off, p_filesz, p_memsz, p_flags))

    def off(self, va):
        for p_va, p_off, p_filesz, _, _ in self.segs:
            if p_va <= va < p_va + p_filesz:
                return va - p_va + p_off
        return None

    def seg_of(self, va):
        for s in self.segs:
            if s[0] <= va < s[0] + s[2]:
                return s
        return None

    def exec_ranges(self):
        """(va, filesz) of every executable segment (PF_X set)."""
        return [(s[0], s[2]) for s in self.segs if s[4] & 1]

    def read(self, va, n):
        o = self.off(va)
        return self.b[o:o+n] if o is not None else None

    def u32(self, va):
        return struct.unpack_from("<I", self.read(va, 4))[0]

    def u16(self, va):
        return struct.unpack_from("<H", self.read(va, 2))[0]

    def cstr(self, va, maxn=200):
        o = self.off(va)
        if o is None: return None
        e = self.b.index(b"\0", o)
        return self.b[o:e].decode("latin-1")

    def va_of_off(self, off):
        for p_va, p_off, p_filesz, _, _ in self.segs:
            if p_off <= off < p_off + p_filesz:
                return off - p_off + p_va
        return None

if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__.strip())
    im = Img(sys.argv[1])
    if len(sys.argv) == 2:
        for s in im.segs:
            x = "x" if s[4] & 1 else "-"
            print(f"seg va=0x{s[0]:08x} off=0x{s[1]:06x} filesz=0x{s[2]:x} "
                  f"memsz=0x{s[3]:x} {x}")
    elif sys.argv[2] == "--seg":
        s = im.seg_of(int(sys.argv[3], 16))
        if s is None:
            sys.exit("va not file-backed")
        print(f"0x{s[0]:x} 0x{s[1]:x}")
    else:
        va = int(sys.argv[2], 16)
        o = im.off(va)
        if o is None:
            sys.exit("va not file-backed")
        print(f"0x{o:x}")
