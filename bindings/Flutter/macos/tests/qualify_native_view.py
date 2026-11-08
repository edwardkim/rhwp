#!/usr/bin/env python3
"""Build/capture an actual macOS AppKitView probe; portable validation is separate."""

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import signal
import stat
import subprocess
import sys
import time
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tool"))
from qualify_desktop_texture import compare_pixels, digest, positive_integer, read_json, write_json
import qualify_desktop_texture as pixel_contract
from build_desktop_probe import artifact, checked_sha, sources, verify_copied_sources

MAX_ENTRIES = 4096
MAX_FILES = 2048
MAX_BYTES = 2 * 1024**3
MAX_RECEIPT = 1024 * 1024


def app_executable(bundle):
    """Use the bundle entry point even when Xcode emits adjacent debug dylibs."""
    bundle = Path(bundle).resolve(strict=True)
    info = bundle / "Contents/Info.plist"
    if info.is_symlink() or not info.is_file():
        raise ValueError("Mac app Info.plist must be an owned regular file")
    with info.open("rb") as source:
        data = source.read(64 * 1024 + 1)
    if len(data) > 64 * 1024:
        raise ValueError("Mac app Info.plist exceeds its byte bound")
    try:
        metadata = plistlib.loads(data)
    except Exception as error:
        raise ValueError("invalid Mac app Info.plist") from error
    name = metadata.get("CFBundleExecutable") if isinstance(metadata, dict) else None
    if not isinstance(name, str) or name in (".", "..") or not re.fullmatch(r"[^/\\\x00-\x1f]+", name):
        raise ValueError("Mac app lacks a bounded executable basename")
    executable = bundle / "Contents/MacOS" / name
    resolved = executable.resolve(strict=True)
    resolved.relative_to(bundle)
    if executable.is_symlink() or not resolved.is_file() or not os.access(resolved, os.X_OK):
        raise ValueError("Mac bundle entry point must be an owned executable file")
    return resolved


def bundle_manifest(directory):
    """Allow standard .framework internal links, never escaping or linked inputs."""
    directory = Path(directory)
    if directory.is_symlink():
        raise ValueError("Mac runtime bundle root must not be a symlink")
    root = directory.resolve(strict=True)
    if not root.is_dir():
        raise ValueError("Mac runtime bundle must be a directory")
    result, files, size = {}, 0, 0
    for current, directories, names in os.walk(root, followlinks=False):
        for name in sorted(directories + names):
            item = Path(current) / name
            relative = item.relative_to(root).as_posix()
            if len(result) >= MAX_ENTRIES:
                raise ValueError("Mac runtime bundle entry bound exceeded")
            if item.is_symlink():
                destination = item.resolve(strict=True)
                destination.relative_to(root)
                target = os.readlink(item)
                if os.path.isabs(target):
                    raise ValueError("bundle links must be relative")
                result[relative] = {"kind": "link", "target": target}
                if name in directories:
                    directories.remove(name)
            elif item.is_dir():
                result[relative] = {"kind": "directory"}
            elif item.is_file():
                before = item.stat()
                if not stat.S_ISREG(before.st_mode):
                    raise ValueError("runtime bundle entry is not a regular file")
                length = before.st_size
                files += 1
                size += length
                if files > MAX_FILES or size > MAX_BYTES:
                    raise ValueError("Mac runtime bundle byte/file bound exceeded")
                fingerprint = digest(item, maximum=MAX_BYTES-size+length)
                after = item.stat()
                identity = lambda value: (value.st_size, value.st_mtime_ns, value.st_ctime_ns)
                if identity(before) != identity(after):
                    raise ValueError("Mac runtime input changed while hashing")
                result[relative] = {"kind": "file", "bytes": length, "sha256": fingerprint}
            else:
                raise ValueError("unsupported runtime bundle entry")
    # Debug probes execute this Dart program, not merely the Mach-O entry point.
    if not any(key.endswith("/flutter_assets/kernel_blob.bin") and value.get("kind") == "file" and value["bytes"] > 0
               for key, value in result.items()):
        raise ValueError("actual Debug Dart kernel is absent from the runtime bundle")
    return result


def verify_bundle(directory, expected):
    if not isinstance(expected, dict) or bundle_manifest(directory) != expected:
        raise ValueError("Mac runtime bundle changed or contains missing/extra entries")


def validate_stage(stage, *, nonce, index, pid, previous=None):
    if type(stage.get("protocol")) is not int or stage["protocol"] != 1 or stage.get("nonce") != nonce or type(stage.get("index")) is not int or stage["index"] != index:
        raise ValueError("foreign or out-of-order Mac probe stage")
    if stage.get("platform") != "macos" or stage.get("pid") != pid:
        raise ValueError("stage does not belong to the actual launched Mac process")
    expected_name = ("initial", "revision", "resize")[index-1] if index <= 3 else "continuous"
    if stage.get("stage") != expected_name:
        raise ValueError("missing ordered initial/revision/resize/continuous controls")
    width = positive_integer(stage.get("width"), 4096)
    height = positive_integer(stage.get("height"), 4096)
    if width * height > 4194304:
        raise ValueError("viewport pixel bound exceeded")
    dpr = stage.get("deviceScale")
    if type(dpr) not in (int, float) or not math.isfinite(dpr) or not 0 < dpr <= 16:
        raise ValueError("invalid physical device scale")
    offset = stage.get("physicalOffset")
    if not isinstance(offset, list) or len(offset) != 2 or any(type(x) is not int or not 0 <= x <= 16384 for x in offset):
        raise ValueError("missing aligned physical content offset")
    stamp = stage.get("stamp")
    if not isinstance(stamp, dict) or stamp.get("deviceScale") != dpr:
        raise ValueError("full source stamp/DPR mismatch")
    for key in ("documentId", "viewGeneration", "deviceGeneration"):
        positive_integer(stamp.get(key), 2**63-1)
    for key in ("revision", "fontGeneration"):
        if type(stamp.get(key)) is not int or not 0 <= stamp[key] < 2**63:
            raise ValueError("invalid source generation")
    limits = stage.get("limits", {})
    if not isinstance(limits, dict):
        raise ValueError("missing bounded Native limits")
    if limits.get("backend") != "appkitMetalLayer" or limits.get("presentationKind") != "nativeView":
        raise ValueError("probe used the legacy CVTexture or another renderer")
    if limits.get("continuousPresentationSupported") is not True or limits.get("maximumDrawableCount") != 3:
        raise ValueError("missing bounded Core Animation presentation contract")
    if limits.get("consumerGpuCompletionVerified") is not False or limits.get("maxViews") != 2:
        raise ValueError("unqualified consumer fence or unbounded owner budget")
    if previous:
        old = previous["stamp"]
        if any(stamp[k] != old[k] for k in ("documentId", "fontGeneration", "deviceGeneration", "deviceScale")):
            raise ValueError("unplanned source/device identity change")
        if stamp["revision"] < old["revision"] or stamp["viewGeneration"] <= old["viewGeneration"]:
            raise ValueError("publication is stale or did not advance its view")
        if index == 2 and (stamp["revision"] <= old["revision"] or (width, height) != (previous["width"], previous["height"])):
            raise ValueError("revision control did not change source at a stable viewport")
        if index == 3 and (stamp["revision"] != old["revision"] or (width, height) == (previous["width"], previous["height"])):
            raise ValueError("resize control did not change only the viewport")
    return width, height, tuple(offset), dpr


def validate_result(value, *, nonce, frames):
    if value.get("protocol") != 1 or value.get("nonce") != nonce or value.get("ok") is not True:
        raise ValueError("foreign or failed actual Mac completion")
    if value.get("consumerGpuCompletionVerified") is not False:
        raise ValueError("screen sampling must not fabricate a consumer GPU fence")
    if value.get("continuousPresentationSupported") is not True:
        raise ValueError("probe did not identify continuous presentation")
    closed = value.get("closed", {})
    if closed.get("ownerFinished") is not True or closed.get("registered") is not False or closed.get("nativeViewLayerDetached") is not True:
        raise ValueError("Native producer/layer close was not observed")
    if closed.get("quarantined") is not False:
        raise ValueError("qualification left an unknown-completion owner")
    if type(closed.get("frames")) is not int or closed["frames"] < frames:
        raise ValueError("did not present all requested continuous publications")
    for key in ("sources", "snapshotHandles", "retainedSnapshots", "snapshotEstimatedBytes", "frameHandles",
                "retainedFrames", "frameBytes", "leases", "inFlightReplays"):
        if type(value.get("stats", {}).get(key)) is not int or value["stats"][key] != 0:
            raise ValueError("Native teardown did not return to zero: " + key)


def validate_cover(value, *, nonce, pid, previous):
    if value.get("protocol") != 1 or value.get("nonce") != nonce or value.get("pid") != pid or value.get("platform") != "macos":
        raise ValueError("foreign actual Mac held-cover request")
    for key in ("width", "height", "deviceScale", "physicalOffset"):
        if value.get(key) != previous[key]:
            raise ValueError("held cover changed its published physical geometry")
    for key in ("nativeCovered", "nativeViewMounted", "viewBound", "layerReady"):
        if value.get(key) is not True:
            raise ValueError("held cover lost its mounted native view or Native acknowledgment")
    if value.get("presentationKind") != "nativeView":
        raise ValueError("held cover did not use AppKitView")
    positive_integer(value.get("visibilitySerial"), 2**63-1)
    color = value.get("clearColor")
    if type(color) is not int or not 0 <= color <= 0xffffffff or color >> 24 != 0xff:
        raise ValueError("held-cover color must be opaque ARGB")
    return bytes(((color >> 16) & 0xff, (color >> 8) & 0xff, color & 0xff, 0xff))


def capture_pair(files, *, output, label, pid, width, height, offset, dpr, deadline,
                 validate_sample=None):
    """Require an exact held pair; settle only CPU-valid one-unit quantization."""
    from PIL import Image
    captures, metadata = [], []
    low = high = None
    trace = {"state": "pending", "maximumSamples": 4 if validate_sample else 2,
             "samples": [], "exactSelectedPair": False}
    trace_path = output / f"held-pair-{label}.json"
    write_json(trace_path, trace)
    try:
        for suffix in (("a", "b", "c", "d") if validate_sample else ("a", "b")):
            png = output / f"window-{label}-{suffix}.png"
            command = [str(files["capture"]), str(pid), str(dpr), str(png)]
            remaining = deadline-time.monotonic()
            if remaining <= 0:
                raise TimeoutError("actual Mac capture exceeded its overall deadline")
            completed = subprocess.run(command, capture_output=True, text=True, timeout=min(30, remaining))
            if completed.returncode:
                raise RuntimeError("actual WindowServer capture failed: " + completed.stderr[-2000:])
            if len(completed.stdout) > 64 * 1024:
                raise ValueError("capture helper metadata exceeded its bound")
            meta = json.loads(completed.stdout)
            if meta.get("pid") != pid or meta.get("deviceScale") != dpr or meta.get("actualWindowServerCapture") is not True or meta.get("colorSpace") != "srgb" or meta.get("dynamicRange") != "sdr":
                raise ValueError("capture helper returned a foreign/unsupported image")
            positive_integer(meta.get("windowId"), 2**32-1)
            capture_width = positive_integer(meta.get("width"), 4096)
            capture_height = positive_integer(meta.get("height"), 4096)
            if capture_width * capture_height > 4194304 or png.stat().st_size > 64 * 1024**2:
                raise ValueError("captured image exceeded its physical byte/pixel bound")
            with Image.open(png) as image:
                if image.size != (capture_width, capture_height) or offset[0]+width > image.width or offset[1]+height > image.height:
                    raise ValueError("physical viewport escaped the actual Mac window")
                captures.append(image.convert("RGBA").crop((offset[0], offset[1], offset[0]+width, offset[1]+height)).tobytes())
            metadata.append(meta)
            sample = {"suffix": suffix, "capture": meta,
                      "compositorSha256": hashlib.sha256(captures[-1]).hexdigest()}
            trace["samples"].append(sample)
            write_json(trace_path, trace)
            try:
                if meta != metadata[0]:
                    raise ValueError("actual Mac window metadata changed during a held publication")
                if validate_sample:
                    # Every observation must match the independent immutable CPU
                    # snapshot, including observations before the selected pair.
                    sample["comparison"] = validate_sample(captures[-1])
                if time.monotonic() >= deadline:
                    raise TimeoutError("actual Mac capture validation exceeded its overall deadline")
                if low is None:
                    low, high = bytearray(captures[-1]), bytearray(captures[-1])
                else:
                    for i, value in enumerate(captures[-1]):
                        low[i], high[i] = min(low[i], value), max(high[i], value)
                        if high[i] - low[i] > (0 if i % 4 == 3 else 1):
                            raise ValueError("actual Mac held samples exceeded their one-unit RGB cluster")
                if time.monotonic() >= deadline:
                    raise TimeoutError("actual Mac capture stability exceeded its overall deadline")
                if len(captures) >= 2:
                    if captures[-1] == captures[-2]:
                        trace.update(state="passed", exactSelectedPair=True,
                                     selectedSuffixes=[trace["samples"][-2]["suffix"], suffix])
                        write_json(trace_path, trace)
                        return captures[-1], dict(meta, heldCapture=trace)
                    if not validate_sample:
                        raise ValueError("actual Mac compositor/window changed during a held publication")
                write_json(trace_path, trace)
            except BaseException as error:
                trace.update(state="failed", error=f"{type(error).__name__}: {error}")
                write_json(trace_path, trace)
                raise
            time.sleep(.1)
        trace.update(state="failed", error="no exact consecutive pair within the four-sample bound")
        write_json(trace_path, trace)
        raise ValueError("actual Mac held samples never formed an exact consecutive pair")
    except BaseException as error:
        trace.update(state="failed", error=f"{type(error).__name__}: {error}")
        write_json(trace_path, trace)
        raise


def require_macos():
    if sys.platform != "darwin":
        raise RuntimeError("actual macOS/Apple SDK is required; portable tests cannot qualify this target")


def stop_owned_process_group(child):
    """Reap the leader and kill its owned descendants even if TERM exits early."""
    try:
        os.killpg(child.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(child.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    child.wait()


def run_step(command, *, cwd, env, output, receipt, state, timeout=600, read_output=False):
    log_path = output / f"step-{len(state['steps'])}.log"
    step = {"command": command, "cwd": str(cwd), "log": str(log_path), "started": time.time()}
    state["steps"].append(step)
    with log_path.open("wb") as log:
        log_path.chmod(0o600)
        child = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                 stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        step["pid"] = child.pid
        try:
            write_json(receipt, state)
            code = child.wait(timeout=timeout)
        except BaseException:
            stop_owned_process_group(child)
            raise
        finally:
            step.update(exitCode=child.returncode, finished=time.time())
            write_json(receipt, state)
    if code:
        raise RuntimeError("actual build step failed; inspect its completed log: " + str(log_path))
    if read_output:
        if log_path.stat().st_size > MAX_RECEIPT:
            raise ValueError("machine-readable build output exceeded its bound")
        return log_path.read_text(encoding="utf-8", errors="replace")
    return None


def build(args):
    require_macos()
    repository = Path(__file__).resolve().parents[4]
    if subprocess.check_output(["git", "status", "--porcelain", "--", "bindings/Flutter", "bindings/swift/Sources/CRhwpNative"], cwd=repository, text=True):
        raise ValueError("commit the Mac qualification inputs before building")
    source_head = checked_sha(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repository, text=True).strip(), "source head")
    native = args.native.resolve(strict=True)
    native_receipt = read_json(args.native_receipt, limit=16 * 1024 * 1024)
    native_head = checked_sha(native_receipt.get("nativeSourceHead", native_receipt.get("source_head")), "Native source head")
    native_hash = digest(native)
    if native_receipt.get("platform") != "macos" or native_receipt.get("build") != "passed" or native_receipt.get("artifacts", {}).get("native", {}).get("sha256") != native_hash:
        raise ValueError("Native library does not match its actual successful Apple SDK receipt")
    sdk = args.flutter_sdk.resolve(strict=True)
    flutter = str(sdk / "bin/flutter")
    output = args.output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    receipt = output / "macos-view-build.json"
    expected = sources(repository)
    expected["LICENSE"] = digest(repository / "LICENSE")
    copied = output / "source"
    for relative in expected:
        destination = copied / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repository / relative, destination)
    env = dict(os.environ, CI="true", FLUTTER_SUPPRESS_ANALYTICS="true", DART_SUPPRESS_ANALYTICS="true")
    state = {"platform": "macos", "sourceHead": source_head, "nativeSourceHead": native_head,
             "nativeSdkReceiptSha256": digest(args.native_receipt), "sourceFiles": expected,
             "steps": [], "started": time.time(), "pid": os.getpid(), "buildExitCode": None}
    try:
        version = json.loads(run_step([flutter, "--version", "--machine"], cwd=output, env=env, output=output, receipt=receipt, state=state, read_output=True))
        state["flutterRevision"] = checked_sha(version.get("frameworkRevision"), "Flutter revision")
        state["engineRevision"] = checked_sha(version.get("engineRevision"), "engine revision")
        if state["flutterRevision"] != native_receipt.get("flutterRevision") or state["engineRevision"] != native_receipt.get("engineRevision"):
            raise ValueError("actual SDK and Native compile/link receipt use different framework/engine revisions")
        run_step([flutter, "create", "--platforms=macos", "--project-name=rhwp_metal_view_qualification", "runner"], cwd=output, env=env, output=output, receipt=receipt, state=state)
        runner = output / "runner"
        (runner / "pubspec.yaml").write_text("name: rhwp_metal_view_qualification\npublish_to: none\nenvironment:\n  sdk: '>=3.13.0 <4.0.0'\ndependencies:\n  flutter:\n    sdk: flutter\n  rhwp_native_reference:\n    path: ../source/bindings/Flutter\n")
        shutil.copyfile(copied / "bindings/Flutter/test/support/desktop_consumer_probe.dart", runner / "lib/main.dart")
        # This disposable runner needs external fixture/DLL paths. It is borderless
        # for exact content coordinates; production example and sandbox stay intact.
        (runner / "macos/Runner/DebugProfile.entitlements").write_text('<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>com.apple.security.cs.allow-jit</key><true/></dict></plist>')
        project = runner / "macos/Runner.xcodeproj/project.pbxproj"
        project.write_text(re.sub(r"MACOSX_DEPLOYMENT_TARGET = [0-9.]+;", "MACOSX_DEPLOYMENT_TARGET = 12.0;", project.read_text()))
        (runner / "macos/Runner/MainFlutterWindow.swift").write_text('''import Cocoa
import FlutterMacOS
class MainFlutterWindow: NSWindow {
  override var canBecomeKey: Bool { true }
  override func awakeFromNib() {
    let controller = FlutterViewController()
    contentViewController = controller
    styleMask = [.borderless]
    hasShadow = false
    setFrame(NSRect(x: 64, y: 64, width: 640, height: 480), display: true)
    RegisterGeneratedPlugins(registry: controller)
    super.awakeFromNib()
    makeKeyAndOrderFront(nil)
    NSApp.activate(ignoringOtherApps: true)
  }
}
''')
        run_step([flutter, "pub", "get"], cwd=runner, env=env, output=output, receipt=receipt, state=state)
        run_step([flutter, "build", "macos", "--debug", "--no-pub"], cwd=runner, env=env, output=output, receipt=receipt, state=state, timeout=1200)
        apps = list((runner / "build/macos/Build/Products/Debug").glob("*.app"))
        if len(apps) != 1:
            raise ValueError("expected exactly one built actual Mac app")
        app = apps[0]
        executable = app_executable(app)
        dispatcher = output / "librhwp_flutter_owner.dylib"
        run_step([sys.executable, str(copied / "bindings/Flutter/tool/build_dispatcher.py"), "--output", str(dispatcher)], cwd=output, env=env, output=output, receipt=receipt, state=state)
        capture = output / "capture-metal-window"
        run_step(["xcrun", "swiftc", "-parse-as-library", "-target", platform.machine()+"-apple-macos14.0", str(copied / "bindings/Flutter/macos/tests/capture_metal_window.swift"), "-framework", "AppKit", "-framework", "ScreenCaptureKit", "-framework", "CoreGraphics", "-framework", "ImageIO", "-o", str(capture)], cwd=output, env=env, output=output, receipt=receipt, state=state)
        state["artifacts"] = {key: artifact(path) for key, path in {"runner": executable, "native": native,
            "dispatcher": dispatcher, "fixture": copied / "bindings/Flutter/test/fixtures/desktop-consumer.hwpx", "capture": capture}.items()}
        state["runtimeBundle"] = bundle_manifest(app)
        actual_sources = sources(repository)
        actual_sources["LICENSE"] = digest(repository / "LICENSE")
        if actual_sources != expected:
            raise ValueError("source checkout changed during the actual Mac build")
        if state["artifacts"]["native"]["sha256"] != native_hash:
            raise ValueError("Native SDK artifact changed during the actual Mac build")
        verify_copied_sources(copied, expected)
        state.update(buildExitCode=0, bundleRoot=str(app), disposableRunnerSandboxDisabled=True,
                     productionSandboxChanged=False, captureSource="WindowServer ScreenCaptureKit")
    except BaseException as error:
        state.update(buildExitCode=1, error=f"{type(error).__name__}: {error}")
        raise
    finally:
        state["finished"] = time.time()
        if len(json.dumps(state, indent=2).encode("utf-8")) + 1 > MAX_RECEIPT:
            state.update(buildExitCode=1, error="Mac build receipt exceeds its byte bound")
            state.pop("runtimeBundle", None)
        write_json(receipt, state)
    if state["buildExitCode"]:
        raise ValueError(state["error"])
    print(receipt)


def capture(args):
    require_macos()
    receipt = read_json(args.build_receipt, limit=MAX_RECEIPT)
    if receipt.get("platform") != "macos" or receipt.get("buildExitCode") != 0:
        raise ValueError("successful actual Mac build receipt is required")
    for key in ("sourceHead", "nativeSourceHead", "flutterRevision", "engineRevision"):
        checked_sha(receipt.get(key), key)
    if not isinstance(receipt.get("artifacts"), dict) or set(receipt["artifacts"]) != {"runner", "native", "dispatcher", "fixture", "capture"}:
        raise ValueError("Mac build receipt must contain exactly the owned runtime artifacts")
    files = {key: Path(value["path"]).resolve(strict=True) for key, value in receipt["artifacts"].items()}
    for key, path in files.items():
        if digest(path) != receipt["artifacts"][key]["sha256"]:
            raise ValueError("built runtime artifact changed: " + key)
    bundle = files["runner"].parents[2]
    if bundle.suffix != ".app" or bundle != Path(receipt["bundleRoot"]).resolve(strict=True):
        raise ValueError("Mac executable escaped its recorded app bundle")
    if files["fixture"].stat().st_size > 16 * 1024**2:
        raise ValueError("fixture exceeds its Native document byte bound")
    verify_bundle(bundle, receipt["runtimeBundle"])
    output = args.output.resolve()
    try:
        output.relative_to(bundle)
    except ValueError:
        pass
    else:
        raise ValueError("capture output must be outside the immutable app bundle")
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    nonce = uuid.uuid4().hex
    config = {"protocol": 1, "nonce": nonce, "output": str(output), "frameCount": args.frames, "cpuReference": True, "nativeCoverProbe": True,
              "backend": "macosMetalView", "presentationKind": "nativeView",
              **{key: str(value) for key, value in files.items()}}
    write_json(output / "config.json", config)
    state = {"platform": "macos", "qualification": "pending", "started": time.time(), "stages": [],
             "consumerGpuCompletionVerified": False, "hardwareAccelerationVerified": False,
             "continuousPresentationSupported": True, "build": receipt, "heldCoverVerified": False,
             "nativeCoverStrategy": "nativeHideAndOpaqueFlutterCover",
             "flutterOverlayOccludesVisibleNativeViewVerified": False}
    process = None
    log_path = output / "runner.log"
    try:
        with log_path.open("wb") as log:
            log_path.chmod(0o600)
            process = subprocess.Popen([str(files["runner"])], cwd=files["runner"].parent,
                env=dict(os.environ, RHWP_CONSUMER_PROBE_CONFIG=str(output / "config.json")), stdin=subprocess.DEVNULL,
                stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            state["pid"] = process.pid
            write_json(output / "receipt.json", state)
            deadline = time.monotonic() + args.timeout
            previous = None
            for index in range(1, args.frames+1):
                stage_file = output / f"stage-{index}.json"
                while not stage_file.exists():
                    cover_file = output / "cover-1.json"
                    if index == 2 and not state["heldCoverVerified"] and cover_file.exists():
                        cover = read_json(cover_file)
                        color = validate_cover(cover, nonce=nonce, pid=process.pid, previous=previous)
                        actual, metadata = capture_pair(files, output=output, label="cover-1", pid=process.pid,
                            width=cover["width"], height=cover["height"], offset=cover["physicalOffset"],
                            dpr=cover["deviceScale"], deadline=deadline)
                        if actual != color * (cover["width"] * cover["height"]):
                            raise ValueError("AppKitView pierced the opaque Flutter held cover")
                        if metadata["windowId"] != state["stages"][0]["capture"]["windowId"]:
                            raise ValueError("held cover used a different native window")
                        state.update(heldCoverVerified=True, heldCover={"request": cover, "capture": metadata,
                            "compositorSha256": hashlib.sha256(actual).hexdigest()})
                        write_json(output / "receipt.json", state)
                        write_json(output / "cover-ack-1.json", {"protocol": 1, "nonce": nonce, "index": 1})
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError("actual Mac probe exited or timed out before stage " + str(index))
                    time.sleep(.025)
                stage = read_json(stage_file)
                width, height, offset, dpr = validate_stage(stage, nonce=nonce, index=index, pid=process.pid, previous=previous)
                reference_scale = pixel_contract.validate_source_reference(stage, width, height)
                oracle = output / f"cpu-{index}.rgba"
                if oracle.stat().st_size != width * height * 4:
                    raise ValueError("bounded CPU oracle byte length mismatch")
                expected = oracle.read_bytes()
                reference = output / f"cpu-reference-{index}.rgba"
                if reference.stat().st_size != width * height * 4 * reference_scale**2 or reference.stat().st_size > 16 * 1024**2:
                    raise ValueError("bounded independent source reference byte length mismatch")
                source_reference = reference.read_bytes()
                if index == 2 and digest(oracle) == state["stages"][0]["cpuSha256"]:
                    raise ValueError("revision control did not change the independently replayed CPU pixels")
                actual, metadata = capture_pair(files, output=output, label=str(index), pid=process.pid,
                    width=width, height=height, offset=offset, dpr=dpr, deadline=deadline,
                    validate_sample=lambda pixels: compare_pixels(expected, pixels, width, height,
                        source_reference=source_reference, reference_scale=reference_scale))
                if previous is not None and metadata["windowId"] != state["stages"][0]["capture"]["windowId"]:
                    raise ValueError("publication used a different native window")
                comparison = compare_pixels(expected, actual, width, height,
                    source_reference=source_reference, reference_scale=reference_scale)
                compositor = output / f"compositor-{index}.rgba"
                compositor.write_bytes(actual)
                compositor.chmod(0o600)
                state["stages"].append({"stage": stage, "comparison": comparison, "cpuSha256": digest(oracle), "sourceReferenceSha256": digest(reference),
                    "capture": metadata, "compositorSha256": hashlib.sha256(actual).hexdigest()})
                write_json(output / "receipt.json", state)
                write_json(output / f"ack-{index}.json", {"protocol": 1, "nonce": nonce, "index": index})
                previous = stage
            code = process.wait(timeout=max(.1, deadline-time.monotonic()))
            result = read_json(output / "result.json")
            if code:
                raise ValueError("actual Mac Native close failed")
            validate_result(result, nonce=nonce, frames=args.frames)
            if not state["heldCoverVerified"]:
                raise ValueError("actual held-cover/native-view composition was never observed")
            verify_bundle(bundle, receipt["runtimeBundle"])
            for key, path in files.items():
                if digest(path) != receipt["artifacts"][key]["sha256"]:
                    raise ValueError("runtime artifact changed during capture: " + key)
            state.update(qualification="macos-appkit-metal-view-passed", exitCode=code, result=result,
                         flutterNativeViewSamplingVerified=True)
    except BaseException as error:
        state.update(qualification="failed", error=f"{type(error).__name__}: {error}")
        if process is not None:
            stop_owned_process_group(process)
        raise
    finally:
        state["finished"] = time.time()
        write_json(output / "receipt.json", state)
    print(output / "receipt.json")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_subparsers(dest="mode", required=True)
    builder = modes.add_parser("build")
    builder.add_argument("--flutter-sdk", type=Path, required=True)
    builder.add_argument("--native", type=Path, required=True)
    builder.add_argument("--native-receipt", type=Path, required=True)
    builder.add_argument("--output", type=Path, required=True)
    driver = modes.add_parser("capture")
    driver.add_argument("--build-receipt", type=Path, required=True)
    driver.add_argument("--output", type=Path, required=True)
    driver.add_argument("--frames", type=int, default=32)
    driver.add_argument("--timeout", type=float, default=240)
    args = parser.parse_args()
    if args.mode == "capture" and (not 8 <= args.frames <= 64 or not math.isfinite(args.timeout) or not 30 <= args.timeout <= 600):
        parser.error("capture requires 8..64 frames and a finite 30..600 second deadline")
    (build if args.mode == "build" else capture)(args)


if __name__ == "__main__":
    main()
