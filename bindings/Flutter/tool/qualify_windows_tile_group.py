#!/usr/bin/env python3
"""Observe two real Windows Flutter textures through edit, resize and recovery.

A receipt covers the actual OS DPR reported by both Win32 and FlutterView. A
second invocation at another OS scale can bind distinct-DPR evidence; this tool
never fabricates a display change or treats descriptor opening as a GPU fence.
"""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import math
import os
from pathlib import Path
import re
import subprocess
import time
import uuid

from qualify_desktop_texture import (MAX_BUILD_RECEIPT_BYTES, WindowsCompositor,
    compare_pixels, digest, read_json, require_actual_windows,
    verify_runtime_artifacts, verify_runtime_bundle, write_json)
from qualify_windows_continuous import (MODE, MONOTONIC_COUNTERS, NATIVE_STATS,
    counter, read_pixels, validate_limits)

STAGES = ("initial", "revision", "resize", "recovered")
MAX_COUNTER = 2**63 - 1
COLLECTOR_FILES = ("qualify_windows_tile_group.py", "qualify_windows_continuous.py", "qualify_desktop_texture.py")


def collector_sources():
    return {"bindings/Flutter/tool/" + name: digest(Path(__file__).with_name(name)) for name in COLLECTOR_FILES}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def integer(value, minimum=0, maximum=MAX_COUNTER):
    return type(value) is int and minimum <= value <= maximum


def finite(value, minimum, maximum):
    return type(value) in (int, float) and math.isfinite(value) and minimum <= value <= maximum


def snapshot_ids_match(value, expected):
    return isinstance(value, list) and all(integer(item, 1) for item in value) and value == expected


def validate_stamp(stamp):
    require(isinstance(stamp, dict) and set(stamp) == {
        "documentId", "revision", "fontGeneration", "viewGeneration", "deviceGeneration", "deviceScale"},
        "complete source/view/device stamp is required")
    for key in ("documentId", "revision", "fontGeneration", "viewGeneration", "deviceGeneration"):
        require(integer(stamp[key], 0 if key in ("revision", "fontGeneration") else 1), "invalid stamp " + key)
    require(finite(stamp["deviceScale"], .25, 8), "actual bounded device scale is required")


def validate_geometry(stage):
    width, height = stage["width"], stage["height"]
    require(integer(width, 2, 2048) and integer(height, 1, 512), "bounded complete Windows viewport")
    stamp, group, submit = stage["stamp"], stage["group"], stage["submission"]
    validate_stamp(stamp)
    validate_stamp(submit["stamp"])
    require(finite(stage.get("deviceScale"), .25, 8) and stage["deviceScale"] == stamp["deviceScale"], "source and displayed DPR disagree")
    offset = stage.get("physicalOffset")
    require(isinstance(offset, list) and len(offset) == 2 and all(integer(x, 0, 16384) for x in offset),
            "integer physical capture offset is required")
    view = stage.get("flutterView")
    require(isinstance(view, dict) and integer(view.get("width"), width, 16384) and
            integer(view.get("height"), height, 16384) and
            finite(view.get("deviceScale"), .25, 8) and view["deviceScale"] == stamp["deviceScale"] and
            offset[0] + width <= view["width"] and offset[1] + height <= view["height"],
            "target must fit the actual FlutterView at its observed DPR")
    members, tiles, rects = group["members"], submit["tiles"], stage["tileRects"]
    require(all(isinstance(v, list) and len(v) == 2 for v in (members, tiles, rects)),
            "two actual native texture members, submissions and rectangles are required")
    ids = [member["textureId"] for member in members]
    require(all(integer(value.get(key), 1) for value in (group, submit) for key in ("groupId", "deviceGeneration")) and
            all(integer(v, 1) for v in ids) and ids[0] != ids[1] and
            group["groupId"] == submit["groupId"] == ids[0] and
            group["deviceGeneration"] == submit["deviceGeneration"] == stamp["deviceGeneration"] and
            submit["stamp"] == stamp, "tile owner identity or full-stamp mismatch")
    snapshots = stage["snapshotIds"]
    require(isinstance(snapshots, list) and len(snapshots) == 1 and integer(snapshots[0], 1),
            "one immutable page capture must be shared by both tiles")
    accepted = stage.get("acceptance")
    require(isinstance(accepted, dict) and accepted.get("accepted") is True and
            accepted.get("acceptedStamp") == stamp and snapshot_ids_match(accepted.get("acceptedSnapshots"), snapshots),
            "actual group acceptance receipt must bind the unique snapshot and full stamp")
    validate_stamp(accepted["acceptedStamp"])
    left = 0
    for member, tile, rect in zip(members, tiles, rects):
        require(all(integer(rect.get(key), 0, 4096) for key in ("left", "top", "width", "height")) and
                rect["left"] == left and rect["top"] == 0 and rect["height"] == height and rect["width"] > 0 and
                tile["textureId"] == member["textureId"] and tile["width"] == rect["width"] and tile["height"] == height,
                "integer adjacent tile rectangles must cover the viewport")
        left += rect["width"]
    require(left == width, "tile union does not cover the complete viewport")
    first, second = tiles[0]["placements"], tiles[1]["placements"]
    require(isinstance(first, list) and isinstance(second, list) and len(first) == len(second) == 1,
            "one placement per immutable shared source")
    first, second = first[0], second[0]
    require(integer(first["snapshot"], 1) and integer(second["snapshot"], 1) and
            first["snapshot"] == second["snapshot"] == snapshots[0] and
            finite(first["scale"], .000001, 16) and first["scale"] == second["scale"] and
            finite(first["originX"], -16777216, 16777216) and finite(first["originY"], -16777216, 16777216) and
            first["originX"] - rects[1]["left"] == second["originX"] and first["originY"] == second["originY"],
            "tile seam must use one unchanged page transform")
    strips = stage.get("strips")
    require(isinstance(strips, list) and len(strips) == 1, "one independent CPU strip must cross the native seam")
    strip = strips[0]
    require(strip["left"] == 0 and strip["width"] == width, "independent oracle must cover every visible pixel")
    for factor, proof in ((1, strip["cpu"]), (2, strip["reference"])):
        validate_stamp(proof["stamp"])
        require(integer(proof["snapshot"], 1) and proof["snapshot"] == snapshots[0] and proof["stamp"] == stamp and
                proof["coordinateSpace"] == "viewportPixels" and
                proof["viewport"] == {"width": width * factor, "height": height * factor} and
                proof["placement"] == {"scale": first["scale"] * factor,
                    "originX": first["originX"] * factor, "originY": first["originY"] * factor},
                "independent CPU replay must use the same immutable source and page placement")
        descriptor = proof["descriptor"]
        require(descriptor["width"] == width * factor and descriptor["height"] == height * factor and
                descriptor["stride"] == width * factor * 4 and descriptor["pixelFormat"] == "rgba8Premultiplied" and
                descriptor["colorSpace"] == "srgb" and descriptor["origin"] == "topLeft", "CPU oracle geometry differs")
    return width, height, tuple(offset)


def validate_stage(stage, *, nonce, index, pid):
    try:
        require(integer(index, 1, 4) and isinstance(stage, dict) and
                integer(stage.get("protocol"), 1, 1) and stage.get("nonce") == nonce and
                integer(stage.get("index"), index, index) and integer(stage.get("pid"), pid, pid) and
                stage.get("platform") == "windows" and stage.get("stage") == STAGES[index - 1], "foreign tile publication")
        dimensions = validate_geometry(stage)
        for member in stage["group"]["members"]:
            validate_stamp(member["acceptedStamp"])
            validate_stamp(member["renderedStamp"])
            require(member.get("publicationMode") == MODE and counter(member, "deviceGeneration") == stage["stamp"]["deviceGeneration"],
                    "missing actual Windows immutable-transfer owner")
            for key, expected in (("ready", True), ("registered", True), ("hasFrame", True),
                                  ("closed", False), ("ownerFinished", False), ("inFlight", False),
                                  ("unregistered", False), ("deviceFailure", False)):
                require(member.get(key) is expected, "tile is not a held current native publication: " + key)
            require(member.get("error") == member.get("errorCode") == "" and
                    member.get("acceptedStamp") == member.get("renderedStamp") == stage["stamp"] and
                    snapshot_ids_match(member.get("acceptedSnapshots"), stage["snapshotIds"]), "tile acceptance/render source mismatch")
            serial = counter(member, "acceptedSerial")
            require(serial > 0 and counter(member, "renderedSerial") == counter(member, "descriptorOpenedSerial") == serial,
                    "both actual texture descriptors must be opened for the current native serial")
            limits = validate_limits(member.get("limits"))
            require(limits["producerOwnedFrames"] >= 2 and limits["descriptorOpenedNotifications"] >= 2 * index and
                    limits["handedOffFrames"] + limits["producerOwnedFrames"] >= 2 * index,
                    "both tile publications must be represented in process-wide ownership accounting")
        return dimensions
    except (KeyError, TypeError, IndexError) as error:
        raise ValueError("malformed two-tile publication receipt") from error


def validate_closed(closed, last):
    try:
        require(closed["groupId"] == last["group"]["groupId"] and
                closed["deviceGeneration"] == last["stamp"]["deviceGeneration"] and
                isinstance(closed["members"], list) and len(closed["members"]) == 2, "foreign or incomplete tile retirement")
        for member, prior in zip(closed["members"], last["group"]["members"]):
            for key in ("textureId", "deviceGeneration", "publicationMode", "acceptedSerial", "acceptedStamp", "acceptedSnapshots"):
                require(member.get(key) == prior[key], "retirement belongs to a different owner/publication: " + key)
            for key, expected in (("closed", True), ("ownerFinished", True), ("unregistered", True),
                                  ("registered", False), ("ready", False), ("inFlight", False),
                                  ("hasFrame", False), ("deviceFailure", False)):
                require(member.get(key) is expected, "tile unregister/retirement was not observed: " + key)
            require(member.get("error") == member.get("errorCode") == "" and member.get("renderedStamp") is None and
                    counter(member, "renderedSerial") == counter(member, "descriptorOpenedSerial") == 0, "closed owner retained a rendered target")
            limits = validate_limits(member.get("limits"), closed=True)
            for key in MONOTONIC_COUNTERS:
                require(limits[key] >= prior["limits"][key], "retirement lost lifetime evidence")
    except (KeyError, TypeError, IndexError) as error:
        raise ValueError("malformed tile retirement receipt") from error


def validate_transition(previous, current):
    before, after = previous["stamp"], current["stamp"]
    index = current["index"]
    require(index == previous["index"] + 1 and all(after[key] == before[key] for key in
        ("documentId", "fontGeneration", "deviceScale")), "document owner, font or actual DPR changed within one run")
    require(after["viewGeneration"] > before["viewGeneration"], "full view generation did not advance")
    require(after["revision"] > before["revision"] if index == 2 else after["revision"] == before["revision"],
            "edit/viewport/recovery changed the wrong source revision")
    old_ids = [m["textureId"] for m in previous["group"]["members"]]
    new_ids = [m["textureId"] for m in current["group"]["members"]]
    if index == 4:
        validate_closed(current.get("recoveryRetirement"), previous)
        require(after["deviceGeneration"] != before["deviceGeneration"],
                "recovery must replace the retired native device generation")
    else:
        require(new_ids == old_ids and after["deviceGeneration"] == before["deviceGeneration"], "edit/resize replaced tile owners")
        for old, new in zip(previous["group"]["members"], current["group"]["members"]):
            require(new["acceptedSerial"] > old["acceptedSerial"], "native tile serial did not advance")
    for old, new in zip(previous["group"]["members"], current["group"]["members"]):
        for key in MONOTONIC_COUNTERS:
            require(new["limits"][key] >= old["limits"][key], "process-wide lifetime counter went backwards")
    if index == 3:
        require((current["width"], current["height"]) != (previous["width"], previous["height"]), "target did not resize")
    if index == 4:
        require((current["width"], current["height"]) == (previous["width"], previous["height"]), "recovery changed the viewport")


def validate_result(value, *, nonce, stages):
    require(isinstance(value, dict) and integer(value.get("protocol"), 1, 1) and value.get("nonce") == nonce and
            value.get("ok") is True and value.get("platform") == "windows" and
            integer(value.get("publications"), 4, 4) and integer(value.get("captures"), 4, 4) and
            integer(value.get("callerReleases"), 0, 0) and
            value.get("consumerGpuCompletionVerified") is False and value.get("hardwareAccelerationVerified") is False,
            "incomplete or overstated Windows tile completion")
    require(len(stages) == 4 and value.get("recoveryRetirement") == stages[3].get("recoveryRetirement"),
            "same-document recovery evidence is missing")
    validate_closed(value.get("closed"), stages[-1])
    stats = value.get("stats")
    require(isinstance(stats, dict), "Native owner teardown counters are missing")
    for key in NATIVE_STATS:
        require(counter(stats, key) == 0, "Native teardown did not drain " + key)


def compare_stage(stage, output, captured, index):
    width, height = stage["width"], stage["height"]
    require(len(captured) == width * height * 4, "actual desktop capture length differs")
    expected = read_pixels(output / f"cpu-{index}-0.rgba", width * height * 4)
    reference = read_pixels(output / f"reference-{index}-0.rgba", width * height * 16)
    compared = compare_pixels(expected, captured, width, height, source_reference=reference, reference_scale=2)
    # This complete independent replay crosses both sides of the native seam.
    # The predicate rejects each bad interior pixel rather than averaging it
    # across the viewport, including a one-column gap at that boundary.
    seam = stage["tileRects"][1]["left"]
    return {"comparison": compared, "seam": seam,
            "cpuSha256": hashlib.sha256(expected).hexdigest(),
            "referenceSha256": hashlib.sha256(reference).hexdigest()}


def validate_pixel_transition(previous, current, index):
    if index in (2, 3):
        require(current["cpuSha256"] != previous["cpuSha256"], "edit/resize did not change independent CPU output")
    if index == 4:
        require(current["cpuSha256"] == previous["cpuSha256"], "same-owner recovery changed independent source pixels")


class TileCompositor(WindowsCompositor):
    def __init__(self, pid):
        super().__init__(pid)
        self.user.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
        self.user.GetDpiForWindow.argtypes = [wintypes.HWND]
        self.user.GetDpiForWindow.restype = wintypes.UINT
        self.user.SetWindowPos.argtypes = [wintypes.HWND, wintypes.HWND, ctypes.c_int, ctypes.c_int,
                                           ctypes.c_int, ctypes.c_int, wintypes.UINT]

    def metrics(self):
        window = self.window()
        client = wintypes.RECT()
        require(bool(self.user.GetClientRect(window, ctypes.byref(client))), "Win32 client geometry unavailable")
        dpi = self.user.GetDpiForWindow(window)
        require(24 <= dpi <= 768, "actual per-window DPI unavailable")
        return {"width": client.right - client.left, "height": client.bottom - client.top, "deviceScale": dpi / 96}

    def resize_client(self):
        before = self.metrics()
        width = before["width"] + 32
        height = before["height"] + 32
        require(width <= 8192 and height <= 8192, "bounded Win32 client resize")
        window = self.window()
        frame = wintypes.RECT()
        require(bool(self.user.GetWindowRect(window, ctypes.byref(frame))), "Win32 outer geometry unavailable")
        require(bool(self.user.SetWindowPos(window, None, 0, 0,
            width + frame.right - frame.left - before["width"],
            height + frame.bottom - frame.top - before["height"], 0x0002 | 0x0004 | 0x0010)), "actual Win32 client resize failed")
        after = self.metrics()
        require((after["width"], after["height"]) == (width, height), "Windows did not apply the requested client resize")
        require(after["deviceScale"] == before["deviceScale"], "OS DPR changed during client resize; restart at a stable scale")
        return {"before": before, "after": after}


def validate_capture_evidence(item):
    stage, evidence = item["stage"], item.get("comparison")
    require(isinstance(evidence, dict) and evidence.get("seam") == stage["tileRects"][1]["left"],
            "DPR evidence lacks the independent comparison across the native seam")
    comparison = evidence.get("comparison")
    pixels = stage["width"] * stage["height"]
    require(isinstance(comparison, dict) and comparison.get("comparison") ==
            "opaque-interiors-1-source-palette13-edge1px-zero-integer-shift" and
            integer(comparison.get("sourceReferenceScale"), 2, 2) and
            integer(comparison.get("sourceReferenceBytes"), pixels * 16, pixels * 16) and
            integer(comparison.get("uniformInteriorPixels"), pixels // 4, pixels) and
            integer(comparison.get("aaEdgeDifferentPixels"), 0, pixels) and
            integer(comparison.get("maxChannelDifference"), 0, 255), "DPR evidence lacks a complete independent-source comparison")
    errors = comparison.get("integerAlignmentErrors")
    require(isinstance(errors, dict) and set(errors) == {f"{dx},{dy}" for dy in (-1, 0, 1) for dx in (-1, 0, 1)} and
            all(integer(error, 0, pixels * 3 * 255) for error in errors.values()),
            "DPR evidence lacks the complete bounded integer-alignment controls")
    require(all(error > errors["0,0"] for shift, error in errors.items() if shift != "0,0"),
            "DPR CPU source/compositor integer alignment is shifted or ambiguous")
    require(all(re.fullmatch(r"[0-9a-f]{64}", str(value)) for value in
                (evidence.get("cpuSha256"), evidence.get("referenceSha256"), item.get("compositorSha256"))),
            "DPR evidence lacks CPU/source/actual capture identities")


def validate_dpr_pair(previous, current):
    for value in (previous, current):
        require(value.get("qualification") == "windows-two-tile-consumer-passed" and integer(value.get("exitCode"), 0, 0) and
                value.get("platform") == "windows" and value.get("capture") == "visible-win32-desktop" and
                len(value.get("stages", [])) == 4,
                "distinct-DPR comparison requires two completed actual Windows tile runs")
        require(all(value.get(key) is False for key in ("consumerGpuCompletionVerified", "hardwareAccelerationVerified",
                    "targetReuse", "consumerAndDriverMemoryIncluded")), "DPR evidence cannot overstate consumer/GPU guarantees")
        stages = [item["stage"] for item in value["stages"]]
        hashes = value.get("collectorSourceFiles")
        require(isinstance(hashes, dict) and set(hashes) == {"bindings/Flutter/tool/" + name for name in COLLECTOR_FILES} and
                all(re.fullmatch(r"[0-9a-f]{64}", str(fingerprint)) and value["build"].get("sourceFiles", {}).get(path) == fingerprint
                    for path, fingerprint in hashes.items()), "DPR evidence lacks pinned collector dependency identities")
        for item in value["stages"]:
            validate_capture_evidence(item)
        require(all(item.get("window") == item["stage"].get("flutterView") for item in value["stages"]),
                "DPR evidence must retain matching actual Win32 and FlutterView metrics")
        resize = value.get("windowResize")
        require(isinstance(resize, dict) and resize.get("before") == stages[1]["flutterView"] and
                resize.get("after") == stages[2]["flutterView"] == stages[3]["flutterView"] and
                all(resize["after"][key] == resize["before"][key] + 32 for key in ("width", "height")),
                "DPR evidence must retain the actual Win32 client resize")
        for index, stage in enumerate(stages, 1):
            validate_stage(stage, nonce=value["nonce"], index=index, pid=value["pid"])
            if index > 1:
                validate_transition(stages[index - 2], stage)
                validate_pixel_transition(value["stages"][index - 2]["comparison"], value["stages"][index - 1]["comparison"], index)
        validate_result(value["result"], nonce=value["nonce"], stages=stages)
    require(previous["stages"][0]["stage"]["deviceScale"] != current["stages"][0]["stage"]["deviceScale"],
            "DPR comparison requires two different actual OS scales")
    for key in ("sourceHead", "nativeSourceHead", "flutterRevision", "engineRevision"):
        require(previous["build"][key] == current["build"][key], "distinct-DPR runs used different source/engine identities")
    for key in ("runner", "native", "dispatcher", "plugin", "engineRuntime", "fixture"):
        require(previous["build"]["artifacts"][key]["sha256"] == current["build"]["artifacts"][key]["sha256"],
                "distinct-DPR runs used different runtime artifacts")
    require(previous["build"]["runtimeBundle"] == current["build"]["runtimeBundle"], "distinct-DPR runtime content changed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("runner", "native", "dispatcher", "fixture", "build-receipt", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=280)
    parser.add_argument("--expected-device-scale", type=float)
    parser.add_argument("--previous-dpr-receipt", type=Path)
    args = parser.parse_args()
    try:
        require_actual_windows()
    except RuntimeError as error:
        parser.error(str(error))
    require(finite(args.timeout, 30, 300), "bounded qualification duration")
    require(args.expected_device_scale is None or finite(args.expected_device_scale, .25, 8), "bounded expected actual DPR")
    files = {key: getattr(args, key).resolve(strict=True) for key in ("runner", "native", "dispatcher", "fixture")}
    require(files["fixture"].stat().st_size <= 16 * 1024 * 1024, "bounded source fixture")
    build = read_json(args.build_receipt, limit=MAX_BUILD_RECEIPT_BYTES)
    require(build.get("platform") == "windows" and build.get("buildExitCode") == 0 and
            build.get("probeKind") == "twoTileGroup", "matching actual Windows two-tile build receipt required")
    collector_hashes = collector_sources()
    collector_hash = collector_hashes["bindings/Flutter/tool/qualify_windows_tile_group.py"]
    require(all(build.get("sourceFiles", {}).get(path) == fingerprint for path, fingerprint in collector_hashes.items()),
            "collector and its runtime helpers must match the pinned build source")
    for key in ("sourceHead", "nativeSourceHead", "flutterRevision", "engineRevision"):
        require(re.fullmatch(r"[0-9a-f]{40}", str(build.get(key, ""))) is not None, "missing pinned build identity: " + key)
    files = verify_runtime_artifacts(files, build)
    verify_runtime_bundle(files["runner"].parent, build.get("runtimeBundle"))
    output = args.output.resolve()
    require(not output.is_relative_to(files["runner"].parent), "capture output must be outside the immutable runtime bundle")
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    nonce = uuid.uuid4().hex
    write_json(output / "config.json", {"protocol": 1, "nonce": nonce, "output": str(output),
        "backend": "windowsTwoTileGroup", **{key: str(value) for key, value in files.items()}})
    state = {"platform": "windows", "qualification": "pending", "nonce": nonce, "build": build,
        "collectorSha256": collector_hash, "collectorSourceFiles": collector_hashes, "capture": "visible-win32-desktop", "stages": [],
        "started": time.time(), "exitCode": None, "actualDprComparison": False,
        "consumerGpuCompletionVerified": False, "hardwareAccelerationVerified": False,
        "targetReuse": False, "consumerAndDriverMemoryIncluded": False}
    env = dict(os.environ, RHWP_CONSUMER_PROBE_CONFIG=str(output / "config.json"))
    process = None
    deadline = time.monotonic() + args.timeout

    def check_deadline():
        require(time.monotonic() < deadline, "Windows tile qualification deadline expired")

    try:
        with (output / "runner.log").open("wb") as log:
            (output / "runner.log").chmod(0o600)
            process = subprocess.Popen([str(files["runner"])], cwd=files["runner"].parent,
                env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
            state["pid"] = process.pid
            write_json(output / "receipt.json", state)
            compositor = TileCompositor(process.pid)
            for index in range(1, 5):
                path = output / f"stage-{index}.json"
                while not path.exists():
                    check_deadline()
                    require(process.poll() is None, "runner exited before tile publication")
                    time.sleep(.025)
                check_deadline()
                stage = read_json(path)
                width, height, offset = validate_stage(stage, nonce=nonce, index=index, pid=process.pid)
                metrics = compositor.metrics()
                require(metrics == stage["flutterView"], "Win32 client/DPI differs from actual FlutterView")
                if args.expected_device_scale is not None:
                    require(metrics["deviceScale"] == args.expected_device_scale, "actual OS DPR differs from requested qualification scale")
                if index > 1:
                    validate_transition(state["stages"][-1]["stage"], stage)
                if index >= 3:
                    require(metrics == state["windowResize"]["after"], "resized actual client metrics were not consumed")
                captured = compositor.capture(width, height, offset)
                image = output / f"compositor-{index}.rgba"
                image.write_bytes(captured)
                image.chmod(0o600)
                comparison = compare_stage(stage, output, captured, index)
                check_deadline()
                time.sleep(.1)
                require(compositor.capture(width, height, offset) == captured, "held two-tile desktop pixels changed")
                check_deadline()
                if index > 1:
                    validate_pixel_transition(state["stages"][-1]["comparison"], comparison, index)
                state["stages"].append({"stage": stage, "window": metrics, "comparison": comparison,
                    "compositorSha256": hashlib.sha256(captured).hexdigest()})
                if index == 2:
                    state["windowResize"] = compositor.resize_client()
                    require(state["windowResize"]["before"] == metrics, "window changed between capture and owned resize")
                write_json(output / "receipt.json", state)
                check_deadline()
                write_json(output / f"ack-{index}.json", {"protocol": 1, "nonce": nonce, "index": index})
            check_deadline()
            code = process.wait(timeout=deadline - time.monotonic())
        check_deadline()
        require(code == 0, "runner did not complete actual tile close/unregister")
        result = read_json(output / "result.json")
        validate_result(result, nonce=nonce, stages=[item["stage"] for item in state["stages"]])
        verify_runtime_artifacts(files, build)
        verify_runtime_bundle(files["runner"].parent, build.get("runtimeBundle"))
        require(collector_sources() == collector_hashes, "collector inputs changed during qualification")
        check_deadline()
        state.update(qualification="windows-two-tile-consumer-passed", exitCode=code, result=result)
        if args.previous_dpr_receipt is not None:
            previous = read_json(args.previous_dpr_receipt, limit=2 * MAX_BUILD_RECEIPT_BYTES)
            validate_dpr_pair(previous, state)
            state.update(actualDprComparison=True, previousDprReceiptSha256=digest(args.previous_dpr_receipt))
        check_deadline()
    except BaseException as error:
        state.update(qualification="failed", error=f"{type(error).__name__}: {error}")
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=10)
        if process is not None:
            state["exitCode"] = process.returncode
        raise
    finally:
        state["finished"] = time.time()
        write_json(output / "receipt.json", state)
    print(output / "receipt.json")


if __name__ == "__main__":
    main()
