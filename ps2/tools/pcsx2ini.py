#!/usr/bin/env python3
"""Merge PCSX2 ini fragments:  pcsx2ini.py <base> [override ...]

Prints the merge to stdout.  Later files win key by key.  The base's section
order is preserved; override keys the base lacks are appended to the end of
the base's matching section, and whole new sections go at the end.  Comment
lines (# or ;) are for the repo reader and are not carried into the output,
which PCSX2 rewrites on exit anyway.
"""
import re, sys

SEC = re.compile(r"\s*\[(.+?)\]\s*$")
KEY = re.compile(r"\s*([^=;#\[\s][^=]*?)\s*=")
COMMENT = re.compile(r"\s*[;#]")


def parse(path):
    """{section: {key: full_line}}, in file order."""
    out, cur = {}, None
    for line in open(path):
        m = SEC.match(line)
        if m:
            cur = m.group(1)
            out.setdefault(cur, {})
            continue
        m = KEY.match(line)
        if m and cur is not None:
            out[cur][m.group(1)] = line.rstrip("\n")
    return out


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__.strip())
    ov = {}
    for p in sys.argv[2:]:
        for sec, keys in parse(p).items():
            ov.setdefault(sec, {}).update(keys)

    done = {sec: set() for sec in ov}
    out, cur = [], None

    def close_section():
        """Append override-only keys before the blank lines ending a section."""
        if cur not in ov:
            return
        tail = []
        while out and out[-1] == "":
            tail.append(out.pop())
        for k, line in ov[cur].items():
            if k not in done[cur]:
                out.append(line)
                done[cur].add(k)
        out.extend(tail)

    for line in open(sys.argv[1]):
        line = line.rstrip("\n")
        if COMMENT.match(line):
            continue
        m = SEC.match(line)
        if m:
            close_section()
            cur = m.group(1)
            out.append(line)
            continue
        m = KEY.match(line)
        if m and cur in ov and m.group(1) in ov[cur]:
            out.append(ov[cur][m.group(1)])
            done[cur].add(m.group(1))
        else:
            out.append(line)
    close_section()

    for sec, keys in ov.items():
        rest = [line for k, line in keys.items() if k not in done[sec]]
        if rest:
            out += ["", f"[{sec}]"] + rest

    while out and out[0] == "":
        out.pop(0)
    print("\n".join(out))


if __name__ == "__main__":
    main()
