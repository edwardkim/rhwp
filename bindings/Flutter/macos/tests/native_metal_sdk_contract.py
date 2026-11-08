#!/usr/bin/env python3
"""Actual macOS Native/Metal producer, with separate portable receipt checks."""

import argparse
import base64
import ctypes
import hashlib
import json
from pathlib import Path
import sys
import time


def expected_bgra(color):
    alpha = color >> 24 & 255
    return tuple(((color >> shift & 255) * alpha + 127) // 255 for shift in (0, 8, 16)) + (alpha,)


def validate_response(value, stamp, color, *, stale=False):
    if value.get("ok") is not True or value.get("consumerGpuCompletionVerified") is not False:
        raise ValueError("probe failed or fabricated consumer completion")
    if value.get("hostVerificationReadbacks") != 1 or value.get("width") != 32 or value.get("height") != 24:
        raise ValueError("unbounded or missing actual Metal target readback")
    data = base64.b64decode(value.get("bgraBase64", ""), validate=True)
    if len(data) != 32 * 24 * 4:
        raise ValueError("Metal readback byte length mismatch")
    native = value.get("native", {})
    if stale:
        if native.get("ok") is not False or native.get("code") != "staleFrame" or native.get("unpublishedTargetSafeToDiscard") is not True:
            raise ValueError("stale source was not explicitly rejected before target publication")
        if data != bytes([0x5a]) * len(data):
            raise ValueError("stale rejection changed the unpublished target")
        return {"staleRejected": True, "targetPreserved": True}
    frame = native.get("frame", {})
    if native.get("ok") is not True or frame.get("backend") != "metalGanesh" or frame.get("stamp") != stamp:
        raise ValueError("Native Metal frame/stamp mismatch")
    if frame.get("producerGpuComplete") is not True or frame.get("consumerGpuComplete") is not False:
        raise ValueError("producer/consumer completion evidence is invalid")
    if frame.get("wholeFrameReadbacks") != 0:
        raise ValueError("normal Native render introduced a whole-frame readback")
    expected = expected_bgra(color)
    maximum = max(abs(channel - expected[index % 4]) for index, channel in enumerate(data))
    if maximum > 1:
        raise ValueError("Metal target does not contain the independent premultiplied clear color")
    return {"expectedBgra": expected, "maxChannelDifference": maximum}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--bridge", type=Path, required=True)
    parser.add_argument("--source-head", required=True)
    parser.add_argument("--native-source-head")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("actual macOS/Metal is required; portable tests are not target execution")
    if args.output.exists():
        parser.error("output receipt already exists")
    repository = Path(__file__).resolve().parents[4]
    sys.path.insert(0, str(repository / "bindings/Native/examples"))
    from native_worker import NativeWorker

    native = args.native.resolve(strict=True)
    bridge_path = args.bridge.resolve(strict=True)
    bridge = ctypes.CDLL(str(bridge_path))
    call = bridge.rhwp_probe_metal_sdk
    call.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
    call.restype = ctypes.c_void_p
    free = bridge.rhwp_probe_metal_free
    free.argtypes = [ctypes.c_void_p]
    free.restype = None
    report = {"platform": "macos", "source_head": args.source_head,
              "nativeSourceHead": args.native_source_head or args.source_head,
              "nativeSha256": hashlib.sha256(native.read_bytes()).hexdigest(),
              "bridgeSha256": hashlib.sha256(bridge_path.read_bytes()).hexdigest(),
              "consumerGpuCompletionVerified": False, "continuousPresentationSupported": False,
              "flutterConsumerSamplingVerified": False, "results": [], "started": time.time()}
    try:
        with NativeWorker(native) as worker:
            def run(command):
                result = worker.wait(worker.submit(command))
                if result.get("ok") is not True:
                    raise RuntimeError("Native worker command failed: " + str(result))
                return result

            info = run({"op": "new", "replace": None})["info"]
            target = {key: info[key] for key in ("documentId", "revision")}
            stamp = dict(target, fontGeneration=info["fontGeneration"], viewGeneration=1,
                         deviceGeneration=1, deviceScale=1.0)
            worker.host.host_request(op="presentationConfigure", worker=worker.worker, stamp=stamp)
            snapshot = run({"op": "exportRenderSnapshot", "target": target,
                            "fontGeneration": info["fontGeneration"], "page": 0,
                            "profile": "screen"})["snapshot"]
            try:
                for name, color, stale in [("opaque", 0xff102030, False),
                                            ("premultiplied-alpha", 0x80402010, False),
                                            ("stale-preserves-target", 0xff102030, True)]:
                    if stale:
                        point = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
                        run({"op": "beginEdit", "target": target,
                             "selection": {"anchor": point, "focus": point}})
                        run({"op": "pasteText", "target": target, "text": "changed"})
                    request = {"stamp": stamp, "target": {"deviceGeneration": 1, "width": 32, "height": 24},
                               # Keep the authoritative blank page entirely outside the
                               # target. Expected opaque/premultiplied pixels are then
                               # independent clear-color controls, not a copied oracle.
                               "placements": [{"snapshot": snapshot, "scale": .25,
                                               "originX": 4096, "originY": 4096,
                                               "backgroundColor": 0xffffffff}], "clearColor": color}
                    encoded = json.dumps(request).encode()
                    data = (ctypes.c_uint8 * len(encoded)).from_buffer_copy(encoded)
                    pointer = call(str(native).encode(), data, len(encoded))
                    if not pointer:
                        raise RuntimeError("Metal bridge returned null")
                    try:
                        raw = ctypes.cast(pointer, ctypes.POINTER(ctypes.c_uint8))
                        length = 0
                        while length <= 64 * 1024 and raw[length]:
                            length += 1
                        if length > 64 * 1024:
                            raise ValueError("Metal bridge reply exceeded its byte bound")
                        response = json.loads(ctypes.string_at(pointer, length))
                    finally:
                        free(pointer)
                    checks = validate_response(response, stamp, color, stale=stale)
                    report["results"].append({"name": name, "response": response, "checks": checks})
            finally:
                worker.host.presentation_request(op="snapshotRelease", snapshot=snapshot)
        stats = worker.host.presentation_request(op="stats")["stats"]
        if any(stats[key] != 0 for key in ("sources", "snapshotHandles", "retainedSnapshots",
                "snapshotEstimatedBytes", "frameHandles", "retainedFrames", "frameBytes", "leases", "inFlightReplays")):
            raise ValueError("Native source/snapshot/frame/lease resources did not return to zero")
        report.update(status="passed", stats=stats, producerReadbackVerified=True)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        raise
    finally:
        report["finished"] = time.time()
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        args.output.chmod(0o600)
    print(args.output)


if __name__ == "__main__":
    main()
