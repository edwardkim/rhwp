#!/usr/bin/env python3
"""Portable negative controls for the actual Windows qualification collector.

These tests prove receipt/capture rejection behavior, never Windows execution.
"""

import importlib.util
from pathlib import Path
import sys
import shutil
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


if __name__ == "__main__":
    unittest.main()
