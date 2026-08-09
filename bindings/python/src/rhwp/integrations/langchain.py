"""[#4340 U1] LangChain 문서 로더 — HWP/HWPX 를 쪽 단위 Document 로.

사용법::

    pip install rhwp langchain-core

    from rhwp.integrations.langchain import RHWPLoader

    docs = RHWPLoader("공문.hwp").load()          # 쪽마다 Document 하나
    docs = RHWPLoader("공문.hwp", per_page=False).load()  # 문서 전체 하나

데이터 계약(실측, #4340): ``export-text --json`` 봉투의 ``pages[] = {page, text}``
를 그대로 소비한다. metadata 는 ``{source, format, page, total_pages}`` — 쪽
번호가 있어야 검색 결과를 "몇 쪽" 으로 되짚을 수 있다(발췌 근거 요구는 공공
문서 RAG 의 상수다).

의존성: ``langchain-core`` 만 지연 임포트한다(미설치면 설치 힌트를 담은
``ImportError``). 모듈 임포트 자체는 프레임워크 없이 성공한다.
"""

from __future__ import annotations

import importlib
from pathlib import Path
from typing import Any, Dict, Iterator, List, Optional, Union

from .. import commands

__all__ = ["RHWPLoader"]

PathLike = Union[str, Path]

_INSTALL_HINT = (
    "rhwp.integrations.langchain 은 langchain-core 가 필요합니다.\n"
    "  pip install langchain-core\n"
    "(rhwp 본체는 프레임워크 없이 동작합니다 — 이 어댑터만 선택 의존성입니다.)"
)


def _document_cls() -> Any:
    """``langchain_core.documents.Document`` 를 지연 임포트한다."""
    try:
        module = importlib.import_module("langchain_core.documents")
    except ImportError as exc:  # pragma: no cover - 힌트 경로는 테스트가 모킹으로 고정
        raise ImportError(_INSTALL_HINT) from exc
    return module.Document


class RHWPLoader:
    """HWP/HWPX → LangChain ``Document`` 로더.

    LangChain ``BaseLoader`` 프로토콜(``load``/``lazy_load``)을 구현한다.
    기반 클래스를 상속하지 않는 이유는 하나다 — 상속하면 이 모듈의 임포트가
    프레임워크 설치를 요구하게 되고, 선택 의존성 원칙이 깨진다.
    """

    def __init__(
        self,
        path: PathLike,
        *,
        per_page: bool = True,
        timeout: Optional[float] = commands.DEFAULT_TIMEOUT,
    ) -> None:
        self._path = Path(path)
        self._per_page = per_page
        self._timeout = timeout

    def lazy_load(self) -> Iterator[Any]:
        """쪽 단위(기본) 또는 문서 단위로 ``Document`` 를 낸다."""
        document = _document_cls()
        meta = commands.info(self._path, timeout=self._timeout)
        base: Dict[str, Any] = {
            "source": str(self._path),
            "format": meta["format"],
            "total_pages": meta["pageCount"],
        }
        envelope = commands.export_text(self._path, timeout=self._timeout)
        pages = envelope["pages"]
        if self._per_page:
            # 쪽 번호는 배열 위치로 1-기반 부여한다. 실측(#4340): 봉투의
            # pages[].page 는 전체 내보내기에서 0-기반, -p 지정에서 1-기반으로
            # 일관되지 않아, 인용용 번호는 위치에서 결정론적으로 만든다.
            for number, page in enumerate(pages, start=1):
                yield document(
                    page_content=page["text"],
                    metadata={**base, "page": number},
                )
        else:
            yield document(
                page_content="".join(page["text"] for page in pages),
                metadata=base,
            )

    def load(self) -> List[Any]:
        """``lazy_load`` 를 전부 소진한 리스트."""
        return list(self.lazy_load())
