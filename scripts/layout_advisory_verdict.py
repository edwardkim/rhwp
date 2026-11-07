#!/usr/bin/env python3
"""Distinguish a completed advisory scan from a green skipped/failed job.

Anomalies remain advisory. Missing samples, parser errors, interrupted scans,
and malformed evidence cannot serve as release promotion execution evidence.
"""
import argparse
import json
from pathlib import Path


def verdict(report, *, build_outcome, scan_exit):
    result = {"schemaVersion": 1, "verdict": "incomplete"}
    if build_outcome != "success" or scan_exit != "0":
        return result
    if not isinstance(report, dict) or report.get("schema") != "layout_anomaly.advisory_report/v1":
        return result
    rows, skipped = report.get("rows"), report.get("skipped")
    summary = report.get("summary")
    if not isinstance(rows, list) or not rows or skipped != [] or not isinstance(summary, dict):
        return result
    if any(not isinstance(row, dict) or row.get("verdict") not in ("CLEAN", "ANOMALY") for row in rows):
        return result
    if any(row.get("exit") not in (0, 3) or row.get("parseError") != ""
           or type(row.get("pageCount")) is not int or row["pageCount"] < 1 for row in rows):
        return result
    counts = {
        "docs": len(rows),
        "clean": sum(row["verdict"] == "CLEAN" for row in rows),
        "anomaly": sum(row["verdict"] == "ANOMALY" for row in rows),
        "error": 0,
        "skipped": 0,
    }
    if any(type(summary.get(key)) is not int or summary[key] != value for key, value in counts.items()):
        return result
    return {**result, "verdict": "completed", **counts}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--build-outcome", default="")
    parser.add_argument("--scan-exit", default="")
    args = parser.parse_args()
    try:
        report = json.loads(args.report.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError):
        report = None
    result = verdict(report, build_outcome=args.build_outcome, scan_exit=args.scan_exit)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
