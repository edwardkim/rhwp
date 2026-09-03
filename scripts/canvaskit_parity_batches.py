#!/usr/bin/env python3
"""Run canvaskit-parity work batches 2 and 3 on existing harnesses.

Batch names come from mydocs/tech/canvaskit-parity-implementation.md:

  2. Paint Family Parity Closures
  3. Strict Text Variant Replay

This driver records command results as data. Failures are judgment, not a
reason to rewrite scripts/renderer_baseline_manifest.json. Manifest updates
stay on the existing renderer_baseline / renderer-contract process.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
STUDIO_ROOT = ROOT / "rhwp-studio"
DEFAULT_MANIFEST = ROOT / "scripts" / "renderer_baseline_manifest.json"
PLAN_DOC = ROOT / "mydocs" / "tech" / "canvaskit-parity-implementation.md"
NPM_CMD = "npm.cmd" if sys.platform == "win32" else "npm"
CARGO_CMD = "cargo.exe" if sys.platform == "win32" else "cargo"

BATCH_NAMES = {
    2: "Paint Family Parity",
    3: "Strict Text Variant Replay",
}


def job(
    job_id: str,
    title: str,
    command: list[str],
    *,
    cwd: Path = ROOT,
    heavy: bool = False,
) -> dict[str, Any]:
    return {
        "id": job_id,
        "title": title,
        "command": command,
        "cwd": cwd,
        "heavy": heavy,
    }


def batch_jobs() -> dict[int, list[dict[str, Any]]]:
    """Map plan batches to existing Windows-runnable harness commands."""
    rust_lib = [CARGO_CMD, "test", "--lib", "--"]
    node_test = ["node", "--test"]
    return {
        2: [
            job(
                "rust-canvaskit-policy",
                "Rust CanvasKit replay-plan / paint-family policy",
                [*rust_lib, "canvaskit_policy"],
            ),
            job(
                "studio-image-header",
                "CanvasKit image-header family (PNG/JPEG/GIF/WebP/BMP)",
                [*node_test, "tests/canvaskit-image-header.test.ts"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "studio-resource-key",
                "CanvasKit resource-key / cache identity",
                [*node_test, "tests/canvaskit-resource-key.test.ts"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "studio-document-preflight",
                "CanvasKit document preflight blockers",
                [*node_test, "tests/canvaskit-document-preflight.test.ts"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "studio-renderer-contract",
                "Renderer contract: LayerPaintOp dispatch + paint-family replay",
                ["node", "e2e/renderer-contract.test.mjs"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "renderer-baseline-readiness",
                "renderer_baseline readiness-only capture (existing manifest)",
                [
                    sys.executable,
                    str(ROOT / "scripts" / "renderer_baseline.py"),
                    "--readiness-only",
                    "--scope",
                    "representative",
                    "--browser-mode",
                    "headless",
                    "--profiles",
                    "screen",
                    "--output",
                    str(ROOT / "output" / "renderer-baseline" / "m11p-batch2"),
                ],
                heavy=True,
            ),
        ],
        3: [
            job(
                "rust-text-variants",
                "Rust schema-v1 text variant grouping",
                [*rust_lib, "text_variants"],
            ),
            job(
                "studio-text-variant-selection",
                "Strict GlyphRun / GlyphOutline leaf selection",
                [*node_test, "tests/canvaskit-text-variant-selection.test.ts"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "studio-sfnt-face",
                "Exact SFNT / TTC face extraction for GlyphRun",
                [*node_test, "tests/canvaskit-sfnt-face.test.ts"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "studio-font-plan",
                "Required-family CanvasKit font plan",
                [*node_test, "tests/canvaskit-font-plan.test.ts"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "studio-renderer-contract",
                "Renderer contract: strict variant replay + reject reasons",
                ["node", "e2e/renderer-contract.test.mjs"],
                cwd=STUDIO_ROOT,
            ),
            job(
                "studio-canvaskit-font-coverage",
                "CanvasKit font-coverage e2e (browser + WASM)",
                [NPM_CMD, "run", "e2e:canvaskit-font-coverage"],
                cwd=STUDIO_ROOT,
                heavy=True,
            ),
        ],
    }


def parse_batches(raw: str) -> list[int]:
    values: list[int] = []
    for item in raw.split(","):
        token = item.strip()
        if not token:
            continue
        number = int(token)
        if number not in BATCH_NAMES:
            raise SystemExit(
                f"unsupported batch {number} (allowed: {', '.join(str(n) for n in BATCH_NAMES)})"
            )
        if number not in values:
            values.append(number)
    if not values:
        raise SystemExit("at least one batch must be specified")
    return values


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run canvaskit-parity batches 2 and 3 against existing harnesses."
    )
    parser.add_argument(
        "--batches",
        default="2,3",
        help="comma-separated batch numbers (default: 2,3)",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="print the batch-to-command map and exit",
    )
    parser.add_argument(
        "--heavy",
        action="store_true",
        help="also run browser/WASM/readiness capture jobs",
    )
    parser.add_argument(
        "--output",
        default="",
        help="optional JSON result path",
    )
    return parser.parse_args(argv)


def serialize_job(entry: dict[str, Any]) -> dict[str, Any]:
    return {
        "id": entry["id"],
        "title": entry["title"],
        "command": entry["command"],
        "cwd": repo_relative(entry["cwd"]),
        "heavy": bool(entry["heavy"]),
    }


def repo_relative(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT)).replace("\\", "/")
    except ValueError:
        return str(path)


def which_or_none(name: str) -> str | None:
    return shutil.which(name)


def skip_reason(entry: dict[str, Any], include_heavy: bool) -> str | None:
    if entry["heavy"] and not include_heavy:
        return "skipped: pass --heavy for browser/WASM/readiness capture"
    command0 = entry["command"][0]
    if command0 in {CARGO_CMD, "cargo", "cargo.exe"} and not which_or_none(CARGO_CMD):
        return f"missing {CARGO_CMD} on PATH"
    if command0 in {"node"} and not which_or_none("node"):
        return "missing node on PATH"
    if command0 in {NPM_CMD, "npm", "npm.cmd"} and not which_or_none(NPM_CMD):
        return f"missing {NPM_CMD} on PATH"
    cwd = Path(entry["cwd"])
    if not cwd.exists():
        return f"missing working directory: {repo_relative(cwd)}"
    if cwd == STUDIO_ROOT and command0 in {"node", NPM_CMD, "npm", "npm.cmd"}:
        if not (STUDIO_ROOT / "node_modules").exists():
            return "missing rhwp-studio/node_modules (run npm install in rhwp-studio)"
    if entry["id"] == "renderer-baseline-readiness":
        if not DEFAULT_MANIFEST.exists():
            return "missing scripts/renderer_baseline_manifest.json"
        if not STUDIO_ROOT.exists():
            return "missing rhwp-studio"
    return None


def run_job(entry: dict[str, Any], include_heavy: bool) -> dict[str, Any]:
    result = serialize_job(entry)
    reason = skip_reason(entry, include_heavy)
    if reason:
        result.update({"status": "skipped", "reason": reason, "exitCode": None, "seconds": 0.0})
        return result

    started = time.perf_counter()
    completed = subprocess.run(
        entry["command"],
        cwd=entry["cwd"],
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
    )
    elapsed = time.perf_counter() - started
    stdout = completed.stdout or ""
    stderr = completed.stderr or ""
    result.update(
        {
            "status": "passed" if completed.returncode == 0 else "failed",
            "exitCode": completed.returncode,
            "seconds": round(elapsed, 3),
            "stdoutTail": stdout[-4000:],
            "stderrTail": stderr[-4000:],
        }
    )
    if completed.returncode != 0:
        result["reason"] = f"exit {completed.returncode}"
    return result


def build_report(batches: list[int], include_heavy: bool, results: list[dict[str, Any]]) -> dict[str, Any]:
    counts = {"passed": 0, "failed": 0, "skipped": 0}
    for item in results:
        counts[item["status"]] = counts.get(item["status"], 0) + 1
    return {
        "schemaVersion": 1,
        "plan": repo_relative(PLAN_DOC),
        "manifest": repo_relative(DEFAULT_MANIFEST),
        "manifestUpdated": False,
        "manifestUpdatePolicy": (
            "Do not edit renderer_baseline_manifest.json from this driver. "
            "Threshold/sample changes stay on the existing renderer-contract "
            "and renderer_baseline review process."
        ),
        "platform": sys.platform,
        "batches": [
            {"number": number, "name": BATCH_NAMES[number]} for number in batches
        ],
        "heavy": include_heavy,
        "summary": counts,
        "results": results,
    }


def print_listing(batches: list[int]) -> None:
    print("canvaskit-parity batch map (existing harnesses only)")
    print(f"plan: {repo_relative(PLAN_DOC)}")
    print("manifest updates: never from this driver")
    for number in batches:
        print(f"\nbatch {number}: {BATCH_NAMES[number]}")
        for entry in batch_jobs()[number]:
            marker = " [heavy]" if entry["heavy"] else ""
            command = " ".join(entry["command"])
            print(f"  - {entry['id']}{marker}")
            print(f"      cwd: {repo_relative(entry['cwd'])}")
            print(f"      cmd: {command}")


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    batches = parse_batches(args.batches)
    if args.list:
        print_listing(batches)
        return 0

    seen_ids: set[str] = set()
    results: list[dict[str, Any]] = []
    for number in batches:
        for entry in batch_jobs()[number]:
            if entry["id"] in seen_ids:
                continue
            seen_ids.add(entry["id"])
            result = run_job(entry, args.heavy)
            result["batch"] = number
            result["batchName"] = BATCH_NAMES[number]
            results.append(result)
            status = result["status"]
            detail = result.get("reason") or ""
            print(f"[{status}] {entry['id']} {detail}".rstrip())

    report = build_report(batches, args.heavy, results)
    payload = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        output_path = Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(payload, encoding="utf-8")
        print(f"wrote {repo_relative(output_path)}")
    else:
        sys.stdout.write(payload)
    return 0 if report["summary"]["failed"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
