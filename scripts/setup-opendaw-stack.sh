#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
external_dir="${repo_root}/external"
opendaw_dir="${external_dir}/OpenDaw"

mkdir -p "${external_dir}"

if [ ! -d "${opendaw_dir}/.git" ]; then
  git clone --depth 1 https://github.com/glenwrhodes/OpenDaw.git "${opendaw_dir}"
fi

cd "${opendaw_dir}"

git submodule update --init --depth 1 libs/JUCE libs/tracktion_engine

if [ -d libs/JUCE/.git ]; then
  git -C libs/JUCE checkout 7c89e11f6b7316c369f3d3f22227c60e816e738b
fi

cmake -S . -B build-linux -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_PREFIX_PATH=/usr/lib/x86_64-linux-gnu/cmake/Qt6

cmake --build build-linux --target OpenDaw --parallel "$(nproc)"

echo "OpenDaw build output:"
find build-linux -type f -perm -111 -name 'OpenDaw*' -print
