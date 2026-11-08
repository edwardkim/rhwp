#!/usr/bin/env python3
"""Capture a real Windows Flutter compositor; never infer sampling from import.

The executable must use test/support/desktop_consumer_probe.dart. Every run is
one disposable process, with three immutable publications and no frame reuse.
"""

import argparse
import ctypes
from ctypes import wintypes
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import time
import uuid

MAX_PIXELS = 1024 * 1024
STAGES = ("initial", "revision", "resize")
MAX_BUNDLE_FILES = 1024
MAX_BUNDLE_ENTRIES = 4096
MAX_BUNDLE_BYTES = 1024 * 1024 * 1024
MAX_BUILD_RECEIPT_BYTES = 512 * 1024


def digest(path, maximum=None):
    result = hashlib.sha256()
    total = 0
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            total += len(block)
            if maximum is not None and total > maximum:
                raise ValueError("artifact exceeds its byte bound")
            result.update(block)
    return result.hexdigest()


def read_json(path, limit=64 * 1024):
    with Path(path).open("rb") as source:
        data = source.read(limit + 1)
    if len(data) > limit:
        raise ValueError("JSON receipt exceeds its byte bound")
    result = json.loads(data)
    if not isinstance(result, dict):
        raise ValueError("JSON receipt must be an object")
    return result


def write_json(path, value):
    path = Path(path)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    temporary.chmod(0o600)
    temporary.replace(path)


def runtime_bundle_manifest(directory):
    """Hash the exact app directory: Dart kernel/data, DLLs and owned inputs.

    This supports only the helper's debug runner, whose executable starts the
    Dart project from ``data``. No generated or diagnostic file is silently
    excluded. Capture output must be outside this runtime directory.
    """
    directory = Path(directory)
    original = os.stat(directory, follow_symlinks=False)
    if directory.is_symlink() or getattr(original, "st_file_attributes", 0) & 0x400:
        raise ValueError("runtime bundle must not be a symlink or reparse point")
    root = directory.resolve(strict=True)
    if not root.is_dir():
        raise ValueError("runtime bundle must be a directory")
    pending = [root]
    files = {}
    entries = total = 0
    while pending:
        folder = pending.pop()
        with os.scandir(folder) as children:
            for child in children:
                entries += 1
                if entries > MAX_BUNDLE_ENTRIES:
                    raise ValueError("runtime bundle entry bound exceeded")
                metadata = child.stat(follow_symlinks=False)
                if child.is_symlink() or getattr(metadata, "st_file_attributes", 0) & 0x400:
                    raise ValueError("runtime bundle symlink/reparse point is not allowed")
                path = Path(child.path)
                try:
                    path.resolve(strict=True).relative_to(root)
                except ValueError as error:
                    raise ValueError("runtime bundle path escapes its root") from error
                if stat.S_ISDIR(metadata.st_mode):
                    pending.append(path)
                    continue
                if not stat.S_ISREG(metadata.st_mode):
                    raise ValueError("runtime bundle entry is not a regular file")
                if len(files) >= MAX_BUNDLE_FILES:
                    raise ValueError("runtime bundle file bound exceeded")
                remaining = MAX_BUNDLE_BYTES - total
                if metadata.st_size > remaining:
                    raise ValueError("runtime bundle byte bound exceeded")
                relative = path.relative_to(root).as_posix()
                if len(relative.encode("utf-8")) > 1024:
                    raise ValueError("runtime bundle relative path is too long")
                fingerprint = digest(path, maximum=remaining)
                after = os.stat(path, follow_symlinks=False)
                identity = lambda value: (value.st_size, value.st_mtime_ns, value.st_ctime_ns)
                if identity(metadata) != identity(after):
                    raise ValueError("runtime bundle input changed while hashing")
                total += metadata.st_size
                files[relative] = {"sha256": fingerprint, "bytes": metadata.st_size}
    kernel = files.get("data/flutter_assets/kernel_blob.bin")
    if kernel is None or kernel["bytes"] <= 0:
        raise ValueError("debug runtime bundle requires data/flutter_assets/kernel_blob.bin")
    return {"files": dict(sorted(files.items())), "bytes": total}


def verify_runtime_bundle(directory, expected):
    if not isinstance(expected, dict) or not isinstance(expected.get("files"), dict):
        raise ValueError("build receipt has no full runtime bundle manifest")
    if len(expected["files"]) > MAX_BUNDLE_FILES:
        raise ValueError("recorded runtime bundle file bound exceeded")
    for relative in expected["files"]:
        if not isinstance(relative, str) or not relative or "\\" in relative or ":" in relative:
            raise ValueError("invalid runtime bundle relative path")
        parts = relative.split("/")
        if any(part in ("", ".", "..") for part in parts):
            raise ValueError("runtime bundle path escapes or aliases its root")
    actual = runtime_bundle_manifest(directory)
    if actual != expected:
        raise ValueError("runtime bundle files or content differ from the build receipt")
    return actual


def require_actual_windows(*, platform=None, load_library=None):
    """Reject Wine even when its Windows Python reports ``sys.platform=win32``.

    The injectable loader exists for portable rejection controls, not as an
    execution mode for the qualification entry points.
    """
    if (sys.platform if platform is None else platform) != "win32":
        raise RuntimeError("actual Windows desktop is required; Linux/Wine is not qualification")
    loader = load_library if load_library is not None else getattr(ctypes, "WinDLL", None)
    if loader is None:
        raise RuntimeError("actual Windows NT loader is required")
    try:
        ntdll = loader("ntdll", use_last_error=True)
    except (OSError, AttributeError) as error:
        raise RuntimeError("actual Windows NT environment could not be verified") from error
    if getattr(ntdll, "wine_get_version", None) is not None:
        raise RuntimeError("Wine is not an actual Windows qualification target")


def verify_runtime_artifacts(files, receipt, *, platform="windows"):
    """Bind the copied runner to the plugin/engine it will actually load.

    A relocated executable loads its adjacent runtime DLLs. Recorded artifact
    paths are provenance, not authority to inspect DLLs in a different folder.
    """
    verified = {key: Path(value).resolve(strict=True) for key, value in files.items()}
    directory = verified["runner"].parent
    locations = {
        "windows": ("rhwp_native_reference_plugin.dll", "flutter_windows.dll"),
        "linux": ("lib/librhwp_native_reference_plugin.so", "lib/libflutter_linux_gtk.so"),
    }
    if platform not in locations:
        raise ValueError("unsupported runtime bundle platform")
    plugin, engine = locations[platform]
    verified["plugin"] = (directory / plugin).resolve(strict=True)
    verified["engineRuntime"] = (directory / engine).resolve(strict=True)
    artifacts = receipt.get("artifacts")
    if not isinstance(artifacts, dict):
        raise ValueError("build receipt has no runtime artifact identities")
    for key in ("runner", "native", "dispatcher", "fixture", "plugin", "engineRuntime"):
        entry = artifacts.get(key)
        if not isinstance(entry, dict) or digest(verified[key]) != entry.get("sha256"):
            raise ValueError("runtime artifact does not match the build receipt: " + key)
    return verified


def positive_integer(value, maximum):
    if type(value) is not int or not 0 < value <= maximum:
        raise ValueError("expected a bounded positive integer")
    return value


def validate_stage_identity(value, *, nonce, index, pid, platform, name):
    if value.get("protocol") != 1 or value.get("nonce") != nonce:
        raise ValueError("foreign or unsupported probe stage")
    if value.get("stage") != name or value.get("index") != index:
        raise ValueError("out-of-order probe stage")
    if value.get("pid") != pid or value.get("platform") != platform:
        raise ValueError("probe did not come from the launched " + platform + " process")
    width = positive_integer(value.get("width"), 4096)
    height = positive_integer(value.get("height"), 4096)
    if width * height > MAX_PIXELS:
        raise ValueError("probe viewport exceeds its pixel bound")
    dpr = value.get("deviceScale")
    if type(dpr) not in (int, float) or not math.isfinite(dpr) or not 0 < dpr <= 16:
        raise ValueError("invalid device scale")
    offset = value.get("physicalOffset")
    if not isinstance(offset, list) or len(offset) != 2:
        raise ValueError("missing physical client origin")
    for item in offset:
        if type(item) is not int or item < 0 or item > 16384:
            raise ValueError("client origin must be bounded and pixel aligned")
    stamp = value.get("stamp")
    if not isinstance(stamp, dict):
        raise ValueError("missing full source stamp")
    for key in ("documentId", "viewGeneration", "deviceGeneration"):
        positive_integer(stamp.get(key), 2**63 - 1)
    for key in ("revision", "fontGeneration"):
        if type(stamp.get(key)) is not int or not 0 <= stamp[key] < 2**63:
            raise ValueError("invalid source generation")
    if stamp.get("deviceScale") != dpr:
        raise ValueError("DPR and full source stamp disagree")
    return width, height, tuple(offset)


def validate_stage(value, *, nonce, index, pid):
    width, height, offset = validate_stage_identity(
        value, nonce=nonce, index=index, pid=pid, platform="windows", name=STAGES[index - 1])
    limits = value.get("limits")
    if not isinstance(limits, dict):
        raise ValueError("missing immutable frame budget receipt")
    if limits.get("continuousPresentationSupported") is not False:
        raise ValueError("this probe does not qualify a new continuous backend")
    if limits.get("consumerRetirement") is not False:
        raise ValueError("unqualified consumer completion contract")
    if limits.get("maxFrames") != 4 or limits.get("maxTargetBytes") != 64 * 1024 * 1024:
        raise ValueError("native quarantine budget changed")
    if limits.get("descriptorReleaseIsGpuComplete") is not False:
        raise ValueError("descriptor open must not be treated as GPU completion")
    if type(limits.get("outstandingFrames")) is not int or not 0 < limits["outstandingFrames"] <= 4:
        raise ValueError("unexpected native frame credit use")
    return width, height, tuple(offset)


def validate_source_reference(stage, width, height):
    scale = stage.get("cpuReferenceScale")
    if (type(scale) is not int or not 2 <= scale <= 4 or width * scale > 4096 or
            height * scale > 4096 or width * height * scale**2 > 4 * 1024 * 1024):
        raise ValueError("referenceUnavailable: independent CPU geometry exceeds its 16 MiB bound")
    proof = stage.get("geometryProof")
    if not isinstance(proof, dict):
        raise ValueError("missing independently replayed CPU source geometry")
    low, high = proof.get("cpu"), proof.get("reference")
    if not isinstance(low, dict) or not isinstance(high, dict):
        raise ValueError("missing CPU/reference source receipts")
    snapshot = low.get("snapshot")
    if (type(snapshot) is not int or not 0 < snapshot < 2**63 or
            type(high.get("snapshot")) is not int or high["snapshot"] != snapshot):
        raise ValueError("CPU references must use the same immutable snapshot")
    for item, factor in ((low, 1), (high, scale)):
        if item.get("stamp") != stage["stamp"] or item.get("coordinateSpace") != "viewportPixels":
            raise ValueError("CPU reference changed its full source stamp/coordinate space")
        if item.get("viewport") != {"width": width * factor, "height": height * factor}:
            raise ValueError("CPU reference target geometry was not scaled independently")
        descriptor = item.get("descriptor", {})
        for key, expected in {"width": width * factor, "height": height * factor,
                              "stride": width * factor * 4, "pixelFormat": "rgba8Premultiplied",
                              "colorSpace": "srgb", "origin": "topLeft"}.items():
            if descriptor.get(key) != expected:
                raise ValueError("CPU reference descriptor disagrees with its source geometry")
        placement = item.get("placement", {})
        for key in ("scale", "originX", "originY"):
            number = placement.get(key)
            if type(number) not in (int, float) or not math.isfinite(number):
                raise ValueError("CPU reference placement must be finite")
            if ((key == "scale" and not 0 < number <= 64) or
                    (key != "scale" and abs(number) > 16777216)):
                raise ValueError("CPU reference placement exceeds Native geometry bounds")
    for key in ("scale", "originX", "originY"):
        if not math.isclose(high["placement"][key], low["placement"][key] * scale,
                            rel_tol=1e-12, abs_tol=1e-9):
            raise ValueError("CPU reference placement was not scaled from the same source")
    return scale


def validate_result(value, *, nonce, last_limits):
    if value.get("protocol") != 1 or value.get("nonce") != nonce or value.get("ok") is not True:
        raise ValueError("foreign or failed probe completion")
    if value.get("consumerGpuCompletionVerified") is not False or value.get("continuousPresentationSupported") is not False:
        raise ValueError("bounded sampling is not consumer GPU completion")
    stats = value.get("stats", {})
    for key in ("sources", "snapshotHandles", "retainedSnapshots", "snapshotEstimatedBytes",
                "frameHandles", "retainedFrames", "frameBytes", "leases", "inFlightReplays"):
        if type(stats.get(key)) is not int or stats[key] != 0:
            raise ValueError("native teardown was not observed: " + key)
    closed = value.get("closed", {})
    if closed.get("ownerFinished") is not True or not (
            closed.get("unregistered") is True or closed.get("registered") is False):
        raise ValueError("producer completion and actual unregistration are required")
    limits = closed.get("limits", {})
    if limits.get("continuousPresentationSupported") is not False or limits.get("consumerRetirement") is not False:
        raise ValueError("close must not fabricate consumer completion")
    if limits.get("outstandingFrames") != last_limits["outstandingFrames"]:
        raise ValueError("unregistration must not refund immutable GPU frame credits")


_PALETTE_AXES = tuple(axis for axis in itertools.product((-1, 0, 1), repeat=3)
                      if any(axis) and next(value for value in axis if value) > 0)


def _source_palette_support(reference, width, height, scale, x, y, sample):
    # A one-low-pixel geometric neighborhood of an independently replayed CPU
    # source. Fixed RGB directions retain channel correlations, unlike a box
    # which could admit a hue absent from the source's local color mixtures.
    colors = {tuple(reference[(yy * width * scale + xx) * 4:
                              (yy * width * scale + xx) * 4 + 3])
              for yy in range(max(0, y - 1) * scale, min(height, y + 2) * scale)
              for xx in range(max(0, x - 1) * scale, min(width, x + 2) * scale)}
    for axis in _PALETTE_AXES:
        projected = [sum(color[channel] * axis[channel] for channel in range(3)) for color in colors]
        value = sum(sample[channel] * axis[channel] for channel in range(3))
        # One unit per channel is the existing 8-bit quantization allowance.
        margin = sum(abs(component) for component in axis)
        if not min(projected) - margin <= value <= max(projected) + margin:
            return False
    return True


def compare_pixels(expected, captured, width, height, *, source_reference=None, reference_scale=None):
    """Opaque uniform interiors <=1; differences stay in CPU AA support.

    The CPU oracle is a separate strict replay of the exact published snapshot,
    viewport and clear color. Its edge colors need not equal a GPU AA sample.
    """
    if len(expected) != width * height * 4 or len(captured) != len(expected):
        raise ValueError("CPU/compositor pixel lengths do not match")
    if width * height > MAX_PIXELS or width <= 0 or height <= 0:
        raise ValueError("invalid comparison dimensions")
    if source_reference is not None:
        if (type(reference_scale) is not int or not 2 <= reference_scale <= 4 or
                width * reference_scale > 4096 or height * reference_scale > 4096 or
                width * height * reference_scale**2 > 4 * 1024 * 1024 or
                len(source_reference) != width * height * 4 * reference_scale**2):
            raise ValueError("referenceUnavailable: independent CPU source exceeds its geometry/16 MiB bound")
        if any(alpha != 255 for alpha in source_reference[3::4]):
            raise ValueError("independent source reference must already be composited opaque")
    colors = set()
    dark = 0
    for index in range(width * height):
        a = expected[index * 4:index * 4 + 4]
        b = captured[index * 4:index * 4 + 4]
        if a[3] != 255 or b[3] != 255:
            raise ValueError("desktop oracle must already be composited opaque")
        colors.add(a[:3])
        dark += max(a[:3]) < 220
    if len(colors) < 3 or dark < 32:
        raise ValueError("CPU oracle lacks independently observable content")
    uniform = 0
    edge_differences = 0
    maximum = 0
    edges = []
    for index in range(width * height):
        x, y = index % width, index // width
        color = expected[index * 4:index * 4 + 4]
        sample = captured[index * 4:index * 4 + 4]
        difference = max(abs(a - b) for a, b in zip(color, sample))
        maximum = max(maximum, difference)
        neighbors = [expected[(yy * width + xx) * 4:(yy * width + xx) * 4 + 4]
                     for yy in range(max(0, y - 1), min(height, y + 2))
                     for xx in range(max(0, x - 1), min(width, x + 2))]
        if all(item == color for item in neighbors):
            uniform += 1
            if difference > 1:
                raise ValueError(f"compositor differs in uniform CPU interior ({x},{y})")
        else:
            if source_reference is not None and 0 < x < width - 1 and 0 < y < height - 1:
                edges.append((x, y))
            if difference > 1:
                edge_differences += 1
                if source_reference is None:
                    # Preserve the original contract for legacy three-frame
                    # collectors which have no independent high-res source.
                    supported = all(min(item[channel] for item in neighbors) - 1 <= sample[channel]
                                    <= max(item[channel] for item in neighbors) + 1
                                    for channel in range(3))
                else:
                    supported = _source_palette_support(source_reference, width, height,
                                                        reference_scale, x, y, sample)
                if not supported:
                    raise ValueError(f"compositor color escaped the CPU AA envelope ({x},{y}): "
                                     f"CPU {tuple(color)}, display {tuple(sample)}")
    if uniform < width * height // 4:
        raise ValueError("oracle has too few independent uniform interior controls")
    result = {"uniformInteriorPixels": uniform, "aaEdgeDifferentPixels": edge_differences,
              "maxChannelDifference": maximum, "comparison": "opaque-interiors-1-aa-support-1px"}
    if source_reference is not None:
        errors = {}
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                errors[f"{dx},{dy}"] = sum(
                    sum(abs(expected[(y * width + x) * 4 + channel] -
                            captured[((y + dy) * width + x + dx) * 4 + channel]) for channel in range(3))
                    for x, y in edges)
        current = errors["0,0"]
        if any(value <= current for shift, value in errors.items() if shift != "0,0"):
            raise ValueError("CPU source/compositor integer alignment is shifted or ambiguous")
        result.update(comparison="opaque-interiors-1-source-palette13-edge1px-zero-integer-shift",
                      sourceReferenceScale=reference_scale, sourceReferenceBytes=len(source_reference),
                      integerAlignmentErrors=errors)
    return result


class WindowsCompositor:
    """DPI-aware visible desktop capture, not producer texture readback."""

    def __init__(self, pid):
        require_actual_windows()
        self.user = ctypes.WinDLL("user32", use_last_error=True)
        self.gdi = ctypes.WinDLL("gdi32", use_last_error=True)
        handle = wintypes.HANDLE
        self.user.GetDC.argtypes = [wintypes.HWND]
        self.user.GetDC.restype = handle
        self.user.ReleaseDC.argtypes = [wintypes.HWND, handle]
        self.user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
        self.user.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
        self.user.ClientToScreen.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.POINT)]
        self.user.IsWindowVisible.argtypes = [wintypes.HWND]
        self.user.GetForegroundWindow.restype = wintypes.HWND
        self.user.SetForegroundWindow.argtypes = [wintypes.HWND]
        self.gdi.CreateCompatibleDC.argtypes = [handle]
        self.gdi.CreateCompatibleDC.restype = handle
        self.gdi.CreateDIBSection.argtypes = [handle, ctypes.c_void_p, wintypes.UINT,
                                            ctypes.POINTER(ctypes.c_void_p), handle, wintypes.DWORD]
        self.gdi.CreateDIBSection.restype = handle
        self.gdi.SelectObject.argtypes = [handle, handle]
        self.gdi.SelectObject.restype = handle
        self.gdi.DeleteObject.argtypes = [handle]
        self.gdi.DeleteDC.argtypes = [handle]
        self.gdi.BitBlt.argtypes = [handle, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int,
                                    handle, ctypes.c_int, ctypes.c_int, wintypes.DWORD]
        self.user.SetProcessDpiAwarenessContext.argtypes = [handle]
        # If the caller is already per-monitor aware, the setter may return 0.
        self.user.SetProcessDpiAwarenessContext(handle(-4))
        self.user.SetThreadDpiAwarenessContext.argtypes = [handle]
        self.user.SetThreadDpiAwarenessContext.restype = handle
        if not self.user.SetThreadDpiAwarenessContext(handle(-4)):
            raise OSError("per-monitor-v2 physical capture coordinates are required")
        self.pid = pid

    def window(self):
        windows = []
        callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

        @callback_type
        def visit(window, _):
            owner = wintypes.DWORD()
            self.user.GetWindowThreadProcessId(window, ctypes.byref(owner))
            if owner.value == self.pid and self.user.IsWindowVisible(window):
                windows.append(window)
            return True

        self.user.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
        self.user.EnumWindows(visit, 0)
        if len(windows) != 1:
            raise ValueError("expected exactly one visible window from the launched probe")
        self.user.SetForegroundWindow(windows[0])
        return windows[0]

    def capture(self, width, height, offset):
        window = self.window()
        if self.user.GetForegroundWindow() != window:
            raise ValueError("probe window must be foreground and unoccluded")
        rect, point = wintypes.RECT(), wintypes.POINT()
        if not self.user.GetClientRect(window, ctypes.byref(rect)) or not self.user.ClientToScreen(window, ctypes.byref(point)):
            raise OSError("could not locate probe client pixels")
        if offset[0] + width > rect.right or offset[1] + height > rect.bottom:
            raise ValueError("probe rectangle exceeds the actual client area")

        class Header(ctypes.Structure):
            _fields_ = [("size", wintypes.DWORD), ("width", ctypes.c_int32), ("height", ctypes.c_int32),
                        ("planes", wintypes.WORD), ("bits", wintypes.WORD), ("compression", wintypes.DWORD),
                        ("bytes", wintypes.DWORD), ("xppm", ctypes.c_int32), ("yppm", ctypes.c_int32),
                        ("colors", wintypes.DWORD), ("important", wintypes.DWORD)]

        header = Header(ctypes.sizeof(Header), width, -height, 1, 32, 0, width * height * 4, 0, 0, 0, 0)
        screen = self.user.GetDC(None)
        memory = self.gdi.CreateCompatibleDC(screen)
        bits = ctypes.c_void_p()
        bitmap = self.gdi.CreateDIBSection(screen, ctypes.byref(header), 0, ctypes.byref(bits), None, 0)
        previous = None
        try:
            if not screen or not memory or not bitmap or not bits.value:
                raise OSError("desktop capture allocation failed")
            previous = self.gdi.SelectObject(memory, bitmap)
            if not self.gdi.BitBlt(memory, 0, 0, width, height, screen,
                                   point.x + offset[0], point.y + offset[1], 0x40CC0020):
                raise OSError("visible desktop capture failed")
            bgra = ctypes.string_at(bits.value, width * height * 4)
            rgba = bytearray(len(bgra))
            rgba[0::4], rgba[1::4], rgba[2::4], rgba[3::4] = bgra[2::4], bgra[1::4], bgra[0::4], bytes([255]) * (width * height)
            return bytes(rgba)
        finally:
            if previous:
                self.gdi.SelectObject(memory, previous)
            if bitmap:
                self.gdi.DeleteObject(bitmap)
            if memory:
                self.gdi.DeleteDC(memory)
            if screen:
                self.user.ReleaseDC(None, screen)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", type=Path, required=True)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--dispatcher", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--build-receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--cpu-reference", action="store_true",
                        help="require an independent same-snapshot supersampled CPU source reference")
    args = parser.parse_args()
    try:
        require_actual_windows()
    except RuntimeError as error:
        parser.error(str(error))
    if not 10 <= args.timeout <= 300:
        parser.error("timeout must be between 10 and 300 seconds")
    files = {key: getattr(args, key).resolve(strict=True) for key in ("runner", "native", "dispatcher", "fixture")}
    if files["fixture"].stat().st_size > 16 * 1024 * 1024:
        raise ValueError("reference input exceeds its document byte limit")
    receipt = read_json(args.build_receipt, limit=MAX_BUILD_RECEIPT_BYTES)
    if receipt.get("platform") != "windows" or receipt.get("buildExitCode") != 0:
        raise ValueError("matching successful Windows build receipt is required")
    files = verify_runtime_artifacts(files, receipt)
    verify_runtime_bundle(files["runner"].parent, receipt.get("runtimeBundle"))
    for key in ("sourceHead", "nativeSourceHead", "flutterRevision", "engineRevision"):
        if not re.fullmatch(r"[0-9a-f]{40}", str(receipt.get(key, ""))):
            raise ValueError("build receipt has no pinned source/engine identity: " + key)
    output = args.output.resolve()
    try:
        output.relative_to(files["runner"].parent)
    except ValueError:
        pass
    else:
        raise ValueError("capture output must be outside the runtime bundle")
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    nonce = uuid.uuid4().hex
    config = {"protocol": 1, "nonce": nonce, "output": str(output),
              **{key: str(value) for key, value in files.items()}}
    if args.cpu_reference:
        config["cpuReference"] = True
    write_json(output / "config.json", config)
    env = dict(os.environ, RHWP_CONSUMER_PROBE_CONFIG=str(output / "config.json"))
    state = {"platform": "windows", "qualification": "pending", "build": receipt,
             "stages": [], "consumerGpuCompletionVerified": False,
             "continuousPresentationSupported": False, "hardwareAccelerationVerified": False,
             "started": time.time()}
    if args.cpu_reference:
        state["cpuReferenceRequested"] = True
    process = None
    log = (output / "runner.log").open("wb")
    try:
        process = subprocess.Popen([str(files["runner"])], cwd=files["runner"].parent, env=env,
                                   stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
        state["pid"] = process.pid
        write_json(output / "receipt.json", state)
        compositor = WindowsCompositor(process.pid)
        deadline = time.monotonic() + args.timeout
        previous_stamp = None
        for index, name in enumerate(STAGES, 1):
            stage_file = output / f"stage-{index}.json"
            while not stage_file.exists():
                if process.poll() is not None or time.monotonic() >= deadline:
                    raise TimeoutError("probe exited or timed out before " + name)
                time.sleep(0.025)
            if time.monotonic() >= deadline:
                raise TimeoutError("Windows capture deadline expired before a ready stage")
            stage = read_json(stage_file)
            width, height, offset = validate_stage(stage, nonce=nonce, index=index, pid=process.pid)
            if previous_stamp is not None:
                if any(stage["stamp"][key] != previous_stamp[key] for key in
                       ("documentId", "fontGeneration", "deviceGeneration", "deviceScale")):
                    raise ValueError("unplanned document/font/device identity change")
                if index == 2 and stage["stamp"]["revision"] <= previous_stamp["revision"]:
                    raise ValueError("revision stage did not mutate the source")
                if index == 3 and stage["stamp"]["viewGeneration"] <= previous_stamp["viewGeneration"]:
                    raise ValueError("resize stage did not change the view generation")
                if index == 3 and stage["stamp"]["revision"] != previous_stamp["revision"]:
                    raise ValueError("resize must preserve the source revision")
            oracle_path = output / f"cpu-{index}.rgba"
            if oracle_path.stat().st_size != width * height * 4:
                raise ValueError("oracle byte length differs from the bounded viewport")
            cpu_hash = digest(oracle_path)
            if index == 2 and cpu_hash == state["stages"][-1]["cpuSha256"]:
                raise ValueError("revision did not produce an independently observable pixel change")
            with oracle_path.open("rb") as oracle:
                expected = oracle.read(width * height * 4 + 1)
            captured = compositor.capture(width, height, offset)
            # Keep failed real captures too. Reference validation and the pixel
            # oracle must never discard the evidence they are rejecting.
            captured_path = output / f"compositor-{index}.rgba"
            captured_path.write_bytes(captured)
            captured_path.chmod(0o600)
            comparison_arguments = {}
            source_hash = None
            if args.cpu_reference:
                scale = validate_source_reference(stage, width, height)
                reference_path = output / f"cpu-reference-{index}.rgba"
                reference_bytes = width * height * 4 * scale**2
                if reference_path.stat().st_size != reference_bytes:
                    raise ValueError("independent CPU source reference byte length changed")
                with reference_path.open("rb") as reference:
                    source_reference = reference.read(reference_bytes + 1)
                if len(source_reference) != reference_bytes:
                    raise ValueError("independent CPU source reference byte length changed while reading")
                source_hash = hashlib.sha256(source_reference).hexdigest()
                comparison_arguments = {"source_reference": source_reference, "reference_scale": scale}
            comparison = compare_pixels(expected, captured, width, height, **comparison_arguments)
            if time.monotonic() >= deadline:
                raise TimeoutError("Windows capture deadline expired during CPU comparison")
            time.sleep(0.1)
            repeated = compositor.capture(width, height, offset)
            if repeated != captured:
                raise ValueError("visible compositor pixels changed during held publication")
            if time.monotonic() >= deadline:
                raise TimeoutError("Windows capture deadline expired during held publication")
            evidence = {"stage": stage, "comparison": comparison, "cpuSha256": cpu_hash,
                        "compositorSha256": hashlib.sha256(captured).hexdigest()}
            if source_hash is not None:
                evidence["sourceReferenceSha256"] = source_hash
            state["stages"].append(evidence)
            write_json(output / "receipt.json", state)
            if time.monotonic() >= deadline:
                raise TimeoutError("Windows capture deadline expired before acknowledgement")
            write_json(output / f"ack-{index}.json", {"protocol": 1, "nonce": nonce, "index": index})
            previous_stamp = stage["stamp"]
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("Windows capture deadline expired before close completion")
        code = process.wait(timeout=remaining)
        if time.monotonic() >= deadline:
            raise TimeoutError("Windows capture deadline expired during close completion")
        result = read_json(output / "result.json")
        if code:
            raise ValueError("probe did not complete its real close/unregister checks")
        validate_result(result, nonce=nonce, last_limits=state["stages"][-1]["stage"]["limits"])
        verify_runtime_artifacts(files, receipt)
        verify_runtime_bundle(files["runner"].parent, receipt.get("runtimeBundle"))
        if time.monotonic() >= deadline:
            raise TimeoutError("Windows capture deadline expired during final runtime verification")
        state.update(qualification="bounded-windows-consumer-passed", exitCode=code, result=result)
    except BaseException as error:
        state.update(qualification="failed", error=f"{type(error).__name__}: {error}")
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=10)
        raise
    finally:
        log.close()
        state["finished"] = time.time()
        write_json(output / "receipt.json", state)
    print(output / "receipt.json")


if __name__ == "__main__":
    main()
