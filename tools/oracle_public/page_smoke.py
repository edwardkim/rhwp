#!/usr/bin/env python3
"""dump-pages 쪽수 vs 커밋된 한컴 PDF 쪽수 — 렌더 없는 초경량 1차 게이트.

MEGA QUEUE M01-4 (#5344). `scripts/visual_sweep.py` 는 건드리지 않는다.
M01-1 `oracle_resolver` 가 없어도 동작한다: 로컬 글롭 `pdf/{stem}.pdf` ·
`pdf/{stem}-*.pdf` (pdf/ · pdf-2020/ · pdf-large/) 또는 `--manifest`.

판정은 데이터다. 기본 종료 코드는 불일치가 있어도 0 이다. `--strict` 만
MISMATCH/ERROR 에서 1 을 낸다.

사용:
    python tools/oracle_public/page_smoke.py
    python tools/oracle_public/page_smoke.py --strict --json
    python tools/oracle_public/page_smoke.py --manifest tools/oracle_public/fixtures/pairs.json
    python tools/oracle_public/page_smoke.py --pair samples/foo.hwp pdf/foo-2022.pdf

전수 스윕(269쌍·커밋 PDF 전체)은 README 의 Full sweep 절.
CI 는 `python -m unittest tools.oracle_public.test_page_smoke` (tiny fixture).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import zlib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable, Sequence

SCHEMA_VERSION = 1
KIND = "pageSmokeReport"
PAIR_KIND = "pageSmokePairs"
DEFAULT_PDF_DIRS = ("pdf", "pdf-2020", "pdf-large")
DEFAULT_DOCS = ("samples",)
DOC_SUFFIXES = {".hwp", ".hwpx"}
TEXT_PAGE_TOTAL_RE = re.compile(r"\((\d+)\s*페이지\)")
TEXT_PAGE_HEADER_RE = re.compile(r"=== 페이지 (\d+)\b")
# /Type /Page 는 페이지 객체, /Type /Pages 는 트리 노드. 후자는 세면 안 된다.
PDF_PAGE_OBJ_RE = re.compile(rb"/Type\s*/Page(?![sA-Za-z])")
PDF_PAGES_NODE_RE = re.compile(rb"/Type\s*/Pages\b")
PDF_COUNT_RE = re.compile(rb"/Count\s+(\d+)")
PDF_STREAM_START_RE = re.compile(rb"stream\r?\n")

# 저장소 루트: tools/oracle_public/page_smoke.py → parents[2]
REPO_DEFAULT = Path(__file__).resolve().parents[2]


class PageSmokeError(Exception):
    """쪽수 산출 실패 — 판정 ERROR 행으로 내려보낸다."""


@dataclass(frozen=True)
class Pair:
    doc: Path
    pdf: Path
    stem: str = ""

    def rel_doc(self, repo: Path) -> str:
        return relpath(self.doc, repo)

    def rel_pdf(self, repo: Path) -> str:
        return relpath(self.pdf, repo)


@dataclass
class Row:
    doc: str
    pdf: str
    stem: str
    rhwp_pages: int | None
    pdf_pages: int | None
    delta: int | None
    verdict: str
    note: str = ""
    repro: str = ""

    def to_json(self) -> dict[str, Any]:
        return {
            "doc": self.doc,
            "pdf": self.pdf,
            "stem": self.stem,
            "rhwpPages": self.rhwp_pages,
            "pdfPages": self.pdf_pages,
            "delta": self.delta,
            "verdict": self.verdict,
            "note": self.note,
            "repro": self.repro,
        }


@dataclass
class Report:
    schema_version: int = SCHEMA_VERSION
    kind: str = KIND
    strict: bool = False
    rhwp: str = ""
    pair_source: str = "glob"
    rows: list[Row] = field(default_factory=list)
    unpaired: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    @property
    def summary(self) -> dict[str, int]:
        counts = {"pairs": 0, "match": 0, "mismatch": 0, "error": 0, "unpaired": len(self.unpaired)}
        for row in self.rows:
            counts["pairs"] += 1
            key = row.verdict.lower()
            if key in counts:
                counts[key] += 1
        return counts

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": self.schema_version,
            "kind": self.kind,
            "strict": self.strict,
            "rhwp": self.rhwp,
            "pairSource": self.pair_source,
            "summary": self.summary,
            "rows": [r.to_json() for r in self.rows],
            "mismatches": [r.to_json() for r in self.rows if r.verdict == "MISMATCH"],
            "errors": [r.to_json() for r in self.rows if r.verdict == "ERROR"],
            "unpaired": list(self.unpaired),
            "notes": list(self.notes),
        }


def relpath(path: Path, repo: Path) -> str:
    try:
        return path.resolve().relative_to(repo.resolve()).as_posix()
    except ValueError:
        return path.as_posix().replace("\\", "/")


def find_rhwp(explicit: str | None = None, repo: Path | None = None) -> Path | None:
    if explicit:
        cand = Path(explicit)
        return cand if cand.is_file() else None
    env = os.environ.get("RHWP")
    if env:
        cand = Path(env)
        if cand.is_file():
            return cand
    which = shutil.which("rhwp") or shutil.which("rhwp.exe")
    if which:
        return Path(which)
    root = repo or REPO_DEFAULT
    for rel in (
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
    """dump-pages 텍스트·JSON 어느 쪽이든 쪽수를 읽는다."""
    text = stdout.strip()
    if not text:
        raise PageSmokeError("dump-pages 출력이 비어 있다")

    # stdout 이 순수 JSON 이거나 앞에 잡음이 붙은 JSON.
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
                raise PageSmokeError(f"pageCount 파싱 실패: {obj.get('pageCount')!r}") from exc

    m = TEXT_PAGE_TOTAL_RE.search(text)
    if m:
        return int(m.group(1))

    headers = [int(x) for x in TEXT_PAGE_HEADER_RE.findall(text)]
    if headers:
        return max(headers)

    raise PageSmokeError("dump-pages 출력에서 쪽수를 찾지 못했다")


def dump_pages_count(
    rhwp: Path | str,
    doc: Path,
    *,
    timeout: float = 180.0,
    runner: Any = None,
) -> int:
    cmd = [str(rhwp), "dump-pages", str(doc), "--json"]
    run = runner or subprocess.run
    try:
        proc = run(
            cmd,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as exc:
        raise PageSmokeError(f"dump-pages timeout {timeout}s") from exc
    except OSError as exc:
        raise PageSmokeError(f"dump-pages 실행 실패: {exc}") from exc
    if getattr(proc, "returncode", 1) != 0:
        err = (getattr(proc, "stderr", "") or "").strip().splitlines()
        tail = err[-1] if err else f"rc={proc.returncode}"
        raise PageSmokeError(f"dump-pages rc={proc.returncode}: {tail}")
    return parse_dump_pages_count(getattr(proc, "stdout", "") or "")


def write_minimal_pdf(path: Path, page_count: int) -> None:
    """테스트·픽스처용 비압축 PDF. 렌더 없이 /Count 와 /Type /Page 가 일치한다."""
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
    """원문 + FlateDecode 스트림 해제본. 한컴 PDF 1.5+ 객체 스트림용."""
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
    """(/Type /Pages 의 /Count 최댓값 또는 None, /Type /Page 객체 수)."""
    counts: list[int] = []
    for match in PDF_PAGES_NODE_RE.finditer(data):
        window = data[match.start() : match.start() + 800]
        found = PDF_COUNT_RE.search(window)
        if found:
            counts.append(int(found.group(1)))
    page_objs = len(PDF_PAGE_OBJ_RE.findall(data))
    return (max(counts) if counts else None, page_objs)


def pdf_page_count(path: Path) -> int:
    """렌더 없이 PDF 쪽수를 센다. 표준 라이브러리만 사용.

    한컴 커밋 PDF 는 객체 스트림(FlateDecode) 안에 /Count 가 있다.
    1) /Type /Pages 노드의 /Count 최댓값
    2) /Type /Page 객체 수
    3) 선택: pypdf (있으면, 렌더 아님)
    """
    try:
        data = path.read_bytes()
    except OSError as exc:
        raise PageSmokeError(f"PDF 읽기 실패: {exc}") from exc
    if b"%PDF" not in data[:1024]:
        raise PageSmokeError("PDF 시그니처가 없다")

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
            raise PageSmokeError(f"PDF 페이지 객체를 찾지 못했다 ({exc})") from exc
        if n:
            return n

    raise PageSmokeError("PDF 페이지 객체를 찾지 못했다")


def _pair_keys(item: dict[str, Any]) -> tuple[str | None, str | None]:
    doc = item.get("doc") or item.get("sample") or item.get("hwp") or item.get("source")
    pdf = item.get("pdf") or item.get("reference") or item.get("oracle")
    doc_s = str(doc).strip() if doc else None
    pdf_s = str(pdf).strip() if pdf else None
    return doc_s, pdf_s


def load_manifest(path: Path, repo: Path) -> tuple[list[Pair], list[str]]:
    """자체 픽스처 매니페스트. M01-1 이 같은 키를 쓰면 그대로 읽는다(임포트 없음)."""
    raw = json.loads(path.read_text(encoding="utf-8"))
    items: Iterable[Any]
    unpaired: list[str] = []
    if isinstance(raw, list):
        items = raw
    elif isinstance(raw, dict):
        items = raw.get("pairs") or raw.get("matches") or raw.get("items") or []
        extra = raw.get("unpaired") or raw.get("unmatched") or []
        if isinstance(extra, list):
            for entry in extra:
                if isinstance(entry, str) and entry.strip():
                    unpaired.append(entry.replace("\\", "/"))
                elif isinstance(entry, dict):
                    d, _ = _pair_keys(entry)
                    if d:
                        unpaired.append(d.replace("\\", "/"))
    else:
        raise PageSmokeError(f"매니페스트 형식이 아니다: {path}")

    pairs: list[Pair] = []
    for item in items:
        if not isinstance(item, dict):
            continue
        doc_s, pdf_s = _pair_keys(item)
        if not doc_s or not pdf_s:
            continue
        doc = Path(doc_s)
        pdf = Path(pdf_s)
        if not doc.is_absolute():
            doc = repo / doc
        if not pdf.is_absolute():
            pdf = repo / pdf
        stem = str(item.get("stem") or doc.stem)
        pairs.append(Pair(doc=doc, pdf=pdf, stem=stem))
    return pairs, unpaired


def write_manifest(path: Path, pairs: Sequence[Pair], repo: Path, unpaired: Sequence[str] | None = None) -> None:
    payload = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": PAIR_KIND,
        "pairs": [
            {"doc": p.rel_doc(repo), "pdf": p.rel_pdf(repo), "stem": p.stem or p.doc.stem}
            for p in pairs
        ],
        "unpaired": [u.replace("\\", "/") for u in (unpaired or ())],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def find_pdfs_for_stem(stem: str, pdf_roots: Sequence[Path]) -> list[Path]:
    """`{stem}.pdf` 와 `{stem}-*.pdf` 를 각 pdf 루트에서 재귀 검색."""
    found: list[Path] = []
    seen: set[str] = set()
    for root in pdf_roots:
        if not root.is_dir():
            continue
        hits: list[Path] = []
        exact = root / f"{stem}.pdf"
        if exact.is_file():
            hits.append(exact)
        hits.extend(sorted(root.glob(f"{stem}-*.pdf")))
        hits.extend(sorted(root.rglob(f"{stem}.pdf")))
        hits.extend(sorted(root.rglob(f"{stem}-*.pdf")))
        for hit in hits:
            if not hit.is_file():
                continue
            key = str(hit.resolve()).casefold()
            if key in seen:
                continue
            seen.add(key)
            found.append(hit)
    return found


def iter_docs(docs_roots: Sequence[Path]) -> list[Path]:
    docs: list[Path] = []
    seen: set[str] = set()
    for root in docs_roots:
        if root.is_file() and root.suffix.lower() in DOC_SUFFIXES:
            key = str(root.resolve()).casefold()
            if key not in seen:
                seen.add(key)
                docs.append(root)
            continue
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*")):
            if not path.is_file() or path.suffix.lower() not in DOC_SUFFIXES:
                continue
            key = str(path.resolve()).casefold()
            if key in seen:
                continue
            seen.add(key)
            docs.append(path)
    return docs


def discover_pairs(
    repo: Path,
    *,
    docs_roots: Sequence[Path] | None = None,
    pdf_roots: Sequence[Path] | None = None,
) -> tuple[list[Pair], list[str]]:
    docs_list = list(docs_roots) if docs_roots is not None else [repo / d for d in DEFAULT_DOCS]
    pdf_list = list(pdf_roots) if pdf_roots is not None else [repo / d for d in DEFAULT_PDF_DIRS]
    pairs: list[Pair] = []
    unpaired: list[str] = []
    for doc in iter_docs(docs_list):
        stem = doc.stem
        pdfs = find_pdfs_for_stem(stem, pdf_list)
        if not pdfs:
            unpaired.append(relpath(doc, repo))
            continue
        for pdf in pdfs:
            pairs.append(Pair(doc=doc, pdf=pdf, stem=stem))
    return pairs, unpaired


def repro_command(doc: str, pdf: str) -> str:
    return f"python tools/oracle_public/page_smoke.py --pair {doc} {pdf}"


def compare_pair(
    pair: Pair,
    repo: Path,
    rhwp: Path | str | None,
    *,
    timeout: float = 180.0,
    runner: Any = None,
    rhwp_pages: int | None = None,
) -> Row:
    doc_s = pair.rel_doc(repo)
    pdf_s = pair.rel_pdf(repo)
    stem = pair.stem or pair.doc.stem
    repro = repro_command(doc_s, pdf_s)
    pdf_n: int | None = None
    rhwp_n = rhwp_pages
    try:
        pdf_n = pdf_page_count(pair.pdf)
    except PageSmokeError as exc:
        return Row(
            doc=doc_s,
            pdf=pdf_s,
            stem=stem,
            rhwp_pages=rhwp_n,
            pdf_pages=None,
            delta=None,
            verdict="ERROR",
            note=str(exc),
            repro=repro,
        )
    if rhwp_n is None:
        if rhwp is None:
            return Row(
                doc=doc_s,
                pdf=pdf_s,
                stem=stem,
                rhwp_pages=None,
                pdf_pages=pdf_n,
                delta=None,
                verdict="ERROR",
                note="rhwp 바이너리를 찾지 못했다 (RHWP/--rhwp/target/release)",
                repro=repro,
            )
        try:
            rhwp_n = dump_pages_count(rhwp, pair.doc, timeout=timeout, runner=runner)
        except PageSmokeError as exc:
            return Row(
                doc=doc_s,
                pdf=pdf_s,
                stem=stem,
                rhwp_pages=None,
                pdf_pages=pdf_n,
                delta=None,
                verdict="ERROR",
                note=str(exc),
                repro=repro,
            )
    delta = rhwp_n - pdf_n
    verdict = "MATCH" if delta == 0 else "MISMATCH"
    note = "" if verdict == "MATCH" else f"rhwp={rhwp_n} pdf={pdf_n} delta={delta:+d}"
    return Row(
        doc=doc_s,
        pdf=pdf_s,
        stem=stem,
        rhwp_pages=rhwp_n,
        pdf_pages=pdf_n,
        delta=delta,
        verdict=verdict,
        note=note,
        repro=repro,
    )


def run_smoke(
    *,
    repo: Path,
    pairs: Sequence[Pair],
    unpaired: Sequence[str],
    rhwp: Path | str | None,
    strict: bool,
    pair_source: str,
    timeout: float = 180.0,
    runner: Any = None,
    notes: Sequence[str] | None = None,
) -> Report:
    report = Report(
        strict=strict,
        rhwp=str(rhwp) if rhwp else "",
        pair_source=pair_source,
        unpaired=list(unpaired),
        notes=list(notes or ()),
    )
    dump_cache: dict[str, int | PageSmokeError] = {}
    for pair in pairs:
        key = str(pair.doc.resolve())
        injected: int | None = None
        dump_err: PageSmokeError | None = None
        cached = dump_cache.get(key)
        if isinstance(cached, PageSmokeError):
            dump_err = cached
        elif isinstance(cached, int):
            injected = cached
        elif rhwp is not None:
            try:
                injected = dump_pages_count(rhwp, pair.doc, timeout=timeout, runner=runner)
                dump_cache[key] = injected
            except PageSmokeError as exc:
                dump_err = exc
                dump_cache[key] = exc
        if dump_err is not None:
            try:
                pdf_n = pdf_page_count(pair.pdf)
            except PageSmokeError:
                pdf_n = None
            report.rows.append(
                Row(
                    doc=pair.rel_doc(repo),
                    pdf=pair.rel_pdf(repo),
                    stem=pair.stem or pair.doc.stem,
                    rhwp_pages=None,
                    pdf_pages=pdf_n,
                    delta=None,
                    verdict="ERROR",
                    note=str(dump_err),
                    repro=repro_command(pair.rel_doc(repo), pair.rel_pdf(repo)),
                )
            )
            continue
        report.rows.append(
            compare_pair(
                pair,
                repo,
                rhwp,
                timeout=timeout,
                runner=runner,
                rhwp_pages=injected,
            )
        )
    return report


def format_human(report: Report) -> str:
    summary = report.summary
    lines = [
        f"# page-smoke pairs={summary['pairs']} match={summary['match']} "
        f"mismatch={summary['mismatch']} error={summary['error']} unpaired={summary['unpaired']}",
        "# 판정은 데이터다. 기본 종료 코드 0. --strict 이면 MISMATCH/ERROR 시 1.",
        "doc\tpdf\trhwp\thangul\tdelta\tverdict\trepro",
    ]
    for row in report.rows:
        rhwp_s = "" if row.rhwp_pages is None else str(row.rhwp_pages)
        pdf_s = "" if row.pdf_pages is None else str(row.pdf_pages)
        delta_s = "" if row.delta is None else f"{row.delta:+d}"
        lines.append(
            f"{row.doc}\t{row.pdf}\t{rhwp_s}\t{pdf_s}\t{delta_s}\t{row.verdict}\t{row.repro}"
        )
    mismatches = [r for r in report.rows if r.verdict == "MISMATCH"]
    if mismatches:
        lines.append("# mismatches")
        for row in mismatches:
            lines.append(f"# REPRO {row.repro}")
            lines.append(f"#   rhwp dump-pages {row.doc} --json")
            lines.append(f"#   {row.note}")
    if report.unpaired:
        lines.append("# unpaired")
        for item in report.unpaired:
            lines.append(f"# UNPAIRED {item}")
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


def _configure_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        reconf = getattr(stream, "reconfigure", None)
        if callable(reconf):
            try:
                reconf(encoding="utf-8")
            except (OSError, ValueError):
                pass


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--repo", type=Path, default=None, help="저장소 루트 (기본: 이 파일 기준)")
    p.add_argument("--rhwp", default=None, help="rhwp 바이너리. 없으면 RHWP / PATH / target/release")
    p.add_argument("--manifest", type=Path, default=None, help="pairs JSON. M01-1 없이도 자체 픽스처 가능")
    p.add_argument("--docs", action="append", default=None, help="문서 루트 또는 파일. 반복 가능. 기본 samples/")
    p.add_argument(
        "--pdf-dirs",
        action="append",
        default=None,
        help="PDF 루트. 반복 가능. 기본 pdf/ pdf-2020/ pdf-large/",
    )
    p.add_argument("--pair", nargs=2, metavar=("DOC", "PDF"), help="단짝 재현")
    p.add_argument("--pdf-count", type=Path, default=None, help="PDF 쪽수만 출력 (렌더 없음)")
    p.add_argument("--write-manifest", type=Path, default=None, help="발견한 짝을 JSON 으로 저장")
    p.add_argument("--limit", type=int, default=None, help="앞에서 N 짝만 (전수 부담 줄이기)")
    p.add_argument("--timeout", type=float, default=180.0, help="dump-pages 초")
    p.add_argument("--strict", action="store_true", help="MISMATCH/ERROR 가 있으면 종료 1")
    p.add_argument("--json", action="store_true", dest="as_json", help="stdout 에 JSON 봉투")
    return p


def main(argv: Sequence[str] | None = None) -> int:
    _configure_stdio()
    args = build_parser().parse_args(list(argv) if argv is not None else None)
    repo = (args.repo or REPO_DEFAULT).resolve()

    if args.pdf_count is not None:
        try:
            n = pdf_page_count(args.pdf_count)
        except PageSmokeError as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            return 2
        print(n)
        return 0

    notes: list[str] = []
    pair_source = "glob"
    pairs: list[Pair] = []
    unpaired: list[str] = []

    if args.pair:
        doc = Path(args.pair[0])
        pdf = Path(args.pair[1])
        if not doc.is_absolute():
            doc = repo / doc
        if not pdf.is_absolute():
            pdf = repo / pdf
        pairs = [Pair(doc=doc, pdf=pdf, stem=doc.stem)]
        pair_source = "pair"
    elif args.manifest:
        try:
            pairs, unpaired = load_manifest(args.manifest, repo)
        except (OSError, json.JSONDecodeError, PageSmokeError) as exc:
            print(f"ERROR: 매니페스트: {exc}", file=sys.stderr)
            return 2
        pair_source = f"manifest:{relpath(args.manifest, repo)}"
    else:
        docs_roots = [repo / Path(d) if not Path(d).is_absolute() else Path(d) for d in (args.docs or DEFAULT_DOCS)]
        pdf_roots = [
            repo / Path(d) if not Path(d).is_absolute() else Path(d)
            for d in (args.pdf_dirs or DEFAULT_PDF_DIRS)
        ]
        missing_pdf = [str(p) for p in pdf_roots if not p.is_dir()]
        if missing_pdf:
            notes.append(
                "PDF 루트 없음: "
                + ", ".join(missing_pdf)
                + " — `git sparse-checkout add pdf pdf-2020 pdf-large` 후 전수 스윕"
            )
        pairs, unpaired = discover_pairs(repo, docs_roots=docs_roots, pdf_roots=pdf_roots)
        pair_source = "glob:pdf/{stem}.pdf|pdf/{stem}-*.pdf"

    if args.limit is not None:
        pairs = list(pairs)[: max(0, args.limit)]

    if args.write_manifest:
        write_manifest(args.write_manifest, pairs, repo, unpaired)
        notes.append(f"manifest written: {relpath(args.write_manifest, repo)}")

    rhwp = find_rhwp(args.rhwp, repo)
    if rhwp is None:
        notes.append("rhwp 없음 — 짝 발견·PDF 쪽수는 세고 dump-pages 는 ERROR 로 남긴다")

    if not pairs:
        notes.append("비교할 짝이 없다. --manifest 또는 --pair 또는 pdf/ 글롭을 확인")

    report = run_smoke(
        repo=repo,
        pairs=pairs,
        unpaired=unpaired,
        rhwp=rhwp,
        strict=bool(args.strict),
        pair_source=pair_source,
        timeout=float(args.timeout),
        notes=notes,
    )

    if args.as_json:
        print(json.dumps(report.to_json(), ensure_ascii=False, indent=2))
    else:
        sys.stdout.write(format_human(report))
    return exit_code(report)


if __name__ == "__main__":
    raise SystemExit(main())
