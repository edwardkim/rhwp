#!/usr/bin/env python3
"""M01-f 오라클 공개화 매니페스트·스윕 고도화 카탈로그.

devel 의 `oracle_resolver` / `page_smoke` / `issue_draft` 를 읽어
쌍 픽스처·커버리지 표·cheap 스윕 전사·짝 없는 샘플 카탈로그·이슈
초안 예시를 디스크에 쓴다. `scripts/visual_sweep.py` 는 읽거나
수정하지 않는다. 렌더·serializer·equation·pdf 엔진은 건드리지 않는다.

    python tools/oracle_public/fatten_catalog.py
    python tools/oracle_public/fatten_catalog.py --repo-root . --out-root tools/oracle_public
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import unicodedata
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable, Sequence

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import issue_draft  # noqa: E402
import oracle_resolver  # noqa: E402
import page_smoke  # noqa: E402

CLAIM_ID = "M01-f"
SCHEMA_VERSION = "1.0"
GENERATOR = "tools/oracle_public/fatten_catalog.py"
KIND = "oracleFattenCatalog"
HANCOM_YEARS = oracle_resolver.HANCOM_YEARS
DEFAULT_ORACLE_ROOTS = oracle_resolver.DEFAULT_ORACLE_ROOTS
SAMPLE_EXTS = oracle_resolver.SAMPLE_EXTS
LFS_PREFIX = b"version https://git-lfs.github.com/spec/v1"
PDF_VERSION_RE = re.compile(rb"%PDF-(\d+\.\d+)")

FAMILY_RULES: tuple[tuple[re.Pattern[str], str], ...] = (
    (re.compile(r"exam", re.I), "exam"),
    (re.compile(r"el-school|학교", re.I), "school"),
    (re.compile(r"^form|서식|기안", re.I), "form"),
    (re.compile(r"^hwp3|hwp-3", re.I), "hwp3"),
    (re.compile(r"footnote|미주|각주", re.I), "footnote"),
    (re.compile(r"calendar|달력", re.I), "calendar"),
    (re.compile(r"(^tb[-_]|table|표[_-])", re.I), "table"),
    (re.compile(r"regulatory|규제", re.I), "regulatory"),
    (re.compile(r"편람|handbook", re.I), "handbook"),
    (re.compile(r"(^eq[-_]|math|수식|equation)", re.I), "equation"),
    (re.compile(r"^shape|도형", re.I), "shape"),
    (re.compile(r"^hy[-_]", re.I), "hangul_font"),
    (re.compile(r"^(issue|pr)[-_0-9]", re.I), "issue"),
    (re.compile(r"^pau[-_]", re.I), "pau"),
    (re.compile(r"교육|통합", re.I), "education"),
    (re.compile(r"수출|수입|해외|직구", re.I), "trade"),
    (re.compile(r"별표|byeolpyo", re.I), "schedule"),
    (re.compile(r"업무계획|행정", re.I), "admin"),
    (re.compile(r"oss|rst", re.I), "oss"),
    (re.compile(r"blank|empty", re.I), "blank"),
)


class UsageError(Exception):
    """CLI 사용법·경로 오류."""


@dataclass
class FileProbe:
    path: str
    exists: bool
    bytes: int | None
    signature: str
    sha256_4k: str | None
    pdf_version: str | None
    pdf_pages: int | None
    note: str = ""


@dataclass
class FattenBundle:
    generated_at: str
    repo_root: Path
    out_root: Path
    manifest: dict[str, Any]
    pairs: list[dict[str, Any]]
    unmatched: list[dict[str, Any]]
    pdf_index: list[dict[str, Any]]
    sample_probes: dict[str, FileProbe]
    pdf_probes: dict[str, FileProbe]
    pair_rows: list[dict[str, Any]] = field(default_factory=list)
    unmatched_rows: list[dict[str, Any]] = field(default_factory=list)
    unused_pdfs: list[dict[str, Any]] = field(default_factory=list)
    written: list[str] = field(default_factory=list)


def nfc(value: str) -> str:
    return unicodedata.normalize("NFC", value)


def posix_rel(path: Path, root: Path) -> str:
    return path.resolve().relative_to(root.resolve()).as_posix()


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
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [json.dumps(row, ensure_ascii=False, separators=(",", ":")) for row in rows]
    write_text(path, "\n".join(lines) + ("\n" if lines else ""))


def md_cell(value: Any) -> str:
    return nfc(str(value)).replace("|", "\\|").replace("\n", " ")


def pct(part: int, whole: int) -> str:
    if whole <= 0:
        return "—"
    return f"{(100.0 * part / whole):.1f}%"


def sample_directory(sample: str) -> str:
    parts = Path(sample).parts
    if len(parts) <= 2:
        return ""
    return "/".join(parts[1:-1])


def classify_family(sample: str, stem: str) -> str:
    directory = sample_directory(sample)
    if directory.startswith("basic"):
        return "basic"
    if directory.startswith("hwpx"):
        return "hwpx_tree"
    blob = f"{directory}/{stem}"
    for pattern, name in FAMILY_RULES:
        if pattern.search(stem) or pattern.search(blob):
            return name
    if directory:
        return f"dir:{directory.split('/')[0]}"
    return "other"


def load_page_count_cache(repo_root: Path) -> dict[str, int]:
    """M01-5 incorporation_manifest 의 실측 쪽수를 재사용한다."""
    cache: dict[str, int] = {}
    path = HERE / "reports" / "incorporation_manifest.json"
    if not path.is_file():
        return cache
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return cache
    incorporation = raw.get("incorporation") if isinstance(raw, dict) else None
    if isinstance(incorporation, dict):
        for rows in incorporation.values():
            if not isinstance(rows, list):
                continue
            for item in rows:
                if not isinstance(item, dict):
                    continue
                rel = item.get("path")
                count = item.get("page_count")
                if isinstance(rel, str) and isinstance(count, int):
                    cache[nfc(rel)] = count
    return cache


def fast_pdf_page_count(path: Path, size: int) -> int | None:
    """머리·꼬리만 읽어 /Type /Pages /Count 를 찾는다. 전 스트림 해제 없음."""
    head_n = min(size, 768 * 1024)
    tail_n = min(size, 256 * 1024)
    try:
        with path.open("rb") as handle:
            head = handle.read(head_n)
            tail = b""
            if size > head_n:
                handle.seek(max(0, size - tail_n))
                tail = handle.read(tail_n)
    except OSError:
        return None
    count, objs = page_smoke._page_count_from_bytes(head + tail)
    if count is not None:
        return count
    if objs:
        return objs
    return None


def probe_file(
    path: Path,
    *,
    count_pdf_pages: bool,
    page_cache: dict[str, int] | None = None,
    repo_root: Path | None = None,
) -> FileProbe:
    rel = path.as_posix()
    cache_key = None
    if repo_root is not None:
        try:
            cache_key = nfc(posix_rel(path, repo_root))
        except ValueError:
            cache_key = nfc(path.name)
    if not path.is_file():
        return FileProbe(rel, False, None, "MISSING", None, None, None, "파일 없음")
    try:
        size = path.stat().st_size
    except OSError as exc:
        return FileProbe(rel, True, None, "ERROR", None, None, None, f"stat 실패: {exc}")
    try:
        head = path.read_bytes() if size <= 4096 else path.open("rb").read(4096)
    except OSError as exc:
        return FileProbe(rel, True, size, "ERROR", None, None, None, f"읽기 실패: {exc}")
    digest = hashlib.sha256(head).hexdigest()
    if size == 0:
        return FileProbe(rel, True, 0, "EMPTY", digest, None, None, "빈 파일")
    if head.startswith(LFS_PREFIX) or head.lstrip().startswith(b"version https://git-lfs"):
        return FileProbe(rel, True, size, "LFS", digest, None, None, "Git LFS 포인터")
    version = None
    pages = None
    note = ""
    signature = "OTHER"
    if head[:1024].find(b"%PDF") >= 0:
        signature = "PDF"
        match = PDF_VERSION_RE.search(head[:1024])
        if match:
            version = match.group(1).decode("ascii", "replace")
        if count_pdf_pages:
            cached = None
            if page_cache and cache_key:
                cached = page_cache.get(cache_key)
            if cached is not None:
                pages = cached
                note = "page_count=incorporation_manifest"
            else:
                pages = fast_pdf_page_count(path, size)
                if pages is None and size <= 8_000_000:
                    try:
                        pages = page_smoke.pdf_page_count(path)
                        note = "page_count=page_smoke"
                    except page_smoke.PageSmokeError as exc:
                        note = str(exc)
                        signature = "PDF_PAGE_ERROR"
                elif pages is None:
                    note = "쪽수 파싱 실패(빠른 경로)"
                    signature = "PDF_PAGE_ERROR"
    elif path.suffix.lower() in SAMPLE_EXTS:
        signature = "HWP" if path.suffix.lower() == ".hwp" else "HWPX"
    return FileProbe(rel, True, size, signature, digest, version, pages, note)


def list_oracle_pdfs(repo_root: Path, roots: Sequence[str]) -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []
    for root_name in roots:
        root_dir = repo_root / root_name
        if not root_dir.is_dir():
            continue
        for path in sorted(root_dir.rglob("*.pdf"), key=lambda p: nfc(p.as_posix()).lower()):
            if not path.is_file():
                continue
            rel = posix_rel(path, repo_root)
            parent = posix_rel(path.parent, root_dir)
            if parent == ".":
                parent = ""
            items.append(
                {
                    "pdf": rel,
                    "oracleRoot": root_name,
                    "relParent": parent,
                    "filename": path.name,
                    "stem": nfc(path.stem),
                }
            )
    return items


def near_miss_pdfs(
    sample_stem: str,
    sample_parent: str,
    pdfs: Sequence[dict[str, Any]],
    *,
    limit: int = 6,
) -> list[dict[str, str]]:
    stem = nfc(sample_stem).lower()
    hits: list[tuple[int, dict[str, str]]] = []
    seen: set[str] = set()
    for item in pdfs:
        name = nfc(item["filename"]).lower()
        pdf_stem = nfc(item["stem"]).lower()
        score = 0
        if name.startswith(stem) or pdf_stem.startswith(stem):
            score = 3
        elif stem and stem in pdf_stem:
            score = 2
        elif pdf_stem and pdf_stem in stem:
            score = 1
        if score == 0:
            continue
        if sample_parent and item.get("relParent") == sample_parent:
            score += 1
        key = item["pdf"]
        if key in seen:
            continue
        seen.add(key)
        parsed = oracle_resolver.parse_oracle_suffix(sample_stem, item["filename"])
        why = "stem_prefix" if score >= 3 else "stem_contains"
        if parsed is None:
            why = "suffix_not_oracle"
        hits.append(
            (
                -score,
                {
                    "pdf": item["pdf"],
                    "why": why,
                    "oracleRoot": item["oracleRoot"],
                },
            )
        )
    hits.sort(key=lambda row: (row[0], nfc(row[1]["pdf"]).lower()))
    return [row[1] for row in hits[:limit]]


def near_miss_samples(
    pdf_stem: str,
    pdf_parent: str,
    samples: Sequence[oracle_resolver.SampleDoc],
    *,
    limit: int = 6,
) -> list[dict[str, str]]:
    stem = nfc(pdf_stem).lower()
    hits: list[tuple[int, dict[str, str]]] = []
    for sample in samples:
        sample_stem = nfc(sample.stem).lower()
        score = 0
        if stem.startswith(sample_stem) or sample_stem.startswith(stem):
            score = 3
        elif sample_stem and sample_stem in stem:
            score = 2
        elif stem and stem in sample_stem:
            score = 1
        if score == 0:
            continue
        if pdf_parent and sample.rel_parent == pdf_parent:
            score += 1
        parsed = oracle_resolver.parse_oracle_suffix(sample.stem, f"{pdf_stem}.pdf")
        why = "stem_prefix" if score >= 3 else "stem_contains"
        if parsed is None:
            why = "suffix_not_oracle"
        hits.append(
            (
                -score,
                {
                    "sample": sample.sample,
                    "why": why,
                    "sourceFormat": sample.source_format,
                },
            )
        )
    hits.sort(key=lambda row: (row[0], nfc(row[1]["sample"]).lower()))
    return [row[1] for row in hits[:limit]]


def unused_reason(pdf_item: dict[str, Any], candidates: list[dict[str, str]]) -> str:
    if not candidates:
        return "no_stem_candidate"
    if all(item.get("why") == "suffix_not_oracle" for item in candidates):
        return "suffix_not_oracle"
    return "unmatched_relative_path"


def cheap_verdict(sample: FileProbe, pdf: FileProbe) -> tuple[str, str]:
    if not sample.exists:
        return "MISSING_SAMPLE", sample.note or "샘플 없음"
    if not pdf.exists:
        return "MISSING_PDF", pdf.note or "PDF 없음"
    if pdf.signature == "LFS":
        return "LFS_POINTER", "한컴 PDF 가 LFS 포인터다"
    if pdf.signature == "EMPTY":
        return "EMPTY_PDF", "한컴 PDF 가 비어 있다"
    if pdf.signature == "PDF_PAGE_ERROR":
        return "PAGE_ERROR", pdf.note or "쪽수 파싱 실패"
    if pdf.signature != "PDF":
        return "NOT_PDF", f"시그니처={pdf.signature}"
    return "CHEAP_OK", ""


def build_bundle(repo_root: Path, out_root: Path, roots: Sequence[str]) -> FattenBundle:
    repo_root = repo_root.resolve()
    out_root = out_root.resolve()
    present_roots = [name for name in roots if (repo_root / name).is_dir()]
    manifest = oracle_resolver.build_manifest(repo_root, roots=present_roots)
    errors = oracle_resolver.validate_manifest(manifest)
    if errors:
        raise UsageError("매니페스트 검증 실패:\n" + "\n".join(errors))
    samples = oracle_resolver.walk_samples(repo_root / "samples", repo_root)
    pdf_index = list_oracle_pdfs(repo_root, present_roots)
    page_cache = load_page_count_cache(repo_root)
    sample_probes: dict[str, FileProbe] = {}
    pdf_probes: dict[str, FileProbe] = {}
    for sample in samples:
        path = repo_root / sample.sample
        sample_probes[sample.sample] = probe_file(
            path, count_pdf_pages=False, page_cache=page_cache, repo_root=repo_root
        )
    for item in pdf_index:
        path = repo_root / item["pdf"]
        pdf_probes[item["pdf"]] = probe_file(
            path, count_pdf_pages=True, page_cache=page_cache, repo_root=repo_root
        )

    pair_rows: list[dict[str, Any]] = []
    for pair in manifest["pairs"]:
        sample_probe = sample_probes.get(pair["sample"]) or probe_file(
            repo_root / pair["sample"],
            count_pdf_pages=False,
            page_cache=page_cache,
            repo_root=repo_root,
        )
        pdf_probe = pdf_probes.get(pair["pdf"]) or probe_file(
            repo_root / pair["pdf"],
            count_pdf_pages=True,
            page_cache=page_cache,
            repo_root=repo_root,
        )
        verdict, note = cheap_verdict(sample_probe, pdf_probe)
        pair_rows.append(
            {
                "id": pair.get("id") or f"{pair['sample']}::{pair['pdf']}",
                "sample": pair["sample"],
                "pdf": pair["pdf"],
                "stem": pair["stem"],
                "hancomVersion": pair["hancomVersion"],
                "variant": pair.get("variant") or pair["hancomVersion"],
                "sourceFormat": pair["sourceFormat"],
                "oracleRoot": pair["oracleRoot"],
                "directory": sample_directory(pair["sample"]),
                "family": classify_family(pair["sample"], pair["stem"]),
                "sampleBytes": sample_probe.bytes,
                "pdfBytes": pdf_probe.bytes,
                "pdfVersion": pdf_probe.pdf_version,
                "pdfPages": pdf_probe.pdf_pages,
                "sampleExists": sample_probe.exists,
                "pdfExists": pdf_probe.exists,
                "pdfSignature": pdf_probe.signature,
                "verdict": verdict,
                "note": note,
                "repro": (
                    "python tools/oracle_public/page_smoke.py --pair "
                    f"{pair['sample']} {pair['pdf']}"
                ),
            }
        )

    unmatched_rows: list[dict[str, Any]] = []
    for item in manifest["unmatched"]:
        sample = item["sample"]
        stem = item.get("stem") or Path(sample).stem
        probe = sample_probes.get(sample) or probe_file(
            repo_root / sample,
            count_pdf_pages=False,
            page_cache=page_cache,
            repo_root=repo_root,
        )
        parent = sample_directory(sample)
        misses = near_miss_pdfs(stem, parent, pdf_index)
        unmatched_rows.append(
            {
                "sample": sample,
                "stem": stem,
                "sourceFormat": item.get("sourceFormat") or Path(sample).suffix.lstrip("."),
                "reason": item.get("reason") or "no_oracle_pdf",
                "directory": parent,
                "family": classify_family(sample, stem),
                "sampleBytes": probe.bytes,
                "sampleExists": probe.exists,
                "nearMissCount": len(misses),
                "suggestedPdfs": misses,
            }
        )

    paired_pdfs = {row["pdf"] for row in pair_rows}
    unused_pdfs: list[dict[str, Any]] = []
    for item in pdf_index:
        if item["pdf"] in paired_pdfs:
            continue
        probe = pdf_probes[item["pdf"]]
        candidates = near_miss_samples(item["stem"], item["relParent"], samples)
        unused_pdfs.append(
            {
                "pdf": item["pdf"],
                "oracleRoot": item["oracleRoot"],
                "relParent": item["relParent"],
                "filename": item["filename"],
                "stem": item["stem"],
                "bytes": probe.bytes,
                "pdfPages": probe.pdf_pages,
                "signature": probe.signature,
                "reason": unused_reason(item, candidates),
                "candidateSamples": candidates,
            }
        )

    return FattenBundle(
        generated_at=utc_now(),
        repo_root=repo_root,
        out_root=out_root,
        manifest=manifest,
        pairs=list(manifest["pairs"]),
        unmatched=list(manifest["unmatched"]),
        pdf_index=pdf_index,
        sample_probes=sample_probes,
        pdf_probes=pdf_probes,
        pair_rows=pair_rows,
        unmatched_rows=unmatched_rows,
        unused_pdfs=unused_pdfs,
    )


def rel_out(bundle: FattenBundle, *parts: str) -> Path:
    return bundle.out_root.joinpath(*parts)


def record(bundle: FattenBundle, path: Path) -> str:
    try:
        rel = posix_rel(path, bundle.out_root)
    except ValueError:
        rel = path.as_posix()
    bundle.written.append(rel.replace("\\", "/"))
    return rel


def emit_pair_fixtures(bundle: FattenBundle) -> None:
    by_year: dict[str, list[dict[str, Any]]] = {year: [] for year in HANCOM_YEARS}
    by_format: dict[str, list[dict[str, Any]]] = defaultdict(list)
    by_root: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for pair in bundle.pairs:
        year = pair["hancomVersion"]
        if year in by_year:
            by_year[year].append(pair)
        by_format[pair["sourceFormat"]].append(pair)
        by_root[pair["oracleRoot"]].append(pair)

    for year, rows in by_year.items():
        path = rel_out(bundle, "fixtures", "pairs", "by_year", f"{year}.json")
        write_json(
            path,
            {
                "schemaVersion": SCHEMA_VERSION,
                "claim": CLAIM_ID,
                "kind": "oraclePairFixture",
                "hancomVersion": year,
                "pairCount": len(rows),
                "pairs": rows,
            },
        )
        record(bundle, path)

    for fmt, rows in sorted(by_format.items()):
        path = rel_out(bundle, "fixtures", "pairs", "by_format", f"{fmt}.jsonl")
        write_jsonl(path, rows)
        record(bundle, path)
    for root_name, rows in sorted(by_root.items()):
        path = rel_out(bundle, "fixtures", "pairs", "by_root", f"{root_name}.jsonl")
        write_jsonl(path, rows)
        record(bundle, path)

    tsv_path = rel_out(bundle, "fixtures", "pairs", "index.tsv")
    tsv_lines = [
        "id\tsample\tpdf\tstem\thancomVersion\tvariant\tsourceFormat\toracleRoot\tfamily"
    ]
    for row in bundle.pair_rows:
        tsv_lines.append(
            "\t".join(
                [
                    row["id"],
                    row["sample"],
                    row["pdf"],
                    row["stem"],
                    row["hancomVersion"],
                    row["variant"],
                    row["sourceFormat"],
                    row["oracleRoot"],
                    row["family"],
                ]
            )
        )
    write_text(tsv_path, "\n".join(tsv_lines))
    record(bundle, tsv_path)


def emit_coverage_tables(bundle: FattenBundle) -> None:
    sample_count = int(bundle.manifest["matchedSampleCount"]) + int(
        bundle.manifest["unmatchedCount"]
    )
    matched = int(bundle.manifest["matchedSampleCount"])
    pair_count = int(bundle.manifest["pairCount"])

    by_dir: dict[str, dict[str, int]] = defaultdict(
        lambda: {"sampleCount": 0, "matchedSampleCount": 0, "unmatchedCount": 0, "pairCount": 0}
    )
    by_family: dict[str, dict[str, int]] = defaultdict(
        lambda: {"sampleCount": 0, "matchedSampleCount": 0, "unmatchedCount": 0, "pairCount": 0}
    )
    matrix: dict[tuple[str, str], int] = defaultdict(int)
    matched_samples = {row["sample"] for row in bundle.pair_rows}
    for row in bundle.pair_rows:
        directory = row["directory"] or "(root)"
        family = row["family"]
        by_dir[directory]["pairCount"] += 1
        by_family[family]["pairCount"] += 1
        matrix[(row["sourceFormat"], row["hancomVersion"])] += 1
    seen_samples: set[str] = set()
    extra_rows = [
        {"sample": item["sample"], "directory": item["directory"], "family": item["family"]}
        for item in bundle.unmatched_rows
    ]
    for row in bundle.pair_rows + extra_rows:
        sample = row["sample"]
        if sample in seen_samples:
            continue
        seen_samples.add(sample)
        directory = row["directory"] or "(root)"
        family = row["family"]
        by_dir[directory]["sampleCount"] += 1
        by_family[family]["sampleCount"] += 1
        if sample in matched_samples:
            by_dir[directory]["matchedSampleCount"] += 1
            by_family[family]["matchedSampleCount"] += 1
        else:
            by_dir[directory]["unmatchedCount"] += 1
            by_family[family]["unmatchedCount"] += 1

    matrix_lines = [
        "# M01-f 오라클 커버리지 행렬",
        "",
        f"생성기: `{GENERATOR}` · 시각: `{bundle.generated_at}`",
        f"샘플 {sample_count} · 짝 있는 샘플 {matched} · 링크 {pair_count}.",
        "",
        "| 형식 \\ 한컴 | " + " | ".join(HANCOM_YEARS) + " | 합 |",
        "| --- | " + " | ".join(["---:" for _ in HANCOM_YEARS]) + " | ---: |",
    ]
    for fmt in ("hwp", "hwpx"):
        cells = [matrix[(fmt, year)] for year in HANCOM_YEARS]
        matrix_lines.append(
            f"| `{fmt}` | " + " | ".join(str(n) for n in cells) + f" | {sum(cells)} |"
        )
    totals = [sum(matrix[(fmt, year)] for fmt in ("hwp", "hwpx")) for year in HANCOM_YEARS]
    matrix_lines.append(
        "| 합 | " + " | ".join(str(n) for n in totals) + f" | {sum(totals)} |"
    )
    matrix_lines.extend(
        [
            "",
            "## 요약",
            "",
            "| 항목 | 수 |",
            "| --- | ---: |",
            f"| 샘플 | {sample_count} |",
            f"| 짝 있는 샘플 | {matched} |",
            f"| 짝 없는 샘플 | {bundle.manifest['unmatchedCount']} |",
            f"| 오라클 링크 | {pair_count} |",
            f"| 커버리지 | {pct(matched, sample_count)} |",
            f"| 미사용 오라클 PDF | {len(bundle.unused_pdfs)} |",
            "",
        ]
    )
    path = rel_out(bundle, "reports", "coverage_matrix.md")
    write_text(path, "\n".join(matrix_lines))
    record(bundle, path)

    def render_bucket_table(title: str, buckets: dict[str, dict[str, int]]) -> str:
        lines = [
            f"# {title}",
            "",
            f"생성기: `{GENERATOR}` · 시각: `{bundle.generated_at}`",
            "",
            "| 키 | 샘플 | 짝 있음 | 짝 없음 | 링크 | 커버리지 |",
            "| --- | ---: | ---: | ---: | ---: | ---: |",
        ]
        for key in sorted(buckets, key=lambda item: nfc(item).lower()):
            bucket = buckets[key]
            lines.append(
                "| `{key}` | {sampleCount} | {matchedSampleCount} | {unmatchedCount} | "
                "{pairCount} | {coverage} |".format(
                    key=md_cell(key),
                    sampleCount=bucket["sampleCount"],
                    matchedSampleCount=bucket["matchedSampleCount"],
                    unmatchedCount=bucket["unmatchedCount"],
                    pairCount=bucket["pairCount"],
                    coverage=pct(bucket["matchedSampleCount"], bucket["sampleCount"]),
                )
            )
        lines.append("")
        return "\n".join(lines)

    path = rel_out(bundle, "reports", "coverage_by_directory.md")
    write_text(path, render_bucket_table("M01-f 디렉터리별 오라클 커버리지", by_dir))
    record(bundle, path)
    path = rel_out(bundle, "reports", "coverage_by_family.md")
    write_text(path, render_bucket_table("M01-f 문서군별 오라클 커버리지", by_family))
    record(bundle, path)

    pair_md = [
        "# M01-f 오라클 쌍 색인",
        "",
        f"링크 {len(bundle.pair_rows)}건. cheap 스윕 판정은 `transcripts/cheap_sweep.md`.",
        "",
        "| # | 샘플 | PDF | 연도 | 형식 | 쪽 | 판정 |",
        "| ---: | --- | --- | ---: | --- | ---: | --- |",
    ]
    for i, row in enumerate(bundle.pair_rows, start=1):
        pages = row["pdfPages"] if row["pdfPages"] is not None else "—"
        pair_md.append(
            f"| {i} | `{md_cell(row['sample'])}` | `{md_cell(row['pdf'])}` | "
            f"{row['hancomVersion']} | {row['sourceFormat']} | {pages} | {row['verdict']} |"
        )
    pair_md.append("")
    path = rel_out(bundle, "reports", "pair_index.md")
    write_text(path, "\n".join(pair_md))
    record(bundle, path)

    write_json(
        rel_out(bundle, "catalogs", "sample_families.json"),
        {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM_ID,
            "byFamily": {
                key: value
                for key, value in sorted(by_family.items(), key=lambda item: nfc(item[0]).lower())
            },
            "byDirectory": {
                key: value
                for key, value in sorted(by_dir.items(), key=lambda item: nfc(item[0]).lower())
            },
        },
    )
    record(bundle, rel_out(bundle, "catalogs", "sample_families.json"))


def emit_unmatched_catalogs(bundle: FattenBundle) -> None:
    catalog = {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "kind": "unmatchedSampleCatalog",
        "generator": GENERATOR,
        "generatedAt": bundle.generated_at,
        "unmatchedCount": len(bundle.unmatched_rows),
        "samples": bundle.unmatched_rows,
    }
    path = rel_out(bundle, "catalogs", "unmatched.json")
    write_json(path, catalog)
    record(bundle, path)

    lines = [
        "# M01-f 짝 없는 샘플 카탈로그",
        "",
        f"{len(bundle.unmatched_rows)}건. 이유는 resolver 의 `no_oracle_pdf` 이며,",
        "같은 stem 의 PDF 가 다른 경로·접미사로 있으면 `suggestedPdfs` 에 남긴다.",
        "",
        "| # | 경로 | 형식 | 군 | 바이트 | 근접 PDF |",
        "| ---: | --- | --- | --- | ---: | ---: |",
    ]
    for i, row in enumerate(bundle.unmatched_rows, start=1):
        lines.append(
            f"| {i} | `{md_cell(row['sample'])}` | {row['sourceFormat']} | "
            f"{row['family']} | {row['sampleBytes'] if row['sampleBytes'] is not None else '—'} | "
            f"{row['nearMissCount']} |"
        )
    lines.append("")
    path = rel_out(bundle, "catalogs", "unmatched.md")
    write_text(path, "\n".join(lines))
    record(bundle, path)

    by_dir: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in bundle.unmatched_rows:
        by_dir[row["directory"] or "(root)"].append(row)
    dir_lines = [
        "# M01-f 짝 없는 샘플 · 디렉터리",
        "",
        "| 디렉터리 | 건수 | 근접 PDF 있는 건 |",
        "| --- | ---: | ---: |",
    ]
    for key in sorted(by_dir, key=lambda item: nfc(item).lower()):
        rows = by_dir[key]
        near = sum(1 for item in rows if item["nearMissCount"])
        dir_lines.append(f"| `{md_cell(key)}` | {len(rows)} | {near} |")
    dir_lines.extend(["", "## 목록", ""])
    for key in sorted(by_dir, key=lambda item: nfc(item).lower()):
        dir_lines.append(f"### `{key}`")
        dir_lines.append("")
        for row in by_dir[key]:
            sugg = ", ".join(f"`{item['pdf']}`" for item in row["suggestedPdfs"][:3]) or "(없음)"
            dir_lines.append(f"- `{row['sample']}` · {row['family']} · 근접: {sugg}")
        dir_lines.append("")
    path = rel_out(bundle, "catalogs", "unmatched_by_directory.md")
    write_text(path, "\n".join(dir_lines))
    record(bundle, path)

    unused_path = rel_out(bundle, "catalogs", "unused_oracle_pdfs.json")
    write_json(
        unused_path,
        {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM_ID,
            "kind": "unusedOraclePdfCatalog",
            "count": len(bundle.unused_pdfs),
            "pdfs": bundle.unused_pdfs,
        },
    )
    record(bundle, unused_path)
    unused_md = [
        "# M01-f 매니페스트에 안 묶인 오라클 PDF",
        "",
        f"{len(bundle.unused_pdfs)}건. 상대 경로·포맷 태그·허용 접미사 밖.",
        "",
        "| # | PDF | 루트 | 쪽 | 이유 | 후보 샘플 |",
        "| ---: | --- | --- | ---: | --- | ---: |",
    ]
    for i, row in enumerate(bundle.unused_pdfs, start=1):
        pages = row["pdfPages"] if row["pdfPages"] is not None else "—"
        unused_md.append(
            f"| {i} | `{md_cell(row['pdf'])}` | {row['oracleRoot']} | {pages} | "
            f"{row['reason']} | {len(row['candidateSamples'])} |"
        )
    unused_md.append("")
    path = rel_out(bundle, "catalogs", "unused_oracle_pdfs.md")
    write_text(path, "\n".join(unused_md))
    record(bundle, path)


def emit_sweep_transcripts(bundle: FattenBundle) -> None:
    verdicts = Counter(row["verdict"] for row in bundle.pair_rows)
    by_pages = sorted(
        bundle.pair_rows,
        key=lambda row: (-(row["pdfPages"] or -1), nfc(row["id"]).lower()),
    )
    by_bytes = sorted(
        bundle.pair_rows,
        key=lambda row: (-(row["pdfBytes"] or -1), nfc(row["id"]).lower()),
    )
    errors = [row for row in bundle.pair_rows if row["verdict"] != "CHEAP_OK"]
    transcript = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": "oracleCheapSweepTranscript",
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "generatedAt": bundle.generated_at,
        "mode": "cheap-stat+pdf-count",
        "pairCount": len(bundle.pair_rows),
        "byVerdict": dict(sorted(verdicts.items())),
        "rowsPath": "transcripts/cheap_sweep.ndjson",
        "note": (
            "rhwp dump-pages / export-pdf / visual_sweep.py 는 호출하지 않는다. "
            "한컴 PDF /Count 와 파일 크기만 잰다. 전 행은 cheap_sweep.ndjson."
        ),
        "largestPages": by_pages[:20],
        "largestBytes": by_bytes[:20],
        "errorRows": errors,
    }
    path = rel_out(bundle, "transcripts", "cheap_sweep.json")
    write_json(path, transcript)
    record(bundle, path)
    path = rel_out(bundle, "transcripts", "cheap_sweep.ndjson")
    write_jsonl(path, bundle.pair_rows)
    record(bundle, path)
    path = rel_out(bundle, "transcripts", "verdict_counts.json")
    write_json(
        path,
        {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM_ID,
            "pairCount": len(bundle.pair_rows),
            "byVerdict": dict(sorted(verdicts.items())),
        },
    )
    record(bundle, path)

    md = [
        "# M01-f cheap 스윕 전사",
        "",
        "한컴 PDF 쪽수(`/Count`)와 바이트만. `scripts/visual_sweep.py` 미사용.",
        "",
        "| 판정 | 건 |",
        "| --- | ---: |",
    ]
    for key, count in sorted(verdicts.items()):
        md.append(f"| `{key}` | {count} |")
    md.extend(
        [
            "",
            f"전수 {len(bundle.pair_rows)}건.",
            "",
            "| # | 샘플 | PDF | 쪽 | 샘플B | PDFB | 판정 |",
            "| ---: | --- | --- | ---: | ---: | ---: | --- |",
        ]
    )
    for i, row in enumerate(bundle.pair_rows, start=1):
        md.append(
            "| {i} | `{sample}` | `{pdf}` | {pages} | {sb} | {pb} | {verdict} |".format(
                i=i,
                sample=md_cell(row["sample"]),
                pdf=md_cell(row["pdf"]),
                pages=row["pdfPages"] if row["pdfPages"] is not None else "—",
                sb=row["sampleBytes"] if row["sampleBytes"] is not None else "—",
                pb=row["pdfBytes"] if row["pdfBytes"] is not None else "—",
                verdict=row["verdict"],
            )
        )
    md.append("")
    path = rel_out(bundle, "transcripts", "cheap_sweep.md")
    write_text(path, "\n".join(md))
    record(bundle, path)


def select_anomaly_documents(bundle: FattenBundle) -> list[dict[str, Any]]:
    ranked = sorted(
        bundle.pair_rows,
        key=lambda row: (
            -(row["pdfPages"] or 0),
            -(row["pdfBytes"] or 0),
            nfc(row["id"]).lower(),
        ),
    )
    picked: list[dict[str, Any]] = []
    seen: set[str] = set()
    for row in ranked:
        if row["id"] in seen:
            continue
        pages = row["pdfPages"] or 0
        size = row["pdfBytes"] or 0
        if pages < 12 and size < 1_200_000:
            continue
        seen.add(row["id"])
        picked.append(row)
        if len(picked) >= 18:
            break
    if len(picked) < 8:
        for row in ranked:
            if row["id"] in seen:
                continue
            seen.add(row["id"])
            picked.append(row)
            if len(picked) >= 12:
                break
    return picked


def emit_issue_draft_examples(bundle: FattenBundle) -> None:
    anomalies = select_anomaly_documents(bundle)
    documents: list[dict[str, Any]] = []
    for row in anomalies:
        pages = row["pdfPages"]
        documents.append(
            {
                "id": row["stem"] or Path(row["sample"]).stem,
                "hwp": row["sample"],
                "pdf": row["pdf"],
                "pages": pages if isinstance(pages, int) else 0,
                "metrics": {
                    "pdf_pages": pages if pages is not None else 0,
                    "pdf_bytes": row["pdfBytes"] if row["pdfBytes"] is not None else 0,
                    "sample_bytes": row["sampleBytes"] if row["sampleBytes"] is not None else 0,
                    "worst_pages": [1] if pages else [],
                },
                "repro": {
                    "command": row["repro"],
                    "cwd": ".",
                },
                "notes": (
                    f"cheap 스윕 판정 `{row['verdict']}` · 한컴 {row['hancomVersion']} · "
                    f"군 `{row['family']}` · 쪽수 엔진/visual_sweep 미사용."
                ),
            }
        )
    for row in bundle.unmatched_rows[:4]:
        documents.append(
            {
                "id": f"unmatched-{row['stem']}",
                "hwp": row["sample"],
                "pdf": row["suggestedPdfs"][0]["pdf"] if row["suggestedPdfs"] else "",
                "pages": 0,
                "exceeds": True,
                "metrics": {
                    "pdf_pages": 0,
                    "pdf_bytes": 0,
                    "sample_bytes": row["sampleBytes"] if row["sampleBytes"] is not None else 0,
                    "worst_pages": [],
                },
                "repro": {
                    "command": (
                        "python tools/oracle_public/oracle_resolver.py --pretty --validate"
                    ),
                    "cwd": ".",
                },
                "notes": (
                    f"짝 없는 샘플. 이유 `{row['reason']}` · 근접 PDF {row['nearMissCount']}건."
                ),
            }
        )
    small = [row for row in bundle.pair_rows if (row["pdfPages"] or 0) <= 2]
    for row in small[:3]:
        documents.append(
            {
                "id": f"pass-{row['stem']}",
                "hwp": row["sample"],
                "pdf": row["pdf"],
                "pages": row["pdfPages"] or 0,
                "metrics": {
                    "pdf_pages": row["pdfPages"] or 0,
                    "pdf_bytes": row["pdfBytes"] or 0,
                    "sample_bytes": row["sampleBytes"] or 0,
                    "worst_pages": [],
                },
                "notes": "작은 문서 — 쪽수 게이트 통과 예시.",
            }
        )

    report = {
        "schema": "oracle_public.failure_report/v1",
        "generated_at": bundle.generated_at,
        "source": "oracle_public.fatten_catalog.cheap",
        "rhwp_bin": "(unused)",
        "dpi": 96,
        "pixel_diff_threshold": 12,
        "threshold": {"metric": "pdf_pages", "op": ">=", "value": 12},
        "documents": documents,
    }
    report_path = rel_out(bundle, "drafts", "examples", "report_cheap_anomalies.json")
    write_json(report_path, report)
    record(bundle, report_path)

    parsed = issue_draft.parse_report(report)
    template = issue_draft.DEFAULT_TEMPLATE.read_text(encoding="utf-8")
    out_dir = rel_out(bundle, "drafts", "examples")
    manifest = issue_draft.write_drafts(
        parsed,
        out_dir,
        template,
        force=True,
        dry_run=False,
        report_path=report_path,
    )
    record(bundle, out_dir / "manifest.json")
    for item in manifest.get("drafts") or []:
        path = Path(item["path"])
        if path.is_file():
            record(bundle, path)


def emit_summary(bundle: FattenBundle) -> dict[str, Any]:
    verdicts = Counter(row["verdict"] for row in bundle.pair_rows)
    summary = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": KIND,
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "generatedAt": bundle.generated_at,
        "repoRoot": bundle.repo_root.as_posix(),
        "pairCount": len(bundle.pair_rows),
        "matchedSampleCount": bundle.manifest["matchedSampleCount"],
        "unmatchedCount": len(bundle.unmatched_rows),
        "unusedOraclePdfCount": len(bundle.unused_pdfs),
        "oraclePdfCount": len(bundle.pdf_index),
        "byHancomVersion": bundle.manifest.get("byHancomVersion"),
        "byVerdict": dict(sorted(verdicts.items())),
        "written": bundle.written,
        "constraints": {
            "visualSweepTouched": False,
            "engineTouched": False,
            "gymTouched": False,
        },
    }
    path = rel_out(bundle, "reports", "fatten_summary.json")
    write_json(path, summary)
    record(bundle, path)
    summary["written"] = bundle.written
    write_json(path, summary)

    md = [
        "# M01-f 오라클 공개화 고도화 요약",
        "",
        f"- 클레임: `{CLAIM_ID}`",
        f"- 생성기: `{GENERATOR}`",
        f"- 시각: `{bundle.generated_at}`",
        f"- 링크: **{len(bundle.pair_rows)}**",
        f"- 짝 있는 샘플: **{bundle.manifest['matchedSampleCount']}**",
        f"- 짝 없는 샘플: **{len(bundle.unmatched_rows)}**",
        f"- 미사용 오라클 PDF: **{len(bundle.unused_pdfs)}**",
        "",
        "## 산출물",
        "",
        "| 경로 |",
        "| --- |",
    ]
    for rel in bundle.written:
        md.append(f"| `{md_cell(rel)}` |")
    md.extend(
        [
            "",
            "## 하지 않은 것",
            "",
            "- `scripts/visual_sweep.py` 미수정",
            "- serializer / canvaskit / equation / pdf renderer 미수정",
            "- gym 미수정",
            "",
        ]
    )
    path = rel_out(bundle, "reports", "fatten_summary.md")
    write_text(path, "\n".join(md))
    record(bundle, path)
    return summary


def run(repo_root: Path, out_root: Path, roots: Sequence[str]) -> dict[str, Any]:
    bundle = build_bundle(repo_root, out_root, roots)
    emit_pair_fixtures(bundle)
    emit_coverage_tables(bundle)
    emit_unmatched_catalogs(bundle)
    emit_sweep_transcripts(bundle)
    emit_issue_draft_examples(bundle)
    return emit_summary(bundle)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="M01-f 오라클 공개화 카탈로그 생성")
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=None,
        help="저장소 루트 (기본: 자동 탐지)",
    )
    parser.add_argument(
        "--out-root",
        type=Path,
        default=None,
        help="산출 루트 (기본: tools/oracle_public)",
    )
    parser.add_argument(
        "--oracle-root",
        action="append",
        dest="oracle_roots",
        default=None,
        help="오라클 PDF 루트. 여러 번 지정 가능.",
    )
    parser.add_argument("--json", action="store_true", help="요약을 stdout JSON 으로")
    return parser


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        repo = (
            args.repo_root.resolve()
            if args.repo_root
            else oracle_resolver.discover_repo_root()
        )
        out = args.out_root.resolve() if args.out_root else HERE
        roots = tuple(args.oracle_roots) if args.oracle_roots else DEFAULT_ORACLE_ROOTS
        summary = run(repo, out, roots)
    except (UsageError, oracle_resolver.UsageError, OSError) as exc:
        print(f"오류: {exc}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(summary, ensure_ascii=False, indent=2))
    else:
        print(
            f"M01-f 산출 {len(summary['written'])}파일 · "
            f"pairs={summary['pairCount']} unmatched={summary['unmatchedCount']}"
        )
        for rel in summary["written"]:
            print(f"  {rel}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
