#!/usr/bin/env python3
"""Portable negative controls for the actual Windows qualification collector.

These tests prove receipt/capture rejection behavior, never Windows execution.
"""

import importlib.util
import io
from contextlib import contextmanager
from pathlib import Path
import sys
import shutil
from types import SimpleNamespace
from unittest import mock
import tempfile
import unittest

TOOL = Path(__file__).resolve().parents[1] / "tool" / "qualify_desktop_texture.py"
spec = importlib.util.spec_from_file_location("qualification", TOOL)
qualification = importlib.util.module_from_spec(spec)
spec.loader.exec_module(qualification)


# Build helper imports the same collector utilities by their module filename.
sys.path.insert(0, str(TOOL.parent))
build_spec = importlib.util.spec_from_file_location("desktop_build", TOOL.parent / "build_desktop_probe.py")
build = importlib.util.module_from_spec(build_spec)
build_spec.loader.exec_module(build)



def stage():
    return {"protocol": 1, "nonce": "a" * 32, "pid": 42, "platform": "windows",
            "index": 1, "stage": "initial", "width": 40, "height": 30,
            "deviceScale": 1.25, "physicalOffset": [16, 16],
            "stamp": {"documentId": 1, "revision": 2, "fontGeneration": 0,
                      "viewGeneration": 3, "deviceGeneration": 1, "deviceScale": 1.25},
            "limits": {"continuousPresentationSupported": False, "consumerRetirement": False,
                       "maxFrames": 4, "maxTargetBytes": 64 * 1024 * 1024,
                       "descriptorReleaseIsGpuComplete": False, "outstandingFrames": 1}}


def completion():
    return {"protocol": 1, "nonce": "a" * 32, "ok": True,
            "consumerGpuCompletionVerified": False, "continuousPresentationSupported": False,
            "stats": {key: 0 for key in ("sources", "snapshotHandles", "retainedSnapshots",
                      "snapshotEstimatedBytes", "frameHandles", "retainedFrames", "frameBytes",
                      "leases", "inFlightReplays")},
            "closed": {"ownerFinished": True, "unregistered": True, "limits": stage()["limits"]}}


def image(width=40, height=30):
    return b"".join(bytes((shade, shade, shade, 255))
                    for y in range(height) for x in range(width)
                    for shade in [0 if x < 14 else 128 if x < 27 else 255])


class CollectorContracts(unittest.TestCase):
    def validate(self, value):
        return qualification.validate_stage(value, nonce="a" * 32, index=1, pid=42)

    def test_matching_full_stamp_and_physical_rectangle_are_admitted(self):
        self.assertEqual(self.validate(stage()), (40, 30, (16, 16)))

    def test_foreign_process_nonce_or_platform_is_rejected(self):
        for key, value in [("nonce", "b" * 32), ("pid", 43), ("platform", "linux")]:
            with self.subTest(key=key):
                candidate = stage()
                candidate[key] = value
                with self.assertRaises(ValueError):
                    self.validate(candidate)

    def test_out_of_order_stage_is_rejected(self):
        candidate = stage()
        candidate["stage"] = "resize"
        with self.assertRaises(ValueError):
            self.validate(candidate)

    def test_nonfinite_boolean_or_excess_dimensions_are_rejected(self):
        for key, value in [("width", True), ("width", 0), ("height", 4097),
                           ("deviceScale", float("nan")), ("deviceScale", True)]:
            with self.subTest(key=key, value=value):
                candidate = stage()
                candidate[key] = value
                with self.assertRaises(ValueError):
                    self.validate(candidate)

    def test_fractional_or_negative_client_origin_is_rejected(self):
        for value in [[.5, 0], [-1, 16], [16], [False, 16]]:
            candidate = stage()
            candidate["physicalOffset"] = value
            with self.assertRaises(ValueError):
                self.validate(candidate)

    def test_stamp_generation_and_dpr_are_authoritative(self):
        for key, value in [("revision", -1), ("documentId", 0), ("viewGeneration", True),
                           ("deviceScale", 1)]:
            candidate = stage()
            candidate["stamp"][key] = value
            with self.assertRaises(ValueError):
                self.validate(candidate)

    def test_callbacks_cannot_be_reported_as_consumer_gpu_completion(self):
        for key in ("continuousPresentationSupported", "consumerRetirement", "descriptorReleaseIsGpuComplete"):
            candidate = stage()
            candidate["limits"][key] = True
            with self.assertRaises(ValueError):
                self.validate(candidate)

    def test_native_budget_cannot_be_silently_expanded(self):
        for key, value in [("maxFrames", 5), ("maxTargetBytes", 128 * 1024 * 1024),
                           ("outstandingFrames", 0), ("outstandingFrames", 5)]:
            candidate = stage()
            candidate["limits"][key] = value
            with self.assertRaises(ValueError):
                self.validate(candidate)

    def test_exact_compositor_pixels_pass_the_independent_oracle(self):
        result = qualification.compare_pixels(image(), image(), 40, 30)
        self.assertGreater(result["uniformInteriorPixels"], 1000)
        self.assertEqual(result["maxChannelDifference"], 0)

    def test_one_encoded_unit_is_allowed_in_uniform_interiors(self):
        pixels = bytearray(image())
        pixels[(10 * 40 + 20) * 4] += 1
        result = qualification.compare_pixels(image(), pixels, 40, 30)
        self.assertEqual(result["maxChannelDifference"], 1)

    def test_uniform_color_error_is_a_failed_pixel_qualification(self):
        pixels = bytearray(image())
        pixels[(10 * 40 + 20) * 4] += 2
        with self.assertRaisesRegex(ValueError, "uniform CPU interior"):
            qualification.compare_pixels(image(), pixels, 40, 30)

    def test_stale_shift_and_cover_do_not_pass_as_gpu_sampling(self):
        for captured in [image()[16:] + image()[:16], bytes([85, 85, 85, 255]) * 1200]:
            with self.assertRaises(ValueError):
                qualification.compare_pixels(image(), captured, 40, 30)

    def test_aa_color_outside_the_local_cpu_envelope_is_rejected(self):
        pixels = bytearray(image())
        pixels[(10 * 40 + 13) * 4] = 255
        with self.assertRaisesRegex(ValueError, "AA envelope"):
            qualification.compare_pixels(image(), pixels, 40, 30)

    def test_empty_transparent_or_truncated_oracle_is_rejected(self):
        for expected in [b"", bytes([255]) * 4800, bytes([0]) * 4800, image()[:-1]]:
            with self.assertRaises(ValueError):
                qualification.compare_pixels(expected, image(), 40, 30)

    def test_normal_close_does_not_refund_immutable_frame_credit(self):
        qualification.validate_result(completion(), nonce="a" * 32, last_limits=stage()["limits"])
        value = completion()
        value["closed"]["limits"]["outstandingFrames"] = 0
        with self.assertRaisesRegex(ValueError, "refund"):
            qualification.validate_result(value, nonce="a" * 32, last_limits=stage()["limits"])

    def test_native_leak_or_missing_unregister_is_not_a_success(self):
        for modify in [lambda x: x["stats"].update(leases=1),
                       lambda x: x["closed"].update(ownerFinished=False),
                       lambda x: x["closed"].update(unregistered=False),
                       lambda x: x.update(consumerGpuCompletionVerified=True)]:
            value = completion()
            modify(value)
            with self.assertRaises(ValueError):
                qualification.validate_result(value, nonce="a" * 32, last_limits=stage()["limits"])

    def test_json_read_is_bounded_and_writes_are_atomic(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "receipt.json"
            qualification.write_json(path, {"ok": True})
            self.assertEqual(qualification.read_json(path), {"ok": True})
            self.assertFalse(path.with_suffix(".json.tmp").exists())
            path.write_bytes(b"x" * (65536 + 1))
            with self.assertRaises(ValueError):
                qualification.read_json(path)

    @unittest.skipIf(sys.platform == "win32", "this is the unavailable-platform negative control")
    def test_linux_cannot_be_reported_as_actual_windows(self):
        with self.assertRaisesRegex(RuntimeError, "actual Windows"):
            qualification.WindowsCompositor(42)


class BuildAndRuntimeIdentityContracts(unittest.TestCase):
    def artifacts(self, root):
        names = {"runner": "probe.exe", "plugin": "rhwp_native_reference_plugin.dll",
                 "engineRuntime": "flutter_windows.dll", "native": "rhwp_native_ffi.dll",
                 "dispatcher": "rhwp_flutter_owner.dll", "fixture": "input.hwpx"}
        paths = {}
        for key, name in names.items():
            path = root / name
            path.write_bytes(("portable-identity-control-" + key).encode())
            paths[key] = path
        receipt = {"artifacts": {key: {"path": str(path), "sha256": qualification.digest(path)}
                                 for key, path in paths.items()}}
        supplied = {key: paths[key] for key in ("runner", "native", "dispatcher", "fixture")}
        return paths, supplied, receipt

    def test_matching_adjacent_plugin_and_engine_are_verified(self):
        with tempfile.TemporaryDirectory() as temporary:
            paths, supplied, receipt = self.artifacts(Path(temporary))
            actual = qualification.verify_runtime_artifacts(supplied, receipt)
            self.assertEqual(actual["plugin"], paths["plugin"])
            self.assertEqual(actual["engineRuntime"], paths["engineRuntime"])

    def test_changed_loaded_plugin_or_engine_is_rejected(self):
        for key in ("plugin", "engineRuntime"):
            with self.subTest(key=key), tempfile.TemporaryDirectory() as temporary:
                paths, supplied, receipt = self.artifacts(Path(temporary))
                paths[key].write_bytes(b"different runtime DLL")
                with self.assertRaisesRegex(ValueError, key):
                    qualification.verify_runtime_artifacts(supplied, receipt)

    def test_relocated_runner_checks_its_actual_adjacent_dlls(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = root / "original"
            original.mkdir()
            paths, supplied, receipt = self.artifacts(original)
            relocated = root / "relocated"
            relocated.mkdir()
            runner = relocated / paths["runner"].name
            shutil.copyfile(paths["runner"], runner)
            supplied["runner"] = runner
            # A valid recorded original plugin cannot stand in for the DLLs
            # loaded beside a relocated executable.
            with self.assertRaises(FileNotFoundError):
                qualification.verify_runtime_artifacts(supplied, receipt)
            for key in ("plugin", "engineRuntime"):
                shutil.copyfile(paths[key], relocated / paths[key].name)
            qualification.verify_runtime_artifacts(supplied, receipt)
            (relocated / paths["plugin"].name).write_bytes(b"stale adjacent plugin")
            with self.assertRaisesRegex(ValueError, "plugin"):
                qualification.verify_runtime_artifacts(supplied, receipt)

    def test_wine_windows_python_is_rejected_before_any_gui_capture(self):
        class Wine:
            wine_get_version = object()
        with self.assertRaisesRegex(RuntimeError, "Wine"):
            qualification.require_actual_windows(platform="win32", load_library=lambda *a, **k: Wine())

    def test_real_nt_loader_without_wine_marker_is_admitted_by_platform_guard(self):
        qualification.require_actual_windows(platform="win32", load_library=lambda *a, **k: object())

    def test_unverifiable_nt_environment_fails_closed(self):
        def missing(*args, **kwargs):
            raise OSError("ntdll missing")
        with self.assertRaisesRegex(RuntimeError, "could not be verified"):
            qualification.require_actual_windows(platform="win32", load_library=missing)
        with self.assertRaisesRegex(RuntimeError, "actual Windows"):
            qualification.require_actual_windows(platform="linux", load_library=missing)

    def test_all_copied_package_build_inputs_are_rechecked(self):
        paths = ["bindings/Flutter/lib/src/native_texture_view.dart",
                 "bindings/Flutter/src/owner_dispatcher.c",
                 "bindings/Flutter/windows/rhwp_d3d_plugin.cpp",
                 "bindings/Flutter/windows/FrameLifetime.h",
                 "bindings/Flutter/windows/CMakeLists.txt",
                 "bindings/swift/Sources/CRhwpNative/rhwp_native_ffi.h"]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for relative in paths:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode())
            expected = {relative: qualification.digest(root / relative) for relative in paths}
            self.assertEqual(build.verify_copied_sources(root, expected), expected)
            # Generated runner inputs legitimately differ from package source.
            generated = root / "runner/windows/flutter/generated_plugins.cmake"
            generated.parent.mkdir(parents=True)
            generated.write_text("generated runner data")
            build.verify_copied_sources(root, expected)
            for relative in paths:
                with self.subTest(relative=relative):
                    path = root / relative
                    previous = path.read_bytes()
                    path.write_bytes(b"mutated package input")
                    with self.assertRaisesRegex(ValueError, "copied tracked input"):
                        build.verify_copied_sources(root, expected)
                    path.write_bytes(previous)

    def test_missing_copied_package_input_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(FileNotFoundError):
                build.verify_copied_sources(Path(temporary), {"bindings/Flutter/windows/NativeApi.h": "0" * 64})


class RuntimeBundleContracts(unittest.TestCase):
    def bundle(self, root):
        # The real debug runner starts DartProject("data"). These portable bytes
        # exercise identity rejection and are never executable Windows evidence.
        files = {"probe.exe": b"portable runner", "flutter_windows.dll": b"portable engine",
                 "rhwp_native_reference_plugin.dll": b"portable plugin",
                 "data/icudtl.dat": b"portable ICU",
                 "data/flutter_assets/kernel_blob.bin": bytes(range(256)) * 64,
                 "data/flutter_assets/AssetManifest.bin": b"portable asset manifest"}
        for relative, content in files.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        return files

    def test_dart_program_and_data_match_after_complete_bundle_relocation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = root / "original"
            original.mkdir()
            files = self.bundle(original)
            expected = qualification.runtime_bundle_manifest(original)
            self.assertEqual(expected["files"]["data/flutter_assets/kernel_blob.bin"]["bytes"], 16384)
            relocated = root / "relocated"
            shutil.copytree(original, relocated)
            self.assertEqual(qualification.verify_runtime_bundle(relocated, expected), expected)
            self.assertEqual(set(expected["files"]), set(files))

    def test_changed_missing_added_runtime_files_are_rejected(self):
        for operation in ("changed", "missing", "added"):
            with self.subTest(operation=operation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                self.bundle(root)
                expected = qualification.runtime_bundle_manifest(root)
                if operation == "changed":
                    (root / "data/flutter_assets/kernel_blob.bin").write_bytes(b"different Dart program")
                elif operation == "missing":
                    (root / "data/icudtl.dat").unlink()
                else:
                    (root / "unexpected.dll").write_bytes(b"unrecorded app dependency")
                with self.assertRaisesRegex(ValueError, "runtime bundle files or content"):
                    qualification.verify_runtime_bundle(root, expected)

    def test_non_debug_or_missing_kernel_is_not_qualified(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.bundle(root)
            (root / "data/flutter_assets/kernel_blob.bin").unlink()
            with self.assertRaisesRegex(ValueError, "kernel_blob"):
                qualification.runtime_bundle_manifest(root)

    def test_relative_path_escape_and_alias_in_receipt_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.bundle(root)
            for bad in ("../outside", "/absolute", "C:/absolute", "data/../kernel", "data//alias", "data\\alias"):
                expected = qualification.runtime_bundle_manifest(root)
                expected["files"][bad] = {"sha256": "0" * 64, "bytes": 0}
                with self.subTest(path=bad), self.assertRaisesRegex(ValueError, "path"):
                    qualification.verify_runtime_bundle(root, expected)

    @unittest.skipIf(sys.platform == "win32", "symlink setup requires Windows privilege; NT reparse check remains in code")
    def test_symlink_or_link_escape_is_rejected_before_reading(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            directory = root / "bundle"
            directory.mkdir()
            self.bundle(directory)
            outside = root / "outside"
            outside.write_bytes(b"not a bundle input")
            (directory / "linked-input").symlink_to(outside)
            with self.assertRaisesRegex(ValueError, "symlink/reparse"):
                qualification.runtime_bundle_manifest(directory)

    def test_runtime_file_count_and_total_byte_bounds_are_enforced(self):
        for name, value in (("MAX_BUNDLE_FILES", 2), ("MAX_BUNDLE_BYTES", 16383),
                            ("MAX_BUNDLE_ENTRIES", 2)):
            with self.subTest(bound=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                self.bundle(root)
                with mock.patch.object(qualification, name, value), self.assertRaisesRegex(ValueError, "bound"):
                    qualification.runtime_bundle_manifest(root)


def source_stage(index=1, width=16, height=16, scale=4):
    value = stage()
    value.update(index=index, stage=qualification.STAGES[index - 1], width=width, height=height,
                 cpuReferenceScale=scale)
    value["stamp"].update(viewGeneration=index + 2, revision=2 if index == 1 else 3)
    value["limits"]["outstandingFrames"] = index
    def geometry(factor):
        return {"snapshot": 100 + index, "stamp": dict(value["stamp"]),
                "coordinateSpace": "viewportPixels", "viewport": {"width": width * factor, "height": height * factor},
                "placement": {"scale": .4 * factor, "originX": 0, "originY": 0},
                "descriptor": {"width": width * factor, "height": height * factor,
                               "stride": width * factor * 4, "pixelFormat": "rgba8Premultiplied",
                               "colorSpace": "srgb", "origin": "topLeft"}}
    value["geometryProof"] = {"cpu": geometry(1), "reference": geometry(scale)}
    return value


def source_images(index=1, width=16, height=16):
    # Independently specified CPU source: a thin pure-blue stripe and an
    # asymmetric gray rectangle. No fixture is derived from captured GPU bytes.
    low = bytearray([255] * (width * height * 4))
    reference = bytearray([255] * (width * height * 4 * 16))
    def put(data, row, x, y, color):
        offset = (y * row + x) * 4
        data[offset:offset + 4] = bytes(color)
    shade = 40 if index == 1 else 60
    for y in range(8, 14):
        for x in range(8, 14):
            put(low, width, x, y, (shade, shade, shade, 255))
    for y in range(32, 56):
        for x in range(32, 56):
            put(reference, width * 4, x, y, (shade, shade, shade, 255))
    for y in range(2, 14):
        put(low, width, 3, y, (64, 64, 255, 255))
    for y in range(8, 56):
        for x in range(12, 15):
            put(reference, width * 4, x, y, (0, 0, 255, 255))
    captured = bytearray(low)
    for y in range(2, 14):
        put(captured, width, 3, y, (32, 32, 255, 255))
    return bytes(low), bytes(reference), bytes(captured)


class SourceReferenceCaptureContracts(unittest.TestCase):
    """Mocked orchestration controls, never actual Windows capture evidence."""

    @contextmanager
    def harness(self, *, reference=False, coverage=False, mutate=None, reference_change=None,
                omit_reference=False, repeat_change=False, grow_reference_on_read=False):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runtime = root / "runtime"
            runtime.mkdir()
            RuntimeBundleContracts.bundle(self, runtime)
            _, supplied, receipt = BuildAndRuntimeIdentityContracts.artifacts(self, runtime)
            receipt.update(platform="windows", buildExitCode=0, runtimeBundle=qualification.runtime_bundle_manifest(runtime),
                           sourceHead="1" * 40, nativeSourceHead="2" * 40,
                           flutterRevision="3" * 40, engineRevision="4" * 40)
            build_receipt = root / "build.json"
            qualification.write_json(build_receipt, receipt)
            output = root / "capture"
            case = SimpleNamespace(output=output, config=None, comparisons=[], captures=[], killed=False, referenceReads=[])
            process = SimpleNamespace(pid=42)
            process.poll = lambda: 0 if case.killed else None
            process.wait = lambda **kwargs: 0
            process.kill = lambda: setattr(case, "killed", True)
            frames = []
            original_compare = qualification.compare_pixels
            original_open = Path.open

            class GrowingReference(io.BytesIO):
                def read(self, limit=-1):
                    case.referenceReads.append(limit)
                    return super().read(limit)

            def opening(path, mode="r", *args, **kwargs):
                if grow_reference_on_read and path == output / "cpu-reference-1.rgba" and mode == "rb":
                    with original_open(path, "rb") as existing:
                        return GrowingReference(existing.read() + b"\0")
                return original_open(path, mode, *args, **kwargs)

            def launch(*args, **kwargs):
                case.config = qualification.read_json(kwargs["env"]["RHWP_CONSUMER_PROBE_CONFIG"])
                last = None
                for index in range(1, 4):
                    width = 18 if index == 3 else 16
                    value = source_stage(index, width=width)
                    value["nonce"] = case.config["nonce"]
                    low, high, actual = source_images(index, width=width)
                    if mutate is not None and index == 1:
                        mutate(value)
                    qualification.write_json(output / f"stage-{index}.json", value)
                    (output / f"cpu-{index}.rgba").write_bytes(low)
                    if reference and not omit_reference:
                        if reference_change is not None and index == 1:
                            high = reference_change(high)
                        (output / f"cpu-reference-{index}.rgba").write_bytes(high)
                    frames.append((width, 16, actual if coverage else low))
                    last = value
                result = completion()
                result["nonce"] = case.config["nonce"]
                result["closed"]["limits"] = last["limits"]
                qualification.write_json(output / "result.json", result)
                return process

            def capture(width, height, offset):
                index = len(case.captures)
                expected_width, expected_height, pixels = frames[index // 2]
                self.assertEqual((width, height, offset), (expected_width, expected_height, (16, 16)))
                case.captures.append(pixels)
                return pixels[:-4] + b"\0\0\0\xff" if repeat_change and index == 1 else pixels

            def compare(expected, captured, width, height, **kwargs):
                index = len(case.comparisons) + 1
                self.assertEqual((output / f"compositor-{index}.rgba").read_bytes(), captured)
                case.comparisons.append(kwargs)
                return original_compare(expected, captured, width, height, **kwargs)

            argv = [str(TOOL), *(argument for key, path in supplied.items() for argument in ("--" + key, str(path))),
                    "--build-receipt", str(build_receipt), "--output", str(output)]
            if reference:
                argv.append("--cpu-reference")
            with mock.patch.object(qualification.sys, "argv", argv), \
                    mock.patch.object(qualification, "require_actual_windows"), \
                    mock.patch.object(qualification.subprocess, "Popen", side_effect=launch), \
                    mock.patch.object(qualification, "WindowsCompositor", return_value=SimpleNamespace(capture=capture)), \
                    mock.patch.object(qualification.time, "sleep"), \
                    mock.patch.object(qualification, "compare_pixels", side_effect=compare), \
                    mock.patch.object(Path, "open", new=opening), \
                    mock.patch("builtins.print"):
                yield case

    def assert_deadline_failure(self, expire):
        with self.harness(reference=True, coverage=True) as case:
            clock = SimpleNamespace(now=0.0, reads=0, bundle_checks=0)
            original_compare = qualification.compare_pixels
            original_capture = qualification.WindowsCompositor.return_value.capture
            original_launch = qualification.subprocess.Popen
            original_bundle = qualification.verify_runtime_bundle

            def monotonic():
                clock.reads += 1
                if expire == "ready-stage" and clock.reads > 1:
                    clock.now = 121.0
                return clock.now

            def compare(*args, **kwargs):
                result = original_compare(*args, **kwargs)
                if expire == "comparison":
                    clock.now = 121.0
                return result

            def capture(*args, **kwargs):
                result = original_capture(*args, **kwargs)
                if expire == "held-capture" and len(case.captures) == 2:
                    clock.now = 121.0
                return result

            def launch(*args, **kwargs):
                process = original_launch(*args, **kwargs)
                wait = process.wait

                def finish(**options):
                    result = wait(**options)
                    if expire == "close-wait":
                        clock.now = 121.0
                    return result

                process.wait = finish
                return process

            def bundle(*args, **kwargs):
                result = original_bundle(*args, **kwargs)
                clock.bundle_checks += 1
                if expire == "final-identity" and clock.bundle_checks == 2:
                    clock.now = 121.0
                return result

            with mock.patch.object(qualification.time, "monotonic", side_effect=monotonic), \
                    mock.patch.object(qualification, "compare_pixels", side_effect=compare), \
                    mock.patch.object(qualification.WindowsCompositor.return_value, "capture", side_effect=capture), \
                    mock.patch.object(qualification.subprocess, "Popen", side_effect=launch), \
                    mock.patch.object(qualification, "verify_runtime_bundle", side_effect=bundle):
                with self.assertRaisesRegex(TimeoutError, "deadline"):
                    qualification.main()
            receipt = qualification.read_json(case.output / "receipt.json")
            self.assertEqual(receipt["qualification"], "failed")
            if expire in ("ready-stage", "comparison", "held-capture"):
                self.assertFalse((case.output / "ack-1.json").exists())
            if expire == "ready-stage":
                self.assertEqual(case.captures, [])
            elif expire in ("comparison", "held-capture"):
                self.assertTrue((case.output / "compositor-1.rgba").exists())

    def test_expired_deadline_rejects_an_already_ready_stage_without_capture_or_ack(self):
        self.assert_deadline_failure("ready-stage")

    def test_cpu_comparison_crossing_deadline_cannot_ack_a_publication(self):
        self.assert_deadline_failure("comparison")

    def test_held_capture_crossing_deadline_cannot_ack_a_publication(self):
        self.assert_deadline_failure("held-capture")

    def test_successful_close_wait_finishing_after_deadline_is_not_qualification(self):
        self.assert_deadline_failure("close-wait")

    def test_final_runtime_identity_check_crossing_deadline_is_not_qualification(self):
        self.assert_deadline_failure("final-identity")

    def test_default_cli_keeps_strict_no_reference_comparison(self):
        with self.harness() as case:
            qualification.main()
            self.assertNotIn("cpuReference", case.config)
            self.assertEqual(case.comparisons, [{}, {}, {}])
            result = qualification.read_json(case.output / "receipt.json")
            self.assertNotIn("cpuReferenceRequested", result)
            self.assertEqual(result["qualification"], "bounded-windows-consumer-passed")
            self.assertTrue(all("sourceReferenceSha256" not in item for item in result["stages"]))

    def test_opt_in_passes_verified_same_snapshot_source_without_changing_the_predicate(self):
        with self.harness(reference=True, coverage=True) as case:
            qualification.main()
            self.assertIs(case.config["cpuReference"], True)
            self.assertEqual(len(case.comparisons), 3)
            result = qualification.read_json(case.output / "receipt.json")
            for index, item in enumerate(result["stages"], 1):
                self.assertEqual(item["comparison"]["sourceReferenceScale"], 4)
                self.assertEqual(item["sourceReferenceSha256"], qualification.digest(case.output / f"cpu-reference-{index}.rgba"))
                self.assertEqual(case.comparisons[index - 1]["reference_scale"], 4)
            self.assertTrue(result["cpuReferenceRequested"])

    def test_default_coverage_rejection_also_preserves_the_actual_raw_capture(self):
        with self.harness(coverage=True) as case:
            with self.assertRaisesRegex(ValueError, "AA envelope"):
                qualification.main()
            self.assertEqual((case.output / "compositor-1.rgba").read_bytes(), source_images()[2])
            self.assertEqual(qualification.read_json(case.output / "receipt.json")["qualification"], "failed")

    def test_unavailable_or_wrong_length_reference_fails_closed_after_preserving_raw(self):
        for options in ({"omit_reference": True}, {"reference_change": lambda data: data[:-1]},
                        {"reference_change": lambda data: data + b"\0"}):
            with self.subTest(options=tuple(options)), self.harness(reference=True, **options) as case:
                with self.assertRaises((ValueError, FileNotFoundError)):
                    qualification.main()
                self.assertTrue((case.output / "compositor-1.rgba").exists())
                self.assertFalse((case.output / "ack-1.json").exists())
                self.assertEqual(case.comparisons, [])

    def test_geometry_identity_coordinate_and_scale_failures_never_fall_back_to_strict_mode(self):
        mutations = [
            lambda x: x.pop("geometryProof"),
            lambda x: x["geometryProof"]["reference"].update(snapshot=999),
            lambda x: x["geometryProof"]["reference"]["stamp"].update(revision=999),
            lambda x: x["geometryProof"]["reference"].update(coordinateSpace="pagePixels"),
            lambda x: x["geometryProof"]["reference"]["viewport"].update(width=63),
            lambda x: x["geometryProof"]["reference"]["descriptor"].update(stride=1),
            lambda x: x["geometryProof"]["reference"]["placement"].update(scale=1.7),
            lambda x: x["geometryProof"]["reference"]["placement"].update(originX=float("nan")),
            lambda x: x.update(cpuReferenceScale=True),
            lambda x: x.update(cpuReferenceScale=1),
            lambda x: x.update(cpuReferenceScale=5),
        ]
        for mutate in mutations:
            with self.subTest(mutate=mutate), self.harness(reference=True, mutate=mutate) as case:
                with self.assertRaises(ValueError):
                    qualification.main()
                self.assertTrue((case.output / "compositor-1.rgba").exists())
                self.assertFalse((case.output / "ack-1.json").exists())
                self.assertEqual(case.comparisons, [])

    def test_reference_growing_after_stat_is_read_with_a_bound_and_rejected(self):
        with self.harness(reference=True, grow_reference_on_read=True) as case:
            with self.assertRaisesRegex(ValueError, "while reading"):
                qualification.main()
            self.assertEqual(case.referenceReads, [16 * 16 * 4 * 4**2 + 1])
            self.assertTrue((case.output / "compositor-1.rgba").exists())
            self.assertFalse((case.output / "ack-1.json").exists())
            self.assertEqual(case.comparisons, [])

    def test_reference_geometry_has_an_exact_sixteen_mib_bound(self):
        value = source_stage(width=1024, height=1024, scale=2)
        self.assertEqual(qualification.validate_source_reference(value, 1024, 1024), 2)
        for width, height, scale in ((1024, 1025, 2), (2049, 1, 2), (1, 2049, 2)):
            with self.subTest(width=width, height=height, scale=scale):
                value = source_stage(width=width, height=height, scale=scale)
                with self.assertRaisesRegex(ValueError, "16 MiB"):
                    qualification.validate_source_reference(value, width, height)

    def test_reference_alpha_failure_is_not_retried_with_a_weaker_oracle(self):
        def transparent(data):
            result = bytearray(data)
            result[3] = 0
            return bytes(result)
        with self.harness(reference=True, reference_change=transparent) as case:
            with self.assertRaisesRegex(ValueError, "opaque"):
                qualification.main()
            self.assertEqual(len(case.comparisons), 1)
            self.assertTrue((case.output / "compositor-1.rgba").exists())
            self.assertFalse((case.output / "ack-1.json").exists())

    def test_unstable_held_publication_keeps_its_first_raw_capture(self):
        with self.harness(reference=True, repeat_change=True) as case:
            with self.assertRaisesRegex(ValueError, "held publication"):
                qualification.main()
            self.assertEqual((case.output / "compositor-1.rgba").read_bytes(), source_images()[0])
            self.assertFalse((case.output / "ack-1.json").exists())


if __name__ == "__main__":
    unittest.main()
