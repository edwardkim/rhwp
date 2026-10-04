#!/usr/bin/env python3
"""Run the deployment wasm-pack path and preserve log-boundary wall timings."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
ANSI = re.compile(r"\x1b\[[0-9;]*m")


def capture(*args: str) -> str:
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def observe(line: str, elapsed: float, events: dict, commands: dict) -> None:
    line = ANSI.sub("", line)
    markers = {
        "cargo_start": "Compiling to Wasm",
        "bindgen_start": "Installing wasm-bindgen",
        "opt_start": "Optimizing wasm binaries",
        "package_done": "Done in",
    }
    for key, marker in markers.items():
        if marker in line:
            events.setdefault(key, elapsed)
    # wasm-pack 0.15.0 child::run logs the actual executable and arguments.
    if "wasm_pack::child" in line and "Running " in line:
        argv = shlex.split(line.split("Running ", 1)[1].strip())
        if (argv and Path(argv[0]).name in {"wasm-opt", "wasm-bindgen"}
                and argv[1:] != ["--version"]):
            commands[Path(argv[0]).name] = argv


def intervals(events: dict, total: float) -> dict:
    result = {}
    boundaries = [
        ("preparation", 0, events.get("cargo_start")),
        ("cargo", events.get("cargo_start"), events.get("bindgen_start")),
        ("bindgen_setup_and_run", events.get("bindgen_start"),
         events.get("opt_start", events.get("package_done"))),
        ("wasm_opt_and_finalize", events.get("opt_start"), events.get("package_done")),
        ("after_package", events.get("package_done"), total),
    ]
    for name, start, end in boundaries:
        result[name] = round(end - start, 3) if start is not None and end is not None else None
    return result


def validate(profile: str, returncode: int, commands: dict) -> None:
    if returncode:
        raise RuntimeError(f"wasm-pack failed with exit code {returncode}")
    if profile == "release" and "wasm-opt" not in commands:
        raise RuntimeError("Release validation requires an observed wasm-opt invocation")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=("dev", "release"), default="release")
    parser.add_argument("--output", type=Path, default=ROOT / "output/render-diff-build")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    manifest_path = args.output / "build.json"
    # A failed rerun must never expose the previous successful manifest.
    manifest_path.unlink(missing_ok=True)
    command = ["sh", "scripts/wasm-pack-locked.sh", "--target", "web", f"--{args.profile}"]
    result = {
        "source_sha": capture("git", "rev-parse", "HEAD"),
        "source_dirty": bool(capture("git", "status", "--porcelain", "--untracked-files=no")),
        "profile": args.profile,
        "command": command,
        "rustc": capture("rustc", "--version"),
        "wasm_pack": capture("wasm-pack", "--version"),
        "cargo_cache_hit": os.environ.get("RHWP_CARGO_CACHE_HIT", "unknown"),
        "cargo_cache_matched_key": os.environ.get("RHWP_CARGO_CACHE_MATCHED_KEY", "unknown"),
        "cargo_lock_sha256": digest(ROOT / "Cargo.lock"),
        "cargo_target_dir": os.environ.get("CARGO_TARGET_DIR", "target"),
        "cargo_build_jobs": os.environ.get("CARGO_BUILD_JOBS", "default"),
        "timing_method": "log receipt boundaries; includes setup/finalization, not isolated process timings",
        "success": False,
    }
    events, commands = {}, {}
    started = time.monotonic()
    try:
        with (args.output / "build.log").open("w") as log:
            child = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE,
                                     stderr=subprocess.STDOUT, text=True,
                                     env={**os.environ, "RUST_LOG": "wasm_pack::child=info"})
            for line in child.stdout:
                elapsed = time.monotonic() - started
                log.write(f"{elapsed:.6f} {line}")
                log.flush()
                print(line, end="", flush=True)
                observe(line, elapsed, events, commands)
            returncode = child.wait()
        result["wall_seconds"] = round(time.monotonic() - started, 3)
        result["returncode"] = returncode
        result["events_seconds"] = events
        result["phase_seconds_approx"] = intervals(events, result["wall_seconds"])
        result["tool_commands"] = commands
        validate(args.profile, returncode, commands)
        result["tools"] = {
            name: {"version": capture(argv[0], "--version"), "sha256": digest(Path(argv[0]))}
            for name, argv in commands.items()
        }
        result["artifacts"] = {
            name: {"sha256": digest(ROOT / "pkg" / name), "bytes": (ROOT / "pkg" / name).stat().st_size}
            for name in ("rhwp.js", "rhwp_bg.wasm")
        }
        result["success"] = True
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        result["error"] = str(error)
        print(f"WASM measurement failed: {error}", flush=True)
    manifest_path.write_text(json.dumps(result, indent=2) + "\n")
    rows = "\n".join(f"| {name} | {value if value is not None else 'unobserved'} |"
                     for name, value in result.get("phase_seconds_approx", {}).items())
    (args.output / "summary.md").write_text(
        f"### Render Diff WASM build ({args.profile})\n\n"
        f"Source: `{result['source_sha']}`; success: `{result['success']}`; "
        f"Cargo cache hit: `{result['cargo_cache_hit']}`.\n\n"
        f"Total: {result.get('wall_seconds', 'unobserved')} s. "
        "Approximate wall intervals from log receipt; setup and finalization included.\n\n"
        "| Interval | Seconds |\n| --- | ---: |\n" + rows + "\n\n"
        "Commands, tool versions/digests and artifact SHA-256: `build.json`; raw log: `build.log`.\n"
    )
    return 0 if result["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
