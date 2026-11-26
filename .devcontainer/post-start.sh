#!/usr/bin/env bash
# Shared, run on every container start.
set -euo pipefail
cd "$(dirname "$0")/.."

# Caches of wasm-pack and npm for Claude Code (see "env" in .claude/settings.json).
# /tmp/claude is writable in the Claude Code sandbox, but only if it already exists.
mkdir -p /tmp/claude

# Prune target/: drop artifacts built by uninstalled toolchains or unused for 30 days,
# then remove the oldest ones until it fits in 10 GB.
if [ -f Cargo.toml ]; then
	cargo sweep --installed
	cargo sweep --time 30
	cargo sweep --maxsize 10GB
fi
