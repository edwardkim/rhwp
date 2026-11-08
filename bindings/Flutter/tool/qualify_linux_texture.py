#!/usr/bin/env python3
"""Compare genuine X11 desktop pixels with the same Native snapshot CPU oracle.

This drives the real Flutter GTK runner. Xvfb/Mesa is supported and reported as
software execution; a widget screenshot or a separate GL context is not proof.
"""

import argparse
import ctypes as C
import hashlib
import os
from pathlib import Path
import re
import subprocess
import time
import uuid

from build_linux_probe import require_linux
from check_macos_metal_sdk import stop_process_tree
from qualify_desktop_texture import (MAX_BUILD_RECEIPT_BYTES, compare_pixels,
                                     read_json, validate_stage_identity,
                                     validate_source_reference,
                                     verify_runtime_artifacts, verify_runtime_bundle,
                                     write_json)


def stage_name(index):
    return ("initial", "revision", "resize")[index - 1] if index <= 3 else "continuous"


def validate_stage(value, *, nonce, index, pid):
    dimensions = validate_stage_identity(value, nonce=nonce, index=index, pid=pid,
                                         platform="linux", name=stage_name(index))
    validate_source_reference(value, dimensions[0], dimensions[1])
    status = value.get("nativeStatus")
    if not isinstance(status, dict) or status.get("renderedStamp") != value["stamp"]:
        raise ValueError("Linux publication did not bind its full rendered source stamp")
    for key in ("wholeFrameReadbacks", "abandonedTargets", "orphanedOwners",
                "orphanedRasterSlots", "pendingRetireSlots", "snapshotReleaseFailures"):
        if type(status.get(key)) is not int or status[key] != 0:
            raise ValueError("unexpected Linux native lifetime/readback counter: " + key)
    if (status.get("registered") is not True or status.get("closing") is not False or
            status.get("cleanupComplete") is not False or status.get("liveRasterSlots") != 1 or
            status.get("rasterSlotLimit") != 4):
        raise ValueError("Linux raster owner is not one bounded live registration")
    frames = status.get("framesSubmitted")
    if type(frames) is not int or not index <= frames < 2**63:
        raise ValueError("publication did not advance the real raster frame count")
    for key in ("glVendor", "glRenderer", "glVersion"):
        text = status.get(key)
        if not isinstance(text, str) or not text or len(text) > 256 or "\0" in text:
            raise ValueError("missing bounded actual-context driver identity: " + key)
    return dimensions


def validate_result(value, *, nonce, frames):
    if (value.get("protocol") != 1 or value.get("nonce") != nonce or
            value.get("ok") is not True or value.get("publications") != frames or
            value.get("consumerGpuCompletionVerified") is not False or
            value.get("continuousPresentationSupported") is not (frames > 4)):
        raise ValueError("foreign or incomplete Linux consumer completion")
    closed = value.get("closed", {})
    if closed.get("registered") is not False or closed.get("cleanupComplete") is not True:
        raise ValueError("actual raster cleanup/unregistration did not complete")
    for key in ("liveRasterSlots", "pendingRetireSlots", "orphanedRasterSlots",
                "abandonedTargets", "orphanedOwners", "snapshotReleaseFailures", "wholeFrameReadbacks"):
        if type(closed.get(key)) is not int or closed[key] != 0:
            raise ValueError("Linux teardown did not release its raster resources: " + key)
    stats = value.get("stats", {})
    for key in ("sources", "snapshotHandles", "retainedSnapshots", "snapshotEstimatedBytes",
                "frameHandles", "retainedFrames", "frameBytes", "leases", "inFlightReplays"):
        if type(stats.get(key)) is not int or stats[key] != 0:
            raise ValueError("Native owner teardown was not observed: " + key)


def verify_loaded_artifacts(pid, files):
    """Bind ELF mappings to the files whose hashes the receipt verified.

    LD_LIBRARY_PATH can otherwise select an identically named engine/plugin
    outside the immutable bundle. Check actual kernel device/inode identities.
    """
    with (Path("/proc") / str(pid) / "maps").open("rb") as source:
        data = source.read(2 * 1024 * 1024 + 1)
    if len(data) > 2 * 1024 * 1024:
        raise ValueError("process mapping evidence exceeds its bound")
    expected = {key: Path(path).resolve(strict=True) for key, path in files.items()}
    if "runner" in expected and not os.path.samefile(Path("/proc") / str(pid) / "exe", expected["runner"]):
        raise ValueError("launched PID is not the recorded runner")
    found = {}
    for line in data.decode("utf-8", errors="strict").splitlines():
        fields = line.split(None, 5)
        if len(fields) != 6 or not fields[5].startswith("/"):
            continue
        mapped = fields[5]
        for key, path in expected.items():
            if Path(mapped.removesuffix(" (deleted)")).name != path.name:
                continue
            metadata = path.stat()
            device = f"{os.major(metadata.st_dev):02x}:{os.minor(metadata.st_dev):02x}"
            if mapped.endswith(" (deleted)") or fields[3] != device or int(fields[4]) != metadata.st_ino:
                raise ValueError("loaded module differs from its build artifact: " + key)
            found[key] = {"path": mapped, "device": device, "inode": metadata.st_ino}
    if set(found) != set(expected):
        raise ValueError("recorded artifacts are not all mapped by the actual runner")
    return found


class X11Compositor:
    """Sample visible root-window pixels, with PID and occlusion checks."""

    class Attributes(C.Structure):
        _fields_ = [("x", C.c_int), ("y", C.c_int), ("width", C.c_int), ("height", C.c_int),
                    ("border_width", C.c_int), ("depth", C.c_int), ("visual", C.c_void_p),
                    ("root", C.c_ulong), ("window_class", C.c_int), ("bit_gravity", C.c_int),
                    ("win_gravity", C.c_int), ("backing_store", C.c_int),
                    ("backing_planes", C.c_ulong), ("backing_pixel", C.c_ulong),
                    ("save_under", C.c_int), ("colormap", C.c_ulong),
                    ("map_installed", C.c_int), ("map_state", C.c_int),
                    ("all_event_masks", C.c_long), ("your_event_mask", C.c_long),
                    ("do_not_propagate_mask", C.c_long), ("override_redirect", C.c_int),
                    ("screen", C.c_void_p)]

    class Image(C.Structure):
        _fields_ = [("width", C.c_int), ("height", C.c_int), ("xoffset", C.c_int),
                    ("format", C.c_int), ("data", C.c_void_p), ("byte_order", C.c_int),
                    ("bitmap_unit", C.c_int), ("bitmap_bit_order", C.c_int),
                    ("bitmap_pad", C.c_int), ("depth", C.c_int), ("bytes_per_line", C.c_int),
                    ("bits_per_pixel", C.c_int), ("red_mask", C.c_ulong),
                    ("green_mask", C.c_ulong), ("blue_mask", C.c_ulong)]

    def __init__(self, pid):
        require_linux()
        self.pid = pid
        self.x = C.CDLL("libX11.so.6")
        self.x.XOpenDisplay.argtypes = [C.c_char_p]
        self.x.XOpenDisplay.restype = C.c_void_p
        self.x.XDefaultRootWindow.argtypes = [C.c_void_p]
        self.x.XDefaultRootWindow.restype = C.c_ulong
        self.x.XInternAtom.argtypes = [C.c_void_p, C.c_char_p, C.c_int]
        self.x.XInternAtom.restype = C.c_ulong
        self.x.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong),
                                      C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)),
                                      C.POINTER(C.c_uint)]
        self.x.XGetWindowProperty.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_long,
                                             C.c_long, C.c_int, C.c_ulong,
                                             C.POINTER(C.c_ulong), C.POINTER(C.c_int),
                                             C.POINTER(C.c_ulong), C.POINTER(C.c_ulong),
                                             C.POINTER(C.c_void_p)]
        self.x.XGetWindowAttributes.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(self.Attributes)]
        self.x.XTranslateCoordinates.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_int,
                                                C.c_int, C.POINTER(C.c_int), C.POINTER(C.c_int),
                                                C.POINTER(C.c_ulong)]
        self.x.XGetImage.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_int, C.c_uint,
                                    C.c_uint, C.c_ulong, C.c_int]
        self.x.XGetImage.restype = C.POINTER(self.Image)
        self.x.XDestroyImage.argtypes = [C.POINTER(self.Image)]
        self.x.XRaiseWindow.argtypes = [C.c_void_p, C.c_ulong]
        self.x.XSync.argtypes = [C.c_void_p, C.c_int]
        self.x.XFree.argtypes = [C.c_void_p]
        self.x.XCloseDisplay.argtypes = [C.c_void_p]
        self.display = self.x.XOpenDisplay(None)
        if not self.display:
            raise RuntimeError("a real X11 display is required (an owned Xvfb server is supported)")
        self.root = self.x.XDefaultRootWindow(self.display)
        self.pid_atom = self.x.XInternAtom(self.display, b"_NET_WM_PID", 0)
        self.cardinal = self.x.XInternAtom(self.display, b"CARDINAL", 0)

    def close(self):
        if self.display:
            self.x.XCloseDisplay(self.display)
            self.display = None

    def children(self):
        root, parent, count = C.c_ulong(), C.c_ulong(), C.c_uint()
        children = C.POINTER(C.c_ulong)()
        if not self.x.XQueryTree(self.display, self.root, C.byref(root), C.byref(parent),
                                C.byref(children), C.byref(count)):
            raise RuntimeError("could not inspect actual X11 root children")
        try:
            if count.value > 4096:
                raise ValueError("X11 root window count exceeds its bound")
            return list(children[:count.value])
        finally:
            if children:
                self.x.XFree(children)

    def owner(self, window):
        actual, format_, count, remaining, data = C.c_ulong(), C.c_int(), C.c_ulong(), C.c_ulong(), C.c_void_p()
        code = self.x.XGetWindowProperty(self.display, window, self.pid_atom, 0, 1, 0,
                                         self.cardinal, C.byref(actual), C.byref(format_),
                                         C.byref(count), C.byref(remaining), C.byref(data))
        try:
            if code == 0 and actual.value == self.cardinal and format_.value == 32 and count.value == 1 and not remaining.value:
                return C.cast(data, C.POINTER(C.c_ulong))[0]
            return None
        finally:
            if data:
                self.x.XFree(data)

    def attributes(self, window):
        value = self.Attributes()
        if not self.x.XGetWindowAttributes(self.display, window, C.byref(value)):
            raise RuntimeError("could not inspect actual X11 window")
        return value

    def capture(self, width, height, offset):
        children = self.children()
        windows = [window for window in children
                   if self.owner(window) == self.pid and self.attributes(window).map_state == 2]
        if len(windows) != 1:
            raise ValueError("expected one visible X11 top-level window owned by the launched runner")
        window = windows[0]
        self.x.XRaiseWindow(self.display, window)
        self.x.XSync(self.display, 0)
        children = self.children()
        attr = self.attributes(window)
        x, y, child = C.c_int(), C.c_int(), C.c_ulong()
        if not self.x.XTranslateCoordinates(self.display, window, self.root, 0, 0,
                                            C.byref(x), C.byref(y), C.byref(child)):
            raise RuntimeError("could not locate the actual X11 client origin")
        if offset[0] + width > attr.width or offset[1] + height > attr.height:
            raise ValueError("probe viewport exceeds its actual X11 client")
        left, top = x.value + offset[0], y.value + offset[1]
        screen = self.attributes(self.root)
        if left < 0 or top < 0 or left + width > screen.width or top + height > screen.height:
            raise ValueError("actual probe pixels are outside the visible X11 root")
        for other in children[children.index(window) + 1:]:
            a = self.attributes(other)
            if a.map_state == 2 and a.window_class == 1 and a.x < left + width and a.y < top + height and a.x + a.width > left and a.y + a.height > top:
                raise ValueError("another visible X11 window occludes the probe")
        image = self.x.XGetImage(self.display, self.root, left, top, width, height, C.c_ulong(-1), 2)
        if not image:
            raise RuntimeError("actual visible X11 root capture failed")
        try:
            info = image.contents
            if (info.bits_per_pixel != 32 or info.byte_order != 0 or
                    (info.red_mask, info.green_mask, info.blue_mask) != (0xff0000, 0xff00, 0xff) or
                    not width * 4 <= info.bytes_per_line <= width * 4 + 256):
                raise ValueError("capture requires a bounded X11 24/32-bit little-endian RGB visual")
            raw = C.string_at(info.data, info.bytes_per_line * height)
            rgba = bytearray(width * height * 4)
            for row in range(height):
                source = raw[row * info.bytes_per_line:row * info.bytes_per_line + width * 4]
                destination = row * width * 4
                rgba[destination:destination + width * 4:4] = source[2::4]
                rgba[destination + 1:destination + width * 4:4] = source[1::4]
                rgba[destination + 2:destination + width * 4:4] = source[0::4]
                rgba[destination + 3:destination + width * 4:4] = bytes([255]) * width
            return bytes(rgba)
        finally:
            self.x.XDestroyImage(image)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("runner", "native", "dispatcher", "fixture", "build-receipt", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--frames", type=int, default=32)
    parser.add_argument("--timeout", type=float, default=240)
    args = parser.parse_args()
    require_linux()
    if not 3 <= args.frames <= 64 or not 30 <= args.timeout <= 300:
        parser.error("frame count must be 3..64 and timeout 30..300 seconds")
    files = {key: getattr(args, key).resolve(strict=True) for key in ("runner", "native", "dispatcher", "fixture")}
    receipt = read_json(args.build_receipt, limit=MAX_BUILD_RECEIPT_BYTES)
    if receipt.get("platform") != "linux" or receipt.get("buildExitCode") != 0:
        raise ValueError("a matching successful real Linux runner build is required")
    files = verify_runtime_artifacts(files, receipt, platform="linux")
    verify_runtime_bundle(files["runner"].parent, receipt.get("runtimeBundle"))
    for key in ("sourceHead", "nativeSourceHead", "flutterRevision", "engineRevision"):
        if not re.fullmatch(r"[0-9a-f]{40}", str(receipt.get(key, ""))):
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
              "cpuReference": True,
              **{key: str(value) for key, value in files.items()}}
    write_json(output / "config.json", config)
    env = dict(os.environ, GDK_BACKEND="x11", GTK_CSD="0",
               RHWP_CONSUMER_PROBE_CONFIG=str(output / "config.json"))
    state = {"platform": "linux", "qualification": "pending", "build": receipt,
             "stages": [], "consumerGpuCompletionVerified": False,
             "continuousPresentationSupported": False, "hardwareAccelerationVerified": False,
             "capture": "visible-x11-root", "started": time.time(), "exitCode": None}
    process = compositor = None
    log_path = output / "runner.log"
    with log_path.open("wb") as log:
        log_path.chmod(0o600)
        try:
            process = subprocess.Popen([str(files["runner"])], cwd=files["runner"].parent, env=env,
                                       stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                                       start_new_session=True)
            state["pid"] = process.pid
            write_json(output / "receipt.json", state)
            compositor = X11Compositor(process.pid)
            deadline = time.monotonic() + args.timeout
            previous = None
            for index in range(1, args.frames + 1):
                stage_path = output / f"stage-{index}.json"
                while not stage_path.exists():
                    if process.poll() is not None or time.monotonic() >= deadline:
                        raise TimeoutError("runner exited or timed out before " + stage_name(index))
                    time.sleep(.025)
                if time.monotonic() >= deadline:
                    raise TimeoutError("Linux qualification deadline expired before stage validation")
                stage = read_json(stage_path)
                width, height, offset = validate_stage(stage, nonce=nonce, index=index, pid=process.pid)
                if previous is not None:
                    if any(stage["stamp"][key] != previous[key] for key in
                           ("documentId", "fontGeneration", "deviceGeneration", "deviceScale")):
                        raise ValueError("unexpected document/font/device change")
                    if stage["stamp"]["viewGeneration"] <= previous["viewGeneration"]:
                        raise ValueError("the new publication did not advance its view")
                    if ((index == 2 and stage["stamp"]["revision"] <= previous["revision"]) or
                            (index > 2 and stage["stamp"]["revision"] != previous["revision"])):
                        raise ValueError("revision/resize publication changed the wrong source state")
                oracle_path = output / f"cpu-{index}.rgba"
                if oracle_path.stat().st_size != width * height * 4:
                    raise ValueError("CPU oracle byte length does not match the viewport")
                expected = oracle_path.read_bytes()
                cpu_hash = hashlib.sha256(expected).hexdigest()
                if index >= 2 and cpu_hash == state["stages"][-1]["cpuSha256"]:
                    raise ValueError("the held publication did not change observable CPU pixels")
                captured = compositor.capture(width, height, offset)
                # Preserve the real pixels even when the strict comparison
                # rejects them, so renderer failures remain independently diagnosable.
                (output / f"compositor-{index}.rgba").write_bytes(captured)
                scale = validate_source_reference(stage, width, height)
                reference_path = output / f"cpu-reference-{index}.rgba"
                reference_bytes = width * height * 4 * scale**2
                if reference_path.stat().st_size != reference_bytes:
                    raise ValueError("independent CPU source reference byte length changed")
                source_reference = reference_path.read_bytes()
                comparison = compare_pixels(expected, captured, width, height,
                                            source_reference=source_reference, reference_scale=scale)
                if time.monotonic() >= deadline:
                    raise TimeoutError("Linux qualification deadline expired during pixel comparison")
                time.sleep(.1)
                if compositor.capture(width, height, offset) != captured:
                    raise ValueError("visible compositor pixels changed during a held publication")
                if time.monotonic() >= deadline:
                    raise TimeoutError("Linux qualification deadline expired during held capture")
                state["stages"].append({"stage": stage, "comparison": comparison, "cpuSha256": cpu_hash,
                                        "sourceReferenceSha256": hashlib.sha256(source_reference).hexdigest(),
                                        "compositorSha256": hashlib.sha256(captured).hexdigest()})
                if index in (1, args.frames):
                    loaded = verify_loaded_artifacts(process.pid, {
                        key: files[key] for key in ("runner", "native", "dispatcher", "plugin", "engineRuntime")})
                    state.setdefault("loadedArtifacts", []).append({"index": index, "modules": loaded})
                write_json(output / "receipt.json", state)
                if time.monotonic() >= deadline:
                    raise TimeoutError("Linux qualification deadline expired before acknowledgement")
                write_json(output / f"ack-{index}.json", {"protocol": 1, "nonce": nonce, "index": index})
                previous = stage["stamp"]
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError("Linux qualification deadline expired before runner exit")
            code = process.wait(timeout=remaining)
            if time.monotonic() >= deadline:
                raise TimeoutError("Linux qualification deadline expired during runner exit")
            result = read_json(output / "result.json")
            if code:
                raise ValueError("runner did not complete its actual close/unregister checks")
            validate_result(result, nonce=nonce, frames=args.frames)
            verify_runtime_artifacts(files, receipt, platform="linux")
            verify_runtime_bundle(files["runner"].parent, receipt.get("runtimeBundle"))
            if time.monotonic() >= deadline:
                raise TimeoutError("Linux qualification deadline expired during final artifact validation")
            state.update(qualification="linux-consumer-passed", exitCode=code, result=result,
                         continuousPresentationSupported=args.frames > 4)
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
