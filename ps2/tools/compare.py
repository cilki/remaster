#!/usr/bin/env python3
"""Build labelled side-by-side comparisons from an ab.sh capture pair.

  compare.py <prefix> <outdir> [shot ...]   # reads <prefix>-off.png / -on.png

Each shot is  name:x0,y0,x1,y1:zoom  with the crop box as fractions of the
frame, e.g.  01-signs:0.06,0.20,0.40,0.34:2.4 .  Leave the box empty for the
full frame (full::0.62).  With no shots given, one full-frame comparison is
produced.  --labels A,B overrides the column captions.

Both shots must come through the same capture path: PCSX2's own F8 screenshot
and the compositor's grim render at different sizes, so a pair taken through
different paths cannot be compared honestly.
"""
import os, sys
from PIL import Image, ImageDraw, ImageFont

PAD, GAP, BAR = 12, 14, 42
BG, FG = (18, 18, 20), (238, 238, 238)


def _font(px):
    try:
        return ImageFont.load_default(size=px)
    except TypeError:          # very old Pillow
        return ImageFont.load_default()


def pair(off, on, box=None, zoom=1, labels=("ORIGINAL", "UPSCALED 2x")):
    if box:
        off, on = off.crop(box), on.crop(box)
    if zoom != 1:
        size = (int(off.width * zoom), int(off.height * zoom))
        # Nearest, so the comparison shows the pixels as they are rather than
        # letting the viewer's own resampling flatter either side.
        off, on = off.resize(size, Image.NEAREST), on.resize(size, Image.NEAREST)
    w, h = off.size
    out = Image.new("RGB", (w * 2 + GAP + PAD * 2, h + BAR + PAD * 2), BG)
    out.paste(off, (PAD, PAD + BAR))
    out.paste(on, (PAD + w + GAP, PAD + BAR))
    d = ImageDraw.Draw(out)
    f = _font(max(15, min(30, h // 14)))
    for i, text in enumerate(labels):
        d.text((PAD + i * (w + GAP) + 4, PAD + 6), text, font=f, fill=FG)
    return out


def main():
    argv = sys.argv[1:]
    labels = ("ORIGINAL", "UPSCALED 2x")
    if "--labels" in argv:
        i = argv.index("--labels")
        labels = tuple(argv[i + 1].split(","))
        del argv[i:i + 2]
    if len(argv) < 2:
        sys.exit(__doc__.strip())
    pre, out, specs = argv[0], argv[1], argv[2:] or ["full::1"]

    os.makedirs(out, exist_ok=True)
    off = Image.open(f"{pre}-off.png").convert("RGB")
    on = Image.open(f"{pre}-on.png").convert("RGB")
    if off.size != on.size:
        sys.exit(f"frames differ in size: {off.size} vs {on.size}")
    W, H = off.size

    for spec in specs:
        name, box, zoom = spec.split(":")
        if box:
            x0, y0, x1, y1 = (float(v) for v in box.split(","))
            box = (int(W * x0), int(H * y0), int(W * x1), int(H * y1))
        else:
            box = None
        p = os.path.join(out, name + ".png")
        pair(off, on, box, float(zoom), labels).save(p)
        print(p)


if __name__ == "__main__":
    main()
