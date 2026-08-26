#!/usr/bin/env python3
"""임계 초과 문서 → GitHub 이슈 **초안** markdown 생성기.

입력은 `schema/failure_report.v1.json` 계약의 실패 리포트 JSON 이다. 게이트
(`threshold.metric op threshold.value`)를 넘는 문서마다 재현 커맨드와 수치를
넣은 markdown 을 `--out` 디렉터리에 쓴다. 제출은 수동 — 이 도구는 `gh` 를
호출하지 않으며 `--submit` / `--create-issue` 를 거부한다.

    python tools/oracle_public/issue_draft.py \
        --report tools/oracle_public/fixtures/report_mixed.json \
        --out /tmp/oracle-issue-drafts

종료 코드: 0 = 초안(0건 포함) 기록 성공, 2 = 입력·스키마 오류,
3 = 제출 옵션 거부.
"""

from __future__ import annotations

import argparse
import json
import operator
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

SCHEMA_ID = "oracle_public.failure_report/v1"
MANIFEST_SCHEMA = "oracle_public.issue_drafts/v1"
HERE = Path(__file__).resolve().parent
DEFAULT_TEMPLATE = HERE / "templates" / "issue.md"
DEFAULT_SCHEMA = HERE / "schema" / "failure_report.v1.json"

EXIT_OK = 0
EXIT_USAGE = 2
EXIT_SUBMIT_FORBIDDEN = 3

OPS = {
    "<": operator.lt,
    "<=": operator.le,
    ">": operator.gt,
    ">=": operator.ge,
}

PLACEHOLDER_RE = re.compile(r"\{\{\s*([A-Za-z0-9_]+)\s*\}\}")
SLUG_RE = re.compile(r"[^\w가-힣.-]+", re.UNICODE)
SUBMIT_FLAGS = ("--submit", "--create-issue", "--create-issues", "--gh")

FORBIDDEN_HELP = (
    "이슈 제출은 수동입니다. 이 도구는 초안 markdown 만 디스크에 씁니다. "
    "`gh issue create` 를 호출하지 않으며 --submit/--create-issue/--gh 를 받지 않습니다."
)


class ReportError(ValueError):
    """스키마·값 계약 위반."""


@dataclass(frozen=True)
class Threshold:
    metric: str
    op: str
    value: float

    def failed(self, measured: float) -> bool:
        return bool(OPS[self.op](measured, self.value))

    def as_dict(self) -> dict[str, Any]:
        return {"metric": self.metric, "op": self.op, "value": self.value}


@dataclass
class Document:
    id: str
    hwp: str
    metrics: dict[str, Any]
    pdf: str = ""
    pages: int | None = None
    exceeds: bool | None = None
    repro_command: str = ""
    repro_cwd: str = "."
    notes: str = ""

    def measured(self, metric: str) -> float | None:
        raw = self.metrics.get(metric)
        if isinstance(raw, bool) or not isinstance(raw, (int, float)):
            return None
        return float(raw)


@dataclass
class Report:
    threshold: Threshold
    documents: list[Document]
    generated_at: str = ""
    source: str = ""
    rhwp_bin: str = ""
    dpi: int | None = None
    pixel_diff_threshold: int | None = None


@dataclass
class Draft:
    document: Document
    path: Path
    reason: str
    measured: float | None


@dataclass
class DraftBatch:
    drafts: list[Draft] = field(default_factory=list)
    skipped: list[dict[str, Any]] = field(default_factory=list)


def slugify(value: str) -> str:
    cleaned = SLUG_RE.sub("-", value.strip())
    cleaned = re.sub(r"-{2,}", "-", cleaned).strip(".-")
    return cleaned or "untitled"


def format_number(value: Any) -> str:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return str(value)
    if isinstance(value, int) or (isinstance(value, float) and value.is_integer()):
        return str(int(value))
    text = f"{float(value):.8f}".rstrip("0").rstrip(".")
    return text


def format_worst_pages(metrics: dict[str, Any]) -> str:
    raw = metrics.get("worst_pages")
    if isinstance(raw, list) and raw:
        pages = [str(item) for item in raw if isinstance(item, (int, float))]
        if pages:
            return ", ".join(pages)
    return "(없음)"


def metrics_table(metrics: dict[str, Any]) -> str:
    if not metrics:
        return "(지표 없음)"
    lines = ["| 지표 | 값 |", "| --- | --- |"]
    for key, value in metrics.items():
        if isinstance(value, list):
            rendered = ", ".join(str(item) for item in value)
        else:
            rendered = format_number(value) if isinstance(value, (int, float)) else str(value)
        lines.append(f"| `{key}` | {rendered} |")
    return "\n".join(lines)


def render_template(template: str, context: dict[str, str]) -> str:
    return PLACEHOLDER_RE.sub(lambda match: context.get(match.group(1), ""), template)


def _require_dict(value: Any, path: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ReportError(f"{path} 는 객체여야 합니다.")
    return value


def _require_str(value: Any, path: str, *, allow_empty: bool = False) -> str:
    if not isinstance(value, str):
        raise ReportError(f"{path} 는 문자열이어야 합니다.")
    if not allow_empty and not value.strip():
        raise ReportError(f"{path} 가 비어 있습니다.")
    return value


def parse_threshold(raw: Any) -> Threshold:
    data = _require_dict(raw, "threshold")
    metric = _require_str(data.get("metric"), "threshold.metric")
    op = _require_str(data.get("op"), "threshold.op")
    if op not in OPS:
        raise ReportError(f"threshold.op 는 {sorted(OPS)} 중 하나여야 합니다: {op!r}")
    value = data.get("value")
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ReportError("threshold.value 는 숫자여야 합니다.")
    return Threshold(metric=metric, op=op, value=float(value))


def parse_document(raw: Any, index: int) -> Document:
    path = f"documents[{index}]"
    data = _require_dict(raw, path)
    metrics = data.get("metrics")
    if not isinstance(metrics, dict):
        raise ReportError(f"{path}.metrics 는 객체여야 합니다.")
    pages = data.get("pages")
    if pages is not None and (isinstance(pages, bool) or not isinstance(pages, int) or pages < 0):
        raise ReportError(f"{path}.pages 는 0 이상 정수여야 합니다.")
    exceeds = data.get("exceeds")
    if exceeds is not None and not isinstance(exceeds, bool):
        raise ReportError(f"{path}.exceeds 는 boolean 이어야 합니다.")
    repro = data.get("repro") or {}
    if repro and not isinstance(repro, dict):
        raise ReportError(f"{path}.repro 는 객체여야 합니다.")
    command = repro.get("command", "") if isinstance(repro, dict) else ""
    if command and not isinstance(command, str):
        raise ReportError(f"{path}.repro.command 는 문자열이어야 합니다.")
    cwd = repro.get("cwd", ".") if isinstance(repro, dict) else "."
    if cwd is not None and not isinstance(cwd, str):
        raise ReportError(f"{path}.repro.cwd 는 문자열이어야 합니다.")
    notes = data.get("notes", "")
    if notes and not isinstance(notes, str):
        raise ReportError(f"{path}.notes 는 문자열이어야 합니다.")
    pdf = data.get("pdf", "")
    if pdf and not isinstance(pdf, str):
        raise ReportError(f"{path}.pdf 는 문자열이어야 합니다.")
    return Document(
        id=_require_str(data.get("id"), f"{path}.id"),
        hwp=_require_str(data.get("hwp"), f"{path}.hwp"),
        metrics=metrics,
        pdf=pdf or "",
        pages=pages,
        exceeds=exceeds,
        repro_command=command or "",
        repro_cwd=cwd or ".",
        notes=notes or "",
    )


def parse_report(raw: Any) -> Report:
    data = _require_dict(raw, "$")
    schema = data.get("schema")
    if schema != SCHEMA_ID:
        raise ReportError(f"schema 는 {SCHEMA_ID!r} 여야 합니다: {schema!r}")
    documents_raw = data.get("documents")
    if not isinstance(documents_raw, list):
        raise ReportError("documents 는 배열이어야 합니다.")
    dpi = data.get("dpi")
    if dpi is not None and (isinstance(dpi, bool) or not isinstance(dpi, int) or dpi < 1):
        raise ReportError("dpi 는 1 이상 정수여야 합니다.")
    pixel = data.get("pixel_diff_threshold")
    if pixel is not None and (
        isinstance(pixel, bool) or not isinstance(pixel, int) or not 0 <= pixel <= 255
    ):
        raise ReportError("pixel_diff_threshold 는 0 이상 255 이하 정수여야 합니다.")
    source = data.get("source", "")
    generated_at = data.get("generated_at", "")
    rhwp_bin = data.get("rhwp_bin", "")
    for label, value in (
        ("source", source),
        ("generated_at", generated_at),
        ("rhwp_bin", rhwp_bin),
    ):
        if value and not isinstance(value, str):
            raise ReportError(f"{label} 는 문자열이어야 합니다.")
    return Report(
        threshold=parse_threshold(data.get("threshold")),
        documents=[parse_document(item, index) for index, item in enumerate(documents_raw)],
        generated_at=generated_at or "",
        source=source or "",
        rhwp_bin=rhwp_bin or "",
        dpi=dpi,
        pixel_diff_threshold=pixel,
    )


def load_report(path: Path) -> Report:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ReportError(f"리포트를 읽을 수 없습니다: {path}: {exc}") from exc
    try:
        raw = json.loads(text)
    except json.JSONDecodeError as exc:
        raise ReportError(f"리포트 JSON 이 깨졌습니다: {path}: {exc}") from exc
    return parse_report(raw)


def synthesize_repro(document: Document, report: Report) -> str:
    if document.repro_command.strip():
        return document.repro_command.strip()
    parts = ["python scripts/visual_sweep.py", "--hwp", document.hwp]
    if document.pdf:
        parts += ["--pdf", document.pdf]
    parts += ["--key", document.id]
    if report.dpi is not None:
        parts += ["--dpi", str(report.dpi)]
    if report.pixel_diff_threshold is not None:
        parts += ["--pixel-diff-threshold", str(report.pixel_diff_threshold)]
    return " ".join(parts)


def classify_document(document: Document, threshold: Threshold) -> tuple[bool, str, float | None]:
    if document.exceeds is True:
        return True, "exceeds=true", document.measured(threshold.metric)
    if document.exceeds is False:
        return False, "exceeds=false", document.measured(threshold.metric)
    measured = document.measured(threshold.metric)
    if measured is None:
        return False, f"지표 없음:{threshold.metric}", None
    if threshold.failed(measured):
        return (
            True,
            f"{threshold.metric}={format_number(measured)} {threshold.op} {format_number(threshold.value)}",
            measured,
        )
    return (
        False,
        f"{threshold.metric}={format_number(measured)} 게이트 통과",
        measured,
    )


def draft_title(document: Document, threshold: Threshold, measured: float | None) -> str:
    value = format_number(measured) if measured is not None else "?"
    return (
        f"[오라클] {document.id}: {threshold.metric} {value} "
        f"({threshold.op} {format_number(threshold.value)})"
    )


def draft_context(document: Document, report: Report, measured: float | None) -> dict[str, str]:
    notes = document.notes.strip()
    return {
        "title": draft_title(document, report.threshold, measured),
        "id": document.id,
        "hwp": document.hwp,
        "pdf": document.pdf or "(없음)",
        "pages": str(document.pages) if document.pages is not None else "(미기재)",
        "metric": report.threshold.metric,
        "metric_value": format_number(measured) if measured is not None else "(없음)",
        "threshold_op": report.threshold.op,
        "threshold_value": format_number(report.threshold.value),
        "repro_command": synthesize_repro(document, report),
        "repro_cwd": document.repro_cwd or ".",
        "source": report.source or "(미기재)",
        "rhwp_bin": report.rhwp_bin or "(미기재)",
        "dpi": str(report.dpi) if report.dpi is not None else "(미기재)",
        "pixel_diff_threshold": (
            str(report.pixel_diff_threshold)
            if report.pixel_diff_threshold is not None
            else "(미기재)"
        ),
        "worst_pages": format_worst_pages(document.metrics),
        "notes_block": f"{notes}\n" if notes else "",
        "metrics_table": metrics_table(document.metrics),
        "generated_at": report.generated_at or "(미기재)",
    }


def unique_draft_path(out_dir: Path, document_id: str, used: set[str]) -> Path:
    base = slugify(document_id)
    name = f"{base}.md"
    index = 2
    while name in used:
        name = f"{base}-{index}.md"
        index += 1
    used.add(name)
    return out_dir / name


def select_drafts(report: Report) -> DraftBatch:
    batch = DraftBatch()
    for document in report.documents:
        failed, reason, measured = classify_document(document, report.threshold)
        if failed:
            batch.drafts.append(
                Draft(document=document, path=Path(), reason=reason, measured=measured)
            )
        else:
            batch.skipped.append({"id": document.id, "reason": reason})
    return batch


def write_drafts(
    report: Report,
    out_dir: Path,
    template: str,
    *,
    force: bool,
    dry_run: bool,
    report_path: Path,
) -> dict[str, Any]:
    batch = select_drafts(report)
    used: set[str] = set()
    written: list[dict[str, Any]] = []
    if not dry_run:
        out_dir.mkdir(parents=True, exist_ok=True)
    for draft in batch.drafts:
        path = unique_draft_path(out_dir, draft.document.id, used)
        if path.exists() and not force and not dry_run:
            raise ReportError(f"이미 초안이 있습니다(덮으려면 --force): {path}")
        body = render_template(template, draft_context(draft.document, report, draft.measured))
        if not dry_run:
            path.write_bytes(body.encode("utf-8"))
        written.append(
            {
                "id": draft.document.id,
                "path": str(path),
                "reason": draft.reason,
                "metric": report.threshold.metric,
                "value": draft.measured,
                "repro": synthesize_repro(draft.document, report),
            }
        )
    manifest = {
        "schema": MANIFEST_SCHEMA,
        "source_report": str(report_path),
        "threshold": report.threshold.as_dict(),
        "drafted": len(written),
        "skipped": len(batch.skipped),
        "submitted": False,
        "submit": "manual",
        "drafts": written,
        "skipped_documents": batch.skipped,
    }
    if not dry_run:
        (out_dir / "manifest.json").write_bytes(
            (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
        )
    return manifest


def reject_submit_flags(argv: list[str]) -> None:
    hit = [flag for flag in SUBMIT_FLAGS if flag in argv]
    if hit:
        raise SystemExit(f"{FORBIDDEN_HELP} 거부된 옵션: {' '.join(hit)}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="임계 초과 문서의 GitHub 이슈 초안을 디스크에만 씁니다.",
        epilog=FORBIDDEN_HELP,
    )
    parser.add_argument("--report", required=True, type=Path, help="failure_report v1 JSON 경로")
    parser.add_argument("--out", required=True, type=Path, help="초안 markdown 출력 디렉터리")
    parser.add_argument(
        "--template",
        type=Path,
        default=DEFAULT_TEMPLATE,
        help=f"markdown 템플릿 (기본: {DEFAULT_TEMPLATE})",
    )
    parser.add_argument("--dry-run", action="store_true", help="파일을 쓰지 않고 대상만 출력")
    parser.add_argument("--force", action="store_true", help="기존 초안 파일을 덮어쓴다")
    parser.add_argument("--json", action="store_true", help="매니페스트를 stdout 에 JSON 으로 출력")
    return parser


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")
    args_list = list(sys.argv[1:] if argv is None else argv)
    try:
        reject_submit_flags(args_list)
    except SystemExit as exc:
        print(exc, file=sys.stderr)
        return EXIT_SUBMIT_FORBIDDEN
    parser = build_parser()
    try:
        args = parser.parse_args(args_list)
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else EXIT_USAGE
        return code if code != 0 else EXIT_OK
    try:
        report = load_report(args.report)
        template = args.template.read_text(encoding="utf-8")
        manifest = write_drafts(
            report,
            args.out,
            template,
            force=args.force,
            dry_run=args.dry_run,
            report_path=args.report,
        )
    except (OSError, ReportError) as exc:
        print(f"오류: {exc}", file=sys.stderr)
        return EXIT_USAGE
    if args.json:
        print(json.dumps(manifest, ensure_ascii=False, indent=2))
    else:
        print(
            f"초안 {manifest['drafted']}건, 건너뜀 {manifest['skipped']}건"
            + (" (dry-run)" if args.dry_run else f" → {args.out}")
        )
        for item in manifest["drafts"]:
            print(f"  - {item['id']}: {item['path']}")
        print("제출하지 않음 (수동).")
    return EXIT_OK


if __name__ == "__main__":
    raise SystemExit(main())
