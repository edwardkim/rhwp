#!/usr/bin/env python3
"""실제 native library의 문서/버퍼 수명을 검증한다 (WASM/DOM 불필요)."""
import argparse
import ctypes
import json
import threading


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def accepted(response):
    require(response.get("ok") is True, f"request unexpectedly rejected: {response}")
    return response


def rejected(response, code):
    require(response.get("ok") is False and response.get("code") == code,
            f"expected {code}, got {response}")


class Host:
    def __init__(self, library):
        self.lib = ctypes.CDLL(str(library))
        signatures = {
            "capabilities": [],
            "open": [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t],
            "request": [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t],
            "close": [ctypes.c_uint64],
            "buffer_copy": [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t],
            "buffer_release": [ctypes.c_uint64],
        }
        for name, arguments in signatures.items():
            fn = getattr(self.lib, "rhwp_session_" + name)
            fn.argtypes = arguments
            fn.restype = ctypes.c_void_p
        self.lib.rhwp_string_free.argtypes = [ctypes.c_void_p]
        self.lib.rhwp_string_free.restype = None

    def call(self, name, *arguments):
        pointer = getattr(self.lib, "rhwp_session_" + name)(*arguments)
        require(bool(pointer), f"{name} returned a null result")
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            self.lib.rhwp_string_free(pointer)

    def request(self, handle, **request):
        data = json.dumps(request, ensure_ascii=False, allow_nan=False).encode()
        return self.call("request", handle, data, len(data))


def on_foreign_thread(operation):
    results = []

    def invoke():
        try:
            results.append(operation())
        except BaseException as error:
            results.append(error)

    thread = threading.Thread(target=invoke, daemon=True)
    thread.start()
    thread.join(timeout=10)
    require(not thread.is_alive() and len(results) == 1, "wrong-thread request did not finish")
    if isinstance(results[0], BaseException):
        raise results[0]
    return results[0]


def copy_buffer_contract(host, result):
    """Use owned, valid destination memory even for rejected ABI calls."""
    accepted(result)
    length = result["byteLength"]
    require(length > 0, "expected nonempty output")
    output = (ctypes.c_uint8 * (length + 8))()
    ctypes.memset(output, 0xA5, len(output))
    original = bytes(output)
    buffer = result["buffer"]
    rejected(host.call("buffer_copy", buffer, None, length), "invalidArgument")
    rejected(host.call("buffer_copy", buffer, output, length - 1), "invalidArgument")
    rejected(on_foreign_thread(lambda: host.call("buffer_copy", buffer, output, length)),
             "invalidArgument")
    rejected(on_foreign_thread(lambda: host.call("buffer_release", buffer)), "invalidArgument")
    require(bytes(output) == original, "rejected copy modified caller memory")
    accepted(host.call("buffer_copy", buffer, output, len(output)))
    require(bytes(output[length:]) == original[length:], "copy exceeded its returned byte length")
    return bytes(output[:length])


def release_buffer_contract(host, result):
    buffer = result["buffer"]
    accepted(host.call("buffer_release", buffer))
    rejected(host.call("buffer_release", buffer), "invalidArgument")
    output = (ctypes.c_uint8 * 1)(17)
    rejected(host.call("buffer_copy", buffer, output, 1), "invalidArgument")
    require(output[0] == 17, "released buffer changed caller memory")


def body(offset=0):
    return {"section": 0, "paragraph": 0, "scalarOffset": offset, "cellPath": []}


def edit_and_save_contract(host, handle, other):
    """Editing/serialization stay available without native-skia; pixels do not."""
    info = accepted(host.request(handle, op="info"))
    revision = info["revision"]
    caret = accepted(host.request(handle, op="caret", position=body()))["caret"]
    hit = accepted(host.request(handle, op="hitTest", page=caret["page"],
                                x=caret["x"], y=caret["y"] + caret["height"] / 2))["hit"]
    require(hit["position"]["cellPath"] == [], "blank document hit must address body text")
    selected = {"anchor": body(), "focus": body()}
    require(isinstance(accepted(host.request(handle, op="selectionRects", selection=selected))["rects"], list),
            "selection geometry must have a typed list response")
    cell = body()
    cell["cellPath"] = [{"control": 0, "cell": 0, "paragraph": 0}]
    rejected(host.request(handle, op="beginEdit", expectedRevision=revision,
                          selection={"anchor": cell, "focus": cell}), "unsupported")
    require(accepted(host.request(handle, op="info"))["revision"] == revision,
            "failed beginEdit lost or mutated the viewer session")
    outcome = accepted(host.request(handle, op="beginEdit", expectedRevision=revision,
                                    selection=selected))["outcome"]
    require(outcome["revision"] == revision and not outcome["documentChanged"],
            "opening an editor must preserve document revision and content")
    rejected(host.request(handle, op="replaceSelection", text="x", expectedRevision=revision + 1),
             "staleRevision")
    outcome = accepted(host.request(handle, op="beginComposition", expectedRevision=revision))["outcome"]
    composition = outcome["compositionId"]
    require(composition is not None, "beginComposition must return an event identity")
    outcome = accepted(host.request(handle, op="updateComposition", text="ㄱ", compositionId=composition,
                                    expectedRevision=revision))["outcome"]
    for operation in ("updateComposition", "commitComposition", "cancelComposition"):
        extra = {"text": "stale"} if operation == "updateComposition" else {}
        rejected(host.request(handle, op=operation, compositionId=composition,
                              expectedRevision=revision, **extra), "staleRevision")
    active = accepted(host.request(handle, op="info"))
    require(active["revision"] == outcome["revision"] and active["compositionId"] == composition,
            "rejected stale events changed the composition")
    rejected(host.request(handle, op="prepareSave", format="hwpx"), "invalidArgument")
    for text in ("가", "각"):
        outcome = accepted(host.request(handle, op="updateComposition", text=text, compositionId=composition,
                                        expectedRevision=outcome["revision"]))["outcome"]
    outcome = accepted(host.request(handle, op="commitComposition", compositionId=composition,
                                    expectedRevision=outcome["revision"]))["outcome"]
    outcome = accepted(host.request(handle, op="undo", expectedRevision=outcome["revision"]))["outcome"]
    info = accepted(host.request(handle, op="info"))
    require(not info["canUndo"] and info["canRedo"], "one composition must create one undo entry")
    unchanged = accepted(host.request(handle, op="replaceSelection", text="",
                                      expectedRevision=outcome["revision"]))["outcome"]
    require(unchanged["revision"] == outcome["revision"] and not unchanged["documentChanged"],
            "empty replacement must be a no-op")
    require(accepted(host.request(handle, op="info"))["canRedo"], "no-op discarded redo")
    outcome = accepted(host.request(handle, op="beginComposition", expectedRevision=outcome["revision"]))["outcome"]
    cancelled = outcome["compositionId"]
    outcome = accepted(host.request(handle, op="updateComposition", text="나", compositionId=cancelled,
                                    expectedRevision=outcome["revision"]))["outcome"]
    outcome = accepted(host.request(handle, op="cancelComposition", compositionId=cancelled,
                                    expectedRevision=outcome["revision"]))["outcome"]
    require(accepted(host.request(handle, op="info"))["canRedo"], "cancelled preedit discarded redo")
    outcome = accepted(host.request(handle, op="beginComposition", expectedRevision=outcome["revision"]))["outcome"]
    current = outcome["compositionId"]
    require(current != cancelled, "composition identities were reused")
    rejected(host.request(handle, op="updateComposition", text="late", compositionId=cancelled,
                          expectedRevision=outcome["revision"]), "invalidArgument")
    outcome = accepted(host.request(handle, op="cancelComposition", compositionId=current,
                                    expectedRevision=outcome["revision"]))["outcome"]
    outcome = accepted(host.request(handle, op="redo", expectedRevision=outcome["revision"]))["outcome"]

    prepared = accepted(host.request(handle, op="prepareSave", format="hwpx"))
    require(prepared["revision"] == outcome["revision"], "save bytes must identify their revision")
    require(prepared["caretSaved"] is False and prepared["caretPreservedFromSource"] is False,
            "new blank HWPX must not claim to preserve source settings or current editor focus")
    require("report" in prepared, "prepared bytes must carry their serializer report")
    rejected(host.request(handle, op="prepareSave", format="unknown"), "invalidArgument")
    rejected(host.request(handle, op="acknowledgeSave", token=0), "invalidArgument")
    rejected(host.request(other, op="acknowledgeSave", token=prepared["token"]), "invalidArgument")
    serialized = copy_buffer_contract(host, prepared)
    release_buffer_contract(host, prepared)
    require(serialized.startswith(b"PK"), "HWPX output must be a ZIP container")
    # A host may finish persisting revision N after an edit has produced N+1.
    outcome = accepted(host.request(handle, op="replaceSelection", text="나",
                                    expectedRevision=outcome["revision"]))["outcome"]
    saved = accepted(host.request(handle, op="acknowledgeSave", token=prepared["token"]))
    require(saved["isDirty"] is True, "delayed save success cleared a newer edit")
    rejected(host.request(handle, op="acknowledgeSave", token=prepared["token"]), "invalidArgument")

    obsolete = accepted(host.request(handle, op="prepareSave", format="hwpx"))
    pending = accepted(host.request(handle, op="prepareSave", format="hwpx"))
    rejected(host.request(handle, op="acknowledgeSave", token=obsolete["token"]), "invalidArgument")
    copy_buffer_contract(host, obsolete)  # Superseding a token does not revoke owned bytes.
    release_buffer_contract(host, obsolete)
    copy_buffer_contract(host, pending)
    release_buffer_contract(host, pending)
    cancelled_save = accepted(host.request(handle, op="cancelSave", token=pending["token"]))
    require(cancelled_save["isDirty"] is True, "failed/cancelled host save cleared dirty state")
    rejected(host.request(handle, op="acknowledgeSave", token=pending["token"]), "invalidArgument")

    final = accepted(host.request(handle, op="prepareSave", format="hwpx"))
    copy_buffer_contract(host, final)
    saved = accepted(host.request(handle, op="acknowledgeSave", token=final["token"]))
    require(saved["isDirty"] is False, "acknowledging current bytes did not mark the session clean")
    rejected(host.request(handle, op="cancelSave", token=final["token"]), "invalidArgument")
    # Caller keeps this buffer alive across document close, in either build mode.
    return final


def run(host, expected_native_skia):
    caps = accepted(host.call("capabilities"))
    require(caps["abiVersion"] == 1, "expected native ABI v1")
    require(caps["nativeSkia"] is expected_native_skia,
            f"wrong build artifact: expected nativeSkia={expected_native_skia}, got {caps['nativeSkia']}")
    rejected(host.call("open", None, 1, None, 0), "invalidArgument")
    rejected(host.call("open", None, caps["maxDocumentBytes"] + 1, None, 0), "limitExceeded")
    rejected(host.call("open", None, 0, None, 1), "invalidArgument")
    rejected(host.call("open", None, 0, None, 4097), "limitExceeded")
    a = accepted(host.call("open", None, 0, None, 0))["handle"]
    b = None
    reopened = None
    held = None
    try:
        b = accepted(host.call("open", None, 0, None, 0))["handle"]
        require(a != b, "live sessions shared a handle")
        require(accepted(host.request(a, op="info"))["pageCount"] >= 1, "blank page is missing")
        require(accepted(host.request(a, op="pageSize", page=0))["size"]["width"] > 0,
                "blank page has no width")
        rejected(host.request(a, op="pageSize", page=2**32 - 1), "invalidArgument")
        for operation in ("info", "fontState", "clearCaches", "cacheStats"):
            rejected(host.request(a, op=operation, unexpected=True), "invalidArgument")
        rejected(host.request(a, op="render", page=0, scale=-1), "invalidArgument")
        rejected(host.call("request", a, None, 1), "invalidArgument")
        rejected(host.call("request", a, None, caps["maxRequestBytes"] + 1), "limitExceeded")
        for malformed in (b"\xff", b"{", b'{"op":"missingOperation"}'):
            rejected(host.call("request", a, malformed, len(malformed)), "invalidArgument")
        rejected(on_foreign_thread(lambda: host.request(a, op="info")), "invalidArgument")
        rejected(on_foreign_thread(lambda: host.call("close", a)), "invalidArgument")
        accepted(host.request(a, op="info"))

        frame = host.request(a, op="render", page=0, scale=0.25)
        region = dict(op="renderRegion", page=0, x=0, y=0, width=8, height=8, scale=1)
        if expected_native_skia:
            accepted(frame)
            require(frame["pixelFormat"] == "rgba8Premultiplied", "unexpected pixel format")
            require(frame["byteLength"] == frame["stride"] * frame["height"], "invalid frame extent")
            pixels = copy_buffer_contract(host, frame)
            require(all(pixels[i] == 255 for i in range(3, len(pixels), 4)), "opaque page has nonopaque alpha")
            release_buffer_contract(host, frame)
            cropped = accepted(host.request(a, **region))
            copy_buffer_contract(host, cropped)
            release_buffer_contract(host, cropped)
        else:
            rejected(frame, "featureDisabled")
            rejected(host.request(a, **region), "featureDisabled")
            before = accepted(host.request(a, op="fontState"))
            rejected(host.request(a, op="setFonts", paths=["rhwp-probe-absent-font.ttf"]), "featureDisabled")
            require(accepted(host.request(a, op="fontState")) == before, "disabled font update changed state")
            rejected(host.request(a, op="cacheStats"), "featureDisabled")
            accepted(host.request(a, op="clearCaches"))

        held = edit_and_save_contract(host, a, b)
        accepted(host.call("close", a))
        rejected(host.request(a, op="info"), "invalidArgument")
        rejected(host.call("close", a), "invalidArgument")
        data = copy_buffer_contract(host, held)
        release_buffer_contract(host, held)
        held = None
        accepted(host.request(b, op="info"))
        reopened = accepted(host.call("open", data, len(data), None, 0))["handle"]
        info = accepted(host.request(reopened, op="info"))
        require(info["pageCount"] >= 1 and info["isDirty"] is False, "saved bytes did not reopen cleanly")
    finally:
        if held is not None:
            host.call("buffer_release", held["buffer"])
        for handle in (reopened, b, a):
            if handle is not None:
                host.call("close", handle)
    print(json.dumps({"ok": True, "nativeSkia": caps["nativeSkia"], "contract": "native-session",
                      "editorAndSave": True, "expectedBuildChecked": True}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("library")
    parser.add_argument("--native-skia", choices=("enabled", "disabled"), required=True,
                        help="expected build feature; never infer the intended artifact from the library")
    args = parser.parse_args()
    run(Host(args.library), args.native_skia == "enabled")
