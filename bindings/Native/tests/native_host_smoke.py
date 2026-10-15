#!/usr/bin/env python3
"""Reference-host smoke, or focused host-I/O checks without loading a library."""

import argparse
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


EXAMPLE = Path(__file__).resolve().parents[1] / "examples" / "native_host.py"
SPEC = importlib.util.spec_from_file_location("rhwp_native_host_example", EXAMPLE)
native_host = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(native_host)


class SaveHost:
    """Only the save checkpoint protocol is mocked, not rendering or editing."""

    def __init__(self, output, acknowledge_error=False):
        self.output = output
        self.events = []
        self.acknowledge_error = acknowledge_error

    def request(self, handle, **request):
        operation = request["op"]
        self.events.append(operation)
        if operation == "prepareSave":
            return {"token": 7, "buffer": 11, "byteLength": 7, "revision": 2, "report": None}
        if operation == "acknowledgeSave":
            assert self.output.read_bytes() == b"payload", "acknowledgement preceded disk write"
            if self.acknowledge_error:
                raise RuntimeError("injected acknowledge failure")
            return {"isDirty": False}
        if operation == "cancelSave":
            return {"isDirty": True}
        raise AssertionError(operation)

    def copy_buffer(self, response):
        self.events.append("copy-and-release")
        return b"payload"


class HostIoContracts(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="rhwp-native-host-io-")
        self.addCleanup(self.directory.cleanup)
        self.output = Path(self.directory.name) / "output.hwpx"

    def test_acknowledges_only_after_exclusive_write(self):
        host = SaveHost(self.output)
        result = native_host.persist_new_document(host, 1, self.output)
        self.assertFalse(result["isDirty"])
        self.assertEqual(host.events, ["prepareSave", "copy-and-release", "acknowledgeSave"])
        self.assertEqual(self.output.read_bytes(), b"payload")

    def test_existing_destination_is_preserved_and_save_is_cancelled(self):
        self.output.write_bytes(b"original")
        host = SaveHost(self.output)
        with self.assertRaises(FileExistsError):
            native_host.persist_new_document(host, 1, self.output)
        self.assertEqual(self.output.read_bytes(), b"original")
        self.assertEqual(host.events, ["prepareSave", "copy-and-release", "cancelSave"])

    def test_symlink_destination_does_not_overwrite_its_target(self):
        source = Path(self.directory.name) / "source.hwp"
        source.write_bytes(b"original source")
        try:
            self.output.symlink_to(source)
        except (OSError, NotImplementedError) as error:
            self.skipTest(f"symlinks unavailable: {error}")
        with self.assertRaises(FileExistsError):
            native_host.write_new_file(self.output, b"replacement")
        self.assertEqual(source.read_bytes(), b"original source")
        self.assertTrue(self.output.is_symlink())

    def test_failed_fsync_removes_only_new_partial_output_and_cancels_save(self):
        host = SaveHost(self.output)
        with patch.object(native_host.os, "fsync", side_effect=OSError("injected disk error")):
            with self.assertRaisesRegex(OSError, "injected disk error"):
                native_host.persist_new_document(host, 1, self.output)
        self.assertFalse(self.output.exists())
        self.assertEqual(host.events, ["prepareSave", "copy-and-release", "cancelSave"])

    def test_short_writes_are_completed_before_success(self):
        real_write = os.write
        with patch.object(native_host.os, "write", side_effect=lambda fd, data: real_write(fd, data[:2])):
            native_host.write_new_file(self.output, b"payload")
        self.assertEqual(self.output.read_bytes(), b"payload")

    def test_write_failure_preserves_a_different_file_now_at_the_output_path(self):
        original = Path(self.directory.name) / "moved-partial.hwpx"

        def replaced_while_writing(_descriptor):
            self.output.rename(original)
            self.output.write_bytes(b"different file")
            raise OSError("injected replacement race")

        with patch.object(native_host.os, "fsync", side_effect=replaced_while_writing):
            with self.assertRaisesRegex(OSError, "injected replacement race"):
                native_host.write_new_file(self.output, b"payload")
        self.assertEqual(self.output.read_bytes(), b"different file")
        self.assertEqual(original.read_bytes(), b"payload")

    def test_copy_failure_cancels_checkpoint_without_creating_output(self):
        host = SaveHost(self.output)
        with patch.object(host, "copy_buffer", side_effect=RuntimeError("injected copy failure")):
            with self.assertRaisesRegex(RuntimeError, "injected copy failure"):
                native_host.persist_new_document(host, 1, self.output)
        self.assertEqual(host.events, ["prepareSave", "cancelSave"])
        self.assertFalse(self.output.exists())

    def test_failed_acknowledgement_preserves_successfully_written_output(self):
        host = SaveHost(self.output, acknowledge_error=True)
        with self.assertRaisesRegex(RuntimeError, "injected acknowledge failure"):
            native_host.persist_new_document(host, 1, self.output)
        self.assertEqual(self.output.read_bytes(), b"payload")
        self.assertEqual(host.events[-2:], ["acknowledgeSave", "cancelSave"])

    def test_buffer_is_released_when_native_copy_fails(self):
        host = object.__new__(native_host.NativeHost)
        operations = []

        def call(operation, *arguments):
            operations.append(operation)
            if operation == "buffer_copy":
                raise RuntimeError("injected native copy failure")
            return {"ok": True}

        host.call = call
        with self.assertRaisesRegex(RuntimeError, "injected native copy failure"):
            host.copy_buffer({"buffer": 11, "byteLength": 4})
        self.assertEqual(operations, ["buffer_copy", "buffer_release"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path)
    parser.add_argument("--mock", action="store_true", help="run only host-I/O contracts; no native library needed")
    args = parser.parse_args()
    if args.mock:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(HostIoContracts)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1
    if args.library is None:
        parser.error("--library is required unless --mock is selected")
    host = native_host.NativeHost(args.library)
    with tempfile.TemporaryDirectory(prefix="rhwp-native-host-smoke-") as directory:
        output = Path(directory) / "edited.hwpx"
        edited = native_host.run_demo(host, output)
        original = output.read_bytes()
        copied = native_host.run_demo(host, Path(directory) / "copy.hwpx", input_path=output)
        if output.read_bytes() != original or copied["mode"] != "input-viewer":
            raise AssertionError("input viewer flow changed its source document")
        if host.read_text(Path(directory) / "copy.hwpx").strip() != "각":
            raise AssertionError("viewer save copy lost committed text")
    print(json.dumps({"ok": True, "contract": "native-reference-host",
                      "blankIme": edited["imeRoundtripVerified"], "inputPreserved": True}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
