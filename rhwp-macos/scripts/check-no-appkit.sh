#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

SHARED_FILES=(
  "$ROOT/rhwp-ios/Sources/RhwpDocument.swift"
  "$ROOT/rhwp-ios/Sources/RenderTree.swift"
  "$ROOT/rhwp-ios/Sources/FontFallback.swift"
  "$ROOT/rhwp-ios/Sources/CGTreeRenderer.swift"
)

HITS=""
for file in "${SHARED_FILES[@]}"; do
  FOUND=$(grep -nE 'AppKit|NSColor|NSImage|NSFont|NSView|UIKit|UIColor|UIImage|UIFont|UIBezier' "$file" 2>/dev/null || true)
  if [ -n "$FOUND" ]; then
    HITS+="$file:"$'\n'"$FOUND"$'\n'
  fi
done

if [ -n "$HITS" ]; then
  echo "FAIL: shared Swift code must not depend on AppKit/UIKit"
  printf '%s' "$HITS"
  exit 1
fi

echo "OK: shared Swift code has no AppKit/UIKit dependencies"
