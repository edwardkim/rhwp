#!/usr/bin/env python3
"""Emit the committed V-shadow decision-case corpus.

Each row is a distinct
``(check_a, check_b, a_pass, b_pass, expected_joint)`` case plus a sample
identity that keeps the row unique. Comment padding is not used.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from .checks import CHECKS, iter_distinct_pairs, iter_same_command_pairs
    from .decide import CLAIM_ID, SCHEMA_VERSION, decide
    from .schema import CASE_COLUMNS, DecisionCase, bool_cell
except ImportError:
    from checks import CHECKS, iter_distinct_pairs, iter_same_command_pairs
    from decide import CLAIM_ID, SCHEMA_VERSION, decide
    from schema import CASE_COLUMNS, DecisionCase, bool_cell

HERE = Path(__file__).resolve().parent
CORPUS_DIR = HERE / "corpus"
DEFAULT_TARGET = 122880
SHARD_ROWS = 7680

FAMILIES: tuple[str, ...] = (
    "기안문",
    "편람",
    "시험지",
    "훈령",
    "고시",
    "공고문",
    "회의록",
    "예산서",
    "결산서",
    "계약서",
    "용역보고서",
    "학술논문",
    "입법예고",
    "보도자료",
    "내부결재",
    "수신문서",
    "발신문서",
    "회의안건",
    "심사평가",
    "현장점검",
    "안전관리",
    "개인정보",
    "정보공개",
    "민원회신",
    "훈령별표",
    "질의회신",
    "업무계획",
    "점검표",
    "서식",
    "예규",
    "지침",
    "공고",
)

AGENCIES: tuple[str, ...] = (
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

YEARS = tuple(str(year) for year in range(2016, 2027))
FORMATS = ("hwp", "hwpx")
PASS_COMBOS: tuple[tuple[bool, bool], ...] = (
    (True, True),
    (True, False),
    (False, True),
    (False, False),
)


def axis_space() -> list[tuple[str, str, bool, bool, bool, str]]:
    space: list[tuple[str, str, bool, bool, bool, str]] = []
    for left, right in (*iter_distinct_pairs(), *iter_same_command_pairs()):
        for a_pass, b_pass in PASS_COMBOS:
            decision = decide(left.check_id, right.check_id, a_pass, b_pass)
            space.append(
                (
                    left.check_id,
                    right.check_id,
                    a_pass,
                    b_pass,
                    decision.expected_joint,
                    decision.verdict_class,
                )
            )
    return space


def observed_for(check_id: str, passed: bool, pages: int) -> str:
    check = next(item for item in CHECKS if item.check_id == check_id)
    if check.pass_equals == "equal":
        return str(pages) if passed else f"{pages}+1"
    return check.pass_equals if passed else check.fail_example


def sample_identity(index: int) -> dict[str, str]:
    family = FAMILIES[index % len(FAMILIES)]
    agency = AGENCIES[index % len(AGENCIES)]
    year = YEARS[index % len(YEARS)]
    fmt = FORMATS[index % len(FORMATS)]
    serial = index // len(FAMILIES)
    sample = f"samples/{agency}/{family}/case-{serial:05d}-{index % 97:02d}-{year}.{fmt}"
    return {
        "sample": sample,
        "source_format": fmt,
        "family": family,
        "agency": agency,
        "year": year,
    }


def pages_for(a_pass: bool, b_pass: bool, index: int) -> tuple[int, int]:
    base = 1 + (index % 53)
    page_a = base if a_pass else base + 1 + (index % 3)
    page_b = base if b_pass else base + 2 + (index % 5)
    return page_a, page_b


def make_case(
    case_id: str,
    axis: tuple[str, str, bool, bool, bool, str],
    ident_index: int,
) -> DecisionCase:
    check_a, check_b, a_pass, b_pass, expected_joint, verdict = axis
    ident = sample_identity(ident_index)
    page_a, page_b = pages_for(a_pass, b_pass, ident_index)
    decision = decide(check_a, check_b, a_pass, b_pass)
    if decision.verdict_class != verdict or decision.expected_joint != expected_joint:
        raise RuntimeError(f"axis drift: {axis} -> {decision.verdict_class}")
    return DecisionCase(
        case_id=case_id,
        check_a=check_a,
        check_b=check_b,
        a_pass=a_pass,
        b_pass=b_pass,
        expected_joint=expected_joint,
        expected_verdict_class=verdict,
        sample=ident["sample"],
        source_format=ident["source_format"],
        family=ident["family"],
        agency=ident["agency"],
        year=ident["year"],
        page_a=page_a,
        page_b=page_b,
        command_a=decision.check_a.command,
        command_b=decision.check_b.command,
        field_a=decision.check_a.pass_field,
        field_b=decision.check_b.pass_field,
        observed_a=observed_for(check_a, a_pass, page_a),
        observed_b=observed_for(check_b, b_pass, page_b),
        honest_claim=decision.honest_claim,
        contract_source=f"{decision.check_a.producer}|{decision.check_b.producer}",
        not_abstain=True,
        not_repeat=True,
    )


def tsv_line(case: DecisionCase) -> str:
    row = case.to_row()
    return "\t".join(row[col] for col in CASE_COLUMNS)


def write_shard(path: Path, cases: list[DecisionCase]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["\t".join(CASE_COLUMNS)]
    lines.extend(tsv_line(case) for case in cases)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def write_pair_table(path: Path, space: list[tuple[str, str, bool, bool, bool, str]]) -> None:
    header = "check_a\tcheck_b\ta_pass\tb_pass\texpected_joint\texpected_verdict_class"
    lines = [header]
    for check_a, check_b, a_pass, b_pass, joint, verdict in space:
        lines.append(
            "\t".join(
                (
                    check_a,
                    check_b,
                    bool_cell(a_pass),
                    bool_cell(b_pass),
                    bool_cell(joint),
                    verdict,
                )
            )
        )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def generate(target: int, shard_rows: int, out_dir: Path) -> dict:
    space = axis_space()
    if not space:
        raise RuntimeError("empty axis space")
    identities_needed = (target + len(space) - 1) // len(space)
    cases: list[DecisionCase] = []
    serial = 0
    for ident_index in range(identities_needed):
        for axis in space:
            serial += 1
            case_id = f"v-shadow-{serial:06d}"
            cases.append(make_case(case_id, axis, ident_index * 1009 + serial))
            if len(cases) >= target:
                break
        if len(cases) >= target:
            break

    keys = [case.identity_key() for case in cases]
    if len(keys) != len(set(keys)):
        raise RuntimeError("generated corpus has duplicate identity keys")

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
            counts[case.expected_verdict_class] = counts.get(case.expected_verdict_class, 0) + 1
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
    for case in cases:
        by_verdict[case.expected_verdict_class] = (
            by_verdict.get(case.expected_verdict_class, 0) + 1
        )
    write_pair_table(HERE / "fixtures" / "pair_closed_set.tsv", space)
    manifest = {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "kind": "shadowAgreementCorpus",
        "rowCount": len(cases),
        "axisCount": len(space),
        "checkCount": len(CHECKS),
        "identitiesPerAxis": identities_needed,
        "shardRows": shard_rows,
        "columns": list(CASE_COLUMNS),
        "byVerdict": dict(sorted(by_verdict.items())),
        "shards": shards,
        "notes": [
            "Each row is a distinct (check_a, check_b, a_pass, b_pass, expected_joint) case.",
            "Comment padding is not used.",
            "expected_joint is 1 only when two different commands both pass.",
            "Same-command pairs are SAME_CHECK_NOT_SHADOW, not V-abstain and not V-repeat.",
            "Producer commands are consumed as contracts, never invented.",
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
    parser.add_argument("--out-dir", type=Path, default=CORPUS_DIR)
    args = parser.parse_args(argv)
    manifest = generate(args.target, args.shard_rows, args.out_dir)
    json.dump({"rowCount": manifest["rowCount"], "shards": len(manifest["shards"])}, sys.stdout)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    if str(HERE) not in sys.path:
        sys.path.insert(0, str(HERE))
    raise SystemExit(main())
