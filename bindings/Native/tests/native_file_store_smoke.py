#!/usr/bin/env python3
"""Exercise native save/recovery against an explicit library using temporary documents only."""

import argparse
import json
from pathlib import Path
import sys
import tempfile


sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_file_store import ExternalFileChanged, MAX_BYTES, NativeFileStore, read_document, read_recovery
from native_host import NativeHost, body_position, collapsed


class ObservedHost(NativeHost):
    """Track real C ABI ownership without substituting any native operations."""

    def __init__(self, library):
        self.handles = set()
        self.buffers = set()
        self.released_buffers = 0
        self.closed_handles = 0
        super().__init__(library)

    def call(self, name, *arguments):
        response = super().call(name, *arguments)
        if name == "open":
            self.handles.add(response["handle"])
        elif name == "close":
            self.handles.remove(arguments[0])
            self.closed_handles += 1
        elif name == "buffer_release":
            self.buffers.remove(arguments[0])
            self.released_buffers += 1
        if "buffer" in response:
            self.buffers.add(response["buffer"])
        return response

    def copy_buffer(self, response):
        if response["byteLength"] > MAX_BYTES:
            self.call("buffer_release", response["buffer"])
            raise ValueError("smoke buffer exceeds the document byte limit")
        return super().copy_buffer(response)

    def close_remaining(self):
        errors = []
        for buffer in tuple(self.buffers):
            try:
                self.call("buffer_release", buffer)
            except Exception as error:
                errors.append(error)
        for handle in tuple(self.handles):
            try:
                self.call("close", handle)
            except Exception as error:
                errors.append(error)
        if errors:
            raise RuntimeError("native smoke ownership cleanup failed") from errors[0]


def run(library):
    host = ObservedHost(library)
    capabilities = host.call("capabilities")
    if capabilities.get("abiVersion") != 1 or capabilities.get("nativeSkia") is not True:
        raise RuntimeError("native file smoke requires ABI v1 with native-skia")
    try:
        with tempfile.TemporaryDirectory(prefix="rhwp-native-file-smoke-") as directory:
            root = Path(directory)
            destination = root / "document.hwpx"
            handle = host.open_bytes()
            adapter = NativeFileStore(host, handle)
            outcome = host.request(
                handle, op="beginEdit", expectedRevision=0, selection=collapsed(body_position())
            )["outcome"]
            outcome = host.request(
                handle, op="replaceSelection", text="가", expectedRevision=outcome["revision"]
            )["outcome"]
            first = adapter.save(destination)
            if first["isDirty"] or host.request(handle, op="info")["isDirty"]:
                raise AssertionError("acknowledged first save must be clean")
            first_bytes, _ = read_document(destination)
            if host.read_text(destination).strip() != "가":
                raise AssertionError("first saved document text mismatch")

            outcome = host.request(
                handle, op="replaceSelection", text="나", expectedRevision=outcome["revision"]
            )["outcome"]
            recovery = adapter.autosave(root / "recovery", now=100, interval=30)
            if recovery is None:
                raise AssertionError("dirty revision did not create recovery data")
            header, recovered_bytes = read_recovery(recovery)
            if header["revision"] != outcome["revision"] or not host.request(handle, op="info")["isDirty"]:
                raise AssertionError("autosave must preserve the current dirty revision")
            current_bytes, _ = read_document(destination)
            if current_bytes != first_bytes:
                raise AssertionError("autosave overwrote the acknowledged original")

            recovered = host.open_bytes(recovered_bytes)
            recovered_info = host.request(recovered, op="info")
            if recovered_info["pageCount"] < 1 or recovered_info["isDirty"]:
                raise AssertionError("recovery payload did not reopen as a readable document")
            # Save the reopened native session to inspect its actual text through
            # the existing text ABI; inspecting only the recovery header is insufficient.
            recovery_copy = root / "reopened-recovery.hwpx"
            NativeFileStore(host, recovered).save(recovery_copy)
            if host.read_text(recovery_copy).strip() != "가나":
                raise AssertionError("reopened recovery lost the second edit")
            host.call("close", recovered)

            replaced = adapter.save()
            if replaced["isDirty"] or host.request(handle, op="info")["isDirty"]:
                raise AssertionError("acknowledged original replacement must be clean")
            if host.read_text(destination).strip() != "가나":
                raise AssertionError("original replacement lost edited text")
            host.request(handle, op="replaceSelection", text="다", expectedRevision=outcome["revision"])
            # Simulate another writer using the earlier valid, bounded HWPX bytes.
            # Every path is inside this call's TemporaryDirectory.
            destination.write_bytes(first_bytes)
            try:
                adapter.save()
            except ExternalFileChanged:
                pass
            else:
                raise AssertionError("external change was overwritten")
            changed_bytes, _ = read_document(destination)
            if changed_bytes != first_bytes or not host.request(handle, op="info")["isDirty"]:
                raise AssertionError("conflict must preserve external bytes and local dirty state")
            if host.buffers:
                raise AssertionError("file adapter retained a native buffer")
            summary = {
                "ok": True, "contract": "native-file-store",
                "firstSaveClean": True, "autosavePreservedOriginalAndDirty": True,
                "recoveryText": "가나", "replacementClean": True,
                "externalChangeRejected": True,
            }
    finally:
        host.close_remaining()
    if host.handles or host.buffers:
        raise AssertionError("native ownership remained after cleanup")
    summary.update(closedHandles=host.closed_handles, releasedBuffers=host.released_buffers,
                   temporaryFilesRemoved=True)
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--library", type=Path, required=True, help="explicit native-skia shared library")
    args = parser.parse_args()
    try:
        result = run(args.library)
    except Exception as error:
        print(json.dumps({"ok": False, "error": str(error)}, ensure_ascii=False), file=sys.stderr)
        return 1
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
