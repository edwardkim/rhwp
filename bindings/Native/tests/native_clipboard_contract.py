#!/usr/bin/env python3
"""Actual-library clipboard ABI contracts; no OS clipboard or visual assertions."""

import argparse
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_host import NativeCallError, NativeHost
from native_worker import NativeWorker


def position(paragraph=0, offset=0):
    return {"section": 0, "paragraph": paragraph, "scalarOffset": offset, "cellPath": []}


def selection(start=(0, 0), end=(0, 0)):
    return {"anchor": position(*start), "focus": position(*end)}


def rejected(code, operation):
    try:
        operation()
    except NativeCallError as error:
        assert error.code == code, error.response
    else:
        raise AssertionError(f"expected {code}")


def exercise(call, info):
    outcome = call("pasteText", 0, text="가😀나\r\n다")["outcome"]
    revision = outcome["revision"]
    assert outcome["documentChanged"] and outcome["historyChanged"], outcome
    assert outcome["selection"] == selection((1, 1), (1, 1)), outcome
    # Reversed selection crosses a paragraph and counts the emoji as one scalar.
    call("setSelection", revision, selection=selection((1, 1), (0, 1)))
    copied = call("copySelection", revision)
    assert copied["kind"] == "clipboard" and copied["outcome"] is None, copied
    assert copied["payload"]["plainText"] == "😀나\n다", copied
    assert isinstance(copied["payload"]["html"], str), copied
    assert info()["revision"] == revision, "copy changed the document revision"
    cut = call("cutSelection", revision)
    assert cut["kind"] == "clipboard" and cut["payload"] == copied["payload"], cut
    assert cut["outcome"]["documentChanged"] and cut["outcome"]["historyChanged"], cut
    revision = cut["outcome"]["revision"]
    revision = call("undo", revision)["outcome"]["revision"]
    assert call("copySelection", revision)["payload"] == copied["payload"]

    outcome = call("pasteHtml", revision, html="<p><b>새😀</b></p>")["outcome"]
    revision = outcome["revision"]
    call("setSelection", revision, selection=selection((0, 1), (0, 3)))
    formatted = call("copySelection", revision)["payload"]
    assert formatted["plainText"] == "새😀", formatted
    assert "font-weight:bold" in formatted["html"], formatted
    for operation, fields in [("pasteText", {"text": ""}), ("pasteHtml", {"html": "<p></p>"})]:
        empty = call(operation, revision, **fields)["outcome"]
        assert empty["revision"] == revision and not empty["documentChanged"], empty
        assert not empty["historyChanged"], empty
    rejected("staleRevision", lambda: call("cutSelection", revision - 1))
    rejected("unsupported", lambda: call("pasteText", revision, text="bad\ttext"))
    rejected("unsupported", lambda: call("pasteHtml", revision, html=(
        "<p>first</p><table><tr><td>cell</td></tr></table><p>last</p>"
    )))
    assert info()["revision"] == revision, "rejected paste/cut changed the revision"
    assert call("copySelection", revision)["payload"] == formatted

    composition = call("beginComposition", revision)["outcome"]
    for operation, fields in [
        ("copySelection", {}), ("cutSelection", {}),
        ("pasteText", {"text": "x"}), ("pasteHtml", {"html": "<p>x</p>"}),
    ]:
        rejected("invalidArgument", lambda op=operation, data=fields: call(op, revision, **data))
    cancelled = call("cancelComposition", revision,
                     compositionId=composition["compositionId"])["outcome"]
    assert call("copySelection", cancelled["revision"])["payload"] == formatted


def sync_contract(library):
    host = NativeHost(library)
    handle = host.open_bytes()
    try:
        host.request(handle, op="beginEdit", expectedRevision=0, selection=selection())

        def call(op, revision, **fields):
            return host.request(handle, op=op, expectedRevision=revision, **fields)

        exercise(call, lambda: host.request(handle, op="info"))
        for op in ("copySelection", "cutSelection", "pasteText", "pasteHtml"):
            fields = {"text": "x"} if op == "pasteText" else {"html": "<p>x</p>"} if op == "pasteHtml" else {}
            rejected("invalidArgument", lambda op=op, fields=fields: host.request(handle, op=op, **fields))
            rejected("invalidArgument", lambda op=op, fields=fields: host.request(
                handle, op=op, expectedRevision=0, unexpected=True, **fields))
    finally:
        host.call("close", handle)


def completed(worker, command):
    result = worker.wait(worker.submit(command))
    if not result["ok"]:
        raise NativeCallError(result)
    return result


def worker_contract(library):
    # Two credits but only one clipboard's byte reservation: receiving a copy
    # completion, rather than execution alone, must release that reservation.
    with NativeWorker(library, {"maxOutstanding": 2, "maxPendingBytes": 1024 * 1024}) as worker:
        initial = completed(worker, {"op": "new", "replace": None})["info"]
        document = initial["documentId"]

        def call(op, revision, **fields):
            return completed(worker, {"op": op, "target": {
                "documentId": document, "revision": revision,
            }, **fields})

        def info():
            return completed(worker, {"op": "info"})["info"]

        call("beginEdit", 0, selection=selection())
        exercise(call, info)
        revision = info()["revision"]
        target = {"documentId": document, "revision": revision}
        request = worker.submit({"op": "copySelection", "target": target})
        rejected("byteCapacity", lambda: worker.submit({"op": "cutSelection", "target": target}))
        copied = worker.wait(request)
        assert copied["ok"] and copied["kind"] == "clipboard", copied
        assert copied["payload"]["plainText"] == "새😀", copied
        cut = call("cutSelection", revision)
        assert cut["payload"] == copied["payload"] and cut["outcome"]["documentChanged"], cut
        for op in ("copySelection", "cutSelection", "pasteText", "pasteHtml"):
            fields = {"text": "x"} if op == "pasteText" else {"html": "<p>x</p>"} if op == "pasteHtml" else {}
            rejected("invalidArgument", lambda op=op, fields=fields: worker.submit({"op": op, **fields}))
            rejected("invalidArgument", lambda op=op, fields=fields: worker.submit({
                "op": op, "target": target, "unexpected": True, **fields,
            }))
    assert worker.is_finished()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("library", type=Path)
    args = parser.parse_args()
    sync_contract(args.library)
    worker_contract(args.library)
    print('{"ok":true,"contract":"native-clipboard","sync":true,"worker":true,"byteReservation":true}')
