#!/usr/bin/env python3
"""Reader/writer for .GSL files (GS upload lists), as shipped by e.g. Road Trip.

A .GSL is a flat chain of records.  Each record is a 0x70-byte header followed
by the payload the header describes.  The header is a GS A+D packet -- four
128-bit (data, register) pairs -- so the fields we care about sit in the GS
register bitfields at:

    +0x20 BITBLTBUF   +0x30 TRXPOS   +0x40 TRXREG   +0x50 TRXDIR

The authoring tool ran on Windows and left uninitialised memory in every bit
the GS ignores, so leftover 0x0012xxxx stack and 0x0040xxxx image-base pointers
show up all over the header.  Masking each field to its real width is what
makes the numbers sane again; never compare raw dwords.
"""
import struct, sys

HDR = 0x70
PSM_NAME = {0: "CT32", 1: "CT24", 2: "CT16", 10: "CT16S",
            19: "T8", 20: "T4", 27: "T8H", 36: "T4HL", 44: "T4HH"}
BITS_PER_PIXEL = {0: 32, 1: 24, 2: 16, 10: 16, 19: 8, 20: 4}


class Record:
    __slots__ = ("off", "hdr", "data", "dbp", "dbw", "dpsm", "sbp", "sbw",
                 "psm", "dsax", "dsay", "w", "h", "xdir")

    def __init__(self, off, hdr, data):
        self.off, self.hdr, self.data = off, hdr, data
        blt = struct.unpack_from("<Q", hdr, 0x20)[0]
        pos = struct.unpack_from("<Q", hdr, 0x30)[0]
        reg = struct.unpack_from("<Q", hdr, 0x40)[0]
        dir_ = struct.unpack_from("<Q", hdr, 0x50)[0]
        self.dbp = blt & 0x3FFF
        self.dbw = (blt >> 16) & 0x3F
        self.dpsm = (blt >> 24) & 0x3F
        self.sbp = (blt >> 32) & 0x3FFF
        self.sbw = (blt >> 48) & 0x3F
        # The *source* PSM is the honest one: it describes the bytes in the
        # file.  DPSM is whatever the destination happened to be when the tool
        # dumped the packet and is CT32 on practically every record.
        self.psm = (blt >> 56) & 0x3F
        self.dsax = (pos >> 32) & 0x7FF
        self.dsay = (pos >> 48) & 0x7FF
        self.w = reg & 0xFFF
        self.h = (reg >> 32) & 0xFFF
        self.xdir = dir_ & 3

    @property
    def bpp(self):
        return BITS_PER_PIXEL[self.psm]

    @property
    def nbytes(self):
        """Exact payload length the header describes, before padding."""
        return self.w * self.h * self.bpp // 8

    @property
    def padded(self):
        """Payload length as stored: records are padded to a 16-byte qword."""
        return (self.nbytes + 15) & ~15

    @property
    def is_clut(self):
        """CLUT records are the small direct-colour blocks between images."""
        return self.psm in (0, 2) and self.w * self.h <= 256

    def __repr__(self):
        return (f"<rec @{self.off:#07x} {self.w}x{self.h} {self.bpp}bpp "
                f"{PSM_NAME.get(self.psm, self.psm)} dbp={self.dbp:#x} "
                f"dsa=({self.dsax},{self.dsay}) len={len(self.data)}>")


def signatures(buf):
    """Offsets of every record header in the buffer.

    A header is identified by its four A+D register selectors: the u32 at
    +0x28, +0x38, +0x48 and +0x58 must be exactly 0x50, 0x51, 0x52, 0x53.
    Four exact dwords is a strong enough signature that the tool's trailing
    garbage has never produced a false positive across Road Trip's 60 .GSLs.
    """
    out = []
    for o in range(0, len(buf) - HDR + 1, 16):
        if all(struct.unpack_from("<I", buf, o + 0x28 + 0x10 * i)[0] == 0x50 + i
               for i in range(4)):
            out.append(o)
    return out


def parse(buf):
    """Split a .GSL into records.  Returns (records, trailer_bytes).

    Record boundaries come from the header signatures rather than from DPSM,
    which the tool leaves at a stale value on 4bpp uploads.
    """
    offs = signatures(buf)
    recs = []
    for i, o in enumerate(offs):
        end = offs[i + 1] if i + 1 < len(offs) else len(buf)
        recs.append(Record(o, buf[o:o + HDR], buf[o + HDR:end]))
    # Trailing tool garbage: whatever the last record does not account for.
    if recs:
        last = recs[-1]
        trailer = last.data[last.padded:]
        last.data = last.data[:last.padded]
    else:
        trailer = buf
    return recs, trailer


def build(recs, trailer=b""):
    return b"".join(r.hdr + r.data for r in recs) + trailer


def unswizzle_clut(entries):
    """Undo the GS's CSM1 storage order for a 256-entry CLUT.

    The GS stores a 256-entry palette as eight 32-entry blocks in which the
    middle two runs of 8 are exchanged.  16-entry (4bpp) palettes are stored
    straight and must not be touched.
    """
    if len(entries) != 256:
        return list(entries)
    out = list(entries)
    for base in range(0, 256, 32):
        for j in range(8):
            a, b = base + 8 + j, base + 16 + j
            out[a], out[b] = out[b], out[a]
    return out


def rgba(rec):
    """Decode a direct-colour record into a list of (r, g, b, a) tuples.

    PS2 alpha is 0..128 where 128 means fully opaque, so it is scaled up.
    """
    d, out = rec.data, []
    if rec.psm == 0:
        for i in range(0, rec.nbytes, 4):
            r, g, b, a = d[i], d[i + 1], d[i + 2], d[i + 3]
            out.append((r, g, b, min(255, a * 2)))
    elif rec.psm == 1:
        for i in range(0, rec.nbytes, 3):
            out.append((d[i], d[i + 1], d[i + 2], 255))
    elif rec.psm == 2:
        for i in range(0, rec.nbytes, 2):
            v = d[i] | d[i + 1] << 8
            out.append((((v) & 31) * 255 // 31, ((v >> 5) & 31) * 255 // 31,
                        ((v >> 10) & 31) * 255 // 31, 255 if v >> 15 else 0))
    else:
        raise ValueError(f"not a direct-colour record: {rec}")
    return out


def indices(rec):
    """Decode an indexed record into a flat list of palette indices."""
    d = rec.data
    if rec.psm == 19:
        return list(d[:rec.nbytes])
    if rec.psm == 20:
        out = []
        for i in range(rec.nbytes):
            out.append(d[i] & 0xF)
            out.append(d[i] >> 4)
        return out
    raise ValueError(f"not an indexed record: {rec}")


def textures(recs):
    """Pair each indexed record with the CLUT record that follows it.

    Yields (index_record, palette) where palette is a list of RGBA tuples
    already in index order.
    """
    for i, r in enumerate(recs):
        if r.psm not in (19, 20):
            continue
        nxt = recs[i + 1] if i + 1 < len(recs) else None
        pal = unswizzle_clut(rgba(nxt)) if nxt is not None and nxt.is_clut else []
        yield r, pal
