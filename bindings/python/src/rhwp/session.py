"""API 2층 — 세션(핸들) 클라이언트.

`mcp-serve` 를 stdio JSON-RPC 로 띄우고 ``hwp_doc_*`` 도구를 그대로 노출한다.
1층(무상태)이 호출마다 문서를 재파싱하는 반면, 2층은 한 번 열어 두고 여러 번
만진다 — 대형 문서 반복 작업에서 차이가 크다.

```python
with rhwp.open("서식.hwp") as doc:
    doc.fill_fields({"성명": "홍길동"})
    doc.replace_text("2025년", "2026년")
    result = doc.save("제출본.hwp", verify=True)
    assert result.verify.identical
```

`with` 블록을 벗어나면 핸들이 닫히고 자식 프로세스가 정리된다. 예외로 빠져나가도
마찬가지다 — 서버가 남아 파일을 잡고 있으면 다음 작업이 막힌다.
"""

from __future__ import annotations

import json
import subprocess
import threading
from pathlib import Path
from types import TracebackType
from typing import Any, Dict, List, Mapping, Optional, Type, Union

from ._binary import find_binary
from .errors import (
    ProtocolError,
    RhwpError,
    SessionClosedError,
    UsageError,
)
from .models import Envelope

__all__ = ["Session", "Document", "open"]

PathLike = Union[str, Path]

#: 서버 기동 후 첫 응답을 기다리는 시간(초).
_STARTUP_TIMEOUT = 30.0


class Session:
    """`mcp-serve` 자식 프로세스 하나를 감싼 JSON-RPC 클라이언트.

    보통은 :func:`open` 이 만들어 주는 :class:`Document` 를 쓰면 되고, 여러 문서를
    한 서버에서 열고 싶을 때만 직접 만든다.
    """

    def __init__(self, *, profile: Optional[str] = None, timeout: Optional[float] = 300.0) -> None:
        self._timeout = timeout
        self._next_id = 0
        self._lock = threading.Lock()
        self._closed = False

        argv: List[str] = [str(find_binary()), "mcp-serve"]
        if profile:
            argv.extend(["--profile", profile])
        try:
            self._proc = subprocess.Popen(  # noqa: S603
                argv,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                encoding="utf-8",
                errors="replace",
                bufsize=1,  # 줄 단위 — JSON-RPC 프레임이 줄이다.
            )
        except OSError as exc:
            raise RhwpError(f"mcp-serve 기동에 실패했습니다: {exc}", argv=argv) from exc
        self._argv = argv

    # ── 저수준 ───────────────────────────────────────────────────────────
    def call(self, name: str, arguments: Mapping[str, Any]) -> Envelope:
        """도구 하나를 호출하고 결과 봉투를 돌려준다.

        Raises:
            SessionClosedError: 이미 닫힌 세션.
            UsageError: 도구가 ``isError`` 를 세운 경우. 서버가 ``didYouMean``·
                ``nextCall`` 교정 단서를 실어 보내면 예외 메시지에 함께 담긴다.
            ProtocolError: 응답이 JSON-RPC 계약을 어긴 경우.
        """
        if self._closed:
            raise SessionClosedError(f"세션이 이미 닫혔습니다 (도구: {name})")

        with self._lock:
            self._next_id += 1
            request_id = self._next_id
            request = {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "tools/call",
                "params": {"name": name, "arguments": dict(arguments)},
            }
            self._write(request)
            response = self._read(expect_id=request_id)

        return self._unwrap(name, response)

    def _write(self, payload: Mapping[str, Any]) -> None:
        if self._proc.stdin is None or self._proc.poll() is not None:
            raise ProtocolError(
                "mcp-serve 가 이미 종료되어 요청을 보낼 수 없습니다",
                argv=self._argv,
                stderr=self._drain_stderr(),
            )
        try:
            self._proc.stdin.write(json.dumps(payload, ensure_ascii=False) + "\n")
            self._proc.stdin.flush()
        except (BrokenPipeError, OSError) as exc:
            raise ProtocolError(
                f"mcp-serve 로 쓰기에 실패했습니다: {exc}",
                argv=self._argv,
                stderr=self._drain_stderr(),
            ) from exc

    def _read(self, *, expect_id: int) -> Dict[str, Any]:
        """응답 한 줄을 읽는다. 알림(id 없음)은 건너뛴다."""
        assert self._proc.stdout is not None
        while True:
            line = self._proc.stdout.readline()
            if not line:
                raise ProtocolError(
                    "mcp-serve 가 응답 없이 종료했습니다",
                    argv=self._argv,
                    exit_code=self._proc.poll(),
                    stderr=self._drain_stderr(),
                )
            stripped = line.strip()
            if not stripped:
                continue
            try:
                message = json.loads(stripped)
            except json.JSONDecodeError as exc:
                raise ProtocolError(
                    f"JSON-RPC 프레임이 아닙니다: {exc}",
                    argv=self._argv,
                    stderr=self._drain_stderr(),
                ) from exc
            if not isinstance(message, dict):
                raise ProtocolError("JSON-RPC 프레임은 객체여야 합니다", argv=self._argv)
            # 서버가 보낼 수 있는 알림은 id 가 없다 — 우리 응답이 아니므로 흘린다.
            if message.get("id") is None:
                continue
            if message["id"] != expect_id:
                # 요청은 락으로 직렬화하므로 여기 오면 서버가 순서를 어긴 것이다.
                raise ProtocolError(
                    f"응답 id 불일치 (기대 {expect_id}, 받음 {message['id']})",
                    argv=self._argv,
                )
            return message

    def _unwrap(self, name: str, response: Mapping[str, Any]) -> Envelope:
        """JSON-RPC 응답에서 도구 결과 봉투를 꺼낸다."""
        if "error" in response:
            err = response["error"]
            message = err.get("message", "알 수 없는 오류") if isinstance(err, Mapping) else str(err)
            raise ProtocolError(f"{name}: {message}", argv=self._argv)

        result = response.get("result")
        if not isinstance(result, Mapping):
            raise ProtocolError(f"{name}: result 가 없습니다", argv=self._argv)

        # 구조화 결과가 있으면 그걸 쓴다 (텍스트 재파싱보다 정확).
        structured = result.get("structuredContent")
        body: Optional[Dict[str, Any]] = None
        if isinstance(structured, Mapping):
            body = dict(structured)
        else:
            content = result.get("content")
            if isinstance(content, list) and content:
                first = content[0]
                if isinstance(first, Mapping) and isinstance(first.get("text"), str):
                    try:
                        parsed = json.loads(first["text"])
                        if isinstance(parsed, Mapping):
                            body = dict(parsed)
                    except json.JSONDecodeError:
                        # 텍스트가 JSON 이 아닌 도구도 있다 — 그대로 담아 돌려준다.
                        body = {"text": first["text"]}

        if result.get("isError"):
            detail = json.dumps(body, ensure_ascii=False) if body else ""
            raise UsageError(
                f"{name} 호출이 거부됐습니다",
                argv=self._argv,
                exit_code=2,
                stderr=detail,
                envelope=body,
            )

        if body is None:
            raise ProtocolError(f"{name}: 결과 본문을 해석하지 못했습니다", argv=self._argv)
        return Envelope(body)

    def _drain_stderr(self) -> str:
        """진단을 최대한 건져 낸다 (블로킹 없이 — 이미 죽은 프로세스 대상)."""
        if self._proc.stderr is None:
            return ""
        try:
            if self._proc.poll() is not None:
                return self._proc.stderr.read() or ""
        except (OSError, ValueError):
            pass
        return ""

    # ── 수명 ─────────────────────────────────────────────────────────────
    def close(self) -> None:
        """서버를 정리한다. 여러 번 불러도 안전하다."""
        if self._closed:
            return
        self._closed = True
        try:
            if self._proc.stdin is not None:
                try:
                    self._proc.stdin.close()
                except OSError:
                    pass
            try:
                self._proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                # stdin 을 닫아도 안 죽으면 강제 종료 — 서버가 남아 파일을
                # 잡고 있으면 다음 작업이 막힌다.
                self._proc.kill()
                self._proc.wait(timeout=5)
        except Exception:  # noqa: BLE001 - 정리 경로에서 새 예외를 만들지 않는다
            pass
        finally:
            for stream in (self._proc.stdout, self._proc.stderr):
                if stream is not None:
                    try:
                        stream.close()
                    except OSError:
                        pass

    def __enter__(self) -> "Session":
        return self

    def __exit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc: Optional[BaseException],
        tb: Optional[TracebackType],
    ) -> None:
        self.close()

    def __del__(self) -> None:  # pragma: no cover - GC 시점 의존
        try:
            self.close()
        except Exception:  # noqa: BLE001
            pass


class Document:
    """열린 문서 핸들 — 세션 위의 얇은 편의 계층."""

    def __init__(self, session: Session, doc_id: str, *, owns_session: bool = True) -> None:
        self._session = session
        self._doc_id = doc_id
        self._owns_session = owns_session
        self._closed = False

    @property
    def doc_id(self) -> str:
        """서버가 발급한 핸들 식별자."""
        return self._doc_id

    def _call(self, tool: str, **arguments: Any) -> Envelope:
        if self._closed:
            raise SessionClosedError(f"닫힌 문서 핸들입니다 ({self._doc_id})")
        return self._session.call(tool, {"docId": self._doc_id, **arguments})

    # ── 조회 ─────────────────────────────────────────────────────────────
    def info(self) -> Envelope:
        """문서 요약 (재파싱 없음)."""
        return self._call("hwp_doc_info")

    def text(self, *, page: Optional[int] = None) -> Envelope:
        """평문. ``page`` 를 주면 그 쪽만."""
        return self._call("hwp_doc_text", **({"page": page} if page is not None else {}))

    def fields(self) -> Envelope:
        """누름틀 목록."""
        return self._call("hwp_doc_fields")

    def tables(self) -> Envelope:
        """표 전량."""
        return self._call("hwp_doc_tables")

    def search(self, query: str, *, case_sensitive: bool = True) -> Envelope:
        """주소가 붙은 검색."""
        return self._call("hwp_doc_search", query=query, caseSensitive=case_sensitive)

    def render_page(self, page: int, output: PathLike) -> Envelope:
        """한 쪽을 SVG 파일로 — 편집 직후 눈검증 루프를 닫는 도구.

        Args:
            page: 0 기준 쪽 번호. 편집 봉투의 ``changed_pages`` 를 그대로 넘기면
                바뀐 쪽만 상수 비용으로 확인할 수 있다.
            output: SVG 를 쓸 경로. 도구 계약상 필수다.
        """
        return self._call("hwp_doc_render_page", page=page, output=str(output))

    # ── 편집 ─────────────────────────────────────────────────────────────
    def fill_fields(self, data: Mapping[str, Any]) -> Envelope:
        """누름틀 채우기."""
        return self._call("hwp_doc_fill_fields", data=dict(data))

    def replace_text(
        self, find: str, replace: str, *, case_sensitive: bool = True
    ) -> Envelope:
        """문자열 치환."""
        return self._call(
            "hwp_doc_replace_text", find=find, replace=replace, caseSensitive=case_sensitive
        )

    def set_cell(self, table: int, row: int, col: int, text: str) -> Envelope:
        """표 셀 기록."""
        return self._call("hwp_doc_set_cell", table=table, row=row, col=col, text=text)

    # ── 저장·정리 ────────────────────────────────────────────────────────
    def save(self, output: PathLike, *, verify: bool = False) -> Envelope:
        """저장. ``verify=True`` 면 저장 직후 자기검증 보고가 봉투에 담긴다."""
        return self._call("hwp_doc_save", output=str(output), verify=verify)

    def close(self) -> None:
        """핸들을 닫는다 (세션을 소유하면 서버도 함께 정리)."""
        if self._closed:
            return
        try:
            self._session.call("hwp_close", {"docId": self._doc_id})
        except RhwpError:
            # 이미 서버가 죽었거나 핸들이 만료된 경우 — 정리 경로에서
            # 새 예외를 만들지 않는다.
            pass
        finally:
            self._closed = True
            if self._owns_session:
                self._session.close()

    def __enter__(self) -> "Document":
        return self

    def __exit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc: Optional[BaseException],
        tb: Optional[TracebackType],
    ) -> None:
        self.close()

    def __repr__(self) -> str:  # pragma: no cover - 표현만
        state = "closed" if self._closed else "open"
        return f"Document({self._doc_id}, {state})"


def open(  # noqa: A001 - 모듈 함수로서 rhwp.open(...) 이 자연스럽다
    path: PathLike,
    *,
    password: Optional[str] = None,
    session: Optional[Session] = None,
    profile: Optional[str] = None,
) -> Document:
    """문서를 열어 핸들을 돌려준다.

    Args:
        path: 문서 경로.
        password: 보호 문서 암호.
        session: 이미 만든 세션에 얹고 싶을 때. 주면 문서를 닫아도 세션은 남는다.
        profile: 새 세션을 만들 때의 역할 프로필 (도구 노출 범위 제한).

    Returns:
        :class:`Document` — ``with`` 문에서 쓰면 자동으로 닫힌다.
    """
    owns = session is None
    sess = session if session is not None else Session(profile=profile)
    try:
        arguments: Dict[str, Any] = {"path": str(path)}
        if password is not None:
            arguments["password"] = password
        result = sess.call("hwp_open", arguments)
    except Exception:
        if owns:
            sess.close()
        raise

    doc_id = result.raw.get("docId")
    if not isinstance(doc_id, str) or not doc_id:
        if owns:
            sess.close()
        raise ProtocolError(f"hwp_open 이 docId 를 돌려주지 않았습니다: {result.raw}")
    return Document(sess, doc_id, owns_session=owns)
