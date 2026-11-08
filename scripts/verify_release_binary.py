"""Verify PNG capability in the binary extracted from a release archive.

This is a packaging smoke check, not a typesetting fidelity comparison.
Only the named binary is extracted; no archive paths are trusted.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import struct
import subprocess
import tarfile
import tempfile
import zipfile


def run(binary: Path, *args: str) -> str:
    result = subprocess.run(
        [str(binary), *args],
        capture_output=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
        check=False,
    )
    if result.returncode:
        raise RuntimeError(
            f"{args[0]} failed (exit {result.returncode}): "
            f"{result.stdout}\n{result.stderr}"
        )
    return result.stdout


def verify(archive: Path, sample: Path) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="rhwp-release-smoke-") as tmp:
        directory = Path(tmp)
        # The five official archives have one CLI at this exact member path.
        binary_name = "rhwp.exe" if archive.suffix == ".zip" else "rhwp"
        member_name = f"rhwp/{binary_name}"
        binary = directory / binary_name
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as source:
                if source.namelist().count(member_name) != 1:
                    raise ValueError(f"Expected exactly one {member_name}")
                binary.write_bytes(source.read(member_name))
        else:
            with tarfile.open(archive, "r:gz") as source:
                members = [m for m in source.getmembers() if m.name == member_name]
                if len(members) != 1 or not members[0].isfile():
                    raise ValueError(f"Expected exactly one regular {member_name}")
                stream = source.extractfile(members[0])
                assert stream is not None
                binary.write_bytes(stream.read())
        if os.name != "nt":
            binary.chmod(0o700)

        version = run(binary, "--version").strip()
        capabilities = json.loads(
            run(binary, "capabilities", "--search", "export-png", "--json")
        )
        entries = [
            item for item in capabilities["commands"]
            if item["name"] == "export-png"
        ]
        if len(entries) != 1 or entries[0].get("available") is not True:
            raise RuntimeError("Packaged CLI lacks the native-skia export-png capability")

        output = directory / "png"
        run(binary, "export-png", str(sample), "-p", "0", "-o", str(output))
        images = list(output.glob("*.png"))
        if len(images) != 1:
            raise RuntimeError(f"Expected one page PNG, got {len(images)}")
        header = images[0].read_bytes()[:33]
        if (
            len(header) != 33
            or header[:8] != b"\x89PNG\r\n\x1a\n"
            or header[8:16] != b"\x00\x00\x00\rIHDR"
        ):
            raise RuntimeError("Export did not produce a PNG with an IHDR header")
        width, height = struct.unpack(">II", header[16:24])
        if width == 0 or height == 0:
            raise RuntimeError("Export produced an empty PNG")
        return {
            "archive": archive.name,
            "version": version,
            "exportPngAvailable": True,
            "pngWidth": width,
            "pngHeight": height,
        }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--sample", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = verify(args.archive.resolve(strict=True), args.sample.resolve(strict=True))
    except (OSError, ValueError, KeyError, RuntimeError, subprocess.TimeoutExpired) as error:
        parser.exit(1, f"Release binary verification failed: {error}\n")
    print(json.dumps(result, ensure_ascii=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
