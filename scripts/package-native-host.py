#!/usr/bin/env python3
"""Package an explicitly supplied, capability-checked native host library."""

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile


SOURCE_ROOT = Path(__file__).resolve().parents[1]
LIBRARY_NAMES = {
    "Linux": "librhwp_native_ffi.so",
    "Darwin": "librhwp_native_ffi.dylib",
    "Windows": "rhwp_native_ffi.dll",
}
SOURCE_FILES = {
    **{
        f"examples/{name}": f"bindings/Native/examples/{name}"
        for name in (
            "native_host.py", "native_worker.py", "native_file_store.py",
            "qt_native_host.py", "qt-requirements.txt",
        )
    },
    "docs/README.md": "bindings/Native/examples/README.md",
    "docs/PACKAGING.md": "bindings/Native/examples/PACKAGING.md",
    "docs/QT_NATIVE_HOST.md": "bindings/Native/examples/QT_NATIVE_HOST.md",
    "LICENSE": "LICENSE",
}

# This is part of the package, not a launcher that depends on the source tree.
LAUNCHER = '''#!/usr/bin/env python3
"""Run the packaged native host from any working directory."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import sys


def architecture():
    machine = platform.machine().lower()
    return {"amd64": "x86_64", "arm64": "aarch64"}.get(machine, machine)


def checked_file(root, relative):
    if (not isinstance(relative, str) or "\\\\" in relative
            or relative.startswith("/")
            or any(part in ("", ".", "..") for part in relative.split("/"))):
        raise ValueError("invalid package-relative path")
    path = (root / relative).resolve(strict=True)
    path.relative_to(root)
    if not path.is_file():
        raise ValueError("package member is not a regular file")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("mode", choices=("headless", "window"))
    parser.add_argument("--input", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.mode == "headless" and args.output is None:
        parser.error("headless mode requires --output (a new HWPX file)")
    root = Path(__file__).resolve().parent
    try:
        with (root / "manifest.json").open("rb") as source:
            raw = source.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError("package manifest exceeds 1 MiB")
        manifest = json.loads(raw)
        if (manifest["schemaVersion"] != 1
                or manifest["platform"] != platform.system()
                or manifest["architecture"] != architecture()):
            raise ValueError("package platform/architecture does not match this host")
        files = manifest["files"]
        for relative, digest in files.items():
            path = checked_file(root, relative)
            if not isinstance(digest, str) or not re.fullmatch("[0-9a-f]{64}", digest):
                raise ValueError("invalid package digest")
            checksum = hashlib.sha256()
            with path.open("rb") as source:
                for chunk in iter(lambda: source.read(1024 * 1024), b""):
                    checksum.update(chunk)
            if checksum.hexdigest() != digest:
                raise ValueError("package digest mismatch: " + relative)
        artifact = manifest["artifact"]
        expected = {"Linux": "lib/librhwp_native_ffi.so",
                    "Darwin": "lib/librhwp_native_ffi.dylib",
                    "Windows": "lib/rhwp_native_ffi.dll"}[platform.system()]
        if artifact["path"] != expected or artifact["sha256"] != files[expected]:
            raise ValueError("invalid packaged library entry")
        library = checked_file(root, expected)
        example = "examples/" + ("native_host.py" if args.mode == "headless"
                                 else "qt_native_host.py")
        if example not in files:
            raise ValueError("missing example digest")
        command = [sys.executable, str(checked_file(root, example)),
                   "--library", str(library)]
        if args.input is not None:
            command.extend(["--input", str(args.input)])
        if args.output is not None:
            command.extend(["--output", str(args.output)])
        return subprocess.run(command, check=False).returncode
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        print("native host package: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
'''


def architecture():
    machine = platform.machine().lower()
    return {"amd64": "x86_64", "arm64": "aarch64"}.get(machine, machine)


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_capabilities(capabilities):
    if not isinstance(capabilities, dict):
        raise ValueError("native capabilities must be a JSON object")
    if (type(capabilities.get("abiVersion")) is not int
            or capabilities["abiVersion"] != 1):
        raise ValueError("native host packaging requires ABI version 1")
    for name in ("ok", "nativeSkia", "hostRequests", "worker"):
        if capabilities.get(name) is not True:
            raise ValueError(f"native host packaging requires {name}=true")
    # Preserve the actual response; do not silently turn non-JSON values into data.
    json.dumps(capabilities, allow_nan=False)
    return capabilities


def _probe_in_process(library):
    """Only called by the short-lived probe child, so Windows can release the DLL."""
    native = ctypes.CDLL(str(Path(library).resolve(strict=True)))
    capabilities = native.rhwp_session_capabilities
    capabilities.argtypes = []
    capabilities.restype = ctypes.c_void_p
    release = native.rhwp_string_free
    release.argtypes = [ctypes.c_void_p]
    release.restype = None
    # The claimed hostRequests capability must also have its dispatch entry point.
    getattr(native, "rhwp_native_host_request")
    pointer = capabilities()
    if not pointer:
        raise ValueError("native capabilities returned a null pointer")
    try:
        response = json.loads(ctypes.string_at(pointer).decode("utf-8"))
    finally:
        release(pointer)
    return validate_capabilities(response)


def probe_library(library):
    """Load the supplied library in a subprocess; never search for a substitute."""
    result = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--probe-library", str(library)],
        capture_output=True, text=True, encoding="utf-8", timeout=30, check=False,
    )
    if result.returncode != 0:
        raise ValueError("native capability probe failed: " + result.stderr[-2000:].strip())
    if len(result.stdout) > 64 * 1024:
        raise ValueError("native capability response exceeds 64 KiB")
    return validate_capabilities(json.loads(result.stdout))


def package(library, output, declared_source_revision, *, source_root=SOURCE_ROOT):
    """Create one new package. The binary/revision association is caller-declared.

    The explicit binary is trusted executable code, not sandboxed input. Only
    this call's private staging directory and exclusively created output can be
    removed on failure; existing output paths (including symlinks) are rejected.
    """
    output = Path(output).absolute()
    if os.path.lexists(output):
        raise FileExistsError(f"output already exists: {output}")
    if not re.fullmatch(r"[0-9a-fA-F]{40}", declared_source_revision):
        raise ValueError("declared source revision must be a full 40-digit Git commit ID")
    system = platform.system()
    if system not in LIBRARY_NAMES:
        raise ValueError(f"unsupported native host platform: {system}")
    library = Path(library).resolve(strict=True)
    suffix = Path(LIBRARY_NAMES[system]).suffix
    if not library.is_file() or library.suffix.lower() != suffix:
        raise ValueError(f"--library must be an explicit regular {suffix} file")
    source_root = Path(source_root).resolve(strict=True)
    sources = {destination: source_root / source for destination, source in SOURCE_FILES.items()}
    for source in sources.values():
        if not source.is_file():
            raise ValueError(f"missing package source file: {source}")
    if not output.parent.is_dir():
        raise ValueError("output parent directory must already exist")
    created = None
    try:
        with tempfile.TemporaryDirectory(prefix=".rhwp-native-package-", dir=output.parent) as temporary:
            staging = Path(temporary)
            artifact_path = "lib/" + LIBRARY_NAMES[system]
            artifact = staging / artifact_path
            artifact.parent.mkdir()
            shutil.copyfile(library, artifact)
            capabilities = validate_capabilities(probe_library(artifact))
            for destination, source in sources.items():
                path = staging / destination
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, path)
            (staging / "run.py").write_text(LAUNCHER, encoding="utf-8")
            files = {
                relative: sha256(staging / relative)
                for relative in sorted([artifact_path, "run.py", *sources])
            }
            manifest = {
                "schemaVersion": 1,
                "platform": system,
                "architecture": architecture(),
                "declaredSourceRevision": declared_source_revision.lower(),
                "sourceRevisionEvidence": "caller-declared; binary provenance not verified",
                "artifact": {"path": artifact_path, "sha256": files[artifact_path]},
                "capabilities": capabilities,
                "files": files,
            }
            (staging / "manifest.json").write_text(
                json.dumps(manifest, ensure_ascii=False, allow_nan=False, indent=2) + "\n",
                encoding="utf-8",
            )
            # Reserve the destination exclusively. rename(staging, output) alone
            # can overwrite an existing empty directory on POSIX.
            output.mkdir()
            created = output.stat()
            for child in sorted(staging.iterdir(), key=lambda path: path.name == "manifest.json"):
                child.rename(output / child.name)
            return manifest
    except BaseException:
        if created is not None:
            try:
                current = output.lstat()
                if (current.st_dev, current.st_ino) == (created.st_dev, created.st_ino):
                    shutil.rmtree(output)
            except FileNotFoundError:
                pass
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--library", type=Path, help="explicit current-OS native-skia shared library")
    parser.add_argument("--output", type=Path, help="new package directory; its parent must exist")
    parser.add_argument("--source-revision", help="caller-declared full Git commit ID")
    parser.add_argument("--probe-library", type=Path, help="only inspect the explicit native library")
    args = parser.parse_args()
    if args.probe_library is not None:
        if any(value is not None for value in (args.library, args.output, args.source_revision)):
            parser.error("--probe-library cannot be combined with packaging arguments")
    elif any(value is None for value in (args.library, args.output, args.source_revision)):
        parser.error("packaging requires --library, --output and --source-revision")
    try:
        result = (_probe_in_process(args.probe_library) if args.probe_library is not None
                  else package(args.library, args.output, args.source_revision))
    except (OSError, ValueError, AttributeError, subprocess.SubprocessError) as error:
        print(json.dumps({"ok": False, "error": str(error)}, ensure_ascii=False), file=sys.stderr)
        return 1
    print(json.dumps(result, ensure_ascii=False, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
