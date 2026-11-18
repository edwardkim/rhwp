#!/usr/bin/env python3
"""Check an ELF artifact's declared glibc requirements against a Linux baseline.

This inspects version *needs*, not the versions an ELF exports. A successful
result is an ABI prerequisite; actual dependency loading and capability/smoke
checks still run separately on the supported runtime.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import sys


VERSION = re.compile(r"[0-9]+(?:\.[0-9]+)+\Z")


def version_tuple(value: str) -> tuple[int, ...]:
    if not VERSION.fullmatch(value):
        raise ValueError("glibc version must contain only dot-separated numbers")
    return tuple(int(part) for part in value.split("."))


def required_glibc_versions(text: str) -> list[str]:
    """Read GNU readelf's C-locale version-needs sections only."""
    in_needs = False
    versions: set[str] = set()
    for line in text.splitlines():
        if line.startswith("Version needs section "):
            in_needs = True
        elif line.startswith(("Version definition section ", "Version symbols section ")):
            in_needs = False
        if not in_needs:
            continue
        for token in re.findall(r"\bName:\s+(GLIBC_[^\s]+)", line):
            value = token.removeprefix("GLIBC_")
            # Unmapped nonnumeric ABI contracts are not silently declared safe.
            version_tuple(value)
            versions.add(value)
    return sorted(versions, key=version_tuple)


def verify(artifact: Path, maximum: str) -> dict:
    limit = version_tuple(maximum)
    artifact = artifact.resolve(strict=True)
    if not artifact.is_file():
        raise ValueError("artifact must be a regular ELF file")
    with artifact.open("rb") as source:
        header = source.read(64)
    if (len(header) < 20 or header[:4] != b"\x7fELF"
            or header[4] not in (1, 2) or header[5] not in (1, 2)):
        raise ValueError("artifact is not a supported ELF file")
    byte_order = "<" if header[5] == 1 else ">"
    kind, machine = struct.unpack_from(byte_order + "HH", header, 16)
    if kind not in (2, 3):
        raise ValueError("artifact must be an ELF executable or shared object")
    result = subprocess.run(
        ["readelf", "--version-info", "--wide", str(artifact)],
        capture_output=True, text=True, encoding="utf-8", errors="strict",
        env={**os.environ, "LC_ALL": "C"}, timeout=30, check=False,
    )
    if result.returncode:
        raise ValueError("readelf failed: " + result.stderr[-2000:].strip())
    # A warning about an invalid/missing ELF section must not become an empty
    # requirement list and a false success, even if readelf exits successfully.
    if result.stderr.strip():
        raise ValueError("readelf reported a diagnostic: " + result.stderr[-2000:].strip())
    versions = required_glibc_versions(result.stdout)
    if not versions and "No version information found in this file." not in result.stdout:
        # A valid version table may have other requirements but no glibc names.
        if "Version needs section " not in result.stdout:
            raise ValueError("readelf did not report a recognizable version-needs result")
    exceeded = [value for value in versions if version_tuple(value) > limit]
    if exceeded:
        raise ValueError(f"glibc baseline {maximum} exceeded: " + ", ".join(exceeded))
    digest = hashlib.sha256()
    with artifact.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return {
        "ok": True,
        "artifact": str(artifact),
        "sha256": digest.hexdigest(),
        "elfClass": 64 if header[4] == 2 else 32,
        "elfMachine": machine,
        "maxGlibc": maximum,
        "maxRequiredGlibc": versions[-1] if versions else None,
        "requiredGlibcVersions": versions,
        "scope": "declared ELF version needs; runtime loading verified separately",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--artifact", required=True, type=Path)
    parser.add_argument("--max-glibc", required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(verify(args.artifact, args.max_glibc), ensure_ascii=False))
        return 0
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        print(json.dumps({"ok": False, "error": str(error)}, ensure_ascii=False), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
