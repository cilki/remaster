#!/usr/bin/env python3
"""Build replacement textures from PCSX2's texture dumps.

  upscale.py <pack-root> [factor] [--min N] [--force]

<pack-root> is the per-game texture directory, e.g.
/workspace/textures/SLUS-20398: dumps are read from <pack-root>/dumps and
replacements written to <pack-root>/replacements.

PCSX2 keys replacements by texture hash, so only textures that have actually
been drawn -- and therefore dumped -- can be replaced.  The pack grows as the
game is played: reach somewhere new, re-run this, press F7, and the new
textures are live without rebuilding the ISO or rebooting the emulator.

Each texture is upscaled by the kernel that suits its content; see scalers.py
for why there are two.
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scalers
from PIL import Image


def main():
    argv = sys.argv[1:]
    if not argv or argv[0].startswith("--"):
        sys.exit(__doc__.strip())
    root = argv.pop(0)
    factor = int(argv[0]) if argv and not argv[0].startswith("--") else 2
    force = "--force" in argv
    minpx = int(argv[argv.index("--min") + 1]) if "--min" in argv else 8

    src, dst = f"{root}/dumps", f"{root}/replacements"
    os.makedirs(dst, exist_ok=True)
    made = skipped = kept = 0
    flat = 0
    for name in sorted(os.listdir(src)):
        if not name.endswith(".png"):
            continue
        out = os.path.join(dst, name)
        if os.path.exists(out) and not force:
            kept += 1
            continue
        im = Image.open(os.path.join(src, name)).convert("RGBA")
        if min(im.size) < minpx:
            skipped += 1
            continue
        if scalers.is_flat(im):
            flat += 1
        scalers.upscale(im, factor).save(out)
        made += 1
    print(f"{made} new ({flat} flat-art), {kept} already present, "
          f"{skipped} skipped under {minpx}px  ->  {dst}")


if __name__ == "__main__":
    main()
