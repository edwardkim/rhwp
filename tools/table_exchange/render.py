"""Markdown / TSV transcripts for the fatten catalog."""

from __future__ import annotations

from collections import Counter
from typing import Any, Iterable

from .cases import Case


def md_cell(value: Any) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")


def render_summary_md(coverage: dict[str, Any]) -> str:
    lines = [
        "# M-tbl 표 CSV 왕복 픽스처 요약",
        "",
        f"- issue: #{coverage['issue']}",
        f"- cases: **{coverage['caseCount']}**",
        f"- generator: `{coverage['generator']}`",
        "",
        "## 가족",
        "",
        "| family | count |",
        "|---|---:|",
    ]
    for family, count in coverage["families"].items():
        lines.append(f"| {family} | {count} |")
    lines.extend(
        [
            "",
            "## 종료 코드",
            "",
            "| exit | count |",
            "|---|---:|",
        ]
    )
    for exit_code, count in coverage["exits"].items():
        lines.append(f"| {exit_code} | {count} |")
    lines.extend(
        [
            "",
            "## invalid reason",
            "",
            "| reason | cases |",
            "|---|---:|",
        ]
    )
    for reason, count in coverage["reasons"].items():
        lines.append(f"| {reason} | {count} |")
    lines.extend(
        [
            "",
            "## 하지 않은 것",
            "",
            "- 새 CLI 없음",
            "- DocumentCore 편집 로직 없음",
            "- 병합 풀기·표 리사이즈 없음",
            "- gym/ 미수정",
            "- 다른 진행 석 파일 미수정",
            "",
        ]
    )
    return "\n".join(lines)


def render_family_md(family: str, cases: list[Case]) -> str:
    lines = [
        f"# {family} 케이스",
        "",
        f"{len(cases)}건. 기존 CLI 계약만.",
        "",
        "| id | sample | table | size | exit | writes | next |",
        "|---|---|---:|---|---:|---|---|",
    ]
    for case in cases:
        lines.append(
            "| {id} | {sample} | {idx} | {rows}×{cols} | {exit} | {writes} | {next} |".format(
                id=md_cell(case.case_id),
                sample=md_cell(case.sample),
                idx=case.table_index,
                rows=case.rows,
                cols=case.cols,
                exit=case.expect_exit,
                writes="yes" if case.writes else "no",
                next=md_cell(case.next_action),
            )
        )
    lines.append("")
    return "\n".join(lines)


def render_case_md(case: Case) -> str:
    reasons = ",".join(item.get("reason", "") for item in case.invalid) or "—"
    lines = [
        f"# {case.case_id}",
        "",
        f"- family: `{case.family}`",
        f"- command: `{case.command}`",
        f"- sample: `{case.sample}`",
        f"- table: {case.table_index} ({case.rows}×{case.cols})",
        f"- mode: `{case.mode}`",
        f"- exit: {case.expect_exit}",
        f"- writes: {str(case.writes).lower()}",
        f"- csvRoundtrip: `{case.csv_roundtrip}`",
        f"- invalid: {reasons}",
        f"- changedCount: {len(case.changed)}",
        f"- next: {case.next_action}",
        "",
        case.notes,
        "",
        "## argv",
        "",
        "```bash",
        " ".join(case.argv),
        "```",
        "",
    ]
    if case.csv_text is not None:
        preview = case.csv_text if len(case.csv_text) <= 800 else case.csv_text[:800] + "\n…"
        lines.extend(["## csv", "", "```csv", preview.replace("\r\n", "\n"), "```", ""])
    if case.invalid:
        lines.extend(["## invalid[]", "", "```json"])
        import json

        lines.append(json.dumps(case.invalid, ensure_ascii=False, indent=2))
        lines.extend(["```", ""])
    lines.extend(
        [
            "## 점유",
            "",
            f"- cellCount: {case.occupancy_public.get('cellCount')}",
            f"- coveredCount: {case.occupancy_public.get('coveredCount')}",
            f"- mergedAnchorCount: {case.occupancy_public.get('mergedAnchorCount')}",
            f"- areaSum: {case.occupancy_public.get('areaSum')} / grid {case.occupancy_public.get('gridArea')}",
            "",
        ]
    )
    return "\n".join(lines)


def render_matrix_md(cases: list[Case]) -> str:
    lines = [
        "# 치수·병합 판정 행렬",
        "",
        "| id | family | rows | cols | covered | reasons | exit | writes |",
        "|---|---|---:|---:|---:|---|---:|---|",
    ]
    for case in cases:
        reasons = ",".join(sorted({item.get("reason", "") for item in case.invalid})) or "—"
        covered = case.occupancy_public.get("coveredCount", 0)
        lines.append(
            f"| {md_cell(case.case_id)} | {case.family} | {case.rows} | {case.cols} | {covered} | {md_cell(reasons)} | {case.expect_exit} | {str(case.writes).lower()} |"
        )
    lines.append("")
    return "\n".join(lines)


def tsv_escape(value: Any) -> str:
    return str(value).replace("\t", " ").replace("\n", " ").replace("\r", "")


def render_cases_tsv(cases: Iterable[Case]) -> str:
    header = (
        "caseId",
        "family",
        "command",
        "sample",
        "tableIndex",
        "rows",
        "cols",
        "mode",
        "expectExit",
        "writes",
        "csvRoundtrip",
        "invalidReasons",
        "changedCount",
        "coveredCount",
        "nextAction",
    )
    rows = ["\t".join(header)]
    for case in cases:
        reasons = ",".join(item.get("reason", "") for item in case.invalid)
        rows.append(
            "\t".join(
                tsv_escape(part)
                for part in (
                    case.case_id,
                    case.family,
                    case.command,
                    case.sample,
                    case.table_index,
                    case.rows,
                    case.cols,
                    case.mode,
                    case.expect_exit,
                    case.writes,
                    case.csv_roundtrip,
                    reasons,
                    len(case.changed),
                    case.occupancy_public.get("coveredCount", 0),
                    case.next_action,
                )
            )
        )
    return "\n".join(rows) + "\n"


def reason_counter(cases: list[Case]) -> Counter[str]:
    counter: Counter[str] = Counter()
    for case in cases:
        seen = {item.get("reason", "") for item in case.invalid}
        for reason in seen:
            if reason:
                counter[reason] += 1
    return counter
