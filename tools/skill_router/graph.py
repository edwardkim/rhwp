#!/usr/bin/env python3
"""Build the execution graph for a classified intent.

Node: {id, skill, action, command}. Edge: {from, to}.
Lookup is by intent id *or* catalog skill id so every catalog skill
gets a real DAG (never a single dummy node).
"""

from __future__ import annotations

from typing import Any, Callable


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
                "rhwp replay --plan-json <계획> --capsule work.capsule.json --json; "
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
            ("doctor", "doctor", "rhwp --version"),
            ("binary", "binary", "rhwp --version"),
            (
                "selftest",
                "selftest",
                "rhwp info samples/basic/english.hwp --json; "
                "rhwp export-text samples/basic/english.hwp --json --max-chars 2000",
            ),
            ("mcp-json", "mcp-json", "rhwp mcp-serve"),
            (
                "first-5-min",
                "first-5-min",
                "rhwp explore samples/basic/english.hwp --json",
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
                "rhwp fields <파일> --json",
            ),
            (
                "dry-run",
                "dry-run",
                "rhwp edit replace-text <파일> --dry-run --json",
            ),
            (
                "apply-verify",
                "apply --verify",
                "rhwp edit replace-text <파일> -o <출력> --verify --json",
            ),
            (
                "reread",
                "reread",
                "rhwp search <산출> --json --limit N -- <질의>",
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
                "rhwp inspect hidden-text <산출> --json; "
                "rhwp inspect injection <산출> --json; "
                "rhwp inspect unicode <산출> --json",
            ),
        ],
    )


def bulk_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("list", "list", "rhwp batch info --json < 목록.txt"),
            ("batch-info", "batch info", "rhwp batch info --json < 목록.txt"),
            (
                "batch-axis",
                "batch axis",
                "rhwp batch export-text --json",
            ),
            (
                "split-retry",
                "jq split/retry",
                "rhwp batch search --json < 실패목록.txt",
            ),
            (
                "n-gate",
                "N=성공+실패",
                "rhwp batch info --json",
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
                "rhwp mcp-serve",
            ),
            (
                "manifest",
                "capabilities --mcp",
                "rhwp capabilities --mcp",
            ),
            ("open", "hwp_open", "rhwp info <파일> --json"),
            (
                "doc",
                "hwp_doc_*",
                "rhwp search <파일> --json --limit N -- <질의>",
            ),
            ("close", "hwp_close", "rhwp digest <파일> --json"),
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
                "rhwp export-provenance-map --json",
            ),
            (
                "inspect",
                "inspect 3축",
                "rhwp inspect injection <파일> --json",
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
                "rhwp --version",
            ),
            (
                "normalize",
                "normalize input",
                "rhwp info <입력> --json",
            ),
            (
                "ingest",
                "ingest.json",
                "rhwp explore <입력> --json",
            ),
            (
                "crop",
                "crop",
                "rhwp export-png <파일> -p 0",
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
                "rhwp capabilities",
            ),
            (
                "tree",
                "request tree",
                "rhwp capabilities --search <키워드>",
            ),
            (
                "search",
                "capabilities --search",
                "rhwp capabilities --search <키워드>",
            ),
            (
                "chapter",
                "chapter",
                "rhwp info <파일> --json",
            ),
        ],
    )


def explore_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("explore", "explore", "rhwp explore <파일> --json"),
            ("info", "info", "rhwp info <파일> --json"),
            ("fields", "fields", "rhwp fields <파일> --json"),
            (
                "export-tables",
                "export-tables",
                "rhwp export-tables <파일> --json",
            ),
        ],
    )


def agent_surface_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("capabilities", "capabilities", "rhwp capabilities"),
            (
                "capabilities-mcp",
                "capabilities --mcp",
                "rhwp capabilities --mcp",
            ),
            (
                "capabilities-search",
                "capabilities --search",
                "rhwp capabilities --search <키워드>",
            ),
        ],
    )


def bug_hunter_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("info", "info", "rhwp info <파일> --json"),
            ("export-svg", "export-svg", "rhwp export-svg <파일> -p 0"),
            ("render-diff", "render-diff", "rhwp render-diff <파일>"),
            ("inspect", "inspect injection", "rhwp inspect injection <파일> --json"),
        ],
    )


def chief_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("info", "info", "rhwp info <파일> --json"),
            ("export-pdf", "export-pdf", "rhwp export-pdf <파일> -o <출력.pdf>"),
            (
                "export-tables",
                "export-tables",
                "rhwp export-tables <파일> --json",
            ),
            (
                "fill",
                "fill-fields",
                "rhwp edit fill-fields <파일> --data <JSON> -o <출력> --json",
            ),
        ],
    )


def fde_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("capabilities", "capabilities", "rhwp capabilities"),
            ("info", "info", "rhwp info <파일> --json"),
            ("explain", "explain", "rhwp explain <파일> --json"),
            (
                "export-structure",
                "export-structure",
                "rhwp export-structure <파일> --json",
            ),
            ("digest", "digest", "rhwp digest <파일> --json"),
        ],
    )


def fidelity_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("info", "info", "rhwp info <파일> --json"),
            (
                "export-svg",
                "export-svg",
                "rhwp export-svg <파일> --font-style -p 0",
            ),
            (
                "export-render-tree",
                "export-render-tree",
                "rhwp export-render-tree <파일> -p 0",
            ),
        ],
    )


def handoff_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            (
                "replay",
                "replay capsule",
                "rhwp replay --plan-json <계획> --capsule <파일> --json",
            ),
            (
                "parent",
                "replay --parent",
                "rhwp replay --plan-json <계획> --capsule <파일> --parent <이전> --json",
            ),
            ("lineage", "lineage", "rhwp lineage <머리캡슐> --json"),
        ],
    )


def knowledge_map_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("capabilities", "capabilities", "rhwp capabilities"),
            (
                "capabilities-mcp",
                "capabilities --mcp",
                "rhwp capabilities --mcp",
            ),
            (
                "capabilities-search",
                "capabilities --search",
                "rhwp capabilities --search <키워드>",
            ),
        ],
    )


def recipes_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("fields", "fields", "rhwp fields <파일> --json"),
            (
                "export-tables",
                "export-tables",
                "rhwp export-tables <파일> --json",
            ),
            (
                "inspect",
                "inspect hidden-text",
                "rhwp inspect hidden-text <파일> --json",
            ),
            ("batch", "batch info", "rhwp batch info --json"),
            (
                "render-diff",
                "render-diff",
                "rhwp render-diff <파일> --via hwpx",
            ),
        ],
    )


def strategist_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("info", "info", "rhwp info <파일> --json"),
            (
                "search",
                "search",
                "rhwp search <파일> --json --limit N -- <질의>",
            ),
            (
                "extract-data",
                "extract-data",
                "rhwp extract-data <파일> --json --kind date|amount|number",
            ),
            (
                "capabilities-search",
                "capabilities --search",
                "rhwp capabilities --search <키워드>",
            ),
        ],
    )


def route_graph(skill: str) -> dict[str, Any]:
    return _chain(
        skill,
        [
            ("capabilities", "capabilities", "rhwp capabilities"),
            ("info", "info", "rhwp info <파일> --json"),
            ("export-svg", "export-svg", "rhwp export-svg <파일> -p 0"),
        ],
    )


def _document_fallback_graph(skill: str) -> dict[str, Any]:
    """Real DAG for an unknown skill — never a single dummy node."""
    return _chain(
        skill,
        [
            ("info", "info", "rhwp info <파일> --json"),
            ("explore", "explore", "rhwp explore <파일> --json"),
            ("export-svg", "export-svg", "rhwp export-svg <파일> -p 0"),
        ],
    )


Builder = Callable[[str], dict[str, Any]]

# Intent-id keys (intents.py + catalog.json intents[]) and catalog skill ids.
_BUILDERS: dict[str, Builder] = {
    # intents.py slugs
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
    "route": route_graph,
    # catalog.json intents[]
    "add-surface": agent_surface_graph,
    "capabilities-ssot": agent_surface_graph,
    "hunt-bug": bug_hunter_graph,
    "batch-folder": bulk_graph,
    "run-request-queue": chief_graph,
    "analyze-export": inspect_cli_graph,
    "debug-layout": inspect_cli_graph,
    "navigate-codex": codex_graph,
    "open-pr": contribute_graph,
    "triage-doc": triage_graph,
    "ingest-exam": exam_ingest_graph,
    "explore-doc": explore_graph,
    "triage-symptom": fde_graph,
    "compare-fidelity": fidelity_graph,
    "handoff-session": handoff_graph,
    "find-canonical": knowledge_map_graph,
    "attach-mcp": mcp_graph,
    "onboard-agent": onboard_graph,
    "mark-provenance": provenance_graph,
    "pick-recipe": recipes_graph,
    "security-sweep": security_graph,
    "build-strategy": strategist_graph,
    "table-csv-roundtrip": table_csv_graph,
    "visual-regression": visual_graph,
    "attest-work": receipt_graph,
    # catalog.json skill ids
    "rhwp-agent-surface": agent_surface_graph,
    "rhwp-bug-hunter": bug_hunter_graph,
    "rhwp-bulk-pipeline": bulk_graph,
    "rhwp-chief": chief_graph,
    "rhwp-cli": inspect_cli_graph,
    "rhwp-codex": codex_graph,
    "rhwp-contributor": contribute_graph,
    "rhwp-doc-triage": triage_graph,
    "rhwp-exam-ingest": exam_ingest_graph,
    "rhwp-explore": explore_graph,
    "rhwp-fde": fde_graph,
    "rhwp-fidelity-compare": fidelity_graph,
    "rhwp-form-fill": fill_form_graph,
    "rhwp-handoff": handoff_graph,
    "rhwp-knowledge-map": knowledge_map_graph,
    "rhwp-mcp-session": mcp_graph,
    "rhwp-onboarding": onboard_graph,
    "rhwp-provenance": provenance_graph,
    "rhwp-recipes": recipes_graph,
    "rhwp-safe-edit": safe_edit_graph,
    "rhwp-security-sweep": security_graph,
    "rhwp-skill-router": route_graph,
    "rhwp-strategist": strategist_graph,
    "rhwp-table-exchange": table_csv_graph,
    "rhwp-visual-regression": visual_graph,
    "rhwp-work-receipt": receipt_graph,
}


def build_graph(intent_id: str, skill: str) -> dict[str, Any]:
    builder = _BUILDERS.get(intent_id) or _BUILDERS.get(skill)
    if builder is None:
        return _document_fallback_graph(skill)
    return builder(skill)
