"""[#4340 U1] LlamaIndex 리더 — HWP/HWPX 를 쪽 단위 Document 로.

사용법::

    pip install rhwp llama-index-core

    from rhwp.integrations.llama_index import RHWPReader

    docs = RHWPReader().load_data("공문.hwp")

계약은 :mod:`rhwp.integrations.langchain` 과 동형이다 — 같은 ``pages[]`` 실측
봉투, 같은 metadata(``source``/``format``/``page``/``total_pages``), 같은 선택
의존성 원칙(지연 임포트 + 미설치 시 설치 힌트 ``ImportError``).
"""

from __future__ import annotations

import importlib
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from .. import commands

__all__ = ["RHWPReader"]

PathLike = Union[str, Path]

_INSTALL_HINT = (
    "rhwp.integrations.llama_index 는 llama-index-core 가 필요합니다.\n"
    "  pip install llama-index-core\n"
    "(rhwp 본체는 프레임워크 없이 동작합니다 — 이 어댑터만 선택 의존성입니다.)"
)


def _document_cls() -> Any:
    """``llama_index.core.schema.Document`` 를 지연 임포트한다."""
    try:
        module = importlib.import_module("llama_index.core.schema")
    except ImportError as exc:  # pragma: no cover - 힌트 경로는 테스트가 모킹으로 고정
        raise ImportError(_INSTALL_HINT) from exc
    return module.Document


class RHWPReader:
    """HWP/HWPX → LlamaIndex ``Document`` 리더 (``load_data`` 프로토콜)."""

    def __init__(self, *, per_page: bool = True) -> None:
        self._per_page = per_page

    def load_data(
        self,
        file: PathLike,
        extra_info: Optional[Dict[str, Any]] = None,
        *,
        timeout: Optional[float] = commands.DEFAULT_TIMEOUT,
    ) -> List[Any]:
        document = _document_cls()
        path = Path(file)
        meta = commands.info(path, timeout=timeout)
        base: Dict[str, Any] = {
            "source": str(path),
            "format": meta["format"],
            "total_pages": meta["pageCount"],
            **(extra_info or {}),
        }
        envelope = commands.export_text(path, timeout=timeout)
        pages = envelope["pages"]
        if self._per_page:
            # 쪽 번호는 배열 위치로 1-기반 부여 — langchain 어댑터와 동일 사유.
            return [
                document(text=page["text"], metadata={**base, "page": number})
                for number, page in enumerate(pages, start=1)
            ]
        return [
            document(text="".join(page["text"] for page in pages), metadata=base)
        ]
