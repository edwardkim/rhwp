#!/usr/bin/env python3
"""Reject stale, experimental or changed WASM packages before release packaging."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def verify(manifest_path: Path, package_dirs: list[Path], source_sha: str,
           require_consumption: bool = False) -> None:
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("success") is not True or manifest.get("source_dirty") is not False:
        raise ValueError("Release requires a successful build from clean tracked sources")
    if manifest.get("source_sha") != source_sha:
        raise ValueError("Release WASM source does not match the checkout")
    config = manifest.get("build_configuration", {})
    expected = {"profile": "release", "build_mode": "standard", "cdylib_only": "0",
                "crate_types": "rlib+cdylib", "lto_policy": "manifest"}
    if manifest.get("profile") != "release" or any(config.get(k) != v for k, v in expected.items()):
        raise ValueError("Release requires standard release, without the cdylib opt-in")
    if not manifest.get("tool_commands", {}).get("wasm-opt"):
        raise ValueError("Release requires observed wasm-opt execution")
    if not package_dirs:
        raise ValueError("At least one package directory is required")
    for directory in package_dirs:
        for name in ("rhwp.js", "rhwp_bg.wasm"):
            data = (directory / name).read_bytes()
            expected_artifact = manifest.get("artifacts", {}).get(name, {})
            if (expected_artifact.get("sha256") != hashlib.sha256(data).hexdigest()
                    or expected_artifact.get("bytes") != len(data)):
                raise ValueError(f"Release artifact changed: {directory / name}")
    if require_consumption:
        records = sorted((manifest_path.parent / "consumption").glob("*.json"))
        if not records:
            raise ValueError("Release requires actual browser WASM consumption evidence")
        expected_hash = manifest["artifacts"]["rhwp_bg.wasm"]["sha256"]
        for path in records:
            record = json.loads(path.read_text())
            received = record.get("received", [])
            if (record.get("matched") is not True
                    or record.get("source_sha") != source_sha
                    or record.get("profile") != "release"
                    or record.get("expected_sha256") != expected_hash
                    or not received
                    or any(item.get("status") != 200 or item.get("sha256") != expected_hash
                           for item in received)):
                raise ValueError(f"Release WASM consumption mismatch: {path}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--package-dir", type=Path, action="append", required=True)
    parser.add_argument("--require-consumption", action="store_true")
    args = parser.parse_args()
    source_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    try:
        verify(args.manifest, args.package_dir, source_sha, args.require_consumption)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f"Release WASM verification failed: {error}\n")
    print(f"Verified standard release WASM for {source_sha}")


if __name__ == "__main__":
    main()
