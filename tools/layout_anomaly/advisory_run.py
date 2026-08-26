#!/usr/bin/env python3
"""layout-anomaly advisory sample runner.

Never passes --strict. Always exits 0 so CI cannot become a merge gate.
If tools/layout_anomaly/batch_report.py exists, that script is preferred.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

BATCH_REPORT = Path("tools/layout_anomaly/batch_report.py")
DEFAULT_LIST = Path("tools/layout_anomaly/advisory_samples.txt")


def _parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--rhwp", required=True, help="rhwp binary path")
    p.add_argument("--out", required=True, help="output directory")
    p.add_argument("--list", default=str(DEFAULT_LIST), help="sample list file")
    p.add_argument(
        "--limit",
        type=int,
        default=0,
        help="max samples (0 = all listed)",
    )
    p.add_argument(
        "--timeout",
        type=int,
        default=60,
        help="per-document timeout seconds",
    )
    return p.parse_args()


def _read_list(path: Path, limit: int) -> list[str]:
    if not path.is_file():
        return []
    rows: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        rows.append(line)
        if limit > 0 and len(rows) >= limit:
            break
    return rows


def _unwrap(payload: object) -> dict:
    if not isinstance(payload, dict):
        return {}
    inner = payload.get("untrustedContent")
    if isinstance(inner, dict):
        return inner
    return payload


def _try_batch_report(args: argparse.Namespace, out: Path) -> bool:
    if not BATCH_REPORT.is_file():
        return False
    cmd = [
        sys.executable,
        str(BATCH_REPORT),
        "--rhwp",
        args.rhwp,
        "--json",
        "--out",
        str(out),
    ]
    if args.limit > 0:
        cmd.extend(["--limit", str(args.limit)])
    print("batch report command:", " ".join(cmd), flush=True)
    proc = subprocess.run(cmd, check=False)
    print(f"batch_report_exit={proc.returncode}", flush=True)
    return proc.returncode == 0 and (out / "summary.md").is_file()


def _run_one(rhwp: str, doc: Path, timeout: int) -> dict:
    cmd = [rhwp, "layout-anomaly", "--json", str(doc)]
    print("scan:", " ".join(cmd), flush=True)
    try:
        proc = subprocess.run(
            cmd,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return {
            "doc": str(doc),
            "verdict": "ERROR",
            "note": f"timeout>{timeout}s",
            "exit": 124,
        }
    except OSError as exc:
        return {
            "doc": str(doc),
            "verdict": "ERROR",
            "note": str(exc),
            "exit": 1,
        }

    envelope: dict = {}
    parse_error = ""
    raw = (proc.stdout or "").strip()
    if raw:
        try:
            envelope = _unwrap(json.loads(raw))
        except json.JSONDecodeError as exc:
            parse_error = str(exc)

    overflow = int(envelope.get("overflowCount") or 0)
    overlap = int(envelope.get("overlapCount") or 0)
    empty_page = int(envelope.get("emptyPageCount") or 0)
    has_signal = bool(envelope.get("hasSignal"))
    if proc.returncode not in (0, 3):
        verdict = "ERROR"
    elif has_signal or overflow or overlap:
        verdict = "ANOMALY"
    else:
        verdict = "CLEAN"

    return {
        "doc": str(doc).replace("\\", "/"),
        "verdict": verdict,
        "exit": proc.returncode,
        "overflowCount": overflow,
        "overlapCount": overlap,
        "emptyPageCount": empty_page,
        "pageCount": envelope.get("pageCount"),
        "hasSignal": has_signal,
        "parseError": parse_error,
        "stderr": (proc.stderr or "").strip()[:500],
        "repro": f"rhwp layout-anomaly --json {doc}",
    }


def _write_report(out: Path, rows: list[dict], skipped: list[str], mode: str) -> None:
    out.mkdir(parents=True, exist_ok=True)
    (out / "rows").mkdir(exist_ok=True)
    summary = {
        "docs": len(rows),
        "clean": sum(1 for r in rows if r.get("verdict") == "CLEAN"),
        "anomaly": sum(1 for r in rows if r.get("verdict") == "ANOMALY"),
        "error": sum(1 for r in rows if r.get("verdict") == "ERROR"),
        "skipped": len(skipped),
    }
    report = {
        "schema": "layout_anomaly.advisory_report/v1",
        "mode": mode,
        "strict": False,
        "summary": summary,
        "skipped": skipped,
        "rows": rows,
    }
    (out / "report.json").write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    for i, row in enumerate(rows, start=1):
        stem = Path(str(row.get("doc") or f"row-{i}")).name
        (out / "rows" / f"{i:02d}-{stem}.json").write_text(
            json.dumps(row, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    lines = [
        "## layout-anomaly (advisory)",
        "",
        "이 잡은 **advisory** 입니다. required check 가 아니며 PR 게이트를 막지 않습니다.",
        "`visual_sweep.py` 는 호출하지 않았습니다. `--strict` 는 쓰지 않았습니다.",
        "",
        f"mode: `{mode}`",
        "",
        "| docs | clean | anomaly | error | skipped |",
        "| ---: | ---: | ---: | ---: | ---: |",
        (
            f"| {summary['docs']} | {summary['clean']} | {summary['anomaly']} | "
            f"{summary['error']} | {summary['skipped']} |"
        ),
        "",
    ]
    if rows:
        lines.extend(
            [
                "| verdict | overflow | overlap | empty | doc |",
                "| --- | ---: | ---: | ---: | --- |",
            ]
        )
        for row in rows:
            lines.append(
                f"| {row.get('verdict')} | {row.get('overflowCount', 0)} | "
                f"{row.get('overlapCount', 0)} | {row.get('emptyPageCount', 0)} | "
                f"`{row.get('doc', '')}` |"
            )
        lines.append("")
    if skipped:
        lines.append("건너뛴 경로: " + ", ".join(f"`{s}`" for s in skipped))
        lines.append("")
    (out / "summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(lines), flush=True)


def main() -> int:
    args = _parse_args()
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    if _try_batch_report(args, out):
        print("used batch_report.py", flush=True)
        return 0

    samples = _read_list(Path(args.list), args.limit)
    if not samples:
        reason = "samples-absent"
        (out / "summary.md").write_text(
            "## layout-anomaly (advisory) — skipped\n\n"
            f"reason: `{reason}`\n\n"
            "이 잡은 advisory 이며 required check 가 아닙니다.\n",
            encoding="utf-8",
        )
        print(f"reason={reason}", flush=True)
        return 0

    rows: list[dict] = []
    skipped: list[str] = []
    for rel in samples:
        path = Path(rel)
        if not path.is_file():
            skipped.append(rel)
            continue
        rows.append(_run_one(args.rhwp, path, args.timeout))

    _write_report(out, rows, skipped, mode="sample-list")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:  # noqa: BLE001 — advisory must not fail CI
        print(f"advisory_run caught: {exc}", file=sys.stderr)
        out = Path(os.environ.get("ADVISORY_OUT") or "layout-anomaly-advisory")
        out.mkdir(parents=True, exist_ok=True)
        (out / "summary.md").write_text(
            "## layout-anomaly (advisory) — skipped\n\n"
            f"reason: `runner-exception`\n\n`{exc}`\n",
            encoding="utf-8",
        )
        sys.exit(0)
