"""rhwp 프로세스 실행 — 봉투 계약을 지키는 얇은 껍데기.

계약 요지 (`--json` 모드):

- stdout 은 **순수 JSON**(배치는 NDJSON). 진단·진행·요약은 stderr.
- 실패 경로의 stdout 은 **0바이트** — 반쪽 JSON 을 흘리지 않는다.
- 종료 코드는 #2707 사전을 따른다 (:mod:`rhwp.errors` 참조).

이 모듈은 그 계약을 신뢰하되 **검증한다**. 계약이 깨졌을 때 조용히 넘기면
호출자는 빈 결과를 "차이 없음"으로 오독한다.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any, Dict, Iterator, List, Mapping, Optional, Sequence, Union

from ._binary import find_binary
from .errors import ProtocolError, RhwpError, RhwpTimeoutError, raise_for_exit

__all__ = ["run_json", "run_ndjson", "run_raw", "CompletedRun", "DEFAULT_TIMEOUT"]

#: 기본 제한 시간(초). 대형 문서 렌더가 수십 초 걸릴 수 있어 넉넉히 잡는다.
#: ``None`` 을 넘기면 무제한.
DEFAULT_TIMEOUT: Optional[float] = 300.0

PathLike = Union[str, Path]


class CompletedRun:
    """실행 결과 원문.

    Attributes:
        argv: 실제 실행한 명령줄.
        exit_code: 종료 코드.
        stdout: 표준 출력 (디코딩된 문자열).
        stderr: 표준 오류.
    """

    __slots__ = ("argv", "exit_code", "stdout", "stderr")

    def __init__(self, argv: Sequence[str], exit_code: int, stdout: str, stderr: str) -> None:
        self.argv = list(argv)
        self.exit_code = exit_code
        self.stdout = stdout
        self.stderr = stderr

    def __repr__(self) -> str:  # pragma: no cover - 표현만
        return (
            f"CompletedRun(exit={self.exit_code}, "
            f"stdout={len(self.stdout)}B, stderr={len(self.stderr)}B)"
        )


def _stringify(value: Any) -> str:
    """인자 하나를 문자열로. ``Path`` 와 숫자·불리언을 안전하게 받는다."""
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, bool):
        # 파이썬 bool 이 "True"/"False" 로 나가면 CLI 가 못 읽는다. 불리언은
        # 플래그로 표현해야 하므로, 값 위치에 오면 호출 조립 버그다.
        raise TypeError(
            "불리언은 인자 값이 될 수 없습니다 — 플래그로 표현하세요 (예: dry_run=True)"
        )
    return str(value)


def run_raw(
    args: Sequence[Any],
    *,
    stdin: Optional[str] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
    cwd: Optional[PathLike] = None,
    check: bool = True,
    raise_on_verdict: bool = False,
    envelope_hint: Optional[Mapping[str, Any]] = None,
) -> CompletedRun:
    """rhwp 를 실행하고 원문 결과를 돌려준다.

    Args:
        args: 실행 인자 (프로그램 이름 제외).
        stdin: 표준 입력으로 흘려 넣을 문자열 (batch 파일 목록, 암호 등).
        timeout: 제한 시간(초). ``None`` 이면 무제한.
        cwd: 작업 디렉터리.
        check: 참이면 종료 코드를 검사해 예외를 올린다.
        raise_on_verdict: exit 3/4 도 예외로 올릴지. 기본은 판정을 값으로 다룬다.
        envelope_hint: 예외에 담을 봉투 (이미 파싱했을 때).

    Raises:
        RhwpTimeoutError: 제한 시간 초과. 자식 프로세스는 죽인 뒤 올라온다.
        BinaryNotFoundError: 실행 파일을 못 찾음.
        UsageError / RhwpRuntimeError / VerdictFailed: ``check`` 가 참일 때.
    """
    binary = find_binary()
    argv: List[str] = [str(binary), *(_stringify(a) for a in args)]

    try:
        proc = subprocess.run(  # noqa: S603 - 경로는 우리가 탐색한 실행 파일
            argv,
            input=stdin,
            capture_output=True,
            # 봉투는 UTF-8 이 계약이다. 잘못된 바이트가 섞여도 죽지 않고
            # 치환하도록 두되, 그 경우 JSON 파싱이 실패해 ProtocolError 로 드러난다.
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            cwd=str(cwd) if cwd is not None else None,
        )
    except subprocess.TimeoutExpired as exc:
        raise RhwpTimeoutError(
            f"제한 시간 {timeout}초를 초과했습니다",
            argv=argv,
            stderr=_decode(exc.stderr),
        ) from exc
    except OSError as exc:
        # 실행 파일이 탐색 직후 사라졌거나, 권한이 회수됐거나, 플랫폼 불일치.
        raise RhwpError(
            f"rhwp 실행에 실패했습니다: {exc}",
            argv=argv,
        ) from exc

    result = CompletedRun(argv, proc.returncode, proc.stdout or "", proc.stderr or "")
    if check:
        raise_for_exit(
            result.exit_code,
            argv=argv,
            stderr=result.stderr,
            envelope=envelope_hint,
            raise_on_verdict=raise_on_verdict,
        )
    return result


def _decode(raw: Any) -> str:
    """``TimeoutExpired`` 가 넘겨주는 부분 출력을 안전하게 문자열로."""
    if raw is None:
        return ""
    if isinstance(raw, bytes):
        return raw.decode("utf-8", errors="replace")
    return str(raw)


def run_json(
    args: Sequence[Any],
    *,
    stdin: Optional[str] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
    cwd: Optional[PathLike] = None,
    raise_on_verdict: bool = False,
) -> Dict[str, Any]:
    """`--json` 명령을 실행하고 봉투를 dict 로 돌려준다.

    종료 코드 검사는 **파싱 뒤**에 한다 — exit 3(판정 실패)일 때도 봉투가 나오고,
    그 봉투에 판정 근거가 들어 있기 때문이다. 순서를 뒤집으면 가장 중요한
    정보를 버리게 된다.

    Raises:
        ProtocolError: stdout 이 JSON 이 아니거나, 성공했는데 비어 있을 때.
    """
    result = run_raw(
        args, stdin=stdin, timeout=timeout, cwd=cwd, check=False
    )

    envelope: Optional[Dict[str, Any]] = None
    text = result.stdout.strip()
    if text:
        try:
            parsed = json.loads(text)
        except json.JSONDecodeError as exc:
            raise ProtocolError(
                f"stdout 이 순수 JSON 이 아닙니다: {exc}",
                argv=result.argv,
                exit_code=result.exit_code,
                stderr=result.stderr,
            ) from exc
        if not isinstance(parsed, dict):
            raise ProtocolError(
                f"봉투는 JSON 객체여야 합니다 (받음: {type(parsed).__name__})",
                argv=result.argv,
                exit_code=result.exit_code,
                stderr=result.stderr,
            )
        envelope = parsed

    # 봉투를 예외에 실어 판정 근거를 보존한 채로 코드 검사.
    raise_for_exit(
        result.exit_code,
        argv=result.argv,
        stderr=result.stderr,
        envelope=envelope,
        raise_on_verdict=raise_on_verdict,
    )

    if envelope is None:
        # 성공(또는 판정 실패)인데 stdout 이 비었다 = 계약 위반.
        raise ProtocolError(
            "성공했는데 stdout 이 비어 있습니다 — --json 봉투 계약 위반입니다",
            argv=result.argv,
            exit_code=result.exit_code,
            stderr=result.stderr,
        )
    return envelope


def run_ndjson(
    args: Sequence[Any],
    *,
    stdin: Optional[str] = None,
    timeout: Optional[float] = DEFAULT_TIMEOUT,
    cwd: Optional[PathLike] = None,
) -> List[Dict[str, Any]]:
    """batch 계열을 실행하고 NDJSON 레코드 목록을 돌려준다.

    batch 는 **부분 실패도 실패**다 — 성공 레코드는 스트림에 남고 종료 코드가
    신호한다. 그래서 여기서는 exit 1 을 예외로 올리지 않고, 레코드에 담긴
    ``error`` 필드를 호출자가 보게 한다. 스트림을 통째로 버리면 성공분까지
    잃는다.
    """
    result = run_raw(args, stdin=stdin, timeout=timeout, cwd=cwd, check=False)

    records: List[Dict[str, Any]] = []
    for lineno, line in enumerate(result.stdout.splitlines(), 1):
        stripped = line.strip()
        if not stripped:
            continue
        try:
            parsed = json.loads(stripped)
        except json.JSONDecodeError as exc:
            raise ProtocolError(
                f"NDJSON {lineno}번째 줄이 JSON 이 아닙니다: {exc}",
                argv=result.argv,
                exit_code=result.exit_code,
                stderr=result.stderr,
            ) from exc
        if not isinstance(parsed, dict):
            raise ProtocolError(
                f"NDJSON {lineno}번째 줄이 객체가 아닙니다",
                argv=result.argv,
                exit_code=result.exit_code,
                stderr=result.stderr,
            )
        records.append(parsed)

    # 사용법 오류(2)는 스트림이 아예 성립하지 않은 것이므로 예외.
    if result.exit_code == 2:
        raise_for_exit(2, argv=result.argv, stderr=result.stderr)
    return records


def iter_ndjson(
    args: Sequence[Any],
    *,
    stdin: Optional[str] = None,
    cwd: Optional[PathLike] = None,
) -> Iterator[Dict[str, Any]]:
    """NDJSON 을 **스트리밍**으로 읽는다 — 대량 배치에서 메모리를 아낀다.

    전량을 모으는 :func:`run_ndjson` 과 달리 레코드가 나오는 대로 넘긴다.
    소비자가 중간에 멈추면 자식 프로세스도 정리한다.
    """
    binary = find_binary()
    argv: List[str] = [str(binary), *(_stringify(a) for a in args)]
    proc = subprocess.Popen(  # noqa: S603
        argv,
        stdin=subprocess.PIPE if stdin is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        encoding="utf-8",
        errors="replace",
        cwd=str(cwd) if cwd is not None else None,
    )
    try:
        if stdin is not None and proc.stdin is not None:
            proc.stdin.write(stdin)
            proc.stdin.close()
        assert proc.stdout is not None
        for lineno, line in enumerate(proc.stdout, 1):
            stripped = line.strip()
            if not stripped:
                continue
            try:
                parsed = json.loads(stripped)
            except json.JSONDecodeError as exc:
                raise ProtocolError(
                    f"NDJSON {lineno}번째 줄이 JSON 이 아닙니다: {exc}", argv=argv
                ) from exc
            if isinstance(parsed, dict):
                yield parsed
    finally:
        # 소비자가 break 로 빠져나가도 자식이 남지 않게 한다.
        if proc.poll() is None:
            proc.kill()
        proc.wait()
