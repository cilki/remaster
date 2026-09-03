## REmaster

AI harness for breathing new life into forgotten games.

### PS2

You need:

- **A game disc image** (ISO/CHD/BIN+CUE), placed under `./workspace`.
- **PS2 BIOS files** dumped from your own console. Mount the directory
  containing them at `/root/.config/PCSX2/bios` — PCSX2 will not boot anything
  without them.

#### Build and run

```sh
docker build -t remaster-ps2 ps2/

docker run -it --rm \
  --device /dev/dri \
  --shm-size=256m \
  -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/runtime/wayland-0 \
  -v $PWD/workspace:/workspace \
  -v ~/.claude/.credentials.json:/root/.claude/.credentials.json \
  -v ~/.claude.json:/root/.claude.json \
  -v /path/to/your/bios:/root/.config/PCSX2/bios \
  remaster-ps2
```

The container starts `claude` directly. Invoke the `remaster-ps2` skill to get
started. Progress is tracked in `/workspace/WORK.md` so sessions can be
resumed.

#### Screen capture and input

The agent launches PCSX2 inside a nested `cage` compositor, which appears as one
ordinary window on your desktop. Through cage's own Wayland socket the agent
screenshots the game with `grim` and sends keystrokes with `wtype` — it never
sees or touches the rest of your desktop. Pad 1 is pre-bound to the keyboard in
the image (WASD/TFGH = sticks, IJKL = face buttons) so the agent can play the
game.

#### Debugger notes

The image enables PINE in PCSX2 (port 28011), which gives the MCP bridge memory
read/write, pattern search, and savestates on stock PCSX2.
