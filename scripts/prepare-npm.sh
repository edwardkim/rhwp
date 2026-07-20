#!/bin/bash
# npm 배포 전 pkg/ 디렉토리 보완
# wasm-pack build 후 실행

set -e

PKG_DIR="pkg"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

echo "📦 npm 패키지 준비 (v${VERSION})"

# package.json 보완
cat > "${PKG_DIR}/package.json" << EOF
{
  "name": "@rhwp/core",
  "version": "${VERSION}",
  "description": "HWP/HWPX file parser and renderer — Rust + WebAssembly",
  "type": "module",
  "main": "rhwp.js",
  "types": "rhwp.d.ts",
  "files": [
    "rhwp_bg.wasm",
    "rhwp.js",
    "rhwp.d.ts",
    "rhwp_bg.wasm.d.ts",
    "README.md",
    "LICENSE"
  ],
  "keywords": [
    "hwp",
    "hwpx",
    "hml",
    "hancom",
    "hangul",
    "한글",
    "document",
    "parser",
    "renderer",
    "wasm",
    "webassembly",
    "rust"
  ],
  "repository": {
    "type": "git",
    "url": "git+https://github.com/edwardkim/rhwp.git"
  },
  "homepage": "https://edwardkim.github.io/rhwp/",
  "bugs": {
    "url": "https://github.com/edwardkim/rhwp/issues"
  },
  "funding": "https://github.com/sponsors/edwardkim",
  "license": "MIT",
  "author": "Edward Kim",
  "sideEffects": [
    "./snippets/*"
  ]
}
EOF

# npm 패키지용 README 복사
cp "$(dirname "$0")/../npm/README.md" "${PKG_DIR}/README.md"

echo "✅ 완료: ${PKG_DIR}/package.json + README.md"
