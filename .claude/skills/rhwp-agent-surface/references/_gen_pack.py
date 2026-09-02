#!/usr/bin/env python3
"""Extract rhwp agent-surface allowlists from source and emit fixtures.

SSOT: src/main.rs mcp_tool_definitions() / capabilities_command_entries(),
src/agent_profiles.rs ALL_SESSION_TOOLS / PROFILES.

Does not invent CLI commands or MCP tool names. Counts are not contracts.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
SKILL = REPO / ".claude" / "skills" / "rhwp-agent-surface"
FIXT = SKILL / "fixtures"
MAIN = REPO / "src" / "main.rs"
PROFILES_RS = REPO / "src" / "agent_profiles.rs"

HWP_NAME = re.compile(r"hwp_[a-z0-9_]+")


def _fn_body(src: str, sig: str) -> str:
    start = src.find(sig)
    if start < 0:
        raise SystemExit(f"missing {sig}")
    # Const slices use `= &[ ... ];`. Skip the type `&[&str]`.
    bracket = src.find("= &[", start, start + 240)
    if bracket >= 0:
        depth = 0
        for i, ch in enumerate(src[bracket:], bracket):
            if ch == "[":
                depth += 1
            elif ch == "]":
                depth -= 1
                if depth == 0:
                    return src[start : i + 1]
        raise SystemExit(f"unclosed slice {sig}")
    depth = 0
    begun = False
    for i, ch in enumerate(src[start:], start):
        if ch == "{":
            depth += 1
            begun = True
        elif ch == "}":
            depth -= 1
            if begun and depth == 0:
                return src[start : i + 1]
    raise SystemExit(f"unclosed {sig}")


def extract_session_tools() -> list[str]:
    src = PROFILES_RS.read_text(encoding="utf-8")
    body = _fn_body(src, "pub const ALL_SESSION_TOOLS")
    names = re.findall(r'"(hwp_[a-z0-9_]+)"', body)
    if "hwp_open" not in names or "hwp_close" not in names:
        raise SystemExit(f"session tools look wrong: {names}")
    return names


def extract_session_read_tools() -> list[str]:
    src = PROFILES_RS.read_text(encoding="utf-8")
    body = _fn_body(src, "pub const SESSION_READ_TOOLS")
    return re.findall(r'"(hwp_[a-z0-9_]+)"', body)


def extract_stateless_tools() -> list[dict]:
    src = MAIN.read_text(encoding="utf-8")
    body = _fn_body(src, "fn mcp_tool_definitions()")
    # First string after tool( / tool_with_optional_args( is the name,
    # second is the description.
    pat = re.compile(
        r'tool(?:_with_optional_args)?\(\s*"(hwp_[a-z0-9_]+)"\s*,\s*"([^"]*)"',
        re.S,
    )
    out = []
    seen = set()
    for name, desc in pat.findall(body):
        if name in seen:
            continue
        seen.add(name)
        # cli.command is the 4th string-ish after name/desc/schema — harvest
        # nearby `,"command",` template is unreliable. Pull from the next
        # quoted token that is not hwp_ / not a description sentence.
        out.append({"name": name, "description": desc})
    if len(out) < 40:
        raise SystemExit(f"too few stateless tools: {len(out)}")
    return out


def extract_cli_commands() -> list[dict]:
    src = MAIN.read_text(encoding="utf-8")
    body = _fn_body(src, "fn capabilities_command_entries()")
    items: list[dict] = []
    # cmd_json("name", "family", "summary", json_bool, ...
    for m in re.finditer(
        r'cmd_json\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(true|false)',
        body,
    ):
        items.append(
            {
                "name": m.group(1),
                "family": m.group(2),
                "summary": m.group(3),
                "json": m.group(4) == "true",
                "kind": "json",
            }
        )
    for m in re.finditer(
        r'cmd\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)',
        body,
    ):
        items.append(
            {
                "name": m.group(1),
                "family": m.group(2),
                "summary": m.group(3),
                "json": False,
                "kind": "plain",
            }
        )
    # preserve first-seen order, drop dups
    seen = set()
    uniq = []
    for it in items:
        if it["name"] in seen:
            continue
        seen.add(it["name"])
        uniq.append(it)
    if len(uniq) < 40:
        raise SystemExit(f"too few CLI commands: {len(uniq)}")
    return uniq


def extract_profiles() -> list[dict]:
    src = PROFILES_RS.read_text(encoding="utf-8")
    profiles = []
    chunks = re.split(r"AgentProfile\s*\{", src)[1:]
    for chunk in chunks:
        name_m = re.search(r'name:\s*"([^"]+)"', chunk)
        sum_m = re.search(r'summary:\s*"([^"]+)"', chunk)
        if not name_m:
            continue
        tools_m = re.search(r"tools:\s*&\[(.*?)\]", chunk, re.S)
        sess_m = re.search(r"session_tools:\s*([^,\n]+)", chunk)
        tools = re.findall(r'"(hwp_[a-z0-9_]+)"', tools_m.group(1) if tools_m else "")
        sess_raw = (sess_m.group(1) if sess_m else "None").strip()
        if sess_raw.startswith("None"):
            session = None
        elif "SESSION_READ_TOOLS" in sess_raw:
            session = "SESSION_READ_TOOLS"
        elif sess_raw.startswith("Some"):
            session = "ALL" if "&[]" in sess_raw or "ALL_SESSION" in sess_raw else "SOME"
        else:
            session = sess_raw
        profiles.append(
            {
                "name": name_m.group(1),
                "summary": sum_m.group(1) if sum_m else "",
                "stateless_tools": tools,
                "session": session,
            }
        )
    if len(profiles) < 5:
        raise SystemExit(f"too few profiles: {profiles}")
    return profiles


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_layers() -> None:
    dump(
        FIXT / "layers.json",
        {
            "schemaVersion": "1.0",
            "countsAreNotContracts": True,
            "layers": [
                {
                    "id": "cli-json",
                    "title": "CLI JSON",
                    "what": "stdout 순수 JSON 봉투 + 종료 코드 0/1/2/3/4",
                    "ssot": "각 명령 구현 + 봉투 helper (*_json_value)",
                    "entry": ["rhwp capabilities", "rhwp <cmd> --json"],
                    "not": "rhwp --help 파싱은 계약이 아니다",
                    "guard": "tests/cli_json_contract.rs",
                },
                {
                    "id": "mcp-stateless",
                    "title": "MCP 무상태",
                    "what": "capabilities --mcp 선언과 mcp-serve tools/list 가 공유하는 도구",
                    "ssot": "src/main.rs mcp_tool_definitions()",
                    "entry": ["rhwp capabilities --mcp", "rhwp://capabilities/mcp"],
                    "not": "세션 도구는 이 층에 없다",
                    "guard": "capabilities_mcp_covers_every_json_command",
                },
                {
                    "id": "mcp-session",
                    "title": "MCP 세션",
                    "what": "열린 핸들(docId) 위의 재파싱 없는 연산",
                    "ssot": "src/mcp_serve.rs served_tools() + src/agent_profiles.rs ALL_SESSION_TOOLS",
                    "entry": ["mcp-serve tools/list", "hwp_open"],
                    "not": "capabilities --mcp 선언에 세션 이름이 없다",
                    "guard": "tools_list_matches_capabilities_manifest",
                },
            ],
        },
    )


def write_rules() -> None:
    dump(
        FIXT / "rules.json",
        {
            "schemaVersion": "1.0",
            "rules": [
                {
                    "id": 1,
                    "name": "declare-execute-docs-fork",
                    "text": "선언·실행·문서는 한 곳에서 갈라진다",
                    "fork": "mcp_tool_definitions()",
                    "do_not": "도구 목록을 호스트 설정·스킬·문서에 복제하지 않는다",
                    "guards": [
                        "capabilities_mcp_covers_every_json_command",
                        "tools_list_matches_capabilities_manifest",
                        "capabilities_mcp_tool_definitions_contract",
                    ],
                },
                {
                    "id": 2,
                    "name": "never-invent-edit-logic",
                    "text": "새 편집·조회 로직을 만들지 않는다",
                    "reuse": [
                        "set_field_value_by_name_at",
                        "replace_all_native",
                        "grep",
                        "collect_field_records",
                        "extract_tables",
                        "edit_serialize",
                        "*_json_value 봉투 helper",
                    ],
                    "do_not": "서버 전용 경로를 새로 만들면 CLI 와 계약이 갈라진다",
                },
                {
                    "id": 3,
                    "name": "judgment-is-data",
                    "text": "판정은 데이터다",
                    "data_fields": [
                        "identical",
                        "replacedCount",
                        "notFound",
                        "ambiguous",
                        "invalid",
                        "matchCount",
                        "regression",
                        "verifyPages",
                    ],
                    "is_error_only": ["없는 파일", "닫힌 핸들", "필수 인자 누락", "알 수 없는 도구"],
                    "cli_exit": {
                        "0": "실행 성공 — 봉투 판정 필드를 마저 읽는다",
                        "1": "런타임 실패",
                        "2": "호출 조립 버그 — 같은 인자 재시도 금지",
                        "3": "검증 단언 실패(판정) — isError 가 아니다",
                        "4": "쪽 수 불일치",
                    },
                },
            ],
        },
    )


def write_allowlist(session, read, stateless, commands, profiles) -> dict:
    pairing = [
        {"session": "hwp_doc_info", "stateless": "hwp_info", "cli": "info"},
        {"session": "hwp_doc_text", "stateless": "hwp_export_text", "cli": "export-text"},
        {"session": "hwp_doc_fields", "stateless": "hwp_fields", "cli": "fields"},
        {"session": "hwp_doc_tables", "stateless": "hwp_export_tables", "cli": "export-tables"},
        {"session": "hwp_doc_search", "stateless": "hwp_search", "cli": "search"},
        {"session": "hwp_doc_render_page", "stateless": "hwp_export_svg", "cli": "export-svg"},
        {"session": "hwp_doc_structure", "stateless": "hwp_export_structure", "cli": "export-structure"},
        {"session": "hwp_doc_extract_data", "stateless": "hwp_extract_data", "cli": "extract-data"},
        {"session": "hwp_doc_replace_text", "stateless": "hwp_replace_text", "cli": "edit replace-text"},
        {"session": "hwp_doc_set_cell", "stateless": "hwp_set_cell", "cli": "edit set-cell"},
        {"session": "hwp_doc_fill_fields", "stateless": "hwp_fill_fields", "cli": "edit fill-fields"},
    ]
    obj = {
        "schemaVersion": "1.0",
        "counts_are_not_contracts": True,
        "ssot": {
            "cli_json": "capabilities_command_entries() + *_json_value",
            "mcp_stateless": "mcp_tool_definitions()",
            "mcp_session": "ALL_SESSION_TOOLS + served_tools()",
            "how_to_read": [
                "rhwp capabilities",
                "rhwp capabilities --mcp",
                "rhwp capabilities --search <키워드>",
            ],
        },
        "session_tools": session,
        "session_read_tools": read,
        "stateless_tools": [t["name"] for t in stateless],
        "cli_commands": [c["name"] for c in commands],
        "json_commands": [c["name"] for c in commands if c["json"]],
        "profiles": [p["name"] for p in profiles],
        "pairing": pairing,
        "not_this_skill": [
            "rhwp-mcp-session host attach .mcp.json",
            "rhwp-cli request-to-command mapping",
            "rhwp-codex handbook navigation",
        ],
        "invent_forbidden": [
            "hwp_doc_redact",
            "hwp_doc_insert_row",
            "hwp_doc_run",
            "hwp_doc_convert",
        ],
    }
    dump(FIXT / "allowlist.json", obj)
    return obj


def write_command_cards(commands: list[dict], stateless_names: set[str]) -> None:
    rows = []
    for c in commands:
        slug = "hwp_" + c["name"].replace("-", "_")
        mcp = slug if slug in stateless_names else None
        if c["name"] == "edit":
            mcp = "edit-subcommands-via-search"
        rows.append(
            {
                "name": c["name"],
                "family": c["family"],
                "summary": c["summary"],
                "json": c["json"],
                "layer": "cli-json" if c["json"] else "cli-human",
                "ssot": "capabilities_command_entries()",
                "mcp_stateless": mcp,
                "how_to_discover": f"rhwp capabilities --search {c['name']}",
            }
        )
    dump(
        FIXT / "commands" / "catalog.json",
        {
            "ssot": "capabilities_command_entries()",
            "counts_are_not_contracts": True,
            "add_piece": [
                "red 계약 테스트 tests/*_contract.rs 신설",
                "코어 함수 재사용 (규칙 2)",
                "봉투 helper 재사용",
                "mcp_tool_definitions() 한 줄 (json 이면)",
                "cli_commands.md + 지식 지도 행",
                "드리프트 가드 green",
            ],
            "items": rows,
        },
    )


def write_tool_cards(stateless: list[dict], session: list[str]) -> None:
    dump(
        FIXT / "tools" / "catalog.json",
        {
            "layer": "mcp-stateless",
            "ssot": "mcp_tool_definitions()",
            "in_capabilities_mcp": True,
            "in_tools_list": True,
            "counts_are_not_contracts": True,
            "rule1": "선언과 실행이 이 한 함수에서 갈라진다",
            "rule2": "새 로직 금지 — 검증된 코어 + 봉투 helper",
            "rule3": "판정 필드는 봉투, isError 는 런타임만",
            "items": [
                {
                    "name": t["name"],
                    "description": t["description"],
                    "cli_guess": t["name"].removeprefix("hwp_").replace("_", "-"),
                }
                for t in stateless
            ],
        },
    )
    for name in session:
        dump(
            FIXT / "session" / f"{name}.json",
            {
                "name": name,
                "layer": "mcp-session",
                "in_capabilities_mcp": False,
                "in_tools_list": True,
                "ssot": "ALL_SESSION_TOOLS + served_tools()",
                "handle": "docId" if name.startswith("hwp_doc_") or name == "hwp_close" else None,
                "write_point": name == "hwp_doc_save",
                "rule1": "세션 이름은 선언 매니페스트에 복제하지 않는다",
                "closed_handle": (
                    "isError + nextCall hwp_open" if name.startswith("hwp_doc_") else None
                ),
            },
        )


def write_search_transcripts(commands: list[dict]) -> None:
    queries = [
        "info",
        "search",
        "redact",
        "표",
        "표 병합",
        "누름틀",
        "verify",
        "ir-diff",
        "batch",
        "fill",
        "csv",
        "svg",
        "pdf",
        "digest",
        "없음XYZ",
        "replace-text",
        "set-cell",
        "fill-fields",
        "export-text",
        "extract-data",
        "render-diff",
        "provenance",
        "replay",
        "lineage",
        "bookmark",
        "footnote",
        "header",
        "equation",
        "picture",
        "ungroup",
    ]
    dump(
        FIXT / "search" / "queries.json",
        {
            "and_semantics": True,
            "case_insensitive": True,
            "includes_subcommands": True,
            "cannot_combine": ["--mcp", "--profile"],
            "json_flag": "only with --search",
            "queries": queries,
            "empty_result_is_data": True,
            "empty_message": "'<query>' 에 매치하는 명령이 없습니다.",
        },
    )
    for q in queries:
        kws = q.lower().split()
        matched = []
        for c in commands:
            hay = f"{c['name']} {c['summary']}".lower()
            if all(k in hay for k in kws):
                matched.append(
                    {
                        "name": c["name"],
                        "family": c["family"],
                        "summary": c["summary"],
                        "json": c["json"],
                    }
                )
        dump(
            FIXT / "search" / f"{_slug(q)}.json",
            {
                "invocation": f"rhwp capabilities --search {q}",
                "invocation_json": f"rhwp capabilities --search {q} --json",
                "query": q,
                "keywords_and": kws,
                "match_count": len(matched),
                "empty": len(matched) == 0,
                "layer": "cli-json",
                "ssot": "capabilities_command_entries() name+summary+subcommands",
                "matches": matched[:40],
                "truncated": len(matched) > 40,
                "total_match_count": len(matched),
                "note": "개수는 계약이 아니다. 바이너리 출력이 이긴다.",
            },
        )


def _slug(s: str) -> str:
    out = []
    for ch in s:
        if ch.isalnum() or ch in "-_":
            out.append(ch)
        else:
            out.append("_")
    slug = "".join(out).strip("_")
    return slug or "q"


def write_exceptions() -> None:
    cases = [
        {
            "id": "missing_capabilities_key",
            "title": "봉투에 untrustedContent 키가 없다",
            "layer": "cli-json",
            "symptom": "표지 파서가 키 부재에서 죽거나 false 로 단정",
            "measured_missing": [
                "edit redact --json",
                "edit sanitize --json",
                "run --dry-run --json",
                "edit insert-image --json",
                "export-ir-schema --json",
                "export-capabilities-schema --json",
            ],
            "consumer": "키 부재 = 미표기. false 가 아니다. 보수적으로 문서 파생.",
            "implementer": "새 봉투는 모든 모드에서 untrustedContent/untrustedFields 명시",
            "map": "rhwp export-provenance-map --json",
            "not_is_error": True,
        },
        {
            "id": "drift_guard_fail",
            "title": "선언과 실행이 갈라졌다",
            "layer": "mcp-stateless",
            "guards": [
                {
                    "name": "capabilities_mcp_covers_every_json_command",
                    "file": "tests/cli_json_contract.rs",
                    "means": "--json 명령이 MCP 도구로 안 나온다",
                    "fix": "mcp_tool_definitions() 에 한 줄 추가. 목록을 다른 곳에 베끼지 말 것",
                },
                {
                    "name": "tools_list_matches_capabilities_manifest",
                    "file": "tests/mcp_server_contract.rs",
                    "means": "tools/list 와 --mcp 선언이 어긋남",
                    "fix": "같은 mcp_tool_definitions() 를 쓰는지 확인",
                },
                {
                    "name": "capabilities_mcp_tool_definitions_contract",
                    "file": "tests/cli_json_contract.rs",
                    "means": "name/description/inputSchema/cli.command 누락",
                    "fix": "tool() helper 로 추가 — 손수 JSON 복제 금지",
                },
                {
                    "name": "capabilities_covers_every_help_command",
                    "file": "tests/cli_json_contract.rs",
                    "means": "help 에 있는 명령이 capabilities 에 없다",
                    "fix": "capabilities_command_entries() 에 등재",
                },
            ],
            "do_not": "가드를 느슨하게 만들거나 제외 목록을 감으로 늘리지 않는다",
        },
        {
            "id": "closed_handle",
            "title": "닫힌/만료 docId 재사용",
            "layer": "mcp-session",
            "signal": "isError:true",
            "envelope": {
                "error": "열려 있지 않은 핸들: doc-1 (hwp_open 먼저)",
                "nextCall": {
                    "name": "hwp_open",
                    "arguments": {"path": "<열 문서 경로>"},
                    "why": "핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도",
                },
            },
            "retry": True,
            "same_args_forbidden": False,
            "fix": "hwp_open 으로 새 docId 를 받은 뒤 같은 조회를 반복",
        },
        {
            "id": "profile_blocked",
            "title": "프로필이 열지 않는 도구를 불렀다",
            "layer": "mcp-stateless",
            "signal": "실행 전 거부 또는 tools/list 에 없음",
            "unknown_profile": {
                "cli": "오류: 알 수 없는 프로필 '<name>'",
                "exit": 2,
                "available": "agent_profiles::names()",
            },
            "tool_outside_profile": {
                "why": "프로필은 추천이 아니라 서버가 제공하는 집합의 경계 — tools/call 우회 불가",
                "bypass": "tools/call 로도 우회할 수 없다 (allows_tool / allows_session_tool)",
                "example": "경영보고 프로필에서 hwp_fill_fields 는 목록에 없다",
            },
            "retry": False,
            "fix": "프로필을 바꾸거나 개발통합(필터 없음)을 쓴다. 이름을 발명하지 않는다. 우회 금지",
        },
        {
            "id": "search_combined_with_mcp",
            "title": "--search 와 --mcp/--profile 동시",
            "layer": "cli-json",
            "stderr": "오류: --search 는 --mcp/--profile 과 함께 쓸 수 없습니다.",
            "exit": 2,
            "fix": "한 번에 하나만. 검색 후 나온 이름을 --mcp 목록에서 확인",
        },
        {
            "id": "json_without_search",
            "title": "capabilities --json 만",
            "layer": "cli-json",
            "stderr": "오류: --json 은 --search 와 함께 사용합니다 (capabilities --search <키워드> --json).",
            "exit": 2,
            "note": "인자 없는 rhwp capabilities 는 언제나 JSON 이다. --json 을 붙이지 않는다",
        },
        {
            "id": "identical_false_is_data",
            "title": "identical:false 를 isError 로 오독",
            "layer": "cli-json",
            "cli_exit": 3,
            "mcp_is_error": False,
            "field": "identical",
            "fix": "봉투를 읽고 categories/diffCount 로 판단. 재시도 금지",
        },
        {
            "id": "replaced_zero_is_data",
            "title": "치환 0건을 성공으로 오독",
            "layer": "cli-json",
            "cli_exit": 0,
            "mcp_is_error": False,
            "field": "replacedCount",
            "side_effect": "출력 파일 없음, output 키 없음",
            "fix": "replacedCount > 0 을 확인한 뒤에만 output 을 읽는다",
        },
        {
            "id": "not_found_is_data",
            "title": "notFound 가 찬 exit 0",
            "layer": "cli-json",
            "cli_exit": 0,
            "mcp_is_error": False,
            "field": "notFound",
            "done_when": "notFound == [] && ambiguous == []",
            "fix": "filledCount 를 완료 조건으로 쓰지 않는다",
        },
    ]
    for case in cases:
        dump(FIXT / "exceptions" / f"{case['id']}.json", case)


def write_add_surface() -> None:
    pieces = [
        {
            "id": "cli-json-command",
            "layer": "cli-json",
            "ssot": "명령 구현 + *_json_value",
            "steps": [
                "이슈 잠금 (assignee 또는 착수 코멘트)",
                "red 계약 테스트 tests/<name>_contract.rs 신설",
                "코어 함수 재사용 — 새 편집 로직 금지",
                "stdout 순수 JSON, 실패 시 0바이트 (run 예외는 문서화)",
                "schemaVersion + untrustedContent/untrustedFields 모든 모드",
                "capabilities_command_entries() 등재",
                "json 이면 mcp_tool_definitions() 한 줄",
                "cli_commands.md + 지식 지도 행",
                "드리프트 가드 green",
            ],
        },
        {
            "id": "mcp-stateless-tool",
            "layer": "mcp-stateless",
            "ssot": "mcp_tool_definitions()",
            "steps": [
                "대응하는 --json CLI 가 이미 있는가? 없으면 CLI 를 먼저",
                "tool() / tool_with_optional_args() 로 한 곳에서 선언",
                "inputSchema.required 와 cli.args 자리표시자 1:1",
                "선택 인자는 optionalArgs 만 — 미치환 문자열 사고 방지",
                "outputFields 에 판정 필드(overflow, identical, notFound) 명시",
                "코어 + 봉투 helper 재사용",
                "capabilities_mcp_covers_every_json_command green",
            ],
        },
        {
            "id": "mcp-session-tool",
            "layer": "mcp-session",
            "ssot": "ALL_SESSION_TOOLS + served_tools()",
            "steps": [
                "무상태 짝이 있는가? 없으면 무상태를 먼저",
                "ALL_SESSION_TOOLS 에 이름 추가 (한 곳)",
                "served_tools() 디스패치가 같은 코어를 부르게",
                "docId 필수, 닫힌 핸들 isError + nextCall hwp_open",
                "디스크 기록은 hwp_doc_save 만",
                "판정 어휘는 무상태 짝과 동형",
                "프로필 allows_session_tool 경계 확인",
            ],
        },
    ]
    dump(FIXT / "add_surface" / "kinds.json", {"kinds": pieces})
    checklist = [
        "stdout 순수성: --json 모드에서 stdout 에 JSON 하나(배치는 NDJSON)만",
        "실패 경로: 런타임 실패 시 stdout 비움, exit 1. 조립 오류 exit 2",
        "schemaVersion 필드 포함",
        "출처 표지: untrustedContent·untrustedFields 를 모든 모드에서",
        "무상태: inputSchema.required 와 cli.args 자리표시자 1:1",
        "세션: 닫힌 핸들 isError, 기록은 hwp_doc_save 만",
        "실패 응답에 nextCall{name,arguments,why}",
        "문서: cli_commands.md + 지식 지도 행",
    ]
    dump(FIXT / "add_surface" / "acceptance.json", {"items": checklist, "source": "agent_surface_playbook.md §3"})
    for p in pieces:
        dump(FIXT / "add_surface" / f"{p['id']}.json", p)


def write_envelopes() -> None:
    envelopes = [
        {
            "id": "ir_diff_not_identical",
            "command": "ir-diff",
            "mcp": "hwp_ir_diff",
            "cli_exit": 3,
            "isError": False,
            "fields": {"identical": False, "diffCount": 2},
            "read_as": "data",
            "sample": {
                "a": "samples/추진일정.hwp",
                "b": "out/추진일정.hwpx",
                "categories": {"cc": 1, "char_offsets[0]: A=32 vs B=16": 1},
                "diffCount": 2,
                "identical": False,
                "schemaVersion": "1.0",
                "untrustedContent": True,
                "untrustedFields": ["categories"],
            },
        },
        {
            "id": "replace_zero",
            "command": "edit replace-text",
            "mcp": "hwp_replace_text",
            "cli_exit": 0,
            "isError": False,
            "fields": {"replacedCount": 0},
            "read_as": "data",
            "sample": {
                "find": "존재하지않는문자열ZZZ",
                "replace": "X",
                "replacedCount": 0,
                "changedPages": None,
                "dryRun": False,
                "schemaVersion": "1.0",
            },
            "no_output_file": True,
        },
        {
            "id": "fill_not_found",
            "command": "edit fill-fields",
            "mcp": "hwp_fill_fields",
            "cli_exit": 0,
            "isError": False,
            "fields": {"notFound": ["없는필드"], "ambiguous": [{"name": "목차1", "matched": 1, "total": 5}]},
            "read_as": "data",
            "done_when": "notFound == [] && ambiguous == []",
            "sample": {
                "dryRun": True,
                "filledCount": 2,
                "notFound": ["없는필드"],
                "ambiguous": [{"name": "목차1", "matched": 1, "total": 5}],
                "untrustedContent": False,
                "untrustedFields": [],
            },
        },
        {
            "id": "search_zero",
            "command": "search",
            "mcp": "hwp_search",
            "cli_exit": 0,
            "isError": False,
            "fields": {"matchCount": 0, "totalMatchCount": 0},
            "read_as": "data",
        },
        {
            "id": "csv_invalid",
            "command": "csv-to-table",
            "mcp": "hwp_csv_to_table",
            "cli_exit": 2,
            "isError": True,
            "fields": {"invalid": [{"reason": "rowCountMismatch"}], "changedCount": 0},
            "read_as": "assembly-or-contract",
            "note": "한 칸도 쓰지 않는다",
        },
        {
            "id": "run_invalid",
            "command": "run",
            "mcp": "hwp_run_plan",
            "cli_exit": 2,
            "isError": False,
            "fields": {"invalid": [{"step": 1, "reason": "일치 0건"}]},
            "read_as": "data",
            "note": "MCP 는 isError:false. CLI 는 exit 2 이지만 봉투를 낸다",
        },
        {
            "id": "render_diff_over",
            "command": "render-diff",
            "mcp": "hwp_render_diff",
            "cli_exit": 3,
            "isError": False,
            "fields": {"status": "OVER", "regression": True},
            "read_as": "data",
        },
        {
            "id": "closed_handle_runtime",
            "command": "hwp_doc_search",
            "mcp": "hwp_doc_search",
            "cli_exit": None,
            "isError": True,
            "fields": {"error": "열려 있지 않은 핸들"},
            "read_as": "runtime",
        },
        {
            "id": "missing_file",
            "command": "info",
            "mcp": "hwp_info",
            "cli_exit": 1,
            "isError": True,
            "fields": {},
            "stdout_bytes": 0,
            "read_as": "runtime",
        },
        {
            "id": "usage_merged_cell",
            "command": "edit set-cell",
            "mcp": "hwp_set_cell",
            "cli_exit": 2,
            "isError": True,
            "stdout_bytes": 0,
            "read_as": "assembly",
            "hint": "(0,2) 는 병합으로 덮인 칸입니다 — 앵커 (0,1) 를 지정하세요.",
        },
    ]
    for env in envelopes:
        dump(FIXT / "envelopes" / f"{env['id']}.json", env)


def write_profiles(profiles: list[dict], session: list[str], stateless: list[dict]) -> None:
    dump(FIXT / "profiles" / "index.json", {"names": [p["name"] for p in profiles], "ssot": "src/agent_profiles.rs PROFILES"})
    stateless_names = [t["name"] for t in stateless]
    for p in profiles:
        blocked_stateless = []
        if p["stateless_tools"]:
            allow = set(p["stateless_tools"])
            blocked_stateless = [n for n in stateless_names if n not in allow]
        blocked_session = []
        if p["session"] is None:
            blocked_session = list(session)
        elif p["session"] == "SESSION_READ_TOOLS":
            # mutate tools blocked
            blocked_session = [
                n
                for n in session
                if n
                in {
                    "hwp_doc_replace_text",
                    "hwp_doc_set_cell",
                    "hwp_doc_fill_fields",
                    "hwp_doc_save",
                }
            ]
        dump(
            FIXT / "profiles" / f"{p['name']}.json",
            {
                "name": p["name"],
                "summary": p["summary"],
                "stateless_tools": p["stateless_tools"],
                "session": p["session"],
                "filter_none": p["stateless_tools"] == [] and p["session"] in {"ALL", "SOME"},
                "blocked_stateless_sample": blocked_stateless,
                "blocked_session": blocked_session,
                "boundary": "프로필은 추천이 아니라 서버가 제공하는 집합의 경계",
                "bypass_forbidden": True,
            },
        )


def write_drift() -> None:
    guards = [
        {
            "id": "capabilities_mcp_covers_every_json_command",
            "file": "tests/cli_json_contract.rs",
            "from": "capabilities_command_entries json:true",
            "to": "mcp_tool_definitions cli.command",
            "exempt": ["capabilities", "dump-pages"],
            "on_fail": "json 명령을 mcp_tool_definitions 에 추가",
        },
        {
            "id": "tools_list_matches_capabilities_manifest",
            "file": "tests/mcp_server_contract.rs",
            "from": "capabilities --mcp tools",
            "to": "mcp-serve tools/list (minus session)",
            "on_fail": "선언과 실행이 다른 배열을 쓰지 않는지 확인",
        },
        {
            "id": "capabilities_mcp_tool_definitions_contract",
            "file": "tests/cli_json_contract.rs",
            "checks": ["name", "description", "inputSchema.type=object", "cli.command"],
            "on_fail": "tool() helper 로 고친다",
        },
        {
            "id": "capabilities_covers_every_help_command",
            "file": "tests/cli_json_contract.rs",
            "from": "rhwp --help",
            "to": "capabilities commands[].name",
            "on_fail": "capabilities_command_entries 등재",
        },
        {
            "id": "capabilities_version_matches_version_flag",
            "file": "tests/cli_json_contract.rs",
            "from": "capabilities.version",
            "to": "rhwp --version",
            "on_fail": "version 원천을 하나로",
        },
        {
            "id": "skills_reference_only_real_commands",
            "file": "tests/skills_contract.rs",
            "from": ".claude/skills/**/SKILL.md rhwp <tok>",
            "to": "capabilities ∪ --help",
            "on_fail": "죽은 명령을 스킬에서 지운다",
        },
    ]
    for g in guards:
        dump(FIXT / "drift" / f"{g['id']}.json", g)
    dump(FIXT / "drift" / "index.json", {"guards": [g["id"] for g in guards], "rule": 1})


def write_transcripts(commands: list[dict], stateless: list[dict], session: list[str], profiles: list[dict]) -> None:
    dump(
        FIXT / "transcripts" / "capabilities_bare.json",
        {
            "invocation": "rhwp capabilities",
            "always_json": True,
            "do_not_pass_json_flag": True,
            "read": ["version", "commands", "formats", "exitCodes", "jsonContract", "batch", "schemaRegistry"],
            "first_call": True,
            "help_is_not_contract": True,
            "command_count_not_contract": True,
            "sample_keys": {
                "schemaVersion": "1.0",
                "tool": "rhwp",
                "formats.read": ["hwp5", "hwpx", "hwp3", "hml"],
                "formats.write": ["hwp5", "hwpx", "hml", "pdf", "svg", "png", "txt", "md", "doclang"],
                "exitCodes.0": "성공",
                "exitCodes.1": "런타임 실패 (읽기·파싱·렌더·쓰기)",
                "exitCodes.2": "사용법 오류",
                "exitCodes.3": "검증 단언 실패",
                "exitCodes.4": "--verify-pages 페이지 수 불일치",
            },
            "families": sorted({c["family"] for c in commands}),
        },
    )
    dump(
        FIXT / "transcripts" / "capabilities_mcp.json",
        {
            "invocation": "rhwp capabilities --mcp",
            "protocol": "mcp",
            "tools_from": "mcp_tool_definitions()",
            "session_tools_present": False,
            "tool_count_not_contract": True,
            "each_tool_has": ["name", "description", "inputSchema", "cli.command"],
            "placeholder_rule": "required 에 없는 값은 자리표시자로 쓰지 않는다",
            "sample_search": {
                "name": "hwp_search",
                "cli": {"command": "search", "args": ["search", "{path}", "--json", "--", "{query}"]},
                "inputSchema.required": ["path", "query"],
            },
            "stateless_names": [t["name"] for t in stateless],
        },
    )
    dump(
        FIXT / "transcripts" / "capabilities_search_redact.json",
        {
            "invocation": "rhwp capabilities --search redact",
            "why": "edit 하위 redact 가 name/summary/subcommands 에 걸려야 한다 (#3884 G4)",
            "and_semantics": True,
            "json_ok": True,
            "combine_mcp_forbidden": True,
        },
    )
    dump(
        FIXT / "transcripts" / "capabilities_mcp_profile.json",
        {
            "invocation": "rhwp capabilities --mcp --profile 행정서식",
            "profile_ssot": "src/agent_profiles.rs PROFILES",
            "unknown_exit": 2,
            "names": [p["name"] for p in profiles],
        },
    )
    dump(
        FIXT / "transcripts" / "tools_list_session.json",
        {
            "invocation": "printf initialize… | rhwp mcp-serve",
            "session_from": "ALL_SESSION_TOOLS",
            "session_tools": session,
            "not_in_capabilities_mcp": True,
            "handshake": [
                {"method": "initialize", "protocolVersion_request": "2024-11-05"},
                {"method": "notifications/initialized"},
                {"method": "tools/list"},
            ],
            "note": "요청 프로토콜과 응답 버전이 다를 수 있다. 응답 쪽을 기준으로.",
        },
    )
    dump(
        FIXT / "transcripts" / "export_capabilities_schema.json",
        {
            "invocation": "rhwp export-capabilities-schema --json",
            "why": "외부 소비자 코드 생성의 스키마 출처",
            "bare": "rhwp export-capabilities-schema --bare",
            "companion": "rhwp export-ir-schema --json",
        },
    )


def write_core_reuse() -> None:
    mapping = [
        {"surface": "hwp_fill_fields / hwp_doc_fill_fields", "core": "set_field_value_by_name_at / collect_field_records", "envelope": "fill_fields_json_value"},
        {"surface": "hwp_replace_text / hwp_doc_replace_text", "core": "replace_all_native", "envelope": "replace_text_json_value"},
        {"surface": "hwp_search / hwp_doc_search", "core": "grep", "envelope": "search_json_value"},
        {"surface": "hwp_export_tables / hwp_doc_tables", "core": "extract_tables", "envelope": "export_tables_json_value"},
        {"surface": "hwp_set_cell / hwp_doc_set_cell", "core": "set_cell + overflow probe", "envelope": "set_cell_json_value"},
        {"surface": "edit * -o / hwp_doc_save", "core": "edit_serialize", "envelope": "save/verify envelope"},
        {"surface": "hwp_fields / hwp_doc_fields", "core": "collect_field_records", "envelope": "fields_json_value"},
        {"surface": "hwp_extract_data / hwp_doc_extract_data", "core": "extract_data", "envelope": "extract_data_json_value"},
        {"surface": "hwp_export_structure / hwp_doc_structure", "core": "export_structure", "envelope": "export_structure_json_value"},
        {"surface": "hwp_ir_diff", "core": "ir_diff", "envelope": "ir_diff_json_value"},
    ]
    dump(FIXT / "reuse" / "core_map.json", {"rule": 2, "items": mapping, "do_not": "서버 전용 경로 신설"})
    for i, m in enumerate(mapping, 1):
        dump(FIXT / "reuse" / f"{i:02d}_{m['core'].split()[0]}.json", m)


def write_scenarios(stateless: list[dict], session: list[str], commands: list[dict]) -> None:
    """Representative operate/add scenarios — not a full dump, not gym."""
    json_cmds = [c["name"] for c in commands if c["json"]][:12]
    human_cmds = [c["name"] for c in commands if not c["json"]][:6]
    items = []
    for name in json_cmds:
        items.append(
            {
                "id": f"discover-{name}",
                "kind": "operate",
                "goal": f"{name} 가 어느 층에 있는가",
                "first": f"rhwp capabilities --search {name}",
                "then": "rhwp capabilities --mcp 에서 짝 확인",
                "layer": "cli-json",
            }
        )
    for name in human_cmds:
        items.append(
            {
                "id": f"discover-{name}",
                "kind": "operate",
                "goal": f"{name} 는 json:false 진단 축일 수 있다",
                "first": f"rhwp capabilities --search {name}",
                "then": "json:false 면 MCP 짝이 없을 수 있다 — 가드 제외를 감으로 지우지 말 것",
                "layer": "cli-human",
            }
        )
    for t in stateless[:12]:
        items.append(
            {
                "id": f"stateless-{t['name']}",
                "kind": "operate",
                "first": "rhwp capabilities --mcp",
                "tool": t["name"],
                "layer": "mcp-stateless",
            }
        )
    for name in session:
        items.append(
            {
                "id": f"session-{name}",
                "kind": "operate",
                "first": "tools/list (capabilities --mcp 아님)",
                "tool": name,
                "layer": "mcp-session",
                "needs_open": name.startswith("hwp_doc_"),
            }
        )
    for layer, sid in [
        ("cli-json", "add-cli-json"),
        ("mcp-stateless", "add-stateless"),
        ("mcp-session", "add-session"),
    ]:
        items.append(
            {
                "id": sid,
                "kind": "add",
                "layer": layer,
                "playbook": "mydocs/manual/agent_surface_playbook.md",
                "acceptance": "fixtures/add_surface/acceptance.json",
            }
        )
    dump(
        FIXT / "scenarios.json",
        {
            "ssot": "capabilities --mcp + tools/list + capabilities_command_entries",
            "counts_are_not_contracts": True,
            "not_gym": True,
            "items": items,
        },
    )


def main() -> None:
    session = extract_session_tools()
    read = extract_session_read_tools()
    stateless = extract_stateless_tools()
    commands = extract_cli_commands()
    profiles = extract_profiles()
    stateless_names = {t["name"] for t in stateless}

    write_layers()
    write_rules()
    write_allowlist(session, read, stateless, commands, profiles)
    write_command_cards(commands, stateless_names)
    write_tool_cards(stateless, session)
    write_search_transcripts(commands)
    write_exceptions()
    write_add_surface()
    write_envelopes()
    write_profiles(profiles, session, stateless)
    write_drift()
    write_transcripts(commands, stateless, session, profiles)
    write_core_reuse()
    write_scenarios(stateless, session, commands)

    meta = {
        "session": len(session),
        "stateless": len(stateless),
        "commands": len(commands),
        "profiles": len(profiles),
        "note": "counts are not contracts",
    }
    dump(FIXT / "meta.json", meta)
    print(json.dumps(meta, ensure_ascii=False))


if __name__ == "__main__":
    main()
