#!/usr/bin/env python3
"""Compare a real two-target Flutter viewport with independent CPU strip replays.

The >4096px viewport cannot use one native target. Source references are replayed
at 2x from the same immutable snapshot; each CPU request stays below 16 MiB.
Xvfb/Mesa execution is recorded as software evidence, never hardware completion.
"""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import time
import uuid

from check_macos_metal_sdk import stop_process_tree
from qualify_desktop_texture import (MAX_BUILD_RECEIPT_BYTES, compare_pixels,
    read_json, verify_runtime_artifacts, verify_runtime_bundle, write_json)
from qualify_linux_texture import X11Compositor, require_linux, verify_loaded_artifacts


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_stage(stage, nonce, index, pid):
    require(stage.get("protocol") == 1 and stage.get("nonce") == nonce and
            stage.get("index") == index and stage.get("pid") == pid, "foreign tile stage")
    width, height = stage.get("width"), stage.get("height")
    require(type(width) is int and type(height) is int and 4096 < width <= 8192 and
            0 < height <= 4096 and width * height <= 8 * 1024 * 1024, "bounded large viewport")
    stamp, group, submit = stage["stamp"], stage["group"], stage["submission"]
    members, tiles, rects = group["members"], submit["tiles"], stage["tileRects"]
    require(len(members) == len(tiles) == len(rects) == 2, "exactly two actual native outputs")
    ids = [member["textureId"] for member in members]
    require(all(type(i) is int and i > 0 for i in ids) and ids[0] != ids[1] and
            group["groupId"] == ids[0] == submit["groupId"] and
            group["deviceGeneration"] == submit["deviceGeneration"] == stamp["deviceGeneration"],
            "one stamp generation and two distinct owners")
    require(submit["stamp"] == stamp and stage["deviceScale"] == stamp["deviceScale"], "full stamp")
    left = 0
    for member, tile, rect in zip(members, tiles, rects):
        require(all(type(rect.get(key)) is int for key in ("left", "top", "width", "height")), "integer physical tile boundaries")
        require(tile["textureId"] == member["textureId"] and member["deviceGeneration"] == stamp["deviceGeneration"] and
                member["acceptedStamp"] == member["renderedStamp"] == stamp and
                member["registered"] is True and member["closing"] is False and
                member["liveRasterSlots"] == 2 and member["rasterSlotLimit"] == 4 and
                member["framesSubmitted"] >= index, "both current physical publications")
        for key in ("wholeFrameReadbacks", "abandonedTargets", "orphanedOwners",
                    "orphanedRasterSlots", "pendingRetireSlots", "snapshotReleaseFailures"):
            require(member.get(key) == 0, "unexpected native counter: " + key)
        require(rect["left"] == left and rect["top"] == 0 and rect["height"] == height and
                0 < rect["width"] <= 4096 and tile["width"] == rect["width"] and tile["height"] == height,
                "adjacent integer physical rectangles")
        left += rect["width"]
    require(left == width and len(stage["snapshotIds"]) == 1, "full union / one unique capture")
    first, second = tiles[0]["placements"], tiles[1]["placements"]
    require(len(first) == len(second) == 1 and first[0]["snapshot"] == second[0]["snapshot"] == stage["snapshotIds"][0] and
            first[0]["scale"] == second[0]["scale"] and
            first[0]["originX"] - rects[1]["left"] == second[0]["originX"] and
            first[0]["originY"] == second[0]["originY"], "shared page transform across seam")
    left = 0
    for strip in stage["strips"]:
        cpu, ref = strip["cpu"], strip["reference"]
        n = strip["width"]
        require(strip["left"] == left and type(n) is int and 0 < n <= 2048 and
                n * 2 <= 4096 and height * 2 <= 4096 and n * height * 4 <= 4 * 1024 * 1024,
                "bounded complete CPU strips and 2x reference")
        left += n
        for factor, proof in ((1, cpu), (2, ref)):
            require(proof["snapshot"] == stage["snapshotIds"][0] and proof["stamp"] == stamp and
                    proof["coordinateSpace"] == "viewportPixels" and
                    proof["viewport"] == {"width": n * factor, "height": height * factor}, "independent immutable CPU source")
            descriptor = proof["descriptor"]
            require(descriptor["width"] == n * factor and descriptor["height"] == height * factor and
                    descriptor["stride"] == n * factor * 4 and descriptor["pixelFormat"] == "rgba8Premultiplied" and
                    descriptor["colorSpace"] == "srgb" and descriptor["origin"] == "topLeft", "CPU descriptor geometry")
            placement = proof["placement"]
            require(placement == {"scale": first[0]["scale"] * factor,
                    "originX": (first[0]["originX"] - strip["left"]) * factor,
                    "originY": first[0]["originY"] * factor}, "independent source strip translation")
    require(left == width, "every compositor pixel has an independent CPU oracle")
    return width, height, tuple(stage["physicalOffset"])


def crop(rgba, width, left, extent, height):
    return b"".join(rgba[(row * width + left) * 4:(row * width + left + extent) * 4] for row in range(height))


def compare_stage(stage, output, captured, index):
    width, height = stage["width"], stage["height"]
    comparisons = []
    total_colors = set()
    for strip in stage["strips"]:
        left, extent = strip["left"], strip["width"]
        expected = (output / f"cpu-{index}-{left}.rgba").read_bytes()
        reference = (output / f"reference-{index}-{left}.rgba").read_bytes()
        require(len(expected) == extent * height * 4 and len(reference) == extent * height * 16, "CPU strip lengths")
        displayed = crop(captured, width, left, extent, height)
        colors = set(expected[i:i+4] for i in range(0, len(expected), 4))
        total_colors.update(colors)
        if len(colors) < 3:
            require(all(abs(a-b) <= 1 for a, b in zip(expected, displayed)), "uniform compositor strip changed")
            compared = {"comparison": "uniform-source-interiors-1", "pixels": extent * height}
        else:
            compared = compare_pixels(expected, displayed, extent, height,
                                      source_reference=reference, reference_scale=2)
        comparisons.append({"left": left, "width": extent, "comparison": compared,
            "cpuSha256": hashlib.sha256(expected).hexdigest(), "referenceSha256": hashlib.sha256(reference).hexdigest()})
    require(len(total_colors) >= 3, "independent oracle lacks visible document content")
    seam = stage["tileRects"][1]["left"]
    require(any(item["left"] < seam < item["left"] + item["width"] for item in comparisons), "a CPU comparison must cross the actual native seam")
    return comparisons


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("runner", "native", "dispatcher", "fixture", "build-receipt", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=240)
    args = parser.parse_args()
    require_linux()
    require(30 <= args.timeout <= 300, "bounded qualification duration")
    files = {key: getattr(args, key).resolve(strict=True) for key in ("runner", "native", "dispatcher", "fixture")}
    build = read_json(args.build_receipt, limit=MAX_BUILD_RECEIPT_BYTES)
    require(build.get("buildExitCode") == 0 and build.get("probeKind") == "twoTileGroup", "clean pinned tile-group build receipt")
    files = verify_runtime_artifacts(files, build, platform="linux")
    verify_runtime_bundle(files["runner"].parent, build["runtimeBundle"])
    output = args.output.resolve()
    require(not output.is_relative_to(files["runner"].parent), "capture output must be outside the runtime bundle")
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    nonce = uuid.uuid4().hex
    write_json(output / "config.json", {"protocol": 1, "nonce": nonce, "output": str(output),
        **{key: str(value) for key, value in files.items()}})
    state = {"qualification": "pending", "build": build, "stages": [], "started": time.time(),
        "capture": "visible-x11-root", "consumerGpuCompletionVerified": False, "hardwareAccelerationVerified": False}
    env = dict(os.environ, GDK_BACKEND="x11", GTK_CSD="0", RHWP_CONSUMER_PROBE_CONFIG=str(output / "config.json"))
    process = compositor = None
    log_path = output / "runner.log"
    try:
        with log_path.open("wb") as log:
            log_path.chmod(0o600)
            process = subprocess.Popen([str(files["runner"])], cwd=files["runner"].parent, env=env,
                stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                start_new_session=True, preexec_fn=lambda: os.umask(0o077))
            state["pid"] = process.pid
            compositor = X11Compositor(process.pid)
            deadline = time.monotonic() + args.timeout
            for index in range(1, 4):
                path = output / f"stage-{index}.json"
                while not path.exists():
                    require(process.poll() is None and time.monotonic() < deadline, "runner exited/timed out before tile stage")
                    time.sleep(.025)
                stage = read_json(path)
                width, height, offset = validate_stage(stage, nonce, index, process.pid)
                captured = compositor.capture(width, height, offset)
                image = output / f"compositor-{index}.rgba"
                image.write_bytes(captured); image.chmod(0o600)
                comparisons = compare_stage(stage, output, captured, index)
                time.sleep(.1)
                require(compositor.capture(width, height, offset) == captured, "held compositor pixels changed")
                loaded = verify_loaded_artifacts(process.pid, {key: files[key] for key in ("runner", "native", "dispatcher", "plugin", "engineRuntime")})
                state["stages"].append({"stage": stage, "comparisons": comparisons, "loaded": loaded,
                    "compositorSha256": hashlib.sha256(captured).hexdigest()})
                write_json(output / "receipt.json", state)
                write_json(output / f"ack-{index}.json", {"protocol": 1, "nonce": nonce, "index": index})
            code = process.wait(timeout=max(.1, deadline-time.monotonic()))
            result = read_json(output / "result.json")
            require(code == 0 and result.get("ok") is True and result.get("nonce") == nonce and
                    result.get("publications") == result.get("captures") == 3 and result.get("callerReleases") == 0,
                    "actual group lifecycle result")
            for member in result["closed"]["members"]:
                require(member["registered"] is False and member["cleanupComplete"] is True, "both registrations retired")
                for key in ("liveRasterSlots", "pendingRetireSlots", "orphanedRasterSlots", "abandonedTargets", "orphanedOwners", "snapshotReleaseFailures", "wholeFrameReadbacks"):
                    require(member[key] == 0, "native tile teardown " + key)
            for key in ("sources", "snapshotHandles", "retainedSnapshots", "snapshotEstimatedBytes", "frameHandles", "retainedFrames", "frameBytes", "leases", "inFlightReplays"):
                require(result["stats"][key] == 0, "Native owner teardown " + key)
            verify_runtime_artifacts(files, build, platform="linux")
            verify_runtime_bundle(files["runner"].parent, build["runtimeBundle"])
            state.update(qualification="linux-two-tile-consumer-passed", exitCode=code, result=result)
    except BaseException as error:
        state.update(qualification="failed", error=f"{type(error).__name__}: {error}")
        if process is not None and process.poll() is None:
            stop_process_tree(process)
        if process is not None:
            state["exitCode"] = process.returncode
        raise
    finally:
        if compositor is not None:
            compositor.close()
        state["finished"] = time.time()
        write_json(output / "receipt.json", state)
    print(output / "receipt.json")


if __name__ == "__main__":
    main()
