#!/usr/bin/env python3
"""Portable rejection controls; these are not actual GTK/GL execution evidence."""

from pathlib import Path
from contextlib import contextmanager
import os
import shutil
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tool"))
from qualify_linux_texture import stage_name, validate_result, validate_stage, verify_loaded_artifacts
from qualify_desktop_texture import compare_pixels, validate_source_reference
import qualify_linux_texture as qualification


def stage(index=1):
    stamp = {"documentId": 1, "revision": 0, "fontGeneration": 0,
             "viewGeneration": index, "deviceGeneration": 1, "deviceScale": 1}
    def geometry(scale):
        return {"snapshot": 7, "stamp": dict(stamp), "coordinateSpace": "viewportPixels",
                "viewport": {"width": 320 * scale, "height": 240 * scale},
                "placement": {"scale": .4 * scale, "originX": 0, "originY": 0},
                "descriptor": {"width": 320 * scale, "height": 240 * scale,
                               "stride": 320 * scale * 4, "pixelFormat": "rgba8Premultiplied",
                               "colorSpace": "srgb", "origin": "topLeft"}}
    return {"protocol": 1, "nonce": "owned", "pid": 42, "platform": "linux", "index": index,
            "stage": stage_name(index), "width": 320, "height": 240,
            "physicalOffset": [16, 16], "deviceScale": 1, "stamp": stamp,
            "cpuReferenceScale": 4, "geometryProof": {"cpu": geometry(1), "reference": geometry(4)},
            "nativeStatus": {"renderedStamp": dict(stamp), "framesSubmitted": index,
                             "wholeFrameReadbacks": 0, "abandonedTargets": 0,
                             "orphanedOwners": 0, "orphanedRasterSlots": 0,
                             "pendingRetireSlots": 0, "snapshotReleaseFailures": 0,
                             "registered": True, "closing": False, "cleanupComplete": False,
                             "liveRasterSlots": 1, "rasterSlotLimit": 4,
                             "glVendor": "Mesa", "glRenderer": "llvmpipe", "glVersion": "4.5"}}


def result():
    return {"protocol": 1, "nonce": "owned", "ok": True, "publications": 32,
            "consumerGpuCompletionVerified": False, "continuousPresentationSupported": True,
            "closed": {"registered": False, "cleanupComplete": True, "liveRasterSlots": 0,
                       "pendingRetireSlots": 0, "orphanedRasterSlots": 0, "abandonedTargets": 0,
                       "orphanedOwners": 0, "snapshotReleaseFailures": 0, "wholeFrameReadbacks": 0},
            "stats": dict.fromkeys(("sources", "snapshotHandles", "retainedSnapshots",
                                    "snapshotEstimatedBytes", "frameHandles", "retainedFrames",
                                    "frameBytes", "leases", "inFlightReplays"), 0)}


class LinuxQualification(unittest.TestCase):
    def test_identity_and_thirty_two_frame_contract(self):
        for index in (1, 2, 3, 4, 32):
            self.assertEqual(validate_stage(stage(index), nonce="owned", index=index, pid=42),
                             (320, 240, (16, 16)))
        validate_result(result(), nonce="owned", frames=32)

    def test_foreign_stage_identity(self):
        for key, replacement in (("nonce", "foreign"), ("pid", 43), ("platform", "windows"),
                                 ("index", 2), ("stage", "continuous"), ("protocol", 2)):
            with self.subTest(key=key):
                value = stage()
                value[key] = replacement
                with self.assertRaises(ValueError):
                    validate_stage(value, nonce="owned", index=1, pid=42)

    def test_source_stamp_and_device_scale_must_match(self):
        for key, replacement in (("viewGeneration", 7), ("revision", 1), ("deviceScale", 2)):
            with self.subTest(key=key):
                value = stage()
                value["stamp"][key] = replacement
                with self.assertRaises(ValueError):
                    validate_stage(value, nonce="owned", index=1, pid=42)

    def test_reference_keeps_the_same_snapshot_stamp_and_scaled_geometry(self):
        for field, change in (("snapshot", 8), ("snapshot", True), ("coordinateSpace", "pagePixels")):
            with self.subTest(field=field, value=change):
                value = stage()
                value["geometryProof"]["reference"][field] = change
                with self.assertRaises(ValueError):
                    validate_source_reference(value, 320, 240)
        for field, change in (("scale", 1.7), ("originX", 1), ("originY", float("nan"))):
            with self.subTest(field=field):
                value = stage()
                value["geometryProof"]["reference"]["placement"][field] = change
                with self.assertRaises(ValueError):
                    validate_source_reference(value, 320, 240)
        value = stage()
        value["geometryProof"]["reference"]["stamp"]["deviceScale"] = 4
        with self.assertRaises(ValueError):
            validate_source_reference(value, 320, 240)

    def test_reference_unavailable_is_closed_at_native_bounds(self):
        for scale, width, height in ((1, 320, 240), (4, 4096, 240), (2, 2048, 1024)):
            value = stage()
            value["cpuReferenceScale"] = scale
            with self.assertRaisesRegex(ValueError, "referenceUnavailable"):
                validate_source_reference(value, width, height)

    def test_readback_and_orphan_retirement_are_failures(self):
        for key in ("wholeFrameReadbacks", "abandonedTargets", "orphanedOwners",
                    "orphanedRasterSlots", "pendingRetireSlots", "snapshotReleaseFailures"):
            with self.subTest(key=key):
                value = stage()
                value["nativeStatus"][key] = 1
                with self.assertRaises(ValueError):
                    validate_stage(value, nonce="owned", index=1, pid=42)

    def test_owner_and_raster_progress_are_required(self):
        for key, replacement in (("registered", False), ("closing", True),
                                 ("cleanupComplete", True), ("liveRasterSlots", 2),
                                 ("rasterSlotLimit", 5), ("framesSubmitted", 0)):
            with self.subTest(key=key):
                value = stage()
                value["nativeStatus"][key] = replacement
                with self.assertRaises(ValueError):
                    validate_stage(value, nonce="owned", index=1, pid=42)

    def test_actual_context_driver_identity_is_bounded(self):
        for key in ("glVendor", "glRenderer", "glVersion"):
            for replacement in (None, "", "x" * 257, "Mesa\0foreign"):
                with self.subTest(key=key, value=replacement):
                    value = stage()
                    value["nativeStatus"][key] = replacement
                    with self.assertRaises(ValueError):
                        validate_stage(value, nonce="owned", index=1, pid=42)

    def test_close_is_not_ownership_or_gpu_completion_fiction(self):
        for key, replacement in (("registered", True), ("cleanupComplete", False),
                                 ("liveRasterSlots", 1), ("pendingRetireSlots", 1),
                                 ("orphanedRasterSlots", 1), ("wholeFrameReadbacks", 1)):
            with self.subTest(key=key):
                value = result()
                value["closed"][key] = replacement
                with self.assertRaises(ValueError):
                    validate_result(value, nonce="owned", frames=32)
        value = result()
        value["consumerGpuCompletionVerified"] = True
        with self.assertRaises(ValueError):
            validate_result(value, nonce="owned", frames=32)

    def test_native_owner_leaks_are_not_completion(self):
        for key in result()["stats"]:
            with self.subTest(key=key):
                value = result()
                value["stats"][key] = 1
                with self.assertRaises(ValueError):
                    validate_result(value, nonce="owned", frames=32)

    def test_four_frame_probe_does_not_prove_continuous_publication(self):
        value = result()
        value["publications"] = 4
        with self.assertRaises(ValueError):
            validate_result(value, nonce="owned", frames=4)
        value["continuousPresentationSupported"] = False
        validate_result(value, nonce="owned", frames=4)

    @unittest.skipUnless(sys.platform == "linux", "requires actual Linux process mappings")
    def test_loaded_elf_identity_uses_the_actual_process(self):
        executable = Path("/proc/self/exe").resolve()
        actual = verify_loaded_artifacts(os.getpid(), {"runner": executable})
        self.assertEqual(set(actual), {"runner"})

    @unittest.skipUnless(sys.platform == "linux", "requires actual Linux process mappings")
    def test_identical_module_bytes_in_another_inode_are_not_the_loaded_artifact(self):
        executable = Path("/proc/self/exe").resolve()
        with tempfile.TemporaryDirectory() as directory:
            other = Path(directory) / executable.name
            shutil.copyfile(executable, other)
            with self.assertRaises(ValueError):
                verify_loaded_artifacts(os.getpid(), {"engineRuntime": other})


class IndependentSourcePalette(unittest.TestCase):
    def setUp(self):
        self.width = self.height = 16
        self.low = bytearray([255] * (16 * 16 * 4))
        self.reference = bytearray([255] * (64 * 64 * 4))
        for y in range(8, 14):
            for x in range(8, 14):
                self.put(self.low, 16, x, y, (40, 40, 40, 255))
        for y in range(32, 56):
            for x in range(32, 56):
                self.put(self.reference, 64, x, y, (40, 40, 40, 255))
        # The independently specified source stripe has a pure-blue region.
        # Its low-res AA samples need not contain that source color at all.
        for y in range(2, 14):
            self.put(self.low, 16, 3, y, (64, 64, 255, 255))
        for y in range(8, 56):
            for x in range(12, 15):
                self.put(self.reference, 64, x, y, (0, 0, 255, 255))
        self.actual = bytearray(self.low)
        for y in range(2, 14):
            self.put(self.actual, 16, 3, y, (32, 32, 255, 255))
        self.low = bytes(self.low)
        self.reference = bytes(self.reference)

    @staticmethod
    def put(data, width, x, y, color):
        offset = (y * width + x) * 4
        data[offset:offset + 4] = bytes(color)

    def compare(self, candidate):
        return compare_pixels(self.low, candidate, 16, 16,
                              source_reference=self.reference, reference_scale=4)

    def test_independent_source_supports_coverage_absent_from_low_resolution_samples(self):
        with self.assertRaises(ValueError):
            compare_pixels(self.low, self.actual, 16, 16)
        result = self.compare(self.actual)
        self.assertEqual(result["sourceReferenceScale"], 4)
        self.assertGreater(result["aaEdgeDifferentPixels"], 0)

    def test_color_correlation_rejects_hues_inside_the_source_rgb_box(self):
        for color in ((255, 0, 255, 255), (0, 255, 255, 255)):
            with self.subTest(color=color):
                candidate = bytearray(self.actual)
                self.put(candidate, 16, 3, 6, color)
                with self.assertRaises(ValueError):
                    self.compare(candidate)

    def test_uniform_interior_and_outside_edge_halo_remain_strict(self):
        for x, y, color in ((10, 10, (0, 255, 0, 255)), (1, 1, (0, 0, 255, 255))):
            candidate = bytearray(self.actual)
            self.put(candidate, 16, x, y, color)
            with self.assertRaises(ValueError):
                self.compare(candidate)

    def test_one_and_two_pixel_translation_and_stale_image_are_rejected(self):
        for dx in (1, 2, 4):
            with self.subTest(dx=dx):
                candidate = bytearray([255] * len(self.actual))
                for y in range(16):
                    start = y * 16 * 4
                    candidate[start + dx * 4:start + 64] = self.actual[start:start + (16 - dx) * 4]
                with self.assertRaises(ValueError):
                    self.compare(candidate)

    def test_blank_cover_is_not_source_color_support(self):
        with self.assertRaises(ValueError):
            self.compare(bytes([255] * len(self.actual)))

    def test_unavailable_reference_is_not_a_fallback_to_a_weaker_comparison(self):
        with self.assertRaisesRegex(ValueError, "referenceUnavailable"):
            compare_pixels(self.low, self.actual, 16, 16, source_reference=b"", reference_scale=4)

    def test_ambiguous_integer_alignment_is_not_qualified(self):
        low = self.low[9 * 64:10 * 64] * 16
        actual = self.actual[9 * 64:10 * 64] * 16
        reference = self.reference[36 * 256:37 * 256] * 64
        with self.assertRaisesRegex(ValueError, "alignment.*ambiguous"):
            compare_pixels(low, actual, 16, 16, source_reference=reference, reference_scale=4)


class CollectorDeadlineContracts(unittest.TestCase):
    """Portable orchestration controls, not genuine GTK execution evidence."""

    @contextmanager
    def harness(self, expire_at=None):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runtime = root / "runtime"
            runtime.mkdir()
            files = {}
            for key in ("runner", "native", "dispatcher", "fixture", "plugin", "engineRuntime"):
                files[key] = runtime / key
                files[key].write_bytes(key.encode())
            receipt = {"platform": "linux", "buildExitCode": 0,
                       **dict.fromkeys(("sourceHead", "nativeSourceHead", "flutterRevision", "engineRevision"), "a" * 40)}
            build = root / "build.json"
            qualification.write_json(build, receipt)
            output = root / "capture"
            clock = SimpleNamespace(now=0.)
            case = SimpleNamespace(output=output, captures=0, closed=False)
            process = SimpleNamespace(pid=42, returncode=None)
            process.poll = lambda: process.returncode
            def wait(**kwargs):
                process.returncode = 0
                if expire_at == "wait":
                    clock.now = 31.
                return 0
            process.wait = wait
            frames = []

            def launch(*args, **kwargs):
                config = qualification.read_json(output / "config.json")
                for index in range(1, 4):
                    value = stage(index)
                    value["nonce"] = config["nonce"]
                    value["stamp"]["revision"] = 0 if index == 1 else 1
                    value["nativeStatus"]["renderedStamp"] = dict(value["stamp"])
                    value.update(width=16, height=16)
                    for name, factor in (("cpu", 1), ("reference", 4)):
                        geometry = value["geometryProof"][name]
                        geometry["stamp"] = dict(value["stamp"])
                        geometry["viewport"] = {"width": 16 * factor, "height": 16 * factor}
                        geometry["descriptor"].update(width=16 * factor, height=16 * factor, stride=64 * factor)
                    low = bytearray([255] * (16 * 16 * 4))
                    for y in range(3, 13):
                        for x in range(2, 10):
                            at = (y * 16 + x) * 4
                            low[at:at + 4] = bytes([30 + index] * 3 + [255])
                    for y in range(2, 14):
                        at = (y * 16 + 12) * 4
                        low[at:at + 4] = bytes([0, 0, 255, 255])
                    high = b"".join(b"".join(bytes(low[y * 64 + x * 4:y * 64 + x * 4 + 4]) * 4
                                            for x in range(16)) * 4 for y in range(16))
                    qualification.write_json(output / f"stage-{index}.json", value)
                    (output / f"cpu-{index}.rgba").write_bytes(low)
                    (output / f"cpu-reference-{index}.rgba").write_bytes(high)
                    frames.append(bytes(low))
                completion = result()
                completion.update(nonce=config["nonce"], publications=3, continuousPresentationSupported=False)
                qualification.write_json(output / "result.json", completion)
                return process

            def capture(*args):
                pixels = frames[case.captures // 2]
                case.captures += 1
                if expire_at == "capture" and case.captures == 2:
                    clock.now = 31.
                return pixels
            def compare(*args, **kwargs):
                value = compare_pixels(*args, **kwargs)
                if expire_at == "comparison":
                    clock.now = 31.
                return value
            bundle_checks = 0
            def bundle(*args):
                nonlocal bundle_checks
                bundle_checks += 1
                if expire_at == "final artifacts" and bundle_checks == 2:
                    clock.now = 31.
            def stop(child):
                child.returncode = -9
            def close():
                case.closed = True
            argv = ["qualify_linux_texture.py",
                    *(part for name in ("runner", "native", "dispatcher", "fixture")
                      for part in ("--" + name, str(files[name]))),
                    "--build-receipt", str(build), "--output", str(output), "--frames", "3", "--timeout", "30"]
            with mock.patch.object(sys, "argv", argv), \
                    mock.patch.object(qualification.subprocess, "Popen", side_effect=launch), \
                    mock.patch.object(qualification, "X11Compositor", return_value=SimpleNamespace(capture=capture, close=close)), \
                    mock.patch.object(qualification, "verify_runtime_artifacts", return_value=files), \
                    mock.patch.object(qualification, "verify_runtime_bundle", side_effect=bundle), \
                    mock.patch.object(qualification, "verify_loaded_artifacts", return_value={}), \
                    mock.patch.object(qualification, "stop_process_tree", side_effect=stop), \
                    mock.patch.object(qualification.time, "monotonic", side_effect=lambda: clock.now), \
                    mock.patch.object(qualification.time, "sleep"), \
                    mock.patch.object(qualification, "compare_pixels", side_effect=compare), \
                    mock.patch("builtins.print"):
                yield case

    def test_on_time_held_frames_and_cleanup_still_qualify(self):
        with self.harness() as case:
            qualification.main()
            self.assertEqual(qualification.read_json(case.output / "receipt.json")["qualification"], "linux-consumer-passed")
            self.assertEqual(len(list(case.output.glob("ack-*.json"))), 3)
            self.assertTrue(case.closed)

    def test_expired_work_cannot_acknowledge_or_finish_qualification(self):
        for phase in ("comparison", "capture", "wait", "final artifacts"):
            with self.subTest(phase=phase), self.harness(expire_at=phase) as case:
                with self.assertRaisesRegex(TimeoutError, "deadline"):
                    qualification.main()
                self.assertEqual(qualification.read_json(case.output / "receipt.json")["qualification"], "failed")
                if phase in ("comparison", "capture"):
                    self.assertFalse((case.output / "ack-1.json").exists())
                self.assertTrue(case.closed)


if __name__ == "__main__":
    unittest.main()
