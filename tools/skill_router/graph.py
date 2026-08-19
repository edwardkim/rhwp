#!/usr/bin/env python3
"""Build the execution graph for a classified intent.

Node: {id, skill, action, command}. Edge: {from, to}.
"""

from __future__ import annotations

from typing import Any


def _chain(skill: str, steps: list[tuple[str, str, str]]) -> dict[str, Any]:
    nodes = [
        {"id": node_id, "skill": skill, "action": action, "command": command}
        for node_id, action, command in steps
    ]
    edges = [
        {"from": steps[i][0], "to": steps[i + 1][0]}
        for i in range(len(steps) - 1)
    ]
    return {"nodes": nodes, "edges": edges}


def contribute_graph(skill: str) -> dict[str, Any]:
    """CONTRIBUTING.md order: issue → analyze → branch → implement → gate → doc → pr."""
    return _chain(
        skill,
        [
            (
                "issue",
                "issue",
                "gh issue list; gh pr list --search <키워드>; "
                "없으면 gh issue create (DoD·판단 근거)",
            ),
            (
                "analyze",
                "analyze",
                "mydocs/manual/README.md 선택표와 기존 계약 테스트를 읽고 이슈에 기록",
            ),
            (
                "branch",
                "branch(upstream/devel)",
                "git fetch upstream devel; isolation worktree from upstream/devel",
            ),
            (
                "implement",
                "implement",
                "기존 결을 따라 구현. git add <경로> (git add -A 금지)",
            ),
            (
                "fmt-clippy-test",
                "fmt/clippy/test",
                "cargo fmt --all -- --check; cargo clippy -- -D warnings; cargo test",
            ),
            (
                "working-doc",
                "working-doc",
                "mydocs/working/<이름>.md 에 무엇·왜·어떻게·검증 실측",
            ),
            (
                "pr",
                "pr(devel, Korean template, closes #)",
                "gh pr create --base devel --body-file <한국어 템플릿> (closes #)",
            ),
        ],
    )


def fill_form_graph(skill: str) -> dict[str, Any]:
    """fields → dry-run fill → fill --verify → sanitize."""
    return _chain(
        skill,
        [
            ("fields", "fields", "rhwp fields <서식> --json"),
            (
                "dry-run-fill",
                "dry-run fill",
                "rhwp edit fill-fields <서식> --data <JSON> --dry-run --json",
            ),
            (
                "fill-verify",
                "fill --verify",
                "rhwp edit fill-fields <서식> --data <JSON> -o <출력> --verify --json",
            ),
            (
                "sanitize",
                "sanitize",
                "rhwp edit sanitize <산출> -o <제출본> --json",
            ),
        ],
    )


def onboard_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "doctor",
                "doctor",
                "python tools/agent_onboarding/rhwp_doctor.py --json",
            ),
            ("binary", "binary", "rhwp --version"),
            (
                "selftest",
                "selftest",
                "rhwp info samples/basic/english.hwp --json; "
                "rhwp export-text samples/basic/english.hwp --json --max-chars 2000",
            ),
            (
                "mcp-json",
                "mcp-json",
                "emit .mcp.json {mcpServers.rhwp: rhwp mcp-serve}",
            ),
            (
                "first-5-min",
                "first-5-min",
                "triage → tables → fields → inspect → replay 지도",
            ),
        ],
    )


def triage_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("info", "info", "rhwp info <파일> --json"),
            ("explain", "explain", "rhwp explain <파일> --json"),
            (
                "export-structure",
                "export-structure",
                "rhwp export-structure <파일> --json",
            ),
            ("digest", "digest", "rhwp digest <파일> --json --max-chars N"),
            ("search", "search", "rhwp search <파일> --json --limit N -- <질의>"),
            (
                "extract-data",
                "extract-data",
                "rhwp extract-data <파일> --json --kind date|amount|number",
            ),
        ],
    )


def table_csv_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("export-tables", "export-tables", "rhwp export-tables <파일> --json"),
            (
                "table-to-csv",
                "table-to-csv",
                "rhwp table-to-csv <파일> --table N -o <csv> --json",
            ),
            (
                "csv-dry-run",
                "csv-to-table dry-run",
                "rhwp csv-to-table <파일> --csv <csv> --table N --dry-run --json",
            ),
            (
                "csv-verify",
                "csv-to-table --verify",
                "rhwp csv-to-table <파일> --csv <csv> --table N -o <출력> --verify --json",
            ),
        ],
    )


def safe_edit_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "discover",
                "discover",
                "rhwp fields <파일> --json 또는 rhwp export-tables <파일> --json",
            ),
            (
                "dry-run",
                "dry-run",
                "rhwp edit <하위명령> <파일> --dry-run --json (또는 rhwp run <계획> --dry-run --json)",
            ),
            (
                "apply-verify",
                "apply --verify",
                "rhwp edit <하위명령> <파일> -o <출력> --verify --json",
            ),
            (
                "reread",
                "reread",
                "rhwp search|export-tables <산출> --json 으로 재독 대조",
            ),
        ],
    )


def security_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "hidden-text",
                "inspect hidden-text",
                "rhwp inspect hidden-text <파일> --json",
            ),
            (
                "injection",
                "inspect injection",
                "rhwp inspect injection <파일> --json",
            ),
            (
                "unicode",
                "inspect unicode",
                "rhwp inspect unicode <파일> --json",
            ),
            (
                "redact-dry-run",
                "redact dry-run",
                "rhwp edit redact <파일> --dry-run --no-raw --json",
            ),
            (
                "redact-sanitize",
                "redact/sanitize",
                "rhwp edit redact <파일> -o <마스킹> --no-raw --verify --json; "
                "rhwp edit sanitize <마스킹> -o <배포본> --json",
            ),
            (
                "resweep",
                "resweep",
                "findingCount==0 AND inspect 3축 clean==true",
            ),
        ],
    )


def bulk_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("list", "list", "한 줄당 경로 하나인 목록.txt"),
            ("batch-info", "batch info", "rhwp batch info --json < 목록.txt"),
            (
                "batch-axis",
                "batch axis",
                "rhwp batch <export-text|export-tables|search|convert|fill> --json",
            ),
            (
                "split-retry",
                "jq split/retry",
                "jq 로 실패 행만 분리해 재시도. 성공 행은 다시 돌리지 않음",
            ),
            (
                "n-gate",
                "N=성공+실패",
                "입력 N == 성공 + 실패 게이트",
            ),
        ],
    )


def visual_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "self-diff",
                "render-diff self",
                "rhwp render-diff <파일> [--via hwpx|hwp]",
            ),
            (
                "pair-diff",
                "render-diff pair",
                "rhwp render-diff <전> <후> [--max-disp PX] [-p N]",
            ),
            ("ir-diff", "ir-diff", "rhwp ir-diff <A> <B> --json"),
            (
                "eye",
                "thumbnail/export-png",
                "rhwp thumbnail <파일> --json; rhwp export-png <파일> [-p N]",
            ),
        ],
    )


def receipt_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "attest",
                "replay attest",
                "rhwp replay --plan-json <계획> --json",
            ),
            (
                "capsule",
                "capsule --parent",
                "rhwp replay --plan-json <계획> --capsule <파일> [--parent <이전>] --json",
            ),
            ("audit", "audit", "rhwp audit <캡슐폴더> --json"),
            ("lineage", "lineage", "rhwp lineage <머리캡슐> --json"),
        ],
    )


def mcp_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "register",
                ".mcp.json",
                '{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }',
            ),
            (
                "manifest",
                "capabilities --mcp",
                "rhwp capabilities --mcp; tools/list (세션 정본)",
            ),
            ("open", "hwp_open", 'hwp_open {"path":"<절대경로>"}'),
            ("doc", "hwp_doc_*", "hwp_doc_search|hwp_doc_fill_fields|hwp_doc_render_page|hwp_doc_save"),
            ("close", "hwp_close", 'hwp_close {"docId":"<id>"}'),
        ],
    )


def provenance_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "map",
                "export-provenance-map",
                "rhwp export-provenance-map --json",
            ),
            (
                "flags",
                "untrustedFields",
                "봉투의 untrustedContent / untrustedFields 를 읽고 문서 파생 값만 격리",
            ),
            (
                "inspect",
                "inspect 3축",
                "rhwp inspect injection|hidden-text|unicode <파일> --json",
            ),
            ("armor", "armor", "rhwp armor <파일> --json"),
        ],
    )


def exam_ingest_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "deps",
                "check_deps",
                "bash .claude/skills/rhwp-exam-ingest/helpers/check_deps.sh --json",
            ),
            (
                "normalize",
                "normalize input",
                "pdf_to_pngs.sh | extract_docx.py | image passthrough | MD ![alt](path)",
            ),
            (
                "ingest",
                "ingest.json",
                "Vision 구조 인식 후 tools/rhwp-ingest/schema/ingest_schema_v1.json 으로 기록",
            ),
            (
                "crop",
                "crop",
                "bash .claude/skills/rhwp-exam-ingest/helpers/crop_image.sh <src> x y w h <out>",
            ),
            (
                "build",
                "build-from-ingest",
                "rhwp build-from-ingest <ingest.json> --media-dir <dir> -o <out.hwpx>",
            ),
            (
                "gate",
                "dump/export-text",
                "rhwp dump <out.hwpx>; rhwp export-text <out.hwpx>",
            ),
        ],
    )


def inspect_cli_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "overlay",
                "export-svg overlay",
                "rhwp export-svg <파일> --debug-overlay -p N",
            ),
            ("dump-pages", "dump-pages", "rhwp dump-pages <파일> -p N"),
            ("dump", "dump", "rhwp dump <파일> -s N -p M"),
            ("ir-diff", "ir-diff", "rhwp ir-diff <a.hwpx> <b.hwp> --json"),
            (
                "render-tree",
                "export-render-tree",
                "rhwp export-render-tree <파일> -p N",
            ),
        ],
    )


def codex_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "covenants",
                "covenants",
                "mydocs/manual/agent_codex/00_서문.md (판정=데이터·결정론·출처 표지·원본 무훼손)",
            ),
            (
                "tree",
                "request tree",
                "mydocs/manual/agent_codex/01_판단트리.md 로 장 번호",
            ),
            (
                "search",
                "capabilities --search",
                "rhwp capabilities --search <키워드>",
            ),
            (
                "chapter",
                "chapter",
                "해당 생성 장의 실측 표본을 흉내. 깊으면 이웃 스킬로 인계",
            ),
        ],
    )


_BUILDERS = {
    "contribute": contribute_graph,
    "fill-form": fill_form_graph,
    "onboard": onboard_graph,
    "triage": triage_graph,
    "table-csv": table_csv_graph,
    "safe-edit": safe_edit_graph,
    "security": security_graph,
    "bulk": bulk_graph,
    "visual": visual_graph,
    "receipt": receipt_graph,
    "mcp": mcp_graph,
    "provenance": provenance_graph,
    "exam-ingest": exam_ingest_graph,
    "inspect-cli": inspect_cli_graph,
    "codex": codex_graph,
}


def build_graph(intent_id: str, skill: str) -> dict[str, Any]:
    builder = _BUILDERS.get(intent_id, inspect_cli_graph)
    return builder(skill)
