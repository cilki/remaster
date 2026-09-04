#!/usr/bin/env python3
"""Export every texture in a .GSL to PNG:  gsldump.py <file.gsl> <outdir>"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gsl
from PIL import Image


def to_image(rec, pal):
    px = gsl.indices(rec)
    if not pal:  # no palette shipped -- show raw index as grey
        n = 1 << rec.bpp
        pal = [(i * 255 // (n - 1),) * 3 + (255,) for i in range(n)]
    flat = [pal[i] if i < len(pal) else (255, 0, 255, 255) for i in px]
    im = Image.new("RGBA", (rec.w, rec.h))
    im.putdata(flat)
    return im


def main():
    src, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    recs, _ = gsl.parse(open(src, "rb").read())
    stem = os.path.splitext(os.path.basename(src))[0]
    for n, (rec, pal) in enumerate(gsl.textures(recs)):
        p = f"{out}/{stem}_{n:03d}_{rec.w}x{rec.h}_{gsl.PSM_NAME[rec.psm]}.png"
        to_image(rec, pal).save(p)
        print(p)


if __name__ == "__main__":
    main()
