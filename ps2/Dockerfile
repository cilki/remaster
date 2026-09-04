FROM docker.io/nixos/nix:latest

# Pinned nixos-unstable; keep in sync with the pin in shell.nix
ENV NIX_PATH=nixpkgs=https://github.com/NixOS/nixpkgs/archive/56c02bc00adcf003215cc4bd996d6efaf4cff188.tar.gz
ENV NIXPKGS_ALLOW_UNFREE=1

# Nix can't install its own seccomp filter inside the container runtime's
# seccomp sandbox (EINVAL); the container already provides the sandboxing
RUN echo 'filter-syscalls = false' >> /etc/nix/nix.conf

# The base image ships git-minimal, which collides with the full git package
RUN nix-env -e git-minimal || true
RUN nix-env -f '<nixpkgs>' -iA git nodejs_22

# Realize the shell.nix environment into the store at build time. It lives
# outside /workspace, which is hidden by the bind mount at run time
COPY shell.nix /opt/remaster/shell.nix
RUN nix-shell /opt/remaster/shell.nix --run true
WORKDIR /workspace

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

# Register the pcsx2-mcp server via managed MCP config; this is exclusive,
# so sessions in the container get only this server
COPY mcp.json /etc/claude-code/managed-mcp.json

# Built-in skill; mipsel-ps2-elf.json is the custom rustc target for the
# PS2's Emotion Engine, used by the skill's Fabricate step
COPY SKILL.md mipsel-ps2-elf.json /root/.claude/skills/remaster-ps2/

# Skill helper tools, on PATH so the agent can invoke them by name (gsl.py
# and scalers.py are modules imported by their neighbours, which locate them
# via their own directory)
COPY tools/*.py tools/*.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/*.py /usr/local/bin/*.sh

# PCSX2 (Qt) supports Wayland natively; select it explicitly since Qt's
# auto-detection has no session info inside the container and would fall
# back to X11. The host Wayland socket is mounted at the fixed path
# /tmp/runtime/wayland-0 (see README). PCSX2 is launched inside a nested
# cage compositor (a regular window on the host) so the agent can
# screenshot it with grim and send keys with wtype.
#
# The default env targets cage's socket, not the host's: cage cannot bind
# in a runtime dir that already holds the host's wayland-0, so it gets its
# own (/tmp/cagert), where it creates its own wayland-0. With this as the
# default, pcsx2/grim/wtype (and PINE's pcsx2.sock) all land in /tmp/cagert
# with no per-command env. Only cage's outbound connection needs the host
# socket, and the `cage` wrapper in shell.nix supplies it by absolute path.
ENV QT_QPA_PLATFORM=wayland
ENV XDG_RUNTIME_DIR=/tmp/cagert
ENV WAYLAND_DISPLAY=wayland-0
ENV LANG=C.UTF-8
RUN mkdir -m 700 -p /tmp/runtime /tmp/cagert

# Claude Code refuses --dangerously-skip-permissions as root unless it
# knows it's inside a sandboxed container
ENV IS_SANDBOX=1

# Pre-baked PCSX2 config: skips the first-run setup wizard (the BIOS lands at
# the default path, so the wizard has nothing to configure), enables PINE, and
# binds pad 1 to the keyboard — see PCSX2.base.ini itself. BIOS files must be
# mounted at /root/.config/PCSX2/bios (see README).
#
# The base is installed as the initial config so a bare `cage -- pcsx2-qt`
# works; launch.sh regenerates the live ini from the base merged with the
# game's own /workspace/pcsx2.ini overrides (via pcsx2ini.py) on each launch.
COPY PCSX2.base.ini /opt/remaster/PCSX2.base.ini
RUN mkdir -p /root/.config/PCSX2/inis /root/.config/PCSX2/bios \
    && sed '/^\s*[;#]/d' /opt/remaster/PCSX2.base.ini \
       > /root/.config/PCSX2/inis/PCSX2.ini

# Launch claude in the foreground; the container is the permission boundary
ENTRYPOINT ["nix-shell", "/opt/remaster/shell.nix", "--run", "claude --dangerously-skip-permissions"]
