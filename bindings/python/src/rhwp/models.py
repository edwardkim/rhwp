"""봉투 → 파이썬 객체 매핑.

설계 판단: dataclass 를 **명령마다 손으로 쓰지 않는다.** 봉투는 "필드 추가 허용"
계약이라 명령이 늘거나 필드가 붙을 때마다 수기 클래스는 뒤처진다. 대신
:class:`Envelope` 하나가 봉투를 감싸고, 키를 snake_case 속성으로 노출한다.

그러면서도 dict 의 편의(``env["pageCount"]``)를 잃지 않는다 — 원문 키와 변환 키를
모두 받는다. 봉투에 없는 필드를 물으면 조용히 ``None`` 이 아니라 :class:`KeyError`
계열로 실패한다: 오타가 "값 없음"으로 둔갑하면 그게 가장 찾기 어려운 버그다.
"""

from __future__ import annotations

from typing import Any, Dict, Iterator, List, Mapping, Optional

from ._naming import to_camel, to_snake

__all__ = ["Envelope", "VerifyReport", "envelope_of"]


class Envelope(Mapping[str, Any]):
    """`--json` 봉투 하나를 감싸는 읽기 전용 매핑.

    세 가지 방식으로 같은 값에 닿는다::

        env.page_count      # 속성 (snake_case)
        env["pageCount"]    # 원문 키
        env["page_count"]   # 변환 키

    중첩 객체는 접근 시점에 :class:`Envelope` 로 감싸고, 리스트는 항목별로 감싼다.
    """

    __slots__ = ("_raw", "_snake_index")

    def __init__(self, raw: Mapping[str, Any]) -> None:
        if not isinstance(raw, Mapping):
            raise TypeError(f"봉투는 매핑이어야 합니다 (받음: {type(raw).__name__})")
        object.__setattr__(self, "_raw", dict(raw))
        # snake → 원문 키 색인. 원문에 이미 snake 인 키가 있으면 그대로 매핑된다.
        index: Dict[str, str] = {}
        for key in self._raw:
            if isinstance(key, str):
                index.setdefault(to_snake(key), key)
        object.__setattr__(self, "_snake_index", index)

    # ── 매핑 프로토콜 ────────────────────────────────────────────────────
    def __getitem__(self, key: str) -> Any:
        raw = self._raw
        if key in raw:
            return _wrap(raw[key])
        # snake 로 물었을 수도 있다.
        original = self._snake_index.get(key)
        if original is not None:
            return _wrap(raw[original])
        # camel 로 물었는데 원문이 snake 인 경우.
        camel = to_camel(key)
        if camel in raw:
            return _wrap(raw[camel])
        raise KeyError(
            f"봉투에 '{key}' 필드가 없습니다. 있는 필드: {', '.join(sorted(map(str, raw)))}"
        )

    def __iter__(self) -> Iterator[str]:
        return iter(self._raw)

    def __len__(self) -> int:
        return len(self._raw)

    # ── 속성 접근 ────────────────────────────────────────────────────────
    def __getattr__(self, name: str) -> Any:
        # __slots__ 밖의 이름만 여기 온다.
        try:
            return self[name]
        except KeyError as exc:
            raise AttributeError(str(exc)) from exc

    def __setattr__(self, name: str, value: Any) -> None:
        raise AttributeError("봉투는 읽기 전용입니다 — 도구가 내놓은 판정을 고치지 않습니다")

    # ── 편의 ─────────────────────────────────────────────────────────────
    @property
    def raw(self) -> Dict[str, Any]:
        """원문 봉투 사본 (직렬화·로깅용)."""
        return dict(self._raw)

    def get_path(self, dotted: str, default: Any = None) -> Any:
        """``"verify.identical"`` 처럼 점 경로로 꺼낸다. 없으면 ``default``."""
        cur: Any = self
        for part in dotted.split("."):
            if isinstance(cur, Envelope):
                try:
                    cur = cur[part]
                except KeyError:
                    return default
            elif isinstance(cur, Mapping):
                if part not in cur:
                    return default
                cur = cur[part]
            else:
                return default
        return cur

    @property
    def schema_version(self) -> Optional[str]:
        """봉투 스키마 버전. 없으면 ``None`` (모든 봉투가 갖지만 방어적으로)."""
        value = self._raw.get("schemaVersion")
        return value if isinstance(value, str) else None

    @property
    def verify(self) -> Optional["VerifyReport"]:
        """`--verify` 보고가 있으면 :class:`VerifyReport`, 미요청이면 ``None``.

        봉투 규약상 미요청 시 ``null`` 이므로, ``None`` 은 "검증 안 함"이지
        "검증 실패"가 아니다. 이 둘을 섞으면 검증하지 않은 저장을 통과로 읽는다.
        """
        value = self._raw.get("verify")
        return VerifyReport(value) if isinstance(value, Mapping) else None

    @property
    def verify_pages(self) -> Optional["VerifyPagesReport"]:
        """[트랙 G R61 D-10] `--verify-pages` 보고가 있으면 :class:`VerifyPagesReport`,
        미요청이면 ``None``.

        :attr:`verify` 와 같은 규약이다 — ``None`` 은 "검증 안 함"이지 "검증 실패"가
        아니다.
        """
        value = self._raw.get("verifyPages")
        return VerifyPagesReport(value) if isinstance(value, Mapping) else None

    @property
    def changed_pages(self) -> Optional[List[int]]:
        """편집이 바꾼 쪽 목록(0 기준). 확정 불가·무산출이면 ``None``.

        ``None`` 과 ``[]`` 는 다르다 — 전자는 "모른다", 후자는 "바뀐 쪽이 없다".
        """
        value = self._raw.get("changedPages")
        if value is None:
            return None
        if isinstance(value, list):
            return [int(v) for v in value]
        return None

    def __repr__(self) -> str:  # pragma: no cover - 표현만
        keys = ", ".join(sorted(map(str, self._raw))[:6])
        more = "…" if len(self._raw) > 6 else ""
        return f"Envelope({keys}{more})"


class VerifyReport(Envelope):
    """`verify` 하위 봉투 — 저장 직후 자기검증 결과."""

    __slots__ = ()

    @property
    def identical(self) -> bool:
        """저장본이 메모리 IR 과 동일한가. 이 값이 판정의 전부다."""
        return bool(self._raw.get("identical", False))

    @property
    def diff_count(self) -> Optional[int]:
        """차이 개수. 재파싱 자체가 실패했으면 ``None``."""
        value = self._raw.get("diffCount")
        return None if value is None else int(value)

    @property
    def reparse_error(self) -> Optional[str]:
        """저장본을 다시 읽지 못했을 때의 사유. 정상이면 ``None``."""
        value = self._raw.get("reparseError")
        return value if isinstance(value, str) else None

    def __bool__(self) -> bool:
        """``if result.verify:`` 가 "통과했나"로 읽히도록."""
        return self.identical


class VerifyPagesReport(Envelope):
    """`verifyPages` 하위 봉투 — `--verify-pages` 요청 시 저장 전/후 쪽수 비교."""

    __slots__ = ()

    @property
    def before(self) -> Optional[int]:
        """저장 전(메모리 IR) 쪽수."""
        value = self._raw.get("before")
        return None if value is None else int(value)

    @property
    def after(self) -> Optional[int]:
        """저장 후 재파싱한 쪽수."""
        value = self._raw.get("after")
        return None if value is None else int(value)

    @property
    def identical(self) -> bool:
        """저장 전후 쪽수가 같은가. 이 값이 판정의 전부다."""
        return bool(self._raw.get("identical", False))

    def __bool__(self) -> bool:
        """``if result.verify_pages:`` 가 "통과했나"로 읽히도록."""
        return self.identical


def _wrap(value: Any) -> Any:
    """중첩 매핑·리스트를 접근 시점에 감싼다 (지연 변환 — 큰 봉투에서 낭비 없이)."""
    if isinstance(value, Envelope):
        return value
    if isinstance(value, Mapping):
        return Envelope(value)
    if isinstance(value, list):
        return [_wrap(item) for item in value]
    return value


def envelope_of(value: Mapping[str, Any]) -> Envelope:
    """dict 를 :class:`Envelope` 로 (이미 봉투면 그대로)."""
    return value if isinstance(value, Envelope) else Envelope(value)
