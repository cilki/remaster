FROM docker.io/nixos/nix:latest

# Pinned nixos-unstable; keep in sync with the pin in shell.nix
ENV NIX_PATH=nixpkgs=https://github.com/NixOS/nixpkgs/archive/56c02bc00adcf003215cc4bd996d6efaf4cff188.tar.gz
ENV NIXPKGS_ALLOW_UNFREE=1

# The base image ships git-minimal, which collides with the full git package
RUN nix-env -e git-minimal || true
RUN nix-env -f '<nixpkgs>' -iA git nodejs_22

# Realize the shell.nix environment into the store at build time
WORKDIR /workspace
COPY shell.nix .
RUN nix-shell shell.nix --run true

# Install the pcsx2-mcp server (https://github.com/hkmodd/pcsx2-mcp)
RUN git clone --depth 1 https://github.com/hkmodd/pcsx2-mcp /opt/pcsx2-mcp \
    && cd /opt/pcsx2-mcp/pcsx2-mcp-server \
    && npm ci \
    && npm run build \
    && npm prune --omit=dev

RUN mkdir -p /usr/local/bin \
    && printf '#!/bin/sh\nexec node /opt/pcsx2-mcp/pcsx2-mcp-server/dist/index.js "$@"\n' \
       > /usr/local/bin/pcsx2-mcp \
    && chmod +x /usr/local/bin/pcsx2-mcp
ENV PATH=/usr/local/bin:$PATH

# Register the pcsx2-mcp server with claude-code and install the skill
COPY mcp.json /workspace/.mcp.json
COPY SKILL.md /workspace/.claude/skills/remaster-ps2/SKILL.md

# Custom rustc target for the PS2's Emotion Engine
COPY mipsel-ps2-elf.json /workspace/mipsel-ps2-elf.json

# PCSX2 (Qt) supports Wayland natively; select it explicitly since Qt's
# auto-detection has no session info inside the container and would fall
# back to X11. The host Wayland socket is mounted at the fixed path
# /tmp/runtime/wayland-0 (see README), so these must not be overridden
# with host values at run time.
ENV QT_QPA_PLATFORM=wayland
ENV XDG_RUNTIME_DIR=/tmp/runtime
ENV WAYLAND_DISPLAY=wayland-0
ENV LANG=C.UTF-8
RUN mkdir -m 700 -p /tmp/runtime

# Enable PINE so pcsx2-mcp can connect to stock PCSX2 (port 28011).
# BIOS files must be mounted at /root/.config/PCSX2/bios (see README).
RUN mkdir -p /root/.config/PCSX2/inis /root/.config/PCSX2/bios \
    && printf '[EmuCore]\nEnablePINE = true\n' > /root/.config/PCSX2/inis/PCSX2.ini

ENTRYPOINT ["nix-shell", "/workspace/shell.nix"]
