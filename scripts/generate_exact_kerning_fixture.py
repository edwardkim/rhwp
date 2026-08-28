#!/usr/bin/env python3
"""Materialize the deterministic Issue #4968 GPOS kerning smoke face."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path

from font_kerning_boundary import synthetic_font


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "tests" / "fixtures" / "fonts" / "RHWPExactKerningSmoke.ttf"
EXPECTED_SHA256 = "775667d1980cd734e331f01e9390e02191bc35d669325291c842968cb0a4a9fc"


def fixture_bytes() -> bytes:
    data = synthetic_font(gpos=True, legacy=False)
    digest = hashlib.sha256(data).hexdigest()
    if digest != EXPECTED_SHA256:
        raise RuntimeError(f"exact kerning fixture drift: {digest}")
    return data


def main() -> int:
    data = fixture_bytes()
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(data)
    print(f"{OUTPUT.relative_to(ROOT)} {len(data)} {EXPECTED_SHA256}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
