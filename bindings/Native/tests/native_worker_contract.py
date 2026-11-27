#!/usr/bin/env python3
"""Actual-library worker queue, ownership and delayed-save contracts."""

import argparse
from contextlib import ExitStack
from pathlib import Path
import sys
import threading

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_host import NativeCallError
from native_worker import NativeWorker


def accepted(worker, command):
    result = worker.wait(worker.submit(command))
    assert result["ok"], result
    return result


def position(offset=0):
    return {"section": 0, "paragraph": 0, "scalarOffset": offset, "cellPath": []}


def cell_edit_contract(library):
    fixture = Path(__file__).resolve().parents[3] / "samples/hwpx/basic-table-01.hwpx"
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(fixture.read_bytes()))
        assert opened["ok"], opened
        info = opened["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}

        def snapshot():
            return accepted(worker, {"op": "accessibility", "target": target, "request": {}})[
                "snapshot"
            ]

        before = snapshot()
        cell, = (node for node in before["nodes"]
                 if node["role"] == "tableCellParagraph" and node["text"] == "10")
        assert len(cell["start"]["cellPath"]) == 1, cell
        selection = {"anchor": cell["start"], "focus": cell["end"]}
        rect, = accepted(worker, {"op": "selectionRects", "target": target,
                                  "selection": selection})["rects"]
        bounds = rect["bounds"]
        assert bounds["width"] > 0 and bounds["height"] > 0, rect
        hit = accepted(worker, {
            "op": "hitTest", "target": target, "page": rect["page"],
            "x": bounds["x"] + bounds["width"] / 2,
            "y": bounds["y"] + bounds["height"] / 2,
        })["hit"]
        hit_position = hit["position"]
        assert {**hit_position, "scalarOffset": 0} == cell["start"], hit
        assert 0 <= hit_position["scalarOffset"] <= cell["end"]["scalarOffset"], hit
        caret = accepted(worker, {"op": "caretRect", "target": target,
                                  "position": hit_position})["caret"]
        assert caret["page"] == rect["page"] and caret["height"] > 0, caret

        def select_cell(start, end):
            return {
                "anchor": {**cell["start"], "scalarOffset": start},
                "focus": {**cell["start"], "scalarOffset": end},
            }

        def edit(operation, **arguments):
            outcome = accepted(worker, {"op": operation, "target": target, **arguments})["outcome"]
            target["revision"] = outcome["revision"]
            return outcome

        def contents(receipt):
            return [(node["role"], node["start"], node["end"], node["text"])
                    for node in receipt["nodes"]]

        def assert_cell_text(text):
            expected = contents(before)
            index = before["nodes"].index(cell)
            expected[index] = (cell["role"], cell["start"],
                               {**cell["end"], "scalarOffset": len(text)}, text)
            assert contents(snapshot()) == expected

        beginning = select_cell(1, 1)
        begun = accepted(worker, {"op": "beginEdit", "target": target, "selection": beginning})
        assert begun["info"]["selection"] == beginning, begun
        original_target = target.copy()
        inserted = edit("replace", text="가")
        assert inserted["documentChanged"] and inserted["selection"] == select_cell(2, 2), inserted
        assert_cell_text("1가0")

        stale = worker.wait(worker.submit({"op": "replace", "target": original_target, "text": "X"}))
        assert stale["ok"] is False and stale["code"] == "staleRevision", stale
        neighbor = next(node for node in before["nodes"]
                        if node["role"] == "tableCellParagraph" and node["text"] == "11")
        selected_info = accepted(worker, {"op": "info"})["info"]
        selected_snapshot = snapshot()
        mixed = {"anchor": cell["start"], "focus": neighbor["end"]}
        rejected = worker.wait(worker.submit({"op": "setSelection", "target": target,
                                              "selection": mixed}))
        assert rejected["ok"] is False and rejected["code"] == "unsupported", rejected
        assert accepted(worker, {"op": "info"})["info"] == selected_info
        assert snapshot() == selected_snapshot

        edit("setSelection", selection=select_cell(1, 2))
        replaced = edit("replace", text="한글")
        assert replaced["documentChanged"] and replaced["selection"] == select_cell(3, 3), replaced
        assert_cell_text("1한글0")
        delete_range = select_cell(1, 3)
        edit("setSelection", selection=delete_range)
        deleted = edit("replace", text="")
        assert deleted["documentChanged"] and deleted["selection"] == beginning, deleted
        assert_cell_text("10")

        for text, expected_selection in (("1한글0", delete_range), ("1가0", select_cell(1, 2)),
                                         ("10", beginning)):
            undone = edit("undo")
            assert undone["documentChanged"] and undone["selection"] == expected_selection, undone
            assert_cell_text(text)
        for text, expected_selection in (("1가0", select_cell(2, 2)), ("1한글0", select_cell(3, 3)),
                                         ("10", beginning)):
            redone = edit("redo")
            assert redone["documentChanged"] and redone["selection"] == expected_selection, redone
            assert_cell_text(text)

        edit("undo")
        assert_cell_text("1한글0")
        expected = contents(snapshot())

        def verify_save(format, signature, caret_policy):
            saved = accepted(worker, {"op": "prepareSave", "target": target, "format": format})
            data = worker.copy_buffer(saved)
            assert data.startswith(signature) and saved["report"]["count"] == 0, saved
            assert saved["caretPolicy"] == caret_policy, saved
            accepted(worker, {"op": "acknowledgeSave", "documentId": target["documentId"],
                              "token": saved["token"]})
            with NativeWorker(library) as verifier:
                for pass_number in range(2):
                    reopened = verifier.wait(verifier.open_bytes(data))
                    assert reopened["ok"] and not reopened["info"]["isDirty"], reopened
                    reopened_target = {key: reopened["info"][key] for key in target}
                    receipt = accepted(verifier, {"op": "accessibility", "target": reopened_target,
                                                  "request": {}})["snapshot"]
                    assert contents(receipt) == expected
                    if pass_number == 0:
                        resaved = accepted(verifier, {"op": "prepareSave", "target": reopened_target,
                                                       "format": format})
                        data = verifier.copy_buffer(resaved)
                        assert resaved["report"]["count"] == 0, resaved
                        accepted(verifier, {"op": "closeDocument", "target": reopened_target})

        # HWP can save the edited cell content after the cursor moves to body text.
        # HWP active-cell cursor persistence has its own format contract.
        before_save = accepted(worker, {"op": "info"})["info"]
        hwp = worker.wait(worker.submit({"op": "prepareSave", "target": target, "format": "hwp"}))
        assert hwp["ok"] is False and hwp["code"] == "unsupported", hwp
        assert accepted(worker, {"op": "info"})["info"] == before_save
        verify_save("hwpx", b"PK", "hwpxSourceSettings")

        body = before["nodes"][-1]
        assert body["role"] == "bodyParagraph" and body["text"] == "", body
        edit("setSelection", selection={"anchor": body["start"], "focus": body["end"]})
        verify_save("hwp", bytes.fromhex("d0cf11e0a1b11ae1"), "currentFocus")


def buffer_quota_contract(library):
    # Exercise the handle-count limit using only 256 retained pixel bytes. Each
    # completion is received before another render, so coalescing cannot hide it.
    with NativeWorker(library, {"maxOutstanding": 1}) as worker:
        with ExitStack() as buffers:
            info = accepted(worker, {"op": "new", "replace": None})["info"]
            target = {"documentId": info["documentId"], "revision": info["revision"]}
            selection = {"anchor": position(), "focus": position()}
            accepted(worker, {"op": "beginEdit", "target": target, "selection": selection})
            outcome = accepted(worker, {"op": "replace", "target": target,
                                        "text": "버퍼 한도"})["outcome"]
            target["revision"] = outcome["revision"]
            retained = set()
            for _ in range(64):
                frame = accepted(worker, {
                    "op": "renderRegion", "target": target, "fontGeneration": 0,
                    "page": 0, "region": {"x": 0, "y": 0, "width": 1, "height": 1},
                    "scale": 1,
                })
                buffers.callback(worker.host.call, "buffer_release", frame["buffer"])
                assert frame["kind"] == "frame", frame
                assert (frame["width"], frame["height"], frame["byteLength"]) == (1, 1, 4), frame
                assert frame["buffer"] not in retained, "retained buffer handles were reused"
                retained.add(frame["buffer"])

            request = worker.submit({"op": "prepareSave", "target": target, "format": "hwpx"})
            failed = worker.wait(request)
            if "buffer" in failed:
                buffers.callback(worker.host.call, "buffer_release", failed["buffer"])
            assert failed["requestId"] == request, "quota failure lost the accepted save request ID"
            assert not failed["ok"] and failed.get("code") == "limitExceeded", failed
            assert "buffer" not in failed and "token" not in failed, failed
            cleanup = failed.get("saveCleanup")
            assert isinstance(cleanup, dict) and cleanup.get("ok") is True, (
                "retaining save bytes failed without an accepted token cleanup", failed
            )
            cleanup_id = cleanup.get("requestId")
            assert type(cleanup_id) is int and cleanup_id > 0 and cleanup_id != request, cleanup
            cleaned = worker.wait(cleanup_id)
            assert cleaned["ok"] and cleaned.get("kind") == "saveStatus", (
                "automatic save-token cleanup did not complete successfully", cleaned
            )
            assert cleaned["requestId"] == cleanup_id, cleaned
            assert cleaned["documentId"] == target["documentId"], cleaned
            assert cleaned["isDirty"] and cleaned["status"] == {
                "revision": target["revision"], "savedRevision": None, "dirty": True,
            }, cleaned
            dirty = accepted(worker, {"op": "info"})["info"]
            assert dirty["isDirty"] and dirty["revision"] == target["revision"], dirty

            # Release after receiving cleanup, even if any assertion above fails.
            # ExitStack also runs every release if another release raises.
            buffers.close()
            prepared = accepted(worker, {"op": "prepareSave", "target": target,
                                         "format": "hwpx"})
            data = worker.copy_buffer(prepared)
            assert data.startswith(b"PK") and prepared["report"]["count"] == 0, prepared
            assert prepared["revision"] == target["revision"], prepared
            dirty = accepted(worker, {"op": "info"})["info"]
            assert dirty["isDirty"], "preparing/copying bytes falsely marked the document clean"
            outcome = accepted(worker, {"op": "replace", "target": target,
                                        "text": " 이후 변경"})["outcome"]
            target["revision"] = outcome["revision"]
            saved = accepted(worker, {"op": "acknowledgeSave",
                                      "documentId": target["documentId"],
                                      "token": prepared["token"]})
            assert saved["isDirty"] and saved["status"] == {
                "revision": target["revision"], "savedRevision": prepared["revision"],
                "dirty": True,
            }, saved

    assert worker.is_finished()
    try:
        worker.host.host_request(op="workerStatus", worker=worker.worker)
    except NativeCallError as error:
        assert error.code == "invalidArgument", error.response
    else:
        raise AssertionError("closed worker handle remained registered after release")


def tiled_cancellation_contract(library):
    with NativeWorker(library, {"maxOutstanding": 1}) as worker:
        info = accepted(worker, {"op": "new", "replace": None})["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}
        command = {"op": "renderTiledRegion", "target": target, "fontGeneration": 0,
                   "page": 0, "region": {"x": 0, "y": 0, "width": 1024, "height": 1024},
                   "scale": 1}
        request = worker.submit(command)
        outcome = worker.cancel(request)
        assert outcome in ("cancelled", "tooLate"), outcome
        completion = worker.wait(request)
        assert completion["ok"], completion
        if outcome == "cancelled":
            assert completion["kind"] == "cancelled" and "buffer" not in completion, completion
        else:
            assert completion["kind"] == "frame", completion
            worker.copy_buffer(completion)
        assert worker.cancel(request) == "unknown"
        assert accepted(worker, {"op": "info"})["info"]["revision"] == target["revision"]


def viewport_budget_contract(library):
    for operation in ("renderRegion", "renderTiledRegion"):
        with NativeWorker(library, {"maxOutstanding": 4, "maxPendingBytes": 80}) as worker:
            info = accepted(worker, {"op": "new", "replace": None})["info"]
            target = {"documentId": info["documentId"], "revision": info["revision"]}
            command = {"op": operation, "target": target, "fontGeneration": 0, "page": 0,
                       "region": {"x": -0.3, "y": 0.2, "width": 3.4, "height": 2.9},
                       "scale": 1.25}
            request = worker.submit(command)
            try:
                worker.submit(command)
            except NativeCallError as error:
                assert error.code == "byteCapacity", error.response
            else:
                raise AssertionError("unread frame did not reserve its 80-byte output")
            frame = worker.wait(request)
            assert frame["ok"] and frame["kind"] == "frame", frame
            assert (frame["width"], frame["height"], frame["byteLength"]) == (5, 4, 80), frame
            assert len(worker.copy_buffer(frame)) == 80
            assert len(worker.copy_buffer(accepted(worker, command))) == 80


def run(library):
    with NativeWorker(library, {"maxOutstanding": 1}) as worker:
        first = worker.submit({"op": "new", "replace": None})
        try:
            worker.submit({"op": "info"})
        except NativeCallError as error:
            assert error.code == "queueFull"
        else:
            raise AssertionError("unread completion did not reserve queue credit")
        info = worker.wait(first)["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}
        original = target.copy()
        selection = {"anchor": position(), "focus": position()}
        accepted(worker, {"op": "beginEdit", "target": target, "selection": selection})
        outcome = accepted(worker, {"op": "replace", "target": target, "text": "가"})["outcome"]
        target["revision"] = outcome["revision"]
        stale = worker.wait(worker.submit({"op": "replace", "target": original, "text": "bad"}))
        assert not stale["ok"] and stale["code"] == "staleRevision"
        size = accepted(worker, {"op": "pageSize", "target": target, "page": 0})
        assert size["size"]["width"] > 0
        caret = accepted(worker, {"op": "caretRect", "target": target, "position": position(1)})
        assert caret["caret"]["height"] > 0

        foreign = []
        def other_thread():
            try:
                worker.host.host_request(op="workerPoll", worker=worker.worker)
            except NativeCallError as error:
                foreign.append(error.code)
        thread = threading.Thread(target=other_thread)
        thread.start()
        thread.join()
        assert foreign == ["invalidArgument"]

        prepared = accepted(worker, {"op": "prepareSave", "target": target, "format": "hwpx"})
        data = worker.copy_buffer(prepared)
        assert data.startswith(b"PK") and prepared["report"]["count"] == 0
        outcome = accepted(worker, {"op": "replace", "target": target, "text": "나"})["outcome"]
        target["revision"] = outcome["revision"]
        saved = accepted(worker, {"op": "acknowledgeSave", "documentId": target["documentId"],
                                  "token": prepared["token"]})
        assert saved["isDirty"]
        request = worker.open_bytes(data, replace=target)
        reopened = worker.wait(request)
        assert reopened["ok"] and reopened["info"]["documentId"] != target["documentId"]
        assert reopened["info"]["revision"] == 0 and not reopened["info"]["isDirty"]
        rejected = worker.wait(worker.submit({"op": "pageSize", "target": target, "page": 0}))
        assert not rejected["ok"] and rejected["code"] == "invalidArgument"

    assert worker.is_finished()
    with NativeWorker(library) as viewer:
        info = accepted(viewer, {"op": "new", "replace": None})["info"]
        target = {"documentId": info["documentId"], "revision": 0}
        frame = accepted(viewer, {"op": "renderRegion", "target": target, "fontGeneration": 0,
                                  "page": 0, "region": {"x": 0, "y": 0, "width": 8, "height": 8},
                                  "scale": 1})
        accepted(viewer, {"op": "setRasterCacheLimit", "target": target,
                          "maxBytes": 8 * 1024 * 1024})
        tiled = {"op": "renderTiledRegion", "target": target, "fontGeneration": 0,
                 "page": 0, "region": {"x": -0.1, "y": 1.25, "width": 1.2, "height": 2.5},
                 "scale": 8}
        for _ in range(2):
            viewport = accepted(viewer, tiled)
            assert (viewport["originX"], viewport["originY"]) == (-1, 10), viewport
            assert (viewport["width"], viewport["height"], viewport["stride"]) == (10, 20, 40), viewport
            assert len(viewer.copy_buffer(viewport)) == 800
        stats = accepted(viewer, {"op": "rasterCacheStats", "target": target})["stats"]
        assert stats["tileHits"] > 0 and stats["retainedBytes"] <= 8 * 1024 * 1024, stats
        stale = viewer.wait(viewer.submit(dict(tiled, fontGeneration=1)))
        assert not stale["ok"] and stale["code"] == "invalidArgument", stale
        stale = viewer.wait(viewer.submit(dict(tiled, target=dict(target, revision=1))))
        assert not stale["ok"] and stale["code"] == "staleRevision", stale
        viewer.close()
        pixels = viewer.copy_buffer(frame)
        assert len(pixels) == frame["height"] * frame["stride"]
        viewer.wait_closed()
    buffer_quota_contract(library)
    viewport_budget_contract(library)
    tiled_cancellation_contract(library)
    cell_edit_contract(library)
    print('{"ok":true,"contract":"native-worker","queueCredit":true,"saveRevision":true,'
          '"threadOwnership":true,"saveBufferQuotaCleanup":true,"cellEdit":true}')


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    run(parser.parse_args().library)
