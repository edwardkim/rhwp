#!/usr/bin/env python3
"""해시로 고정한 공식 전체 폰트를 글자·이름 삭제 없이 WOFF2로 압축한다."""
import argparse
import hashlib
import json
from pathlib import Path

from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]


def generate(source_dir, output_dir):
    registry = json.loads((ROOT / "assets/fonts/licenses/font-notices.json").read_text())
    output_dir.mkdir(parents=True, exist_ok=True)
    for entry in registry["fonts"]:
        source = entry.get("officialProgram")
        if source is None:
            continue
        original = source_dir / source["localSourceName"]
        digest = hashlib.sha256(original.read_bytes()).hexdigest()
        if digest != source["sha256"]:
            raise ValueError(f"공식 원본 SHA-256 불일치: {original}")
        font = TTFont(original, recalcTimestamp=False)
        # subset·스타일 변경·이름 변경을 하지 않는다. 원본의 고지와 수정 시각도 보존한다.
        font.flavor = "woff2"
        target = output_dir / entry["file"]
        font.save(target)
        print(target.name, hashlib.sha256(target.read_bytes()).hexdigest(), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, default=ROOT / "assets/fonts")
    args = parser.parse_args()
    generate(args.source_dir, args.output_dir)
