#!/usr/bin/env python3
"""Portable rejection controls for the Mac qualifier; never target execution."""

from contextlib import contextmanager
import hashlib
import importlib.util
import json
from pathlib import Path
import plistlib
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location(
    "qualify_native_view", Path(__file__).with_name("qualify_native_view.py"))
qualifier = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(qualifier)


def stage(index=1):
    return {
        "protocol": 1, "nonce": "owned", "index": index,
        "stage": ("initial", "revision", "resize")[index-1] if index <= 3 else "continuous",
        "platform": "macos", "pid": 123, "width": 320 if index < 3 else 300, "height": 240,
        "deviceScale": 2.0, "physicalOffset": [0, 0],
        "stamp": {"documentId": 1, "revision": 0 if index == 1 else 1,
                  "fontGeneration": 0, "viewGeneration": index,
                  "deviceGeneration": 1, "deviceScale": 2.0},
        "limits": {"backend": "appkitMetalLayer", "presentationKind": "nativeView",
                   "continuousPresentationSupported": True, "maximumDrawableCount": 3,
                   "consumerGpuCompletionVerified": False, "maxViews": 2},
    }


def result():
    return {
        "protocol": 1, "nonce": "owned", "ok": True,
        "consumerGpuCompletionVerified": False, "continuousPresentationSupported": True,
        "closed": {"ownerFinished": True, "registered": False,
                   "nativeViewLayerDetached": True, "quarantined": False, "frames": 32},
        "stats": {key: 0 for key in (
            "sources", "snapshotHandles", "retainedSnapshots", "snapshotEstimatedBytes",
            "frameHandles", "retainedFrames", "frameBytes", "leases", "inFlightReplays")},
    }


class AppEntryPointTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.bundle = Path(temporary.name) / "probe.app"
        self.main = self.bundle / "Contents/MacOS/probe"
        self.main.parent.mkdir(parents=True)
        self.main.write_bytes(b"portable selection control; never executed")
        self.main.chmod(0o700)
        self.info = self.bundle / "Contents/Info.plist"
        self.set_info({"CFBundleExecutable": "probe"})

    def set_info(self, value, fmt=plistlib.FMT_XML):
        self.info.write_bytes(plistlib.dumps(value, fmt=fmt))

    def test_plist_entry_point_permits_adjacent_debug_dylib(self):
        (self.main.parent / "probe.debug.dylib").write_bytes(b"separate debug code")
        for fmt in (plistlib.FMT_XML, plistlib.FMT_BINARY):
            self.set_info({"CFBundleExecutable": "probe"}, fmt)
            self.assertEqual(qualifier.app_executable(self.bundle), self.main)

    def test_missing_or_nonbasename_entry_point_is_rejected(self):
        for value in ({}, [], {"CFBundleExecutable": True}, *[
            {"CFBundleExecutable": name} for name in ("", ".", "..", "../probe", "/tmp/probe", "nested/probe", "nested\\probe", "probe\n")
        ]):
            with self.subTest(value=value):
                self.set_info(value)
                with self.assertRaises(ValueError):
                    qualifier.app_executable(self.bundle)

    def test_invalid_or_oversized_plist_is_rejected(self):
        for data in (b"not a plist", b"x" * (64 * 1024 + 1)):
            self.info.write_bytes(data)
            with self.assertRaises(ValueError):
                qualifier.app_executable(self.bundle)

    def test_missing_nonexecutable_and_linked_main_are_rejected(self):
        self.main.chmod(0o600)
        with self.assertRaises(ValueError):
            qualifier.app_executable(self.bundle)
        self.main.unlink()
        with self.assertRaises(FileNotFoundError):
            qualifier.app_executable(self.bundle)
        other = self.main.parent / "other"
        other.write_bytes(b"not the declared main")
        other.chmod(0o700)
        self.main.symlink_to("other")
        with self.assertRaises(ValueError):
            qualifier.app_executable(self.bundle)

    def test_linked_plist_is_rejected(self):
        other = self.info.with_name("external.plist")
        self.info.rename(other)
        self.info.symlink_to(other.name)
        with self.assertRaises(ValueError):
            qualifier.app_executable(self.bundle)


class RuntimeBundleTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "probe.app"
        self.kernel = self.root / "Contents/Frameworks/App.framework/Versions/A/Resources/flutter_assets/kernel_blob.bin"
        self.kernel.parent.mkdir(parents=True)
        self.kernel.write_bytes(b"independent Dart program")

    def test_records_actual_debug_program_and_valid_framework_links(self):
        framework = self.root / "Contents/Frameworks/App.framework"
        (framework / "Versions/Current").symlink_to("A", target_is_directory=True)
        (framework / "Resources").symlink_to("Versions/Current/Resources", target_is_directory=True)
        manifest = qualifier.bundle_manifest(self.root)
        qualifier.verify_bundle(self.root, manifest)
        self.assertEqual(manifest["Contents/Frameworks/App.framework/Resources"],
                         {"kind": "link", "target": "Versions/Current/Resources"})

    def test_changed_kernel_is_not_hidden_by_unchanged_entry_executable(self):
        manifest = qualifier.bundle_manifest(self.root)
        self.kernel.write_bytes(b"different Dart program")
        with self.assertRaisesRegex(ValueError, "changed"):
            qualifier.verify_bundle(self.root, manifest)

    def test_extra_and_missing_entries_are_rejected(self):
        manifest = qualifier.bundle_manifest(self.root)
        extra = self.root / "extra.dylib"
        extra.write_bytes(b"unexpected")
        with self.assertRaises(ValueError):
            qualifier.verify_bundle(self.root, manifest)
        extra.unlink()
        self.kernel.unlink()
        with self.assertRaisesRegex(ValueError, "kernel"):
            qualifier.verify_bundle(self.root, manifest)

    def test_external_and_absolute_framework_links_are_rejected(self):
        external = Path(self.temporary.name) / "foreign"
        external.write_bytes(b"outside the immutable app")
        link = self.root / "foreign"
        for target in ("../foreign", str(external)):
            with self.subTest(target=target):
                link.symlink_to(target)
                with self.assertRaises(ValueError):
                    qualifier.bundle_manifest(self.root)
                link.unlink()

    def test_dangling_loop_and_symlinked_bundle_roots_are_rejected(self):
        link = self.root / "link"
        for target in ("missing", "link"):
            with self.subTest(target=target):
                link.symlink_to(target)
                with self.assertRaises((ValueError, FileNotFoundError, RuntimeError)):
                    qualifier.bundle_manifest(self.root)
                link.unlink()
        alias = Path(self.temporary.name) / "alias.app"
        alias.symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "root"):
            qualifier.bundle_manifest(alias)

    def test_nonempty_kernel_and_byte_file_entry_bounds_are_enforced(self):
        for name, limit in (("MAX_BYTES", 1), ("MAX_FILES", 0), ("MAX_ENTRIES", 1)):
            with self.subTest(bound=name), mock.patch.object(qualifier, name, limit):
                with self.assertRaises(ValueError):
                    qualifier.bundle_manifest(self.root)
        self.kernel.write_bytes(b"")
        with self.assertRaisesRegex(ValueError, "kernel"):
            qualifier.bundle_manifest(self.root)


class StageTests(unittest.TestCase):
    def validate(self, value, *, index=1, previous=None):
        return qualifier.validate_stage(value, nonce="owned", index=index, pid=123, previous=previous)

    def test_32_ordered_publications_include_independent_revision_and_resize(self):
        previous = None
        for index in range(1, 33):
            value = stage(index)
            self.assertEqual(self.validate(value, index=index, previous=previous)[-1], 2.0)
            previous = value

    def test_foreign_process_nonce_platform_and_order_are_rejected(self):
        for key, wrong in (("pid", 124), ("nonce", "foreign"), ("platform", "linux"),
                           ("protocol", True), ("index", True), ("index", 2), ("stage", "initial-copy")):
            value = stage()
            value[key] = wrong
            with self.subTest(key=key, wrong=wrong), self.assertRaises(ValueError):
                self.validate(value)

    def test_geometry_nonfinite_scale_and_missing_full_stamp_are_rejected(self):
        for key, wrong in (("width", 0), ("width", 4097), ("height", True),
                           ("deviceScale", float("nan")), ("deviceScale", float("inf")),
                           ("physicalOffset", [0.1, 0]), ("physicalOffset", [-1, 0]),
                           ("stamp", {})):
            value = stage()
            value[key] = wrong
            with self.subTest(key=key, wrong=wrong), self.assertRaises(ValueError):
                self.validate(value)
        value = stage()
        value.update(width=4096, height=1025)
        with self.assertRaisesRegex(ValueError, "pixel bound"):
            self.validate(value)

    def test_cv_fallback_and_fabricated_fence_and_pool_expansion_are_rejected(self):
        for key, wrong in (("backend", "cvPixelBuffer"), ("presentationKind", "texture"),
                           ("continuousPresentationSupported", False), ("maximumDrawableCount", 4),
                           ("consumerGpuCompletionVerified", True), ("maxViews", 3)):
            value = stage()
            value["limits"][key] = wrong
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value)

    def test_stale_view_and_unplanned_source_device_dpr_changes_are_rejected(self):
        previous = stage(1)
        for key, wrong in (("viewGeneration", 1), ("documentId", 2), ("deviceGeneration", 2),
                           ("fontGeneration", 1), ("deviceScale", 1), ("revision", 0)):
            value = stage(2)
            value["stamp"][key] = wrong
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value, index=2, previous=previous)

    def test_revision_and_resize_controls_cannot_be_ordinary_repaints(self):
        value = stage(2)
        value["width"] = 319
        with self.assertRaisesRegex(ValueError, "revision control"):
            self.validate(value, index=2, previous=stage(1))
        value = stage(3)
        value["width"] = 320
        with self.assertRaisesRegex(ValueError, "resize control"):
            self.validate(value, index=3, previous=stage(2))


class CompletionTests(unittest.TestCase):
    def test_zero_leases_and_detached_finished_owner_are_required(self):
        qualifier.validate_result(result(), nonce="owned", frames=32)
        for key, wrong in (("ownerFinished", False), ("registered", True),
                           ("nativeViewLayerDetached", False), ("quarantined", True), ("frames", 31)):
            value = result()
            value["closed"][key] = wrong
            with self.subTest(key=key), self.assertRaises(ValueError):
                qualifier.validate_result(value, nonce="owned", frames=32)
        for key in result()["stats"]:
            value = result()
            value["stats"][key] = 1
            with self.subTest(lease=key), self.assertRaises(ValueError):
                qualifier.validate_result(value, nonce="owned", frames=32)

    def test_capture_is_not_a_gpu_fence_or_cross_target_execution(self):
        for key, wrong in (("nonce", "foreign"), ("ok", False),
                           ("consumerGpuCompletionVerified", True), ("continuousPresentationSupported", False)):
            value = result()
            value[key] = wrong
            with self.subTest(key=key), self.assertRaises(ValueError):
                qualifier.validate_result(value, nonce="owned", frames=32)
        with mock.patch.object(qualifier.sys, "platform", "linux"), self.assertRaisesRegex(RuntimeError, "actual macOS"):
            qualifier.require_macos()


class CoverTests(unittest.TestCase):
    def cover(self):
        return {key: stage()[key] for key in (
            "protocol", "nonce", "pid", "platform", "width", "height", "deviceScale", "physicalOffset")} | {
                "clearColor": 0xff123456, "nativeCovered": True, "presentationKind": "nativeView",
                "nativeViewMounted": True, "viewBound": True, "layerReady": True, "visibilitySerial": 2}

    def test_same_mounted_view_is_covered_by_an_opaque_flutter_color(self):
        color = qualifier.validate_cover(self.cover(), nonce="owned", pid=123, previous=stage())
        self.assertEqual(color, b"\x12\x34\x56\xff")

    def test_cover_without_native_ack_or_a_mounted_layer_cannot_qualify(self):
        for key, wrong in (("nativeCovered", False), ("nativeViewMounted", False), ("viewBound", False),
                           ("layerReady", False), ("visibilitySerial", 0), ("clearColor", 0x7f123456),
                           ("presentationKind", "texture"), ("width", 321), ("nonce", "foreign"), ("pid", 124)):
            value = self.cover()
            value[key] = wrong
            with self.subTest(key=key), self.assertRaises(ValueError):
                qualifier.validate_cover(value, nonce="owned", pid=123, previous=stage())


class HeldCaptureTests(unittest.TestCase):
    """Fake capture I/O only; independent CPU pixels use the real comparator."""

    width, height = 16, 16

    def oracle(self, background=255):
        # Three independently generated colors, observable edges and over 32
        # dark pixels satisfy the same content controls as actual capture.
        return b"".join(bytes((20, 30, 40, 255) if x < 4 and y < 8 else
                              (160, 40, 80, 255) if 4 <= x < 8 and y < 8 else
                              (background, background, background, 255))
                        for y in range(self.height) for x in range(self.width))

    def reference(self, expected):
        rows = []
        for y in range(self.height):
            row = b"".join(expected[(y*self.width+x)*4:(y*self.width+x+1)*4] * 2
                           for x in range(self.width))
            rows.extend((row, row))
        return b"".join(rows)

    def metadata(self):
        return {"pid": 123, "deviceScale": 1, "actualWindowServerCapture": True,
                "colorSpace": "srgb", "dynamicRange": "sdr", "windowId": 7,
                "width": self.width, "height": self.height,
                "windowBoundsPoints": [0, 0, self.width, self.height]}

    @contextmanager
    def capture_harness(self, samples, *, expected=None, metadata=None,
                        deadline=10, comparison_seconds=0, cover=False):
        expected = self.oracle() if expected is None else expected
        reference = self.reference(expected)
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        output = Path(temporary.name)
        clock = SimpleNamespace(now=0.0, sleeps=[])
        calls, comparisons = [], []
        width, height = self.width, self.height

        class PortableImage:
            # This is raw temporary test data, never a real compositor PNG.
            def __init__(self, pixels, image_width=width, image_height=height):
                self.pixels = pixels
                self.width, self.height = image_width, image_height
                self.size = (image_width, image_height)

            def __enter__(self):
                return self

            def __exit__(self, *_):
                return False

            def convert(self, mode):
                self_test.assertEqual(mode, "RGBA")
                return self

            def crop(self, rectangle):
                left, top, right, bottom = rectangle
                pixels = b"".join(self.pixels[(y*self.width+left)*4:(y*self.width+right)*4]
                                  for y in range(top, bottom))
                return PortableImage(pixels, right-left, bottom-top)

            def tobytes(self):
                return self.pixels

        self_test = self
        fake_pillow = SimpleNamespace(Image=SimpleNamespace(
            open=lambda path: PortableImage(Path(path).read_bytes())))

        def run(command, *, capture_output, text, timeout):
            self.assertTrue(capture_output and text)
            self.assertGreater(timeout, 0)
            self.assertLessEqual(timeout, min(30, deadline-clock.now))
            index = len(calls)
            calls.append({"command": command, "timeout": timeout})
            self.assertLess(index, len(samples), "capture exceeded the supplied bounded observations")
            sample = samples[index]
            if isinstance(sample, BaseException):
                raise sample
            destination = Path(command[-1])
            destination.write_bytes(sample)
            destination.chmod(0o600)
            meta = self.metadata() if metadata is None else metadata[index]
            return SimpleNamespace(returncode=0, stdout=json.dumps(meta), stderr="")

        def sleep(seconds):
            clock.sleeps.append(seconds)
            clock.now += seconds

        def validate(pixels):
            try:
                value = qualifier.compare_pixels(expected, pixels, width, height,
                    source_reference=reference, reference_scale=2)
            except ValueError as error:
                comparisons.append({"error": str(error)})
                raise
            comparisons.append(value)
            clock.now += comparison_seconds
            return value

        def capture():
            return qualifier.capture_pair({"capture": Path("portable-capture-not-executed")},
                output=output, label="control", pid=123, width=width, height=height,
                offset=(0, 0), dpr=1, deadline=deadline,
                validate_sample=None if cover else validate)

        def trace():
            return json.loads((output / "held-pair-control.json").read_text())

        with mock.patch.dict(sys.modules, {"PIL": fake_pillow}), \
                mock.patch.object(qualifier.subprocess, "run", side_effect=run), \
                mock.patch.object(qualifier.time, "monotonic", side_effect=lambda: clock.now), \
                mock.patch.object(qualifier.time, "sleep", side_effect=sleep):
            yield SimpleNamespace(capture=capture, trace=trace, calls=calls,
                                  comparisons=comparisons, clock=clock, output=output)

    def assert_observations_preserved(self, harness, samples):
        trace = harness.trace()
        self.assertEqual(len(trace["samples"]), len(samples))
        for item, pixels in zip(trace["samples"], samples):
            self.assertEqual(item["compositorSha256"], hashlib.sha256(pixels).hexdigest())
            self.assertEqual((harness.output / f'window-control-{item["suffix"]}.png').read_bytes(), pixels)
        return trace

    def test_one_unit_quantization_requires_three_samples_and_an_exact_pair(self):
        samples = [self.oracle(254), self.oracle(), self.oracle()]
        with self.capture_harness(samples) as harness:
            actual, metadata = harness.capture()
            self.assertEqual(actual, samples[-1])
            trace = self.assert_observations_preserved(harness, samples)
            self.assertEqual(trace["state"], "passed")
            self.assertTrue(trace["exactSelectedPair"])
            self.assertEqual(trace["selectedSuffixes"], ["b", "c"])
            self.assertEqual(trace["maximumSamples"], 4)
            self.assertEqual(metadata["heldCapture"], trace)
            self.assertEqual(len(harness.calls), 3)
            self.assertEqual([sample["comparison"] for sample in trace["samples"]], harness.comparisons)

    def test_already_exact_pair_uses_two_validated_observations(self):
        samples = [self.oracle()] * 2
        with self.capture_harness(samples) as harness:
            harness.capture()
            trace = self.assert_observations_preserved(harness, samples)
            self.assertEqual(trace["selectedSuffixes"], ["a", "b"])
            self.assertEqual(len(harness.calls), 2)
            self.assertEqual([sample["comparison"] for sample in trace["samples"]], harness.comparisons)

    def test_alternating_one_unit_samples_fail_after_four_without_a_fifth_capture(self):
        samples = [self.oracle(254), self.oracle()] * 2
        with self.capture_harness(samples) as harness:
            with self.assertRaisesRegex(ValueError, "exact consecutive pair"):
                harness.capture()
            trace = self.assert_observations_preserved(harness, samples)
            self.assertEqual(trace["state"], "failed")
            self.assertFalse(trace["exactSelectedPair"])
            self.assertEqual(len(harness.calls), 4)
            self.assertEqual([sample["comparison"] for sample in trace["samples"]], harness.comparisons)

    def test_exact_pair_can_be_found_at_the_fourth_and_final_sample(self):
        samples = [self.oracle(254), self.oracle(), self.oracle(254), self.oracle(254)]
        with self.capture_harness(samples) as harness:
            harness.capture()
            trace = self.assert_observations_preserved(harness, samples)
            self.assertEqual(trace["selectedSuffixes"], ["c", "d"])
            self.assertEqual(len(harness.calls), 4)

    def test_cpu_valid_127_to_128_to_129_drift_is_not_an_adjacent_only_cluster(self):
        samples = [self.oracle(127), self.oracle(128), self.oracle(129)]
        with self.capture_harness(samples, expected=self.oracle(128)) as harness:
            with self.assertRaisesRegex(ValueError, "one-unit RGB cluster"):
                harness.capture()
            trace = self.assert_observations_preserved(harness, samples)
            self.assertEqual(trace["state"], "failed")
            self.assertEqual(len(harness.calls), 3)
            self.assertEqual([sample["comparison"] for sample in trace["samples"]], harness.comparisons)

    def test_greater_than_one_unit_cpu_valid_difference_fails_without_settling(self):
        samples = [self.oracle(127), self.oracle(129)]
        with self.capture_harness(samples, expected=self.oracle(128)) as harness:
            with self.assertRaisesRegex(ValueError, "one-unit RGB cluster"):
                harness.capture()
            trace = self.assert_observations_preserved(harness, samples)
            self.assertEqual(trace["state"], "failed")
            self.assertEqual(len(harness.calls), 2)
            self.assertEqual([sample["comparison"] for sample in trace["samples"]], harness.comparisons)

    def test_cpu_mismatch_on_first_observation_does_not_wait_for_an_identical_bad_pair(self):
        wrong = bytearray(self.oracle())
        wrong[(12*self.width+12)*4:(12*self.width+13)*4] = bytes((0, 0, 255, 255))
        with self.capture_harness([bytes(wrong), bytes(wrong)]) as harness:
            with self.assertRaisesRegex(ValueError, "uniform CPU interior"):
                harness.capture()
            trace = self.assert_observations_preserved(harness, [bytes(wrong)])
            self.assertEqual(trace["state"], "failed")
            self.assertIn(harness.comparisons[-1]["error"], trace["error"])
            self.assertEqual(len(harness.calls), 1)

    def test_metadata_change_fails_without_retry_and_preserves_the_first_cpu_comparison(self):
        samples = [self.oracle(), self.oracle()]
        changed = self.metadata() | {"windowId": 8}
        with self.capture_harness(samples, metadata=[self.metadata(), changed]) as harness:
            with self.assertRaisesRegex(ValueError, "metadata changed"):
                harness.capture()
            trace = self.assert_observations_preserved(harness, samples)
            self.assertEqual(trace["state"], "failed")
            self.assertEqual(trace["samples"][0]["comparison"], harness.comparisons[0])
            self.assertEqual(len(harness.calls), 2)

    def test_nonopaque_alpha_fails_the_independent_cpu_control_without_retry(self):
        wrong = bytearray(self.oracle())
        wrong[(12*self.width+12)*4+3] = 254
        with self.capture_harness([self.oracle(), bytes(wrong), self.oracle()]) as harness:
            with self.assertRaisesRegex(ValueError, "opaque"):
                harness.capture()
            trace = self.assert_observations_preserved(harness, [self.oracle(), bytes(wrong)])
            self.assertEqual(trace["state"], "failed")
            self.assertIn(harness.comparisons[-1]["error"], trace["error"])
            self.assertEqual(len(harness.calls), 2)

    def test_cover_rejects_one_unit_change_without_settling_or_cpu_tolerance(self):
        samples = [self.oracle(254), self.oracle(), self.oracle()]
        with self.capture_harness(samples, cover=True) as harness:
            with self.assertRaisesRegex(ValueError, "changed during a held publication"):
                harness.capture()
            trace = self.assert_observations_preserved(harness, samples[:2])
            self.assertEqual(trace["state"], "failed")
            self.assertEqual(trace["maximumSamples"], 2)
            self.assertEqual(len(harness.calls), 2)
            self.assertFalse(harness.comparisons)

    def test_already_expired_deadline_records_failure_without_starting_capture(self):
        with self.capture_harness([], deadline=0) as harness:
            with self.assertRaisesRegex(TimeoutError, "overall deadline"):
                harness.capture()
            self.assertFalse(harness.calls)
            self.assertEqual(harness.trace()["state"], "failed")

    def test_deadline_during_settling_keeps_failed_trace_and_cpu_comparison(self):
        with self.capture_harness([self.oracle(254), self.oracle()], deadline=.05) as harness:
            with self.assertRaisesRegex(TimeoutError, "overall deadline"):
                harness.capture()
            self.assertEqual(len(harness.calls), 1)
            trace = self.assert_observations_preserved(harness, [self.oracle(254)])
            self.assertEqual(trace["state"], "failed")
            self.assertEqual(trace["samples"][0]["comparison"], harness.comparisons[0])

    def test_cpu_processing_cannot_return_success_after_the_overall_deadline(self):
        with self.capture_harness([self.oracle()] * 2, deadline=.5,
                                  comparison_seconds=.25) as harness:
            with self.assertRaisesRegex(TimeoutError, "overall deadline"):
                harness.capture()
            self.assertEqual(len(harness.calls), 2)
            trace = self.assert_observations_preserved(harness, [self.oracle()] * 2)
            self.assertEqual(trace["state"], "failed")
            self.assertEqual([sample["comparison"] for sample in trace["samples"]], harness.comparisons)

    def test_cluster_processing_cannot_return_success_after_the_overall_deadline(self):
        from builtins import min as original_min
        with self.capture_harness([self.oracle()] * 2, deadline=.5) as harness:
            def advance_during_cluster(*values):
                if len(harness.comparisons) == 2:
                    harness.clock.now = .6
                return original_min(*values)
            with mock.patch.object(qualifier, "min", side_effect=advance_during_cluster, create=True):
                with self.assertRaisesRegex(TimeoutError, "overall deadline"):
                    harness.capture()
            self.assertEqual(len(harness.calls), 2)
            trace = self.assert_observations_preserved(harness, [self.oracle()] * 2)
            self.assertEqual(trace["state"], "failed")
            self.assertFalse(trace["exactSelectedPair"])

    def test_capture_timeout_records_failure_and_preserves_prior_comparison(self):
        expired = qualifier.subprocess.TimeoutExpired("portable-capture-not-executed", .5)
        with self.capture_harness([self.oracle(254), expired]) as harness:
            with self.assertRaises(qualifier.subprocess.TimeoutExpired):
                harness.capture()
            trace = harness.trace()
            self.assertEqual(trace["state"], "failed")
            self.assertIn("TimeoutExpired", trace["error"])
            self.assertEqual(trace["samples"][0]["comparison"], harness.comparisons[0])
            self.assertEqual(len(harness.calls), 2)

    def test_invalid_capture_metadata_records_failure_before_any_retry(self):
        changed = self.metadata() | {"pid": 124}
        with self.capture_harness([self.oracle()] * 2,
                                  metadata=[self.metadata(), changed]) as harness:
            with self.assertRaisesRegex(ValueError, "foreign/unsupported"):
                harness.capture()
            trace = harness.trace()
            self.assertEqual(trace["state"], "failed")
            self.assertEqual(trace["samples"][0]["comparison"], harness.comparisons[0])
            self.assertEqual(len(harness.calls), 2)


class OwnedBuildStepTests(unittest.TestCase):
    def test_initial_receipt_failure_stops_the_owned_child_and_records_exit(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            child = mock.Mock(pid=123, returncode=None)
            state = {"steps": []}
            record_error = OSError("receipt write failed")

            def stopped(process):
                self.assertIs(process, child)
                process.returncode = -9

            with mock.patch.object(qualifier.subprocess, "Popen", return_value=child) as spawn, \
                    mock.patch.object(qualifier, "write_json", side_effect=[record_error, None]) as record, \
                    mock.patch.object(qualifier, "stop_owned_process_group", side_effect=stopped) as stop:
                with self.assertRaises(OSError) as raised:
                    qualifier.run_step(["portable-child-not-executed"], cwd=output, env={},
                        output=output, receipt=output / "receipt.json", state=state)
            self.assertIs(raised.exception, record_error)
            self.assertTrue(spawn.call_args.kwargs["start_new_session"])
            stop.assert_called_once_with(child)
            child.wait.assert_not_called()
            self.assertEqual(record.call_count, 2)
            self.assertEqual(state["steps"][0]["exitCode"], -9)
            self.assertIn("finished", state["steps"][0])


if __name__ == "__main__":
    unittest.main()
