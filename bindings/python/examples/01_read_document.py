#!/usr/bin/env python3
"""문서 읽기 — 요약·평문·표·누름틀.

    python examples/01_read_document.py 문서.hwp
"""

from __future__ import annotations

import sys

import rhwp


def main(path: str) -> int:
    meta = rhwp.info(path)
    print(f"포맷: {meta.format}  쪽수: {meta.page_count}  구역: {meta.sections}")

    # 쪽별 평문 — 첫 쪽만 미리보기.
    text = rhwp.export_text(path)
    if text.pages:
        preview = text.pages[0].text[:120].replace("\n", " ")
        print(f"1쪽 미리보기: {preview}…")

    # 누름틀 — 서식 문서라면 여기에 채울 자리가 나온다.
    form_fields = rhwp.fields(path).fields
    if form_fields:
        print(f"\n누름틀 {len(form_fields)}개:")
        for f in form_fields[:10]:
            current = f.raw.get("value", "")
            print(f"  - {f.name}: {current!r}")

    # 표 — 좌표는 set_cell 에 그대로 쓴다.
    tables = rhwp.export_tables(path).tables
    if tables:
        print(f"\n표 {len(tables)}개:")
        for t in tables[:5]:
            print(f"  - 표 {t.index}: {len(t.cells)}칸")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
