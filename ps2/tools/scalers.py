"""Upscaling kernels, and the rule for choosing between them.

Two kernels, because the game mixes two kinds of texture and no single filter
suits both:

  lanczos  -- for shaded/photographic art.  Resamples in premultiplied alpha so
              transparent texels cannot bleed their colour into soft edges,
              then restores with a light unsharp mask the acutance that any
              resample necessarily removes.

  scale2x  -- for flat-colour art (logos, signs, HUD).  Lanczos rounds off the
              hard edges such art is made of; Scale2x instead extends a
              diagonal only where three neighbours agree one exists, so edges
              stay crisp and no colour that was not already present is ever
              invented.
"""
import numpy as np
from PIL import Image, ImageFilter


def lanczos(im, factor=2, sharpen=True):
    im = im.convert("RGBA")
    a = np.asarray(im, dtype=np.float32)
    alpha = a[..., 3:4] / 255.0
    pm = Image.fromarray((a[..., :3] * alpha).astype(np.uint8), "RGB")
    am = Image.fromarray(a[..., 3].astype(np.uint8), "L")
    big = (im.size[0] * factor, im.size[1] * factor)
    pm = pm.resize(big, Image.LANCZOS)
    if sharpen:
        pm = pm.filter(ImageFilter.UnsharpMask(radius=factor, percent=60, threshold=3))
    am = am.resize(big, Image.LANCZOS)
    out_rgb = np.asarray(pm, dtype=np.float32)
    out_a = np.asarray(am, dtype=np.float32)[..., None]
    safe = np.where(out_a > 0, out_a / 255.0, 1.0)
    out = np.concatenate([np.clip(out_rgb / safe, 0, 255), out_a], axis=2)
    return Image.fromarray(out.astype(np.uint8), "RGBA")


def scale2x(im):
    """AdvMAME2x, vectorised.  Doubles size; output uses only input colours."""
    a = np.asarray(im.convert("RGBA"))
    p = np.pad(a, ((1, 1), (1, 1), (0, 0)), mode="edge")
    e = p[1:-1, 1:-1]
    b, h = p[:-2, 1:-1], p[2:, 1:-1]      # up, down
    d, f = p[1:-1, :-2], p[1:-1, 2:]      # left, right
    eq = lambda x, y: (x == y).all(-1)[..., None]
    ne = lambda x, y: (x != y).any(-1)[..., None]
    e0 = np.where(eq(d, b) & ne(d, h) & ne(b, f), d, e)
    e1 = np.where(eq(b, f) & ne(b, d) & ne(f, h), f, e)
    e2 = np.where(eq(d, h) & ne(d, b) & ne(h, f), d, e)
    e3 = np.where(eq(h, f) & ne(h, d) & ne(f, b), f, e)
    hgt, wid = a.shape[:2]
    out = np.empty((hgt * 2, wid * 2, 4), a.dtype)
    out[0::2, 0::2], out[0::2, 1::2] = e0, e1
    out[1::2, 0::2], out[1::2, 1::2] = e2, e3
    return Image.fromarray(out, "RGBA")


def blend(a, b, t):
    """Mix two same-size images; t is the weight of b."""
    x = np.asarray(a, np.float32) * (1 - t) + np.asarray(b, np.float32) * t
    return Image.fromarray(x.astype(np.uint8), "RGBA")


def is_flat(im, max_colours=48):
    """True for art made of flat colour fields rather than continuous shading.

    Counted on opaque texels only: a sprite's fully transparent surround is
    one colour in RGBA terms but says nothing about the artwork.
    """
    a = np.asarray(im.convert("RGBA"))
    vis = a[a[..., 3] > 0][..., :3]
    if len(vis) == 0:
        return False
    return len(np.unique(vis, axis=0)) <= max_colours


def upscale(im, factor=2):
    """Upscale by the kernel that suits this texture's content."""
    im = im.convert("RGBA")
    if factor == 2 and is_flat(im):
        # Scale2x alone stair-steps on shallow angles; a light pull toward the
        # resampled version softens that without rounding the hard edges off.
        return blend(scale2x(im), lanczos(im, 2, sharpen=False), 0.35)
    return lanczos(im, factor)
