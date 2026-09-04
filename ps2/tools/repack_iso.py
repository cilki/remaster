#!/usr/bin/env python3
"""Rebuild a modified PS2 game ISO without touching its ISO9660 filesystem.

Many PS2 games are layout-sensitive: they read assets by raw sector number
(from tables baked into the boot ELF), and the PS2 kernel's ISO9660 parser is
picky enough that images rebuilt with modern mastering tools (xorriso etc.)
may not even boot. So this tool never rebuilds the filesystem. It keeps the
original image byte-for-byte and applies changes from an extracted directory
tree on top:

- unchanged file        -> left alone
- same-size change      -> overwritten in place at its original sectors
- size change           -> appended at the end of the image (2048-aligned),
                           then every reference to the old location is
                           retargeted:

  1. Its ISO9660 directory record (both-endian extent LBA + data length) is
     repointed at the appended copy, so by-name loads through the real
     filesystem — including the PS2 kernel loading the boot ELF — find it.
  2. Any LBA-table records for it inside the boot ELF are rewritten. A
     record is located heuristically but reliably: the file's start LBA as a
     little-endian u32 with the file's size in 2048-byte blocks as a u32
     within +/-8 bytes, matched against the ELF's pre-retarget content.
     Files with no such record are assumed to be loaded by name only (a
     warning is printed so surprises are visible).

The boot ELF itself is discovered from BOOT2 in SYSTEM.CNF. The ELF may
change size like any other file; its own LBA table is patched wherever the
ELF ends up.

Usage:
  repack_iso.py <original.iso> <extracted_dir> <out.iso> [--test-move /PATH]

--test-move relocates the named file to the appended region even though its
content is unchanged; the game booting identically afterwards proves the
retargeting works for it.
"""

import os
import re
import struct
import sys

SECTOR = 2048


def blocks(size):
    return (size + SECTOR - 1) // SECTOR


def read_dir_records(img, lba, size):
    """Yield (img_offset, ident, extent_lba, data_len, is_dir) for each
    record of one directory extent."""
    base = lba * SECTOR
    off = 0
    while off < size:
        ln = img[base + off]
        if ln == 0:  # records never span sectors; skip to the next one
            off = (off // SECTOR + 1) * SECTOR
            continue
        ident_len = img[base + off + 32]
        ident = bytes(img[base + off + 33 : base + off + 33 + ident_len])
        extent = struct.unpack_from("<I", img, base + off + 2)[0]
        length = struct.unpack_from("<I", img, base + off + 10)[0]
        flags = img[base + off + 25]
        yield base + off, ident, extent, length, bool(flags & 2)
        off += ln


def walk_tree(img):
    """Return {"/PATH": (record_offset, extent_lba, data_len)} for every file
    in the primary ISO9660 tree."""
    root_lba = struct.unpack_from("<I", img, 16 * SECTOR + 156 + 2)[0]
    root_size = struct.unpack_from("<I", img, 16 * SECTOR + 156 + 10)[0]
    files = {}
    todo = [("", root_lba, root_size)]
    while todo:
        prefix, lba, size = todo.pop()
        for rec, ident, extent, length, is_dir in read_dir_records(img, lba, size):
            if ident in (b"\x00", b"\x01"):
                continue
            name = ident.split(b";")[0].decode("latin1")
            path = f"{prefix}/{name}"
            if is_dir:
                todo.append((path, extent, length))
            else:
                files[path] = (rec, extent, length)
    return files


def repoint_dir_record(img, rec_off, new_lba, new_size):
    """Rewrite a directory record's extent (both-endian LBA and length)."""
    struct.pack_into("<I", img, rec_off + 2, new_lba)
    struct.pack_into(">I", img, rec_off + 6, new_lba)
    struct.pack_into("<I", img, rec_off + 10, new_size)
    struct.pack_into(">I", img, rec_off + 14, new_size)


def boot_elf_path(img, tree):
    """Parse BOOT2 out of SYSTEM.CNF to find the boot ELF's path."""
    cnf = tree.get("/SYSTEM.CNF")
    if cnf is None:
        sys.exit("no /SYSTEM.CNF in image; not a PS2 disc?")
    _, lba, size = cnf
    text = bytes(img[lba * SECTOR : lba * SECTOR + size]).decode("latin1")
    m = re.search(r"BOOT2\s*=\s*cdrom0?:\\?([^;\r\n]+)", text)
    if not m:
        sys.exit(f"no BOOT2 line in SYSTEM.CNF: {text!r}")
    return "/" + m.group(1).replace("\\", "/")


def main():
    argv = sys.argv[1:]
    test_moves = set()
    while "--test-move" in argv:
        i = argv.index("--test-move")
        test_moves.add(argv[i + 1])
        del argv[i : i + 2]
    if len(argv) != 3:
        sys.exit(__doc__)
    orig_iso, extracted, out_iso = argv

    img = bytearray(open(orig_iso, "rb").read())
    if len(img) % SECTOR:
        sys.exit("original image is not sector-aligned")

    tree = walk_tree(img)
    elf_path = boot_elf_path(img, tree)
    if elf_path not in tree:
        sys.exit(f"boot ELF {elf_path} not present in image")
    _, elf_lba, elf_size = tree[elf_path]
    if img[elf_lba * SECTOR : elf_lba * SECTOR + 4] != b"\x7fELF":
        sys.exit(f"no ELF magic at {elf_path}")

    inplace, appended, unchanged = [], [], 0
    moves = {}  # path -> (new_lba, new_size)
    elf_new_off, elf_new_size = elf_lba * SECTOR, elf_size

    for path in sorted(tree):
        rec, lba, size = tree[path]
        src = os.path.join(extracted, path.lstrip("/"))
        if not os.path.exists(src):
            sys.exit(f"missing from extracted dir: {path}")
        new = open(src, "rb").read()
        old = bytes(img[lba * SECTOR : lba * SECTOR + size])
        if new == old and path not in test_moves:
            unchanged += 1
            continue
        if len(new) == size and path not in test_moves:
            img[lba * SECTOR : lba * SECTOR + size] = new
            inplace.append(path)
            continue
        # Needs relocation: append, repoint the directory record, and queue
        # the LBA-table retarget.
        new_lba = len(img) // SECTOR
        img += new + b"\0" * (-len(new) % SECTOR)
        appended.append(path)
        repoint_dir_record(img, rec, new_lba, len(new))
        moves[path] = (new_lba, len(new))
        if path == elf_path:
            elf_new_off, elf_new_size = new_lba * SECTOR, len(new)
        print(f"  moved {path}: lba {lba}->{new_lba}, size {size}->{len(new)}")

    # Retarget LBA-table records inside the boot ELF at its final home.
    # Records are located against the ELF's pre-retarget content (which still
    # holds the original LBA values wherever its table ended up), so earlier
    # writes can't create false matches.
    current_elf = memoryview(img)[elf_new_off : elf_new_off + elf_new_size]
    pristine_elf = bytes(current_elf)
    writes = {}
    for path, (new_lba, new_size) in sorted(moves.items()):
        if path == elf_path:
            continue  # loaded via its directory record, patched above
        _, olba, osize = tree[path]
        pl = struct.pack("<I", olba)
        pb = struct.pack("<I", blocks(osize))
        hits = 0
        for m in re.finditer(re.escape(pl), pristine_elf):
            o = m.start()
            if o % 4:
                continue
            for d in (4, 8, -4, -8):
                if (
                    0 <= o + d <= len(pristine_elf) - 4
                    and pristine_elf[o + d : o + d + 4] == pb
                ):
                    for at, val in ((o, new_lba), (o + d, blocks(new_size))):
                        prev = writes.get(at)
                        if prev is not None and prev[0] != val:
                            sys.exit(
                                f"conflicting patch at elf+{at:#x}: {prev[1]} vs {path}"
                            )
                        writes[at] = (val, path)
                    print(f"    LBA record for {path} at elf+{o:#x} retargeted")
                    hits += 1
                    break
        if hits == 0:
            print(
                f"    warning: no LBA record for {path}; assuming it is "
                f"loaded by name only"
            )
    for at, (val, _) in writes.items():
        current_elf[at : at + 4] = struct.pack("<I", val)

    open(out_iso, "wb").write(img)
    print(
        f"{unchanged} unchanged, {len(inplace)} patched in place, "
        f"{len(appended)} appended; image {len(img) // SECTOR} sectors "
        f"-> {out_iso}"
    )
    for p in inplace:
        print(f"  in-place: {p}")


if __name__ == "__main__":
    main()
