"""[#4340 U1] 프레임워크 통합 어댑터 계약.

세 가지를 고정한다: ① 어댑터 모듈 임포트는 프레임워크 없이 성공한다(선택
의존성), ② 미설치면 설치 힌트를 담은 ImportError 가 난다, ③ 설치돼 있으면
실물 문서에서 쪽 단위 Document 계약(개수·metadata)이 지켜진다.
"""

from __future__ import annotations

import importlib
from pathlib import Path

import pytest

SAMPLE = (
    Path(__file__).resolve().parents[3]
    / "samples"
    / "basic"
    / "issue2007_nested_cell_pagination_42065.hwp"
)


def test_import_needs_no_framework() -> None:
    """어댑터 모듈 임포트 자체는 프레임워크 미설치여도 성공해야 한다."""
    for name in ("rhwp.integrations.langchain", "rhwp.integrations.llama_index"):
        importlib.import_module(name)


def test_missing_framework_raises_install_hint(monkeypatch: pytest.MonkeyPatch) -> None:
    """미설치 경로: pip 설치 힌트를 담은 ImportError — 조용한 실패 금지."""
    from rhwp.integrations import langchain as lc

    def boom(name: str) -> object:
        raise ImportError(f"No module named {name!r}")

    monkeypatch.setattr(lc.importlib, "import_module", boom)
    with pytest.raises(ImportError, match="pip install langchain-core"):
        lc.RHWPLoader(SAMPLE).load()


@pytest.mark.integration
def test_langchain_loader_yields_page_documents() -> None:
    """실물 문서 → 쪽수만큼 Document, metadata 계약(source/format/page/total_pages)."""
    pytest.importorskip("langchain_core")
    from rhwp.integrations.langchain import RHWPLoader

    docs = RHWPLoader(SAMPLE).load()
    assert len(docs) == 17  # 실측 쪽수 (rhwp info)
    first = docs[0]
    assert first.metadata["page"] == 1
    assert first.metadata["total_pages"] == 17
    assert first.metadata["format"] == "hwp5"
    assert first.metadata["source"].endswith(".hwp")
    assert any(doc.page_content.strip() for doc in docs)

    whole = RHWPLoader(SAMPLE, per_page=False).load()
    assert len(whole) == 1
    assert "page" not in whole[0].metadata
    assert whole[0].page_content == "".join(d.page_content for d in docs)


@pytest.mark.integration
def test_llama_index_reader_matches_contract() -> None:
    pytest.importorskip("llama_index.core")
    from rhwp.integrations.llama_index import RHWPReader

    docs = RHWPReader().load_data(SAMPLE, extra_info={"corpus": "unit"})
    assert len(docs) == 17
    assert docs[0].metadata["corpus"] == "unit"
    assert docs[0].metadata["page"] == 1
