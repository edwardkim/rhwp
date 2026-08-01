#!/usr/bin/env python3
"""RAG 색인 — 주소를 잃지 않는 청킹.

평문을 추출해 외부에서 자르면 "몇 쪽"에 답할 수 없다. rhwp 는 조판 엔진을 갖고
있으므로 청크마다 주소를 붙일 수 있고, 그래야 인용을 검증할 수 있다.

    python examples/07_rag_index.py 문서.hwp [검증할문구]
"""

from __future__ import annotations

import json
import sys
from typing import Any, Dict, List

import rhwp


def build_index(path: str) -> List[Dict[str, Any]]:
    """절 단위로 자르되 주소를 보존한다."""
    digest = rhwp.digest(path, sections=True)
    chunks: List[Dict[str, Any]] = []

    for section in digest.raw.get("sections", []):
        text = section.get("text", "").strip()
        if not text:
            continue
        chunks.append(
            {
                "text": text,
                "source": path,
                "page": section.get("page"),
                "heading": section.get("heading"),
                "charCount": len(text),
            }
        )
    return chunks


def verify_citation(path: str, quote: str) -> List[int]:
    """인용문이 실제로 나오는 쪽 번호. 비어 있으면 그 인용은 근거가 없다."""
    hits = rhwp.search(path, quote)
    pages = []
    for match in hits.raw.get("matches", []):
        page = match.get("page")
        if page is not None:
            pages.append(page)
    return sorted(set(pages))


def main(path: str, quote: str | None) -> int:
    chunks = build_index(path)
    if not chunks:
        print("색인할 내용이 없습니다 (빈 문서이거나 절 구조가 없음)")
        return 1

    total = sum(c["charCount"] for c in chunks)
    print(f"청크 {len(chunks)}개, 총 {total:,}자")

    addressed = sum(1 for c in chunks if c["page"] is not None)
    print(f"주소 있는 청크: {addressed}/{len(chunks)}")
    if addressed < len(chunks):
        # 조판에 배치되지 않은 문단은 쪽을 알 수 없다 — 부분 목록보다 정직하다.
        print("  (일부 청크는 쪽을 확정할 수 없습니다)")

    print("\n앞 3개:")
    for c in chunks[:3]:
        head = c["heading"] or "(제목 없음)"
        preview = c["text"][:60].replace("\n", " ")
        print(f"  [{c['page']}쪽] {head}: {preview}…")

    if quote:
        pages = verify_citation(path, quote)
        if pages:
            print(f"\n인용 '{quote}' → {pages}쪽에서 확인됨")
        else:
            print(f"\n인용 '{quote}' 를 문서에서 찾지 못했습니다 — 근거 없는 인용입니다")
            return 1

    out = f"{path}.index.json"
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(chunks, fh, ensure_ascii=False, indent=2)
    print(f"\n색인 저장: {out}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) not in (2, 3):
        print(__doc__)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None))
