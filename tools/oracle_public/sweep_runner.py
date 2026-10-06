#!/usr/bin/env python3
"""매니페스트 기반 rhwp 렌더 vs 커밋된 한컴 PDF 전수 스윕 (M01-2).

planet6897 비공개 corpus compare 의 공개판. 신규 파일만.
`scripts/visual_sweep.py` 는 읽거나 수정하지 않는다.

짝:
  `--manifest` JSON (M01-1 스키마: pairs[].sample + pairs[].pdf).
  매니페스트가 없고 형제 `oracle_resolver.py` 가 있으면 그 CLI 계약을 호출한다.
  둘 다 없으면 같은 매칭 규칙을 이 파일에 재구현한다. #5360 병합을 기다리지 않는다.

비교 (기본 cheap):
  page-count — `rhwp dump-pages` vs 한컴 PDF `/Count`
  file-size  — 한컴 PDF·샘플 바이트, `--export-pdf` 이면 rhwp export-pdf 용량
  선택 `--mode export-svg|render-diff|fidelity`

devel 실측 pairCount 는 409 (매칭 샘플 389). 이슈 참고값 269를 실제 개수로
주장하지 않는다. 리포트의 pairCount 는 매니페스트·발견 결과 그대로다.

판정은 데이터다. 기본 종료 코드 0. `--strict` 만 MISMATCH/ERROR 에서 1.

한 줄 전수 (로컬, 무거움 — 409쌍을 돌릴 수 있다):

    git sparse-checkout add pdf crates
    cargo build --release --bin rhwp
    python tools/oracle_public/sweep_runner.py --top 20 -o oracle-sweep.json

매니페스트가 있으면:

    python tools/oracle_public/sweep_runner.py \\
        --manifest tools/oracle_public/oracle_pairs.json --top 20

CI 는 tiny fixture 만 (`python tools/oracle_public/tests/test_sweep_runner.py`).
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import unicodedata
import zlib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Sequence

SCHEMA_VERSION = "1.0"
KIND = "oracleSweepReport"
GENERATOR = "tools/oracle_public/sweep_runner.py"
CLAIM_ID = "M01-2"
# 이슈 참고값. 실측이 아니다. resolver 실측은 409.
REFERENCE_TARGET_PAIR_COUNT = 269
MEASURED_DEVEL_PAIR_COUNT = 409
DEFAULT_TOP = 10
DEFAULT_ORACLE_ROOTS = ("pdf",)
SAMPLE_EXTS = {".hwp", ".hwpx"}
HANCOM_YEARS = ("2018", "2020", "2022", "2024")
MODES = ("cheap", "export-svg", "render-diff", "fidelity")
REPO_DEFAULT = Path(__file__).resolve().parents[2]
RESOLVER_REL = Path("tools") / "oracle_public" / "oracle_resolver.py"
FIDELITY_REL = Path("tools") / "fidelity_compare" / "fidelity_compare.py"

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
TEXT_PAGE_TOTAL_RE = re.compile(r"\((\d+)\s*페이지\)")
TEXT_PAGE_HEADER_RE = re.compile(r"=== 페이지 (\d+)\b")
PDF_PAGE_OBJ_RE = re.compile(rb"/Type\s*/Page(?![sA-Za-z])")
PDF_PAGES_NODE_RE = re.compile(rb"/Type\s*/Pages\b")
PDF_COUNT_RE = re.compile(rb"/Count\s+(\d+)")
PDF_STREAM_START_RE = re.compile(rb"stream\r?\n")
RENDER_DIFF_STATUS_RE = re.compile(r"status:\s*(\S+)", re.IGNORECASE)
SCORE_ERROR = 1_000_000_000_000.0
SCORE_PAGE = 1_000_000.0
SCORE_SIZE = 1_000.0


class SweepError(Exception):
    """짝·비교 실패. 행 verdict=ERROR 로 내린다."""


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


@dataclass(frozen=True)
class Pair:
    sample: str
    pdf: str
    stem: str = ""
    hancom_version: str = ""
    variant: str = ""
    source_format: str = ""
    oracle_root: str = ""
    pair_id: str = ""

    def doc_path(self, repo: Path) -> Path:
        path = Path(self.sample)
        return path if path.is_absolute() else repo / path

    def pdf_path(self, repo: Path) -> Path:
        path = Path(self.pdf)
        return path if path.is_absolute() else repo / path


@dataclass
class Row:
    sample: str
    pdf: str
    stem: str
    verdict: str
    score: float
    metric: str
    rhwp_pages: int | None = None
    pdf_pages: int | None = None
    page_delta: int | None = None
    sample_bytes: int | None = None
    pdf_bytes: int | None = None
    rhwp_pdf_bytes: int | None = None
    size_ratio: float | None = None
    svg_pages: int | None = None
    fidelity: dict[str, Any] | None = None
    note: str = ""
    repro: str = ""
    hancom_version: str = ""
    pair_id: str = ""

    def to_json(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "id": self.pair_id or f"{self.sample}::{self.pdf}",
            "sample": self.sample,
            "pdf": self.pdf,
            "stem": self.stem,
            "hancomVersion": self.hancom_version,
            "verdict": self.verdict,
            "score": self.score,
            "metric": self.metric,
            "rhwpPages": self.rhwp_pages,
            "pdfPages": self.pdf_pages,
            "pageDelta": self.page_delta,
            "sampleBytes": self.sample_bytes,
            "pdfBytes": self.pdf_bytes,
            "rhwpPdfBytes": self.rhwp_pdf_bytes,
            "sizeRatio": self.size_ratio,
            "svgPages": self.svg_pages,
            "note": self.note,
            "repro": self.repro,
        }
        if self.fidelity is not None:
            payload["fidelity"] = self.fidelity
        return payload


@dataclass
class Report:
    schema_version: str = SCHEMA_VERSION
    kind: str = KIND
    generator: str = GENERATOR
    claim: str = CLAIM_ID
    mode: str = "cheap"
    strict: bool = False
    rhwp: str = ""
    pair_source: str = "glob"
    pair_count: int = 0
    target_pair_count: int = REFERENCE_TARGET_PAIR_COUNT
    measured_devel_pair_count: int = MEASURED_DEVEL_PAIR_COUNT
    top_n: int = DEFAULT_TOP
    rows: list[Row] = field(default_factory=list)
    unmatched: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    @property
    def summary(self) -> dict[str, int]:
        counts = {
            "pairs": 0,
            "match": 0,
            "mismatch": 0,
            "error": 0,
            "unmatched": len(self.unmatched),
        }
        for row in self.rows:
            counts["pairs"] += 1
            key = row.verdict.lower()
            if key in counts:
                counts[key] += 1
        return counts

    def failures(self) -> list[Row]:
        return [row for row in self.rows if row.verdict in {"MISMATCH", "ERROR"}]

    def top_failures(self) -> list[Row]:
        ranked = sorted(
            self.failures(),
            key=lambda row: (-row.score, nfc(row.sample).lower(), nfc(row.pdf).lower()),
        )
        if self.top_n <= 0:
            return ranked
        return ranked[: self.top_n]

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": self.schema_version,
            "kind": self.kind,
            "generator": self.generator,
            "claim": self.claim,
            "mode": self.mode,
            "strict": self.strict,
            "rhwp": self.rhwp,
            "pairSource": self.pair_source,
            "pairCount": self.pair_count,
            "comparedCount": len(self.rows),
            "targetPairCount": self.target_pair_count,
            "measuredDevelPairCount": self.measured_devel_pair_count,
            "targetPairCountNote": "이슈 참고값. 실측이 아니다.",
            "topN": self.top_n,
            "summary": self.summary,
            "topFailures": [row.to_json() for row in self.top_failures()],
            "rows": [row.to_json() for row in self.rows],
            "unmatched": list(self.unmatched),
            "notes": list(self.notes),
        }


def nfc(value: str) -> str:
    return unicodedata.normalize("NFC", value)


def posix_rel(path: Path, root: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return path.as_posix().replace("\\", "/")


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
        if not path.is_file() or path.suffix.lower() not in SAMPLE_EXTS:
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


def pair_from_sample(sample: SampleDoc, hit: OracleHit) -> Pair:
    return Pair(
        sample=sample.sample,
        pdf=hit.pdf,
        stem=sample.stem,
        hancom_version=hit.hancom_version,
        variant=hit.variant,
        source_format=sample.source_format,
        oracle_root=hit.oracle_root,
        pair_id=f"{sample.sample}::{hit.pdf}",
    )


def discover_pairs(
    repo_root: Path,
    samples_dir: Path | None = None,
    roots: Sequence[str] = DEFAULT_ORACLE_ROOTS,
) -> tuple[list[Pair], list[str]]:
    samples_dir = samples_dir or (repo_root / "samples")
    present = [name for name in roots if (repo_root / name).is_dir()]
    samples = walk_samples(samples_dir, repo_root)
    index = index_oracle_pdfs(repo_root, present)
    pairs: list[Pair] = []
    unmatched: list[str] = []
    for sample in samples:
        hits = match_sample(sample, index, present)
        if hits:
            for hit in hits:
                pairs.append(pair_from_sample(sample, hit))
        else:
            unmatched.append(sample.sample)
    return pairs, unmatched


def _pair_keys(item: dict[str, Any]) -> tuple[str | None, str | None]:
    sample = item.get("sample") or item.get("doc") or item.get("hwp") or item.get("source")
    pdf = item.get("pdf") or item.get("reference") or item.get("oracle")
    sample_s = str(sample).strip() if sample else None
    pdf_s = str(pdf).strip() if pdf else None
    return sample_s, pdf_s


def pairs_from_manifest_dict(
    raw: dict[str, Any], repo: Path
) -> tuple[list[Pair], list[str]]:
    items = raw.get("pairs") or raw.get("matches") or raw.get("items") or []
    if not isinstance(items, list):
        raise UsageError("$.pairs 가 array 가 아닙니다")
    unmatched: list[str] = []
    extra = raw.get("unmatched") or raw.get("unpaired") or []
    if isinstance(extra, list):
        for entry in extra:
            if isinstance(entry, str) and entry.strip():
                unmatched.append(entry.replace("\\", "/"))
            elif isinstance(entry, dict):
                sample, _ = _pair_keys(entry)
                if sample:
                    unmatched.append(sample.replace("\\", "/"))
    pairs: list[Pair] = []
    for item in items:
        if not isinstance(item, dict):
            continue
        sample_s, pdf_s = _pair_keys(item)
        if not sample_s or not pdf_s:
            continue
        sample_path = Path(sample_s)
        pdf_path = Path(pdf_s)
        rel_sample = (
            posix_rel(sample_path, repo)
            if sample_path.is_absolute()
            else sample_s.replace("\\", "/")
        )
        rel_pdf = (
            posix_rel(pdf_path, repo) if pdf_path.is_absolute() else pdf_s.replace("\\", "/")
        )
        stem = str(item.get("stem") or Path(rel_sample).stem)
        year = str(item.get("hancomVersion") or "")
        oracle_root = str(item.get("oracleRoot") or "")
        if not oracle_root:
            parts = Path(rel_pdf).parts
            oracle_root = parts[0] if parts else ""
        pairs.append(
            Pair(
                sample=rel_sample,
                pdf=rel_pdf,
                stem=stem,
                hancom_version=year,
                variant=str(item.get("variant") or year),
                source_format=str(
                    item.get("sourceFormat") or Path(rel_sample).suffix.lstrip(".").lower()
                ),
                oracle_root=oracle_root,
                pair_id=str(item.get("id") or f"{rel_sample}::{rel_pdf}"),
            )
        )
    return pairs, unmatched


def load_manifest(path: Path, repo: Path) -> tuple[list[Pair], list[str], dict[str, Any]]:
    """M01-1 오라클 쌍 매니페스트. sample/pdf 키. 형제 resolver 를 임포트하지 않는다."""
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise UsageError(f"매니페스트가 object 가 아닙니다: {path}")
    pairs, unmatched = pairs_from_manifest_dict(raw, repo)
    return pairs, unmatched, raw


def load_resolver_module(path: Path) -> Any:
    spec = importlib.util.spec_from_file_location("oracle_resolver_m01", path)
    if spec is None or spec.loader is None:
        raise UsageError(f"oracle_resolver 를 불러오지 못했습니다: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def pairs_from_resolver_module(
    module: Any, repo: Path, roots: Sequence[str]
) -> tuple[list[Pair], list[str]]:
    manifest = module.build_manifest(repo, None, roots)
    if not isinstance(manifest, dict):
        raise UsageError("oracle_resolver.build_manifest 가 object 가 아닙니다")
    return pairs_from_manifest_dict(manifest, repo)


def find_resolver(repo: Path, explicit: Path | None = None) -> Path | None:
    if explicit is not None:
        return explicit if explicit.is_file() else None
    sibling = Path(__file__).resolve().parent / "oracle_resolver.py"
    if sibling.is_file():
        return sibling
    cand = repo / RESOLVER_REL
    return cand if cand.is_file() else None


def find_fidelity_compare(repo: Path) -> Path | None:
    cand = repo / FIDELITY_REL
    return cand if cand.is_file() else None


def find_rhwp(explicit: str | None = None, repo: Path | None = None) -> Path | None:
    if explicit:
        cand = Path(explicit)
        return cand if cand.is_file() else None
    for key in ("RHWP_BIN", "RHWP"):
        env = os.environ.get(key)
        if env:
            cand = Path(env)
            if cand.is_file():
                return cand
    which = shutil.which("rhwp") or shutil.which("rhwp.exe")
    if which:
        return Path(which)
    root = repo or REPO_DEFAULT
    for rel in (
        Path("target") / "release-test" / "rhwp.exe",
        Path("target") / "release-test" / "rhwp",
        Path("target") / "release" / "rhwp.exe",
        Path("target") / "release" / "rhwp",
        Path("target") / "debug" / "rhwp.exe",
        Path("target") / "debug" / "rhwp",
    ):
        cand = root / rel
        if cand.is_file():
            return cand
    return None


def parse_dump_pages_count(stdout: str) -> int:
    text = stdout.strip()
    if not text:
        raise SweepError("dump-pages 출력이 비어 있다")
    blob = text
    if not blob.startswith("{"):
        brace = blob.find("{")
        if brace >= 0:
            blob = blob[brace:]
    if blob.startswith("{"):
        try:
            obj = json.loads(blob)
        except json.JSONDecodeError:
            obj = None
        if isinstance(obj, dict) and "pageCount" in obj:
            try:
                return int(obj["pageCount"])
            except (TypeError, ValueError) as exc:
                raise SweepError(f"pageCount 파싱 실패: {obj.get('pageCount')!r}") from exc
    match = TEXT_PAGE_TOTAL_RE.search(text)
    if match:
        return int(match.group(1))
    headers = [int(x) for x in TEXT_PAGE_HEADER_RE.findall(text)]
    if headers:
        return max(headers)
    raise SweepError("dump-pages 출력에서 쪽수를 찾지 못했다")


def run_cmd(
    cmd: Sequence[str],
    *,
    timeout: float,
    runner: Any = None,
    cwd: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    run = runner or subprocess.run
    try:
        return run(
            list(cmd),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            cwd=str(cwd) if cwd is not None else None,
        )
    except subprocess.TimeoutExpired as exc:
        raise SweepError(f"timeout {timeout}s: {' '.join(cmd[:3])}") from exc
    except OSError as exc:
        raise SweepError(f"실행 실패: {exc}") from exc


def dump_pages_count(
    rhwp: Path | str,
    doc: Path,
    *,
    timeout: float = 180.0,
    runner: Any = None,
) -> int:
    proc = run_cmd(
        [str(rhwp), "dump-pages", str(doc), "--json"],
        timeout=timeout,
        runner=runner,
    )
    if getattr(proc, "returncode", 1) != 0:
        err = (getattr(proc, "stderr", "") or "").strip().splitlines()
        tail = err[-1] if err else f"rc={proc.returncode}"
        raise SweepError(f"dump-pages rc={proc.returncode}: {tail}")
    return parse_dump_pages_count(getattr(proc, "stdout", "") or "")


def write_minimal_pdf(path: Path, page_count: int) -> None:
    if page_count < 1:
        raise ValueError("page_count 는 1 이상이어야 한다")
    kids = " ".join(f"{i} 0 R" for i in range(3, 3 + page_count))
    objects = [
        "1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n",
        f"2 0 obj << /Type /Pages /Kids [{kids}] /Count {page_count} >> endobj\n",
    ]
    for i in range(page_count):
        objects.append(
            f"{3 + i} 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >> endobj\n"
        )
    header = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
    chunks: list[bytes] = []
    offsets: list[int] = []
    pos = len(header)
    for obj in objects:
        raw = obj.encode("ascii")
        offsets.append(pos)
        chunks.append(raw)
        pos += len(raw)
    xref_pos = pos
    xref = [f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode("ascii")]
    for off in offsets:
        xref.append(f"{off:010d} 00000 n \n".encode("ascii"))
    trailer = (
        f"trailer << /Size {len(objects) + 1} /Root 1 0 R >>\n"
        f"startxref\n{xref_pos}\n%%EOF\n"
    ).encode("ascii")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(header + b"".join(chunks) + b"".join(xref) + trailer)


def _inflate_pdf_stream(blob: bytes) -> bytes | None:
    if blob.endswith(b"\r\n"):
        blob = blob[:-2]
    elif blob.endswith(b"\n") or blob.endswith(b"\r"):
        blob = blob[:-1]
    for wbits in (zlib.MAX_WBITS, -zlib.MAX_WBITS):
        try:
            return zlib.decompress(blob, wbits)
        except zlib.error:
            continue
    return None


def iter_pdf_payloads(data: bytes) -> list[bytes]:
    payloads = [data]
    pos = 0
    while True:
        start = PDF_STREAM_START_RE.search(data, pos)
        if not start:
            break
        end = data.find(b"endstream", start.end())
        if end < 0:
            break
        inflated = _inflate_pdf_stream(data[start.end() : end])
        if inflated:
            payloads.append(inflated)
        pos = end + 9
    return payloads


def _page_count_from_bytes(data: bytes) -> tuple[int | None, int]:
    counts: list[int] = []
    for match in PDF_PAGES_NODE_RE.finditer(data):
        window = data[match.start() : match.start() + 800]
        found = PDF_COUNT_RE.search(window)
        if found:
            counts.append(int(found.group(1)))
    page_objs = len(PDF_PAGE_OBJ_RE.findall(data))
    return (max(counts) if counts else None, page_objs)


def pdf_page_count(path: Path) -> int:
    try:
        data = path.read_bytes()
    except OSError as exc:
        raise SweepError(f"PDF 읽기 실패: {exc}") from exc
    if b"%PDF" not in data[:1024]:
        raise SweepError("PDF 시그니처가 없다")
    best_count: int | None = None
    best_objs = 0
    for payload in iter_pdf_payloads(data):
        count, objs = _page_count_from_bytes(payload)
        if count is not None:
            best_count = count if best_count is None else max(best_count, count)
        best_objs += objs
    if best_count is not None:
        return best_count
    if best_objs:
        return best_objs
    try:
        from pypdf import PdfReader  # type: ignore
    except ImportError:
        PdfReader = None  # type: ignore
    if PdfReader is not None:
        try:
            n = len(PdfReader(str(path)).pages)
        except Exception as exc:  # noqa: BLE001
            raise SweepError(f"PDF 페이지 객체를 찾지 못했다 ({exc})") from exc
        if n:
            return n
    raise SweepError("PDF 페이지 객체를 찾지 못했다")


def file_size(path: Path) -> int:
    try:
        return path.stat().st_size
    except OSError as exc:
        raise SweepError(f"파일 크기 실패: {exc}") from exc


def export_pdf_bytes(
    rhwp: Path | str,
    doc: Path,
    out_pdf: Path,
    *,
    timeout: float,
    runner: Any = None,
) -> int:
    out_pdf.parent.mkdir(parents=True, exist_ok=True)
    proc = run_cmd(
        [str(rhwp), "export-pdf", str(doc), "-o", str(out_pdf)],
        timeout=timeout,
        runner=runner,
    )
    if getattr(proc, "returncode", 1) != 0:
        err = (getattr(proc, "stderr", "") or "").strip().splitlines()
        tail = err[-1] if err else f"rc={proc.returncode}"
        raise SweepError(f"export-pdf rc={proc.returncode}: {tail}")
    if not out_pdf.is_file():
        raise SweepError("export-pdf 산출물이 없다")
    return file_size(out_pdf)


def export_svg_pages(
    rhwp: Path | str,
    doc: Path,
    out_dir: Path,
    *,
    timeout: float,
    runner: Any = None,
) -> int:
    out_dir.mkdir(parents=True, exist_ok=True)
    proc = run_cmd(
        [str(rhwp), "export-svg", str(doc), "-o", str(out_dir)],
        timeout=timeout,
        runner=runner,
    )
    if getattr(proc, "returncode", 1) != 0:
        err = (getattr(proc, "stderr", "") or "").strip().splitlines()
        tail = err[-1] if err else f"rc={proc.returncode}"
        raise SweepError(f"export-svg rc={proc.returncode}: {tail}")
    return len(sorted(out_dir.glob("*.svg")))


def parse_render_diff_status(stdout: str) -> str:
    match = RENDER_DIFF_STATUS_RE.search(stdout or "")
    if match:
        return match.group(1).upper()
    text = (stdout or "").strip()
    if not text:
        raise SweepError("render-diff 출력이 비어 있다")
    return "UNKNOWN"


def run_render_diff(
    rhwp: Path | str,
    doc: Path,
    *,
    timeout: float,
    runner: Any = None,
) -> str:
    """자기 라운드트립. 한컴 PDF 가 아니라 렌더 경로 생존 확인."""
    proc = run_cmd(
        [str(rhwp), "render-diff", str(doc)],
        timeout=timeout,
        runner=runner,
    )
    status = parse_render_diff_status(getattr(proc, "stdout", "") or "")
    if getattr(proc, "returncode", 1) != 0 and status in {"UNKNOWN", ""}:
        err = (getattr(proc, "stderr", "") or "").strip().splitlines()
        tail = err[-1] if err else f"rc={proc.returncode}"
        raise SweepError(f"render-diff rc={proc.returncode}: {tail}")
    return status


def parse_fidelity_metric(out_dir: Path) -> dict[str, Any]:
    metric: dict[str, Any] = {"outDir": str(out_dir)}
    ledger = out_dir / "page-count-ledger.tsv"
    report = out_dir / "text-report.tsv"
    if ledger.is_file():
        lines = ledger.read_text(encoding="utf-8", errors="replace").splitlines()
        metric["pageCountLedger"] = lines[:8]
    if report.is_file():
        lines = report.read_text(encoding="utf-8", errors="replace").splitlines()
        metric["textReportRows"] = max(0, len(lines) - 1)
        worst = 0
        for line in lines[1:]:
            parts = line.split("\t")
            if len(parts) >= 2:
                try:
                    worst = max(worst, int(parts[1]))
                except ValueError:
                    continue
        metric["textDeltaWorst"] = worst
    return metric


def run_fidelity_compare(
    repo: Path,
    pair: Pair,
    work_dir: Path,
    *,
    timeout: float,
    python_exe: str,
    runner: Any = None,
    end_page: int | None = None,
) -> dict[str, Any]:
    script = find_fidelity_compare(repo)
    if script is None:
        raise SweepError("tools/fidelity_compare/fidelity_compare.py 가 없다")
    work_dir.mkdir(parents=True, exist_ok=True)
    last = 0 if end_page is None else max(0, end_page)
    label = re.sub(r"[^A-Za-z0-9._-]+", "_", pair.stem or Path(pair.sample).stem)[:40] or "pair"
    cmd = [
        python_exe,
        str(script),
        "0",
        str(last),
        "--source",
        str(pair.doc_path(repo)),
        "--reference-pdf",
        str(pair.pdf_path(repo)),
        "--label",
        f"oracle-sweep-{label}",
        "--text-only",
        "--out-dir",
        str(work_dir),
    ]
    proc = run_cmd(cmd, timeout=timeout, runner=runner, cwd=repo)
    if getattr(proc, "returncode", 1) != 0:
        err = (getattr(proc, "stderr", "") or "").strip().splitlines()
        tail = err[-1] if err else f"rc={proc.returncode}"
        raise SweepError(f"fidelity_compare rc={proc.returncode}: {tail}")
    return parse_fidelity_metric(work_dir)


def repro_command(sample: str, pdf: str, mode: str = "cheap") -> str:
    cmd = f"python tools/oracle_public/sweep_runner.py --pair {sample} {pdf}"
    if mode != "cheap":
        cmd += f" --mode {mode}"
    return cmd


def score_row(
    *,
    verdict: str,
    page_delta: int | None,
    size_ratio: float | None,
    pdf_bytes: int | None,
) -> float:
    if verdict == "ERROR":
        return SCORE_ERROR
    if verdict == "MATCH":
        return 0.0
    score = 0.0
    if page_delta is not None:
        score += abs(page_delta) * SCORE_PAGE
    if size_ratio is not None:
        score += abs(size_ratio - 1.0) * SCORE_SIZE
    if pdf_bytes:
        score += min(pdf_bytes, 10_000_000) / 10_000_000.0
    return score if score > 0 else 1.0


def compare_pair(
    pair: Pair,
    repo: Path,
    rhwp: Path | str | None,
    *,
    mode: str = "cheap",
    export_pdf: bool = False,
    timeout: float = 180.0,
    runner: Any = None,
    work_dir: Path | None = None,
    python_exe: str | None = None,
    rhwp_pages: int | None = None,
    size_threshold: float | None = None,
) -> Row:
    sample_s = pair.sample
    pdf_s = pair.pdf
    stem = pair.stem or Path(pair.sample).stem
    repro = repro_command(sample_s, pdf_s, mode)
    doc = pair.doc_path(repo)
    pdf_path = pair.pdf_path(repo)
    sample_bytes: int | None = None
    pdf_bytes: int | None = None
    pdf_n: int | None = None
    rhwp_n = rhwp_pages
    rhwp_pdf_bytes: int | None = None
    size_ratio: float | None = None
    svg_pages: int | None = None
    fidelity: dict[str, Any] | None = None
    notes: list[str] = []

    try:
        if doc.is_file():
            sample_bytes = file_size(doc)
        else:
            raise SweepError(f"샘플이 없다: {sample_s}")
        if not pdf_path.is_file():
            raise SweepError(f"한컴 PDF 가 없다: {pdf_s}")
        pdf_bytes = file_size(pdf_path)
        pdf_n = pdf_page_count(pdf_path)
        if rhwp_n is None:
            if rhwp is None:
                raise SweepError("rhwp 바이너리를 찾지 못했다 (RHWP_BIN/--rhwp/target/release)")
            rhwp_n = dump_pages_count(rhwp, doc, timeout=timeout, runner=runner)
        if export_pdf:
            if rhwp is None:
                raise SweepError("export-pdf 에 rhwp 가 필요하다")
            out_pdf = (work_dir or repo / "output" / "oracle_sweep") / "export" / f"{stem}.pdf"
            rhwp_pdf_bytes = export_pdf_bytes(
                rhwp, doc, out_pdf, timeout=timeout, runner=runner
            )
            if pdf_bytes:
                size_ratio = rhwp_pdf_bytes / pdf_bytes
        if mode in {"export-svg", "render-diff", "fidelity"}:
            if rhwp is None:
                raise SweepError(f"{mode} 에 rhwp 가 필요하다")
            svg_dir = (work_dir or repo / "output" / "oracle_sweep") / "svg" / stem
            if mode in {"export-svg", "render-diff"}:
                svg_pages = export_svg_pages(
                    rhwp, doc, svg_dir, timeout=timeout, runner=runner
                )
            if mode == "render-diff":
                status = run_render_diff(rhwp, doc, timeout=timeout, runner=runner)
                notes.append(f"render-diff={status} (HWP 자기비교, 한컴 PDF 아님)")
            if mode == "fidelity":
                fidelity = run_fidelity_compare(
                    repo,
                    pair,
                    (work_dir or repo / "output" / "oracle_sweep") / "fidelity" / stem,
                    timeout=timeout,
                    python_exe=python_exe or sys.executable,
                    runner=runner,
                    end_page=(pdf_n - 1) if pdf_n else 0,
                )
    except SweepError as exc:
        return Row(
            sample=sample_s,
            pdf=pdf_s,
            stem=stem,
            verdict="ERROR",
            score=SCORE_ERROR,
            metric="error",
            rhwp_pages=rhwp_n,
            pdf_pages=pdf_n,
            page_delta=None,
            sample_bytes=sample_bytes,
            pdf_bytes=pdf_bytes,
            rhwp_pdf_bytes=rhwp_pdf_bytes,
            size_ratio=size_ratio,
            svg_pages=svg_pages,
            fidelity=fidelity,
            note=str(exc),
            repro=repro,
            hancom_version=pair.hancom_version,
            pair_id=pair.pair_id,
        )

    page_delta = None if rhwp_n is None or pdf_n is None else rhwp_n - pdf_n
    reasons: list[str] = []
    if page_delta != 0:
        reasons.append(f"pages rhwp={rhwp_n} pdf={pdf_n} delta={page_delta:+d}")
    if svg_pages is not None and pdf_n is not None and svg_pages != pdf_n:
        reasons.append(f"svg={svg_pages} pdf={pdf_n}")
    if (
        size_threshold is not None
        and size_ratio is not None
        and abs(size_ratio - 1.0) > size_threshold
    ):
        reasons.append(f"size_ratio={size_ratio:.3f} threshold={size_threshold}")
    if fidelity and int(fidelity.get("textDeltaWorst") or 0) > 0:
        reasons.append(f"fidelity_text_delta={fidelity['textDeltaWorst']}")
    verdict = "MATCH" if not reasons else "MISMATCH"
    metric = "pages+size" if export_pdf else "pages"
    if mode != "cheap":
        metric = f"{metric}+{mode}"
    note = "; ".join(reasons + notes)
    score = score_row(
        verdict=verdict,
        page_delta=page_delta,
        size_ratio=size_ratio,
        pdf_bytes=pdf_bytes,
    )
    return Row(
        sample=sample_s,
        pdf=pdf_s,
        stem=stem,
        verdict=verdict,
        score=score,
        metric=metric,
        rhwp_pages=rhwp_n,
        pdf_pages=pdf_n,
        page_delta=page_delta,
        sample_bytes=sample_bytes,
        pdf_bytes=pdf_bytes,
        rhwp_pdf_bytes=rhwp_pdf_bytes,
        size_ratio=size_ratio,
        svg_pages=svg_pages,
        fidelity=fidelity,
        note=note,
        repro=repro,
        hancom_version=pair.hancom_version,
        pair_id=pair.pair_id,
    )


def run_sweep(
    *,
    repo: Path,
    pairs: Sequence[Pair],
    unmatched: Sequence[str],
    rhwp: Path | str | None,
    mode: str,
    strict: bool,
    pair_source: str,
    top_n: int,
    export_pdf: bool = False,
    timeout: float = 180.0,
    runner: Any = None,
    work_dir: Path | None = None,
    python_exe: str | None = None,
    size_threshold: float | None = None,
    notes: Sequence[str] | None = None,
    pair_count: int | None = None,
) -> Report:
    report = Report(
        mode=mode,
        strict=strict,
        rhwp=str(rhwp) if rhwp else "",
        pair_source=pair_source,
        pair_count=pair_count if pair_count is not None else len(pairs),
        top_n=top_n,
        unmatched=list(unmatched),
        notes=list(notes or ()),
    )
    dump_cache: dict[str, int | SweepError] = {}
    for pair in pairs:
        key = str(pair.doc_path(repo).resolve()) if pair.doc_path(repo).exists() else pair.sample
        injected: int | None = None
        dump_err: SweepError | None = None
        cached = dump_cache.get(key)
        if isinstance(cached, SweepError):
            dump_err = cached
        elif isinstance(cached, int):
            injected = cached
        elif rhwp is not None:
            doc = pair.doc_path(repo)
            if doc.is_file():
                try:
                    injected = dump_pages_count(rhwp, doc, timeout=timeout, runner=runner)
                    dump_cache[key] = injected
                except SweepError as exc:
                    dump_err = exc
                    dump_cache[key] = exc
        if dump_err is not None:
            pdf_n = None
            pdf_bytes = None
            try:
                pdf_path = pair.pdf_path(repo)
                if pdf_path.is_file():
                    pdf_n = pdf_page_count(pdf_path)
                    pdf_bytes = file_size(pdf_path)
            except SweepError:
                pass
            report.rows.append(
                Row(
                    sample=pair.sample,
                    pdf=pair.pdf,
                    stem=pair.stem or Path(pair.sample).stem,
                    verdict="ERROR",
                    score=SCORE_ERROR,
                    metric="error",
                    rhwp_pages=None,
                    pdf_pages=pdf_n,
                    page_delta=None,
                    sample_bytes=None,
                    pdf_bytes=pdf_bytes,
                    note=str(dump_err),
                    repro=repro_command(pair.sample, pair.pdf, mode),
                    hancom_version=pair.hancom_version,
                    pair_id=pair.pair_id,
                )
            )
            continue
        report.rows.append(
            compare_pair(
                pair,
                repo,
                rhwp,
                mode=mode,
                export_pdf=export_pdf,
                timeout=timeout,
                runner=runner,
                work_dir=work_dir,
                python_exe=python_exe,
                rhwp_pages=injected,
                size_threshold=size_threshold,
            )
        )
    return report


def format_human(report: Report) -> str:
    summary = report.summary
    lines = [
        f"# oracle-sweep pairs={summary['pairs']} match={summary['match']} "
        f"mismatch={summary['mismatch']} error={summary['error']} "
        f"unmatched={summary['unmatched']} top={report.top_n} mode={report.mode}",
        f"# pairCount={report.pair_count} measuredDevel={report.measured_devel_pair_count} "
        f"targetPairCount={report.target_pair_count}(참고, 실측 아님)",
        "# 판정은 데이터다. 기본 종료 코드 0. --strict 이면 MISMATCH/ERROR 시 1.",
        "sample\tpdf\trhwp\thangul\tdelta\tpdfBytes\tscore\tverdict\tmetric\trepro",
    ]
    for row in report.rows:
        rhwp_s = "" if row.rhwp_pages is None else str(row.rhwp_pages)
        pdf_s = "" if row.pdf_pages is None else str(row.pdf_pages)
        delta_s = "" if row.page_delta is None else f"{row.page_delta:+d}"
        bytes_s = "" if row.pdf_bytes is None else str(row.pdf_bytes)
        lines.append(
            f"{row.sample}\t{row.pdf}\t{rhwp_s}\t{pdf_s}\t{delta_s}\t"
            f"{bytes_s}\t{row.score:.3f}\t{row.verdict}\t{row.metric}\t{row.repro}"
        )
    top = report.top_failures()
    if top:
        lines.append(f"# TOP {len(top)} FAILURES")
        for i, row in enumerate(top, start=1):
            lines.append(
                f"# {i}. {row.verdict} score={row.score:.3f} metric={row.metric} {row.note}"
            )
            lines.append(f"#    REPRO {row.repro}")
            lines.append(f"#    rhwp dump-pages {row.sample} --json")
    if report.unmatched:
        lines.append("# unmatched")
        for item in report.unmatched:
            lines.append(f"# UNMATCHED {item}")
    for note in report.notes:
        lines.append(f"# note: {note}")
    return "\n".join(lines) + "\n"


def exit_code(report: Report) -> int:
    if not report.strict:
        return 0
    summary = report.summary
    if summary["mismatch"] or summary["error"]:
        return 1
    return 0


def emit_json(data: dict[str, Any], output: Path | None) -> None:
    text = json.dumps(data, ensure_ascii=False, indent=2)
    if not text.endswith("\n"):
        text += "\n"
    if output is None or str(output) == "-":
        sys.stdout.write(text)
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text, encoding="utf-8", newline="\n")


def _configure_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        reconf = getattr(stream, "reconfigure", None)
        if callable(reconf):
            try:
                reconf(encoding="utf-8")
            except (OSError, ValueError):
                pass


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--repo-root", type=Path, default=None, help="저장소 루트")
    parser.add_argument("--rhwp", default=None, help="rhwp 바이너리")
    parser.add_argument(
        "--manifest",
        type=Path,
        default=None,
        help="M01-1 oracle pair JSON (pairs[].sample + pairs[].pdf)",
    )
    parser.add_argument(
        "--resolver",
        type=Path,
        default=None,
        help="oracle_resolver.py 경로. 생략 시 형제 파일 또는 tools/oracle_public/oracle_resolver.py",
    )
    parser.add_argument("--samples-dir", type=Path, default=None)
    parser.add_argument(
        "--roots",
        default=",".join(DEFAULT_ORACLE_ROOTS),
        help="오라클 PDF 루트(쉼표 구분)",
    )
    parser.add_argument("--pair", nargs=2, metavar=("SAMPLE", "PDF"), help="단짝 재현")
    parser.add_argument("--limit", type=int, default=None, help="앞에서 N 짝만")
    parser.add_argument("--top", type=int, default=DEFAULT_TOP, help="실패 상위 N (기본 10)")
    parser.add_argument(
        "--mode",
        choices=MODES,
        default="cheap",
        help="cheap=쪽수+용량, export-svg/render-diff/fidelity 는 선택 경로",
    )
    parser.add_argument(
        "--export-pdf",
        action="store_true",
        help="rhwp export-pdf 용량을 한컴 PDF 와 비교 (cheap 경로 확장)",
    )
    parser.add_argument(
        "--size-threshold",
        type=float,
        default=None,
        help="export-pdf 용량비 |ratio-1| 가 이 값을 넘으면 MISMATCH",
    )
    parser.add_argument("--timeout", type=float, default=180.0)
    parser.add_argument("--work-dir", type=Path, default=None)
    parser.add_argument("--output", "-o", type=Path, default=None, help="JSON 리포트 경로")
    parser.add_argument("--json", action="store_true", dest="as_json", help="stdout JSON")
    parser.add_argument("--strict", action="store_true", help="MISMATCH/ERROR 가 있으면 종료 1")
    return parser


def resolve_pairs(
    *,
    repo: Path,
    args: argparse.Namespace,
    roots: Sequence[str],
    notes: list[str],
) -> tuple[list[Pair], list[str], str, int | None]:
    if args.pair:
        sample = args.pair[0].replace("\\", "/")
        pdf = args.pair[1].replace("\\", "/")
        pair = Pair(
            sample=sample,
            pdf=pdf,
            stem=Path(sample).stem,
            source_format=Path(sample).suffix.lstrip(".").lower(),
            pair_id=f"{sample}::{pdf}",
        )
        return [pair], [], "pair", 1

    if args.manifest:
        pairs, unmatched, raw = load_manifest(args.manifest, repo)
        source = f"manifest:{posix_rel(args.manifest, repo)}"
        declared = raw.get("pairCount")
        pair_count = declared if isinstance(declared, int) else len(pairs)
        if pair_count != REFERENCE_TARGET_PAIR_COUNT:
            notes.append(
                f"매니페스트 pairCount={pair_count} "
                f"(참고 target={REFERENCE_TARGET_PAIR_COUNT}, devel 실측={MEASURED_DEVEL_PAIR_COUNT})"
            )
        return pairs, unmatched, source, pair_count

    resolver = find_resolver(repo, args.resolver)
    if resolver is not None:
        try:
            module = load_resolver_module(resolver)
            pairs, unmatched = pairs_from_resolver_module(module, repo, roots)
            notes.append(f"oracle_resolver CLI 계약: {posix_rel(resolver, repo)}")
            return pairs, unmatched, f"resolver:{posix_rel(resolver, repo)}", len(pairs)
        except (UsageError, OSError, AttributeError) as exc:
            notes.append(f"oracle_resolver 호출 실패, 내장 매칭으로 진행: {exc}")

    missing = [name for name in roots if not (repo / name).is_dir()]
    if missing:
        notes.append(
            "PDF 루트 없음: "
            + ", ".join(missing)
            + " — `git sparse-checkout add pdf crates` 후 전수 스윕"
        )
    pairs, unmatched = discover_pairs(repo, args.samples_dir, roots)
    notes.append(
        f"내장 매칭 pairCount={len(pairs)} "
        f"(참고 target={REFERENCE_TARGET_PAIR_COUNT}, devel 실측={MEASURED_DEVEL_PAIR_COUNT})"
    )
    return pairs, unmatched, "match:stem-{year}.pdf", len(pairs)


def main(argv: Sequence[str] | None = None) -> int:
    _configure_stdio()
    args = build_parser().parse_args(list(argv) if argv is not None else None)
    try:
        repo = args.repo_root.resolve() if args.repo_root else discover_repo_root()
        roots = tuple(item.strip() for item in args.roots.split(",") if item.strip())
        if not roots:
            raise UsageError("--roots 가 비어 있습니다")
        if args.mode not in MODES:
            raise UsageError(f"--mode 는 {MODES} 중 하나여야 합니다")
        notes: list[str] = []
        pairs, unmatched, pair_source, declared_count = resolve_pairs(
            repo=repo, args=args, roots=roots, notes=notes
        )
        if args.limit is not None:
            pairs = list(pairs)[: max(0, args.limit)]
        rhwp = find_rhwp(args.rhwp, repo)
        if rhwp is None:
            notes.append("rhwp 없음 — 짝은 읽고 비교는 ERROR 로 남긴다")
        if not pairs:
            notes.append("비교할 짝이 없다. --manifest 또는 --pair 또는 pdf/ 매칭을 확인")
        report = run_sweep(
            repo=repo,
            pairs=pairs,
            unmatched=unmatched,
            rhwp=rhwp,
            mode=args.mode,
            strict=bool(args.strict),
            pair_source=pair_source,
            top_n=int(args.top),
            export_pdf=bool(args.export_pdf),
            timeout=float(args.timeout),
            work_dir=args.work_dir,
            size_threshold=args.size_threshold,
            notes=notes,
            pair_count=declared_count,
        )
        if args.output is not None:
            emit_json(report.to_json(), args.output)
            notes.append(f"json written: {posix_rel(args.output, repo)}")
            report.notes.append(notes[-1])
        if args.as_json:
            if args.output is None:
                emit_json(report.to_json(), None)
            else:
                sys.stdout.write(json.dumps(report.to_json(), ensure_ascii=False, indent=2) + "\n")
        else:
            sys.stdout.write(format_human(report))
        return exit_code(report)
    except UsageError as exc:
        sys.stderr.write(f"사용법 오류: {exc}\n")
        return 2
    except (OSError, json.JSONDecodeError) as exc:
        sys.stderr.write(f"사용법 오류: {exc}\n")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
