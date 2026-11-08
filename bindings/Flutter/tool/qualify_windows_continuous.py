#!/usr/bin/env python3
"""Qualify fresh immutable Windows publications on the real Flutter desktop.

Opening transfers creator references; it does not establish consumer GPU
completion, target reuse, or a bound on consumer/driver allocations.
"""

import argparse
import hashlib
import os
from pathlib import Path
import re
import subprocess
import time
import uuid

from qualify_desktop_texture import (MAX_BUILD_RECEIPT_BYTES, WindowsCompositor,
                                     compare_pixels, digest, read_json,
                                     require_actual_windows, validate_source_reference,
                                     validate_stage_identity, verify_runtime_artifacts,
                                     verify_runtime_bundle, write_json)


MODE = "immutableOwnershipTransfer"
MAX_FRAMES = 4
MAX_BYTES = 64 * 1024 * 1024
MAX_FRAME_BYTES = 16 * 1024 * 1024
MAX_TICKETS = 8
MAX_COUNTER = 2**63 - 1
NATIVE_STATS = ("sources", "snapshotHandles", "retainedSnapshots", "snapshotEstimatedBytes",
                "frameHandles", "retainedFrames", "frameBytes", "leases", "inFlightReplays")
MONOTONIC_COUNTERS = ("handedOffFrames", "discardedUnexposedFrames",
                      "descriptorOpenedNotifications", "descriptorDuplicateOrLateNotifications")


def stage_name(index):
    return ("initial", "revision", "resize")[index - 1] if index <= 3 else "continuous"


def counter(value, key, maximum=MAX_COUNTER):
    number = value.get(key)
    if type(number) is not int or not 0 <= number <= maximum:
        raise ValueError("missing or out-of-bound native counter: " + key)
    return number


def validate_limits(value, *, closed=False):
    if not isinstance(value, dict) or value.get("publicationMode") != MODE:
        raise ValueError("immutable ownership transfer mode was not observed")
    for key, expected in (("continuousPresentationSupported", True), ("consumerRetirement", False),
                          ("consumerGpuCompletionVerified", False), ("targetReuse", False),
                          ("consumerAndDriverMemoryIncluded", False),
                          ("descriptorReleaseIsGpuComplete", False),
                          ("cpuFallbackRequiredWhenSaturated", True)):
        if value.get(key) is not expected:
            raise ValueError("unqualified ownership/completion claim: " + key)
    if value.get("frameAccountingScope") != "legacyRetainedAndPluginOwnedReferences":
        raise ValueError("producer accounting must exclude consumer/driver allocations")
    for key, expected in (("maxFrames", MAX_FRAMES), ("maxTargetBytes", MAX_BYTES),
                          ("maxSingleFrameBytes", MAX_FRAME_BYTES), ("maxDescriptorTickets", MAX_TICKETS)):
        if counter(value, key) != expected:
            raise ValueError("native ownership budget changed: " + key)
    owned = counter(value, "producerOwnedFrames", MAX_FRAMES)
    owned_bytes = counter(value, "producerOwnedBytes", MAX_BYTES)
    pending = counter(value, "importPendingFrames", owned)
    pending_bytes = counter(value, "importPendingBytes", owned_bytes)
    tickets = counter(value, "descriptorTickets", MAX_TICKETS)
    counter(value, "descriptorCpuPendingTickets", tickets)
    counter(value, "descriptorAwaitingOpenedTickets", tickets)
    for count, size in ((owned, owned_bytes), (pending, pending_bytes)):
        if (count == 0) != (size == 0) or size > count * MAX_FRAME_BYTES:
            raise ValueError("frame count and charged bytes disagree")
    for key in ("legacyRetainedFrames", "immutableFrames", "quarantinedFrames", "wholeFrameReadbacks"):
        if counter(value, key) != 0:
            raise ValueError("legacy retention/quarantine/readback was observed: " + key)
    if (counter(value, "outstandingFrames", MAX_FRAMES) != owned or
            counter(value, "pendingRetireFrames", MAX_FRAMES) != owned or
            counter(value, "reservedTargetBytes", MAX_BYTES) != owned_bytes):
        raise ValueError("process-wide frame accounting is contradictory")
    for key in MONOTONIC_COUNTERS:
        counter(value, key)
    if value["handedOffFrames"] > value["descriptorOpenedNotifications"]:
        raise ValueError("creator references were handed off without a descriptor opening")
    if closed:
        if owned or pending or owned_bytes or pending_bytes or tickets:
            raise ValueError("confirmed unregister did not drain creator references/descriptor tickets")
    elif not owned:
        raise ValueError("held publication has no producer-owned immutable target")
    return value


def validate_stage(value, *, nonce, index, pid):
    if any(type(value.get(key)) is not int for key in ("protocol", "index", "pid")):
        raise ValueError("protocol/stage/process identity must use integers")
    dimensions = validate_stage_identity(value, nonce=nonce, index=index, pid=pid,
                                         platform="windows", name=stage_name(index))
    validate_source_reference(value, dimensions[0], dimensions[1])
    status = value.get("nativeStatus")
    if not isinstance(status, dict) or status.get("publicationMode") != MODE:
        raise ValueError("missing actual Windows ownership-transfer status")
    limits = validate_limits(status.get("limits"))
    if value.get("limits") != limits:
        raise ValueError("stage and native owner budget receipts disagree")
    for key, expected in (("ready", True), ("closed", False), ("ownerFinished", False),
                          ("inFlight", False), ("unregistered", False), ("hasFrame", True),
                          ("deviceFailure", False)):
        if status.get(key) is not expected:
            raise ValueError("held native publication is not ready/current: " + key)
    if status.get("error") != "" or status.get("errorCode") != "":
        raise ValueError("native publication reported an error")
    counter(status, "textureId")
    if counter(status, "deviceGeneration") != value["stamp"]["deviceGeneration"]:
        raise ValueError("native owner device generation differs from its source stamp")
    for key in ("acceptedStamp", "renderedStamp"):
        if status.get(key) != value["stamp"]:
            raise ValueError("native publication did not bind its full source stamp: " + key)
    serial = counter(status, "acceptedSerial")
    if (serial < index or counter(status, "renderedSerial") != serial or
            counter(status, "descriptorOpenedSerial") != serial):
        raise ValueError("current accepted/rendered/opened publication serial was not observed")
    snapshots = status.get("acceptedSnapshots")
    if (not isinstance(snapshots, list) or not 1 <= len(snapshots) <= 8 or
            any(type(item) is not int or not 0 < item <= MAX_COUNTER for item in snapshots) or
            len(set(snapshots)) != len(snapshots) or
            value["geometryProof"]["cpu"]["snapshot"] not in snapshots):
        raise ValueError("CPU oracle snapshot was not accepted by the native publication")
    if (limits["descriptorOpenedNotifications"] < index or
            limits["handedOffFrames"] + limits["producerOwnedFrames"] < index):
        raise ValueError("observed publications are missing opening/creator-ownership accounting")
    return dimensions


def validate_transition(previous, current):
    before, after = previous["stamp"], current["stamp"]
    if any(after[key] != before[key] for key in
           ("documentId", "fontGeneration", "deviceGeneration", "deviceScale")):
        raise ValueError("unexpected document/font/device change")
    if after["viewGeneration"] <= before["viewGeneration"]:
        raise ValueError("publication did not advance its full view generation")
    index = current["index"]
    if ((index == 2 and after["revision"] <= before["revision"]) or
            (index > 2 and after["revision"] != before["revision"])):
        raise ValueError("revision/viewport publication changed the wrong source state")
    if index == 3 and (current["width"], current["height"]) == (previous["width"], previous["height"]):
        raise ValueError("resize did not change actual physical target dimensions")
    old_status, new_status = previous["nativeStatus"], current["nativeStatus"]
    if new_status["textureId"] != old_status["textureId"]:
        raise ValueError("continuous publication replaced its registration")
    if new_status["acceptedSerial"] <= old_status["acceptedSerial"]:
        raise ValueError("native publication serial did not advance")
    for key in MONOTONIC_COUNTERS:
        if new_status["limits"][key] < old_status["limits"][key]:
            raise ValueError("native cumulative ownership counter went backwards: " + key)


def validate_result(value, *, nonce, frames, last_stage):
    if (type(value.get("protocol")) is not int or value["protocol"] != 1 or
            value.get("nonce") != nonce or value.get("ok") is not True or
            type(value.get("publications")) is not int or value["publications"] != frames or
            value.get("continuousPresentationSupported") is not True or
            value.get("consumerGpuCompletionVerified") is not False):
        raise ValueError("foreign, incomplete, or overstated Windows continuous completion")
    closed = value.get("closed")
    if not isinstance(closed, dict) or closed.get("publicationMode") != MODE:
        raise ValueError("actual ownership-transfer close receipt is missing")
    for key, expected in (("closed", True), ("ownerFinished", True), ("unregistered", True),
                          ("ready", False), ("inFlight", False), ("hasFrame", False),
                          ("deviceFailure", False)):
        if closed.get(key) is not expected:
            raise ValueError("actual producer completion/unregister was not observed: " + key)
    if (closed.get("error") != "" or closed.get("errorCode") != "" or
            closed.get("renderedStamp") is not None or counter(closed, "renderedSerial") or
            counter(closed, "descriptorOpenedSerial")):
        raise ValueError("closed native owner retained a rendered frame or error")
    last = last_stage["nativeStatus"]
    for key in ("textureId", "deviceGeneration", "acceptedSerial", "acceptedStamp"):
        if closed.get(key) != last[key]:
            raise ValueError("close receipt belongs to a different publication/owner: " + key)
    limits = validate_limits(closed.get("limits"), closed=True)
    for key in MONOTONIC_COUNTERS:
        if limits[key] < last["limits"][key]:
            raise ValueError("close lost cumulative native lifetime evidence: " + key)
    # Opening is observed for every captured frame. The latest creator pin can
    # survive until the confirmed unregister receipt; no per-open GPU fence is inferred.
    if limits["handedOffFrames"] < frames - 1:
        raise ValueError("too few immutable creator-reference handoffs for the observed publications")
    stats = value.get("stats")
    if not isinstance(stats, dict):
        raise ValueError("Native owner teardown statistics are missing")
    for key in NATIVE_STATS:
        if counter(stats, key) != 0:
            raise ValueError("Native source/snapshot/frame/lease teardown was not observed: " + key)


def read_pixels(path, size):
    if type(size) is not int or not 0 < size <= MAX_FRAME_BYTES:
        raise ValueError("CPU pixel input exceeds its 16 MiB bound")
    with path.open("rb") as source:
        data = source.read(size + 1)
    if len(data) != size:
        raise ValueError("CPU pixel byte length does not match its physical target")
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("runner", "native", "dispatcher", "fixture", "build-receipt", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--frames", type=int, default=32)
    parser.add_argument("--timeout", type=float, default=600)
    args = parser.parse_args()
    try:
        require_actual_windows()
    except RuntimeError as error:
        parser.error(str(error))
    if not 32 <= args.frames <= 64 or not 30 <= args.timeout <= 600:
        parser.error("frame count must be 32..64 and timeout 30..600 seconds")
    files = {key: getattr(args, key).resolve(strict=True) for key in ("runner", "native", "dispatcher", "fixture")}
    if files["fixture"].stat().st_size > 16 * 1024 * 1024:
        raise ValueError("reference document exceeds its byte limit")
    build = read_json(args.build_receipt, limit=MAX_BUILD_RECEIPT_BYTES)
    if build.get("platform") != "windows" or build.get("buildExitCode") != 0:
        raise ValueError("a matching successful actual Windows runner build is required")
    files = verify_runtime_artifacts(files, build)
    verify_runtime_bundle(files["runner"].parent, build.get("runtimeBundle"))
    for key in ("sourceHead", "nativeSourceHead", "flutterRevision", "engineRevision"):
        if not re.fullmatch(r"[0-9a-f]{40}", str(build.get(key, ""))):
            raise ValueError("build receipt lacks pinned source/engine identity: " + key)
    output = args.output.resolve()
    try:
        output.relative_to(files["runner"].parent)
    except ValueError:
        pass
    else:
        raise ValueError("capture output must be outside the immutable runtime bundle")
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    nonce = uuid.uuid4().hex
    config = {"protocol": 1, "nonce": nonce, "output": str(output), "frameCount": args.frames,
              "cpuReference": True, "backend": "windowsImmutableTransfer",
              "presentationKind": "externalTexture",
              **{key: str(value) for key, value in files.items()}}
    write_json(output / "config.json", config)
    env = dict(os.environ, RHWP_CONSUMER_PROBE_CONFIG=str(output / "config.json"))
    state = {"platform": "windows", "qualification": "pending", "build": build,
             "collectorSha256": digest(Path(__file__)), "capture": "visible-win32-desktop",
             "stages": [], "consumerGpuCompletionVerified": False, "targetReuse": False,
             "continuousPresentationSupported": False, "hardwareAccelerationVerified": False,
             "consumerAndDriverMemoryIncluded": False, "started": time.time(), "exitCode": None}
    process = None
    log_path = output / "runner.log"
    with log_path.open("wb") as log:
        log_path.chmod(0o600)
        try:
            process = subprocess.Popen([str(files["runner"])], cwd=files["runner"].parent, env=env,
                                       stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
            state["pid"] = process.pid
            write_json(output / "receipt.json", state)
            compositor = WindowsCompositor(process.pid)
            deadline = time.monotonic() + args.timeout
            previous = None
            for index in range(1, args.frames + 1):
                stage_path = output / f"stage-{index}.json"
                while not stage_path.exists():
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError("runner exited or timed out before " + stage_name(index))
                    time.sleep(.025)
                if time.monotonic() >= deadline:
                    raise TimeoutError("Windows continuous capture deadline expired before a ready stage")
                stage = read_json(stage_path)
                width, height, offset = validate_stage(stage, nonce=nonce, index=index, pid=process.pid)
                if previous is not None:
                    validate_transition(previous, stage)
                oracle_path = output / f"cpu-{index}.rgba"
                expected = read_pixels(oracle_path, width * height * 4)
                cpu_hash = hashlib.sha256(expected).hexdigest()
                if previous is not None and cpu_hash == state["stages"][-1]["cpuSha256"]:
                    raise ValueError("new publication did not change independently observable CPU pixels")
                scale = validate_source_reference(stage, width, height)
                reference_path = output / f"cpu-reference-{index}.rgba"
                reference = read_pixels(reference_path, width * height * 4 * scale**2)
                captured = compositor.capture(width, height, offset)
                # Retain failed actual captures as well as passing evidence.
                (output / f"compositor-{index}.rgba").write_bytes(captured)
                comparison = compare_pixels(expected, captured, width, height,
                                            source_reference=reference, reference_scale=scale)
                if time.monotonic() >= deadline:
                    raise TimeoutError("Windows continuous capture deadline expired during CPU comparison")
                time.sleep(.1)
                if compositor.capture(width, height, offset) != captured:
                    raise ValueError("visible desktop pixels changed during the held publication")
                if time.monotonic() >= deadline:
                    raise TimeoutError("Windows continuous capture deadline expired during held publication")
                state["stages"].append({"stage": stage, "comparison": comparison, "cpuSha256": cpu_hash,
                                        "sourceReferenceSha256": hashlib.sha256(reference).hexdigest(),
                                        "compositorSha256": hashlib.sha256(captured).hexdigest()})
                write_json(output / "receipt.json", state)
                if time.monotonic() >= deadline:
                    raise TimeoutError("Windows continuous capture deadline expired before acknowledgement")
                write_json(output / f"ack-{index}.json", {"protocol": 1, "nonce": nonce, "index": index})
                previous = stage
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError("Windows continuous capture deadline expired before close completion")
            code = process.wait(timeout=remaining)
            if time.monotonic() >= deadline:
                raise TimeoutError("Windows continuous capture deadline expired during close completion")
            result = read_json(output / "result.json")
            if code:
                raise ValueError("runner did not complete actual close/unregister checks")
            validate_result(result, nonce=nonce, frames=args.frames, last_stage=previous)
            verify_runtime_artifacts(files, build)
            verify_runtime_bundle(files["runner"].parent, build.get("runtimeBundle"))
            if time.monotonic() >= deadline:
                raise TimeoutError("Windows continuous capture deadline expired during final runtime verification")
            state.update(qualification="windows-immutable-transfer-consumer-passed", exitCode=code,
                         result=result, continuousPresentationSupported=True)
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
