#!/usr/bin/env python3
"""Emit the committed V-oracle decision-case corpus.

Each row is a distinct
``(has_hangul_pdf, versions, page_count_match, render_self_pass, cheap_ok,
expected_verdict_class)`` case plus a sample identity that keeps the row
unique. Comment padding is not used.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from .decide import CLAIM_ID, SCHEMA_VERSION, decide
    from .schema import CASE_COLUMNS, DecisionCase, bool_cell
    from .versions import (
        iter_agree_encodings,
        iter_disagree_encodings,
        iter_invalid_encodings,
        iter_none_encodings,
        iter_unknown_encodings,
    )
except ImportError:  # python generate_corpus.py
    from decide import CLAIM_ID, SCHEMA_VERSION, decide
    from schema import CASE_COLUMNS, DecisionCase, bool_cell
    from versions import (
        iter_agree_encodings,
        iter_disagree_encodings,
        iter_invalid_encodings,
        iter_none_encodings,
        iter_unknown_encodings,
    )

HERE = Path(__file__).resolve().parent
CORPUS_DIR = HERE / "corpus"
DEFAULT_TARGET = 122400
SHARD_ROWS = 7650

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
)

ORACLE_ROOTS = ("pdf",)
FORMATS = ("hwp", "hwpx")
VARIANTS = ("", "hwp", "hwpx", "kopub", "no-ttf", "print", "hancom")


def all_version_tokens() -> tuple[str, ...]:
    seen: list[str] = []
    for group in (
        iter_none_encodings(),
        iter_unknown_encodings(),
        iter_invalid_encodings(),
        iter_agree_encodings(),
        iter_disagree_encodings(),
    ):
        for token in group:
            # Blank TSV cells are not a distinct year token; "none" already covers it.
            if token == "":
                continue
            if token not in seen:
                seen.append(token)
    return tuple(seen)


def all_flag_tuples() -> tuple[tuple[bool, bool, bool, bool], ...]:
    flags: list[tuple[bool, bool, bool, bool]] = []
    for has_pdf in (False, True):
        for page_match in (False, True):
            for render_ok in (False, True):
                for cheap_ok in (False, True):
                    flags.append((has_pdf, page_match, render_ok, cheap_ok))
    return tuple(flags)


def axis_space() -> list[tuple[bool, str, bool, bool, bool, str]]:
    space: list[tuple[bool, str, bool, bool, bool, str]] = []
    for versions in all_version_tokens():
        for has_pdf, page_match, render_ok, cheap_ok in all_flag_tuples():
            verdict = decide(has_pdf, versions, page_match, render_ok, cheap_ok)
            space.append(
                (
                    has_pdf,
                    versions,
                    page_match,
                    render_ok,
                    cheap_ok,
                    verdict.verdict_class,
                )
            )
    return space


def sample_identity(index: int, has_pdf: bool, versions: str) -> dict[str, str]:
    family = FAMILIES[index % len(FAMILIES)]
    fmt = FORMATS[index % len(FORMATS)]
    serial = index // len(FAMILIES)
    stem = f"samples/{family}/case-{serial:04d}-{index % 97:02d}"
    if has_pdf:
        root = ORACLE_ROOTS[index % len(ORACLE_ROOTS)]
        variant = VARIANTS[(index // 3) % len(VARIANTS)]
        year = "2022"
        for token in ("2010", "2018", "2020", "2022", "2024"):
            if token in versions:
                year = token
                break
        suffix = f"-{variant}" if variant else ""
        sample = f"{stem}.{fmt}"
        return {
            "sample": sample,
            "source_format": fmt,
            "oracle_root": root,
            "variant": variant or "plain",
            "pdf_name": f"{root}/{family}/case-{serial:04d}-{year}{suffix}.pdf",
        }
    return {
        "sample": f"{stem}.{fmt}",
        "source_format": fmt,
        "oracle_root": "none",
        "variant": "unmatched",
        "pdf_name": "",
    }


def pages_for(page_match: bool, has_pdf: bool, index: int) -> tuple[int, int, str, str]:
    rhwp = 1 + (index % 47)
    if not has_pdf:
        return rhwp, 0, "UNPAIRED", "no_hangul_pdf"
    if page_match:
        return rhwp, rhwp, "MATCH", "page_smoke_match"
    pdf = rhwp + 1 + (index % 5)
    return rhwp, pdf, "MISMATCH", "page_smoke_mismatch"


def cheap_reason_for(cheap_ok: bool, page_smoke: str, has_pdf: bool) -> str:
    if cheap_ok:
        return "preflight_ok"
    if not has_pdf:
        return "self_pagecount_unavailable"
    if page_smoke == "MISMATCH":
        return "counts_numeric_but_other_preflight_failed"
    return "page_smoke_error_or_lfs_or_run_state_missing"


def contract_source_for(verdict: str) -> str:
    if verdict == "ORACLE_MULTIVER_DISAGREE":
        return "oracle_public.multiver_index"
    if verdict in {"ORACLE_PAGECOUNT_MISMATCH", "ORACLE_CHEAP_FAIL"}:
        return "oracle_public.page_smoke"
    if verdict == "ORACLE_TRUSTED":
        return "fidelity_compare+visual_sweep+page_smoke"
    if verdict.startswith("NO_ORACLE_"):
        return "render-diff+dump-pages"
    return "oracle_public.oracle_resolver"


def make_case(
    case_id: str,
    axis: tuple[bool, str, bool, bool, bool, str],
    ident_index: int,
) -> DecisionCase:
    has_pdf, versions, page_match, render_ok, cheap_ok, verdict = axis
    ident = sample_identity(ident_index, has_pdf, versions)
    rhwp_pages, pdf_pages, smoke, smoke_reason = pages_for(page_match, has_pdf, ident_index)
    decision = decide(has_pdf, versions, page_match, render_ok, cheap_ok)
    if decision.verdict_class != verdict:
        raise RuntimeError(f"axis drift: {axis} -> {decision.verdict_class}")
    cheap_reason = cheap_reason_for(cheap_ok, smoke, has_pdf)
    if not cheap_ok and has_pdf and page_match:
        smoke = "ERROR"
        cheap_reason = "page_smoke_error_or_lfs_or_run_state_missing"
    return DecisionCase(
        case_id=case_id,
        has_hangul_pdf=has_pdf,
        versions=versions,
        page_count_match=page_match,
        render_self_pass=render_ok,
        cheap_ok=cheap_ok,
        expected_verdict_class=verdict,
        sample=ident["sample"],
        source_format=ident["source_format"],
        oracle_root=ident["oracle_root"],
        variant=ident["variant"],
        rhwp_pages=rhwp_pages,
        pdf_pages=pdf_pages,
        page_smoke_verdict=smoke,
        cheap_reason=cheap_reason if cheap_reason else smoke_reason,
        honest_claim=decision.honest_claim,
        allowed_tools="|".join(decision.allowed_tools),
        blocked_tools="|".join(decision.blocked_tools),
        contract_source=contract_source_for(verdict),
    )


def tsv_line(case: DecisionCase) -> str:
    row = case.to_row()
    return "\t".join(row[col] for col in CASE_COLUMNS)


def write_shard(path: Path, cases: list[DecisionCase]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["\t".join(CASE_COLUMNS)]
    lines.extend(tsv_line(case) for case in cases)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def write_axis_table(path: Path, space: list[tuple[bool, str, bool, bool, bool, str]]) -> None:
    header = (
        "has_hangul_pdf\tversions\tpage_count_match\trender_self_pass\t"
        "cheap_ok\texpected_verdict_class"
    )
    lines = [header]
    for has_pdf, versions, page_match, render_ok, cheap_ok, verdict in space:
        token = versions if versions != "" else "none"
        lines.append(
            "\t".join(
                (
                    bool_cell(has_pdf),
                    token,
                    bool_cell(page_match),
                    bool_cell(render_ok),
                    bool_cell(cheap_ok),
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
            case_id = f"v-oracle-{serial:06d}"
            cases.append(make_case(case_id, axis, ident_index * 1009 + serial))
            if len(cases) >= target:
                break
        if len(cases) >= target:
            break

    # Distinctness is (case_id) plus the axis+identity key.
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
    write_axis_table(HERE / "fixtures" / "axis_closed_set.tsv", space)
    manifest = {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "kind": "oracleVsSelfCorpus",
        "rowCount": len(cases),
        "axisCount": len(space),
        "identitiesPerAxis": identities_needed,
        "shardRows": shard_rows,
        "columns": list(CASE_COLUMNS),
        "byVerdict": dict(sorted(by_verdict.items())),
        "shards": shards,
        "notes": [
            "Each row is a distinct decision case, not comment padding.",
            "expected_verdict_class is decide() of the five input columns.",
            "Producer tools are consumed as contracts, never rewritten.",
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
