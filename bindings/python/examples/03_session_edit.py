#!/usr/bin/env python3
"""세션 편집 — 한 번 열어 여러 번 만지고, 바뀐 쪽만 렌더한다.

    python examples/03_session_edit.py 서식.hwp 결과.hwp
"""

from __future__ import annotations

import sys
from pathlib import Path

import rhwp


def main(source: str, target: str) -> int:
    with rhwp.open(source) as doc:
        info = doc.info()
        print(f"열림: {info.page_count}쪽")

        form_fields = doc.fields().fields
        if form_fields:
            name = form_fields[0].name
            doc.fill_fields({name: "세션에서 입력"})
            print(f"'{name}' 채움")

        hits = doc.search("보고")
        print(f"'보고' 검색: {hits.raw.get('matchCount', 0)}건")

        saved = doc.save(target, verify=True)
        verify = saved.verify
        status = "통과" if verify and verify.identical else "실패"
        print(f"저장: {target} (검증 {status})")

        # 바뀐 쪽만 렌더 — 전 쪽을 그리지 않으므로 상수 비용이다.
        for page in saved.changed_pages or []:
            svg = Path(target).with_suffix(f".p{page}.svg")
            doc.render_page(page, svg)
            print(f"  눈검증용 렌더: {svg}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2]))
