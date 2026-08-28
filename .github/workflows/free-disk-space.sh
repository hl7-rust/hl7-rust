#!/usr/bin/env bash
# Free disk space on a GitHub-hosted ubuntu-latest runner before a Rust
# build, by removing preinstalled toolchains this workspace never touches.
#
# Called from ci.yml's `checks` and `msrv` jobs — the two that invoke
# cargo — before the Rust toolchain is even installed. The trademarks and
# docs jobs never build any code and do not need this.
#
# Every path below is one of the large, well-known preinstalled toolchains
# on GitHub's standard ubuntu-latest image (Android, .NET, Haskell/GHC,
# Swift, PowerShell, CodeQL, and unused Docker images) — not a guess at
# what might be there. Each removal is `|| true` because the runner image
# changes over time and a path going missing is not a reason to fail the
# build; what matters is freeing what is actually present today. `df -h /`
# before and after makes the effect checkable in the job log rather than
# asserted.

set -u

echo "--- disk before ---"
df -h /

free() {
  # "$1" is a path, "$2" is what it is, for the log.
  if [ -e "$1" ]; then
    sudo rm -rf "$1"
    echo "freed: $2 ($1)"
  fi
}

free /usr/local/lib/android "Android SDK"
free /usr/share/dotnet ".NET SDK"
free /usr/local/.ghcup "GHC/Haskell toolchain (ghcup)"
free /opt/ghc "GHC/Haskell toolchain"
free /usr/local/share/powershell "PowerShell"
free /usr/local/share/chromium "Chromium"
free /opt/hostedtoolcache/CodeQL "CodeQL bundle"
free /usr/local/lib/node_modules "global npm packages"

# Docker keeps its own preloaded images regardless of the paths above; this
# workspace's CI never runs a container.
if command -v docker >/dev/null 2>&1; then
  docker image prune --all --force >/dev/null 2>&1 || true
  echo "freed: preloaded Docker images"
fi

echo "--- disk after ---"
df -h /
