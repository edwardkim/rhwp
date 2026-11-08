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
    print('{"ok":true,"contract":"native-worker","queueCredit":true,"saveRevision":true,"threadOwnership":true,"saveBufferQuotaCleanup":true}')


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    run(parser.parse_args().library)
