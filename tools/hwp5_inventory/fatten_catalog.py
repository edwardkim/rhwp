#!/usr/bin/env python3
"""M-hwp5 저장 계약 인벤토리 픽스처 고도화 (#5469).

devel 의 hwp5-inventory / inventory-diff / table-probe 계약을 읽어
케이스 픽스처·JSONL 인벤토리·리포트 전사·커버리지 표를 디스크에 쓴다.

    python tools/hwp5_inventory/fatten_catalog.py
    python -m unittest tools.hwp5_inventory.tests.test_fatten_catalog

시리얼라이저 페이지 수 로직은 읽거나 쓰지 않는다.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable

HERE = Path(__file__).resolve().parent
if str(HERE.parent) not in sys.path:
    sys.path.insert(0, str(HERE.parent))

from hwp5_inventory import CLAIM_ID, GENERATOR, ISSUE, KIND, SCHEMA_VERSION
from hwp5_inventory.cases import CASES, assert_catalog_coverage
from hwp5_inventory.catalog import (
    ALIGN_MODES,
    CLI_EXIT_CODES,
    CONTROLS,
    DIFF_COLUMNS,
    DIFF_KINDS,
    FAILURE_CLASSES,
    FOCUS_MODES,
    HANCOM_JUDGMENTS,
    INVENTORY_COLUMNS,
    PROBE_AXES,
    PROBE_VARIANTS,
    REPORT_MODES,
    SHAPE_INNER_IDS,
    TABLE_CTRL_FIELDS,
    TABLE_RECORD_FIELDS,
    TAGS,
    ctrl_id_hex,
)
from hwp5_inventory.model import (
    build_index_diff,
    build_lcs_diff,
    is_table_candidate,
    table_probe_axes,
)
from hwp5_inventory.render import (
    render_bundles_markdown,
    render_diff_markdown,
    render_hints_markdown,
    render_inventory_markdown,
    render_table_fields_markdown,
    render_table_probe_generation,
    render_table_probe_plan_markdown,
)


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_json(path: Path, data: Any) -> None:
    write_text(path, json.dumps(data, ensure_ascii=False, indent=2))


def write_jsonl(path: Path, rows: Iterable[dict[str, Any]]) -> None:
    lines = [json.dumps(row, ensure_ascii=False, separators=(",", ":")) for row in rows]
    write_text(path, "\n".join(lines) + ("\n" if lines else ""))


def md_cell(value: Any) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")


def item_public(item) -> dict[str, Any]:
    data = item.to_public_dict()
    data["payload_hex"] = item.payload_hex
    return data


def emit_catalogs(out_root: Path, written: list[str]) -> None:
    tag_rows = [
        {
            "tag_id": tag.tag_id,
            "tag_id_hex": f"0x{tag.tag_id:03x}",
            "tag_name": tag.tag_name,
            "tuple_role": tag.role,
            "owner": tag.owner,
            "stream_hint": tag.stream_hint,
            "required_children": list(tag.required_children),
            "inventory_note": tag.inventory_note,
        }
        for tag in TAGS
    ]
    write_jsonl(out_root / "fixtures" / "tags.jsonl", tag_rows)
    written.append("fixtures/tags.jsonl")

    control_rows = [
        {
            "fourcc": control.fourcc,
            "ctrl_id": control.ctrl_id,
            "ctrl_id_hex": ctrl_id_hex(control.fourcc),
            "ctrl_name": control.ctrl_name,
            "family": control.family,
            "required_tuple": list(control.required_tuple),
            "inventory_focus": control.inventory_focus,
            "failure_hint": control.failure_hint,
            "is_field": control.fourcc.startswith("%"),
        }
        for control in CONTROLS
    ]
    write_jsonl(out_root / "fixtures" / "controls.jsonl", control_rows)
    written.append("fixtures/controls.jsonl")

    field_rows = [
        {
            "record_kind": spec.record_kind,
            "field_name": spec.field_name,
            "offset": spec.offset,
            "offset_hex": f"0x{spec.offset:02x}",
            "width": spec.width,
            "kind": spec.kind,
            "probe_axis": spec.probe_axis,
            "observation_name": spec.observation_name,
            "meaning": spec.meaning,
        }
        for spec in (*TABLE_CTRL_FIELDS, *TABLE_RECORD_FIELDS)
    ]
    write_jsonl(out_root / "fixtures" / "fields.jsonl", field_rows)
    written.append("fixtures/fields.jsonl")

    write_json(
        out_root / "fixtures" / "failure_classes.json",
        {
            "schema": "hwp5InventoryFailureClass.v1",
            "source": "mydocs/troubleshootings/hwpx2hwp-rule.md §5",
            "classes": [
                {
                    "code": item.code,
                    "name": item.name,
                    "inspect": list(item.inspect),
                    "signals": list(item.signals),
                    "inventory_columns": list(item.inventory_columns),
                    "typical_diff_kinds": list(item.typical_diff_kinds),
                    "typical_focus": item.typical_focus,
                    "next_probe": item.next_probe,
                }
                for item in FAILURE_CLASSES
            ],
        },
    )
    written.append("fixtures/failure_classes.json")

    write_json(
        out_root / "fixtures" / "cli_contract.json",
        {
            "schema": "hwp5InventoryCliContract.v1",
            "align_modes": list(ALIGN_MODES),
            "report_modes": list(REPORT_MODES),
            "focus_modes": list(FOCUS_MODES),
            "diff_kinds": list(DIFF_KINDS),
            "inventory_columns": list(INVENTORY_COLUMNS),
            "diff_columns": list(DIFF_COLUMNS),
            "hancom_judgments": list(HANCOM_JUDGMENTS),
            "probe_axes": [
                {
                    "name": name,
                    "record_kind": kind,
                    "fields": fields,
                    "description": desc,
                }
                for name, kind, fields, desc in PROBE_AXES
            ],
            "probe_variants": [
                {"name": name, "axes": list(axes), "purpose": purpose}
                for name, axes, purpose in PROBE_VARIANTS
            ],
            "shape_inner_ids": [
                {"fourcc": fourcc, "kind": kind} for fourcc, kind in SHAPE_INNER_IDS
            ],
            "exit_codes": list(CLI_EXIT_CODES),
            "page_count_owned_by": 4882,
            "notes": [
                "stdout 는 데이터 전용, 사용법은 stderr.",
                "인자 없음은 exit 2. --help 만 exit 0.",
                "페이지 수 로직은 이 도구가 바꾸지 않는다.",
            ],
        },
    )
    written.append("fixtures/cli_contract.json")


def emit_schemas(out_root: Path, written: list[str]) -> None:
    write_json(
        out_root / "schema" / "inventory_item.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Hwp5InventoryItem",
            "type": "object",
            "required": list(INVENTORY_COLUMNS),
            "properties": {name: {"type": ["string", "integer", "null"]} for name in INVENTORY_COLUMNS},
        },
    )
    write_json(
        out_root / "schema" / "inventory_diff.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Hwp5InventoryDiffItem",
            "type": "object",
            "required": list(DIFF_COLUMNS),
            "properties": {name: {} for name in DIFF_COLUMNS},
        },
    )
    write_json(
        out_root / "schema" / "table_probe_plan.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Hwp5TableProbePlan",
            "type": "object",
            "required": ["case_id", "axes", "variants"],
        },
    )
    write_json(
        out_root / "schema" / "fatten_catalog.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Hwp5InventoryFattenCatalog",
            "type": "object",
            "required": ["kind", "claimId", "schemaVersion", "issue", "caseCount"],
        },
    )
    written.extend(
        [
            "schema/inventory_item.v1.json",
            "schema/inventory_diff.v1.json",
            "schema/table_probe_plan.v1.json",
            "schema/fatten_catalog.v1.json",
        ]
    )


SHOWCASE_IDS = {
    "T01",
    "T02",
    "T03",
    "T04",
    "T05",
    "T06",
    "T07",
    "T08",
    "S01",
    "S02",
    "P01",
    "P03",
    "D01",
    "D02",
    "C01",
    "C02",
    "F01",
    "F04",
    "G02",
    "X01",
    "X16",
}


def emit_case(out_root: Path, case, written: list[str]) -> dict[str, Any]:
    oracle_items, generated_items = case.build()
    index_items, index_stats = build_index_diff(oracle_items, generated_items)
    lcs_items, lcs_stats = build_lcs_diff(oracle_items, generated_items)
    preferred = lcs_items if case.align_preferred == "lcs" else index_items
    preferred_stats = lcs_stats if case.align_preferred == "lcs" else index_stats
    table_candidates = [item for item in preferred if is_table_candidate(item)]
    axes = table_probe_axes(table_candidates, oracle_items, generated_items)

    case_dir = out_root / "fixtures" / "cases"
    write_text(
        case_dir / f"{case.case_id}.json",
        json.dumps(
        {
            "case_id": case.case_id,
            "sample": case.sample,
            "construct": case.construct,
            "family": case.family,
            "failure_class": case.failure_class,
            "hancom_judgment": case.hancom_judgment,
            "align_preferred": case.align_preferred,
            "focus": case.focus,
            "probe_axes": list(case.probe_axes),
            "next_probe": case.next_probe,
            "lowering_contract": case.lowering_contract,
            "contract_status": case.contract_status,
            "oracle_path": case.oracle_path,
            "generated_path": case.generated_path,
            "notes": list(case.notes),
            "oracle_record_count": len(oracle_items),
            "generated_record_count": len(generated_items),
            "index": {
                "matched": index_stats.matched,
                "changed": index_stats.changed,
                "missing": index_stats.missing,
                "extra": index_stats.extra,
                "diff_count": len(index_items),
            },
            "lcs": {
                "matched": lcs_stats.matched,
                "changed": lcs_stats.changed,
                "missing": lcs_stats.missing,
                "extra": lcs_stats.extra,
                "diff_count": len(lcs_items),
            },
            "table_candidate_count": len(table_candidates),
            "affected_probe_axes": {
                axis.name: len(axis.rows) for axis in axes if axis.rows
            },
        },
            ensure_ascii=False,
            separators=(",", ":"),
        ),
    )
    written.append(f"fixtures/cases/{case.case_id}.json")

    write_jsonl(
        out_root / "fixtures" / "inventories" / f"{case.case_id}.oracle.jsonl",
        (item_public(item) for item in oracle_items),
    )
    write_jsonl(
        out_root / "fixtures" / "inventories" / f"{case.case_id}.generated.jsonl",
        (item_public(item) for item in generated_items),
    )
    write_jsonl(
        out_root / "fixtures" / "diffs" / f"{case.case_id}.index.jsonl",
        (item.to_dict() for item in index_items),
    )
    write_jsonl(
        out_root / "fixtures" / "diffs" / f"{case.case_id}.lcs.jsonl",
        (item.to_dict() for item in lcs_items),
    )
    written.extend(
        [
            f"fixtures/inventories/{case.case_id}.oracle.jsonl",
            f"fixtures/inventories/{case.case_id}.generated.jsonl",
            f"fixtures/diffs/{case.case_id}.index.jsonl",
            f"fixtures/diffs/{case.case_id}.lcs.jsonl",
        ]
    )

    want_plan = case.family == "table" or bool(table_candidates) or case.case_id in SHOWCASE_IDS
    if want_plan:
        write_text(
            out_root / "fixtures" / "table_probe" / f"{case.case_id}.plan.json",
            json.dumps(
            {
                "case_id": case.case_id,
                "sample": case.sample,
                "candidate_count": len(table_candidates),
                "axes": [
                    {
                        "name": axis.name,
                        "record_kind": axis.record_kind,
                        "description": axis.description,
                        "affected_records": len(axis.rows),
                        "rows": [
                            {
                                "key": row.key,
                                "oracle_record": row.oracle_record,
                                "generated_record": row.generated_record,
                                "fields": row.fields,
                                "oracle_values": row.oracle_values,
                                "generated_values": row.generated_values,
                            }
                            for row in axis.rows
                        ],
                    }
                    for axis in axes
                ],
                "variants": [
                    {"name": name, "axes": list(names), "purpose": purpose}
                    for name, names, purpose in PROBE_VARIANTS
                ],
            },
                ensure_ascii=False,
                separators=(",", ":"),
            ),
        )
        written.append(f"fixtures/table_probe/{case.case_id}.plan.json")

    if case.case_id in SHOWCASE_IDS or case.family == "table":
        write_text(
            out_root / "transcripts" / "inventory" / f"{case.case_id}.oracle.md",
            render_inventory_markdown(oracle_items, source=case.oracle_path, sample=case.sample),
        )
        write_text(
            out_root / "transcripts" / "inventory" / f"{case.case_id}.generated.md",
            render_inventory_markdown(
                generated_items, source=case.generated_path, sample=case.sample
            ),
        )
        write_text(
            out_root / "transcripts" / "inventory_diff" / f"{case.case_id}.diff.md",
            render_diff_markdown(
                preferred,
                preferred_stats,
                oracle_path=case.oracle_path,
                generated_path=case.generated_path,
                align_mode=case.align_preferred,
            ),
        )
        write_text(
            out_root / "transcripts" / "inventory_diff" / f"{case.case_id}.hints.md",
            render_hints_markdown(
                preferred,
                preferred_stats,
                oracle_path=case.oracle_path,
                generated_path=case.generated_path,
                align_mode=case.align_preferred,
            ),
        )
        write_text(
            out_root / "transcripts" / "inventory_diff" / f"{case.case_id}.bundles.md",
            render_bundles_markdown(
                preferred,
                oracle_items,
                generated_items,
                preferred_stats,
                oracle_path=case.oracle_path,
                generated_path=case.generated_path,
                align_mode=case.align_preferred,
                focus=case.focus,
            ),
        )
        written.extend(
            [
                f"transcripts/inventory/{case.case_id}.oracle.md",
                f"transcripts/inventory/{case.case_id}.generated.md",
                f"transcripts/inventory_diff/{case.case_id}.diff.md",
                f"transcripts/inventory_diff/{case.case_id}.hints.md",
                f"transcripts/inventory_diff/{case.case_id}.bundles.md",
            ]
        )
        if table_candidates or case.family == "table":
            write_text(
                out_root / "transcripts" / "inventory_diff" / f"{case.case_id}.table-fields.md",
                render_table_fields_markdown(
                    preferred,
                    oracle_items,
                    generated_items,
                    oracle_path=case.oracle_path,
                    generated_path=case.generated_path,
                    align_mode=case.align_preferred,
                ),
            )
            write_text(
                out_root / "transcripts" / "inventory_diff" / f"{case.case_id}.table-probe-plan.md",
                render_table_probe_plan_markdown(
                    preferred,
                    oracle_items,
                    generated_items,
                    oracle_path=case.oracle_path,
                    generated_path=case.generated_path,
                    align_mode=case.align_preferred,
                ),
            )
            write_text(
                out_root / "transcripts" / "table_probe" / f"{case.case_id}.generation.md",
                render_table_probe_generation(
                    case.case_id,
                    case.sample,
                    axes,
                    oracle_path=case.oracle_path,
                    generated_path=case.generated_path,
                ),
            )
            written.extend(
                [
                    f"transcripts/inventory_diff/{case.case_id}.table-fields.md",
                    f"transcripts/inventory_diff/{case.case_id}.table-probe-plan.md",
                    f"transcripts/table_probe/{case.case_id}.generation.md",
                ]
            )

    return {
        "case_id": case.case_id,
        "sample": case.sample,
        "family": case.family,
        "failure_class": case.failure_class,
        "hancom_judgment": case.hancom_judgment,
        "contract_status": case.contract_status,
        "align_preferred": case.align_preferred,
        "focus": case.focus,
        "oracle_records": len(oracle_items),
        "generated_records": len(generated_items),
        "index_diff": len(index_items),
        "lcs_diff": len(lcs_items),
        "table_candidates": len(table_candidates),
        "construct": case.construct,
        "next_probe": case.next_probe,
    }


def emit_reports(out_root: Path, summaries: list[dict[str, Any]], written: list[str]) -> dict[str, Any]:
    family_counts = Counter(row["family"] for row in summaries)
    class_counts = Counter(row["failure_class"] for row in summaries)
    judgment_counts = Counter(row["hancom_judgment"] for row in summaries)
    status_counts = Counter(row["contract_status"] for row in summaries)
    align_counts = Counter(row["align_preferred"] for row in summaries)

    coverage = {
        "kind": KIND,
        "claimId": CLAIM_ID,
        "schemaVersion": SCHEMA_VERSION,
        "issue": ISSUE,
        "generatedAt": utc_now(),
        "generator": GENERATOR,
        "caseCount": len(summaries),
        "tagCount": len(TAGS),
        "controlCount": len(CONTROLS),
        "fieldCount": len(TABLE_CTRL_FIELDS) + len(TABLE_RECORD_FIELDS),
        "failureClasses": sorted(class_counts),
        "families": dict(sorted(family_counts.items())),
        "failureClassCounts": dict(sorted(class_counts.items())),
        "hancomJudgments": dict(judgment_counts),
        "contractStatus": dict(status_counts),
        "alignPreferred": dict(align_counts),
        "pageCountLogic": "out of scope — owned by #4882",
        "forbiddenPaths": [
            "src/serializer page-count export",
            "canvaskit_policy",
            "pdf",
            "layout-anomaly",
            "oracle_public",
            "render_backend",
            "proptest",
            "fidelity_compare",
            "gym",
        ],
    }
    write_json(out_root / "reports" / "coverage.json", coverage)
    written.append("reports/coverage.json")

    cov_md = [
        "# HWP5 inventory fatten coverage",
        "",
        f"- claim: `{CLAIM_ID}`",
        f"- issue: #{ISSUE}",
        f"- cases: **{len(summaries)}**",
        f"- tags: {len(TAGS)}",
        f"- controls: {len(CONTROLS)}",
        "",
        "## Family",
        "",
        "| family | cases |",
        "|---|---:|",
    ]
    for family, count in sorted(family_counts.items()):
        cov_md.append(f"| `{family}` | {count} |")
    cov_md.extend(["", "## Failure class", "", "| class | cases |", "|---|---:|"])
    for code, count in sorted(class_counts.items()):
        cov_md.append(f"| `{code}` | {count} |")
    cov_md.extend(
        [
            "",
            "## Hancom judgment",
            "",
            "| judgment | cases |",
            "|---|---:|",
        ]
    )
    for name, count in judgment_counts.most_common():
        cov_md.append(f"| {md_cell(name)} | {count} |")
    cov_md.extend(
        [
            "",
            "## 하지 않은 것",
            "",
            "- 시리얼라이저 페이지 수 로직 (#4882 석)",
            "- canvaskit_policy / pdf / layout-anomaly / oracle_public / render_backend / proptest / fidelity_compare",
            "- gym/",
            "",
        ]
    )
    write_text(out_root / "reports" / "coverage.md", "\n".join(cov_md))
    written.append("reports/coverage.md")

    matrix = [
        "# Failure class × family",
        "",
        "| class \\ family | "
        + " | ".join(sorted(family_counts))
        + " | total |",
        "|" + "---|" * (len(family_counts) + 2),
    ]
    grid: dict[str, Counter[str]] = defaultdict(Counter)
    for row in summaries:
        grid[row["failure_class"]][row["family"]] += 1
    for code in ("A", "B", "C", "D", "E", "F"):
        cells = [str(grid[code][family]) for family in sorted(family_counts)]
        matrix.append(f"| `{code}` | " + " | ".join(cells) + f" | {sum(grid[code].values())} |")
    matrix.append("")
    write_text(out_root / "reports" / "failure_class_matrix.md", "\n".join(matrix))
    written.append("reports/failure_class_matrix.md")

    probe_md = [
        "# Table probe axis matrix",
        "",
        "| case | sample | outer_margin | common_attr | table_attr | table_tail | next probe |",
        "|---|---|---:|---:|---:|---:|---|",
    ]
    for row in summaries:
        plan_path = out_root / "fixtures" / "table_probe" / f"{row['case_id']}.plan.json"
        if plan_path.is_file():
            plan = json.loads(plan_path.read_text(encoding="utf-8"))
            counts = {axis["name"]: axis["affected_records"] for axis in plan["axes"]}
        else:
            counts = {}
        probe_md.append(
            "| `{id}` | `{sample}` | {m} | {c} | {a} | {t} | {next} |".format(
                id=row["case_id"],
                sample=row["sample"],
                m=counts.get("ctrl_outer_margin", 0),
                c=counts.get("ctrl_common_attr", 0),
                a=counts.get("table_attr", 0),
                t=counts.get("table_tail", 0),
                next=md_cell(row["next_probe"]),
            )
        )
    probe_md.append("")
    write_text(out_root / "reports" / "probe_axis_matrix.md", "\n".join(probe_md))
    written.append("reports/probe_axis_matrix.md")

    index_md = [
        "# Contract case index",
        "",
        "| id | sample | family | class | judgment | status | index Δ | lcs Δ | construct |",
        "|---|---|---|---|---|---|---:|---:|---|",
    ]
    for row in summaries:
        index_md.append(
            "| `{id}` | `{sample}` | `{family}` | `{cls}` | {judge} | `{status}` | {idx} | {lcs} | {construct} |".format(
                id=row["case_id"],
                sample=row["sample"],
                family=row["family"],
                cls=row["failure_class"],
                judge=md_cell(row["hancom_judgment"]),
                status=row["contract_status"],
                idx=row["index_diff"],
                lcs=row["lcs_diff"],
                construct=md_cell(row["construct"]),
            )
        )
    index_md.append("")
    write_text(out_root / "reports" / "pair_index.md", "\n".join(index_md))
    written.append("reports/pair_index.md")

    summary = {
        **coverage,
        "written": written,
        "cases": summaries,
    }
    write_json(out_root / "reports" / "fatten_summary.json", summary)
    written.append("reports/fatten_summary.json")
    write_text(
        out_root / "reports" / "fatten_summary.md",
        "\n".join(
            [
                "# M-hwp5 fatten summary",
                "",
                f"- cases: {len(summaries)}",
                f"- written files: {len(written)}",
                f"- families: {', '.join(f'{k}={v}' for k, v in sorted(family_counts.items()))}",
                f"- failure classes: {', '.join(f'{k}={v}' for k, v in sorted(class_counts.items()))}",
                "",
                "한 줄:",
                "",
                "```",
                "python tools/hwp5_inventory/fatten_catalog.py",
                "python -m unittest tools.hwp5_inventory.tests.test_fatten_catalog tools.hwp5_inventory.tests.test_model tools.hwp5_inventory.tests.test_cases tools.hwp5_inventory.tests.test_transcripts",
                "```",
                "",
            ]
        ),
    )
    written.append("reports/fatten_summary.md")
    return coverage


def run(out_root: Path) -> dict[str, Any]:
    assert_catalog_coverage()
    written: list[str] = []
    emit_schemas(out_root, written)
    emit_catalogs(out_root, written)
    summaries = [emit_case(out_root, case, written) for case in CASES]
    coverage = emit_reports(out_root, summaries, written)
    write_json(
        out_root / "reports" / "incorporation_manifest.json",
        {
            "kind": KIND,
            "claimId": CLAIM_ID,
            "issue": ISSUE,
            "schemaVersion": SCHEMA_VERSION,
            "generator": GENERATOR,
            "caseCount": len(summaries),
            "written": written,
        },
    )
    coverage["writtenCount"] = len(written)
    return coverage


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Emit HWP5 inventory fatten fixtures")
    parser.add_argument("--out-root", type=Path, default=HERE)
    args = parser.parse_args(argv)
    coverage = run(args.out_root)
    print(
        json.dumps(
            {
                "ok": True,
                "cases": coverage["caseCount"],
                "written": coverage.get("writtenCount"),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
