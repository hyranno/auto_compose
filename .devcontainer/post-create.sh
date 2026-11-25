#!/usr/bin/env bash
# Shared initialization, also called from local/post-create.sh.
set -euo pipefail
cd "$(dirname "$0")/.."

# Node.js and other tools pinned in mise.toml
mise trust --yes
mise install --yes
