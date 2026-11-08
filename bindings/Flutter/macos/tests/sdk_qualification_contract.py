#!/usr/bin/env python3
"""Portable rejection tests; no Apple SDK or Metal device is executed here."""

import base64
import importlib.util
import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import time
import unittest
from unittest import mock

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
    def test_cxx_driver_invocation_preserves_its_symlink_name(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            compiler = directory / "clang"
            compiler.write_text("portable path control; never executed")
            alias = directory / "clang++"
            alias.symlink_to(compiler.name)
            with mock.patch.object(builder.subprocess, "check_output", return_value=str(alias) + "\n"):
                selected = builder.find_apple_cxx()
            self.assertEqual(selected, alias.absolute())
            self.assertNotEqual(selected, compiler.resolve())

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
        cargo = next(command for command in commands if "cargo" in command)
        self.assertEqual(cargo[cargo.index("--target-dir") + 1], "/private/probe/native-target")
        self.assertEqual(cargo[cargo.index("--features") + 1], "native-skia-metal")
        self.assertEqual(artifacts["native"], Path("/private/probe/native-target/aarch64-apple-darwin/debug/librhwp_native_ffi.dylib"))
        compilers = [command for command in commands if command[0] == "/Xcode/clang++"]
        self.assertEqual(len(compilers), 4)
        self.assertLess(max(commands.index(command) for command in compilers), commands.index(cargo))
        cv_status = next(command for command in compilers if any(value.endswith("cv_closed_status_sdk_test.mm") for value in command))
        self.assertNotIn("-dynamiclib", cv_status)
        self.assertIn("-framework", cv_status)
        self.assertLess(commands.index([str(artifacts["cvCloseStatus"])]), commands.index(cargo))
        animation = next(command for command in compilers if any(value.endswith("metal_view_animation_sdk_test.mm") for value in command))
        self.assertNotIn("-dynamiclib", animation)
        self.assertLess(commands.index(animation), commands.index([str(artifacts["metalViewAnimationPolicy"])]))
        self.assertLess(commands.index([str(artifacts["metalViewAnimationPolicy"])]), commands.index(cargo))
        for command in compilers:
            self.assertIn("-fobjc-arc", command)
            self.assertIn("-fblocks", command)
            self.assertIn("-Werror", command)

    def test_unknown_architecture_and_toolchain_shell_text_are_rejected(self):
        for architecture, toolchain in [("riscv64", "1.93.1"), ("arm64", "nightly; shell")]:
            with self.assertRaises(ValueError):
                builder.command_plan(Path("/source"), Path("/probe"), Path("/SDK"),
                                     Path("/clang"), Path("/Flutter.framework"), architecture, toolchain)

    def test_verified_native_reuse_still_compiles_plugin_and_runs_producer(self):
        native = Path("/verified/librhwp_native_ffi.dylib")
        commands, artifacts = builder.command_plan(
            Path("/source"), Path("/probe"), Path("/SDK"), Path("/Xcode/clang++"),
            Path("/FlutterMacOS.framework"), "arm64", "1.93.1", native)
        self.assertEqual(artifacts["native"], native)
        self.assertFalse(any("cargo" in command for command in commands))
        self.assertEqual(sum(command[0] == "/Xcode/clang++" for command in commands), 4)
        self.assertIn([str(artifacts["cvCloseStatus"])], commands)
        self.assertIn([str(artifacts["metalViewAnimationPolicy"])], commands)
        producer = commands[-1]
        self.assertIn(str(native), producer)
        self.assertIn("--native-source-head", producer)
        self.assertIn("NATIVE_SOURCE_HEAD", producer)


class SourceAndProcessContracts(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="rhwp-sdk-source-contract-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repository = self.root / "repository"
        self.repository.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "SDK contract")
        self.git("config", "user.email", "sdk-contract@example.invalid")
        for path, data in {
            "Cargo.toml": "[workspace]\nmembers=[]\n",
            "vendor/dependency/src/lib.rs": "pub fn original() {}\n",
            "tools/member/Cargo.toml": "[package]\nname='member'\n",
            "assets/fonts/embedded.bin": "embedded font bytes",
            "ttfs/embedded.ttf": "another embedded font",
            "saved/blank2010.hwp": "embedded template",
            "mydocs/compiled.txt": "compile-time documentation",
        }.items():
            target = self.repository / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(data)
        self.git("add", ".")
        self.git("commit", "-qm", "fixture")

    def git(self, *args):
        return subprocess.check_output(["git", "-c", "gc.auto=0", *args], cwd=self.repository)

    def test_complete_tracked_identity_includes_vendor_workspace_and_embedded_inputs(self):
        actual = builder.source_hashes(self.repository)
        for path in ("vendor/dependency/src/lib.rs", "tools/member/Cargo.toml",
                     "assets/fonts/embedded.bin", "ttfs/embedded.ttf", "saved/blank2010.hwp",
                     "mydocs/compiled.txt"):
            self.assertIn(path, actual)
            self.assertRegex(actual[path], r"^100644:[0-9a-f]{40}$")
        self.assertEqual(actual, builder.source_hashes(self.repository))

    def test_dirty_vendor_and_untracked_source_are_rejected_then_new_commit_changes_identity(self):
        original = builder.source_hashes(self.repository)
        vendor = self.repository / "vendor/dependency/src/lib.rs"
        vendor.write_text("pub fn changed() {}\n")
        with self.assertRaisesRegex(ValueError, "tracked build inputs"):
            builder.source_hashes(self.repository)
        self.git("add", ".")
        self.git("commit", "-qm", "changed vendor")
        self.assertNotEqual(original, builder.source_hashes(self.repository))
        (vendor.parent / "untracked.rs").write_text("pub fn untracked() {}\n")
        with self.assertRaisesRegex(ValueError, "untracked source"):
            builder.source_hashes(self.repository)

    def test_explicit_rust_architecture_matches_bridge_architecture_and_artifact(self):
        builds = []
        for architecture, target in [("arm64", "aarch64-apple-darwin"),
                                     ("x86_64", "x86_64-apple-darwin")]:
            commands, artifacts = builder.command_plan(
                self.repository, self.root, Path("/SDK"), Path("/clang"),
                Path("/Flutter/FlutterMacOS.framework"), architecture, "1.93.1")
            cargo = next(command for command in commands if "cargo" in command)
            self.assertEqual(cargo[cargo.index("--target") + 1], target)
            self.assertEqual(commands[2][commands[2].index("-arch") + 1], architecture)
            self.assertEqual(artifacts["native"], self.root / "native-target" / target /
                             "debug/librhwp_native_ffi.dylib")
            builds.append(cargo)
        self.assertNotEqual(*builds)

    @unittest.skipUnless(os.name == "posix", "Apple SDK runner uses POSIX process groups")
    def test_step_records_success_exit_and_private_log(self):
        step = {"exitCode": None}
        receipt = self.root / "receipt.json"
        state = {"steps": [step]}
        record = lambda: builder.write_json(receipt, state)
        log = self.root / "step.log"
        result = builder.execute_step([sys.executable, "-c", "print('portable child')"],
                                      repository=self.repository, env=os.environ.copy(), log_path=log,
                                      step=step, record=record, timeout=5)
        self.assertEqual(result, 0)
        self.assertEqual(json.loads(receipt.read_text())["steps"][0]["exitCode"], 0)
        self.assertIn("portable child", log.read_text())
        self.assertEqual(log.stat().st_mode & 0o777, 0o600)

    @unittest.skipUnless(os.name == "posix", "Apple SDK runner uses POSIX process groups")
    def test_timeout_records_failure_and_stops_parent_and_descendant(self):
        child_pid = self.root / "child.pid"
        child_code = "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)"
        code = ("import subprocess,sys,time,pathlib; "
                "p=subprocess.Popen([sys.executable,'-c',sys.argv[2]]); "
                "pathlib.Path(sys.argv[1]).write_text(str(p.pid)); time.sleep(60)")
        step = {"exitCode": None}
        receipt = self.root / "receipt.json"
        state = {"build": "pending", "steps": [step]}
        record = lambda: builder.write_json(receipt, state)
        started = time.monotonic()
        try:
            with self.assertRaises(TimeoutError):
                builder.execute_step([sys.executable, "-c", code, str(child_pid), child_code],
                                     repository=self.repository, env=os.environ.copy(),
                                     log_path=self.root / "timeout.log", step=step, record=record,
                                     timeout=.8, stop_grace=.1)
        finally:
            state["build"] = "failed"
            record()
        self.assertLess(time.monotonic() - started, 5)
        self.assertTrue(step["timedOut"])
        self.assertIsInstance(step["exitCode"], int)
        self.assertNotEqual(step["exitCode"], 0)
        self.assertEqual(json.loads(receipt.read_text())["build"], "failed")
        descendant = int(child_pid.read_text())
        try:
            status = subprocess.check_output(["ps", "-o", "stat=", "-p", str(descendant)], text=True).strip()
        except subprocess.CalledProcessError:
            status = ""
        self.assertTrue(not status or status.startswith("Z"), "owned descendant survived the timeout")

    @unittest.skipUnless(os.name == "posix", "Apple SDK runner uses POSIX process groups")
    def test_record_failure_still_stops_owned_process(self):
        step = {"exitCode": None}
        attempts = []
        def record():
            attempts.append(dict(step))
            if len(attempts) == 1:
                raise OSError("injected receipt write failure")
        with self.assertRaisesRegex(OSError, "receipt write failure"):
            builder.execute_step([sys.executable, "-c", "import time; time.sleep(60)"],
                                 repository=self.repository, env=os.environ.copy(),
                                 log_path=self.root / "interrupted.log", step=step, record=record,
                                 timeout=5, stop_grace=.1)
        self.assertTrue(step["interrupted"])
        self.assertIsInstance(step["exitCode"], int)
        self.assertGreaterEqual(len(attempts), 2)
        with self.assertRaises(ProcessLookupError):
            os.kill(step["pid"], 0)

    def test_nonfinite_or_excessive_deadlines_reject_before_spawning(self):
        for timeout in (float("nan"), float("inf"), 0, -1, 7201):
            with self.assertRaises(ValueError):
                builder.execute_step(["must-not-start"], repository=self.repository,
                                     env=os.environ.copy(), log_path=self.root / "invalid.log",
                                     step={}, record=lambda: None, timeout=timeout)
        self.assertFalse((self.root / "invalid.log").exists())


if __name__ == "__main__":
    unittest.main()
