#!/usr/bin/env python3
"""samples/ 전수 layout-anomaly 배치 리포트 (M02-8).

devel CLI 는 단건만 있다 (`rhwp layout-anomaly <파일> --json`).
#5371 의 `--batch` 는 이 스크립트가 기다리지 않는다. `.hwp`/`.hwpx` 를
재귀 수집한 뒤 파일마다 같은 명령을 돌린다.

카운트: overflow, overlap, empty_page.
바이너리가 내면 text-overlap (`textOverlapCount`) / off-canvas
(`offCanvasCount`) 도 집계한다. devel 에는 아직 없다 (#5379, #5389).

엔진·레이아웃을 고치지 않는다. gym/ 와 scripts/visual_sweep.py 는 읽지 않는다.
판정은 데이터다. 기본 종료 코드 0. `--strict` 만 ERROR/TIMEOUT 에서 1.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
import unicodedata
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable, Sequence

SCHEMA_VERSION = "1.0"
KIND = "layoutAnomalyBatchReport"
GENERATOR = "tools/layout_anomaly_batch_report.py"
CLAIM_ID = "M02-8"
ISSUE_ID = 5390
SAMPLE_EXTS = {".hwp", ".hwpx"}
DEFAULT_TIMEOUT_SEC = 180.0
DEFAULT_TOP = 20
DEFAULT_JOBS = 1
REPO_DEFAULT = Path(__file__).resolve().parents[1]
CLI_CONTRACT = "per-file: rhwp layout-anomaly <file> --json (devel; --batch is #5371)"

COUNT_KEYS = (
    "overflow",
    "overlap",
    "empty_page",
    "off_canvas",
    "text_overlap",
)
OPTIONAL_COUNT_FIELDS = {
    "off_canvas": ("offCanvasCount", "off_canvas_count"),
    "text_overlap": ("textOverlapCount", "text_overlap_count"),
}
REQUIRED_COUNT_FIELDS = {
    "overflow": ("overflowCount", "overflow_count"),
    "overlap": ("overlapCount", "overlap_count"),
    "empty_page": ("emptyPageCount", "empty_page_count"),
}


class UsageError(Exception):
    """CLI 사용법·경로 오류."""


@dataclass
class FileRow:
    path: str
    status: str
    page_count: int = 0
    overflow: int = 0
    overlap: int = 0
    empty_page: int = 0
    off_canvas: int | None = None
    text_overlap: int | None = None
    has_signal: bool = False
    elapsed_ms: int = 0
    error: str = ""
    exit_code: int | None = None

    @property
    def total_signals(self) -> int:
        total = self.overflow + self.overlap + self.empty_page
        if self.off_canvas:
            total += self.off_canvas
        if self.text_overlap:
            total += self.text_overlap
        return total

    def to_json(self) -> dict[str, Any]:
        return {
            "path": self.path,
            "status": self.status,
            "pageCount": self.page_count,
            "overflow": self.overflow,
            "overlap": self.overlap,
            "emptyPage": self.empty_page,
            "offCanvas": self.off_canvas,
            "textOverlap": self.text_overlap,
            "hasSignal": self.has_signal,
            "elapsedMs": self.elapsed_ms,
            "error": self.error or None,
            "exitCode": self.exit_code,
        }


@dataclass
class Report:
    rows: list[FileRow]
    binary_path: str
    binary_version: str
    supports_batch: bool
    supports_off_canvas: bool | None
    supports_text_overlap: bool | None
    git_commit: str
    git_branch: str
    root: str
    file_count: int
    limit: int | None
    timeout_sec: float
    jobs: int
    top_n: int
    started_at: str
    finished_at: str
    notes: list[str] = field(default_factory=list)
    strict: bool = False

    def summary(self) -> dict[str, Any]:
        scanned = len(self.rows)
        clean = sum(1 for r in self.rows if r.status == "CLEAN")
        anomaly = sum(1 for r in self.rows if r.status == "ANOMALY")
        error = sum(1 for r in self.rows if r.status == "ERROR")
        timeout = sum(1 for r in self.rows if r.status == "TIMEOUT")
        return {
            "scanned": scanned,
            "clean": clean,
            "anomaly": anomaly,
            "error": error,
            "timeout": timeout,
            "overflowFiles": sum(1 for r in self.rows if r.overflow > 0),
            "overlapFiles": sum(1 for r in self.rows if r.overlap > 0),
            "emptyPageFiles": sum(1 for r in self.rows if r.empty_page > 0),
            "offCanvasFiles": (
                sum(1 for r in self.rows if (r.off_canvas or 0) > 0)
                if self.supports_off_canvas
                else None
            ),
            "textOverlapFiles": (
                sum(1 for r in self.rows if (r.text_overlap or 0) > 0)
                if self.supports_text_overlap
                else None
            ),
            "overflowCount": sum(r.overflow for r in self.rows),
            "overlapCount": sum(r.overlap for r in self.rows),
            "emptyPageCount": sum(r.empty_page for r in self.rows),
            "offCanvasCount": (
                sum(r.off_canvas or 0 for r in self.rows)
                if self.supports_off_canvas
                else None
            ),
            "textOverlapCount": (
                sum(r.text_overlap or 0 for r in self.rows)
                if self.supports_text_overlap
                else None
            ),
        }

    def top_offenders(self) -> dict[str, list[dict[str, Any]]]:
        return {
            "overflow": top_by(self.rows, lambda r: r.overflow, self.top_n),
            "overlap": top_by(self.rows, lambda r: r.overlap, self.top_n),
            "emptyPage": top_by(self.rows, lambda r: r.empty_page, self.top_n),
            "offCanvas": (
                top_by(self.rows, lambda r: r.off_canvas or 0, self.top_n)
                if self.supports_off_canvas
                else []
            ),
            "textOverlap": (
                top_by(self.rows, lambda r: r.text_overlap or 0, self.top_n)
                if self.supports_text_overlap
                else []
            ),
            "totalSignals": top_by(self.rows, lambda r: r.total_signals, self.top_n),
        }

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": SCHEMA_VERSION,
            "kind": KIND,
            "claimId": CLAIM_ID,
            "issue": ISSUE_ID,
            "generator": GENERATOR,
            "cliContract": CLI_CONTRACT,
            "binary": {
                "path": self.binary_path,
                "version": self.binary_version,
                "supportsBatch": self.supports_batch,
                "supportsOffCanvas": self.supports_off_canvas,
                "supportsTextOverlap": self.supports_text_overlap,
            },
            "git": {"commit": self.git_commit, "branch": self.git_branch},
            "input": {
                "root": self.root,
                "fileCount": self.file_count,
                "limit": self.limit,
                "timeoutSec": self.timeout_sec,
                "jobs": self.jobs,
            },
            "startedAt": self.started_at,
            "finishedAt": self.finished_at,
            "summary": self.summary(),
            "topOffenders": self.top_offenders(),
            "rows": [r.to_json() for r in self.rows],
            "notes": list(self.notes),
        }


def nfc(value: str) -> str:
    return unicodedata.normalize("NFC", value)


def sanitize_error(text: str, repo: Path) -> str:
    if not text:
        return text
    roots = {str(repo), str(repo.resolve())}
    out = text
    for root in roots:
        out = out.replace(root + "\\", "").replace(root + "/", "")
        out = out.replace(root, "")
    return out


def posix_rel(path: Path, root: Path) -> str:
    try:
        rel = path.resolve().relative_to(root.resolve())
    except ValueError:
        return nfc(path.as_posix())
    return nfc(rel.as_posix())


def iso_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def discover_repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "Cargo.toml").is_file() and (parent / "samples").is_dir():
            return parent
    return REPO_DEFAULT


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
        Path("target") / "release" / "rhwp.exe",
        Path("target") / "release" / "rhwp",
        Path("target") / "release-test" / "rhwp.exe",
        Path("target") / "release-test" / "rhwp",
        Path("target") / "debug" / "rhwp.exe",
        Path("target") / "debug" / "rhwp",
    ):
        cand = root / rel
        if cand.is_file():
            return cand
    return None


def walk_docs(root: Path, repo: Path) -> list[Path]:
    if not root.is_dir():
        raise UsageError(f"입력 폴더가 없습니다: {root}")
    files: list[Path] = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix.lower() not in SAMPLE_EXTS:
            continue
        if any(part.startswith(".") for part in path.parts):
            continue
        files.append(path)
    files.sort(key=lambda p: posix_rel(p, repo).casefold())
    return files


def first_int(obj: dict[str, Any], keys: Sequence[str]) -> int | None:
    for key in keys:
        if key in obj and obj[key] is not None:
            try:
                return int(obj[key])
            except (TypeError, ValueError):
                return None
    return None


def extract_json_object(text: str) -> dict[str, Any]:
    start = text.find("{")
    end = text.rfind("}")
    if start < 0 or end <= start:
        raise ValueError("JSON object 없음")
    raw = json.loads(text[start : end + 1])
    if not isinstance(raw, dict):
        raise ValueError("JSON object 가 아님")
    return raw


def parse_envelope(payload: dict[str, Any]) -> dict[str, Any]:
    """단건 `--json` 봉투에서 카운트만 꺼낸다. pages[] 는 버린다."""
    overflow = first_int(payload, REQUIRED_COUNT_FIELDS["overflow"])
    overlap = first_int(payload, REQUIRED_COUNT_FIELDS["overlap"])
    empty_page = first_int(payload, REQUIRED_COUNT_FIELDS["empty_page"])
    if overflow is None or overlap is None or empty_page is None:
        raise ValueError("overflowCount/overlapCount/emptyPageCount 없음")
    off_canvas = first_int(payload, OPTIONAL_COUNT_FIELDS["off_canvas"])
    text_overlap = first_int(payload, OPTIONAL_COUNT_FIELDS["text_overlap"])
    page_count = first_int(payload, ("pageCount", "page_count")) or 0
    has_signal = payload.get("hasSignal")
    if not isinstance(has_signal, bool):
        has_signal = overflow > 0 or overlap > 0
    error = payload.get("error")
    return {
        "page_count": page_count,
        "overflow": overflow,
        "overlap": overlap,
        "empty_page": empty_page,
        "off_canvas": off_canvas,
        "text_overlap": text_overlap,
        "has_signal": bool(has_signal),
        "error": str(error) if error else "",
        "has_off_canvas_field": any(k in payload for k in OPTIONAL_COUNT_FIELDS["off_canvas"]),
        "has_text_overlap_field": any(
            k in payload for k in OPTIONAL_COUNT_FIELDS["text_overlap"]
        ),
    }


def top_by(
    rows: Sequence[FileRow], score: Any, limit: int
) -> list[dict[str, Any]]:
    ranked = [r for r in rows if score(r) > 0]
    ranked.sort(key=lambda r: (-int(score(r)), r.path))
    out = []
    for row in ranked[: max(0, limit)]:
        item = row.to_json()
        item["score"] = int(score(row))
        out.append(item)
    return out


def run_cmd(
    cmd: Sequence[str],
    *,
    timeout: float,
    cwd: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(cmd),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
        cwd=cwd,
        check=False,
    )


def probe_binary(rhwp: Path, timeout: float = 30.0) -> tuple[str, bool]:
    version = ""
    try:
        proc = run_cmd([str(rhwp), "--version"], timeout=timeout)
        version = (proc.stdout or proc.stderr or "").strip().splitlines()[0] if (proc.stdout or proc.stderr) else ""
    except (OSError, subprocess.TimeoutExpired):
        version = ""
    supports_batch = False
    try:
        proc = run_cmd([str(rhwp), "layout-anomaly"], timeout=timeout)
        blob = f"{proc.stdout}\n{proc.stderr}"
        supports_batch = "--batch" in blob
    except (OSError, subprocess.TimeoutExpired):
        supports_batch = False
    return version, supports_batch


def git_identity(repo: Path) -> tuple[str, str]:
    commit = ""
    branch = ""
    try:
        proc = run_cmd(["git", "rev-parse", "HEAD"], timeout=15.0, cwd=repo)
        if proc.returncode == 0:
            commit = proc.stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        commit = ""
    try:
        proc = run_cmd(["git", "rev-parse", "--abbrev-ref", "HEAD"], timeout=15.0, cwd=repo)
        if proc.returncode == 0:
            branch = proc.stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        branch = ""
    return commit, branch


def run_one(
    rhwp: Path,
    path: Path,
    repo: Path,
    timeout_sec: float,
    retries: int = 2,
) -> FileRow:
    rel = posix_rel(path, repo)
    cmd = [str(rhwp), "layout-anomaly", str(path.resolve()), "--json"]
    started = time.perf_counter()
    last_os: OSError | None = None
    attempts = max(1, retries + 1)
    proc: subprocess.CompletedProcess[str] | None = None
    for attempt in range(attempts):
        try:
            proc = run_cmd(cmd, timeout=timeout_sec, cwd=repo)
            last_os = None
            break
        except subprocess.TimeoutExpired:
            elapsed = int((time.perf_counter() - started) * 1000)
            return FileRow(
                path=rel,
                status="TIMEOUT",
                elapsed_ms=elapsed,
                error=f"timeout {timeout_sec:.0f}s",
            )
        except OSError as exc:
            last_os = exc
            time.sleep(0.15 * (attempt + 1))
    if proc is None:
        elapsed = int((time.perf_counter() - started) * 1000)
        return FileRow(
            path=rel,
            status="ERROR",
            elapsed_ms=elapsed,
            error=str(last_os) if last_os else "spawn failed",
        )
    elapsed = int((time.perf_counter() - started) * 1000)
    stdout = proc.stdout or ""
    stderr = (proc.stderr or "").strip()
    if stdout.strip():
        try:
            payload = extract_json_object(stdout)
            parsed = parse_envelope(payload)
            error = parsed["error"] or (stderr if proc.returncode not in (0, 3) else "")
            if error and proc.returncode not in (0, 3):
                status = "ERROR"
            elif parsed["has_signal"] or parsed["empty_page"] > 0:
                status = "ANOMALY"
            else:
                status = "CLEAN"
            return FileRow(
                path=rel,
                status=status,
                page_count=parsed["page_count"],
                overflow=parsed["overflow"],
                overlap=parsed["overlap"],
                empty_page=parsed["empty_page"],
                off_canvas=parsed["off_canvas"],
                text_overlap=parsed["text_overlap"],
                has_signal=parsed["has_signal"],
                elapsed_ms=elapsed,
                error=error,
                exit_code=proc.returncode,
            )
        except (ValueError, json.JSONDecodeError) as exc:
            return FileRow(
                path=rel,
                status="ERROR",
                elapsed_ms=elapsed,
                error=f"json: {exc}",
                exit_code=proc.returncode,
            )
    msg = stderr or f"exit {proc.returncode}, empty stdout"
    return FileRow(
        path=rel,
        status="ERROR",
        elapsed_ms=elapsed,
        error=sanitize_error(msg[:500], repo),
        exit_code=proc.returncode,
    )


def infer_optional_support(rows: Sequence[FileRow], attr: str) -> bool | None:
    seen = False
    for row in rows:
        if row.status in {"ERROR", "TIMEOUT", "SKIP"}:
            continue
        value = getattr(row, attr)
        if value is not None:
            return True
        seen = True
    if seen:
        return False
    return None


def scan_files(
    rhwp: Path,
    files: Sequence[Path],
    repo: Path,
    timeout_sec: float,
    jobs: int,
    progress: bool = True,
) -> list[FileRow]:
    rows: list[FileRow] = []
    total = len(files)
    if total == 0:
        return rows
    workers = max(1, jobs)
    if workers == 1:
        for index, path in enumerate(files, start=1):
            row = run_one(rhwp, path, repo, timeout_sec)
            rows.append(row)
            if progress:
                sys.stderr.write(
                    f"[{index}/{total}] {row.status:8} ovr={row.overflow} "
                    f"olp={row.overlap} empty={row.empty_page} {row.path}\n"
                )
        return rows

    with ThreadPoolExecutor(max_workers=workers) as pool:
        future_map = {
            pool.submit(run_one, rhwp, path, repo, timeout_sec): path for path in files
        }
        done = 0
        for future in as_completed(future_map):
            row = future.result()
            rows.append(row)
            done += 1
            if progress:
                sys.stderr.write(
                    f"[{done}/{total}] {row.status:8} ovr={row.overflow} "
                    f"olp={row.overlap} empty={row.empty_page} {row.path}\n"
                )
    rows.sort(key=lambda r: r.path.casefold())
    return rows


def tsv_header() -> str:
    return (
        "path\tstatus\tpage_count\toverflow\toverlap\tempty_page\t"
        "off_canvas\ttext_overlap\thas_signal\telapsed_ms\terror"
    )


def format_cell(value: Any) -> str:
    if value is None:
        return ""
    text = str(value).replace("\t", " ").replace("\n", " ").replace("\r", "")
    return text


def format_tsv(rows: Sequence[FileRow]) -> str:
    lines = [tsv_header()]
    for row in rows:
        lines.append(
            "\t".join(
                format_cell(v)
                for v in (
                    row.path,
                    row.status,
                    row.page_count,
                    row.overflow,
                    row.overlap,
                    row.empty_page,
                    row.off_canvas,
                    row.text_overlap,
                    int(row.has_signal),
                    row.elapsed_ms,
                    row.error,
                )
            )
        )
    return "\n".join(lines) + "\n"


def _md_table(rows: Sequence[dict[str, Any]], count_key: str) -> str:
    if not rows:
        return "_없음_\n"
    lines = [
        "| 순위 | 파일 | score | overflow | overlap | empty_page | 상태 |",
        "| ---: | --- | ---: | ---: | ---: | ---: | --- |",
    ]
    for idx, row in enumerate(rows, start=1):
        lines.append(
            f"| {idx} | `{row['path']}` | {row['score']} | {row['overflow']} | "
            f"{row['overlap']} | {row['emptyPage']} | {row['status']} |"
        )
    return "\n".join(lines) + "\n"


def format_markdown(report: Report) -> str:
    summary = report.summary()
    top = report.top_offenders()
    off = summary["offCanvasCount"]
    text = summary["textOverlapCount"]
    off_s = "미지원 (devel / #5389 미병합)" if off is None else str(off)
    text_s = "미지원 (devel / #5379 미병합)" if text is None else str(text)
    cmd = (
        "python tools/layout_anomaly_batch_report.py "
        "--root samples --top 20 "
        "--json-out mydocs/working/m02-8-layout-anomaly-batch-report.json "
        "--tsv-out mydocs/working/m02-8-layout-anomaly-batch-report.tsv "
        "--md-out mydocs/working/m02-8-layout-anomaly-batch-report.md"
    )
    lines = [
        "# M02-8 samples/ layout-anomaly 배치 리포트",
        "",
        "M08 착수 근거 데이터. 레이아웃 버그를 고치지 않았다. `--batch`(#5371) 없이",
        "devel 단건 `layout-anomaly --json` 을 파일마다 돌렸다.",
        "",
        "## 재현",
        "",
        "```bash",
        "cargo build --release --bin rhwp",
        cmd,
        "```",
        "",
        f"- 이슈: #{ISSUE_ID}",
        f"- 생성기: `{GENERATOR}`",
        f"- CLI 계약: `{CLI_CONTRACT}`",
        f"- 바이너리: `{report.binary_path}` ({report.binary_version or 'version unknown'})",
        f"- git: `{report.git_commit}` (`{report.git_branch}`)",
        f"- 입력: `{report.root}`  fileCount={report.file_count}  limit={report.limit}",
        f"- timeout={report.timeout_sec:.0f}s  jobs={report.jobs}",
        f"- `--batch` 지원: {'yes' if report.supports_batch else 'no'}",
        f"- off-canvas 필드: {report.supports_off_canvas}",
        f"- text-overlap 필드: {report.supports_text_overlap}",
        f"- 시각: {report.started_at} → {report.finished_at}",
        "",
        "## 헤드라인 카운트",
        "",
        "| 항목 | 값 |",
        "| --- | ---: |",
        f"| 스캔 | {summary['scanned']} |",
        f"| CLEAN | {summary['clean']} |",
        f"| ANOMALY | {summary['anomaly']} |",
        f"| ERROR | {summary['error']} |",
        f"| TIMEOUT | {summary['timeout']} |",
        f"| overflow 건수 (파일 수) | {summary['overflowCount']} ({summary['overflowFiles']}) |",
        f"| overlap 건수 (파일 수) | {summary['overlapCount']} ({summary['overlapFiles']}) |",
        f"| empty_page 건수 (파일 수) | {summary['emptyPageCount']} ({summary['emptyPageFiles']}) |",
        f"| off-canvas 건수 (파일 수) | {off_s} |",
        f"| text-overlap 건수 (파일 수) | {text_s} |",
        "",
        "## Top overflow",
        "",
        _md_table(top["overflow"], "overflow"),
        "",
        "## Top overlap",
        "",
        _md_table(top["overlap"], "overlap"),
        "",
        "## Top empty_page",
        "",
        _md_table(top["emptyPage"], "empty_page"),
    ]
    if report.supports_off_canvas:
        lines.extend(["", "## Top off-canvas", "", _md_table(top["offCanvas"], "off_canvas")])
    else:
        lines.extend(
            [
                "",
                "## Top off-canvas",
                "",
                "devel 바이너리에 `offCanvasCount` 가 없다. #5389 병합 후 같은 명령으로 다시 돌린다.",
                "",
            ]
        )
    if report.supports_text_overlap:
        lines.extend(
            ["", "## Top text-overlap", "", _md_table(top["textOverlap"], "text_overlap")]
        )
    else:
        lines.extend(
            [
                "",
                "## Top text-overlap",
                "",
                "devel 바이너리에 `textOverlapCount` 가 없다. #5379 병합 후 같은 명령으로 다시 돌린다.",
                "",
            ]
        )
    lines.extend(
        [
            "",
            "## Top 총 신호",
            "",
            _md_table(top["totalSignals"], "total"),
            "",
            "## ERROR / TIMEOUT",
            "",
        ]
    )
    failed = [r for r in report.rows if r.status in {"ERROR", "TIMEOUT"}]
    if failed:
        lines.append("| 파일 | 상태 | 오류 |")
        lines.append("| --- | --- | --- |")
        for row in failed:
            err = (row.error or "").replace("|", "\\|")
            if len(err) > 160:
                err = err[:157] + "..."
            lines.append(f"| `{row.path}` | {row.status} | {err} |")
        lines.append("")
    else:
        lines.append("_없음_")
        lines.append("")
    lines.extend(["## 메모", ""])
    if report.notes:
        for note in report.notes:
            lines.append(f"- {note}")
    else:
        lines.append("- (없음)")
    lines.append("")
    return "\n".join(lines)


def emit_text(text: str, output: Path | None) -> None:
    if not text.endswith("\n"):
        text += "\n"
    if output is None or str(output) == "-":
        sys.stdout.write(text)
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text, encoding="utf-8", newline="\n")


def emit_json(data: dict[str, Any], output: Path | None) -> None:
    emit_text(json.dumps(data, ensure_ascii=False, indent=2), output)


KEEP_STATUSES = {"CLEAN", "ANOMALY"}


def row_from_json(item: dict[str, Any]) -> FileRow:
    return FileRow(
        path=str(item.get("path") or ""),
        status=str(item.get("status") or "ERROR"),
        page_count=int(item.get("pageCount") or 0),
        overflow=int(item.get("overflow") or 0),
        overlap=int(item.get("overlap") or 0),
        empty_page=int(item.get("emptyPage") or 0),
        off_canvas=item.get("offCanvas"),
        text_overlap=item.get("textOverlap"),
        has_signal=bool(item.get("hasSignal")),
        elapsed_ms=int(item.get("elapsedMs") or 0),
        error=str(item.get("error") or ""),
        exit_code=item.get("exitCode"),
    )


def load_resume_rows(path: Path) -> dict[str, FileRow]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise UsageError(f"--resume-json 이 object 가 아닙니다: {path}")
    rows = raw.get("rows") or []
    kept: dict[str, FileRow] = {}
    if not isinstance(rows, list):
        return kept
    for item in rows:
        if not isinstance(item, dict):
            continue
        row = row_from_json(item)
        if row.path and row.status in KEEP_STATUSES:
            kept[row.path] = row
    return kept


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
    parser.add_argument("--root", type=Path, default=None, help="스캔 폴더 (기본 samples/)")
    parser.add_argument("--limit", type=int, default=None, help="앞에서 N 파일만")
    parser.add_argument("--top", type=int, default=DEFAULT_TOP, help="유형별 상위 N")
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_SEC)
    parser.add_argument("--jobs", type=int, default=DEFAULT_JOBS, help="병렬 프로세스 수")
    parser.add_argument("--json-out", type=Path, default=None)
    parser.add_argument("--tsv-out", type=Path, default=None)
    parser.add_argument("--md-out", type=Path, default=None)
    parser.add_argument("--json", action="store_true", dest="as_json")
    parser.add_argument("--strict", action="store_true", help="ERROR/TIMEOUT 이면 종료 1")
    parser.add_argument(
        "--collect-only",
        action="store_true",
        help="파일 목록만 세고 rhwp 를 돌리지 않는다",
    )
    parser.add_argument("--quiet", action="store_true")
    parser.add_argument(
        "--resume-json",
        type=Path,
        default=None,
        help="이전 리포트 JSON. CLEAN/ANOMALY 행은 건너뛰고 ERROR/TIMEOUT 만 다시 돈다",
    )
    return parser


def build_report(
    *,
    repo: Path,
    rhwp: Path | None,
    files: Sequence[Path],
    root: Path,
    limit: int | None,
    timeout_sec: float,
    jobs: int,
    top_n: int,
    notes: Iterable[str],
    strict: bool,
    progress: bool,
    collect_only: bool,
) -> Report:
    started = iso_now()
    version = ""
    supports_batch = False
    if rhwp is not None and not collect_only:
        version, supports_batch = probe_binary(rhwp)
    commit, branch = git_identity(repo)
    if collect_only or rhwp is None:
        rows: list[FileRow] = [
            FileRow(path=posix_rel(path, repo), status="SKIP", error="collect-only")
            for path in files
        ]
    else:
        rows = scan_files(
            rhwp,
            files,
            repo,
            timeout_sec=timeout_sec,
            jobs=jobs,
            progress=progress,
        )
    supports_off = infer_optional_support(rows, "off_canvas")
    supports_text = infer_optional_support(rows, "text_overlap")
    note_list = list(notes)
    if supports_batch:
        note_list.append("바이너리가 --batch 를 알지만, 이 리포트는 단건 --json 루프만 썼다")
    else:
        note_list.append("--batch 없음 (#5371). 단건 layout-anomaly --json 루프로 산출")
    if supports_off is False:
        note_list.append("offCanvasCount 없음 — #5389 미병합. 카운트는 null")
    if supports_text is False:
        note_list.append("textOverlapCount 없음 — #5379 미병합. 카운트는 null")
    return Report(
        rows=rows,
        binary_path="" if rhwp is None else posix_rel(rhwp, repo) if rhwp.is_absolute() else str(rhwp),
        binary_version=version,
        supports_batch=supports_batch,
        supports_off_canvas=supports_off,
        supports_text_overlap=supports_text,
        git_commit=commit,
        git_branch=branch,
        root=posix_rel(root, repo),
        file_count=len(files),
        limit=limit,
        timeout_sec=timeout_sec,
        jobs=jobs,
        top_n=top_n,
        started_at=started,
        finished_at=iso_now(),
        notes=note_list,
        strict=strict,
    )


def exit_code(report: Report) -> int:
    if not report.strict:
        return 0
    summary = report.summary()
    if summary["error"] or summary["timeout"]:
        return 1
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    _configure_stdio()
    args = build_parser().parse_args(list(argv) if argv is not None else None)
    try:
        repo = args.repo_root.resolve() if args.repo_root else discover_repo_root()
        root = (args.root or (repo / "samples")).resolve()
        files = walk_docs(root, repo)
        limit = args.limit
        if limit is not None:
            files = files[: max(0, limit)]
        notes: list[str] = []
        kept_rows: dict[str, FileRow] = {}
        if args.resume_json is not None:
            kept_rows = load_resume_rows(args.resume_json)
            before = len(files)
            files = [p for p in files if posix_rel(p, repo) not in kept_rows]
            notes.append(
                f"resume {posix_rel(args.resume_json, repo)}: "
                f"kept {len(kept_rows)}, rerun {len(files)}/{before}"
            )
        rhwp = None if args.collect_only else find_rhwp(args.rhwp, repo)
        if not args.collect_only and rhwp is None:
            raise UsageError(
                "rhwp 바이너리가 없습니다. --rhwp 또는 cargo build --release --bin rhwp"
            )
        if not files:
            notes.append("스캔할 .hwp/.hwpx 가 없다")
        report = build_report(
            repo=repo,
            rhwp=rhwp,
            files=files,
            root=root,
            limit=limit,
            timeout_sec=float(args.timeout),
            jobs=max(1, int(args.jobs)),
            top_n=int(args.top),
            notes=notes,
            strict=bool(args.strict),
            progress=not args.quiet,
            collect_only=bool(args.collect_only),
        )
        if kept_rows:
            merged = {row.path: row for row in report.rows}
            merged.update(kept_rows)
            report.rows = sorted(merged.values(), key=lambda r: r.path.casefold())
            report.file_count = len(report.rows)
        if args.json_out is not None:
            emit_json(report.to_json(), args.json_out)
            report.notes.append(f"json written: {posix_rel(args.json_out, repo)}")
        if args.tsv_out is not None:
            emit_text(format_tsv(report.rows), args.tsv_out)
            report.notes.append(f"tsv written: {posix_rel(args.tsv_out, repo)}")
        if args.md_out is not None:
            emit_text(format_markdown(report), args.md_out)
            report.notes.append(f"md written: {posix_rel(args.md_out, repo)}")
        if args.as_json:
            emit_json(report.to_json(), None)
        elif args.md_out is None and args.json_out is None and args.tsv_out is None:
            sys.stdout.write(format_markdown(report))
        elif not args.quiet and not args.as_json:
            sys.stdout.write(format_markdown(report))
        return exit_code(report)
    except UsageError as exc:
        sys.stderr.write(f"사용법 오류: {exc}\n")
        return 2
    except (OSError, json.JSONDecodeError) as exc:
        sys.stderr.write(f"사용법 오류: {exc}\n")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
