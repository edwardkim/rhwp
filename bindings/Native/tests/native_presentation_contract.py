#!/usr/bin/env python3
"""Actual C ABI snapshot handles and cross-thread CPU consumer lease contracts."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import ctypes
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_host import NativeCallError, NativeHost
from native_worker import NativeWorker


def accepted(worker, command):
    result = worker.wait(worker.submit(command))
    assert result["ok"], result
    return result


def rejected(operation, code="invalidArgument"):
    try:
        operation()
    except NativeCallError as error:
        assert error.code == code, error.response
        return error.response
    raise AssertionError("invalid presentation operation succeeded")


def setup(worker):
    info = accepted(worker, {"op": "new", "replace": None})["info"]
    stamp = {"documentId": info["documentId"], "revision": info["revision"],
             "fontGeneration": info["fontGeneration"], "viewGeneration": 1,
             "deviceGeneration": 1, "deviceScale": 1.0}
    worker.host.host_request(op="presentationConfigure", worker=worker.worker, stamp=stamp)
    return info, stamp


def exported(worker, info):
    return accepted(worker, {
        "op": "exportRenderSnapshot",
        "target": {"documentId": info["documentId"], "revision": info["revision"]},
        "fontGeneration": info["fontGeneration"], "page": 0, "profile": "screen",
    })


def stats(host):
    return host.presentation_request(op="stats")["stats"]


def lease_bytes(host, lease):
    function = host.library.rhwp_presentation_lease_data
    function.argtypes = [ctypes.c_uint64, ctypes.POINTER(ctypes.c_size_t)]
    function.restype = ctypes.c_void_p
    length = ctypes.c_size_t(999)
    pointer = function(lease, ctypes.byref(length))
    if not pointer:
        assert length.value == 0, "invalid lease retained an old output length"
        return None
    return ctypes.string_at(pointer, length.value)


def snapshot_quota_and_thread_contract(library):
    with NativeWorker(library, {"maxPendingBytes": 64 * 1024}) as worker:
        info, stamp = setup(worker)
        host = worker.host
        caps = host.call("capabilities")
        presentation = caps["presentation"]
        assert presentation["workerRenderSnapshots"]
        assert presentation["crossThreadHandles"]
        assert presentation["fullStampChecks"]
        assert presentation["gpuFrames"] is False
        assert presentation["cpuFrames"] is caps["nativeSkia"]
        assert presentation["cpuBatchFrames"] is caps["nativeSkia"]
        initial = stats(host)
        handles = []
        try:
            for _ in range(presentation["maxSnapshots"]):
                result = exported(worker, info)
                handle = result["snapshot"]
                assert handle not in handles
                handles.append(handle)
                assert result["kind"] == "renderSnapshot", result
                assert result["info"]["usage"]["estimatedBytes"] <= 64 * 1024, result
            before = stats(host)
            failed = worker.wait(worker.submit({
                "op": "exportRenderSnapshot",
                "target": {"documentId": info["documentId"], "revision": info["revision"]},
                "fontGeneration": info["fontGeneration"], "page": 0, "profile": "screen",
            }))
            assert not failed["ok"] and failed["code"] == "limitExceeded", failed
            assert "snapshot" not in failed, failed
            assert stats(host) == before
            with ThreadPoolExecutor(max_workers=1) as consumer:
                result = consumer.submit(host.presentation_request, op="snapshotInfo",
                                         snapshot=handles[0]).result(timeout=30)
                assert result["info"]["current"], result
                checked = consumer.submit(host.presentation_request, op="snapshotCheck",
                    snapshot=handles[0], stamp=stamp).result(timeout=30)
                assert checked["current"]
                wrong = dict(stamp, viewGeneration=stamp["viewGeneration"] + 1)
                assert not consumer.submit(host.presentation_request, op="snapshotCheck",
                    snapshot=handles[0], stamp=wrong).result(timeout=30)["current"]
                # The new cross-thread exception does not move old worker handles.
                consumer.submit(rejected, lambda: host.host_request(
                    op="presentationConfigure", worker=worker.worker, stamp=stamp)).result(timeout=30)
            changed_view = dict(stamp, viewGeneration=stamp["viewGeneration"] + 1, deviceScale=1.5)
            host.host_request(op="presentationConfigure", worker=worker.worker, stamp=changed_view)
            assert not host.presentation_request(op="snapshotCheck", snapshot=handles[0], stamp=stamp)["current"]
            assert host.presentation_request(op="snapshotCheck", snapshot=handles[0], stamp=changed_view)["current"]
            assert host.presentation_request(op="snapshotInfo", snapshot=handles[0])["info"]["current"]
            released = handles.pop()
            host.presentation_request(op="snapshotRelease", snapshot=released)
            before = stats(host)
            rejected(lambda: host.presentation_request(op="snapshotRelease", snapshot=released))
            rejected(lambda: host.presentation_request(op="snapshotInfo", snapshot=0))
            rejected(lambda: host.presentation_request(op="stats", unexpected=True))
            assert stats(host) == before
        finally:
            for handle in handles:
                host.presentation_request(op="snapshotRelease", snapshot=handle)
        after = stats(host)
        assert after["snapshotHandles"] == initial["snapshotHandles"], after
        assert after["snapshotEstimatedBytes"] == initial["snapshotEstimatedBytes"], after


def cpu_lifetime_contract(library):
    with NativeWorker(library) as worker:
        info, stamp = setup(worker)
        host = worker.host
        snapshot = exported(worker, info)["snapshot"]
        frame = lease = None
        try:
            with ThreadPoolExecutor(max_workers=1) as consumer:
                rendered = consumer.submit(host.presentation_request, op="renderCpu",
                    snapshot=snapshot, stamp=stamp,
                    region={"x": 0, "y": 0, "width": 1, "height": 1}, scale=1).result(timeout=30)
                frame = rendered["frame"]
                descriptor = rendered["info"]["descriptor"]
                assert descriptor == {"backend": "cpu", "width": 1, "height": 1, "stride": 4,
                    "byteLength": 4, "pixelFormat": "rgba8Premultiplied", "colorSpace": "srgb",
                    "alpha": "premultiplied", "origin": "topLeft", "originX": 0,
                    "originY": 0, "rasterScale": 1.0, "deviceScale": 1.0}, descriptor
                acquired = consumer.submit(host.presentation_request, op="frameAcquire",
                                           frame=frame, stamp=stamp).result(timeout=30)
                lease = acquired["lease"]
                expected = consumer.submit(lease_bytes, host, lease).result(timeout=30)
                assert len(expected) == 4
                before = stats(host)
                wrong = dict(stamp, deviceGeneration=2)
                rejected(lambda: host.presentation_request(op="frameAcquire", frame=frame, stamp=wrong))
                assert stats(host) == before
                # Close the actual source while retaining independent consumers.
                worker.close()
                rejected(lambda: host.presentation_request(op="frameAcquire", frame=frame, stamp=stamp))
                assert not host.presentation_request(op="frameInfo", frame=frame)["info"]["current"]
                assert not host.presentation_request(op="snapshotInfo", snapshot=snapshot)["info"]["current"]
                assert not host.presentation_request(op="snapshotCheck", snapshot=snapshot, stamp=stamp)["current"]
                host.presentation_request(op="frameRelease", frame=frame)
                frame = None
                current = stats(host)
                assert current["frameHandles"] == 0 and current["retainedFrames"] == 1, current
                assert current["frameBytes"] == 4 and current["leases"] == 1, current
                assert consumer.submit(lease_bytes, host, lease).result(timeout=30) == expected
                released = lease
                consumer.submit(host.presentation_request, op="leaseRelease", lease=lease).result(timeout=30)
                lease = None
                assert lease_bytes(host, released) is None
                rejected(lambda: host.presentation_request(op="leaseRelease", lease=released))
                after = stats(host)
                assert after["frameBytes"] == 0 and after["retainedFrames"] == 0, after
        finally:
            if lease is not None:
                host.presentation_request(op="leaseRelease", lease=lease)
            if frame is not None:
                host.presentation_request(op="frameRelease", frame=frame)
            host.presentation_request(op="snapshotRelease", snapshot=snapshot)


def deferred_snapshot_cleanup_contract(library):
    with NativeWorker(library) as worker:
        info, _ = setup(worker)
        host = worker.host
        baseline = stats(host)
        worker.submit({"op": "exportRenderSnapshot",
            "target": {"documentId": info["documentId"], "revision": info["revision"]},
            "fontGeneration": info["fontGeneration"], "page": 0, "profile": "screen"})
        # wait(info) transfers the earlier snapshot into the wrapper's deferred
        # queue. close must release handles that no caller has received.
        accepted(worker, {"op": "info"})
        assert stats(host)["snapshotHandles"] == baseline["snapshotHandles"] + 1
        worker.close()
        after = stats(host)
        assert after["snapshotHandles"] == baseline["snapshotHandles"], after
        assert after["snapshotEstimatedBytes"] == baseline["snapshotEstimatedBytes"], after


def owner_thread_teardown_contract(library):
    host = NativeHost(library)
    baseline = stats(host)

    def leave_owner_thread():
        worker = NativeWorker(library)
        info, _ = setup(worker)
        snapshot = exported(worker, info)["snapshot"]
        # Deliberately leave the owner thread without explicit close. Keep the
        # CDLL alive in the process while native shutdown runs asynchronously.
        return worker.host, snapshot

    with ThreadPoolExecutor(max_workers=1) as owner:
        retained_host, snapshot = owner.submit(leave_owner_thread).result(timeout=30)
    try:
        after = stats(host)
        assert after["sources"] == baseline["sources"], after
        assert after["snapshotHandles"] == baseline["snapshotHandles"] + 1, after
        assert not retained_host.presentation_request(
            op="snapshotInfo", snapshot=snapshot)["info"]["current"]
    finally:
        host.presentation_request(op="snapshotRelease", snapshot=snapshot)
    assert stats(host) == baseline


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    args = parser.parse_args()
    snapshot_quota_and_thread_contract(args.library)
    with NativeWorker(args.library) as worker:
        native_skia = worker.host.call("capabilities")["nativeSkia"]
    if native_skia:
        cpu_lifetime_contract(args.library)
    else:
        with NativeWorker(args.library) as worker:
            _, stamp = setup(worker)
            rejected(lambda: worker.host.presentation_request(op="renderCpu", snapshot=0,
                stamp=stamp, region={"x": 0, "y": 0, "width": 1, "height": 1}, scale=1),
                code="featureDisabled")
            rejected(lambda: worker.host.presentation_request(op="renderCpuBatch", stamp=stamp,
                viewport={"width":1,"height":1}, placements=[], clearColor=0),
                code="featureDisabled")
    deferred_snapshot_cleanup_contract(args.library)
    owner_thread_teardown_contract(args.library)
    print("native presentation contract PASS (CPU/frame transport only; GPU capability false)")


if __name__ == "__main__":
    main()
