# REmaster

AI harness for breathing new life into forgotten games.

> This repo doesn't distribute any ISOs or copyrighted content, but it does
> contain reverse-engineered rust code that "shims" the original game
> executables. This allows you to modify parts of the game in rust as long as
> you have a legitimate copy.

## PS2

You need:

- **A game disc image** (ISO), placed under `./ps2/<game>/`.
- **PS2 BIOS files** dumped from your own console. Mount the directory
  containing them at `/root/.config/PCSX2/bios` — PCSX2 will not boot anything
  without them.
- Wayland

### Optional: check if your game includes debug symbols

Check this list: https://www.retroreversing.com/ps2-unstripped

If your game was shipped with debug symbols, the reversing process will be quite
easy for the agent.

### Build and run

```sh
docker build -t remaster-ps2 ps2/

docker run -it --rm \
  --device /dev/dri \
  --shm-size=256m \
  -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/runtime/wayland-0 \
  -v $PWD/ps2/game_dir:/workspace \
  -v ~/.claude/.credentials.json:/root/.claude/.credentials.json \
  -v ~/.claude.json:/root/.claude.json \
  -v /path/to/your/bios:/root/.config/PCSX2/bios \
  remaster-ps2
```

The container starts `claude` directly. Just invoke the `/remaster-ps2` skill to
get started. The agent will extract the ISO and begin reversing interesting
parts of the game into a rust-based "shim". Finally it will add new content and
enhance the existing assets. Progress is tracked in `README.md` so sessions can
be resumed.

The agent will repeatedly launch PCSX2 in a separate window on your screen. This
allows the agent to manipulate the game autonomously as it works. You can still
interact with the game to help the agent. For some games, the agent is painfully
slow, so it helps to just tell it "I did X for you in-game".
