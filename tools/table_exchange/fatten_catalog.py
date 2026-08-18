#!/usr/bin/env python3
"""M-tbl 표 CSV 왕복 픽스처 고도화 (#5485).

devel 의 export-tables / table-to-csv / csv-to-table 계약을 읽어
치수·coveredCellNotEmpty·dry-run/verify 픽스처와 리포트 전사를
디스크에 쓴다.

    python tools/table_exchange/fatten_catalog.py
    python -m unittest tools.table_exchange.tests.test_fatten_catalog

바이너리 HWP 와 rhwp CLI 는 부르지 않는다.
새 편집 로직을 발명하지 않는다.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
if str(HERE.parent) not in sys.path:
    sys.path.insert(0, str(HERE.parent))

from table_exchange import CLAIM_ID, GENERATOR, ISSUE, KIND, SCHEMA_VERSION, SKILL
from table_exchange.cases import CASES, Case, assert_catalog_coverage, cases_by_family
from table_exchange.catalog import cli_contract_public
from table_exchange.render import (
    reason_counter,
    render_case_md,
    render_cases_tsv,
    render_family_md,
    render_matrix_md,
    render_summary_md,
)


SHOWCASE_IDS = {
    "D-hwp_table_test_t0-row_short_1",
    "D-hwp_table_test_t0-col_long_first",
    "D-table_001-both-2x2",
    "C-table001_header-first",
    "C-block_2x2-all",
    "C-header_plus_note-first",
    "R-recipe02-edited",
    "R-hwp_table_test_t0-preview",
    "R-hwp_table_test_t0-ctrl-lf",
    "V-recipe02-verify-ok",
    "V-hwp_table_test_t0-exit3-diff2",
    "V-hwpx_basic_01-identical",
    "E-scan-hwp_table_test_t0",
    "E-occ-table001_header",
    "T-hwp_table_test_t0-extract",
    "T-table_001-extract",
    "T-hwp_table_test_t0-bom",
    "T-recipe02-rfc4180",
    "T-unknown-table-99999",
}


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_json(path: Path, data: Any) -> None:
    write_text(path, json.dumps(data, ensure_ascii=False, indent=2))


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    lines = [json.dumps(row, ensure_ascii=False, separators=(",", ":")) for row in rows]
    write_text(path, "\n".join(lines) + ("\n" if lines else ""))


INLINE_KEYS = {"invalid", "covered", "mergedAnchors", "reasons", "argv"}


def write_ledger(path: Path, family: str, cards: list[dict[str, Any]]) -> None:
    """Pretty ledger, but keep repeated arrays on one line each."""
    path.parent.mkdir(parents=True, exist_ok=True)
    chunks = [
        "{",
        f'  "family": {json.dumps(family, ensure_ascii=False)},',
        f'  "issue": {ISSUE},',
        f'  "count": {len(cards)},',
        '  "cases": [',
    ]
    for idx, card in enumerate(cards):
        comma = "," if idx + 1 < len(cards) else ""
        inner: list[str] = ["    {"]
        keys = list(card.keys())
        for k_i, key in enumerate(keys):
            value = card[key]
            key_comma = "," if k_i + 1 < len(keys) else ""
            if key in INLINE_KEYS or isinstance(value, (list, dict)) and key in {
                "verify",
                "outputKept",
            }:
                dumped = json.dumps(value, ensure_ascii=False, separators=(",", ":"))
                inner.append(f"      {json.dumps(key)}: {dumped}{key_comma}")
            else:
                dumped = json.dumps(value, ensure_ascii=False)
                inner.append(f"      {json.dumps(key)}: {dumped}{key_comma}")
        inner.append(f"    }}{comma}")
        chunks.extend(inner)
    chunks.extend(["  ]", "}", ""])
    path.write_text("\n".join(chunks), encoding="utf-8", newline="\n")


def case_index_row(case: Case) -> dict[str, Any]:
    return {
        "caseId": case.case_id,
        "family": case.family,
        "command": case.command,
        "sample": case.sample,
        "tableIndex": case.table_index,
        "rows": case.rows,
        "cols": case.cols,
        "mode": case.mode,
        "expectExit": case.expect_exit,
        "writes": case.writes,
        "csvRoundtrip": case.csv_roundtrip,
        "invalidReasons": [item.get("reason") for item in case.invalid],
        "changedCount": len(case.changed),
        "coveredCount": case.occupancy_public.get("coveredCount", 0),
        "documented": case.documented,
    }


def occupancy_rows(case: Case) -> list[dict[str, Any]]:
    occ = case.occupancy_public
    rows: list[dict[str, Any]] = []
    for covered in occ.get("covered", []):
        rows.append(
            {
                "caseId": case.case_id,
                "kind": "covered",
                "row": covered["row"],
                "col": covered["col"],
                "anchorRow": covered["anchorRow"],
                "anchorCol": covered["anchorCol"],
            }
        )
    for anchor in occ.get("mergedAnchors", occ.get("anchors", [])):
        if anchor.get("rowSpan", 1) > 1 or anchor.get("colSpan", 1) > 1:
            rows.append(
                {
                    "caseId": case.case_id,
                    "kind": "merged-anchor",
                    "row": anchor["row"],
                    "col": anchor["col"],
                    "rowSpan": anchor["rowSpan"],
                    "colSpan": anchor["colSpan"],
                    "text": anchor.get("text", ""),
                }
            )
    return rows


def emit_showcase(out_root: Path, case: Case, written: list[str]) -> None:
    rel = f"fixtures/cases/{case.case_id}.json"
    write_json(out_root / rel, case.to_public_dict())
    written.append(rel)
    if case.csv_text is not None:
        csv_rel = f"fixtures/csv/{case.case_id}.csv"
        path = out_root / csv_rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(case.csv_text.encode("utf-8"))
        written.append(csv_rel)
    env_rel = f"fixtures/envelopes/{case.case_id}.json"
    write_json(out_root / env_rel, case.envelope)
    written.append(env_rel)
    occ_rows = occupancy_rows(case)
    if occ_rows:
        occ_rel = f"fixtures/occupancy/{case.case_id}.jsonl"
        write_jsonl(out_root / occ_rel, occ_rows)
        written.append(occ_rel)
    tr_rel = f"transcripts/{case.family}/{case.case_id}.md"
    write_text(out_root / tr_rel, render_case_md(case))
    written.append(tr_rel)


def coverage_blob(cases: list[Case]) -> dict[str, Any]:
    families = {family: len(items) for family, items in cases_by_family().items()}
    exits = Counter(str(case.expect_exit) for case in cases)
    reasons = reason_counter(cases)
    writes = sum(1 for case in cases if case.writes)
    return {
        "kind": KIND,
        "claimId": CLAIM_ID,
        "issue": ISSUE,
        "skill": SKILL,
        "schemaVersion": SCHEMA_VERSION,
        "generator": GENERATOR,
        "generatedAt": utc_now(),
        "caseCount": len(cases),
        "families": families,
        "exits": dict(sorted(exits.items())),
        "reasons": dict(reasons),
        "writeCases": writes,
        "dryRunNullPages": sum(
            1
            for case in cases
            if case.mode == "dry-run" and case.envelope.get("changedPages") is None
        ),
        "verifyExit3": sum(1 for case in cases if case.expect_exit == 3),
        "pageCountLogic": "out of scope",
        "editLogic": "out of scope — existing CLI only",
        "gym": "out of scope",
    }


def run(out_root: Path | None = None) -> dict[str, Any]:
    root = out_root or HERE
    assert_catalog_coverage()
    written: list[str] = []
    cases = list(CASES)

    write_json(root / "fixtures" / "cli_contract.json", cli_contract_public())
    written.append("fixtures/cli_contract.json")

    index_rows = [case_index_row(case) for case in cases]
    write_jsonl(root / "fixtures" / "index.jsonl", index_rows)
    written.append("fixtures/index.jsonl")

    write_jsonl(root / "fixtures" / "cases.jsonl", [case.to_ledger_card() for case in cases])
    written.append("fixtures/cases.jsonl")

    grouped = cases_by_family()
    for family, items in grouped.items():
        write_ledger(
            root / "fixtures" / "ledgers" / f"{family}.json",
            family,
            [case.to_ledger_card() for case in items],
        )
        written.append(f"fixtures/ledgers/{family}.json")
        write_text(root / "reports" / f"family_{family}.md", render_family_md(family, items))
        written.append(f"reports/family_{family}.md")

    for case in cases:
        if case.case_id in SHOWCASE_IDS:
            emit_showcase(root, case, written)

    write_text(root / "tables" / "cases.tsv", render_cases_tsv(cases))
    written.append("tables/cases.tsv")
    write_text(root / "reports" / "decision_matrix.md", render_matrix_md(cases))
    written.append("reports/decision_matrix.md")

    coverage = coverage_blob(cases)
    coverage["writtenFiles"] = len(set(written))
    write_json(root / "reports" / "fatten_summary.json", coverage)
    written.append("reports/fatten_summary.json")
    write_text(root / "reports" / "fatten_summary.md", render_summary_md(coverage))
    written.append("reports/fatten_summary.md")
    write_json(root / "reports" / "coverage.json", coverage)
    written.append("reports/coverage.json")

    write_json(
        root / "reports" / "incorporation_manifest.json",
        {
            "issue": ISSUE,
            "claimId": CLAIM_ID,
            "showcase": sorted(SHOWCASE_IDS),
            "families": coverage["families"],
            "commands": ["export-tables", "table-to-csv", "csv-to-table"],
            "forbidden": [
                "new CLI",
                "DocumentCore edit invention",
                "merge split/join writer",
                "gym/",
                "other live seats",
            ],
        },
    )
    written.append("reports/incorporation_manifest.json")

    # loops: documented dry-run → verify for recipe 02
    recipe = next(case for case in cases if case.case_id == "R-recipe02-edited")
    verify = next(case for case in cases if case.case_id == "V-recipe02-verify-ok")
    write_json(
        root / "fixtures" / "loops" / "roundtrip_plain.json",
        {
            "id": "roundtrip_plain",
            "sample": recipe.sample,
            "steps": [
                {
                    "id": "scan",
                    "command": ["rhwp", "export-tables", recipe.sample, "--json"],
                    "expectExit": 0,
                },
                {
                    "id": "extract",
                    "command": [
                        "rhwp",
                        "table-to-csv",
                        recipe.sample,
                        "--table",
                        "0",
                        "-o",
                        "table0.csv",
                        "--json",
                    ],
                    "expectExit": 0,
                },
                {
                    "id": "dry-run",
                    "command": recipe.argv,
                    "expectExit": 0,
                    "forbidFieldsPresent": ["output"],
                    "expect": {"changedCount": recipe.envelope["changedCount"], "dryRun": True},
                },
                {
                    "id": "write-verify",
                    "command": verify.argv,
                    "expectExit": 0,
                    "expect": {
                        "changedCount": verify.envelope["changedCount"],
                        "verify.identical": True,
                    },
                },
            ],
        },
    )
    written.append("fixtures/loops/roundtrip_plain.json")

    dim = next(case for case in cases if case.case_id == "D-table_001-both-2x2")
    write_json(
        root / "fixtures" / "loops" / "dimension_reject.json",
        {
            "id": "dimension_reject",
            "sample": dim.sample,
            "steps": [
                {
                    "id": "scan",
                    "command": ["rhwp", "export-tables", dim.sample, "--json"],
                    "expectExit": 0,
                    "expect": {"rows": 19, "cols": 9},
                },
                {
                    "id": "dry-run-bad",
                    "command": dim.argv,
                    "expectExit": 2,
                    "expect": {"changedCount": 0},
                    "expectReasons": ["rowCountMismatch", "colCountMismatch"],
                },
            ],
        },
    )
    written.append("fixtures/loops/dimension_reject.json")

    covered = next(case for case in cases if case.case_id == "C-table001_header-first")
    write_json(
        root / "fixtures" / "loops" / "merge_fallback.json",
        {
            "id": "merge_fallback",
            "sample": covered.sample,
            "forbiddenNext": ["csv-to-table"],
            "steps": [
                {
                    "id": "scan",
                    "command": ["rhwp", "export-tables", covered.sample, "--json"],
                    "expectExit": 0,
                },
                {
                    "id": "set-cell",
                    "command": [
                        "rhwp",
                        "edit",
                        "set-cell",
                        covered.sample,
                        "--table",
                        "0",
                        "--row",
                        "0",
                        "--col",
                        "1",
                        "--text",
                        "5월",
                    ],
                    "expectExit": 0,
                },
            ],
        },
    )
    written.append("fixtures/loops/merge_fallback.json")

    write_json(
        root / "schema" / "case.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.table_exchange.case.v1",
            "type": "object",
            "required": [
                "caseId",
                "family",
                "command",
                "expectExit",
                "writes",
                "envelope",
            ],
            "properties": {
                "caseId": {"type": "string"},
                "family": {
                    "enum": [
                        "dimension",
                        "covered",
                        "dry-run",
                        "verify",
                        "export-tables",
                        "table-to-csv",
                    ]
                },
                "command": {
                    "enum": ["export-tables", "table-to-csv", "csv-to-table"]
                },
                "expectExit": {"enum": [0, 1, 2, 3]},
                "writes": {"type": "boolean"},
            },
        },
    )
    written.append("schema/case.v1.json")

    coverage["written"] = sorted(set(written))
    write_json(root / "reports" / "fatten_summary.json", coverage)
    return coverage


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=None)
    args = parser.parse_args(argv)
    coverage = run(args.out)
    print(
        json.dumps(
            {
                "issue": coverage["issue"],
                "caseCount": coverage["caseCount"],
                "families": coverage["families"],
                "writtenFiles": coverage["writtenFiles"],
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
