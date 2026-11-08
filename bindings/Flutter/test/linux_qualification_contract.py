#!/usr/bin/env python3
"""Portable rejection controls; these are not actual GTK/GL execution evidence."""

from pathlib import Path
import os
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tool"))
from qualify_linux_texture import stage_name, validate_result, validate_stage, verify_loaded_artifacts


def stage(index=1):
    stamp = {"documentId": 1, "revision": 0, "fontGeneration": 0,
             "viewGeneration": index, "deviceGeneration": 1, "deviceScale": 1}
    return {"protocol": 1, "nonce": "owned", "pid": 42, "platform": "linux", "index": index,
            "stage": stage_name(index), "width": 320, "height": 240,
            "physicalOffset": [16, 16], "deviceScale": 1, "stamp": stamp,
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


if __name__ == "__main__":
    unittest.main()
