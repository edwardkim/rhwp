#!/usr/bin/env python3
"""Portable protocol/ownership/pixel rejection controls, not Windows execution.

The integer bounds come from FrameLifetime/SurfaceImportTickets. Synthetic RGB
shapes exercise the independent-source predicate, without layout golden values.
"""

import copy
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
from unittest import mock
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tool"))
import qualify_windows_continuous as qualification


def stage(index=1):
    width, height = (40, 30) if index < 3 else (44, 32)
    stamp = {"documentId": 1, "revision": 2 if index == 1 else 3, "fontGeneration": 0,
             "viewGeneration": index + 2, "deviceGeneration": 1, "deviceScale": 1.25}
    snapshot = index + 7

    def geometry(factor):
        return {"snapshot": snapshot, "stamp": dict(stamp), "coordinateSpace": "viewportPixels",
                "viewport": {"width": width * factor, "height": height * factor},
                "placement": {"scale": .5 * factor, "originX": 0, "originY": 0},
                "descriptor": {"width": width * factor, "height": height * factor,
                               "stride": width * factor * 4, "pixelFormat": "rgba8Premultiplied",
                               "colorSpace": "srgb", "origin": "topLeft"}}

    limits = {"publicationMode": qualification.MODE, "continuousPresentationSupported": True,
              "consumerRetirement": False, "consumerGpuCompletionVerified": False, "targetReuse": False,
              "consumerAndDriverMemoryIncluded": False, "descriptorReleaseIsGpuComplete": False,
              "cpuFallbackRequiredWhenSaturated": True,
              "frameAccountingScope": "legacyRetainedAndPluginOwnedReferences",
              "maxFrames": 4, "maxTargetBytes": 64 * 1024 * 1024,
              "maxSingleFrameBytes": 16 * 1024 * 1024, "maxDescriptorTickets": 8,
              "producerOwnedFrames": 1, "outstandingFrames": 1, "pendingRetireFrames": 1,
              "producerOwnedBytes": 1024 * 1024, "reservedTargetBytes": 1024 * 1024,
              "importPendingFrames": 0, "importPendingBytes": 0,
              "legacyRetainedFrames": 0, "immutableFrames": 0, "quarantinedFrames": 0,
              "wholeFrameReadbacks": 0, "descriptorTickets": 1, "descriptorCpuPendingTickets": 1,
              "descriptorAwaitingOpenedTickets": 0, "descriptorDuplicateOrLateNotifications": 0,
              "handedOffFrames": index - 1, "discardedUnexposedFrames": 0,
              "descriptorOpenedNotifications": index}
    native = {"textureId": 4, "deviceGeneration": 1, "publicationMode": qualification.MODE,
              "ready": True, "closed": False, "ownerFinished": False, "inFlight": False,
              "unregistered": False, "hasFrame": True, "deviceFailure": False, "error": "", "errorCode": "",
              "acceptedSerial": index, "renderedSerial": index, "descriptorOpenedSerial": index,
              "acceptedStamp": dict(stamp), "renderedStamp": dict(stamp), "acceptedSnapshots": [snapshot],
              "limits": limits}
    return {"protocol": 1, "nonce": "owned", "pid": 42, "platform": "windows", "index": index,
            "stage": qualification.stage_name(index), "width": width, "height": height,
            "physicalOffset": [16, 16], "deviceScale": 1.25, "stamp": stamp,
            "limits": limits, "nativeStatus": native, "cpuReferenceScale": 4,
            "geometryProof": {"cpu": geometry(1), "reference": geometry(4)}}


def completion(frames=32):
    closed = copy.deepcopy(stage(frames)["nativeStatus"])
    closed.update(closed=True, ownerFinished=True, unregistered=True, ready=False, inFlight=False,
                  hasFrame=False, renderedStamp=None, renderedSerial=0, descriptorOpenedSerial=0)
    for key in ("producerOwnedFrames", "producerOwnedBytes", "outstandingFrames", "pendingRetireFrames",
                "reservedTargetBytes", "descriptorTickets", "descriptorCpuPendingTickets"):
        closed["limits"][key] = 0
    closed["limits"]["handedOffFrames"] = frames
    return {"protocol": 1, "nonce": "owned", "ok": True, "publications": frames,
            "continuousPresentationSupported": True, "consumerGpuCompletionVerified": False,
            "closed": closed, "stats": dict.fromkeys(qualification.NATIVE_STATS, 0)}


class WindowsContinuousContracts(unittest.TestCase):
    def validate(self, value, index=1):
        return qualification.validate_stage(value, nonce="owned", index=index, pid=42)

    def finish(self, value):
        return qualification.validate_result(value, nonce="owned", frames=32, last_stage=stage(32))

    def test_thirty_two_publications_and_actual_owner_close_contract(self):
        previous = None
        for index in range(1, 33):
            current = stage(index)
            self.assertEqual(self.validate(current, index),
                             (current["width"], current["height"], (16, 16)))
            if previous is not None:
                qualification.validate_transition(previous, current)
            previous = current
        self.finish(completion())

    def test_foreign_or_out_of_order_publications_fail(self):
        for key, replacement in (("nonce", "foreign"), ("pid", 43), ("platform", "linux"),
                                 ("index", 2), ("stage", "continuous"), ("protocol", 2),
                                 ("protocol", True), ("index", True)):
            with self.subTest(key=key):
                value = stage()
                value[key] = replacement
                with self.assertRaises(ValueError):
                    self.validate(value)

    def test_cpu_reference_and_native_publication_bind_the_same_snapshot_and_stamp(self):
        for key in ("acceptedStamp", "renderedStamp"):
            value = stage()
            value["nativeStatus"][key]["revision"] += 1
            with self.assertRaises(ValueError):
                self.validate(value)
        for snapshots in ([], [99], [8, 8], [True], [2**63], list(range(1, 10))):
            value = stage()
            value["nativeStatus"]["acceptedSnapshots"] = snapshots
            with self.assertRaises(ValueError):
                self.validate(value)
        value = stage()
        value["geometryProof"]["reference"]["snapshot"] += 1
        with self.assertRaises(ValueError):
            self.validate(value)

    def test_held_status_must_be_ready_current_and_opened(self):
        for key in ("ready", "hasFrame", "closed", "ownerFinished", "inFlight", "unregistered", "deviceFailure"):
            value = stage()
            value["nativeStatus"][key] = not value["nativeStatus"][key]
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value)
        for key in ("acceptedSerial", "renderedSerial", "descriptorOpenedSerial", "deviceGeneration"):
            for replacement in (0, True, 2**63):
                value = stage()
                value["nativeStatus"][key] = replacement
                with self.subTest(key=key, value=replacement), self.assertRaises(ValueError):
                    self.validate(value)
        value = stage()
        value["nativeStatus"]["error"] = "device lost"
        with self.assertRaises(ValueError):
            self.validate(value)

    def test_legacy_or_contradictory_mode_cannot_qualify_continuous_transfer(self):
        for location in ("native", "limits", "stage"):
            value = stage()
            if location == "native":
                value["nativeStatus"]["publicationMode"] = "legacyImmutable"
            elif location == "limits":
                value["limits"]["publicationMode"] = "legacyImmutable"
            else:
                value["limits"] = dict(value["limits"], producerOwnedFrames=2)
            with self.subTest(location=location), self.assertRaises(ValueError):
                self.validate(value)

    def test_creator_reference_handoff_is_never_gpu_completion_or_target_reuse(self):
        for key in ("consumerRetirement", "consumerGpuCompletionVerified", "targetReuse",
                    "consumerAndDriverMemoryIncluded", "descriptorReleaseIsGpuComplete"):
            value = stage()
            value["limits"][key] = True
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value)
        for key in ("continuousPresentationSupported", "cpuFallbackRequiredWhenSaturated"):
            value = stage()
            value["limits"][key] = False
            with self.assertRaises(ValueError):
                self.validate(value)

    def test_fixed_admission_caps_cannot_be_expanded(self):
        for key in ("maxFrames", "maxTargetBytes", "maxSingleFrameBytes", "maxDescriptorTickets"):
            for replacement in (True, stage()["limits"][key] + 1):
                value = stage()
                value["limits"][key] = replacement
                with self.subTest(key=key, value=replacement), self.assertRaises(ValueError):
                    self.validate(value)

    def test_count_byte_ticket_and_quarantine_overflows_are_rejected(self):
        for key, replacement in (("producerOwnedFrames", 5), ("producerOwnedBytes", 64 * 1024 * 1024 + 1),
                                 ("producerOwnedBytes", 0), ("importPendingFrames", 2),
                                 ("importPendingBytes", 1), ("descriptorTickets", 9),
                                 ("descriptorCpuPendingTickets", 2), ("descriptorAwaitingOpenedTickets", 2),
                                 ("outstandingFrames", 2), ("reservedTargetBytes", 1),
                                 ("legacyRetainedFrames", 1), ("immutableFrames", 1),
                                 ("quarantinedFrames", 1), ("wholeFrameReadbacks", 1),
                                 ("handedOffFrames", 2**63), ("descriptorOpenedNotifications", True)):
            value = stage()
            value["limits"][key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value)

    def test_current_export_pin_and_bounded_pending_import_are_allowed(self):
        value = stage(32)
        value["limits"].update(producerOwnedFrames=2, producerOwnedBytes=2 * 1024 * 1024,
                               outstandingFrames=2, pendingRetireFrames=2, reservedTargetBytes=2 * 1024 * 1024,
                               importPendingFrames=1, importPendingBytes=1024 * 1024,
                               descriptorTickets=2, descriptorCpuPendingTickets=2,
                               descriptorAwaitingOpenedTickets=1, handedOffFrames=30)
        self.validate(value, 32)

    def test_missing_opening_and_handoff_accounting_is_rejected(self):
        for key, replacement in (("handedOffFrames", 0), ("descriptorOpenedNotifications", 31)):
            value = stage(32)
            value["limits"][key] = replacement
            with self.assertRaises(ValueError):
                self.validate(value, 32)

    def test_revision_view_registration_and_native_serial_must_advance_together(self):
        for modify in (lambda x: x["stamp"].update(revision=2),
                       lambda x: x["stamp"].update(viewGeneration=3),
                       lambda x: x["stamp"].update(documentId=2),
                       lambda x: x["nativeStatus"].update(textureId=5),
                       lambda x: x["nativeStatus"].update(acceptedSerial=1)):
            value = stage(2)
            modify(value)
            with self.assertRaises(ValueError):
                qualification.validate_transition(stage(), value)
        value = stage(3)
        value.update(width=40, height=30)
        with self.assertRaises(ValueError):
            qualification.validate_transition(stage(2), value)
        value = stage(4)
        value["stamp"]["revision"] += 1
        with self.assertRaises(ValueError):
            qualification.validate_transition(stage(3), value)

    def test_cumulative_ownership_evidence_cannot_reset_mid_run(self):
        for key in qualification.MONOTONIC_COUNTERS:
            previous, current = stage(31), stage(32)
            previous["limits"][key] = 5
            current["limits"][key] = 4
            with self.subTest(key=key), self.assertRaises(ValueError):
                qualification.validate_transition(previous, current)

    def test_close_requires_real_unregister_and_zero_native_resources(self):
        for key in ("closed", "ownerFinished", "unregistered"):
            value = completion()
            value["closed"][key] = False
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.finish(value)
        for key in qualification.NATIVE_STATS:
            value = completion()
            value["stats"][key] = 1
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.finish(value)
        for key in ("producerOwnedFrames", "producerOwnedBytes", "importPendingFrames", "importPendingBytes",
                    "legacyRetainedFrames", "descriptorTickets", "descriptorCpuPendingTickets"):
            value = completion()
            value["closed"]["limits"][key] = 1
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.finish(value)

    def test_close_cannot_change_owner_lose_handoffs_or_overstate_qualification(self):
        for modify in (lambda x: x.update(nonce="foreign"), lambda x: x.update(publications=31),
                       lambda x: x.update(protocol=True),
                       lambda x: x.update(consumerGpuCompletionVerified=True),
                       lambda x: x.update(continuousPresentationSupported=False),
                       lambda x: x["closed"].update(textureId=5),
                       lambda x: x["closed"].update(acceptedSerial=33),
                       lambda x: x["closed"]["limits"].update(handedOffFrames=30)):
            value = completion()
            modify(value)
            with self.assertRaises(ValueError):
                self.finish(value)

    def test_cpu_rgba_reads_are_bounded_and_exact(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cpu.rgba"
            path.write_bytes(b"abcd")
            self.assertEqual(qualification.read_pixels(path, 4), b"abcd")
            for length in (0, 3, 5, 16 * 1024 * 1024 + 1, True):
                with self.subTest(length=length), self.assertRaises(ValueError):
                    qualification.read_pixels(path, length)

    @unittest.skipIf(sys.platform == "win32", "requires an unavailable genuine Windows desktop")
    def test_linux_entry_point_rejects_before_creating_output(self):
        result = subprocess.run([sys.executable, "-B", str(Path(qualification.__file__)),
                                 *[argument for name in ("runner", "native", "dispatcher", "fixture",
                                                          "build-receipt", "output")
                                   for argument in ("--" + name, "must-not-exist")]],
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 2)
        self.assertIn("actual Windows desktop is required", result.stderr)

    def test_wine_is_rejected_by_the_shared_real_nt_guard(self):
        class Wine:
            wine_get_version = object()
        with self.assertRaisesRegex(RuntimeError, "Wine"):
            qualification.require_actual_windows(platform="win32", load_library=lambda *a, **k: Wine())


class WindowsContinuousDeadlineContracts(unittest.TestCase):
    """Fake process/capture orchestration, never actual Windows evidence."""

    def run_capture(self, expire=None):
        from desktop_qualification_contract import (BuildAndRuntimeIdentityContracts,
                                                    RuntimeBundleContracts, source_images)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runtime = root / "runtime"
            runtime.mkdir()
            RuntimeBundleContracts.bundle(self, runtime)
            _, supplied, build = BuildAndRuntimeIdentityContracts.artifacts(self, runtime)
            from qualify_desktop_texture import runtime_bundle_manifest
            build.update(platform="windows", buildExitCode=0, runtimeBundle=runtime_bundle_manifest(runtime),
                         sourceHead="1" * 40, nativeSourceHead="2" * 40,
                         flutterRevision="3" * 40, engineRevision="4" * 40)
            build_receipt = root / "build.json"
            qualification.write_json(build_receipt, build)
            output = root / "capture"
            clock = SimpleNamespace(now=0.0, reads=0, bundle_checks=0)
            case = SimpleNamespace(captures=0, config=None)
            process = SimpleNamespace(pid=42, returncode=None)
            process.poll = lambda: process.returncode
            process.kill = lambda: setattr(process, "returncode", -9)
            frames = []

            def wait(**kwargs):
                process.returncode = 0
                if expire == "close-wait":
                    clock.now = 31.0
                return 0

            process.wait = wait

            def launch(*args, **kwargs):
                case.config = qualification.read_json(kwargs["env"]["RHWP_CONSUMER_PROBE_CONFIG"])
                for index in range(1, 33):
                    value = stage(index)
                    value["nonce"] = case.config["nonce"]
                    qualification.write_json(output / f"stage-{index}.json", value)
                    low, reference, _ = source_images(index, width=value["width"], height=value["height"])
                    low, reference = bytearray(low), bytearray(reference)
                    # Keep every publication independently observable after the
                    # resize. Both CPU images come from the same synthetic shape.
                    for pixels, width, factor in ((low, value["width"], 1),
                                                   (reference, value["width"] * 4, 4)):
                        for y in range(8 * factor, 14 * factor):
                            for x in range(8 * factor, 14 * factor):
                                at = (y * width + x) * 4
                                pixels[at:at + 4] = bytes([20 + index] * 3 + [255])
                    frames.append((value["width"], value["height"], bytes(low)))
                    (output / f"cpu-{index}.rgba").write_bytes(low)
                    (output / f"cpu-reference-{index}.rgba").write_bytes(reference)
                result = completion()
                result["nonce"] = case.config["nonce"]
                qualification.write_json(output / "result.json", result)
                return process

            def capture(width, height, offset):
                expected_width, expected_height, pixels = frames[case.captures // 2]
                self.assertEqual((width, height, offset), (expected_width, expected_height, (16, 16)))
                case.captures += 1
                if expire == "held-capture" and case.captures == 2:
                    clock.now = 31.0
                return pixels

            original_compare = qualification.compare_pixels
            original_bundle = qualification.verify_runtime_bundle

            def compare(*args, **kwargs):
                result = original_compare(*args, **kwargs)
                if expire == "comparison":
                    clock.now = 31.0
                return result

            def bundle(*args, **kwargs):
                result = original_bundle(*args, **kwargs)
                clock.bundle_checks += 1
                if expire == "final-identity" and clock.bundle_checks == 2:
                    clock.now = 31.0
                return result

            def monotonic():
                clock.reads += 1
                if expire == "ready-stage" and clock.reads > 1:
                    clock.now = 31.0
                return clock.now

            argv = [str(Path(qualification.__file__)),
                    *(argument for key, path in supplied.items() for argument in ("--" + key, str(path))),
                    "--build-receipt", str(build_receipt), "--output", str(output), "--timeout", "30"]
            with mock.patch.object(sys, "argv", argv), \
                    mock.patch.object(qualification, "require_actual_windows"), \
                    mock.patch.object(qualification.subprocess, "Popen", side_effect=launch), \
                    mock.patch.object(qualification, "WindowsCompositor", return_value=SimpleNamespace(capture=capture)), \
                    mock.patch.object(qualification, "compare_pixels", side_effect=compare), \
                    mock.patch.object(qualification, "verify_runtime_bundle", side_effect=bundle), \
                    mock.patch.object(qualification.time, "monotonic", side_effect=monotonic), \
                    mock.patch.object(qualification.time, "sleep"), mock.patch("builtins.print"):
                if expire is None:
                    qualification.main()
                else:
                    with self.assertRaisesRegex(TimeoutError, "deadline"):
                        qualification.main()
            receipt = qualification.read_json(output / "receipt.json", limit=1024 * 1024)
            if expire is None:
                self.assertEqual(receipt["qualification"], "windows-immutable-transfer-consumer-passed")
                self.assertEqual(len(list(output.glob("ack-*.json"))), 32)
            else:
                self.assertEqual(receipt["qualification"], "failed")
                if expire in ("ready-stage", "comparison", "held-capture"):
                    self.assertFalse((output / "ack-1.json").exists())
                if expire == "ready-stage":
                    self.assertEqual(case.captures, 0)
                elif expire in ("comparison", "held-capture"):
                    self.assertTrue((output / "compositor-1.rgba").exists())

    def test_on_time_complete_publications_still_qualify(self):
        self.run_capture()

    def test_expired_deadline_rejects_an_already_ready_stage_without_capture_or_ack(self):
        self.run_capture("ready-stage")

    def test_cpu_comparison_crossing_deadline_cannot_ack_a_publication(self):
        self.run_capture("comparison")

    def test_held_capture_crossing_deadline_cannot_ack_a_publication(self):
        self.run_capture("held-capture")

    def test_successful_close_wait_finishing_after_deadline_is_not_qualification(self):
        self.run_capture("close-wait")

    def test_final_runtime_identity_check_crossing_deadline_is_not_qualification(self):
        self.run_capture("final-identity")


class WindowsSourcePixelControls(unittest.TestCase):
    def setUp(self):
        self.width = self.height = 20
        # Source shapes are independently specified. No GPU capture supplies
        # expected colors, placement, or the high-resolution reference palette.
        self.low = bytearray([255] * (20 * 20 * 4))
        self.reference = bytearray([255] * (80 * 80 * 4))
        for y in range(8, 15):
            for x in range(10, 16):
                self.put(self.low, 20, x, y, (40, 40, 40, 255))
        for y in range(32, 60):
            for x in range(40, 64):
                self.put(self.reference, 80, x, y, (40, 40, 40, 255))
        for y in range(2, 16):
            self.put(self.low, 20, 3, y, (64, 64, 255, 255))
        for y in range(8, 64):
            for x in range(12, 15):
                self.put(self.reference, 80, x, y, (0, 0, 255, 255))
        self.actual = bytearray(self.low)
        for y in range(2, 16):
            self.put(self.actual, 20, 3, y, (32, 32, 255, 255))
        self.low = bytes(self.low)
        self.reference = bytes(self.reference)

    @staticmethod
    def put(data, width, x, y, color):
        start = (y * width + x) * 4
        data[start:start + 4] = bytes(color)

    def compare(self, candidate):
        return qualification.compare_pixels(self.low, candidate, 20, 20,
                                            source_reference=self.reference, reference_scale=4)

    def test_independent_cpu_palette_supports_valid_aa_and_zero_alignment(self):
        result = self.compare(self.actual)
        self.assertIn("palette13", result["comparison"])
        self.assertLess(result["integerAlignmentErrors"]["0,0"],
                        min(value for key, value in result["integerAlignmentErrors"].items() if key != "0,0"))

    def test_one_and_two_pixel_shift_stale_content_and_blank_are_rejected(self):
        candidates = [bytes([255] * len(self.actual))]
        for dx in (1, 2):
            candidate = bytearray([255] * len(self.actual))
            for y in range(20):
                start = y * 80
                candidate[start + dx * 4:start + 80] = self.actual[start:start + (20 - dx) * 4]
            candidates.append(candidate)
        stale = bytearray(self.actual)
        # The previous revision had the opaque object in a different vertical
        # position, matching the real fixture's independent edit observation.
        for y in range(8, 15):
            for x in range(10, 16):
                self.put(stale, 20, x, y, (255, 255, 255, 255))
        for y in range(2, 9):
            for x in range(10, 16):
                self.put(stale, 20, x, y, (40, 40, 40, 255))
        candidates.append(stale)
        for index, candidate in enumerate(candidates):
            with self.subTest(candidate=index), self.assertRaises(ValueError):
                self.compare(candidate)

    def test_wrong_hue_uniform_color_and_outside_edge_halo_are_rejected(self):
        for x, y, color in ((3, 6, (255, 0, 255, 255)), (3, 6, (0, 255, 255, 255)),
                            (12, 11, (42, 40, 40, 255)), (1, 1, (0, 0, 255, 255))):
            candidate = bytearray(self.actual)
            self.put(candidate, 20, x, y, color)
            with self.subTest(color=color), self.assertRaises(ValueError):
                self.compare(candidate)

    def test_reference_unavailable_fails_closed(self):
        with self.assertRaisesRegex(ValueError, "referenceUnavailable"):
            qualification.compare_pixels(self.low, self.actual, 20, 20,
                                         source_reference=b"", reference_scale=4)


if __name__ == "__main__":
    unittest.main()
