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


def nested_cell_contract(library, verify_save=False):
    fixture = (Path(__file__).resolve().parents[3]
               / "samples/issue5599_oracle/3191107_leave_request_form.hwpx")
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(fixture.read_bytes()))
        assert opened["ok"], opened
        target = {key: opened["info"][key] for key in ("documentId", "revision")}

        def call(op, **fields):
            return completed(worker, {"op": op, "target": target, **fields})

        def edit(op, **fields):
            outcome = call(op, **fields)["outcome"]
            target["revision"] = outcome["revision"]
            return outcome

        def nodes(host, current):
            result = completed(host, {
                "op": "accessibility", "target": current, "request": {"maxNodes": 128},
            })["snapshot"]
            assert not result["truncated"] and all(node["textComplete"] for node in result["nodes"])
            # Accessibility node keys encode revision. Compare addresses and
            # content to verify undo and reopened documents across revisions.
            return [{field: value for field, value in node.items() if field != "key"}
                    for node in result["nodes"]]

        original = nodes(worker, target)
        name_label, = (i for i, node in enumerate(original)
                       if node["text"].replace(" ", "") == "성명")
        cell = original[name_label + 1]
        start = cell["start"]
        assert cell["role"] == "tableCellParagraph" and len(start["cellPath"]) == 2
        assert not cell["text"].strip(), cell
        call("beginEdit", selection={"anchor": start, "focus": cell["end"]})
        pasted = edit("pasteText", text="가😀")
        end = pasted["selection"]["focus"]
        assert end == {**start, "scalarOffset": 2}, pasted
        selected = {"anchor": end, "focus": start}
        edit("setSelection", selection=selected)
        copied = call("copySelection")
        assert copied["payload"] == {"plainText": "가😀", "html": None}, copied
        assert copied["outcome"] is None

        before = nodes(worker, target)
        revision = target["revision"]
        rejected("unsupported", lambda: call("pasteText", text="한\r\n글"))
        rejected("unsupported", lambda: call("pasteHtml", html="<p>한글</p>"))
        assert nodes(worker, target) == before
        info = completed(worker, {"op": "info"})["info"]
        assert info["revision"] == revision and info["selection"] == selected

        cut = call("cutSelection")
        assert cut["payload"] == copied["payload"], cut
        target["revision"] = cut["outcome"]["revision"]
        assert nodes(worker, target)[name_label + 1]["text"] == ""
        undone = edit("undo")
        assert undone["selection"] == selected and nodes(worker, target) == before, undone
        assert worker.host.call("capabilities").get("tableCellTextClipboard") is True

        if verify_save:
            def contents(host, current):
                return [(node["role"], node["start"], node["end"], node["text"])
                        for node in nodes(host, current)]

            expected = [(node["role"], node["start"], node["end"], node["text"])
                        for node in original]
            expected[name_label + 1] = ("tableCellParagraph", start, end, "가😀")
            assert contents(worker, target) == expected

            for file_format, signature, policy in (
                ("hwp", bytes.fromhex("d0cf11e0a1b11ae1"), "storedCore"),
                ("hwpx", b"PK", "hwpxSourceSettings"),
            ):
                def save(host, current):
                    prepared = completed(host, {
                        "op": "prepareSave", "target": current, "format": file_format,
                    })
                    data = host.copy_buffer(prepared)
                    assert data.startswith(signature) and prepared["report"]["count"] == 0, prepared
                    assert prepared["caretPolicy"] == policy and prepared["caretSaved"] is False
                    saved = completed(host, {
                        "op": "acknowledgeSave", "documentId": current["documentId"],
                        "token": prepared["token"],
                    })
                    assert saved["isDirty"] is False, saved
                    assert completed(host, {"op": "info"})["info"]["selection"] == selected
                    return data

                data = save(worker, target)
                with NativeWorker(library) as verifier:
                    for cycle in range(2):
                        reopened = verifier.wait(verifier.open_bytes(data))
                        assert reopened["ok"] and reopened["info"]["pageCount"] == 1, reopened
                        current = {key: reopened["info"][key] for key in target}
                        # Compare every body and nested-cell path and its text. A
                        # persisted edit must preserve adjoining and child cells.
                        assert contents(verifier, current) == expected, (file_format, cycle)
                        if cycle == 0:
                            completed(verifier, {
                                "op": "beginEdit", "target": current, "selection": selected,
                            })
                            data = save(verifier, current)
                            completed(verifier, {"op": "closeDocument", "target": current})


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("library", type=Path)
    parser.add_argument("--verify-save", action="store_true",
                        help="check two HWP and HWPX save and reopen cycles after nested cell paste")
    args = parser.parse_args()
    sync_contract(args.library)
    worker_contract(args.library)
    nested_cell_contract(args.library, verify_save=args.verify_save)
    print('{"ok":true,"contract":"native-clipboard","sync":true,"worker":true,"byteReservation":true,"nestedCellText":true}')
