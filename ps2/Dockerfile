FROM docker.io/nixos/nix:latest

ENV NIX_PATH=nixpkgs=https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz
ENV NIXPKGS_ALLOW_UNFREE=1

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

ENTRYPOINT ["nix-shell", "/workspace/shell.nix"]
