#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  build-essential \
  clang \
  cmake \
  libssl-dev \
  pkg-config \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  libgdk-pixbuf-2.0-dev \
  libglib2.0-dev \
  libpango1.0-dev \
  libcairo2-dev \
  libx11-dev \
  libxdo-dev \
  libxrandr-dev \
  libxcursor-dev \
  libxi-dev \
  libxkbcommon-dev \
  libwayland-dev \
  libgl1-mesa-dev \
  libgstreamer1.0-dev \
  libgstreamer-plugins-base1.0-dev
