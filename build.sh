#!/usr/bin/env bash

set -euo pipefail

(
  cd webui
  pnpm install --frozen-lockfile
  pnpm build
)

wasm-pack build --release --target web --out-dir ../simpleui/pkg fitch-proof
mkdir -p webui/app/dist/simpleui
cp -R simpleui/. webui/app/dist/simpleui/
