#!/usr/bin/env python3
"""Shared deterministic I/O and process guards for Issue #4963 Stage W5-2."""

from __future__ import annotations

import hashlib
import json
import os
import selectors
import signal
import subprocess
import time
from pathlib import Path, PurePosixPath
from typing import Any, Sequence


ROOT = Path(__file__).resolve().parents[1]
INVESTIGATION = ROOT / "mydocs" / "tech" / "investigations" / "issue-4963"
CONTRACT_PATH = INVESTIGATION / "oracle_stage2_contract.json"


class OracleStage2Error(RuntimeError):
    """A fail-closed Stage W5-2 contract violation."""


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def read_contract() -> dict[str, Any]:
    value = read_json(CONTRACT_PATH)
    if value.get("schemaVersion") != 1 or value.get("kind") != "font-oracle-stage2-contract":
        raise OracleStage2Error("Stage W5-2 contract identity mismatch")
    return value


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    ).encode("utf-8")


def pretty_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode(
        "utf-8"
    )


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _relative_parts(relative: str) -> tuple[str, ...]:
    pure = PurePosixPath(relative)
    if pure.is_absolute() or not pure.parts or any(part in {"", ".", ".."} for part in pure.parts):
        raise OracleStage2Error("path must be a non-empty relative path without traversal")
    return pure.parts


def regular_input(root: Path, relative: str, maximum_bytes: int) -> Path:
    if root.is_symlink():
        raise OracleStage2Error("input root must not be a symlink")
    root = root.resolve(strict=True)
    if not root.is_dir():
        raise OracleStage2Error("input root must be a real directory")
    parts = _relative_parts(relative)
    current = root
    for part in parts:
        current = current / part
        if current.is_symlink():
            raise OracleStage2Error("symlink inputs are forbidden")
    try:
        resolved = current.resolve(strict=True)
    except FileNotFoundError as error:
        raise OracleStage2Error("input file is missing") from error
    if root not in resolved.parents or not resolved.is_file():
        raise OracleStage2Error("input must be a regular file under the declared root")
    size = resolved.stat().st_size
    if size <= 0 or size > maximum_bytes:
        raise OracleStage2Error(f"input byte limit exceeded: {size} > {maximum_bytes}")
    return resolved


def output_path(root: Path, relative: str) -> Path:
    if root.is_symlink():
        raise OracleStage2Error("output root must not be a symlink")
    root.mkdir(parents=True, exist_ok=True)
    root = root.resolve(strict=True)
    if not root.is_dir():
        raise OracleStage2Error("output root must be a real directory")
    parts = _relative_parts(relative)
    current = root
    for part in parts[:-1]:
        current = current / part
        if current.exists() and current.is_symlink():
            raise OracleStage2Error("symlink output parents are forbidden")
        current.mkdir(exist_ok=True)
        if current.is_symlink() or not current.is_dir():
            raise OracleStage2Error("output parent must be a real directory")
    target = current / parts[-1]
    if target.exists() and target.is_symlink():
        raise OracleStage2Error("symlink outputs are forbidden")
    resolved_parent = target.parent.resolve(strict=True)
    if root != resolved_parent and root not in resolved_parent.parents:
        raise OracleStage2Error("output escaped the declared root")
    return target


def write_bytes(path: Path, payload: bytes, mode: int = 0o600) -> None:
    if path.exists() and path.is_symlink():
        raise OracleStage2Error("refusing to overwrite a symlink")
    path.write_bytes(payload)
    os.chmod(path, mode)


def write_json(path: Path, value: Any, mode: int = 0o600) -> None:
    write_bytes(path, pretty_json_bytes(value), mode=mode)


def run_bounded(
    command: Sequence[str],
    *,
    timeout_seconds: int,
    maximum_output_bytes: int,
    accepted_returncodes: set[int] | None = None,
) -> tuple[bytes, bytes]:
    """Run a child with wall/output limits and process-group termination."""

    accepted = accepted_returncodes or {0}
    process = subprocess.Popen(
        list(command),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    assert process.stdout is not None and process.stderr is not None
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ, "stdout")
    selector.register(process.stderr, selectors.EVENT_READ, "stderr")
    buffers = {"stdout": bytearray(), "stderr": bytearray()}
    deadline = time.monotonic() + timeout_seconds

    try:
        while selector.get_map():
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise OracleStage2Error("child process timeout exceeded")
            events = selector.select(timeout=min(remaining, 0.25))
            if not events and process.poll() is not None:
                events = [(key, selectors.EVENT_READ) for key in selector.get_map().values()]
            for key, _ in events:
                chunk = os.read(key.fileobj.fileno(), 64 * 1024)
                if not chunk:
                    selector.unregister(key.fileobj)
                    continue
                buffers[key.data].extend(chunk)
                if sum(len(value) for value in buffers.values()) > maximum_output_bytes:
                    raise OracleStage2Error("child process output byte limit exceeded")
        returncode = process.wait(timeout=max(0.1, deadline - time.monotonic()))
    except BaseException:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        raise
    finally:
        selector.close()
        process.stdout.close()
        process.stderr.close()

    if returncode not in accepted:
        stderr = bytes(buffers["stderr"]).decode("utf-8", "replace").strip()
        first_line = stderr.splitlines()[0] if stderr else "no diagnostic"
        raise OracleStage2Error(f"child process failed ({returncode}): {first_line}")
    return bytes(buffers["stdout"]), bytes(buffers["stderr"])
