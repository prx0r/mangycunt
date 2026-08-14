#!/usr/bin/env bash
set -euo pipefail

if ! command -v apt-get >/dev/null 2>&1; then
  echo "This bootstrap currently supports Debian/Ubuntu-compatible systems." >&2
  exit 1
fi

sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  clang \
  cmake \
  git \
  curl \
  libasound2-dev \
  libudev-dev \
  libx11-dev \
  libxi-dev \
  libgl1-mesa-dev \
  ffmpeg

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  . "$HOME/.cargo/env"
fi

cargo build --release
