#!/usr/bin/env python3
"""Portable rejection controls for the Mac qualifier; never target execution."""

import importlib.util
from pathlib import Path
import plistlib
import tempfile
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


if __name__ == "__main__":
    unittest.main()
