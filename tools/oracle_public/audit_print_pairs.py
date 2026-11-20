#!/usr/bin/env python3
"""기존 resolver 후보에 대해 저장소에 보관된 독립 Print 기록을 대조한다."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path

if __package__:
    from . import oracle_resolver
else:
    import oracle_resolver


def digest(path):
    with path.open("rb") as stream:
        result = hashlib.sha256()
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(block)
        return result.hexdigest()


def file_status(root, name, expected):
    if not isinstance(expected, str) or not re.fullmatch(r"[0-9a-fA-F]{64}", expected):
        return {"state": "unrecorded"}
    path = (root / name).resolve()
    if not path.is_relative_to(root) or not path.is_file():
        return {"state": "unavailable"}
    actual = digest(path)
    return {"state": "match" if actual == expected.lower() else "mismatch",
            "recordedSha256": expected, "actualSha256": actual}


def print_details(origin):
    if not isinstance(origin, dict):
        return None
    kind = origin.get("kind")
    engine = origin.get("engine")
    if kind == "MCP Print":
        result = origin.get("print_result")
        modes = {"2020": "hancom2020_pdf_driver_one_up", "2024": "frame_print_to_pdf_ex_one_up"}
        if (not isinstance(result, dict) or not isinstance(engine, str) or not origin.get("job_id")
                or engine not in modes or result.get("engine") != engine
                or result.get("pdf_output_mode") != modes[engine]
                or type(result.get("pdf_print_method")) is not int
                or result["pdf_print_method"] != 0 or not result.get("hancom_version")
                or "preprocess" not in result or result["preprocess"] is not None):
            return None
        return {"kind": kind, "jobId": origin["job_id"], "result": result}
    if (kind == "user-provided manual Print" and origin.get("print_attestation")
            and engine and origin.get("hancom_version")):
        return {key: origin[key] for key in ("kind", "engine", "hancom_version", "print_attestation")}
    return None


def index_receipts(document):
    if not isinstance(document, dict) or not isinstance(document.get("cases"), list):
        raise ValueError("Print 기록의 cases 배열이 필요합니다.")
    receipts = {}
    for index, row in enumerate(document["cases"]):
        if not isinstance(row, dict):
            raise ValueError(f"cases[{index}]는 객체여야 합니다.")
        key = (row.get("input"), row.get("pdf"))
        for name in key:
            if (not isinstance(name, str) or not name or Path(name).is_absolute()
                    or ".." in Path(name).parts or Path(name).as_posix() != name):
                raise ValueError(f"cases[{index}]에 저장소 상대 경로가 필요합니다.")
        if Path(key[0]).suffix.lower() not in oracle_resolver.SAMPLE_EXTS or Path(key[1]).suffix.lower() != ".pdf":
            raise ValueError(f"cases[{index}]에 HWP/HWPX 원본과 PDF 경로가 필요합니다.")
        if key in receipts:
            raise ValueError(f"Print 기록에 중복된 원본·PDF 경로가 있습니다: {key}")
        receipts[key] = row
    return receipts


def audit(root, manifest, document):
    root = root.resolve()
    receipts = index_receipts(document)
    candidates = {(row["sample"], row["pdf"]) for row in manifest["pairs"]}
    rows = []
    for sample, pdf in sorted(candidates | receipts.keys()):
        key = (sample, pdf)
        item = {"sample": sample, "pdf": pdf, "resolverCandidate": key in candidates,
                "status": "candidate", "print": None}
        if key in receipts:
            record = receipts[key]
            item["sampleHash"] = file_status(root, sample, record.get("input_sha256"))
            item["pdfHash"] = file_status(root, pdf, record.get("pdf_sha256"))
            item["print"] = print_details(record.get("origin"))
            states = {item["sampleHash"]["state"], item["pdfHash"]["state"]}
            if "mismatch" in states:
                item["status"] = "mismatch"
            elif "unavailable" in states:
                item["status"] = "unavailable"
            elif states == {"match"} and item["print"]:
                item["status"] = "verified"
        rows.append(item)
    return {"schemaVersion": "1.0", "statusCounts": dict(Counter(row["status"] for row in rows)),
            "pairs": rows, "resolverUnmatched": manifest["unmatched"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--receipts", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        root = args.repo_root.resolve()
        manifest = oracle_resolver.build_manifest(root)
        raw = args.receipts.read_bytes()
        report = audit(root, manifest, json.loads(raw))
        report["receiptSha256"] = hashlib.sha256(raw).hexdigest()
        oracle_resolver.emit_json(report, args.output, pretty=True)
    except (OSError, ValueError, oracle_resolver.UsageError) as exc:
        parser.exit(2, f"Print 기록 대조 실패: {exc}\n")


if __name__ == "__main__":
    main()
