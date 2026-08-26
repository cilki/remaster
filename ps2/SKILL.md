# REmaster skill

Create a `WORK.md` if it doesn't exist yet. This file is used to track the
progress of our work.

## Extract step

If we have an ISO file under `./workspace`, extract it to `./workspace/game`. Do
not proceed to the next step until this one is complete.

## Reverse step

Search `./workspace/game` for the game executable. Catalog the available files
and assets in `WORK.md` for future steps. The final goal of this step is to
reverse-engineer parts of the game that we might want to modify later. Usually
we want to leave the low-level engine code untouched. We use the `resplice`
crate for this which allows us to "lift" certain portions of the code into rust.

Connect to the PCSX2 MCP debugger and ask the user to play the game so you can
watch memory during a real execution. Ask the user for in-game values that we
can use to locate structs in memory. Create `repr(C)` rust structs for these.

## Fabricate step

This is the part where we introduce new content into the game via our Rust shim.
If the user explicitly asked for changes, then just address them and nothing
else.

Otherwise, you are given the open-ended task to generally remaster the game: add
new content, improve existing content, design brand new levels, etc. Be creative
and remember that games should be fun. You can modify the game's code via the
Rust shim and game assets via direct overwrite. If you find you need to modify
some code that hasn't been lifted to rust yet, go back to the reverse step and
do so before continuing.
