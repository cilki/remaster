#!/usr/bin/env python3
"""Import a PNG back into a .GSL texture:  gslimport.py <file.gsl> <n> <png>

The inverse of gsldump.py.  Texture <n> is counted the same way gsldump.py
numbers its output files.  The record's own CLUT is authoritative and is never
rewritten, so every pixel of the PNG must be a colour the palette already
holds; anything else is snapped to the nearest entry and reported.

Palettes have duplicate entries, so a pixel whose colour is unchanged keeps the
index the record already used.  That makes an import of an unmodified dump a
byte-for-byte no-op, and confines the diff to the pixels that really changed.

Writes in place unless -o is given.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gsl
from PIL import Image


def encode(px, rec):
    """Pack a flat list of indices into the record's pixel format."""
    if rec.psm == 19:
        return bytes(px)
    if rec.psm == 20:
        return bytes(px[i] & 0xF | (px[i + 1] & 0xF) << 4
                     for i in range(0, len(px), 2))
    raise ValueError(f"not an indexed record: {rec}")


def main():
    src, n, png = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    out = sys.argv[sys.argv.index("-o") + 1] if "-o" in sys.argv else src

    buf = open(src, "rb").read()
    recs, trailer = gsl.parse(buf)
    rec, pal = list(gsl.textures(recs))[n]
    if not pal:
        sys.exit(f"texture {n} has no CLUT to match against")

    im = Image.open(png).convert("RGBA")
    if im.size != (rec.w, rec.h):
        sys.exit(f"{png} is {im.size[0]}x{im.size[1]}, record is {rec.w}x{rec.h}")

    # colour -> indices that hold it, in index order
    bycolour = {}
    for i, c in enumerate(pal):
        bycolour.setdefault(c, []).append(i)

    old = gsl.indices(rec)
    new, snapped = [], 0
    for i, c in enumerate(im.getdata()):
        hit = bycolour.get(c)
        if hit is None:
            snapped += 1
            j = min(range(len(pal)),
                    key=lambda k: sum((a - b) ** 2 for a, b in zip(pal[k], c)))
        elif old[i] in hit:
            j = old[i]          # unchanged pixel: keep its original index
        else:
            j = hit[0]
        new.append(j)

    data = encode(new, rec)
    assert len(data) == rec.nbytes, (len(data), rec.nbytes)
    rec.data = data + rec.data[rec.nbytes:]     # keep the qword padding
    open(out, "wb").write(gsl.build(recs, trailer))

    changed = sum(a != b for a, b in zip(old, new))
    print(f"{out}: texture {n} {rec.w}x{rec.h} "
          f"{gsl.PSM_NAME[rec.psm]}, {changed} indices changed"
          + (f", {snapped} colours not in the CLUT (snapped)" if snapped else ""))


if __name__ == "__main__":
    main()
