#!/usr/bin/env python3
"""File publication, conflict and recovery contracts without a native library."""

import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch


MODULE = Path(__file__).resolve().parents[1] / "examples" / "native_file_store.py"
SPEC = importlib.util.spec_from_file_location("rhwp_native_file_store", MODULE)
store = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = store
SPEC.loader.exec_module(store)


class Host:
    def __init__(self):
        self.events = []
        self.revision = 3
        self.dirty = True
        self.payload = b"new document bytes"
        self.report = {"count": 0, "losses": []}
        self.ack_error = False
        self.on_ack = lambda: None

    def request(self, handle, **request):
        operation = request["op"]
        self.events.append(operation)
        if operation == "info":
            return {"revision": self.revision, "isDirty": self.dirty}
        if operation == "prepareSave":
            return {"token": 7, "buffer": 11, "revision": self.revision, "report": self.report,
                    "caretSaved": False, "caretPreservedFromSource": False}
        if operation == "acknowledgeSave":
            self.on_ack()
            if self.ack_error:
                raise RuntimeError("acknowledgement failed")
            self.dirty = False
            return {"isDirty": False}
        if operation == "cancelSave":
            return {"isDirty": self.dirty}
        raise AssertionError(request)

    def copy_buffer(self, prepared):
        self.events.append("copy-and-release")
        return self.payload


class FileStoreContracts(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="rhwp-file-contract-")
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "document.hwpx"
        self.host = Host()

    def opened(self):
        self.path.write_bytes(b"original")
        _, stamp = store.read_document(self.path)
        return store.NativeFileStore(self.host, 1, self.path, stamp)

    def test_new_save_is_published_before_ack(self):
        self.host.on_ack = lambda: self.assertEqual(self.path.read_bytes(), self.host.payload)
        result = store.NativeFileStore(self.host, 1).save(self.path)
        self.assertFalse(result["isDirty"])
        self.assertEqual(self.host.events, ["prepareSave", "copy-and-release", "acknowledgeSave"])

    def test_replace_checks_the_opened_source_and_refreshes_stamp(self):
        adapter = self.opened()
        adapter.save()
        self.host.payload = b"next revision"
        self.host.dirty = True
        adapter.save()
        self.assertEqual(self.path.read_bytes(), b"next revision")

    def test_save_as_never_overwrites_an_existing_other_file(self):
        self.path.write_bytes(b"other document")
        with self.assertRaises(FileExistsError):
            store.NativeFileStore(self.host, 1).save(self.path)
        self.assertEqual(self.path.read_bytes(), b"other document")
        self.assertNotIn("prepareSave", self.host.events)

    def test_external_change_prevents_replacement(self):
        adapter = self.opened()
        self.path.write_bytes(b"external edit")
        with self.assertRaises(store.ExternalFileChanged):
            adapter.save()
        self.assertEqual(self.path.read_bytes(), b"external edit")
        self.assertNotIn("acknowledgeSave", self.host.events)

    def test_temp_flush_failure_preserves_original_and_dirty(self):
        adapter = self.opened()
        with patch.object(store.os, "fsync", side_effect=OSError("disk error")):
            with self.assertRaises(OSError):
                adapter.save()
        self.assertEqual(self.path.read_bytes(), b"original")
        self.assertTrue(self.host.dirty)
        self.assertEqual(self.host.events[-1], "cancelSave")
        self.assertFalse(list(self.path.parent.glob(".rhwp-save-*")))

    def test_directory_flush_failure_retains_published_file_without_ack(self):
        adapter = self.opened()
        original_stamp = adapter.source_stamp
        with patch.object(store, "_sync_directory", side_effect=OSError("directory flush error")):
            with self.assertRaises(store.PublicationError) as raised:
                adapter.save()
        self.assertTrue(raised.exception.published)
        self.assertEqual(raised.exception.path, self.path)
        self.assertIsInstance(raised.exception.__cause__, OSError)
        self.assertEqual(adapter.source_stamp, original_stamp)
        self.assertEqual(self.path.read_bytes(), self.host.payload)
        self.assertTrue(self.host.dirty)
        self.assertNotIn("acknowledgeSave", self.host.events)
        self.assertEqual(self.host.events[-1], "cancelSave")
        with self.assertRaises(store.ExternalFileChanged):
            adapter.save()

    def test_ack_failure_preserves_complete_output(self):
        self.host.ack_error = True
        with self.assertRaises(store.PublicationError) as raised:
            store.NativeFileStore(self.host, 1).save(self.path)
        self.assertTrue(raised.exception.published)
        self.assertEqual(raised.exception.path, self.path)
        self.assertIsInstance(raised.exception.__cause__, RuntimeError)
        self.assertEqual(self.path.read_bytes(), self.host.payload)
        self.assertTrue(self.host.dirty)

    def test_new_file_unlink_failure_reports_preserved_publication(self):
        original_unlink = Path.unlink
        failed = False

        def fail_first_temporary_unlink(path, *arguments, **keywords):
            nonlocal failed
            if path.name.startswith(".rhwp-save-") and not failed:
                failed = True
                raise OSError("temporary unlink failed after exclusive link")
            return original_unlink(path, *arguments, **keywords)

        with patch.object(Path, "unlink", fail_first_temporary_unlink):
            with self.assertRaises(store.PublicationError) as raised:
                store.NativeFileStore(self.host, 1).save(self.path)
        self.assertTrue(raised.exception.published)
        self.assertEqual(raised.exception.path, self.path)
        self.assertEqual(self.path.read_bytes(), self.host.payload)
        self.assertTrue(self.host.dirty)
        self.assertNotIn("acknowledgeSave", self.host.events)
        self.assertEqual(self.host.events[-1], "cancelSave")
        self.assertFalse(list(self.path.parent.glob(".rhwp-save-*")))

    def test_autosave_publication_failure_retains_recovery_without_checkpoint(self):
        adapter = store.NativeFileStore(self.host, 1)
        with patch.object(store, "_sync_directory", side_effect=OSError("directory flush error")):
            with self.assertRaises(store.PublicationError) as raised:
                adapter.autosave(self.directory.name, now=0)
        self.assertTrue(raised.exception.published)
        _, data = store.read_recovery(raised.exception.path)
        self.assertEqual(data, self.host.payload)
        self.assertTrue(self.host.dirty)
        self.assertNotIn("acknowledgeSave", self.host.events)
        self.assertEqual(self.host.events[-1], "cancelSave")
        self.assertIsNone(adapter._autosave_revision)
        self.assertIsNone(adapter._autosave_time)

    def test_loss_report_requires_explicit_acceptance(self):
        adapter = self.opened()
        self.host.report = {"count": 1, "losses": [{"code": "resourceMissing"}]}
        with self.assertRaises(store.ContentLossNeedsAcceptance):
            adapter.save()
        self.assertEqual(self.path.read_bytes(), b"original")
        adapter.save(accept_loss=True)
        self.assertFalse(self.host.dirty)

    def test_symlink_destination_preserves_target(self):
        original = Path(self.directory.name) / "original.hwpx"
        original.write_bytes(b"external")
        self.path.symlink_to(original)
        with self.assertRaises(ValueError):
            store.NativeFileStore(self.host, 1).save(self.path)
        self.assertEqual(original.read_bytes(), b"external")

    def test_autosave_roundtrip_retains_dirty_and_obeys_revision_and_time(self):
        adapter = store.NativeFileStore(self.host, 1)
        directory = Path(self.directory.name) / "recovery"
        recovery = adapter.autosave(directory, now=100, interval=30)
        header, data = store.read_recovery(recovery)
        self.assertEqual(data, self.host.payload)
        self.assertEqual(header["revision"], 3)
        self.assertTrue(self.host.dirty)
        self.assertNotIn("acknowledgeSave", self.host.events)
        self.assertIsNone(adapter.autosave(directory, now=200, interval=30))
        self.host.revision = 4
        self.assertIsNone(adapter.autosave(directory, now=110, interval=30))
        self.assertIsNotNone(adapter.autosave(directory, now=131, interval=30))

    def test_corrupt_recovery_is_rejected(self):
        path = store.NativeFileStore(self.host, 1).autosave(self.directory.name, now=0)
        data = path.read_bytes()
        path.write_bytes(data[:-1] + bytes([data[-1] ^ 1]))
        with self.assertRaisesRegex(ValueError, "checksum"):
            store.read_recovery(path)

    def test_invalid_extension_and_clock_do_not_prepare_bytes(self):
        adapter = store.NativeFileStore(self.host, 1)
        with self.assertRaises(ValueError):
            adapter.save(self.path.with_suffix(".txt"))
        with self.assertRaises(ValueError):
            adapter.autosave(self.directory.name, now=float("nan"))
        self.assertNotIn("prepareSave", self.host.events)


if __name__ == "__main__":
    unittest.main(verbosity=2)
