#!/usr/bin/env python3
"""samples/ 의 HWP·HWPX 와 한컴 기준 PDF 를 짝 짓는 오라클 공개 판정 기준 생성기.

`samples/` 를 재귀 순회해 `.hwp`/`.hwpx` 를 모은 뒤, 같은 상대 경로의
`pdf/{stem}-{hancomVersion}.pdf` (및 `-hwp-2020` 같은 변형) 를 매칭한다.

한 샘플이 여러 한컴 버전 PDF 를 가지면 링크를 모두 남긴다. 목표 약 269쌍은
참고값이며, 실제 개수를 맞추려고 자르거나 부풀리지 않는다.

scripts/visual_sweep.py 는 읽거나 수정하지 않는다.
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
GENERATOR = "tools/oracle_public/oracle_resolver.py"
TARGET_PAIR_COUNT = 269
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


def build_manifest(
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
    for sample in samples:
        hits = match_sample(sample, index, present_roots)
        if hits:
            for hit in hits:
                pairs.append(pair_record(sample, hit))
        else:
            unmatched.append(unmatched_record(sample))
    versions = Counter(item["hancomVersion"] for item in pairs)
    return {
        "schemaVersion": SCHEMA_VERSION,
        "generator": GENERATOR,
        "targetPairCount": TARGET_PAIR_COUNT,
        "pairCount": len(pairs),
        "matchedSampleCount": len({item["sample"] for item in pairs}),
        "unmatchedCount": len(unmatched),
        "oracleLinkCount": len(pairs),
        "oracleRoots": present_roots,
        "byHancomVersion": {year: versions.get(year, 0) for year in HANCOM_YEARS},
        "pairs": pairs,
        "unmatched": unmatched,
    }


def validate_manifest(data: Any) -> list[str]:
    """스키마 핵심 계약. 외부 jsonschema 없이 필수 키·타입만 검사한다."""
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["$: object 가 아닙니다"]
    required = (
        "schemaVersion",
        "pairCount",
        "matchedSampleCount",
        "unmatchedCount",
        "oracleLinkCount",
        "pairs",
        "unmatched",
    )
    for key in required:
        if key not in data:
            errors.append(f"$: required 누락 ({key})")
    if data.get("schemaVersion") != SCHEMA_VERSION:
        errors.append("$.schemaVersion: '1.0' 이어야 합니다")
    for key in (
        "pairCount",
        "matchedSampleCount",
        "unmatchedCount",
        "oracleLinkCount",
        "targetPairCount",
    ):
        if key in data and not isinstance(data[key], int):
            errors.append(f"$.{key}: integer 가 아닙니다")
    pairs = data.get("pairs")
    if not isinstance(pairs, list):
        errors.append("$.pairs: array 가 아닙니다")
        pairs = []
    elif data.get("pairCount") != len(pairs):
        errors.append("$.pairCount: pairs 길이와 다릅니다")
    if isinstance(data.get("oracleLinkCount"), int) and data.get("oracleLinkCount") != len(
        pairs
    ):
        errors.append("$.oracleLinkCount: pairs 길이와 다릅니다")
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
    unmatched = data.get("unmatched")
    if not isinstance(unmatched, list):
        errors.append("$.unmatched: array 가 아닙니다")
        unmatched = []
    elif data.get("unmatchedCount") != len(unmatched):
        errors.append("$.unmatchedCount: unmatched 길이와 다릅니다")
    for i, item in enumerate(unmatched):
        loc = f"$.unmatched[{i}]"
        if not isinstance(item, dict):
            errors.append(f"{loc}: object 가 아닙니다")
            continue
        if "sample" not in item:
            errors.append(f"{loc}: required 누락 (sample)")
    matched = {item.get("sample") for item in pairs if isinstance(item, dict)}
    if data.get("matchedSampleCount") != len(matched):
        errors.append("$.matchedSampleCount: 서로 다른 sample 수와 다릅니다")
    return errors


def emit_json(data: dict[str, Any], output: Path | None, pretty: bool) -> None:
    text = json.dumps(data, ensure_ascii=False, indent=2 if pretty else None)
    if not pretty:
        text += "\n"
    elif not text.endswith("\n"):
        text += "\n"
    if output is None or str(output) == "-":
        sys.stdout.write(text)
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text, encoding="utf-8", newline="\n")


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="samples/ ↔ 한컴 기준 PDF 오라클 쌍 매니페스트를 만든다."
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
        "--output",
        "-o",
        type=Path,
        default=None,
        help="매니페스트 JSON 경로. 생략하거나 - 이면 stdout",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="들여쓰기 JSON",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="방출 전 스키마 핵심 계약을 검사한다",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        repo_root = (
            args.repo_root.resolve() if args.repo_root else discover_repo_root()
        )
        roots = tuple(item.strip() for item in args.roots.split(",") if item.strip())
        if not roots:
            raise UsageError("--roots 가 비어 있습니다")
        manifest = build_manifest(repo_root, args.samples_dir, roots)
        if args.validate:
            errors = validate_manifest(manifest)
            if errors:
                sys.stderr.write("매니페스트 스키마 위반:\n")
                for err in errors:
                    sys.stderr.write(f"  {err}\n")
                return 1
        emit_json(manifest, args.output, pretty=args.pretty or args.output is not None)
    except UsageError as exc:
        sys.stderr.write(f"사용법 오류: {exc}\n")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
