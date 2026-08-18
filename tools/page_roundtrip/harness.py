#!/usr/bin/env python3
"""페이지 수 왕복 공통 하네스 — pages(원본)==pages(export→reimport).

MEGA QUEUE M05-1 (#5367). 판정 도구만. #3518 #3521 #3737 #4056 #4882 #5128
은 고치지 않는다. `scripts/visual_sweep.py` · gym · DocumentCore/serializer
구현을 건드리지 않는다.

기존 CLI 가 쪽수를 잰다:

    rhwp export-hwpx <입력> <tmp.hwpx> --verify-pages --json
    rhwp convert     <입력> <tmp.hwp>  --verify-pages --json

판정은 데이터다. 기본 종료 코드는 불일치가 있어도 0. `--strict` 만
MISMATCH/ERROR/UNEXPECTED_PASS/CATALOG_MISSING 에서 1 을 낸다.
카탈로그 expected-fail 은 침묵 스킵하지 않는다.

사용:
    python tools/page_roundtrip/harness.py --ci
    python tools/page_roundtrip/harness.py --docs samples --limit 20
    python tools/page_roundtrip/harness.py --file samples/foo.hwp --route hwpx
    python tools/page_roundtrip/harness.py --docs samples --json

전수 스윕은 README Full sweep 절. CI 는
`python -m unittest tools.page_roundtrip.test_harness` (가짜 rhwp).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable, Sequence

SCHEMA_VERSION = 1
KIND = "pageRoundtripReport"
CATALOG_KIND = "pageRoundtripCatalog"
MANIFEST_KIND = "pageRoundtripManifest"
DOC_SUFFIXES = {".hwp", ".hwpx"}
ROUTES = ("hwpx", "hwp")
DEFAULT_DOCS = ("samples",)
VERIFY_FAIL_RE = re.compile(
    r"검증 실패\(--verify-pages\):\s*변환 전\s*(\d+)\s*쪽,\s*재파싱 후\s*(\d+)\s*쪽"
)
VERIFY_PASS_RE = re.compile(r"검증 통과\(--verify-pages\):\s*(\d+)\s*쪽")

HERE = Path(__file__).resolve().parent
REPO_DEFAULT = HERE.parents[1]
DEFAULT_CATALOG = HERE / "catalog.json"
DEFAULT_CI_MANIFEST = HERE / "fixtures" / "ci-subset.json"


class PageRoundtripError(Exception):
    """하네스 사용법·카탈로그 오류. 종료 코드 2."""


@dataclass(frozen=True)
class CatalogEntry:
    doc: str
    route: str
    issue: int | None = None
    reason: str = ""

    @property
    def key(self) -> tuple[str, str]:
        return (norm_rel(self.doc), self.route)


@dataclass(frozen=True)
class Job:
    doc: Path
    route: str
    rel: str = ""


@dataclass
class Row:
    doc: str
    route: str
    pages_before: int | None
    pages_after: int | None
    equal: bool | None
    verdict: str
    issue: int | None = None
    note: str = ""
    repro: str = ""
    rhwp_rc: int | None = None

    def to_json(self) -> dict[str, Any]:
        return {
            "doc": self.doc,
            "route": self.route,
            "pagesBefore": self.pages_before,
            "pagesAfter": self.pages_after,
            "equal": self.equal,
            "verdict": self.verdict,
            "issue": self.issue,
            "note": self.note,
            "repro": self.repro,
            "rhwpRc": self.rhwp_rc,
        }


@dataclass
class CatalogStatus:
    doc: str
    route: str
    issue: int | None
    reason: str
    state: str
    verdict: str = ""

    def to_json(self) -> dict[str, Any]:
        return {
            "doc": self.doc,
            "route": self.route,
            "issue": self.issue,
            "reason": self.reason,
            "state": self.state,
            "verdict": self.verdict,
        }


@dataclass
class Report:
    schema_version: int = SCHEMA_VERSION
    kind: str = KIND
    strict: bool = False
    rhwp: str = ""
    route: str = "hwpx"
    source: str = "glob"
    rows: list[Row] = field(default_factory=list)
    catalog: list[CatalogStatus] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    @property
    def summary(self) -> dict[str, int]:
        counts = {
            "jobs": 0,
            "match": 0,
            "mismatch": 0,
            "expected_fail": 0,
            "unexpected_pass": 0,
            "error": 0,
            "catalog_missing": 0,
            "catalog_held": 0,
        }
        for row in self.rows:
            counts["jobs"] += 1
            key = row.verdict.lower()
            if key in counts:
                counts[key] += 1
        for item in self.catalog:
            if item.state == "held":
                counts["catalog_held"] += 1
            elif item.state == "missing" and item.verdict == "CATALOG_MISSING":
                # 행으로도 세었으면 jobs 쪽에 이미 있다. held/missing 만 별도.
                pass
        return counts

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": self.schema_version,
            "kind": self.kind,
            "strict": self.strict,
            "rhwp": self.rhwp,
            "route": self.route,
            "source": self.source,
            "summary": self.summary,
            "rows": [r.to_json() for r in self.rows],
            "mismatches": [r.to_json() for r in self.rows if r.verdict == "MISMATCH"],
            "expectedFails": [r.to_json() for r in self.rows if r.verdict == "EXPECTED_FAIL"],
            "unexpectedPasses": [r.to_json() for r in self.rows if r.verdict == "UNEXPECTED_PASS"],
            "errors": [r.to_json() for r in self.rows if r.verdict == "ERROR"],
            "catalog": [c.to_json() for c in self.catalog],
            "notes": list(self.notes),
        }


def norm_rel(path: str) -> str:
    return path.replace("\\", "/").lstrip("./")


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


def expand_routes(route: str) -> list[str]:
    if route == "both":
        return list(ROUTES)
    if route in ROUTES:
        return [route]
    raise PageRoundtripError(f"알 수 없는 --route: {route} (hwpx|hwp|both)")


def repro_command(doc: str, route: str) -> str:
    return f"python tools/page_roundtrip/harness.py --file {doc} --route {route}"


def classify(
    equal: bool | None,
    *,
    cataloged: bool,
    error: bool = False,
    missing: bool = False,
) -> str:
    """쪽수 비교 + 카탈로그 → 기계 판정. 침묵 스킵 없음."""
    if missing:
        return "CATALOG_MISSING"
    if error or equal is None:
        return "ERROR"
    if cataloged and equal:
        return "UNEXPECTED_PASS"
    if cataloged and not equal:
        return "EXPECTED_FAIL"
    if equal:
        return "MATCH"
    return "MISMATCH"


def parse_verify_pages(stdout: str, stderr: str) -> tuple[int, int] | None:
    """export-hwpx/convert --verify-pages --json 봉투 또는 텍스트에서 쪽수를 읽는다."""
    text = (stdout or "").strip()
    blob = text
    if blob and not blob.startswith("{"):
        brace = blob.find("{")
        if brace >= 0:
            blob = blob[brace:]
    if blob.startswith("{"):
        try:
            obj = json.loads(blob)
        except json.JSONDecodeError:
            obj = None
        if isinstance(obj, dict):
            vp = obj.get("verifyPages")
            if isinstance(vp, dict) and "before" in vp and "after" in vp:
                try:
                    return int(vp["before"]), int(vp["after"])
                except (TypeError, ValueError):
                    pass

    combined = "\n".join(part for part in (stdout, stderr) if part)
    fail = VERIFY_FAIL_RE.search(combined)
    if fail:
        return int(fail.group(1)), int(fail.group(2))
    passed = VERIFY_PASS_RE.search(combined)
    if passed:
        n = int(passed.group(1))
        return n, n
    return None


def load_catalog(path: Path) -> list[CatalogEntry]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise PageRoundtripError(f"카탈로그가 객체가 아니다: {path}")
    items = raw.get("entries") or raw.get("expectedFail") or raw.get("items") or []
    if not isinstance(items, list):
        raise PageRoundtripError(f"카탈로그 entries 가 배열이 아니다: {path}")
    entries: list[CatalogEntry] = []
    seen: set[tuple[str, str]] = set()
    for item in items:
        if not isinstance(item, dict):
            continue
        doc = item.get("doc") or item.get("sample") or item.get("path")
        if not doc:
            continue
        route = str(item.get("route") or "hwpx").lower()
        if route not in ROUTES:
            raise PageRoundtripError(f"카탈로그 route 가 잘못됐다: {route} ({doc})")
        issue_raw = item.get("issue")
        issue: int | None
        try:
            issue = int(issue_raw) if issue_raw is not None else None
        except (TypeError, ValueError) as exc:
            raise PageRoundtripError(f"카탈로그 issue 가 숫자가 아니다: {issue_raw}") from exc
        entry = CatalogEntry(
            doc=norm_rel(str(doc)),
            route=route,
            issue=issue,
            reason=str(item.get("reason") or ""),
        )
        if entry.key in seen:
            continue
        seen.add(entry.key)
        entries.append(entry)
    return entries


def load_manifest(path: Path, repo: Path) -> list[Path]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    items: Iterable[Any]
    if isinstance(raw, list):
        items = raw
    elif isinstance(raw, dict):
        items = raw.get("docs") or raw.get("files") or raw.get("items") or []
    else:
        raise PageRoundtripError(f"매니페스트 형식이 아니다: {path}")
    docs: list[Path] = []
    seen: set[str] = set()
    for item in items:
        if isinstance(item, str):
            rel = item
        elif isinstance(item, dict):
            rel = item.get("doc") or item.get("sample") or item.get("path")
        else:
            continue
        if not rel:
            continue
        doc = Path(str(rel))
        if not doc.is_absolute():
            doc = repo / doc
        key = str(doc.resolve()).casefold()
        if key in seen:
            continue
        seen.add(key)
        docs.append(doc)
    return docs


def write_manifest(path: Path, docs: Sequence[Path], repo: Path) -> None:
    payload = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": MANIFEST_KIND,
        "docs": [relpath(d, repo) for d in docs],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def iter_docs(roots: Sequence[Path]) -> list[Path]:
    docs: list[Path] = []
    seen: set[str] = set()
    for root in roots:
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


def catalog_lookup(catalog: Sequence[CatalogEntry]) -> dict[tuple[str, str], CatalogEntry]:
    return {entry.key: entry for entry in catalog}


def run_job(
    job: Job,
    *,
    repo: Path,
    rhwp: Path | str | None,
    catalog: dict[tuple[str, str], CatalogEntry],
    out_dir: Path,
    timeout: float,
    runner: Any = None,
) -> Row:
    rel = job.rel or relpath(job.doc, repo)
    repro = repro_command(rel, job.route)
    cat = catalog.get((norm_rel(rel), job.route))
    issue = cat.issue if cat else None
    if not job.doc.is_file():
        return Row(
            doc=rel,
            route=job.route,
            pages_before=None,
            pages_after=None,
            equal=None,
            verdict=classify(None, cataloged=cat is not None, missing=True),
            issue=issue,
            note="파일을 찾을 수 없다",
            repro=repro,
        )
    if rhwp is None:
        return Row(
            doc=rel,
            route=job.route,
            pages_before=None,
            pages_after=None,
            equal=None,
            verdict=classify(None, cataloged=cat is not None, error=True),
            issue=issue,
            note="rhwp 바이너리를 찾지 못했다 (RHWP/--rhwp/target/release)",
            repro=repro,
        )

    ext = ".hwpx" if job.route == "hwpx" else ".hwp"
    out_path = out_dir / f"{job.doc.stem}.{job.route}.rt{ext}"
    if job.route == "hwpx":
        cmd = [str(rhwp), "export-hwpx", str(job.doc), str(out_path), "--verify-pages", "--json"]
    else:
        cmd = [str(rhwp), "convert", str(job.doc), str(out_path), "--verify-pages", "--json"]

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
        return Row(
            doc=rel,
            route=job.route,
            pages_before=None,
            pages_after=None,
            equal=None,
            verdict=classify(None, cataloged=cat is not None, error=True),
            issue=issue,
            note=f"timeout {timeout}s",
            repro=repro,
        )
    except OSError as exc:
        return Row(
            doc=rel,
            route=job.route,
            pages_before=None,
            pages_after=None,
            equal=None,
            verdict=classify(None, cataloged=cat is not None, error=True),
            issue=issue,
            note=f"실행 실패: {exc}",
            repro=repro,
        )

    rc = getattr(proc, "returncode", 1)
    parsed = parse_verify_pages(getattr(proc, "stdout", "") or "", getattr(proc, "stderr", "") or "")
    if parsed is None:
        err = (getattr(proc, "stderr", "") or "").strip().splitlines()
        tail = err[-1] if err else f"rc={rc}"
        return Row(
            doc=rel,
            route=job.route,
            pages_before=None,
            pages_after=None,
            equal=None,
            verdict=classify(None, cataloged=cat is not None, error=True),
            issue=issue,
            note=f"verify-pages 파싱 실패 rc={rc}: {tail}",
            repro=repro,
            rhwp_rc=rc,
        )

    before, after = parsed
    equal = before == after
    verdict = classify(equal, cataloged=cat is not None)
    note = ""
    if verdict != "MATCH":
        note = f"before={before} after={after}"
        if cat and cat.reason:
            note = f"{note}; {cat.reason}"
    return Row(
        doc=rel,
        route=job.route,
        pages_before=before,
        pages_after=after,
        equal=equal,
        verdict=verdict,
        issue=issue,
        note=note,
        repro=repro,
        rhwp_rc=rc,
    )


def build_jobs(
    docs: Sequence[Path],
    repo: Path,
    routes: Sequence[str],
) -> list[Job]:
    jobs: list[Job] = []
    for doc in docs:
        rel = relpath(doc, repo)
        for route in routes:
            jobs.append(Job(doc=doc, route=route, rel=rel))
    return jobs


def attach_catalog_status(
    report: Report,
    catalog: Sequence[CatalogEntry],
    ran_keys: set[tuple[str, str]],
    repo: Path,
    *,
    require_missing_rows: bool = False,
) -> None:
    """카탈로그 전항을 리포트에 남긴다. 실행 밖은 held, 파일 없음은 missing.

    전수 스윕(`require_missing_rows`)에서만 빠진 카탈로그 파일을 행으로 올린다.
    --limit/--ci 는 catalog[].state=held|missing 으로만 남기고 침묵 삭제는 하지 않는다.
    """
    row_by_key = {(norm_rel(r.doc), r.route): r for r in report.rows}
    for entry in catalog:
        key = entry.key
        if key in ran_keys:
            row = row_by_key.get(key)
            report.catalog.append(
                CatalogStatus(
                    doc=entry.doc,
                    route=entry.route,
                    issue=entry.issue,
                    reason=entry.reason,
                    state="run",
                    verdict=row.verdict if row else "",
                )
            )
            continue
        disk = repo / entry.doc
        if not disk.is_file():
            report.catalog.append(
                CatalogStatus(
                    doc=entry.doc,
                    route=entry.route,
                    issue=entry.issue,
                    reason=entry.reason,
                    state="missing",
                    verdict="CATALOG_MISSING",
                )
            )
            if require_missing_rows and key not in row_by_key:
                report.rows.append(
                    Row(
                        doc=entry.doc,
                        route=entry.route,
                        pages_before=None,
                        pages_after=None,
                        equal=None,
                        verdict="CATALOG_MISSING",
                        issue=entry.issue,
                        note="카탈로그 경로의 파일이 없다 (침묵 스킵 금지)",
                        repro=repro_command(entry.doc, entry.route),
                    )
                )
            continue
        report.catalog.append(
            CatalogStatus(
                doc=entry.doc,
                route=entry.route,
                issue=entry.issue,
                reason=entry.reason,
                state="held",
                verdict="",
            )
        )


def run_harness(
    *,
    repo: Path,
    docs: Sequence[Path],
    routes: Sequence[str],
    catalog: Sequence[CatalogEntry],
    rhwp: Path | str | None,
    strict: bool,
    source: str,
    timeout: float = 180.0,
    out_dir: Path | None = None,
    runner: Any = None,
    notes: Sequence[str] | None = None,
    emit_missing_catalog: bool = True,
    require_missing_rows: bool | None = None,
) -> Report:
    report = Report(
        strict=strict,
        rhwp=str(rhwp) if rhwp else "",
        route=",".join(routes),
        source=source,
        notes=list(notes or ()),
    )
    cat_map = catalog_lookup(catalog)
    jobs = build_jobs(docs, repo, routes)
    ran_keys: set[tuple[str, str]] = set()

    tmp_owned = None
    work_dir = out_dir
    if work_dir is None:
        tmp_owned = tempfile.TemporaryDirectory(prefix="rhwp-page-rt-")
        work_dir = Path(tmp_owned.name)
    else:
        work_dir.mkdir(parents=True, exist_ok=True)

    try:
        for job in jobs:
            ran_keys.add((norm_rel(job.rel or relpath(job.doc, repo)), job.route))
            report.rows.append(
                run_job(
                    job,
                    repo=repo,
                    rhwp=rhwp,
                    catalog=cat_map,
                    out_dir=work_dir,
                    timeout=timeout,
                    runner=runner,
                )
            )
    finally:
        if tmp_owned is not None:
            tmp_owned.cleanup()

    if emit_missing_catalog:
        if require_missing_rows is None:
            require_missing_rows = source.startswith("glob")
        attach_catalog_status(
            report,
            catalog,
            ran_keys,
            repo,
            require_missing_rows=require_missing_rows,
        )
    return report


def format_human(report: Report) -> str:
    summary = report.summary
    lines = [
        f"# page-roundtrip jobs={summary['jobs']} match={summary['match']} "
        f"mismatch={summary['mismatch']} expected_fail={summary['expected_fail']} "
        f"unexpected_pass={summary['unexpected_pass']} error={summary['error']} "
        f"catalog_missing={summary['catalog_missing']} catalog_held={summary['catalog_held']}",
        "# 판정은 데이터다. 기본 종료 코드 0. --strict 이면 신규 위반/ERROR/UNEXPECTED_PASS/CATALOG_MISSING 시 1.",
        "doc\troute\tbefore\tafter\tequal\tverdict\tissue\trepro",
    ]
    for row in report.rows:
        before = "" if row.pages_before is None else str(row.pages_before)
        after = "" if row.pages_after is None else str(row.pages_after)
        equal = "" if row.equal is None else ("yes" if row.equal else "no")
        issue = "" if row.issue is None else f"#{row.issue}"
        lines.append(
            f"{row.doc}\t{row.route}\t{before}\t{after}\t{equal}\t{row.verdict}\t{issue}\t{row.repro}"
        )
    new_violations = [r for r in report.rows if r.verdict == "MISMATCH"]
    if new_violations:
        lines.append("# new-violations")
        for row in new_violations:
            lines.append(f"# REPRO {row.repro}")
            lines.append(f"#   {row.note}")
    if report.catalog:
        lines.append("# catalog")
        for item in report.catalog:
            issue = "" if item.issue is None else f"#{item.issue}"
            extra = item.verdict or item.state
            lines.append(f"# CATALOG {item.state}\t{item.doc}\t{item.route}\t{issue}\t{extra}")
    for note in report.notes:
        lines.append(f"# note: {note}")
    return "\n".join(lines) + "\n"


def exit_code(report: Report) -> int:
    if not report.strict:
        return 0
    summary = report.summary
    if (
        summary["mismatch"]
        or summary["error"]
        or summary["unexpected_pass"]
        or summary["catalog_missing"]
    ):
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
    p.add_argument("--docs", action="append", default=None, help="문서 루트 또는 파일. 반복 가능. 기본 samples/")
    p.add_argument("--file", action="append", default=None, help="단건 재현. 반복 가능")
    p.add_argument("--manifest", type=Path, default=None, help="문서 목록 JSON (CI 부분집합)")
    p.add_argument("--ci", action="store_true", help="fixtures/ci-subset.json 으로 돌린다")
    p.add_argument("--catalog", type=Path, default=None, help="expected-fail 카탈로그 JSON")
    p.add_argument("--no-catalog", action="store_true", help="카탈로그를 비운다 (시험용)")
    p.add_argument(
        "--route",
        default="hwpx",
        choices=("hwpx", "hwp", "both"),
        help="내보내기 경로. 기본 hwpx (이슈 가족과 동일)",
    )
    p.add_argument("--limit", type=int, default=None, help="앞에서 N 문서만 (전수 부담 줄이기)")
    p.add_argument("--timeout", type=float, default=180.0, help="문서 하나 초")
    p.add_argument("--out-dir", type=Path, default=None, help="왕복 산출물을 남길 폴더")
    p.add_argument("--write-manifest", type=Path, default=None, help="선택한 문서 목록을 JSON 으로 저장")
    p.add_argument("--strict", action="store_true", help="신규 위반/ERROR/UNEXPECTED_PASS/CATALOG_MISSING 이면 종료 1")
    p.add_argument("--json", action="store_true", dest="as_json", help="stdout 에 JSON 봉투")
    return p


def main(argv: Sequence[str] | None = None) -> int:
    _configure_stdio()
    args = build_parser().parse_args(list(argv) if argv is not None else None)
    repo = (args.repo or REPO_DEFAULT).resolve()

    notes: list[str] = []
    source = "glob"
    docs: list[Path] = []

    try:
        routes = expand_routes(args.route)
        if args.no_catalog:
            catalog: list[CatalogEntry] = []
        else:
            catalog_path = args.catalog or DEFAULT_CATALOG
            if not catalog_path.is_absolute():
                catalog_path = (repo / catalog_path) if (repo / catalog_path).is_file() else catalog_path
            if not catalog_path.is_file():
                raise PageRoundtripError(f"카탈로그가 없다: {catalog_path}")
            catalog = load_catalog(catalog_path)
            notes.append(f"catalog={relpath(catalog_path, repo)} entries={len(catalog)}")

        if args.file:
            for item in args.file:
                path = Path(item)
                if not path.is_absolute():
                    path = repo / path
                docs.append(path)
            source = "file"
        elif args.ci:
            man = DEFAULT_CI_MANIFEST
            docs = load_manifest(man, repo)
            source = f"ci:{relpath(man, repo)}"
        elif args.manifest:
            man = args.manifest
            if not man.is_absolute():
                man = repo / man
            docs = load_manifest(man, repo)
            source = f"manifest:{relpath(man, repo)}"
        else:
            roots = [repo / Path(d) if not Path(d).is_absolute() else Path(d) for d in (args.docs or DEFAULT_DOCS)]
            docs = iter_docs(roots)
            source = "glob:" + ",".join(relpath(r, repo) for r in roots)
    except (OSError, json.JSONDecodeError, PageRoundtripError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    if args.limit is not None:
        docs = list(docs)[: max(0, args.limit)]

    if args.write_manifest:
        write_manifest(args.write_manifest, docs, repo)
        notes.append(f"manifest written: {relpath(args.write_manifest, repo)}")

    rhwp = find_rhwp(args.rhwp, repo)
    if rhwp is None:
        notes.append("rhwp 없음 — 행은 ERROR 로 남긴다")
    if not docs:
        notes.append("비교할 문서가 없다. --ci / --manifest / --file / --docs 를 확인")

    report = run_harness(
        repo=repo,
        docs=docs,
        routes=routes,
        catalog=catalog,
        rhwp=rhwp,
        strict=bool(args.strict),
        source=source,
        timeout=float(args.timeout),
        out_dir=args.out_dir,
        notes=notes,
    )

    if args.as_json:
        print(json.dumps(report.to_json(), ensure_ascii=False, indent=2))
    else:
        sys.stdout.write(format_human(report))
    return exit_code(report)


if __name__ == "__main__":
    raise SystemExit(main())
