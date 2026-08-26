#!/usr/bin/env python3
"""[#5293] rhwp-mcp-session 레퍼런스·픽스처 생성기.

도구 이름은 src/agent_profiles.rs 의 ALL_SESSION_TOOLS 와 src/main.rs 의
mcp_tool_definitions() 에서만 읽는다. 새 도구를 여기서 발명하지 않는다.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SKILL = Path(__file__).resolve().parents[1]
REF = SKILL / "references"
FIXT = REF / "fixtures"


def extract_session_tools() -> list[str]:
    text = (ROOT / "src" / "agent_profiles.rs").read_text(encoding="utf-8")
    block = re.search(r"ALL_SESSION_TOOLS:.*?= &\[(.*?)\];", text, re.S)
    if not block:
        raise SystemExit("ALL_SESSION_TOOLS 를 찾지 못했다")
    return re.findall(r'"(hwp_[a-z0-9_]+)"', block.group(1))


def extract_read_tools() -> list[str]:
    text = (ROOT / "src" / "agent_profiles.rs").read_text(encoding="utf-8")
    block = re.search(r"SESSION_READ_TOOLS:.*?= &\[(.*?)\];", text, re.S)
    if not block:
        raise SystemExit("SESSION_READ_TOOLS 를 찾지 못했다")
    return re.findall(r'"(hwp_[a-z0-9_]+)"', block.group(1))


def extract_stateless_tools() -> list[str]:
    text = (ROOT / "src" / "main.rs").read_text(encoding="utf-8")
    start = text.find("fn mcp_tool_definitions()")
    if start < 0:
        raise SystemExit("mcp_tool_definitions 를 찾지 못했다")
    chunk = text[start:]
    names = re.findall(
        r'tool(?:_with_optional_args)?\(\s*"(hwp_[a-z0-9_]+)"',
        chunk,
    )
    # 선언 순서를 유지한 채 중복만 제거
    seen: set[str] = set()
    out: list[str] = []
    for name in names:
        if name not in seen:
            seen.add(name)
            out.append(name)
    return out


def extract_profiles() -> list[dict]:
    text = (ROOT / "src" / "agent_profiles.rs").read_text(encoding="utf-8")
    names = re.findall(r'name: "([^"]+)"', text)
    # PROFILES 배열만 — 파일 앞쪽 상수 이름은 제외
    start = text.find("pub const PROFILES")
    names = re.findall(r'name: "([^"]+)"', text[start:])
    return [{"name": n} for n in names]


# 세션↔무상태 짝. 양쪽 모두 실존 도구일 때만 픽스처에 싣는다.
PAIRING_CANDIDATES = {
    "hwp_doc_info": "hwp_info",
    "hwp_doc_text": "hwp_export_text",
    "hwp_doc_fields": "hwp_fields",
    "hwp_doc_tables": "hwp_export_tables",
    "hwp_doc_search": "hwp_search",
    "hwp_doc_render_page": "hwp_export_svg",
    "hwp_doc_structure": "hwp_export_structure",
    "hwp_doc_extract_data": "hwp_extract_data",
    "hwp_doc_replace_text": "hwp_replace_text",
    "hwp_doc_set_cell": "hwp_set_cell",
    "hwp_doc_fill_fields": "hwp_fill_fields",
}

SESSION_META = {
    "hwp_open": {
        "family": "lifecycle",
        "required": ["path"],
        "optional": ["password"],
        "envelope": ["docId", "pageCount", "source", "schemaVersion"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": False,
        "destructive": False,
        "when": "같은 문서를 두 번 이상 조회·편집할 때 파싱 1회로 핸들을 연다.",
        "when_not": "호출 하나면 끝인 단건 작업. 그때는 무상태 도구가 싸다.",
        "next_after_error": "path 를 절대 경로로 고친 뒤 다시 hwp_open.",
    },
    "hwp_close": {
        "family": "lifecycle",
        "required": ["docId"],
        "optional": [],
        "envelope": ["closed", "docId", "schemaVersion"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "핸들을 더 쓰지 않을 때 메모리를 해제한다.",
        "when_not": "저장하지 않은 편집을 남긴 채 닫으면 인메모리 누적이 사라진다.",
        "next_after_error": "이미 닫혔으면 성공으로 보고 끝낸다. 다시 쓰려면 hwp_open.",
    },
    "hwp_doc_info": {
        "family": "query",
        "required": ["docId"],
        "optional": [],
        "envelope": ["format", "pageCount", "paraCount", "fonts", "title", "warnings"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "편집 후 pageCount 변화를 추적하거나 규모를 재확인할 때.",
        "when_not": "파일을 한 번만 보고 끝이면 hwp_info.",
        "next_after_error": "nextCall.name=hwp_open 으로 docId 재발급.",
    },
    "hwp_doc_text": {
        "family": "query",
        "required": ["docId"],
        "optional": ["page", "maxChars", "charOffset"],
        "envelope": ["pages", "truncated", "omittedCount", "nextOffset"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "연 핸들에서 쪽 본문을 이어 읽을 때. nextOffset 으로 창을 잇는다.",
        "when_not": "전문을 한 번만 뽑으면 hwp_export_text.",
        "next_after_error": "핸들 만료면 hwp_open. 쪽 범위 밖이면 page 를 고친다.",
    },
    "hwp_doc_fields": {
        "family": "query",
        "required": ["docId"],
        "optional": [],
        "envelope": ["fieldCount", "fields", "textSecurity"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "hwp_doc_fill_fields 전에 이름·반복 순번을 조사할 때.",
        "when_not": "서식 한 번 조사 후 프로세스 종료면 hwp_fields.",
        "next_after_error": "nextCall 로 hwp_open.",
    },
    "hwp_doc_tables": {
        "family": "query",
        "required": ["docId"],
        "optional": [],
        "envelope": ["tableCount", "tables"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "hwp_doc_set_cell 전에 표 번호·병합 범위를 확인할 때.",
        "when_not": "표만 한 번 뽑아 CSV 로 넘기면 hwp_export_tables 또는 hwp_table_to_csv.",
        "next_after_error": "nextCall 로 hwp_open.",
    },
    "hwp_doc_search": {
        "family": "query",
        "required": ["docId", "query"],
        "optional": ["caseSensitive", "maxMatches", "offset"],
        "envelope": [
            "matchCount",
            "totalMatchCount",
            "truncated",
            "omittedCount",
            "matches",
            "nextOffset",
        ],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "대형 문서에서 '어디를 고칠까'를 반복 탐색할 때.",
        "when_not": "검색 1회면 hwp_search. 폴더 전수면 hwp_batch_search.",
        "next_after_error": "핸들 만료면 hwp_open. query 누락이면 인자를 고친다(재시도 금지).",
    },
    "hwp_doc_render_page": {
        "family": "query",
        "required": ["docId", "page", "output"],
        "optional": [],
        "envelope": ["output", "page", "bytes"],
        "writes_ir": False,
        "writes_disk": True,
        "idempotent": True,
        "destructive": False,
        "when": "changedPages 쪽만 SVG 로 눈검증할 때.",
        "when_not": "문서 전체를 SVG 로 한 번에 렌더하면 hwp_export_svg.",
        "next_after_error": "output 은 절대 경로. page 는 0 기준.",
    },
    "hwp_doc_structure": {
        "family": "query",
        "required": ["docId"],
        "optional": ["mode"],
        "envelope": ["schemaVersion", "source", "mode", "nodeCount", "structure"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "법령·규정 조문 계층을 세션 안에서 반복 인용할 때.",
        "when_not": "목차 한 번이면 hwp_export_structure. 안정 노드 ID 가 필요하면 hwp_doc_tree.",
        "next_after_error": "nextCall 로 hwp_open.",
    },
    "hwp_doc_extract_data": {
        "family": "query",
        "required": ["docId"],
        "optional": ["kind", "limit"],
        "envelope": [
            "schemaVersion",
            "source",
            "kind",
            "itemCount",
            "totalItemCount",
            "truncated",
            "counts",
            "items",
        ],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "연 핸들에서 날짜·금액·수량을 반복 좁혀 뽑을 때.",
        "when_not": "단건 수확은 hwp_extract_data. 폴더 전수는 hwp_batch_extract_data.",
        "next_after_error": "nextCall 로 hwp_open.",
    },
    "hwp_doc_tree": {
        "family": "query",
        "required": ["docId"],
        "optional": [],
        "envelope": ["nodes"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "페이지 p0..·표 t0.. 안정 ID 로 구조를 볼 때(#4357).",
        "when_not": "제목·조문 의미 계층은 hwp_doc_structure.",
        "next_after_error": "nextCall 로 hwp_open.",
    },
    "hwp_doc_replace_text": {
        "family": "mutate",
        "required": ["docId", "find", "replace"],
        "optional": ["caseSensitive"],
        "envelope": ["replacedCount", "changedPages"],
        "writes_ir": True,
        "writes_disk": False,
        "idempotent": False,
        "destructive": False,
        "when": "연 문서에서 문구를 누적 치환하고 나중에 한 번 저장할 때.",
        "when_not": "치환 1회 후 파일만 필요하면 hwp_replace_text.",
        "next_after_error": "replacedCount 0 은 오류가 아니다. 핸들 만료만 hwp_open.",
    },
    "hwp_doc_set_cell": {
        "family": "mutate",
        "required": ["docId", "table", "row", "col", "text"],
        "optional": ["keepStyle"],
        "envelope": ["overflow", "changedPages"],
        "writes_ir": True,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "누름틀 없는 칸을 좌표로 채울 때. 먼저 hwp_doc_tables.",
        "when_not": "칸 하나 고치고 끝이면 hwp_set_cell.",
        "next_after_error": "병합 덮인 칸은 앵커 좌표로 고친다. 재시도로 우회하지 않는다.",
    },
    "hwp_doc_fill_fields": {
        "family": "mutate",
        "required": ["docId", "data"],
        "optional": [],
        "envelope": ["filledCount", "notFound", "ambiguous", "changedPages"],
        "writes_ir": True,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "같은 서식에 값을 여러 번 누적 채울 때.",
        "when_not": "채움 1회면 hwp_fill_fields. 서식1+데이터N은 hwp_batch_fill.",
        "next_after_error": "notFound/ambiguous 는 isError:false 데이터다. 이름을 고친다.",
    },
    "hwp_doc_save": {
        "family": "persist",
        "required": ["docId", "output"],
        "optional": ["verify"],
        "envelope": ["output", "format", "bytes", "verify"],
        "writes_ir": False,
        "writes_disk": True,
        "idempotent": True,
        "destructive": True,
        "when": "누적 편집을 형식 보존으로 기록할 때. 세션의 유일한 기록 지점.",
        "when_not": "저장 없이 조회만 했으면 호출하지 않는다.",
        "next_after_error": "output 은 절대 경로. 원본 덮어쓰기는 의도일 때만.",
    },
    "hwp_ws_list": {
        "family": "workspace",
        "required": [],
        "optional": [],
        "envelope": ["entries", "truncated"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "mcp-serve --workspace 로 기동한 코퍼스 인벤토리를 볼 때.",
        "when_not": "워크스페이스 없이 기동했으면 이 도구는 실패한다. 경로로 hwp_open.",
        "next_after_error": "서버를 --workspace 로 다시 붙이거나 hwp_open 으로 전환.",
    },
    "hwp_ws_open": {
        "family": "workspace",
        "required": ["id"],
        "optional": ["password"],
        "envelope": ["docId", "pageCount", "source", "schemaVersion"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": False,
        "destructive": False,
        "when": "hwp_ws_list 의 w1.. id 로 핸들을 열 때.",
        "when_not": "경로를 알면 hwp_open.",
        "next_after_error": "id 는 hwp_ws_list 의 entries[].id 만.",
    },
    "hwp_ws_journal": {
        "family": "workspace",
        "required": [],
        "optional": [],
        "envelope": ["entries"],
        "writes_ir": False,
        "writes_disk": False,
        "idempotent": True,
        "destructive": False,
        "when": "변이 도구 전/후 본문 SHA-256 을 자기검증할 때.",
        "when_not": "조회만 한 세션에서는 저널이 비어 있는 것이 정상이다.",
        "next_after_error": "워크스페이스 기동이 아니면 저널 축을 쓰지 않는다.",
    },
}


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(obj, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def build_allowlist(session: list[str], stateless: list[str], read: list[str]) -> dict:
    pairing = []
    for session_name, twin in PAIRING_CANDIDATES.items():
        if session_name in session and twin in stateless:
            pairing.append(
                {
                    "session": session_name,
                    "stateless": twin,
                    "same_envelope": True,
                    "note": "세션 판의 봉투 어휘는 무상태 대응 도구와 동형이다.",
                }
            )
    mutate = [n for n in session if SESSION_META.get(n, {}).get("writes_ir")]
    persist = [n for n in session if SESSION_META.get(n, {}).get("writes_disk") and n == "hwp_doc_save"]
    return {
        "issue": 5293,
        "ssot": {
            "stateless": "rhwp capabilities --mcp  (src/main.rs mcp_tool_definitions)",
            "session": "mcp-serve tools/list  (src/agent_profiles.rs ALL_SESSION_TOOLS)",
            "rule": "문서의 개수·이름이 바이너리와 다르면 바이너리가 이긴다. 도구를 발명하지 않는다.",
        },
        "session_tools": session,
        "session_read_tools": read,
        "stateless_tools": stateless,
        "pairing": pairing,
        "mutate_tools": mutate,
        "persist_tools": persist,
        "lifecycle": ["hwp_open", "hwp_doc_*", "hwp_close"],
        "counts_are_not_contracts": True,
    }


def build_tool_card(name: str, session: list[str], stateless: list[str]) -> dict:
    meta = SESSION_META[name]
    twin = PAIRING_CANDIDATES.get(name)
    if twin and twin not in stateless:
        twin = None
    return {
        "name": name,
        "kind": "session",
        "in_capabilities_mcp": False,
        "in_tools_list": True,
        "family": meta["family"],
        "required": meta["required"],
        "optional": meta["optional"],
        "envelope_fields": meta["envelope"],
        "writes_ir": meta["writes_ir"],
        "writes_disk": meta["writes_disk"],
        "idempotentHint": meta["idempotent"],
        "destructiveHint": meta["destructive"],
        "stateless_twin": twin,
        "when": meta["when"],
        "when_not": meta["when_not"],
        "recovery": meta["next_after_error"],
        "sample_call": {
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": {k: f"<{k}>" for k in meta["required"]},
            },
        },
        "closed_handle_signal": (
            None
            if name in ("hwp_open", "hwp_ws_list", "hwp_ws_journal")
            else {
                "isError": True,
                "needle": "열려 있지 않은 핸들",
                "nextCall": {"name": "hwp_open"},
            }
        ),
        "source_const": "ALL_SESSION_TOOLS" if name in session else "UNKNOWN",
    }


TRACE_SPECS = [
    (
        "01_open_info_close",
        "최소 수명 — 열고 규모를 보고 닫는다.",
        [
            ("hwp_open", {"path": "C:/abs/편람.hwp"}, "ok", ["docId", "pageCount"]),
            ("hwp_doc_info", {"docId": "doc-1"}, "ok", ["format", "pageCount"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "02_open_search_text_close",
        "대형 문서 반복 조회 — 검색으로 쪽을 좁힌 뒤 본문을 읽는다.",
        [
            ("hwp_open", {"path": "C:/abs/편람.hwp"}, "ok", ["docId"]),
            ("hwp_doc_search", {"docId": "doc-1", "query": "위임전결"}, "ok", ["matches"]),
            ("hwp_doc_text", {"docId": "doc-1", "page": 41}, "ok", ["pages"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "03_fill_render_save_close",
        "서식 채움 — 조사→채움→눈검증→저장. 디스크는 save 한 번.",
        [
            ("hwp_open", {"path": "C:/abs/서식.hwp"}, "ok", ["docId"]),
            ("hwp_doc_fields", {"docId": "doc-1"}, "ok", ["fields"]),
            (
                "hwp_doc_fill_fields",
                {"docId": "doc-1", "data": {"회사명": "페타플로"}},
                "envelope",
                ["filledCount", "notFound", "changedPages"],
            ),
            (
                "hwp_doc_render_page",
                {"docId": "doc-1", "page": 0, "output": "C:/abs/out/p0.svg"},
                "ok",
                ["output"],
            ),
            (
                "hwp_doc_save",
                {"docId": "doc-1", "output": "C:/abs/out/저장본.hwp", "verify": True},
                "ok",
                ["output", "verify"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "04_tables_set_cell_save",
        "누름틀 없는 표 — 좌표 확인 후 칸을 쓰고 저장한다.",
        [
            ("hwp_open", {"path": "C:/abs/복학원서.hwp"}, "ok", ["docId"]),
            ("hwp_doc_tables", {"docId": "doc-1"}, "ok", ["tables"]),
            (
                "hwp_doc_set_cell",
                {"docId": "doc-1", "table": 0, "row": 1, "col": 1, "text": "홍길동"},
                "envelope",
                ["overflow", "changedPages"],
            ),
            (
                "hwp_doc_save",
                {"docId": "doc-1", "output": "C:/abs/out/복학원서-채움.hwp", "verify": True},
                "ok",
                ["output"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "05_replace_then_save",
        "치환은 IR 누적. replacedCount 0 은 오류가 아니다.",
        [
            ("hwp_open", {"path": "C:/abs/공문.hwp"}, "ok", ["docId"]),
            (
                "hwp_doc_replace_text",
                {"docId": "doc-1", "find": "구명칭", "replace": "신명칭"},
                "envelope",
                ["replacedCount", "changedPages"],
            ),
            (
                "hwp_doc_save",
                {"docId": "doc-1", "output": "C:/abs/out/공문-치환.hwp"},
                "ok",
                ["output"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "06_structure_and_extract",
        "조문 계층과 날짜·금액을 한 핸들에서 반복 조회한다.",
        [
            ("hwp_open", {"path": "C:/abs/규정.hwp"}, "ok", ["docId"]),
            ("hwp_doc_structure", {"docId": "doc-1", "mode": "clause"}, "ok", ["structure"]),
            (
                "hwp_doc_extract_data",
                {"docId": "doc-1", "kind": "amount", "limit": 20},
                "ok",
                ["items", "totalItemCount"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "07_workspace_open",
        "워크스페이스 id 로 연다. --workspace 기동 전제.",
        [
            ("hwp_ws_list", {}, "ok", ["entries"]),
            ("hwp_ws_open", {"id": "w1"}, "ok", ["docId"]),
            ("hwp_doc_tree", {"docId": "doc-1"}, "ok", ["nodes"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "08_mutate_journal",
        "변이 뒤 저널로 전/후 digest 를 확인한다.",
        [
            ("hwp_ws_list", {}, "ok", ["entries"]),
            ("hwp_ws_open", {"id": "w1"}, "ok", ["docId"]),
            (
                "hwp_doc_replace_text",
                {"docId": "doc-1", "find": "A", "replace": "B"},
                "envelope",
                ["replacedCount"],
            ),
            ("hwp_ws_journal", {}, "ok", ["entries"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "09_closed_handle_recovery",
        "닫힌 핸들 재사용 — isError 와 nextCall 로 복구한다.",
        [
            ("hwp_open", {"path": "C:/abs/편람.hwp"}, "ok", ["docId"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
            ("hwp_doc_text", {"docId": "doc-1"}, "isError", ["nextCall"]),
            ("hwp_open", {"path": "C:/abs/편람.hwp"}, "ok", ["docId"]),
            ("hwp_doc_text", {"docId": "doc-2"}, "ok", ["pages"]),
            ("hwp_close", {"docId": "doc-2"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "10_stateless_one_shot_info",
        "단건 규모 파악은 세션을 열지 않는다.",
        [
            ("hwp_info", {"path": "C:/abs/편람.hwp"}, "ok", ["format", "pageCount"]),
        ],
        "stateless",
    ),
    (
        "11_stateless_batch_search",
        "폴더 전수 검색은 stdin 배치 도구.",
        [
            (
                "hwp_batch_search",
                {"query": "위임전결", "paths": ["C:/abs/a.hwp", "C:/abs/b.hwp"]},
                "ok",
                ["content_text_ndjson"],
            ),
        ],
        "stateless",
    ),
    (
        "12_text_window_follow",
        "maxChars 창을 nextOffset 으로 잇는다. truncated 만 보고 끝내지 않는다.",
        [
            ("hwp_open", {"path": "C:/abs/편람.hwp"}, "ok", ["docId"]),
            (
                "hwp_doc_text",
                {"docId": "doc-1", "maxChars": 2000, "charOffset": 0},
                "ok",
                ["nextOffset", "truncated"],
            ),
            (
                "hwp_doc_text",
                {"docId": "doc-1", "maxChars": 2000, "charOffset": 2000},
                "ok",
                ["pages"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "13_search_window_follow",
        "검색 이어보기 — 더 있는지는 nextOffset 으로 판정한다.",
        [
            ("hwp_open", {"path": "C:/abs/편람.hwp"}, "ok", ["docId"]),
            (
                "hwp_doc_search",
                {"docId": "doc-1", "query": "조", "maxMatches": 20, "offset": 0},
                "ok",
                ["nextOffset", "totalMatchCount"],
            ),
            (
                "hwp_doc_search",
                {"docId": "doc-1", "query": "조", "maxMatches": 20, "offset": 20},
                "ok",
                ["matches"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "14_multi_doc_handles",
        "프로세스당 핸들은 여러 개. docId 를 섞지 않는다.",
        [
            ("hwp_open", {"path": "C:/abs/원본.hwp"}, "ok", ["docId"]),
            ("hwp_open", {"path": "C:/abs/비교.hwp"}, "ok", ["docId"]),
            ("hwp_doc_info", {"docId": "doc-1"}, "ok", ["pageCount"]),
            ("hwp_doc_info", {"docId": "doc-2"}, "ok", ["pageCount"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
            ("hwp_close", {"docId": "doc-2"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "15_save_keeps_handle",
        "저장 후에도 핸들은 살아 있어 이어서 편집할 수 있다.",
        [
            ("hwp_open", {"path": "C:/abs/서식.hwp"}, "ok", ["docId"]),
            (
                "hwp_doc_fill_fields",
                {"docId": "doc-1", "data": {"성명": "홍길동"}},
                "envelope",
                ["filledCount"],
            ),
            (
                "hwp_doc_save",
                {"docId": "doc-1", "output": "C:/abs/out/1차.hwp"},
                "ok",
                ["output"],
            ),
            (
                "hwp_doc_fill_fields",
                {"docId": "doc-1", "data": {"생년월일": "1990-01-01"}},
                "envelope",
                ["filledCount"],
            ),
            (
                "hwp_doc_save",
                {"docId": "doc-1", "output": "C:/abs/out/2차.hwp", "verify": True},
                "ok",
                ["output"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "16_abandon_edits",
        "저장하지 않고 닫으면 인메모리 누적은 사라진다. 이것이 계약이다.",
        [
            ("hwp_open", {"path": "C:/abs/서식.hwp"}, "ok", ["docId"]),
            (
                "hwp_doc_replace_text",
                {"docId": "doc-1", "find": "임시", "replace": "폐기"},
                "envelope",
                ["replacedCount"],
            ),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "17_password_open",
        "암호 문서는 password writeOnly. 응답에 값이 실리지 않는다.",
        [
            (
                "hwp_open",
                {"path": "C:/abs/보호.hwp", "password": "<not-logged-in-docs>"},
                "ok",
                ["docId"],
            ),
            ("hwp_doc_info", {"docId": "doc-1"}, "ok", ["format"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "session",
    ),
    (
        "18_unknown_tool_didyoumean",
        "없는 이름은 발명하지 말고 didYouMean/nextCall 을 따른다.",
        [
            ("hwp_doc_foo", {"docId": "doc-1"}, "isError", ["didYouMean", "nextCall"]),
        ],
        "recovery",
    ),
    (
        "19_relative_path_trap",
        "상대 경로는 서버 cwd 기준. MCP 로는 절대 경로만.",
        [
            ("hwp_open", {"path": "편람.hwp"}, "isError", ["error"]),
            ("hwp_open", {"path": "C:/abs/편람.hwp"}, "ok", ["docId"]),
            ("hwp_close", {"docId": "doc-1"}, "ok", ["closed"]),
        ],
        "recovery",
    ),
    (
        "20_ir_diff_is_data",
        "identical:false 는 isError:false. 차이 발견은 오류가 아니다.",
        [
            (
                "hwp_ir_diff",
                {"a": "C:/abs/before.hwp", "b": "C:/abs/after.hwp"},
                "envelope",
                ["identical", "diffCount"],
            ),
        ],
        "stateless",
    ),
]


ERROR_SPECS = [
    {
        "id": "rpc_parse",
        "layer": "jsonrpc",
        "code": -32700,
        "symptom": "JSON 파싱 실패",
        "retry": False,
        "fix": "한 줄에 JSON-RPC 객체 하나. stdout 로그를 섞지 않는다.",
        "tools": [],
    },
    {
        "id": "rpc_invalid_request",
        "layer": "jsonrpc",
        "code": -32600,
        "symptom": "Invalid Request",
        "retry": False,
        "fix": "jsonrpc/method/id 구조를 고친다.",
        "tools": [],
    },
    {
        "id": "rpc_method_not_found",
        "layer": "jsonrpc",
        "code": -32601,
        "symptom": "지원하지 않는 메서드",
        "retry": False,
        "fix": "initialize / notifications/initialized / ping / tools/list / tools/call / resources/list / resources/read 만 쓴다.",
        "tools": [],
    },
    {
        "id": "rpc_invalid_params",
        "layer": "jsonrpc",
        "code": -32602,
        "symptom": "params.name 이 필요합니다",
        "retry": False,
        "fix": "tools/call 의 params.name 과 params.arguments 를 채운다.",
        "tools": [],
    },
    {
        "id": "rpc_resource_not_found",
        "layer": "jsonrpc",
        "code": -32002,
        "symptom": "알 수 없는 리소스",
        "retry": False,
        "fix": "resources/list 로 URI 를 다시 읽는다. 추측 URI 를 만들지 않는다.",
        "tools": [],
    },
    {
        "id": "tool_unknown",
        "layer": "isError",
        "code": None,
        "symptom": "알 수 없는 도구",
        "retry": False,
        "fix": "didYouMean[0] 또는 tools/list. 도구를 발명하지 않는다.",
        "tools": [],
    },
    {
        "id": "tool_missing_path",
        "layer": "isError",
        "code": None,
        "symptom": "path 가 필요합니다",
        "retry": False,
        "fix": "필수 인자를 채운다. 같은 호출을 재시도하지 않는다.",
        "tools": ["hwp_open"],
    },
    {
        "id": "tool_missing_docid",
        "layer": "isError",
        "code": None,
        "symptom": "docId 가 필요합니다",
        "retry": False,
        "fix": "인자 이름은 handle 이 아니라 docId.",
        "tools": ["hwp_doc_text", "hwp_close"],
    },
    {
        "id": "tool_closed_handle",
        "layer": "isError",
        "code": None,
        "symptom": "열려 있지 않은 핸들",
        "retry": True,
        "fix": "nextCall.name=hwp_open 으로 docId 를 재발급한 뒤 같은 조회를 반복한다.",
        "tools": ["hwp_doc_text", "hwp_doc_info", "hwp_doc_search", "hwp_close"],
    },
    {
        "id": "tool_missing_file",
        "layer": "isError",
        "code": None,
        "symptom": "파일을 읽을 수 없습니다",
        "retry": False,
        "fix": "절대 경로와 존재 여부를 확인한다. CLI exit 1 대응.",
        "tools": ["hwp_open", "hwp_info"],
    },
    {
        "id": "tool_usage",
        "layer": "isError",
        "code": None,
        "symptom": "종료 코드 2",
        "retry": False,
        "fix": "호출 조립 버그. 쪽 번호·표 좌표·필수 플래그를 고친다.",
        "tools": ["hwp_export_text", "hwp_doc_render_page"],
    },
    {
        "id": "tool_profile_blocked",
        "layer": "isError",
        "code": None,
        "symptom": "현재 프로필에서는 세션 도구를 제공하지 않습니다",
        "retry": False,
        "fix": "프로필을 바꾸거나 무상태 짝으로 전환한다. 우회 호출은 서버가 막는다.",
        "tools": ["hwp_open"],
    },
    {
        "id": "env_not_found",
        "layer": "envelope",
        "code": None,
        "symptom": "notFound",
        "retry": False,
        "fix": "isError:false. 필드 이름을 hwp_doc_fields 로 다시 읽고 '이름[N]' 으로 지목한다.",
        "tools": ["hwp_doc_fill_fields", "hwp_fill_fields"],
    },
    {
        "id": "env_identical_false",
        "layer": "envelope",
        "code": None,
        "symptom": "identical:false",
        "retry": False,
        "fix": "차이 발견은 데이터. categories/diffCount 를 읽고 사람 큐로 넘긴다.",
        "tools": ["hwp_ir_diff"],
    },
    {
        "id": "env_replaced_zero",
        "layer": "envelope",
        "code": None,
        "symptom": "replacedCount:0",
        "retry": False,
        "fix": "오류가 아니라 계수 보고. find 문자열을 문서에서 다시 확인한다.",
        "tools": ["hwp_doc_replace_text", "hwp_replace_text"],
    },
    {
        "id": "env_overflow",
        "layer": "envelope",
        "code": None,
        "symptom": "overflow",
        "retry": False,
        "fix": "칸은 쓰였다. 넘침은 보고일 뿐. 필요하면 문구를 줄이거나 칸을 키운다.",
        "tools": ["hwp_doc_set_cell", "hwp_set_cell"],
    },
    {
        "id": "env_truncated",
        "layer": "envelope",
        "code": None,
        "symptom": "truncated:true",
        "retry": True,
        "fix": "더 있는지는 nextOffset. 없으면 끝. truncated 만으로 재호출하지 않는다.",
        "tools": ["hwp_doc_text", "hwp_doc_search", "hwp_doc_extract_data"],
    },
    {
        "id": "env_plan_invalid",
        "layer": "envelope",
        "code": None,
        "symptom": "invalid != []",
        "retry": False,
        "fix": "MCP 에서 isError:false 여도 선검증 실패다. invalid 를 게이트로 건다.",
        "tools": ["hwp_run_plan"],
    },
]


DECISION_SPECS = [
    ("once_info", "파일 하나 쪽수·형식만", "stateless", "hwp_info", "세션 오버헤드가 이득을 넘는다."),
    ("once_search", "파일 하나에서 검색어 위치 1회", "stateless", "hwp_search", "호출 하나 = 작업 하나."),
    ("once_fill", "서식 하나 채우고 끝", "stateless", "hwp_fill_fields", "단건 채움은 CLI 계약 그대로."),
    ("once_table_csv", "표 하나를 CSV 로", "stateless", "hwp_table_to_csv", "세션 짝이 없다."),
    ("once_structure", "법령 목차 1회", "stateless", "hwp_export_structure", "한 번이면 무상태."),
    ("once_extract", "날짜·금액 1회 수확", "stateless", "hwp_extract_data", "한 번이면 무상태."),
    ("once_svg", "문서 전체 SVG", "stateless", "hwp_export_svg", "전 쪽 렌더는 무상태."),
    ("once_pdf", "제출용 PDF", "stateless", "hwp_export_pdf", "세션에 PDF 도구가 없다."),
    ("once_ir_diff", "두 파일 IR 비교", "stateless", "hwp_ir_diff", "세션 밖 두 경로 비교."),
    ("once_verify", "저장본 사후검증", "stateless", "hwp_verify", "세션 밖 파일 게이트."),
    ("folder_info", "폴더 메타 스윕", "stateless", "hwp_batch", "stdin 경로 목록."),
    ("folder_search", "폴더 전수 검색", "stateless", "hwp_batch_search", "NDJSON 스트림."),
    ("folder_extract", "폴더 날짜·금액", "stateless", "hwp_batch_extract_data", "문서마다 limit."),
    ("folder_fill", "서식1 + 데이터N", "stateless", "hwp_batch_fill", "메일머지."),
    ("scan_dir", "폴더에서 문서 발견", "stateless", "hwp_scan", "세션 전 인벤토리."),
    ("threat", "미신뢰 문서 선검사", "stateless", "hwp_threat_scan", "열기 전 컨테이너 위협."),
    ("inspect_inject", "프롬프트 주입 신호", "stateless", "hwp_inspect_injection", "세션 편집 전 선별."),
    ("redact", "배포 전 마스킹", "stateless", "hwp_redact", "세션에 redact 가 없다."),
    ("plan", "여러 편집을 원자적으로", "stateless", "hwp_run_plan", "선검증 후 한 번 저장."),
    ("repeat_search", "387쪽 문서 검색 3회", "session", "hwp_open", "재파싱 비용이 이긴다."),
    ("repeat_text", "같은 문서 여러 쪽 읽기", "session", "hwp_open", "page 창을 잇는다."),
    ("edit_loop", "채우고 고치고 눈검증", "session", "hwp_open", "changedPages 쪽만 렌더."),
    ("large_law", "대형 규정 조문 인용", "session", "hwp_open", "structure+text 반복."),
    ("large_amounts", "대형 문서 금액 반복 좁히기", "session", "hwp_open", "extract_data+search."),
    ("ws_corpus", "워크스페이스 코퍼스", "session", "hwp_ws_list", "--workspace 전제."),
    ("ws_mutate_check", "변이 자기검증", "session", "hwp_ws_journal", "전/후 digest."),
    ("multi_query_edit", "검색→치환→저장", "session", "hwp_open", "한 핸들 누적."),
    ("after_fill_pages", "채움 후 쪽수 추적", "session", "hwp_doc_info", "재조판 확인."),
    ("cell_then_render", "칸 수정 후 눈검증", "session", "hwp_doc_set_cell", "changedPages."),
    ("abandon", "미리보기만 하고 폐기", "session", "hwp_open", "save 없이 close."),
]


def build_trace(spec, allowed: set[str]) -> dict:
    tid, title, steps, kind = spec
    out_steps = []
    for i, (name, args, layer, fields) in enumerate(steps, 1):
        if name not in allowed and name != "hwp_doc_foo":
            raise SystemExit(f"발명된 도구: {name}")
        out_steps.append(
            {
                "seq": i,
                "tool": name,
                "arguments": args,
                "expect_layer": layer,
                "read_fields": fields,
                "notes": (
                    "발명된 이름 — didYouMean 을 따른다"
                    if name == "hwp_doc_foo"
                    else ""
                ),
            }
        )
    return {
        "id": tid,
        "title": title,
        "kind": kind,
        "lifecycle": "hwp_open → doc_* → hwp_close" if kind == "session" else kind,
        "absolute_paths": True,
        "single_write_point": "hwp_doc_save" if kind == "session" else None,
        "steps": out_steps,
    }


def build_error(spec, allowed: set[str]) -> dict:
    for name in spec["tools"]:
        if name not in allowed:
            raise SystemExit(f"발명된 도구: {name}")
    return {
        **spec,
        "policy": {
            "jsonrpc": "프로토콜을 고친다. 같은 바이트를 재전송하지 않는다.",
            "isError": "종료 코드 2/필수 인자 = 재시도 금지. 닫힌 핸들만 nextCall 로 재발급.",
            "envelope": "isError:false. 필드를 게이트로 읽는다.",
        }[spec["layer"]],
    }


def build_decision(spec, allowed: set[str]) -> dict:
    did, task, choice, first, why = spec
    if first not in allowed:
        raise SystemExit(f"발명된 도구: {first}")
    return {
        "id": did,
        "task": task,
        "choice": choice,
        "first_tool": first,
        "why": why,
        "forbidden": (
            ["hwp_open"]
            if choice == "stateless"
            else ["invented_tools", "relative_paths"]
        ),
    }


def write_markdown_files(session: list[str], stateless: list[str], read: list[str]) -> None:
    pairing_rows = []
    for s, t in PAIRING_CANDIDATES.items():
        if s in session and t in stateless:
            pairing_rows.append(f"| `{s}` | `{t}` | 봉투 동형 |")

    session_rows = []
    for name in session:
        meta = SESSION_META[name]
        req = ",".join(meta["required"]) or "(없음)"
        twin = PAIRING_CANDIDATES.get(name, "—")
        if twin != "—" and twin not in stateless:
            twin = "—"
        session_rows.append(
            f"| `{name}` | {meta['family']} | `{req}` | `{twin}` | {meta['when']} |"
        )

    (REF / "session_lifecycle.md").write_text(
        f"""# 세션 수명 — hwp_open → hwp_doc_* → hwp_close

권위: `src/mcp_serve.rs` 세션 핸들, `src/agent_profiles.rs` ALL_SESSION_TOOLS.
도구 목록의 정본은 `mcp-serve` 의 `tools/list` 다. `capabilities --mcp` 는 무상태 선언만 낸다.

## 한 줄 계약

1. `hwp_open`(또는 `hwp_ws_open`)이 `docId` 를 발급한다. 파싱은 이 순간 한 번이다.
2. `hwp_doc_*` 는 그 핸들의 IR 을 재파싱 없이 읽거나 누적 편집한다.
3. 디스크에 쓰는 세션 도구는 `hwp_doc_save` 와 `hwp_doc_render_page`(새 SVG) 뿐이다.
4. `hwp_close` 가 메모리를 해제한다. 저장하지 않은 편집은 사라진다.
5. 핸들 수명 = 서버 프로세스 수명. 영속이 아니다.

## 상태 기계

```
(없음)
  --hwp_open/hwp_ws_open--> OPEN(docId)
                               | 조회: hwp_doc_info/text/fields/tables/search/structure/extract_data/tree/render_page
                               | 변이: hwp_doc_fill_fields/replace_text/set_cell   (IR 만, 디스크 아님)
                               | 기록: hwp_doc_save   (핸들은 그대로 OPEN)
                               v
                            CLOSED  --이미 닫힘--> isError + nextCall(hwp_open)
```

서버가 내려가면 모든 핸들이 무효다. 호스트가 MCP 서버를 재시작하면 `doc-1` 을 재사용하지 말고
다시 `hwp_open` 한다.

## 정상 흐름 (실측 어휘)

```jsonc
→ tools/call hwp_open        {{ "path": "C:/절대/경로/편람.hwp" }}
← {{ "docId": "doc-1", "pageCount": 393, "source": "…", "schemaVersion": "1.0" }}

→ tools/call hwp_doc_search  {{ "docId": "doc-1", "query": "위임전결" }}
← matches[].page / section / paragraph   // hwp_search 와 동형

→ tools/call hwp_doc_fill_fields {{ "docId": "doc-1", "data": {{ "회사명": "페타플로" }} }}
← filledCount / notFound / ambiguous / changedPages

→ tools/call hwp_doc_render_page {{ "docId": "doc-1", "page": 0, "output": "C:/abs/out/p0.svg" }}
→ tools/call hwp_doc_save    {{ "docId": "doc-1", "output": "C:/abs/out/저장본.hwp", "verify": true }}
→ tools/call hwp_close       {{ "docId": "doc-1" }}
← {{ "closed": true, "docId": "doc-1" }}
```

## 세션 도구 {len(session)}종 (소스 상수, 개수는 계약이 아님)

{chr(10).join(session_rows)}

조회 축(`SESSION_READ_TOOLS`, {len(read)}종): {", ".join(f"`{n}`" for n in read)}

변이 축: `hwp_doc_fill_fields` · `hwp_doc_replace_text` · `hwp_doc_set_cell`
기록 축: `hwp_doc_save` (`destructiveHint=true` — output 이 원본 경로일 수 있다)

## 하지 않는 것

- `handle` 이라는 인자 이름은 없다. 항상 `docId`.
- 상대 경로. 서버 cwd 와 호스트 cwd 가 다르다.
- 저장 없이 "파일이 바뀌었다"고 보고하기.
- 닫힌 `docId` 를 같은 값으로 재시도하기 — `nextCall` 이 새 `hwp_open` 을 가리킨다.
- 세션에 없는 편집(예: `hwp_doc_redact`, `hwp_doc_insert_row`)을 만들어 부르기.
  그 작업은 무상태 도구이거나 CLI 다.

## 워크스페이스(#4357) 분기

`rhwp mcp-serve --workspace <dir>` 로 기동했을 때만 `hwp_ws_list` / `hwp_ws_open` /
`hwp_doc_tree` / `hwp_ws_journal` 이 의미가 있다. 아니면 경로로 `hwp_open` 한다.
""",
        encoding="utf-8",
    )

    (REF / "stateless_when.md").write_text(
        f"""# 언제 무상태 도구를 쓰는가

무상태 도구는 `rhwp capabilities --mcp` 가 내는 선언이다. 실행은 `mcp-serve` 가
같은 선언의 `cli.args` 를 해석해 rhwp 자신을 서브프로세스로 돌린다. 따라서
**CLI `--json` 계약이 곧 도구 계약**이다.

현재 소스에서 추출한 무상태 도구는 {len(stateless)}종이다. 이 숫자를 외우지 마라.
손에 든 바이너리의 `capabilities --mcp` 가 이긴다.

## 선택 규칙

| 질문 | 예 | 선택 |
|---|---|---|
| 호출 하나면 끝나는가? | 쪽수, 검색 1회, PDF 1회 | **무상태** |
| 같은 파일을 두 번 이상 파싱하게 되는가? | 검색 3회 + 본문 + 채움 | **세션** |
| 대상이 파일 목록인가? | 폴더 스윕, 메일머지 | **무상태 배치** (`hwp_batch*`) |
| 세션에 짝이 없는가? | PDF, redact, run, ir-diff, scan | **무상태만** |
| 워크스페이스 코퍼스인가? | `--workspace` 인벤토리 | **세션** (`hwp_ws_*`) |

## 무상태가 항상 이기는 작업

- 변환·발행: `hwp_export_pdf` · `hwp_export_markdown` · `hwp_convert_hwpx` · `hwp_convert_hwp5`
- 검증 사다리: `hwp_ir_diff` · `hwp_verify` · `hwp_replay` · `hwp_audit` · `hwp_lineage`
- 보안 스윕: `hwp_threat_scan` · `hwp_inspect_*` · `hwp_redact` · `hwp_sanitize`
- 대량: `hwp_scan` · `hwp_batch` · `hwp_batch_search` · `hwp_batch_extract_data` · `hwp_batch_fill`
- 표/차트 왕복: `hwp_table_to_csv` · `hwp_csv_to_table` · `hwp_chart_to_csv` · `hwp_csv_to_chart`
- 원자 다중 편집: `hwp_run_plan` (세션 누적과 다른 축 — 선검증 후 한 파일)

## 세션이 이기는 작업

- 수백 쪽 문서를 검색·발췌·재검색
- 채움/치환/칸 수정을 누적한 뒤 `changedPages` 만 렌더하고 한 번 저장
- 조문 계층을 따라가며 같은 핸들에서 본문·금액을 반복 조회
- 미리보기만 하고 저장하지 않고 폐기

실측(지식 지도): 387쪽 문서에서 검색 3회+info 가 세션 310ms vs 무상태 810ms.

## 배치 함정

`hwp_batch` 계열은 `structuredContent` 가 `null` 이다. `content[0].text` 를 줄 단위
NDJSON 으로 파싱한다. `batch convert` 는 MCP 에 없다 —
`capabilities.batch.mcp.excluded` 가 이유를 문자열로 준다.

## 금지

- 무상태 도구 이름을 손으로 베껴 호스트에 고정하지 않는다.
- `hwp_doc_*` 를 무상태처럼 `path` 로 부르지 않는다. 세션 도구는 `docId` 다.
- 세션에 없는 동사를 만들어 붙이지 않는다.
""",
        encoding="utf-8",
    )

    (REF / "capabilities_ssot.md").write_text(
        """# 단일 출처 — `capabilities --mcp`

도구 정의는 `src/main.rs` 의 `mcp_tool_definitions()` **한 곳**에서 나온다.

| 표면 | 무엇이 나오나 | 무엇이 안 나오나 |
|---|---|---|
| `rhwp capabilities --mcp` | 무상태 도구 선언(name/description/inputSchema/cli/annotations) | 세션 도구 |
| `mcp-serve` `tools/list` | 위 선언 + 세션 도구 | 손으로 베낀 옛 목록 |
| `rhwp://capabilities/mcp` | 매니페스트 리소스(= `--mcp`) | 세션 도구 |

계약 테스트 `tests/mcp_server_contract.rs::tools_list_matches_capabilities_manifest`
가 두 표면의 무상태 부분을 같게 유지한다. `--json` 명령이 늘었는데 MCP 에서 빠지면
`capabilities_mcp_covers_every_json_command` 가 잡는다.

## 에이전트가 할 일

1. 추측하지 말고 `rhwp capabilities --mcp` 또는 리소스 `rhwp://capabilities/mcp` 를 1회 캐시한다.
2. 세션이 필요하면 `tools/list` 를 한 번 더 본다. 세션 이름은 여기에만 있다.
3. 이름이 없으면 `didYouMean` → `tools/list` → 무상태 CLI. **새 이름을 만들지 않는다.**
4. 문서(본 스킬 포함)의 개수가 바이너리와 다르면 바이너리가 이긴다.

## 함수콜 클라이언트 (경로 ①)

MCP 호스트가 아니면 선언을 직접 소비한다.

1. `cli.args` 의 `{키}` 를 `inputSchema` 같은 이름 값으로 치환한다.
2. 객체·숫자는 JSON 문자열로 넣는다 (`hwp_fill_fields` 의 `{data}`).
3. `invocation.stdinTools` (`hwp_batch` · `hwp_batch_search` · `hwp_batch_extract_data`)는
   `paths` 를 stdin 한 줄씩 흘린다.
4. `cli.passwordStdin` 이 있으면 `password` 를 `--password-stdin` 첫 줄로만 넘긴다.

## 프로필

`--profile <이름>` 은 추천이 아니라 **서버가 제공하는 집합의 경계**다.
목록에 없는 세션 도구는 `tools/call` 로도 우회할 수 없다.
없는 프로필 이름은 실행 전에 막힌다 (`오류: 알 수 없는 프로필`).

실측 프로필: `경영보고` · `행정서식` · `데이터분석` · `콘텐츠제작` ·
`아카이브검색` · `품질검증` · `개발통합`.

`개발통합` 만 필터가 없다. 작은 모델은 직무에 맞는 프로필로 물린다.

## 확인 명령

```bash
rhwp capabilities --mcp | jq '.tools[] | {name, description}'
rhwp capabilities --mcp --profile 행정서식
# 세션 포함 목록
printf '%s\\n' \\
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"x","version":"0"}}}' \\
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \\
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | rhwp mcp-serve
```
""",
        encoding="utf-8",
    )

    (REF / "error_recovery.md").write_text(
        """# 오류 복구 — 세 층을 혼동하지 않는다

권위: `mydocs/manual/mcp_integration_guide.md`, 실패 사전 §14.

| 층 | 신호 | 재시도 | 다음 수 |
|---|---|---|---|
| JSON-RPC | `error{code,message}` | 금지 | 메시지/메서드/params 를 고친다 |
| 도구 실패 | `isError:true` | 닫힌 핸들만 | 필수 인자·경로·프로필을 고친다. exit 2 는 재시도 금지 |
| 봉투 판정 | `isError:false` + 필드 | 조건부 | `identical`/`notFound`/`invalid`/`nextOffset` 을 읽는다 |

## 층 1 — JSON-RPC

| code | 뜻 | 복구 |
|---:|---|---|
| -32700 | 줄이 JSON 이 아님 | 한 줄 한 객체. 로그를 stdout 에 섞지 않는다 |
| -32600 | 요청 구조 오류 | jsonrpc 2.0 필드 |
| -32601 | 메서드 없음 | 지원 목록만 |
| -32602 | params 구조 오류 | `params.name` 필수 |
| -32002 | 리소스 없음 | `resources/list` 로 URI 재확인 |

클라이언트가 `2024-11-05` 를 보내도 서버는 `2025-06-18` 로 응답할 수 있다.
버전 불일치를 하드 실패로 보면 핸드셰이크에서 막힌다.

## 층 2 — isError

실측 바늘:

- `알 수 없는 도구` + `didYouMean[]` + `nextCall`
- `path 가 필요합니다` / `docId 가 필요합니다` / `query 가 필요합니다`
- `열려 있지 않은 핸들: doc-1 (hwp_open 먼저)` + `nextCall{name:"hwp_open"}`
- `종료 코드 1:` 파일·권한·파싱 — 입력을 고친다
- `종료 코드 2:` 사용법 — 인자를 고친다. 같은 호출 재시도 금지
- `현재 프로필에서는 세션 도구를 제공하지 않습니다`

닫힌 핸들만 자동 복구 루프다: `hwp_open` → 새 `docId` → 원래 조회.
`hwp_close` 의 만료 문구에는 `(hwp_open 먼저)` 가 없을 수 있다.
매칭 키는 `열려 있지 않은 핸들` 이다.

## 층 3 — 봉투 (오류가 아닌 데이터)

| 필드 | 도구 | 게이트 |
|---|---|---|
| `identical:false` | `hwp_ir_diff` | 차이 발견. 사람 큐 |
| `notFound` / `ambiguous` | `hwp_fill_fields` / `hwp_doc_fill_fields` | 이름·순번을 고친다 |
| `replacedCount:0` | replace 계열 | 찾기 실패 보고 |
| `overflow` | set_cell 계열 | 칸은 쓰임. 넘침 보고 |
| `invalid != []` | `hwp_run_plan` | MCP 는 isError:false |
| `truncated` + `nextOffset` | text/search/extract | 이어보기. nextOffset 없으면 끝 |
| `verify.identical:false` | save/convert | 저장은 됐고 IR 차이 |

`content[0].text` 는 문자열화된 JSON 이다. 단건 도구는 `structuredContent` 를 써라.
배치만 `structuredContent=null`.

## 복구 의사코드

```
if response.error:                 # JSON-RPC
    fix protocol; do not retry
elif result.isError:
    body = parse(content[0].text)
    if "열려 있지 않은 핸들" in body:
        call nextCall              # hwp_open
        retry original with new docId
    elif body.nextCall:
        inspect; do not invent
    else:
        fix args; do not retry same bytes
else:
    env = structuredContent or parse(text)
    gate on env fields
```

## 상대 경로

`path` 는 서버 cwd 기준이다. MCP 로는 항상 절대 경로.
""",
        encoding="utf-8",
    )

    (REF / "pairing.md").write_text(
        f"""# 세션 ↔ 무상태 짝

세션 조회·편집의 봉투 어휘는 무상태 대응 도구와 동형이다.
짝이 없는 세션 도구는 서버 전용이다. 짝이 없는 무상태 도구를 세션에 만들지 않는다.

| 세션 | 무상태 | 비고 |
|---|---|---|
{chr(10).join(pairing_rows)}
| `hwp_open` | — | 세션 진입 |
| `hwp_close` | — | 세션 종료 |
| `hwp_doc_save` | — | 세션 기록 지점. 무상태 편집은 각 호출이 파일을 쓴다 |
| `hwp_doc_tree` | — | 안정 노드 ID (#4357) |
| `hwp_ws_list` | — | `--workspace` |
| `hwp_ws_open` | `hwp_open` (경로 축) | id 축 진입 |
| `hwp_ws_journal` | — | 변이 digest |

## 변환 규칙

- 무상태 `path` → 세션 `docId` (먼저 open).
- 무상태 `-o/--output` 편집 → 세션은 IR 누적 후 `hwp_doc_save.output`.
- 무상태 `hwp_export_svg`(전 쪽) → 세션 `hwp_doc_render_page`(한 쪽).
- 무상태 `hwp_run_plan` 을 세션 누적으로 바꾸지 않는다. 선검증 원자 실행은 별 축이다.

## 세션에 없는 동사

표 편집 확장(`hwp_insert_row` 등), 그림·머리말·각주, `hwp_redact`, `hwp_export_pdf` 는
무상태만 있다. `hwp_doc_insert_row` 같은 이름을 만들지 마라.
""",
        encoding="utf-8",
    )

    (REF / "host_attach.md").write_text(
        """# 호스트에 붙이기 — 세션을 살리는 설정

정본 킷: `mydocs/manual/mcp_attach_kit.md`. 여기서는 세션 관점만 옮긴다.

## 최소 등록

```json
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

- `rhwp` 가 PATH 에 없으면 `command` 를 절대 경로로 (`C:/…/target/release/rhwp.exe`).
- 전송은 stdio JSON-RPC 뿐이다. 포트·인증·URL 설정은 없다.
- 서버는 stdin EOF 에서 종료한다. 호스트가 프로세스를 죽이면 핸들은 전부 사라진다.

## 세션을 살리려면

1. 호스트가 대화마다 새 `mcp-serve` 를 띄우면 세션 이득이 없다. 같은 서버 프로세스가
   여러 `tools/call` 을 받아야 한다.
2. 역할이 정해져 있으면 `"args": ["mcp-serve", "--profile", "행정서식"]`.
3. 코퍼스 인벤토리가 필요하면 `"args": ["mcp-serve", "--workspace", "C:/abs/corpus"]`.
4. 상대 경로를 피하려면 호스트 cwd 와 무관하게 **절대 경로만** 넘긴다.

## 호스트 모양

| 형 | 호스트 | 키 |
|---|---|---|
| A | Claude Code · Desktop · Cursor · Windsurf · Gemini CLI | `mcpServers` |
| B | VS Code / Copilot | `servers` + `type: stdio` |
| YAML | Goose · Continue | `cmd`/`command` + `args` |

저장소 루트 `.mcp.json` 은 Claude Code 용 A형이다.

## 핸드셰이크 후 확인

1. `initialize` → `protocolVersion` / `serverInfo.name=rhwp`
2. `notifications/initialized`
3. `tools/list` — 무상태 + 세션. 프로필이면 축소된 목록.
4. `resources/list` — `rhwp://capabilities/mcp` 가 있는지.

## 하지 않는 것

- 도구 목록을 호스트 설정에 하드코딩하기.
- 세션 도구를 호스트가 임의로 추가하기.
- gym 트레이스를 실사용 세션으로 오인하기. 이 스킬은 실 에이전트 부착용이다.
""",
        encoding="utf-8",
    )

    (REF / "decision_tree.md").write_text(
        """# 판단 트리 — 세션인가 무상태인가

```
문서 작업인가?
├─ 아니오 → 이 스킬 범위 밖 (기여는 rhwp-contributor)
└─ 예
   ├─ 폴더/목록인가? → 무상태 배치 (hwp_scan / hwp_batch*)
   ├─ 세션에 짝이 없는 동사인가? (pdf, redact, run, ir-diff, convert)
   │     → 무상태만. 이름을 만들지 말 것
   ├─ 호출이 정확히 1회인가? → 무상태
   ├─ 같은 문서를 2회 이상 파싱하게 되는가?
   │     ├─ 예, 파일이 크거나 편집 루프다 → hwp_open 세션
   │     └─ 예외적으로 파일 두 개를 비교 → hwp_ir_diff (무상태)
   └─ --workspace 코퍼스인가? → hwp_ws_list → hwp_ws_open
```

## 편집 루프 (세션)

```
hwp_open
  → (조회) fields/tables/search/structure
  → (누적) fill_fields / replace_text / set_cell
  → (눈검증) render_page(changedPages)
  → (기록) save verify=true
  → (더 있으면) 조회부터 반복 — 핸들은 그대로
  → hwp_close
```

## 단건 채움 (무상태)

```
hwp_fields → hwp_fill_fields → hwp_export_svg 또는 hwp_verify
```

여러 편집을 한 파일에 원자적으로 묶으면 `hwp_run_plan` (세션이 아님).

## 실패 시

막히면 리소스 `rhwp://docs/agent_troubleshooting_guide.md` §14.
도구가 생각나지 않으면 `rhwp capabilities --mcp` 와 `tools/list`.
""",
        encoding="utf-8",
    )

    (REF / "session_tools.md").write_text(
        "# 세션 도구 카드\n\n"
        "각 카드의 기계 가독 정본은 `fixtures/tools/<name>.json` 이다.\n"
        "아래는 에이전트가 훑는 요약이다. 스키마는 `tools/list` 가 이긴다.\n\n"
        + "\n".join(
            _tool_md(name) for name in session if name in SESSION_META
        ),
        encoding="utf-8",
    )

    (REF / "README.md").write_text(
        """# rhwp-mcp-session references

실 에이전트가 `rhwp mcp-serve` 를 붙이고 세션/무상태를 고르기 위한 레퍼런스다.
gym 트레이스·벤치와 무관하다.

| 문서 | 내용 |
|---|---|
| [session_lifecycle.md](session_lifecycle.md) | hwp_open → doc_* → close |
| [session_tools.md](session_tools.md) | 세션 도구 카드 |
| [stateless_when.md](stateless_when.md) | 무상태를 고르는 때 |
| [capabilities_ssot.md](capabilities_ssot.md) | capabilities --mcp 단일 출처 |
| [error_recovery.md](error_recovery.md) | 판정 3층과 복구 |
| [pairing.md](pairing.md) | 세션↔무상태 짝 |
| [host_attach.md](host_attach.md) | 호스트 부착 |
| [decision_tree.md](decision_tree.md) | 판단 트리 |
| [fixtures/](fixtures/) | 기계 검증용 JSON |

픽스처는 `scripts/tests/test_agent_mcp_session.py` 가 소스 allowlist 와 대조한다.
""",
        encoding="utf-8",
    )


def _tool_md(name: str) -> str:
    m = SESSION_META[name]
    twin = PAIRING_CANDIDATES.get(name, "—")
    return (
        f"## `{name}`\n\n"
        f"- 가족: {m['family']}\n"
        f"- 필수: {', '.join(m['required']) or '(없음)'}\n"
        f"- 선택: {', '.join(m['optional']) or '(없음)'}\n"
        f"- 봉투: {', '.join(m['envelope'])}\n"
        f"- 무상태 짝: `{twin}`\n"
        f"- 언제: {m['when']}\n"
        f"- 언제 아닌가: {m['when_not']}\n"
        f"- 복구: {m['next_after_error']}\n"
        f"- IR 기록: {m['writes_ir']} / 디스크: {m['writes_disk']} / "
        f"idempotent: {m['idempotent']} / destructive: {m['destructive']}\n"
    )


def main() -> None:
    session = extract_session_tools()
    read = extract_read_tools()
    stateless = extract_stateless_tools()
    allowed = set(session) | set(stateless)
    missing_meta = [n for n in session if n not in SESSION_META]
    if missing_meta:
        raise SystemExit(f"SESSION_META 누락(소스는 있는데 카드 없음): {missing_meta}")

    FIXT.mkdir(parents=True, exist_ok=True)
    dump(FIXT / "allowlist.json", build_allowlist(session, stateless, read))
    dump(
        FIXT / "profiles.json",
        {
            "names": [p["name"] for p in extract_profiles()],
            "note": "프로필 이름은 src/agent_profiles.rs PROFILES 가 정본.",
        },
    )

    tools_dir = FIXT / "tools"
    for name in session:
        dump(tools_dir / f"{name}.json", build_tool_card(name, session, stateless))

    traces_dir = FIXT / "traces"
    for spec in TRACE_SPECS:
        dump(traces_dir / f"{spec[0]}.json", build_trace(spec, allowed))

    err_dir = FIXT / "errors"
    for spec in ERROR_SPECS:
        dump(err_dir / f"{spec['id']}.json", build_error(spec, allowed))

    dec_dir = FIXT / "decisions"
    for spec in DECISION_SPECS:
        dump(dec_dir / f"{spec[0]}.json", build_decision(spec, allowed))

    # 에이전트가 훑는 시나리오 카탈로그 — 작업×선택×복구를 펼친다.
    catalog = []
    samples = [
        "samples/field-01.hwp",
        "samples/table-001.hwp",
        "samples/복학원서.hwp",
        "samples/추진일정.hwp",
        "samples/form-01.hwp",
    ]
    for i, (sample, spec) in enumerate(
        ((s, d) for s in samples for d in DECISION_SPECS), 1
    ):
        did, task, choice, first, why = spec
        if first not in allowed:
            continue
        catalog.append(
            {
                "id": f"cat-{i:03d}",
                "sample": sample,
                "task": task,
                "choice": choice,
                "first_tool": first,
                "why": why,
                "path_rule": "MCP 로는 저장소 루트 기준이 아니라 절대 경로",
                "if_session": [
                    "hwp_open",
                    first if first in session else "hwp_doc_info",
                    "hwp_close",
                ],
                "if_stateless": [first],
                "error_if_relative": "서버 cwd 기준 오탐",
                "ssot": "capabilities --mcp + tools/list",
            }
        )
    dump(FIXT / "scenario_catalog.json", {"issue": 5293, "items": catalog})

    write_markdown_files(session, stateless, read)
    print(
        f"session={len(session)} stateless={len(stateless)} "
        f"traces={len(TRACE_SPECS)} errors={len(ERROR_SPECS)} "
        f"decisions={len(DECISION_SPECS)} catalog={len(catalog)}"
    )


if __name__ == "__main__":
    main()
