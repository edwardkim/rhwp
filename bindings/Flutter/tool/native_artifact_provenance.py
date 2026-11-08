"""Validate an already extracted CI Native artifact without fetching or building."""

import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile
import time


MAX_ENTRIES = 8192
MAX_ARCHIVE_BYTES = 2 * 1024**3
MAX_NATIVE_BYTES = 512 * 1024**2
MAX_RECEIPT_BYTES = 32 * 1024**2
MAX_TREE_BYTES = 64 * 1024**2
MAX_TREE_ENTRIES = 250000
EXCLUDED_PREFIXES = (b"bindings/Flutter/", b"mydocs/pr/assets/issue8701/")
EXCLUDED_FILE = b".github/workflows/desktop-platform-qualification.yml"
TRANSPORTS = ("https", "http", "ssh", "git", "file", "ext")


def _sha(value, length, name):
    if not isinstance(value, str) or not re.fullmatch("[0-9a-f]{" + str(length) + "}", value):
        raise ValueError("invalid " + name)
    return value


def _signature(info):
    if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
        raise ValueError("artifact symlinks/reparse points are forbidden")
    return (info.st_mode, info.st_dev, info.st_ino, info.st_size,
            info.st_mtime_ns, info.st_ctime_ns, info.st_nlink)


def _entry_label(relative):
    label = json.dumps(relative, ensure_ascii=True)
    if len(label) > 160:
        label = label[:156] + '..."'
    return label


def _entry_description(relative, info):
    # Relative archive names only; never print the host path or file contents.
    kind = ("reparse" if getattr(info, "st_file_attributes", 0) & 0x400 else
            "symlink" if stat.S_ISLNK(info.st_mode) else
            "directory" if stat.S_ISDIR(info.st_mode) else
            "regular" if stat.S_ISREG(info.st_mode) else "other")
    return "entry=" + _entry_label(relative) + ", type=" + kind + ", nlink=" + str(info.st_nlink)[:20]


def _file_identity(info):
    value = _signature(info)
    if hasattr(info, "st_file_attributes"):
        # CPython 3.12 win32_xstat reports creation time as ctime, whereas
        # _Py_fstat_noraise reports ChangeTime there. Compare the explicitly
        # supported CreationTime on both sides, without rounding timestamps.
        birth = getattr(info, "st_birthtime_ns", None)
        if type(birth) is not int:
            raise ValueError("Windows stat birthtime_ns metadata is unavailable")
        value = value[:5] + (birth,) + value[6:]
    return value


def _read_error(message, relative, scan, path=None, fd=None, fd_before=None):
    names = ("mode", "dev", "ino", "size", "mtime_ns", "ctime_ns", "nlink")
    def numbers(info):
        if info is None:
            return None
        values = dict(zip(names, info)) if isinstance(info, tuple) else {
            name: getattr(info, "st_" + name, None)
            for name in (*names, "birthtime_ns", "file_attributes")}
        return {key: value if type(value) is int and value.bit_length() <= 128 else None
                for key, value in values.items()}
    context = {"scan": numbers(scan), "path": numbers(path), "fd": numbers(fd)}
    if fd_before is not None:
        context["fdBefore"] = numbers(fd_before)
    return ValueError(message + " (entry=" + _entry_label(relative) + "; " +
                      json.dumps(context, separators=(",", ":")) + ")")


def _scan(directory):
    supplied = Path(directory).absolute()
    _signature(supplied.lstat())
    root = supplied.resolve(strict=True)
    if not root.is_dir():
        raise ValueError("artifact root must be a regular directory")
    entries = {".": _signature(root.lstat())}
    pending, total = [(root, 0)], 0
    while pending:
        directory, depth = pending.pop()
        if depth > 32:
            raise ValueError("artifact directory depth exceeds its bound")
        with os.scandir(directory) as children:
            for child in children:
                path = Path(child.path)
                relative = path.relative_to(root).as_posix()
                if len(entries) >= MAX_ENTRIES or len(os.fsencode(relative)) > 8192:
                    raise ValueError("artifact entry/path bound exceeded")
                # Windows DirEntry.stat can report st_dev/st_ino/st_nlink=0.
                # Use the same authoritative non-following path metadata as
                # _read(), then bind it to the opened descriptor with fstat().
                info = path.lstat()
                try:
                    entries[relative] = _signature(info)
                except ValueError as error:
                    raise ValueError(str(error) + " (" + _entry_description(relative, info) + ")") from None
                if stat.S_ISDIR(info.st_mode):
                    pending.append((path, depth + 1))
                elif stat.S_ISREG(info.st_mode) and info.st_nlink == 1:
                    if info.st_size > MAX_NATIVE_BYTES:
                        raise ValueError("artifact file exceeds its byte bound")
                    total += info.st_size
                    if total > MAX_ARCHIVE_BYTES:
                        raise ValueError("artifact archive exceeds its byte bound")
                else:
                    raise ValueError("artifact entries must be regular files/directories without hardlinks (" +
                                     _entry_description(relative, info) + ")")
    return root, entries


def _unique(root, entries, name):
    matches = [root / path for path, info in entries.items()
               if stat.S_ISREG(info[0]) and Path(path).name.casefold() == name.casefold()]
    if len(matches) != 1:
        raise ValueError("artifact must contain exactly one " + name)
    return matches[0]


def _read(root, entries, path, maximum, *, hashed=False):
    relative = path.relative_to(root).as_posix()
    path.resolve(strict=True).relative_to(root)
    path_info = path.lstat()
    try:
        path_signature = _signature(path_info)
        path_identity = _file_identity(path_info)
    except ValueError as error:
        raise _read_error(str(error), relative, entries[relative], path_info) from None
    if path_signature != entries[relative]:
        raise _read_error("artifact changed before reading", relative, entries[relative], path_info)
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags)
    with os.fdopen(descriptor, "rb") as source:
        info = os.fstat(source.fileno())
        try:
            fd_signature = _signature(info)
            fd_identity = _file_identity(info)
        except ValueError as error:
            raise _read_error(str(error), relative, entries[relative], path_info, info) from None
        if (not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or
                fd_identity != path_identity or not 0 < info.st_size <= maximum):
            raise _read_error("artifact is changed, empty, or exceeds its read bound", relative,
                              entries[relative], path_info, info)
        if hashed:
            result = hashlib.sha256()
            count = 0
            for block in iter(lambda: source.read(1024 * 1024), b""):
                count += len(block)
                if count > maximum:
                    raise ValueError("artifact grew beyond its read bound")
                result.update(block)
            result = result.hexdigest()
        else:
            result = source.read(maximum + 1)
            if len(result) > maximum:
                raise ValueError("receipt exceeds its read bound")
        after = os.fstat(source.fileno())
        try:
            unchanged = _signature(after) == fd_signature and _file_identity(after) == fd_identity
        except ValueError as error:
            raise _read_error(str(error), relative, entries[relative], path_info, after, fd_signature) from None
        # Keep the raw fd ChangeTime check in its own authority. The Windows
        # cross-path normalization above does not discard metadata mutations.
        if not unchanged:
            raise _read_error("artifact changed while reading", relative, entries[relative],
                              path_info, after, fd_signature)
    path.resolve(strict=True).relative_to(root)
    after_path = path.lstat()
    try:
        unchanged = _signature(after_path) == path_signature and _file_identity(after_path) == path_identity
    except ValueError as error:
        raise _read_error(str(error), relative, entries[relative], after_path, after) from None
    if not unchanged:
        raise _read_error("artifact path changed while reading", relative, entries[relative], after_path, after)
    return result


def _object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON receipt key")
        result[key] = value
    return result


def _receipt(root, entries, path):
    def invalid_constant(_):
        raise ValueError("nonfinite JSON receipt value")
    value = json.loads(_read(root, entries, path, MAX_RECEIPT_BYTES),
                       object_pairs_hook=_object, parse_constant=invalid_constant)
    if not isinstance(value, dict):
        raise ValueError("receipt must be an object")
    return value


def _git(repository, arguments, *, limit=MAX_TREE_BYTES, extra=(), allowed=(0,)):
    # Git 2.39 may ignore GIT_NO_LAZY_FETCH. Transport denials remain effective
    # there, including caller configurations which explicitly allow a protocol.
    options = ["-c", "protocol.allow=never", "-c", "core.fsmonitor=false", "-c", "gc.auto=0"]
    for protocol in TRANSPORTS:
        options += ["-c", "protocol." + protocol + ".allow=never"]
    for key in extra:
        options += ["-c", key + "=never"]
    env = {**os.environ, "GIT_NO_LAZY_FETCH": "1", "GIT_TERMINAL_PROMPT": "0",
           "GIT_PROTOCOL_FROM_USER": "0", "GIT_OPTIONAL_LOCKS": "0"}
    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as error:
        process = subprocess.Popen(["git", *options, *arguments], cwd=repository, env=env,
                                   stdin=subprocess.DEVNULL, stdout=output, stderr=error)
        deadline = time.monotonic() + 60
        try:
            while process.poll() is None:
                if (time.monotonic() >= deadline or os.fstat(output.fileno()).st_size > limit or
                        os.fstat(error.fileno()).st_size > 64 * 1024):
                    raise ValueError("Git metadata read exceeded its time/byte bound")
                time.sleep(.01)
            output.seek(0)
            data = output.read(limit + 1)
            if len(data) > limit or os.fstat(error.fileno()).st_size > 64 * 1024:
                raise ValueError("Git metadata exceeds its byte bound")
            if process.returncode not in allowed:
                raise ValueError("Git metadata unavailable; explicitly fetch the exact source objects before reuse")
            return data
        finally:
            if process.poll() is None:
                process.kill()
            process.wait()


def _source_equivalence(repository, expected):
    repository = Path(repository).resolve(strict=True)
    # Also override explicitly allowed custom remote-helper protocols. Reading
    # configuration names cannot itself fetch any object or expose credentials.
    keys = _git(repository, ["config", "--name-only", "--get-regexp", r"^protocol\..*\.allow$"],
                limit=16384, allowed=(0, 1)).decode("ascii").splitlines()
    if any(not re.fullmatch(r"protocol\.[A-Za-z0-9+_.-]+\.allow", key) for key in keys):
        raise ValueError("unsupported Git protocol configuration key")
    def git(arguments, limit=MAX_TREE_BYTES):
        return _git(repository, arguments, limit=limit, extra=keys)
    head = _sha(git(["rev-parse", "--verify", "HEAD^{commit}"], 128).decode().strip(), 40, "current source")
    if git(["status", "--porcelain=v1", "-z", "--untracked-files=all", "--ignore-submodules=none"]):
        raise ValueError("current source must be fully committed and clean")
    if git(["rev-parse", "--verify", expected + "^{commit}"], 128).decode().strip() != expected:
        raise ValueError("expected source is not an available exact commit")

    def manifest(commit):
        entries = git(["ls-tree", "-r", "-z", "--full-tree", commit]).split(b"\0")
        if len(entries) - 1 > MAX_TREE_ENTRIES:
            raise ValueError("Git source tree exceeds its entry bound")
        result = {}
        for entry in entries:
            if not entry:
                continue
            metadata, path = entry.split(b"\t", 1)
            mode, kind, oid = metadata.split()
            if mode not in (b"100644", b"100755", b"120000", b"160000") or kind not in (b"blob", b"commit"):
                raise ValueError("unsupported Git source entry")
            _sha(oid.decode("ascii"), 40, "Git object identity")
            if path in result:
                raise ValueError("duplicate Git source path")
            result[path] = metadata
        return result

    old, current = manifest(expected), manifest(head)
    def excluded(path):
        return path == EXCLUDED_FILE or path.startswith(EXCLUDED_PREFIXES)
    old_inputs = {path: value for path, value in old.items() if not excluded(path)}
    current_inputs = {path: value for path, value in current.items() if not excluded(path)}
    if old_inputs != current_inputs:
        changed = sum(old_inputs.get(path) != current_inputs.get(path) for path in old_inputs.keys() | current_inputs.keys())
        raise ValueError("Native build inputs differ outside the three permitted exclusions (" + str(changed) + " paths)")
    def digest(entries):
        return hashlib.sha256(b"".join(value + b"\t" + path + b"\0" for path, value in sorted(entries.items()))).hexdigest()
    changed = [os.fsdecode(path) for path in sorted(old.keys() | current.keys()) if old.get(path) != current.get(path)]
    if git(["rev-parse", "--verify", "HEAD^{commit}"], 128).decode().strip() != head or git(["status", "--porcelain=v1", "-z", "--untracked-files=all", "--ignore-submodules=none"]):
        raise ValueError("current source changed during provenance verification")
    return {"currentSourceHead": head, "nativeSourceHead": expected,
            "sourceEquivalenceVerified": True, "sourceManifestKind": "git-mode-type-blob-or-gitlink-oid-and-path",
            "sourceManifestSha256": digest(old_inputs), "currentSourceManifestSha256": digest(current_inputs),
            "nativeInputEntries": len(old_inputs), "excludedChangedPaths": changed,
            "excludedPaths": ["bindings/Flutter/**", "mydocs/pr/assets/issue8701/**",
                              ".github/workflows/desktop-platform-qualification.yml"]}


def _identity(value, source, platform=None):
    if _sha(value.get("source_head"), 40, "receipt source") != source:
        raise ValueError("receipt source differs from the requested Native source")
    if platform is not None and value.get("platform") != platform:
        raise ValueError("receipt platform differs from the requested target")


def _native_step(value, feature, target):
    steps = value.get("steps")
    if not isinstance(steps, list) or not 1 <= len(steps) <= 128:
        raise ValueError("missing bounded SDK build steps")
    matches = []
    for step in steps:
        if not isinstance(step, dict):
            raise ValueError("invalid SDK build step")
        command = step.get("command")
        if (not isinstance(command, list) or not 1 <= len(command) <= 256 or
                any(not isinstance(part, str) or not part or len(part) > 8192 for part in command)):
            raise ValueError("invalid SDK build command")
        def executable(part):
            return part.replace("\\", "/").rsplit("/", 1)[-1]
        cargo_index = (0 if executable(command[0]) in ("cargo", "cargo.exe") else
                       3 if executable(command[0]) in ("rustup", "rustup.exe") and
                       len(command) > 4 and command[1] == "run" else -1)
        cargo = (cargo_index >= 0 and len(command) > cargo_index + 1 and
                 executable(command[cargo_index]) in ("cargo", "cargo.exe") and
                 command[cargo_index + 1] == "build")
        def option(name, expected):
            positions = [index for index, part in enumerate(command) if part == name]
            return (len(positions) == 1 and positions[0] + 1 < len(command) and
                    command[positions[0] + 1] == expected)
        package = option("-p", "rhwp-native-ffi") or option("--package", "rhwp-native-ffi")
        manifests = [command[index + 1].replace("\\", "/") for index, part in enumerate(command)
                     if part == "--manifest-path" and index + 1 < len(command)]
        native_manifest = (len(manifests) == 1 and
                           (manifests[0] == "bindings/Native/Cargo.toml" or
                            manifests[0].endswith("/bindings/Native/Cargo.toml")))
        source = package if feature == "native-skia-d3d" else native_manifest
        if cargo and source and option("--features", feature) and option("--target", target):
            matches.append(step)
    if len(matches) != 1 or type(matches[0].get("exitCode")) is not int or matches[0]["exitCode"] != 0:
        raise ValueError("one successful Native cargo feature/target build step is required")
    return matches[0]["command"]


def _artifact_hash(receipt, key, name, size):
    artifacts = receipt.get("artifacts")
    if not isinstance(artifacts, dict):
        raise ValueError("missing Native artifact identity map")
    value = artifacts.get(key)
    if isinstance(value, dict):
        if "bytes" in value and (type(value["bytes"]) is not int or value["bytes"] != size):
            raise ValueError("Native artifact byte receipt is contradictory")
        if "path" in value:
            path = value["path"]
            if (not isinstance(path, str) or not path or len(path) > 8192 or "\0" in path or
                    path.replace("\\", "/").rsplit("/", 1)[-1].casefold() != name.casefold()):
                raise ValueError("historical Native artifact filename is contradictory")
        value = value.get("sha256")
    return _sha(value, 64, "Native SHA-256")


def load_native_artifact(artifact_root, repository, platform, expected_source):
    """Return verified local paths and a serializable source-equivalence proof.

    Receipt paths are historical CI paths and are never opened. Native reuse
    proves neither a new Rust build nor consumer/GUI/hardware qualification.
    """
    expected_source = _sha(expected_source, 40, "expected Native source")
    if platform not in ("windows", "macos"):
        raise ValueError("Native artifact target must be windows or macos")
    root, entries = _scan(artifact_root)
    name = "rhwp_native_ffi.dll" if platform == "windows" else "librhwp_native_ffi.dylib"
    native = _unique(root, entries, name)
    native_hash = _read(root, entries, native, MAX_NATIVE_BYTES, hashed=True)
    if platform == "windows":
        receipt_path = _unique(root, entries, "native-build-receipt.json")
        outer_path = _unique(root, entries, "windows-sdk-receipt.json")
        receipt, outer = (_receipt(root, entries, path) for path in (receipt_path, outer_path))
        _identity(receipt, expected_source)
        if "platform" in receipt:
            _identity(receipt, expected_source, "windows")
        _identity(outer, expected_source, "windows")
        if receipt.get("build") != "passed":
            raise ValueError("Native DLL build did not pass")
        fingerprint = _artifact_hash(receipt, name, name, native.stat().st_size)
        command = _native_step(outer, "native-skia-d3d", "x86_64-pc-windows-msvc")
        target = "x86_64-pc-windows-msvc"
        sdk_receipt_path = outer_path
        sdk_build = outer.get("build")
        producer_hash = None
    else:
        receipt_path = _unique(root, entries, "macos-sdk-receipt.json")
        producer_path = _unique(root, entries, "metal-producer.json")
        receipt, producer = (_receipt(root, entries, path) for path in (receipt_path, producer_path))
        _identity(receipt, expected_source, "macos")
        _identity(producer, expected_source, "macos")
        if (receipt.get("build") != "passed" or receipt.get("targetSdkCompileLinkVerified") is not True or
                receipt.get("metalProducerReadbackVerified") is not True or producer.get("status") != "passed" or
                producer.get("producerReadbackVerified") is not True):
            raise ValueError("actual Apple SDK/Metal producer qualification did not pass")
        target = receipt.get("rustTarget")
        if target not in ("aarch64-apple-darwin", "x86_64-apple-darwin"):
            raise ValueError("unsupported Mac Native target")
        if receipt.get("architecture") != {"aarch64-apple-darwin": "arm64", "x86_64-apple-darwin": "x86_64"}[target]:
            raise ValueError("Mac Native architecture/target receipts disagree")
        command = _native_step(receipt, "native-skia-metal", target)
        fingerprint = _artifact_hash(receipt, "native", name, native.stat().st_size)
        if producer.get("nativeSha256") != native_hash:
            raise ValueError("Metal producer used a different Native artifact")
        sdk_receipt_path = receipt_path
        sdk_build = receipt["build"]
        producer_hash = _read(root, entries, producer_path, MAX_RECEIPT_BYTES, hashed=True)
    if _sha(fingerprint, 64, "Native SHA-256") != native_hash:
        raise ValueError("Native artifact hash differs from its successful build receipt")
    proof = _source_equivalence(repository, expected_source)
    receipt_hash = _read(root, entries, receipt_path, MAX_RECEIPT_BYTES, hashed=True)
    sdk_receipt_hash = _read(root, entries, sdk_receipt_path, MAX_RECEIPT_BYTES, hashed=True)
    if _scan(root)[1] != entries:
        raise ValueError("artifact archive changed during provenance verification")
    proof.update(platform=platform, nativeRebuilt=False, nativeSha256=native_hash, nativeTarget=target,
                 nativeBuildCommand=command, nativeBuildPassed=True, originalSdkBuild=sdk_build,
                 nativeReceiptSha256=receipt_hash, sdkReceiptSha256=sdk_receipt_hash)
    if producer_hash is not None:
        proof["metalProducerReceiptSha256"] = producer_hash
    return {"native": native, "nativeReceipt": receipt_path, "proof": proof}
