#!/usr/bin/env python3
"""Portable rejection tests; no Apple SDK or Metal device is executed here."""

import base64
import importlib.util
from pathlib import Path
import sys
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tool"))


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


producer = module("producer_contract", HERE / "native_metal_sdk_contract.py")
builder = module("sdk_builder", HERE.parents[1] / "tool/check_macos_metal_sdk.py")
STAMP = {"documentId": 1, "revision": 0, "fontGeneration": 0,
         "viewGeneration": 1, "deviceGeneration": 1, "deviceScale": 1.0}


def response(*, stale=False):
    native = {"ok": False, "code": "staleFrame", "unpublishedTargetSafeToDiscard": True} if stale else {
        "ok": True, "frame": {"backend": "metalGanesh", "stamp": dict(STAMP),
                              "producerGpuComplete": True, "consumerGpuComplete": False,
                              "wholeFrameReadbacks": 0}}
    pixel = bytes([0x5a]) * 4 if stale else bytes([0x30, 0x20, 0x10, 0xff])
    return {"ok": True, "consumerGpuCompletionVerified": False,
            "width": 32, "height": 24, "hostVerificationReadbacks": 1,
            "native": native, "bgraBase64": base64.b64encode(pixel * (32 * 24)).decode()}


class MetalReceiptContracts(unittest.TestCase):
    def test_independent_opaque_and_premultiplied_color_controls(self):
        self.assertEqual(producer.expected_bgra(0xff102030), (0x30, 0x20, 0x10, 0xff))
        self.assertEqual(producer.expected_bgra(0x80402010), (8, 16, 32, 128))
        self.assertEqual(producer.expected_bgra(0), (0, 0, 0, 0))

    def test_complete_bounded_producer_receipt_is_admitted(self):
        checks = producer.validate_response(response(), STAMP, 0xff102030)
        self.assertEqual(checks["maxChannelDifference"], 0)

    def test_wrong_color_or_byte_length_does_not_qualify(self):
        for data in [bytes([0, 0, 0, 255]) * 768, b"short"]:
            value = response()
            value["bgraBase64"] = base64.b64encode(data).decode()
            with self.assertRaises(ValueError):
                producer.validate_response(value, STAMP, 0xff102030)

    def test_wrong_stamp_or_backend_is_rejected(self):
        for key, value in [("stamp", dict(STAMP, revision=1)), ("backend", "cpu")]:
            item = response()
            item["native"]["frame"][key] = value
            with self.assertRaises(ValueError):
                producer.validate_response(item, STAMP, 0xff102030)

    def test_producer_completion_does_not_prove_consumer_completion(self):
        for modify in [lambda item: item.update(consumerGpuCompletionVerified=True),
                       lambda item: item["native"]["frame"].update(producerGpuComplete=False),
                       lambda item: item["native"]["frame"].update(consumerGpuComplete=True),
                       lambda item: item["native"]["frame"].update(wholeFrameReadbacks=1)]:
            item = response()
            modify(item)
            with self.assertRaises(ValueError):
                producer.validate_response(item, STAMP, 0xff102030)

    def test_unpublished_stale_target_must_remain_unmodified(self):
        checks = producer.validate_response(response(stale=True), STAMP, 0xff102030, stale=True)
        self.assertTrue(checks["targetPreserved"])
        item = response(stale=True)
        item["bgraBase64"] = response()["bgraBase64"]
        with self.assertRaises(ValueError):
            producer.validate_response(item, STAMP, 0xff102030, stale=True)

    def test_an_arbitrary_error_is_not_safe_stale_retirement(self):
        for modify in [lambda item: item["native"].update(code="deviceLost"),
                       lambda item: item["native"].update(unpublishedTargetSafeToDiscard=False),
                       lambda item: item.update(ok=False)]:
            item = response(stale=True)
            modify(item)
            with self.assertRaises(ValueError):
                producer.validate_response(item, STAMP, 0xff102030, stale=True)

    def test_non_apple_host_is_explicitly_unavailable(self):
        for system in ("linux", "win32", "freebsd"):
            with self.assertRaisesRegex(RuntimeError, "actual macOS"):
                builder.platform_required(system)

    def test_target_output_is_isolated_and_feature_is_explicit(self):
        commands, artifacts = builder.command_plan(
            Path("/source"), Path("/private/probe"), Path("/SDK/MacOSX.sdk"),
            Path("/Xcode/clang++"), Path("/Flutter/FlutterMacOS.framework"), "arm64", "1.93.1")
        cargo = commands[1]
        self.assertEqual(cargo[cargo.index("--target-dir") + 1], "/private/probe/native-target")
        self.assertEqual(cargo[cargo.index("--features") + 1], "native-skia-metal")
        self.assertEqual(artifacts["native"], Path("/private/probe/native-target/debug/librhwp_native_ffi.dylib"))
        for command in commands[2:4]:
            self.assertIn("-fobjc-arc", command)
            self.assertIn("-fblocks", command)
            self.assertIn("-Werror", command)

    def test_unknown_architecture_and_toolchain_shell_text_are_rejected(self):
        for architecture, toolchain in [("riscv64", "1.93.1"), ("arm64", "nightly; shell")]:
            with self.assertRaises(ValueError):
                builder.command_plan(Path("/source"), Path("/probe"), Path("/SDK"),
                                     Path("/clang"), Path("/Flutter.framework"), architecture, toolchain)


if __name__ == "__main__":
    unittest.main()
