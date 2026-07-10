#!/usr/bin/env python
"""HBATANG.TTF(Haansoft Batang) ASCII advance 테이블 → Rust 상수 생성."""
import sys

from fontTools.ttLib import TTFont

sys.stdout.reconfigure(encoding="utf-8")

F = TTFont(r"C:\Program Files (x86)\HNC\Office 2022\HOffice120\Shared\TTF\All\HBATANG.TTF")
upm = F["head"].unitsPerEm
cmap = F.getBestCmap()
hmtx = F["hmtx"]

vals = []
for cp in range(0x20, 0x7F):
    if cp in cmap:
        vals.append(hmtx[cmap[cp]][0] / upm)
    else:
        vals.append(0.0)

print(f"// Haansoft Batang(한컴바탕) ASCII advance/em (upm={upm})")
print("const HAANSOFT_BATANG_ASCII: [f64; 95] = [")
for i in range(0, 95, 8):
    row = vals[i:i + 8]
    chars = "".join(chr(0x20 + j) for j in range(i, min(i + 8, 95)))
    print("    " + ", ".join(f"{v:.4f}" for v in row) + f", // {chars!r}")
print("];")
