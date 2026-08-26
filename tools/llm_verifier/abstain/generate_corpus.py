#!/usr/bin/env python3
"""Emit the V-abstain corpus of contradictory vs consistent field tuples.

Each row is a distinct EnvelopeFields tuple plus expected abstain/pass/fail.
Comment padding is not used. Sample identity is derived from the tuple so
two rows never share the same field combination.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterator

try:
    from .decide import decide
    from .schema import (
        CASE_COLUMNS,
        CLAIM_ID,
        COMMANDS,
        SCHEMA_VERSION,
        AbstainCase,
        EnvelopeFields,
        VERDICT_ABSTAIN,
        VERDICT_FAIL,
        VERDICT_PASS,
    )
except ImportError:
    from decide import decide
    from schema import (
        CASE_COLUMNS,
        CLAIM_ID,
        COMMANDS,
        SCHEMA_VERSION,
        AbstainCase,
        EnvelopeFields,
        VERDICT_ABSTAIN,
        VERDICT_FAIL,
        VERDICT_PASS,
    )

HERE = Path(__file__).resolve().parent
CORPUS_DIR = HERE / "corpus"
DEFAULT_TARGET = 108360
SHARD_ROWS = 6020

AGENCIES = (
    "법제처",
    "행정안전부",
    "국세청",
    "대법원",
    "특허청",
    "교육부",
    "보건복지부",
    "국토교통부",
    "고용노동부",
    "외교부",
    "기획재정부",
    "공정거래위원회",
    "금융위원회",
    "방송통신위원회",
    "개인정보보호위원회",
    "국민권익위원회",
    "국가인권위원회",
    "통계청",
    "기상청",
    "관세청",
    "검찰청",
    "경찰청",
    "소방청",
    "해양경찰청",
    "병무청",
    "산림청",
    "농촌진흥청",
    "중소벤처기업부",
    "과학기술정보통신부",
    "문화체육관광부",
    "환경부",
    "해양수산부",
)

KINDS = (
    "고시",
    "훈령",
    "예규",
    "공고",
    "지침",
    "서식",
    "질의회신",
    "업무계획",
    "예산서",
    "회의록",
)

FAMILIES = (
    "기안문",
    "편람",
    "시험지",
    "계약서",
    "용역보고서",
    "입법예고",
    "보도자료",
    "내부결재",
    "민원회신",
    "현장점검",
    "안전관리",
    "정보공개",
)

YEARS = tuple(str(y) for y in range(2016, 2026))

NODES = (
    "page/0/body",
    "page/0/para/0",
    "page/0/table/0",
    "page/0/table/0/cell/0/0",
    "page/1/header",
    "page/1/footer",
    "page/2/shape/0",
    "page/2/para/3/run/1",
    "section/0/ctrl/tbl",
    "section/0/ctrl/gso",
    "footnote/0",
    "endnote/0",
)


def rule_seeds(page: int, mag: int, node: str) -> tuple[dict[str, Any], ...]:
    """Minimal conflicting field sets. Each seed is a named contradiction."""
    n = 1 + (mag % 7)
    other = f"{node}/alt"
    return (
        {"identical": True, "has_signal": True, "exit": 0},
        {"reproduced": True, "exit": 3},
        {"reproduced": True, "exit": 4},
        {
            "page_count_a": page,
            "page_count_b": page,
            "page_count_mismatch": False,
            "struct_status": "STRUCT_MISMATCH",
            "struct_node": node,
            "page_count_node": node,
            "exit": 1,
        },
        {
            "page_count_a": page,
            "page_count_b": page,
            "page_count_mismatch": True,
            "exit": 1,
        },
        {
            "page_count_a": page,
            "page_count_b": page + n,
            "page_count_mismatch": False,
            "exit": 1,
        },
        {"verify_identical": True, "verify_diff_count": n, "exit": 3},
        {"verify_identical": False, "verify_diff_count": 0, "exit": 3},
        {"identical": True, "diff_count": n, "exit": 3},
        {"identical": True, "fail_count": n, "exit": 3},
        {"identical": True, "verify_identical": False, "exit": 3},
        {"verdict": "pass", "fail_count": n, "exit": 3},
        {"verdict": "pass", "identical": False, "exit": 3},
        {"verdict": "fail", "identical": True, "fail_count": 0, "exit": 3},
        {"exit": 0, "identical": False},
        {"exit": 0, "reproduced": False},
        {"exit": 0, "verdict": "fail"},
        {"exit": 0, "page_count_mismatch": True, "page_count_a": page, "page_count_b": page + n},
        {
            "exit": 0,
            "struct_status": "STRUCT_MISMATCH",
            "struct_node": node,
            "page_count_node": other,
        },
        {"exit": 3, "verdict": "pass"},
        {
            "exit": 3,
            "identical": True,
            "has_signal": False,
            "fail_count": 0,
            "reproduced": True,
        },
        {"exit": 4, "page_count_mismatch": False, "page_count_a": page, "page_count_b": page},
        {"has_signal": False, "overflow_count": n, "exit": 0},
        {"has_signal": False, "overlap_count": n, "exit": 0},
        {"has_signal": False, "signal_count": n, "exit": 0},
        {
            "has_signal": True,
            "signal_count": 0,
            "overflow_count": 0,
            "overlap_count": 0,
            "finding_count": 0,
            "exit": 3,
        },
        {"clean": True, "finding_count": n, "exit": 3},
        {"valid": True, "fail_count": n, "exit": 3},
        {"regression": True, "status": "OK", "exit": 3},
        {"regression": False, "status": "FAIL", "exit": 3},
        {"identical": True, "regression": True, "exit": 3},
        {"identical": True, "clean": False, "exit": 3},
        {"identical": True, "valid": False, "exit": 3},
        {"reproduced": True, "identical": False, "exit": 1},
        {"status": "OK", "fail_count": n, "exit": 3},
        {"pass_count": n, "fail_count": 0, "verdict": "fail", "exit": 3},
    )


def _colorize(fields: EnvelopeFields, **_unused: Any) -> EnvelopeFields:
    return fields


def _identity(serial: int, command: str) -> dict[str, str]:
    agency = AGENCIES[serial % len(AGENCIES)]
    kind = KINDS[(serial // len(AGENCIES)) % len(KINDS)]
    family = FAMILIES[(serial // (len(AGENCIES) * len(KINDS))) % len(FAMILIES)]
    year = YEARS[(serial // 17) % len(YEARS)]
    fmt = "hwpx" if serial % 2 == 0 else "hwp"
    sample = f"samples/{agency}/{year}/{family}-{kind}-{serial:06d}.{fmt}"
    return {
        "sample": sample,
        "source_format": fmt,
        "agency": agency,
        "doc_kind": kind,
        "year": year,
        "family": family,
    }


def _make_case(serial: int, fields: EnvelopeFields) -> AbstainCase:
    decision = decide(fields)
    ident = _identity(serial, fields.command)
    return AbstainCase(
        case_id=f"v-abstain-{serial:06d}",
        fields=fields,
        expected=decision.verdict,
        contradiction_id=decision.contradiction_id,
        success_tokens="|".join(decision.success_tokens),
        fail_tokens="|".join(decision.fail_tokens),
        sample=ident["sample"],
        source_format=ident["source_format"],
        agency=ident["agency"],
        doc_kind=ident["doc_kind"],
        year=ident["year"],
        family=ident["family"],
    )


def iter_abstain_fields() -> Iterator[EnvelopeFields]:
    pages = tuple(range(1, 16))
    mags = tuple(range(0, 4))
    colors = tuple(range(0, 6))
    for command in COMMANDS:
        for page in pages:
            for mag in mags:
                for node in NODES:
                    for color in colors:
                        for seed in rule_seeds(page, mag, node):
                            payload: dict[str, Any] = {
                                "command": command,
                                "exit": 0,
                                "empty_page_count": color,
                                "struct_node": seed.get("struct_node", node),
                                "page_count_node": seed.get("page_count_node", node),
                            }
                            if "page_count_a" not in seed:
                                payload["page_count_a"] = page
                            payload.update(seed)
                            yield EnvelopeFields.from_mapping(payload)


def iter_consistent_pass() -> Iterator[EnvelopeFields]:
    """All present fields lean success. empty_page_count is not a fail."""
    for command in COMMANDS:
        for pages in range(1, 16):
            for empty in range(0, 4):
                for passed in range(0, 6):
                    for node in NODES[:6]:
                        yield EnvelopeFields(
                            command=command,
                            exit=0,
                            identical=True,
                            has_signal=False,
                            reproduced=True if command == "replay" else None,
                            page_count_a=pages,
                            page_count_b=pages,
                            page_count_mismatch=False,
                            struct_status="PASS",
                            struct_node=node,
                            page_count_node=node,
                            verify_identical=True if command in {"fill-fields", "ir-diff"} else None,
                            verify_diff_count=0 if command in {"fill-fields", "ir-diff"} else None,
                            diff_count=0 if command == "ir-diff" else None,
                            fail_count=0,
                            pass_count=passed,
                            verdict="pass" if command == "verify" else None,
                            regression=False if command == "render-diff" else None,
                            status="OK" if command == "render-diff" else None,
                            clean=True if command != "info" else None,
                            signal_count=0 if command == "layout-anomaly" else None,
                            valid=True if command == "verify" else None,
                            finding_count=0,
                            overflow_count=0 if command == "layout-anomaly" else None,
                            overlap_count=0 if command == "layout-anomaly" else None,
                            empty_page_count=empty,
                        )


def iter_consistent_fail() -> Iterator[EnvelopeFields]:
    """All present fields lean fail. Magnitudes stay strictly positive."""
    fail_exits = (1, 2, 3, 4)
    for command in COMMANDS:
        for exit_code in fail_exits:
            for pages in range(1, 12):
                for mag in range(1, 6):
                    for node in NODES[:8]:
                        mismatch = exit_code == 4
                        yield EnvelopeFields(
                            command=command,
                            exit=exit_code,
                            identical=False if exit_code == 3 else None,
                            has_signal=True if command == "layout-anomaly" and exit_code == 3 else None,
                            reproduced=False if command == "replay" and exit_code == 3 else None,
                            page_count_a=pages,
                            page_count_b=pages + mag if mismatch or exit_code == 3 else None,
                            page_count_mismatch=True if mismatch else None,
                            struct_status=(
                                "STRUCT_MISMATCH"
                                if command == "render-diff" and exit_code == 1
                                else ("PAGE_MISMATCH" if mismatch else None)
                            ),
                            struct_node=node,
                            page_count_node=node if command == "render-diff" else f"doc/{pages}",
                            verify_identical=False if command == "fill-fields" and exit_code == 3 else None,
                            verify_diff_count=mag if command == "fill-fields" and exit_code == 3 else None,
                            diff_count=mag if command == "ir-diff" and exit_code == 3 else None,
                            fail_count=mag if command == "verify" and exit_code == 3 else None,
                            pass_count=0 if command == "verify" and exit_code == 3 else None,
                            verdict="fail" if command == "verify" and exit_code == 3 else None,
                            regression=True if command == "render-diff" and exit_code == 3 else None,
                            status="FAIL" if command == "render-diff" and exit_code == 3 else None,
                            clean=False if command != "info" and exit_code == 3 else None,
                            signal_count=mag if command == "layout-anomaly" and exit_code == 3 else None,
                            valid=False if command == "verify" and exit_code == 3 else None,
                            finding_count=mag if exit_code == 3 else None,
                            overflow_count=mag if command == "layout-anomaly" and exit_code == 3 else None,
                            overlap_count=mag if command == "layout-anomaly" and exit_code == 3 else None,
                            empty_page_count=mag,
                        )


def iter_all_fields() -> Iterator[EnvelopeFields]:
    yield from iter_abstain_fields()
    yield from iter_consistent_pass()
    yield from iter_consistent_fail()


def _accept(seen: set[tuple[Any, ...]], raw: EnvelopeFields, serial: int) -> EnvelopeFields | None:
    fields = _colorize(
        raw,
        page=1 + (serial % 47),
        node=NODES[serial % len(NODES)],
        mag=serial % 7,
        serial=serial,
    )
    key = fields.field_tuple()
    if key in seen:
        return None
    seen.add(key)
    return fields


def collect(target: int) -> list[AbstainCase]:
    seen: set[tuple[Any, ...]] = set()
    cases: list[AbstainCase] = []

    def push(raw: EnvelopeFields, serial: int) -> None:
        fields = _accept(seen, raw, serial)
        if fields is None:
            return
        cases.append(_make_case(len(cases) + 1, fields))

    serial = 0
    for raw in iter_consistent_pass():
        serial += 1
        push(raw, serial)
    for raw in iter_consistent_fail():
        serial += 1
        push(raw, serial)
    consistent = len(cases)
    if consistent < 1000:
        raise RuntimeError(f"consistent pass/fail too small: {consistent}")
    if not any(c.expected == VERDICT_PASS for c in cases):
        raise RuntimeError("no consistent pass rows")
    if not any(c.expected == VERDICT_FAIL for c in cases):
        raise RuntimeError("no consistent fail rows")

    for raw in iter_abstain_fields():
        if len(cases) >= target:
            break
        serial += 1
        fields = _accept(seen, raw, serial)
        if fields is None:
            continue
        decision = decide(fields)
        if decision.verdict != VERDICT_ABSTAIN:
            continue
        cases.append(_make_case(len(cases) + 1, fields))
    return cases


def tsv_line(case: AbstainCase) -> str:
    row = case.to_row()
    return "\t".join(row[col] for col in CASE_COLUMNS)


def write_shard(path: Path, cases: list[AbstainCase]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["\t".join(CASE_COLUMNS)]
    lines.extend(tsv_line(case) for case in cases)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def write_axis_table(path: Path, cases: list[AbstainCase]) -> None:
    """One full-field example per (expected, contradiction_id)."""
    seen: set[tuple[str, str]] = set()
    kept: list[AbstainCase] = []
    for case in cases:
        key = (case.expected, case.contradiction_id or case.expected)
        if key in seen:
            continue
        seen.add(key)
        kept.append(case)
    path.parent.mkdir(parents=True, exist_ok=True)
    write_shard(path, kept)


def generate(target: int, shard_rows: int, out_dir: Path) -> dict:
    cases = collect(target)
    if len(cases) < 100_000:
        raise RuntimeError(f"only {len(cases)} distinct tuples; need >= 100000")
    keys = [case.identity_key() for case in cases]
    if len(keys) != len(set(keys)):
        raise RuntimeError("generated corpus has duplicate field tuples")

    for verdict in (VERDICT_ABSTAIN, VERDICT_PASS, VERDICT_FAIL):
        if not any(case.expected == verdict for case in cases):
            raise RuntimeError(f"corpus missing {verdict} rows")

    shards: list[dict] = []
    out_dir.mkdir(parents=True, exist_ok=True)
    for stale in out_dir.glob("shard_*.tsv"):
        stale.unlink()
    for start in range(0, len(cases), shard_rows):
        chunk = cases[start : start + shard_rows]
        name = f"shard_{start // shard_rows:04d}.tsv"
        write_shard(out_dir / name, chunk)
        counts: dict[str, int] = {}
        for case in chunk:
            counts[case.expected] = counts.get(case.expected, 0) + 1
        shards.append(
            {
                "path": f"corpus/{name}",
                "rows": len(chunk),
                "first": chunk[0].case_id,
                "last": chunk[-1].case_id,
                "byVerdict": dict(sorted(counts.items())),
            }
        )

    by_verdict: dict[str, int] = {}
    by_rule: dict[str, int] = {}
    for case in cases:
        by_verdict[case.expected] = by_verdict.get(case.expected, 0) + 1
        rule = case.contradiction_id or case.expected
        by_rule[rule] = by_rule.get(rule, 0) + 1

    write_axis_table(HERE / "fixtures" / "axis_closed_set.tsv", cases)
    manifest = {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "kind": "abstainOnContradictionCorpus",
        "rowCount": len(cases),
        "shardRows": shard_rows,
        "columns": list(CASE_COLUMNS),
        "byVerdict": dict(sorted(by_verdict.items())),
        "byContradiction": dict(sorted(by_rule.items())),
        "shards": shards,
        "notes": [
            "Each row is a distinct envelope field tuple, not comment padding.",
            "expected is decide() of the field columns.",
            "Contradiction => abstain. Never invent pass/fail.",
            "Does not reimplement V-proto's classifier.",
        ],
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", type=int, default=DEFAULT_TARGET)
    parser.add_argument("--shard-rows", type=int, default=SHARD_ROWS)
    parser.add_argument("--out", type=Path, default=CORPUS_DIR)
    args = parser.parse_args(argv)
    manifest = generate(args.target, args.shard_rows, args.out)
    print(
        json.dumps(
            {
                "ok": True,
                "rows": manifest["rowCount"],
                "byVerdict": manifest["byVerdict"],
                "shards": len(manifest["shards"]),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
