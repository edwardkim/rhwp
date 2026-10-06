#!/usr/bin/env python3
"""samples/ ↔ 한컴 기준 PDF 오라클 커버리지 표 (M01-3).

`samples/` 의 `.hwp`/`.hwpx` 를 재귀 수집한 뒤, 같은 상대 경로의
`pdf/{stem}-{year}.pdf` (및 `-hwp-2020` 같은 허용 변형) 를 맞춘다.
한글 버전은 2018 / 2020 / 2022 / 2024 만 센다.

`oracle_resolver.py` 가 devel 에 없어도 동작하도록 매칭을 이 파일 안에 둔다.
짝 없는 개수는 실측한다. 이슈의 참고값(~276)을 코드에 박지 않는다.

scripts/visual_sweep.py 와 tools/oracle_public/issue_draft.py 는
읽거나 수정하지 않는다.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import unicodedata
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence

SCHEMA_VERSION = "1.0"
CLAIM_ID = "M01-3"
GENERATOR = "tools/oracle_public/coverage_report.py"
MATCHING_RULE = "stem-{year}.pdf"
SAMPLE_EXTS = {".hwp", ".hwpx"}
DEFAULT_ORACLE_ROOTS = ("pdf",)
HANCOM_YEARS = ("2018", "2020", "2022", "2024")

# `{stem}` 뒤에 붙는 오라클 접미사. 연도가 반드시 들어가고, 포맷 태그·kopub·
# 스냅샷 날짜·쪽 범위는 선택이다. `-hwpx-2022` 를 stem 끝의 `hwpx` 와 혼동하지
# 않도록 샘플 stem 에서 바깥으로만 잘라 검사한다.
_ORACLE_SUFFIX_RE = re.compile(
    r"^(?:"
    r"(?:-(?P<fmt>hwp|hwpx))?"
    r"(?:-kopub)?"
    r"-(?P<year>2018|2020|2022|2024)"
    r"(?:-kopub)?"
    r"(?:-no-ttf)?"
    r"(?:-print)?"
    r"(?:-\d{8})?"
    r"(?:-p\d+-p\d+)?"
    r")$",
    re.IGNORECASE,
)
_ALT_SUFFIX_RE = re.compile(
    r"^-(?:hancom-?|hwp)(?P<year>2018|2020|2022|2024)(?:-\d{8})?$",
    re.IGNORECASE,
)
_YEAR_TOKEN_RE = re.compile(r"(?<!\d)(2018|2020|2022|2024)(?!\d)")


class UsageError(Exception):
    """CLI 사용법·경로 오류."""


@dataclass(frozen=True)
class OracleHit:
    pdf: str
    hancom_version: str
    variant: str
    format_tag: str | None
    oracle_root: str


@dataclass(frozen=True)
class SampleDoc:
    sample: str
    rel_parent: str
    stem: str
    source_format: str


def nfc(value: str) -> str:
    return unicodedata.normalize("NFC", value)


def posix_rel(path: Path, root: Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


def discover_repo_root(start: Path | None = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    for candidate in (cur, *cur.parents):
        if (candidate / "samples").is_dir() and (
            (candidate / ".git").exists() or (candidate / "Cargo.toml").is_file()
        ):
            return candidate
    raise UsageError("저장소 루트를 찾지 못했습니다 (--repo-root 를 지정하세요)")


def walk_samples(samples_dir: Path, repo_root: Path) -> list[SampleDoc]:
    if not samples_dir.is_dir():
        raise UsageError(f"samples 디렉터리가 없습니다: {samples_dir}")
    found: list[SampleDoc] = []
    for path in samples_dir.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix.lower() not in SAMPLE_EXTS:
            continue
        rel = Path(posix_rel(path, repo_root))
        parent = rel.parent.as_posix()
        if parent == "samples":
            rel_parent = ""
        else:
            rel_parent = Path(parent).relative_to("samples").as_posix()
        found.append(
            SampleDoc(
                sample=rel.as_posix(),
                rel_parent="" if rel_parent == "." else rel_parent,
                stem=nfc(path.stem),
                source_format=path.suffix.lower().lstrip("."),
            )
        )
    found.sort(key=lambda item: (nfc(item.sample).lower(), item.sample))
    return found


def index_oracle_pdfs(
    repo_root: Path, roots: Sequence[str]
) -> dict[tuple[str, str, str], list[str]]:
    """(root, rel_parent, filename_nfc_lower) -> posix paths."""
    index: dict[tuple[str, str, str], list[str]] = {}
    for root_name in roots:
        root_dir = repo_root / root_name
        if not root_dir.is_dir():
            continue
        for path in root_dir.rglob("*.pdf"):
            if not path.is_file():
                continue
            rel = Path(posix_rel(path, root_dir))
            parent = rel.parent.as_posix()
            if parent == ".":
                parent = ""
            key = (root_name, parent, nfc(path.name).lower())
            index.setdefault(key, []).append(posix_rel(path, repo_root))
    for paths in index.values():
        paths.sort(key=lambda item: (nfc(item).lower(), item))
    return index


def parse_oracle_suffix(stem: str, filename: str) -> dict[str, str | None] | None:
    """파일명이 `{stem}{suffix}.pdf` 오라클 규칙이면 연도·변형을 돌려준다."""
    if not filename.lower().endswith(".pdf"):
        return None
    base = nfc(filename[:-4])
    stem_n = nfc(stem)
    if not base.lower().startswith(stem_n.lower()):
        return None
    rest = base[len(stem_n) :]
    if rest == "":
        years = _YEAR_TOKEN_RE.findall(stem_n)
        if not years:
            return None
        return {"year": years[-1], "variant": "exact", "fmt": None}
    match = _ORACLE_SUFFIX_RE.match(rest) or _ALT_SUFFIX_RE.match(rest)
    if not match:
        return None
    groups = match.groupdict()
    year = groups.get("year")
    if year not in HANCOM_YEARS:
        return None
    fmt = (groups.get("fmt") or "").lower() or None
    return {"year": year, "variant": rest.lstrip("-"), "fmt": fmt}


def format_allows(source_format: str, format_tag: str | None) -> bool:
    if not format_tag:
        return True
    return format_tag.lower() == source_format.lower()


def match_sample(
    sample: SampleDoc,
    index: dict[tuple[str, str, str], list[str]],
    roots: Sequence[str],
) -> list[OracleHit]:
    hits: list[OracleHit] = []
    seen: set[str] = set()
    for (root_name, parent, name), paths in index.items():
        if root_name not in roots or parent != sample.rel_parent:
            continue
        info = parse_oracle_suffix(sample.stem, name)
        if info is None:
            continue
        if not format_allows(sample.source_format, info["fmt"]):
            continue
        year = info["year"]
        if year is None:
            continue
        for pdf_path in paths:
            if pdf_path in seen:
                continue
            seen.add(pdf_path)
            hits.append(
                OracleHit(
                    pdf=pdf_path,
                    hancom_version=year,
                    variant=info["variant"] or year,
                    format_tag=info["fmt"],
                    oracle_root=root_name,
                )
            )
    hits.sort(
        key=lambda hit: (
            HANCOM_YEARS[::-1].index(hit.hancom_version)
            if hit.hancom_version in HANCOM_YEARS
            else 99,
            DEFAULT_ORACLE_ROOTS.index(hit.oracle_root)
            if hit.oracle_root in DEFAULT_ORACLE_ROOTS
            else 99,
            nfc(hit.variant).lower(),
            nfc(hit.pdf).lower(),
            hit.pdf,
        )
    )
    return hits


def pair_record(sample: SampleDoc, hit: OracleHit) -> dict[str, Any]:
    return {
        "id": f"{sample.sample}::{hit.pdf}",
        "sample": sample.sample,
        "pdf": hit.pdf,
        "stem": sample.stem,
        "hancomVersion": hit.hancom_version,
        "variant": hit.variant,
        "sourceFormat": sample.source_format,
        "oracleRoot": hit.oracle_root,
    }


def unmatched_record(sample: SampleDoc) -> dict[str, Any]:
    return {
        "sample": sample.sample,
        "stem": sample.stem,
        "sourceFormat": sample.source_format,
        "reason": "no_oracle_pdf",
    }


def _format_bucket() -> dict[str, int]:
    return {"sampleCount": 0, "matchedSampleCount": 0, "unmatchedCount": 0}


def build_report(
    repo_root: Path,
    samples_dir: Path | None = None,
    roots: Sequence[str] = DEFAULT_ORACLE_ROOTS,
) -> dict[str, Any]:
    samples_dir = samples_dir or (repo_root / "samples")
    present_roots = [name for name in roots if (repo_root / name).is_dir()]
    samples = walk_samples(samples_dir, repo_root)
    index = index_oracle_pdfs(repo_root, present_roots)
    pairs: list[dict[str, Any]] = []
    unmatched: list[dict[str, Any]] = []
    version_pairs: Counter[str] = Counter()
    version_samples: dict[str, set[str]] = {year: set() for year in HANCOM_YEARS}
    by_format = {fmt: _format_bucket() for fmt in ("hwp", "hwpx")}

    for sample in samples:
        bucket = by_format[sample.source_format]
        bucket["sampleCount"] += 1
        hits = match_sample(sample, index, present_roots)
        if hits:
            bucket["matchedSampleCount"] += 1
            for hit in hits:
                rec = pair_record(sample, hit)
                pairs.append(rec)
                version_pairs[hit.hancom_version] += 1
                version_samples[hit.hancom_version].add(sample.sample)
        else:
            bucket["unmatchedCount"] += 1
            unmatched.append(unmatched_record(sample))

    sample_count = len(samples)
    matched_count = sample_count - len(unmatched)
    coverage_ratio = (matched_count / sample_count) if sample_count else 0.0
    return {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "matchingRule": MATCHING_RULE,
        "hancomYears": list(HANCOM_YEARS),
        "sampleCount": sample_count,
        "matchedSampleCount": matched_count,
        "unmatchedCount": len(unmatched),
        "pairCount": len(pairs),
        "coverageRatio": coverage_ratio,
        "oracleRoots": present_roots,
        "bySourceFormat": by_format,
        "byHancomVersion": {
            year: {
                "pairCount": version_pairs.get(year, 0),
                "sampleCount": len(version_samples[year]),
            }
            for year in HANCOM_YEARS
        },
        "pairs": pairs,
        "unmatched": unmatched,
    }


def validate_report(data: Any) -> list[str]:
    """핵심 계약. 외부 jsonschema 없이 필수 키·타입·합만 검사한다."""
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["$: object 가 아닙니다"]
    required = (
        "schemaVersion",
        "claim",
        "sampleCount",
        "matchedSampleCount",
        "unmatchedCount",
        "pairCount",
        "byHancomVersion",
        "pairs",
        "unmatched",
    )
    for key in required:
        if key not in data:
            errors.append(f"$: required 누락 ({key})")
    if data.get("schemaVersion") != SCHEMA_VERSION:
        errors.append("$.schemaVersion: '1.0' 이어야 합니다")
    if data.get("claim") != CLAIM_ID:
        errors.append("$.claim: 'M01-3' 이어야 합니다")
    for key in (
        "sampleCount",
        "matchedSampleCount",
        "unmatchedCount",
        "pairCount",
    ):
        if key in data and not isinstance(data[key], int):
            errors.append(f"$.{key}: integer 가 아닙니다")
        if key in data and isinstance(data[key], int) and data[key] < 0:
            errors.append(f"$.{key}: 0 이상이어야 합니다")

    pairs = data.get("pairs")
    if not isinstance(pairs, list):
        errors.append("$.pairs: array 가 아닙니다")
        pairs = []
    elif data.get("pairCount") != len(pairs):
        errors.append("$.pairCount: pairs 길이와 다릅니다")

    unmatched = data.get("unmatched")
    if not isinstance(unmatched, list):
        errors.append("$.unmatched: array 가 아닙니다")
        unmatched = []
    elif data.get("unmatchedCount") != len(unmatched):
        errors.append("$.unmatchedCount: unmatched 길이와 다릅니다")

    matched_samples = {item.get("sample") for item in pairs if isinstance(item, dict)}
    if data.get("matchedSampleCount") != len(matched_samples):
        errors.append("$.matchedSampleCount: 서로 다른 sample 수와 다릅니다")
    if isinstance(data.get("sampleCount"), int) and isinstance(
        data.get("unmatchedCount"), int
    ):
        expected_total = len(matched_samples) + len(unmatched)
        if data["sampleCount"] != expected_total:
            errors.append("$.sampleCount: matched+unmatched 와 다릅니다")
        if (
            isinstance(data.get("matchedSampleCount"), int)
            and data["matchedSampleCount"] + data["unmatchedCount"]
            != data["sampleCount"]
        ):
            errors.append("$.matchedSampleCount+unmatchedCount: sampleCount 와 다릅니다")

    pair_required = (
        "sample",
        "pdf",
        "stem",
        "hancomVersion",
        "sourceFormat",
        "oracleRoot",
    )
    seen_ids: set[str] = set()
    for i, item in enumerate(pairs):
        loc = f"$.pairs[{i}]"
        if not isinstance(item, dict):
            errors.append(f"{loc}: object 가 아닙니다")
            continue
        for key in pair_required:
            if key not in item:
                errors.append(f"{loc}: required 누락 ({key})")
        year = item.get("hancomVersion")
        if year not in HANCOM_YEARS:
            errors.append(f"{loc}.hancomVersion: {HANCOM_YEARS} 중 하나여야 합니다")
        if item.get("sourceFormat") not in {"hwp", "hwpx"}:
            errors.append(f"{loc}.sourceFormat: hwp|hwpx 여야 합니다")
        pair_id = item.get("id")
        if isinstance(pair_id, str):
            if pair_id in seen_ids:
                errors.append(f"{loc}.id: 중복입니다")
            seen_ids.add(pair_id)

    for i, item in enumerate(unmatched):
        loc = f"$.unmatched[{i}]"
        if not isinstance(item, dict):
            errors.append(f"{loc}: object 가 아닙니다")
            continue
        if "sample" not in item:
            errors.append(f"{loc}: required 누락 (sample)")

    by_ver = data.get("byHancomVersion")
    if not isinstance(by_ver, dict):
        errors.append("$.byHancomVersion: object 가 아닙니다")
    else:
        for year in HANCOM_YEARS:
            bucket = by_ver.get(year)
            if not isinstance(bucket, dict):
                errors.append(f"$.byHancomVersion.{year}: object 가 아닙니다")
                continue
            expected_pairs = sum(
                1 for item in pairs if isinstance(item, dict) and item.get("hancomVersion") == year
            )
            expected_samples = len(
                {
                    item.get("sample")
                    for item in pairs
                    if isinstance(item, dict) and item.get("hancomVersion") == year
                }
            )
            if bucket.get("pairCount") != expected_pairs:
                errors.append(f"$.byHancomVersion.{year}.pairCount: pairs 집계와 다릅니다")
            if bucket.get("sampleCount") != expected_samples:
                errors.append(f"$.byHancomVersion.{year}.sampleCount: pairs 집계와 다릅니다")
    return errors


def _md_cell(value: str) -> str:
    return nfc(value).replace("|", "\\|").replace("\n", " ")


def _pct(part: int, whole: int) -> str:
    if whole <= 0:
        return "—"
    return f"{(100.0 * part / whole):.1f}%"


def render_markdown(report: dict[str, Any]) -> str:
    sample_count = int(report["sampleCount"])
    matched = int(report["matchedSampleCount"])
    unmatched_n = int(report["unmatchedCount"])
    pair_count = int(report["pairCount"])
    roots = ", ".join(f"`{name}/`" for name in report.get("oracleRoots") or []) or "(없음)"
    years = report.get("hancomYears") or list(HANCOM_YEARS)
    by_fmt = report.get("bySourceFormat") or {}
    by_ver = report.get("byHancomVersion") or {}
    unmatched = report.get("unmatched") or []

    lines = [
        "# M01-3 오라클 커버리지",
        "",
        "`samples/` 의 `.hwp`/`.hwpx` 와 `{stem}-{year}.pdf` (2018/2020/2022/2024) 짝.",
        "같은 상대 하위 경로를 단일 오라클 루트 `pdf/` 에서 찾는다.",
        "`oracle_resolver.py` 가 없어도 이 도구가 같은 규칙으로 직접 맞춘다.",
        "짝 없는 개수는 아래 표의 실측값이다.",
        "",
        f"- 클레임: `{report.get('claim', CLAIM_ID)}`",
        f"- 생성기: `{report.get('generator', GENERATOR)}`",
        f"- 매칭: `{report.get('matchingRule', MATCHING_RULE)}`",
        f"- 오라클 루트: {roots}",
        "",
        "## 요약",
        "",
        "| 항목 | 수 |",
        "| --- | ---: |",
        f"| 샘플 (`.hwp`/`.hwpx`) | {sample_count} |",
        f"| 짝 있는 샘플 | {matched} |",
        f"| 짝 없는 샘플 | {unmatched_n} |",
        f"| 오라클 링크 (샘플×PDF) | {pair_count} |",
        f"| 커버리지 (짝 있는 샘플 / 전체) | {_pct(matched, sample_count)} |",
        "",
        "### 형식별",
        "",
        "| 형식 | 샘플 | 짝 있음 | 짝 없음 | 커버리지 |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for fmt in ("hwp", "hwpx"):
        bucket = by_fmt.get(fmt) or {}
        total = int(bucket.get("sampleCount") or 0)
        ok = int(bucket.get("matchedSampleCount") or 0)
        miss = int(bucket.get("unmatchedCount") or 0)
        lines.append(f"| `{fmt}` | {total} | {ok} | {miss} | {_pct(ok, total)} |")

    lines.extend(
        [
            "",
            "## 한글 버전별 (2018 / 2020 / 2022 / 2024)",
            "",
            "한 샘플이 여러 버전 PDF 를 가지면 각 버전에 모두 센다.",
            "분모는 전체 샘플 수다. 2010·2014 접미 PDF 는 이 표에 넣지 않는다.",
            "",
            "| 한글 버전 | 링크 수 | 해당 버전이 있는 샘플 | 샘플 대비 |",
            "| --- | ---: | ---: | ---: |",
        ]
    )
    for year in years:
        bucket = by_ver.get(year) or {}
        links = int(bucket.get("pairCount") or 0)
        have = int(bucket.get("sampleCount") or 0)
        lines.append(f"| {year} | {links} | {have} | {_pct(have, sample_count)} |")

    lines.extend(
        [
            "",
            f"## 짝 없는 샘플 ({unmatched_n}건)",
            "",
        ]
    )
    if unmatched_n == 0:
        lines.append("짝 없는 샘플이 없다.")
        lines.append("")
    else:
        lines.extend(
            [
                "| # | 경로 | 형식 | 이유 |",
                "| ---: | --- | --- | --- |",
            ]
        )
        for i, item in enumerate(unmatched, start=1):
            path = _md_cell(str(item.get("sample") or ""))
            fmt = _md_cell(str(item.get("sourceFormat") or ""))
            reason = _md_cell(str(item.get("reason") or "no_oracle_pdf"))
            lines.append(f"| {i} | `{path}` | {fmt} | {reason} |")
        lines.append("")

    text = "\n".join(lines)
    if not text.endswith("\n"):
        text += "\n"
    return text


def emit_text(text: str, output: Path | None) -> None:
    if not text.endswith("\n"):
        text += "\n"
    if output is None or str(output) == "-":
        sys.stdout.write(text)
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text, encoding="utf-8", newline="\n")


def emit_json(data: dict[str, Any], output: Path | None, pretty: bool) -> None:
    text = json.dumps(data, ensure_ascii=False, indent=2 if pretty else None)
    emit_text(text, output)


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="samples/ ↔ 한컴 기준 PDF 오라클 커버리지 표 (짝 없는 샘플·버전별)."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=None,
        help="저장소 루트. 생략 시 cwd 에서 samples/ 와 .git|Cargo.toml 을 찾는다.",
    )
    parser.add_argument(
        "--samples-dir",
        type=Path,
        default=None,
        help="samples 디렉터리. 기본은 <repo-root>/samples",
    )
    parser.add_argument(
        "--roots",
        default=",".join(DEFAULT_ORACLE_ROOTS),
        help="오라클 PDF 루트(쉼표 구분). 기본 pdf",
    )
    parser.add_argument(
        "--json-out",
        type=Path,
        default=None,
        help="커버리지 JSON 경로. 생략하고 --md-out 도 없으면 stdout",
    )
    parser.add_argument(
        "--md-out",
        type=Path,
        default=None,
        help="커버리지 Markdown 표 경로",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="들여쓰기 JSON",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="방출 전 핵심 계약을 검사한다",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        repo_root = args.repo_root.resolve() if args.repo_root else discover_repo_root()
        roots = tuple(item.strip() for item in args.roots.split(",") if item.strip())
        if not roots:
            raise UsageError("--roots 가 비어 있습니다")
        report = build_report(repo_root, args.samples_dir, roots)
        if args.validate:
            errors = validate_report(report)
            if errors:
                sys.stderr.write("커버리지 표 계약 위반:\n")
                for err in errors:
                    sys.stderr.write(f"  {err}\n")
                return 1
        wrote = False
        if args.json_out is not None:
            emit_json(report, args.json_out, pretty=args.pretty or True)
            wrote = True
        if args.md_out is not None:
            emit_text(render_markdown(report), args.md_out)
            wrote = True
        if not wrote:
            emit_json(report, None, pretty=args.pretty)
    except UsageError as exc:
        sys.stderr.write(f"사용법 오류: {exc}\n")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
