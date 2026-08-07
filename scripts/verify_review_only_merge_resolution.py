#!/usr/bin/env python3
"""Validate that a conflicted current-base merge resolved only mydocs paths."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


SAFE_REASON = "current-base-merge-resolution-mydocs-only"


def remerge_resolution_paths(repository: Path, merge_sha: str) -> list[str]:
    result = subprocess.run(
        [
            "git",
            "-C",
            str(repository),
            "show",
            "--remerge-diff",
            "--format=",
            "--name-only",
            "--no-renames",
            "-z",
            merge_sha,
        ],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", "replace").strip()
        raise RuntimeError(detail or "git show --remerge-diff failed")
    return [
        path.decode("utf-8", "surrogateescape")
        for path in result.stdout.split(b"\0")
        if path
    ]


def is_current_base_merge(repository: Path, merge_sha: str, base_sha: str) -> bool:
    result = subprocess.run(
        ["git", "-C", str(repository), "rev-list", "--parents", "-n", "1", merge_sha],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        return False
    parts = result.stdout.decode("ascii", "replace").split()
    return len(parts) == 3 and parts[1:].count(base_sha) == 1


def is_mydocs_only(paths: list[str]) -> bool:
    return bool(paths) and all(path.startswith("mydocs/") for path in paths)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--base-sha", required=True)
    parser.add_argument("merge_sha")
    args = parser.parse_args()

    if not is_current_base_merge(args.repository, args.merge_sha, args.base_sha):
        print("current-base-merge-resolution-invalid-merge", file=sys.stderr)
        return 1

    try:
        paths = remerge_resolution_paths(args.repository, args.merge_sha)
    except RuntimeError as error:
        print(f"current-base-merge-resolution-unavailable: {error}", file=sys.stderr)
        return 1

    if not paths:
        print("current-base-merge-resolution-empty", file=sys.stderr)
        return 1
    if not is_mydocs_only(paths):
        print("current-base-merge-resolution-not-mydocs", file=sys.stderr)
        return 1

    print(SAFE_REASON)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
