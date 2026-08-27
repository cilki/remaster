## REmaster

AI harness for breathing new life into forgotten games.

### PS2

You need:

- **A game disc image** (ISO/CHD/BIN+CUE) that you legally own, placed in a
  host directory that gets mounted at `/workspace/workspace`.
- **PS2 BIOS files** dumped from your own console. Mount the directory
  containing them at `/root/.config/PCSX2/bios` — PCSX2 will not boot
  anything without them.

#### Build and run

```sh
docker build -t remaster-ps2 ps2/

docker run -it --rm \
  --device /dev/dri \
  -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/runtime/wayland-0 \
  -v $PWD/workspace:/workspace/workspace \
  -v ~/.claude:/root/.claude \
  -v ~/.claude.json:/root/.claude.json \
  -v /path/to/your/bios:/root/.config/PCSX2/bios \
  remaster-ps2
```

Inside the container shell, start `claude` and invoke
the `remaster-ps2` skill; progress is tracked in `workspace/WORK.md` so
sessions can be resumed.

#### Debugger notes

The image enables PINE in PCSX2 (port 28011), which gives the MCP bridge
memory read/write, pattern search, and savestates on stock PCSX2. Full
debugging (breakpoints, disassembly, register access) requires building PCSX2
with the DebugServer patch from the pcsx2-mcp repository.
