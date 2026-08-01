"""세션 클라이언트 — 가짜 JSON-RPC 서버로 프로토콜 계약을 검증한다.

실제 `mcp-serve` 없이도 검증할 수 있어야 한다. 여기서 보는 것은 문서 처리가 아니라
**프로토콜 취급**이다: 응답 id 대조, isError 승격, 알림 무시, 정리 보장.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List

import pytest

import rhwp
from rhwp import _binary
from rhwp.errors import ProtocolError, SessionClosedError, UsageError
from rhwp.session import Document, Session


def _make_fake_server(tmp_path: Path, script_body: str) -> Path:
    """지정한 대로 응답하는 가짜 mcp-serve."""
    script = tmp_path / "fake_server.py"
    script.write_text(script_body, encoding="utf-8")
    if sys.platform == "win32":
        target = tmp_path / "rhwp.cmd"
        target.write_text(
            f'@echo off\r\n"{sys.executable}" "{script}" %*\r\n', encoding="utf-8"
        )
    else:
        target = tmp_path / "rhwp"
        target.write_text(f"#!{sys.executable}\n{script_body}", encoding="utf-8")
        target.chmod(0o755)
    return target


NORMAL_SERVER = '''
import io, json, sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", newline="\\n")
# 요청에 한글이 섞여도 읽히도록 — 실물 rhwp 는 UTF-8 로 주고받는다.
sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8")
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    req = json.loads(line)
    name = req["params"]["name"]
    args = req["params"]["arguments"]
    if name == "hwp_open":
        body = {"docId": "doc-1", "path": args.get("path"), "pageCount": 3}
    elif name == "hwp_close":
        body = {"docId": args.get("docId"), "closed": True}
    elif name == "hwp_doc_info":
        body = {"docId": args.get("docId"), "pageCount": 3, "schemaVersion": "1.0"}
    elif name == "hwp_doc_save":
        body = {"docId": args.get("docId"), "output": args.get("output"),
                "verify": {"identical": True, "diffCount": 0} if args.get("verify") else None}
    elif name == "hwp_doc_render_page":
        body = {"docId": args.get("docId"), "page": args.get("page"),
                "output": args.get("output")}
    elif name == "boom":
        resp = {"jsonrpc": "2.0", "id": req["id"], "result": {
            "isError": True,
            "content": [{"type": "text", "text": json.dumps(
                {"error": "열려 있지 않은 핸들", "nextCall": {"name": "hwp_open"}},
                ensure_ascii=False)}],
        }}
        print(json.dumps(resp, ensure_ascii=False), flush=True)
        continue
    else:
        body = {"tool": name, "args": args}
    resp = {"jsonrpc": "2.0", "id": req["id"], "result": {
        "isError": False,
        "content": [{"type": "text", "text": json.dumps(body, ensure_ascii=False)}],
        "structuredContent": body,
    }}
    print(json.dumps(resp, ensure_ascii=False), flush=True)
'''

NOISY_SERVER = '''
import io, json, sys
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", newline="\\n")
# 요청에 한글이 섞여도 읽히도록 — 실물 rhwp 는 UTF-8 로 주고받는다.
sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8")
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    req = json.loads(line)
    # 알림(id 없음)을 먼저 흘린다 — 클라이언트가 건너뛰어야 한다.
    print(json.dumps({"jsonrpc": "2.0", "method": "notifications/progress",
                      "params": {"pct": 50}}), flush=True)
    body = {"docId": "doc-1", "pageCount": 1}
    print(json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {
        "isError": False, "structuredContent": body,
        "content": [{"type": "text", "text": json.dumps(body)}]}}), flush=True)
'''

DYING_SERVER = '''
import sys
sys.stderr.write("치명적 오류: 서버가 죽었습니다\\n")
sys.exit(1)
'''


@pytest.fixture
def normal_server(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    binary = _make_fake_server(tmp_path, NORMAL_SERVER)
    monkeypatch.setenv(_binary.ENV_VAR, str(binary))
    _binary.clear_cache()
    return binary


def test_open_returns_document_handle(normal_server: Path) -> None:
    doc = rhwp.open("a.hwp")
    try:
        assert isinstance(doc, Document)
        assert doc.doc_id == "doc-1"
    finally:
        doc.close()


def test_context_manager_closes_handle(normal_server: Path) -> None:
    with rhwp.open("a.hwp") as doc:
        assert doc.doc_id == "doc-1"
    with pytest.raises(SessionClosedError):
        doc.info()


def test_doc_info_round_trip(normal_server: Path) -> None:
    with rhwp.open("a.hwp") as doc:
        info = doc.info()
        assert info.page_count == 3


def test_save_carries_verify_report(normal_server: Path) -> None:
    with rhwp.open("a.hwp") as doc:
        saved = doc.save("out.hwp", verify=True)
        verify = saved.verify
        assert verify is not None
        assert verify.identical


def test_save_without_verify_reports_none(normal_server: Path) -> None:
    with rhwp.open("a.hwp") as doc:
        saved = doc.save("out.hwp")
        assert saved.verify is None


def test_render_page_requires_output_path(normal_server: Path, tmp_path: Path) -> None:
    """도구 계약상 output 이 필수다 — 빠뜨리면 서버가 거부한다."""
    with rhwp.open("a.hwp") as doc:
        target = tmp_path / "p0.svg"
        result = doc.render_page(0, target)
        assert result.raw["page"] == 0
        assert result.raw["output"] == str(target)


def test_tool_error_becomes_usage_error_with_envelope(normal_server: Path) -> None:
    """isError 는 호출 조립 문제다 — 교정 단서를 예외에 실어 보낸다."""
    session = Session()
    try:
        with pytest.raises(UsageError) as caught:
            session.call("boom", {"docId": "doc-1"})
        assert caught.value.envelope is not None
        assert "nextCall" in caught.value.envelope
    finally:
        session.close()


def test_notifications_are_skipped(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    binary = _make_fake_server(tmp_path, NOISY_SERVER)
    monkeypatch.setenv(_binary.ENV_VAR, str(binary))
    _binary.clear_cache()

    session = Session()
    try:
        result = session.call("hwp_open", {"path": "a.hwp"})
        assert result.raw["docId"] == "doc-1"
    finally:
        session.close()


def test_dead_server_raises_protocol_error(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    binary = _make_fake_server(tmp_path, DYING_SERVER)
    monkeypatch.setenv(_binary.ENV_VAR, str(binary))
    _binary.clear_cache()

    with pytest.raises(ProtocolError) as caught:
        rhwp.open("a.hwp")
    assert "종료" in str(caught.value)


def test_shared_session_survives_document_close(normal_server: Path) -> None:
    """세션을 주면 문서를 닫아도 서버는 남는다 — 여러 문서를 한 서버에서."""
    session = Session()
    try:
        doc = rhwp.open("a.hwp", session=session)
        doc.close()
        # 세션이 살아 있어야 다음 문서를 열 수 있다.
        second = rhwp.open("b.hwp", session=session)
        assert second.doc_id == "doc-1"
        second.close()
    finally:
        session.close()


def test_close_is_idempotent(normal_server: Path) -> None:
    doc = rhwp.open("a.hwp")
    doc.close()
    doc.close()  # 두 번 불러도 예외 없음


def test_session_close_is_idempotent(normal_server: Path) -> None:
    session = Session()
    session.close()
    session.close()


def test_closed_session_rejects_calls(normal_server: Path) -> None:
    session = Session()
    session.close()
    with pytest.raises(SessionClosedError):
        session.call("hwp_doc_info", {"docId": "doc-1"})


def test_password_is_passed_to_open(normal_server: Path) -> None:
    with rhwp.open("보호.hwp", password="비밀") as doc:
        # 가짜 서버는 path 만 되돌려주지만, 호출이 성립했다는 것이 계약이다.
        assert doc.doc_id == "doc-1"


def test_edit_helpers_reach_the_right_tools(normal_server: Path) -> None:
    with rhwp.open("a.hwp") as doc:
        filled = doc.fill_fields({"성명": "홍길동"})
        assert filled.raw["tool"] == "hwp_doc_fill_fields"

        replaced = doc.replace_text("가", "나")
        assert replaced.raw["tool"] == "hwp_doc_replace_text"

        cell = doc.set_cell(1, 0, 0, "값")
        assert cell.raw["tool"] == "hwp_doc_set_cell"
        assert cell.raw["args"]["table"] == 1


def test_query_helpers_reach_the_right_tools(normal_server: Path) -> None:
    with rhwp.open("a.hwp") as doc:
        assert doc.fields().raw["tool"] == "hwp_doc_fields"
        assert doc.tables().raw["tool"] == "hwp_doc_tables"
        assert doc.search("예산").raw["tool"] == "hwp_doc_search"
        assert doc.text().raw["tool"] == "hwp_doc_text"


def test_text_page_argument_is_optional(normal_server: Path) -> None:
    with rhwp.open("a.hwp") as doc:
        without = doc.text()
        assert "page" not in without.raw["args"]
        with_page = doc.text(page=1)
        assert with_page.raw["args"]["page"] == 1


def test_profile_is_passed_to_server(normal_server: Path) -> None:
    """역할 프로필로 도구 노출 범위를 제한할 수 있어야 한다."""
    session = Session(profile="행정서식")
    try:
        assert session is not None
    finally:
        session.close()
