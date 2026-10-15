#!/usr/bin/env python3
"""Run direct-PDF contracts against an explicitly built native C ABI library."""

import argparse
import ctypes
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--native-skia", choices=("enabled", "disabled"), required=True)
    parser.add_argument("--sample", type=Path, help="known supported input for enabled checks")
    args = parser.parse_args()
    library = ctypes.CDLL(str(args.library.resolve(strict=True)))
    export = library.rhwp_export_pdf_direct
    export.argtypes = (ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int32)
    export.restype = ctypes.c_void_p
    release = library.rhwp_string_free
    release.argtypes = (ctypes.c_void_p,)
    release.restype = None
    checks = 0

    def require(condition, message):
        nonlocal checks
        if not condition:
            raise AssertionError(message)
        checks += 1

    def call(source, destination, page=-1):
        def encoded(value):
            return str(value).encode("utf-8") if isinstance(value, Path) else value

        pointer = export(encoded(source), encoded(destination), page)
        require(bool(pointer), "native result pointer is non-null")
        try:
            return json.loads(ctypes.string_at(pointer).decode("utf-8"))
        finally:
            release(pointer)

    with tempfile.TemporaryDirectory(prefix="rhwp-pdf-ffi-") as directory:
        root = Path(directory)
        output = root / "out.pdf"
        for source, destination, page in [
            (None, output, -1),
            (b"input.hwp", None, -1),
            (b"\xff", output, -1),
            (b"input.hwp", b"\xff", -1),
            (b"input.hwp", output, -2),
        ]:
            result = call(source, destination, page)
            require(result.get("ok") is False and bool(result.get("error")), "invalid ABI arguments rejected")
            require(not output.exists(), "invalid arguments do not create a PDF")

        if args.native_skia == "disabled":
            output.write_bytes(b"existing destination")
            result = call(root / "missing.hwp", output)
            require(result.get("code") == "featureDisabled" and result.get("ok") is False,
                    "disabled symbol reports featureDisabled before filesystem access")
            require(output.read_bytes() == b"existing destination", "disabled export preserves destination")
        else:
            if args.sample is None:
                parser.error("--sample is required for enabled checks")
            source = root / "input.hwp"
            shutil.copyfile(args.sample.resolve(strict=True), source)
            original = source.read_bytes()

            for bad_source in [root / "missing.hwp", root]:
                result = call(bad_source, output)
                require(result.get("ok") is False, "unreadable/non-file input rejected")
                require(not output.exists(), "input rejection leaves destination absent")
            if hasattr(os, "mkfifo"):
                fifo = root / "input.fifo"
                os.mkfifo(fifo)
                # Isolate the call so a regression to blocking FIFO open is bounded.
                probe = subprocess.run([
                    sys.executable, "-c", """
import ctypes, json, sys
library = ctypes.CDLL(sys.argv[1])
export = library.rhwp_export_pdf_direct
export.argtypes = (ctypes.c_char_p, ctypes.c_char_p, ctypes.c_int32)
export.restype = ctypes.c_void_p
library.rhwp_string_free.argtypes = (ctypes.c_void_p,)
library.rhwp_string_free.restype = None
pointer = export(sys.argv[2].encode(), sys.argv[3].encode(), -1)
assert pointer
try:
    print(ctypes.string_at(pointer).decode())
finally:
    library.rhwp_string_free(pointer)
""", str(args.library.resolve()), str(fifo), str(output)
                ], capture_output=True, text=True, timeout=10, check=False)
                require(probe.returncode == 0, f"FIFO rejection returns promptly: {probe.stderr}")
                result = json.loads(probe.stdout)
                require(result.get("ok") is False and "regular file" in result.get("error", ""),
                        "FIFO input is rejected before a blocking open")
                require(not output.exists(), "FIFO rejection leaves destination absent")
            malformed = root / "malformed.hwp"
            malformed.write_bytes(b"not an HWP document")
            result = call(malformed, output)
            require(result.get("ok") is False, "malformed input rejected")
            require(not output.exists(), "parse failure leaves destination absent")
            oversized = root / "oversized.hwp"
            with oversized.open("wb") as stream:
                stream.truncate(128 * 1024 * 1024 + 1)
            result = call(oversized, output)
            require(result.get("ok") is False and "128 MiB" in result.get("error", ""),
                    "oversized input is rejected before parsing")
            require(not output.exists(), "size rejection leaves destination absent")
            result = call(source, output, 2**31 - 1)
            require(result.get("ok") is False, "out-of-range page rejected")
            require(not output.exists(), "page selection failure leaves destination absent")

            output.write_bytes(b"existing destination")
            result = call(source, output)
            require(result.get("ok") is False, "existing PDF destination rejected")
            require(output.read_bytes() == b"existing destination", "existing destination is unchanged")
            require(call(source, source).get("ok") is False, "input cannot be overwritten")
            if hasattr(Path, "symlink_to"):
                alias = root / "input-link.pdf"
                alias.symlink_to(source)
                require(call(source, alias).get("ok") is False, "input symlink cannot be overwritten")
            alias = root / "input-hardlink.pdf"
            alias.hardlink_to(source)
            require(call(source, alias).get("ok") is False, "input hard link cannot be overwritten")
            require(source.read_bytes() == original, "source bytes are preserved")

            for page in [-1, 0]:
                destination = root / f"rendered-{page}.pdf"
                result = call(source, destination, page)
                require(result.get("ok") is True, f"direct export succeeded: {result}")
                require(result.get("files") == [str(destination)], "result identifies only the requested PDF")
                pdf = destination.read_bytes()
                require(pdf.startswith(b"%PDF-"), "output is a PDF")
                expected_pages = result["pageCount"] if page == -1 else 1
                require(len(re.findall(rb"/Type\s*/Page(?:\s|/|>)", pdf)) == expected_pages,
                        "PDF page objects match the selected pages")
    print(json.dumps({"nativeSkia": args.native_skia, "checks": checks}))


if __name__ == "__main__":
    main()
