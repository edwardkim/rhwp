#!/usr/bin/env python3
"""IR 스키마 탐색 — 바인딩이 IR 모양을 하드코딩하지 않는 이유.

    python examples/06_ir_schema.py [타입이름]
"""

from __future__ import annotations

import sys

import rhwp


def main(target: str | None) -> int:
    schema = rhwp.ir_schema()
    print(f"IR 스키마 v{schema.version} — 정의 {len(schema)}개")

    broken = schema.dangling_references()
    if broken:
        print(f"끊어진 참조: {broken}")
        return 1

    if target is None:
        print("\n정의 목록:")
        for type_def in schema:
            kind = "union" if type_def.is_union else "object"
            print(f"  {type_def.name:24} {kind:7} {type_def.description[:48]}")
        print("\n특정 타입을 보려면: python examples/06_ir_schema.py Paragraph")
        return 0

    if target not in schema:
        print(f"'{target}' 정의가 없습니다. 있는 것: {', '.join(schema.names())}")
        return 2

    type_def = schema[target]
    print(f"\n{type_def.name} — {type_def.description}")
    if type_def.is_union:
        print(f"  유니온 변형: {', '.join(type_def.variants)}")
        return 0
    for f in type_def.fields:
        mark = "" if f.required else " (선택)"
        print(f"  {f.name:20} {f.python_type:24}{mark}")
        if f.description:
            print(f"      {f.description}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1] if len(sys.argv) > 1 else None))
