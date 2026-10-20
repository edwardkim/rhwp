#!/usr/bin/env python3
"""Headless native C ABI reference host; no WASM, DOM or third-party packages."""

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import sys


class NativeCallError(RuntimeError):
    def __init__(self, response):
        self.response = response
        self.code = response.get("code", "nativeError")
        super().__init__(f"{self.code}: {response.get('error', 'native request failed')}")


class NativeHost:
    """Keep all session calls on the creating thread; free every native result."""

    def __init__(self, library):
        self.library = ctypes.CDLL(str(Path(library).resolve(strict=True)))
        signatures = {
            "capabilities": [],
            "open": [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t],
            "request": [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t],
            "close": [ctypes.c_uint64],
            "buffer_copy": [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t],
            "buffer_release": [ctypes.c_uint64],
        }
        for name, arguments in signatures.items():
            function = getattr(self.library, "rhwp_session_" + name)
            function.argtypes = arguments
            function.restype = ctypes.c_void_p
        self.library.rhwp_string_free.argtypes = [ctypes.c_void_p]
        self.library.rhwp_string_free.restype = None
        self.library.rhwp_read_text.argtypes = [ctypes.c_char_p, ctypes.c_int32]
        self.library.rhwp_read_text.restype = ctypes.c_void_p

    def _result(self, pointer):
        if not pointer:
            raise RuntimeError("native function returned a null result")
        try:
            response = json.loads(ctypes.string_at(pointer).decode("utf-8"))
        finally:
            self.library.rhwp_string_free(pointer)
        if response.get("ok") is not True:
            raise NativeCallError(response)
        return response

    def call(self, name, *arguments):
        return self._result(getattr(self.library, "rhwp_session_" + name)(*arguments))

    def open_bytes(self, data=b""):
        storage = ctypes.create_string_buffer(data) if data else None
        return self.call("open", storage, len(data), None, 0)["handle"]

    def request(self, handle, **request):
        data = json.dumps(request, ensure_ascii=False, allow_nan=False).encode("utf-8")
        storage = ctypes.create_string_buffer(data)
        return self.call("request", handle, storage, len(data))

    def host_request(self, data=b"", **request):
        encoded = json.dumps(request, ensure_ascii=False, allow_nan=False).encode("utf-8")
        arguments = ctypes.create_string_buffer(encoded)
        payload = ctypes.create_string_buffer(data) if data else None
        function = self.library.rhwp_native_host_request
        function.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
        function.restype = ctypes.c_void_p
        return self._result(function(arguments, len(encoded), payload, len(data)))

    def copy_buffer(self, response):
        """Consume one retained buffer; ownership ends even when copying fails."""
        buffer = response["buffer"]
        try:
            length = response["byteLength"]
            if not isinstance(length, int) or length < 0:
                raise ValueError("invalid native buffer length")
            output = (ctypes.c_uint8 * length)()
            self.call("buffer_copy", buffer, output, length)
            return bytes(output)
        finally:
            self.call("buffer_release", buffer)

    def read_text(self, path):
        response = self._result(
            self.library.rhwp_read_text(str(Path(path).resolve()).encode("utf-8"), -1)
        )
        return "".join(page["text"] for page in response["pages"])


def write_new_file(path, data):
    """Create exclusively, flush, then close; never replace an existing file.

    Failed writes remove only the inode this call created. A different file put
    at that path while writing is preserved. This is a new-file example, not an
    implementation of replace-existing Save or a crash-recovery journal.
    """
    path = Path(path)
    descriptor = os.open(
        path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0), 0o600
    )
    identity = None
    try:
        identity = os.fstat(descriptor)
        remaining = memoryview(data)
        while remaining:
            written = os.write(descriptor, remaining)
            if written == 0:
                raise OSError("file write made no progress")
            remaining = remaining[written:]
        os.fsync(descriptor)
    except BaseException:
        # Close before unlink so this also works on Windows.
        closing = descriptor
        descriptor = None
        os.close(closing)
        if identity is not None:
            try:
                current = path.lstat()
                if (current.st_dev, current.st_ino) == (identity.st_dev, identity.st_ino):
                    path.unlink()
            except FileNotFoundError:
                pass
        raise
    finally:
        if descriptor is not None:
            os.close(descriptor)


def persist_new_document(host, handle, output):
    """Acknowledge only bytes successfully copied and written by this host."""
    prepared = host.request(handle, op="prepareSave", format="hwpx")
    token = prepared["token"]
    try:
        data = host.copy_buffer(prepared)
        write_new_file(output, data)
        saved = host.request(handle, op="acknowledgeSave", token=token)
    except BaseException as error:
        try:
            host.request(handle, op="cancelSave", token=token)
        except Exception as cancel_error:
            if hasattr(error, "add_note"):
                error.add_note(f"cancelSave also failed: {cancel_error}")
        # A failed acknowledgement leaves successfully written bytes in place.
        # It must not turn a real output file into silent data loss.
        raise
    return {
        "revision": prepared["revision"],
        "byteLength": len(data),
        "isDirty": saved["isDirty"],
        "report": prepared.get("report"),
        "caretSaved": prepared.get("caretSaved", False),
        "caretPreservedFromSource": prepared.get("caretPreservedFromSource", False),
    }


def body_position(offset=0):
    return {"section": 0, "paragraph": 0, "scalarOffset": offset, "cellPath": []}


def collapsed(position):
    return {"anchor": position.copy(), "focus": position.copy()}


def _input_bytes(path, maximum):
    if path is None:
        return b""
    path = Path(path)
    if not path.is_file():
        raise ValueError("--input must be a regular document file")
    with path.open("rb") as source:
        data = source.read(maximum + 1)
    if not data:
        raise ValueError("an empty --input file is not a blank-document request")
    if len(data) > maximum:
        raise ValueError("input exceeds the native document byte limit")
    return data


def run_demo(host, output, input_path=None):
    caps = host.call("capabilities")
    if caps["abiVersion"] != 1 or not caps["nativeSkia"]:
        raise RuntimeError("this reference flow requires ABI v1 with native-skia enabled")
    source = _input_bytes(input_path, caps["maxDocumentBytes"])
    edit_blank = input_path is None
    handle = host.open_bytes(source)
    try:
        info = host.request(handle, op="info")
        size = host.request(handle, op="pageSize", page=0)["size"]
        frame = host.request(
            handle, op="renderRegion", page=0, x=0.0, y=0.0,
            width=min(size["width"], 256.0), height=min(size["height"], 256.0), scale=1.0,
        )
        pixels = host.copy_buffer(frame)
        if len(pixels) != frame["stride"] * frame["height"]:
            raise RuntimeError("rendered buffer does not match its stride and height")
        render = {key: frame[key] for key in (
            "width", "height", "stride", "revision", "fontGeneration", "originX", "originY"
        )}
        render["sha256"] = hashlib.sha256(pixels).hexdigest()
        queries = {}
        try:
            if edit_blank:
                caret = host.request(handle, op="caret", position=body_position())["caret"]
                hit = host.request(handle, op="hitTest", page=caret["page"],
                                   x=caret["x"], y=caret["y"] + caret["height"] / 2)["hit"]
            else:
                hit = host.request(handle, op="hitTest", page=0,
                                   x=size["width"] / 2, y=size["height"] / 2)["hit"]
                caret = host.request(handle, op="caret", position=hit["position"])["caret"]
            queries = {"hit": hit, "caret": caret}
        except NativeCallError as error:
            if edit_blank or error.code != "unsupported":
                raise
            # Viewing a document containing inline objects does not require it
            # to satisfy the narrower body-text editing/interaction contract.
            queries = {"unsupported": error.response}

        if edit_blank:
            outcome = host.request(handle, op="beginEdit", expectedRevision=info["revision"],
                                   selection=collapsed(body_position()))["outcome"]
            outcome = host.request(handle, op="beginComposition",
                                   expectedRevision=outcome["revision"])["outcome"]
            composition_id = outcome["compositionId"]
            for preedit in ("ㄱ", "가", "각"):
                outcome = host.request(handle, op="updateComposition", text=preedit,
                                       compositionId=composition_id,
                                       expectedRevision=outcome["revision"])["outcome"]
            outcome = host.request(handle, op="commitComposition", compositionId=composition_id,
                                   expectedRevision=outcome["revision"])["outcome"]
            outcome = host.request(handle, op="undo", expectedRevision=outcome["revision"])["outcome"]
            if host.request(handle, op="info")["canUndo"]:
                raise RuntimeError("one IME composition must create only one undo entry")
            outcome = host.request(handle, op="redo", expectedRevision=outcome["revision"])["outcome"]
            queries["selectionRects"] = host.request(handle, op="selectionRects", selection={
                "anchor": body_position(), "focus": body_position(1),
            })["rects"]
        saved = persist_new_document(host, handle, output)
        if saved["isDirty"]:
            raise RuntimeError("unchanged saved revision should be clean after acknowledgement")
    finally:
        host.call("close", handle)

    reopened = host.open_bytes(Path(output).read_bytes())
    try:
        reopened_info = host.request(reopened, op="info")
        if reopened_info["pageCount"] < 1 or reopened_info["isDirty"]:
            raise RuntimeError("reopened output must contain a clean, readable document")
    finally:
        host.call("close", reopened)
    saved_text = host.read_text(output)
    if edit_blank and saved_text.strip() != "각":
        raise RuntimeError("saved/reopened text does not match the committed IME syllable")
    return {
        "ok": True, "mode": "blank-ime" if edit_blank else "input-viewer",
        "output": str(Path(output).resolve()), "render": render, "queries": queries,
        "save": saved, "reopenedPages": reopened_info["pageCount"],
        "imeRoundtripVerified": edit_blank, "savedTextCharacters": len(saved_text),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path, required=True, help="explicit native shared library")
    parser.add_argument("--output", type=Path, required=True, help="new .hwpx file; must not exist")
    parser.add_argument("--input", type=Path, help="view/query/save a copy; never run the IME edit demo on this input")
    args = parser.parse_args()
    try:
        result = run_demo(NativeHost(args.library), args.output, args.input)
    except (OSError, ValueError, RuntimeError, KeyError) as error:
        print(json.dumps({"ok": False, "error": str(error)}, ensure_ascii=False), file=sys.stderr)
        return 1
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
