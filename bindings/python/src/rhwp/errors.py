"""rhwp 종료 코드 → 파이썬 예외 매핑.

핵심 규약 (`mydocs/tech/bindings_foundation.md` §3):

- exit 1 = 런타임 실패 → :class:`RhwpRuntimeError`
- exit 2 = 사용법 오류 → :class:`UsageError` (호출을 조립한 **우리 쪽** 버그)
- exit 3/4 = **검증 단언 실패 — 예외가 아니라 반환값의 판정 필드**

exit 3/4 를 예외로 올리지 않는 이유가 이 모듈의 존재 이유다. `--verify` 가
불일치를 보고하거나 `render-diff` 가 회귀를 검출한 것은 **도구가 정상 동작한
결과**다. 예외로 만들면 호출자가 `try/except` 로 "고장"처럼 다루게 되고, 정작
봉투에 담긴 판정 근거(`diffCount`·`status`·`pages`)를 읽지 않는다. 그래서
기본값은 판정을 값으로 돌려주는 것이고, 예외를 원하면 `raise_on_verdict=True`
로 명시해야 한다.
"""

from __future__ import annotations

from typing import Any, Mapping, Optional, Sequence

__all__ = [
    "RhwpError",
    "BinaryNotFoundError",
    "UsageError",
    "RhwpRuntimeError",
    "VerdictFailed",
    "ProtocolError",
    "SessionClosedError",
    "TimeoutError",
    "EXIT_OK",
    "EXIT_RUNTIME",
    "EXIT_USAGE",
    "EXIT_VERIFY",
    "EXIT_VERIFY_PAGES",
    "raise_for_exit",
]

#: 성공.
EXIT_OK = 0
#: 런타임 실패 (읽기·파싱·렌더·쓰기).
EXIT_RUNTIME = 1
#: 사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 범위 초과, 계획 선검증 위반).
EXIT_USAGE = 2
#: 검증 단언 실패 (convert/export-hwpx --verify, edit --verify, run assertions,
#: render-diff 회귀).
EXIT_VERIFY = 3
#: --verify-pages 페이지 수 불일치.
EXIT_VERIFY_PAGES = 4


class RhwpError(Exception):
    """모든 rhwp 예외의 기반.

    Attributes:
        message: 사람이 읽을 설명.
        argv: 실행한 명령줄 (재현용). 없으면 ``None``.
        exit_code: 프로세스 종료 코드. 프로세스를 못 띄웠으면 ``None``.
        stderr: 도구가 남긴 진단 원문. 진단은 stdout 이 아니라 stderr 에 있다.
        envelope: 파싱에 성공한 봉투가 있으면 그대로 담는다 (판정 근거 보존).
    """

    def __init__(
        self,
        message: str,
        *,
        argv: Optional[Sequence[str]] = None,
        exit_code: Optional[int] = None,
        stderr: str = "",
        envelope: Optional[Mapping[str, Any]] = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.argv = list(argv) if argv is not None else None
        self.exit_code = exit_code
        self.stderr = stderr
        self.envelope = dict(envelope) if envelope is not None else None

    def __str__(self) -> str:  # pragma: no cover - 표현만
        parts = [self.message]
        if self.exit_code is not None:
            parts.append(f"(exit {self.exit_code})")
        detail = self.stderr.strip()
        if detail:
            # stderr 마지막 줄이 대개 가장 구체적이다 (did-you-mean 힌트 포함).
            parts.append(f"— {detail.splitlines()[-1]}")
        return " ".join(parts)

    @property
    def command(self) -> str:
        """재현 가능한 명령 문자열. 버그 리포트에 그대로 붙일 수 있게."""
        if not self.argv:
            return ""
        return " ".join(_quote(a) for a in self.argv)


class BinaryNotFoundError(RhwpError):
    """rhwp 실행 파일을 찾지 못했다.

    탐색 순서(`RHWP_BIN` → 패키지 동봉 → PATH)를 모두 시도한 뒤에만 발생한다.
    메시지에 시도한 경로를 모두 담아, 사용자가 어디에 두면 되는지 알 수 있게 한다.
    """


class UsageError(RhwpError):
    """exit 2 — 호출 조립이 틀렸다.

    이건 **우리 쪽(바인딩 또는 호출자) 버그**다. 재시도해도 같은 결과가 나오므로
    호출자는 인자를 고쳐야 한다. 도구가 did-you-mean 힌트를 stderr 에 남겼다면
    :attr:`suggestion` 으로 꺼내 쓸 수 있다.
    """

    @property
    def suggestion(self) -> Optional[str]:
        """stderr 의 ``힌트:`` 줄에서 교정 제안을 추출한다. 없으면 ``None``."""
        for line in reversed(self.stderr.splitlines()):
            stripped = line.strip()
            if stripped.startswith("힌트:"):
                return stripped[len("힌트:") :].strip()
        return None


class RhwpRuntimeError(RhwpError):
    """exit 1 — 읽기·파싱·렌더·쓰기가 실패했다.

    파일이 없거나 손상됐거나 디스크에 쓸 수 없는 경우다. 인자를 고쳐도 해결되지
    않으며, 입력 자체를 봐야 한다.
    """


class VerdictFailed(RhwpError):
    """exit 3/4 — 검증 단언이 실패했다. **기본적으로는 발생하지 않는다.**

    ``raise_on_verdict=True`` 를 명시했을 때만 올라온다. 기본 경로는 판정을
    반환값으로 돌려준다 — 도구는 정상 동작했고, 실패한 것은 *문서에 대한 단언*이기
    때문이다. :attr:`envelope` 에 판정 근거가 그대로 들어 있다.
    """

    @property
    def is_page_count_mismatch(self) -> bool:
        """exit 4 (페이지 수 불일치)인지."""
        return self.exit_code == EXIT_VERIFY_PAGES


class ProtocolError(RhwpError):
    """stdout 이 계약을 어겼다 — JSON 이 아니거나, 기대한 프레임이 아니다.

    `--json` 모드의 stdout 은 순수 JSON(배치는 NDJSON)이고 실패 경로는 0바이트다.
    그 계약이 깨졌다는 뜻이므로 도구 버그이거나 버전 불일치다.
    """


class SessionClosedError(RhwpError):
    """이미 닫힌 세션 핸들을 다시 썼다."""


class TimeoutError(RhwpError):  # noqa: A001 - 내장 이름 가림은 의도적(패키지 일관성)
    """제한 시간 안에 끝나지 않았다. 자식 프로세스는 종료를 시도한 뒤 올라온다."""


def _quote(arg: str) -> str:
    """공백·따옴표가 있으면 감싼다 — 재현 명령이 그대로 붙여넣기 가능하도록."""
    if arg and not any(ch.isspace() or ch in "\"'" for ch in arg):
        return arg
    escaped = arg.replace('"', '\\"')
    return f'"{escaped}"'


def raise_for_exit(
    exit_code: int,
    *,
    argv: Sequence[str],
    stderr: str = "",
    envelope: Optional[Mapping[str, Any]] = None,
    raise_on_verdict: bool = False,
) -> None:
    """종료 코드를 검사해 필요하면 예외를 올린다.

    Args:
        exit_code: 프로세스 종료 코드.
        argv: 실행한 명령줄 (예외에 재현용으로 담는다).
        stderr: 도구 진단 원문.
        envelope: 파싱된 봉투 (있으면 예외에 판정 근거로 담는다).
        raise_on_verdict: 참이면 exit 3/4 도 :class:`VerdictFailed` 로 올린다.
            기본값 거짓 — 판정은 반환값으로 다루는 것이 이 바인딩의 규약이다.

    Raises:
        UsageError: exit 2.
        RhwpRuntimeError: exit 1, 또는 사전에 없는 0 아닌 코드.
        VerdictFailed: exit 3/4 이면서 ``raise_on_verdict`` 가 참일 때.
    """
    if exit_code == EXIT_OK:
        return

    common = {"argv": argv, "exit_code": exit_code, "stderr": stderr, "envelope": envelope}

    if exit_code == EXIT_USAGE:
        raise UsageError("호출 인자가 올바르지 않습니다", **common)  # type: ignore[arg-type]
    if exit_code == EXIT_RUNTIME:
        raise RhwpRuntimeError("문서 처리에 실패했습니다", **common)  # type: ignore[arg-type]
    if exit_code in (EXIT_VERIFY, EXIT_VERIFY_PAGES):
        if raise_on_verdict:
            label = (
                "페이지 수가 일치하지 않습니다"
                if exit_code == EXIT_VERIFY_PAGES
                else "검증 단언이 실패했습니다"
            )
            raise VerdictFailed(label, **common)  # type: ignore[arg-type]
        # 판정 실패는 정상 반환 — 호출자가 봉투의 판정 필드를 읽는다.
        return

    # 사전에 없는 코드는 조용히 넘기지 않는다. 새 종료 코드가 생겼는데 바인딩이
    # 모르고 통과시키면, 실패한 작업이 성공으로 보고된다.
    raise RhwpRuntimeError(
        f"알 수 없는 종료 코드입니다 ({exit_code}) — rhwp 와 바인딩 버전이 어긋났을 수 있습니다",
        **common,  # type: ignore[arg-type]
    )
