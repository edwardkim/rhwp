#!/usr/bin/env python3
"""Worker save stages, file publication, loss acceptance and stale completions."""

from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
import argparse
import json
from pathlib import Path
import sys
import tempfile
from threading import Event, get_ident
import unittest
from unittest.mock import patch


sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
import native_file_store as files
from native_worker_file_store import NativeWorkerFileStore


class WorkerFileStoreFixture:
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="rhwp-worker-file-")
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "document.hwpx"
        self.adapter = NativeWorkerFileStore(5)
        self.target = {"documentId": 5, "revision": 3}
        self.data = bytearray(b"prepared revision three")
        self.report = {"count": 0, "losses": []}
        self.releases = []

    def begin(self, path=None):
        return self.adapter.begin_save(self.target, self.path if path is None else path)

    def response(self, **changes):
        return {"ok": True, "kind": "preparedSave", "documentId": 5, "revision": 3,
                "token": 7, "buffer": 11, "byteLength": len(self.data),
                "report": self.report, "caretSaved": False,
                "caretPreservedFromSource": True, "caretPolicy": "hwpxSourceSettings",
                **changes}

    def copy_buffer(self, response):
        self.releases.append((response["buffer"], get_ident()))
        return self.data

    def prepared(self, operation=None, **changes):
        operation = operation or self.begin()
        operation.receive_prepared(self.response(**changes), self.copy_buffer)
        return operation

    def acknowledgement(self, current_revision=3, **changes):
        dirty = current_revision != 3
        return {"ok": True, "kind": "saveStatus", "documentId": 5,
                "isDirty": dirty,
                "status": {"revision": current_revision, "savedRevision": 3, "dirty": dirty},
                **changes}

    def opened(self):
        self.path.write_bytes(b"original")
        opened_bytes, stamp = files.read_document(self.path)
        self.assertEqual(stamp.digest, files.hashlib.sha256(opened_bytes).hexdigest())
        self.adapter = NativeWorkerFileStore(5, self.path, stamp)
        return stamp


class WorkerFileStoreContracts(WorkerFileStoreFixture, unittest.TestCase):
    def test_commands_and_bytes_are_bound_to_the_prepared_document(self):
        operation = self.begin()
        self.assertEqual(operation.prepare_command,
                         {"op": "prepareSave", "target": self.target, "format": "hwpx"})
        self.target["revision"] = 99
        command = operation.prepare_command
        command["target"]["documentId"] = 123
        self.assertEqual(operation.prepare_command["target"], {"documentId": 5, "revision": 3})
        self.prepared(operation)
        original = bytes(self.data)
        self.data[:] = b"changed"
        operation.publish()
        self.assertEqual(self.path.read_bytes(), original)
        self.assertEqual(operation.acknowledge_command,
                         {"op": "acknowledgeSave", "documentId": 5, "token": 7})
        result = operation.acknowledged(self.acknowledgement())
        self.assertEqual((result["revision"], result["currentRevision"]), (3, 3))
        self.assertFalse(result["isDirty"])
        self.assertEqual(self.adapter.source_path, self.path)
        self.assertFalse(self.adapter.busy)
        self.assertIsNone(operation.cancel_command)

    def test_save_identity_and_destination_cannot_change_during_loss_acceptance(self):
        operation = self.prepared(report={"count": 1, "losses": []})
        for name, value in (("document_id", 99), ("revision", 99),
                            ("path", self.path.with_name("other.hwpx")), ("format", "hwp")):
            with self.subTest(name=name):
                with self.assertRaises(AttributeError):
                    setattr(operation, name, value)
        with self.assertRaises(RuntimeError):
            _ = operation.prepare_command
        operation.accept_loss()
        operation.publish()
        self.assertEqual(self.path.read_bytes(), self.data)

    def test_hwp_format_is_explicit_and_invalid_extension_is_rejected(self):
        with self.assertRaises(ValueError):
            self.begin(self.path.with_suffix(".txt"))
        self.assertFalse(self.adapter.busy)
        operation = self.begin(self.path.with_suffix(".HWP"))
        self.assertEqual(operation.prepare_command["format"], "hwp")

    def test_source_path_and_the_stamp_of_its_opened_bytes_are_paired(self):
        with self.assertRaises(ValueError):
            NativeWorkerFileStore(5, self.path)
        with self.assertRaises(ValueError):
            NativeWorkerFileStore(5, source_stamp="later stat")
        with self.assertRaises(ValueError):
            NativeWorkerFileStore(5, self.path, object())
        with self.assertRaises(ValueError):
            self.adapter.begin_save(self.target)

    def test_save_rejects_another_document_without_marking_store_busy(self):
        for target in ({"documentId": 8, "revision": 3},
                       {"documentId": 5, "revision": -1},
                       {"documentId": 5, "revision": True}):
            with self.assertRaises(ValueError):
                self.adapter.begin_save(target, self.path)
            self.assertFalse(self.adapter.busy)

    def test_only_one_save_can_be_pending_until_native_cleanup_finishes(self):
        operation = self.prepared()
        with self.assertRaises(RuntimeError):
            self.begin()
        self.assertTrue(self.adapter.busy)
        operation.cancelled()
        self.assertFalse(self.adapter.busy)
        self.begin()

    def test_replace_refreshes_observed_stamp_only_after_ack(self):
        original_stamp = self.opened()
        operation = self.prepared(self.adapter.begin_save(self.target))
        operation.publish()
        self.assertEqual(self.adapter.source_stamp, original_stamp)
        operation.acknowledged(self.acknowledgement())
        self.assertNotEqual(self.adapter.source_stamp, original_stamp)
        operation = self.prepared(self.adapter.begin_save(self.target))
        operation.publish()
        operation.acknowledged(self.acknowledgement())

    def test_external_change_after_prepare_preserves_external_file(self):
        original_stamp = self.opened()
        operation = self.prepared(self.adapter.begin_save(self.target))
        self.path.write_bytes(b"external edit")
        with self.assertRaises(files.ExternalFileChanged):
            operation.publish()
        self.assertEqual(self.path.read_bytes(), b"external edit")
        self.assertEqual(self.adapter.source_stamp, original_stamp)
        self.assertEqual(operation.cancel_command,
                         {"op": "cancelSave", "documentId": 5, "token": 7})
        self.assertTrue(self.adapter.busy)
        operation.cancelled()
        self.assertFalse(self.adapter.busy)

    def test_other_existing_destination_requires_its_own_observed_stamp(self):
        self.opened()
        other = self.path.with_name("other.hwp")
        other.write_bytes(b"another document")
        operation = self.prepared(self.adapter.begin_save(self.target, other))
        with self.assertRaises(FileExistsError):
            operation.publish()
        self.assertEqual(other.read_bytes(), b"another document")
        operation.cancelled()
        _, destination_stamp = files.read_document(other)
        operation = self.prepared(self.adapter.begin_save(
            self.target, other, destination_stamp=destination_stamp))
        operation.publish()
        operation.acknowledged(self.acknowledgement())
        self.assertEqual(other.read_bytes(), self.data)
        self.assertEqual(self.path.read_bytes(), b"original")
        self.assertEqual(self.adapter.source_path, other)

    def test_source_cannot_adopt_an_unrelated_or_newer_destination_stamp(self):
        self.opened()
        self.path.write_bytes(b"external revision")
        _, later_stamp = files.read_document(self.path)
        with self.assertRaises(ValueError):
            self.adapter.begin_save(self.target, destination_stamp=later_stamp)
        self.assertFalse(self.adapter.busy)

    def test_loss_acceptance_keeps_the_report_bytes_and_token_of_one_prepare(self):
        self.report = {"count": 1, "losses": [{"kind": "unsupportedObject"}]}
        expected_report = deepcopy(self.report)
        response = self.response()
        operation = self.begin()
        operation.receive_prepared(response, self.copy_buffer)
        self.assertTrue(operation.requires_loss_acceptance)
        with self.assertRaises(files.ContentLossNeedsAcceptance) as raised:
            operation.publish()
        self.assertEqual(raised.exception.report, expected_report)
        self.assertFalse(self.path.exists())
        response["report"]["losses"][0]["kind"] = "changed after dialog"
        operation.report["count"] = 0
        expected_bytes = bytes(self.data)
        self.data[:] = b"newer document bytes"
        operation.accept_loss()
        operation.publish()
        self.assertEqual(self.path.read_bytes(), expected_bytes)
        result = operation.acknowledged(self.acknowledgement())
        self.assertEqual(result["report"], expected_report)
        self.assertEqual(len(self.releases), 1)

    def test_declined_loss_does_not_write_and_retains_token_for_cleanup(self):
        operation = self.prepared(report={"count": 2, "losses": []})
        self.assertTrue(operation.requires_loss_acceptance)
        self.assertEqual(operation.cancel_command["token"], 7)
        operation.cancelled()
        self.assertFalse(self.path.exists())
        with self.assertRaises(RuntimeError):
            operation.publish()

    def test_prepare_validation_always_releases_its_buffer(self):
        changes = ({"documentId": 99}, {"revision": 4}, {"token": 0},
                   {"byteLength": 1}, {"report": None}, {"report": {"count": -1}})
        for change in changes:
            with self.subTest(change=change):
                operation = self.begin()
                released = len(self.releases)
                with self.assertRaises((RuntimeError, ValueError)):
                    operation.receive_prepared(self.response(**change), self.copy_buffer)
                self.assertEqual(len(self.releases), released + 1)
                if change.get("documentId") == 99 or change.get("token") == 0:
                    self.assertIsNone(operation.cancel_command)
                operation.cancelled()

    def test_copy_failure_still_exposes_original_token_to_cancel(self):
        operation = self.begin()
        with self.assertRaises(OSError):
            operation.receive_prepared(self.response(), lambda _: (_ for _ in ()).throw(OSError("copy")))
        self.assertEqual(operation.cancel_command["token"], 7)
        operation.cancelled()

    def test_oversized_prepared_bytes_are_released_without_publication(self):
        operation = self.begin()
        with patch("native_worker_file_store.MAX_BYTES", 1):
            with self.assertRaises(ValueError):
                operation.receive_prepared(self.response(), self.copy_buffer)
        self.assertEqual(len(self.releases), 1)
        self.assertFalse(self.path.exists())
        self.assertEqual(operation.cancel_command["token"], 7)
        operation.cancelled()

    def test_ack_is_not_available_before_file_publication(self):
        operation = self.prepared()
        with self.assertRaises(RuntimeError):
            _ = operation.acknowledge_command
        with self.assertRaises(RuntimeError):
            operation.acknowledged(self.acknowledgement())

    def test_later_edit_stays_dirty_after_ack_of_the_saved_revision(self):
        operation = self.prepared()
        operation.publish()
        result = operation.acknowledged(self.acknowledgement(current_revision=4))
        self.assertEqual(result["revision"], 3)
        self.assertEqual(result["currentRevision"], 4)
        self.assertTrue(result["isDirty"])

    def test_bad_or_stale_ack_preserves_output_and_requires_separate_cleanup(self):
        for reply in (self.acknowledgement(documentId=99),
                      {"ok": False, "code": "staleDocument", "error": "old document closed"},
                      self.acknowledgement(status={"revision": 3, "savedRevision": 2, "dirty": False})):
            with self.subTest(reply=reply):
                self.path.unlink(missing_ok=True)
                adapter = NativeWorkerFileStore(5)
                operation = self.prepared(adapter.begin_save(self.target, self.path))
                operation.publish()
                with self.assertRaises(files.PublicationError) as raised:
                    operation.acknowledged(reply)
                self.assertEqual(raised.exception.path, self.path)
                self.assertTrue(raised.exception.published)
                self.assertEqual(self.path.read_bytes(), self.data)
                self.assertIsNone(adapter.source_path)
                self.assertTrue(adapter.busy)
                self.assertEqual(operation.cancel_command["documentId"], 5)
                operation.cancelled()
                self.assertFalse(adapter.busy)

    def test_submission_failure_after_publication_preserves_output(self):
        operation = self.prepared()
        operation.publish()
        error = RuntimeError("worker submit failed")
        reported = operation.failure(error)
        self.assertIsInstance(reported, files.PublicationError)
        self.assertIs(reported.__cause__, error)
        self.assertTrue(self.adapter.busy)
        operation.cancelled()
        self.assertEqual(self.path.read_bytes(), self.data)

    def test_temporary_flush_failure_preserves_original_and_cancels_token(self):
        original_stamp = self.opened()
        operation = self.prepared(self.adapter.begin_save(self.target))
        with patch.object(files.os, "fsync", side_effect=OSError("disk full")):
            with self.assertRaises(OSError):
                operation.publish()
        self.assertEqual(self.path.read_bytes(), b"original")
        self.assertEqual(self.adapter.source_stamp, original_stamp)
        self.assertFalse(list(self.path.parent.glob(".rhwp-save-*")))
        self.assertIsInstance(operation.failure(OSError("disk full")), OSError)
        self.assertEqual(operation.cancel_command["token"], 7)

    def test_directory_flush_failure_preserves_published_file_and_source_stamp(self):
        original_stamp = self.opened()
        operation = self.prepared(self.adapter.begin_save(self.target))
        with patch.object(files, "_sync_directory", side_effect=OSError("directory flush")):
            with self.assertRaises(files.PublicationError):
                operation.publish()
        self.assertEqual(self.path.read_bytes(), self.data)
        self.assertEqual(self.adapter.source_stamp, original_stamp)
        with self.assertRaises(RuntimeError):
            _ = operation.acknowledge_command
        operation.cancelled()

    def test_verification_failure_preserves_publication_and_original_source_stamp(self):
        original_stamp = self.opened()
        operation = self.prepared(self.adapter.begin_save(self.target))
        real_read = files.read_document

        def fail_after_publication(path):
            if Path(path).read_bytes() == self.data:
                raise OSError("published destination cannot be verified")
            return real_read(path)

        with patch.object(files, "read_document", side_effect=fail_after_publication):
            with self.assertRaises(files.PublicationError) as raised:
                operation.publish()
        self.assertTrue(raised.exception.published)
        self.assertEqual(self.path.read_bytes(), self.data)
        self.assertEqual(self.adapter.source_stamp, original_stamp)
        self.assertEqual(operation.cancel_command["token"], 7)
        operation.cancelled()

    def test_publication_runs_on_io_thread_and_copy_release_stays_on_owner_thread(self):
        event_loop_thread = get_ident()
        operation = self.prepared()
        real_publish = files.publish_document
        threads = []

        def published(*args):
            threads.append(get_ident())
            return real_publish(*args)

        with patch("native_worker_file_store.publish_document", side_effect=published):
            with ThreadPoolExecutor(max_workers=1) as executor:
                executor.submit(operation.publish).result(timeout=5)
        self.assertNotEqual(threads, [event_loop_thread])
        self.assertEqual(self.releases, [(11, event_loop_thread)])
        operation.acknowledged(self.acknowledgement())

    def test_pending_io_must_finish_before_token_cleanup_or_another_publication(self):
        started, finish = Event(), Event()
        operation = self.prepared()
        real_publish = files.publish_document

        def delayed(*args):
            started.set()
            if not finish.wait(timeout=5):
                raise AssertionError("test did not release publication")
            return real_publish(*args)

        with patch("native_worker_file_store.publish_document", side_effect=delayed):
            with ThreadPoolExecutor(max_workers=1) as executor:
                future = executor.submit(operation.publish)
                try:
                    self.assertTrue(started.wait(timeout=5))
                    with self.assertRaises(RuntimeError):
                        operation.cancelled()
                    with self.assertRaises(RuntimeError):
                        _ = operation.cancel_command
                    with self.assertRaises(RuntimeError):
                        operation.publish()
                finally:
                    finish.set()
                future.result(timeout=5)
        operation.cancelled()
        self.assertEqual(self.path.read_bytes(), self.data)

    def test_new_document_store_is_not_changed_by_an_old_document_save(self):
        operation = self.prepared()
        new_document = NativeWorkerFileStore(99)
        operation.publish()
        self.assertEqual(operation.acknowledge_command["documentId"], 5)
        result = operation.acknowledged(self.acknowledgement())
        self.assertEqual(result["documentId"], 5)
        self.assertIsNone(new_document.source_path)
        self.assertIsNone(new_document.source_stamp)
        self.assertFalse(new_document.busy)


class WorkerRecoveryContracts(WorkerFileStoreFixture, unittest.TestCase):
    def recovery(self, *, now=100, target=None, report=None):
        operation = self.adapter.begin_recovery(
            target or self.target, self.path.parent / "recovery", is_dirty=True, now=now)
        self.assertIsNotNone(operation)
        operation.receive_prepared(self.response(
            revision=operation.revision, report=self.report if report is None else report),
            self.copy_buffer)
        return operation

    def cleanup(self, current_revision=3, saved_revision=0, **changes):
        dirty = current_revision != saved_revision
        return {"ok": True, "kind": "saveStatus", "documentId": 5, "isDirty": dirty,
                "status": {"revision": current_revision, "savedRevision": saved_revision,
                           "dirty": dirty}, **changes}

    def finish_recovery(self, operation, **changes):
        operation.publish()
        result = operation.recovery_completed(self.cleanup(**changes))
        return result

    def rewrite_recovery(self, path, header, data):
        encoded = json.dumps(header).encode("utf-8")
        path.write_bytes(files.MAGIC + files.struct.pack("<I", len(encoded)) + encoded + data)

    def test_recovery_cancels_without_ack_or_source_adoption(self):
        original_stamp = self.opened()
        operation = self.recovery()
        self.assertEqual(operation.format, "hwpx")
        operation.publish()
        self.assertTrue(self.adapter.busy)
        with self.assertRaises(RuntimeError):
            _ = operation.acknowledge_command
        with self.assertRaises(RuntimeError):
            operation.acknowledged(self.acknowledgement())
        self.assertEqual(operation.cancel_command,
                         {"op": "cancelSave", "documentId": 5, "token": 7})
        result = operation.recovery_completed(self.cleanup())
        self.assertTrue(result["recovery"])
        self.assertTrue(result["isDirty"])
        self.assertEqual(result["revision"], 3)
        self.assertEqual(self.path.read_bytes(), b"original")
        self.assertEqual(self.adapter.source_path, self.path)
        self.assertEqual(self.adapter.source_stamp, original_stamp)
        self.assertFalse(self.adapter.busy)

    def test_normal_save_and_recovery_share_the_same_active_slot(self):
        operation = self.begin()
        self.assertIsNone(self.adapter.begin_recovery(
            self.target, self.path.parent, is_dirty=True, now=100))
        operation.cancelled()
        operation = self.recovery()
        with self.assertRaises(RuntimeError):
            self.begin()
        self.finish_recovery(operation)
        self.begin()

    def test_clean_document_and_invalid_clock_do_not_prepare_a_snapshot(self):
        self.assertIsNone(self.adapter.begin_recovery(
            self.target, self.path.parent, is_dirty=False, now=100))
        for arguments in ({"now": float("nan")}, {"now": float("inf")},
                          {"interval": 0}, {"interval": -1}, {"interval": float("inf")}):
            with self.subTest(arguments=arguments):
                with self.assertRaises(ValueError):
                    self.adapter.begin_recovery(self.target, self.path.parent, is_dirty=True, **arguments)
        with self.assertRaises(ValueError):
            self.adapter.begin_recovery(self.target, self.path.parent, is_dirty=1)
        self.assertFalse(self.adapter.busy)
        self.assertFalse(list(self.path.parent.glob("*.rhwp-recovery")))

    def test_duplicate_revision_and_minimum_interval_are_enforced(self):
        self.finish_recovery(self.recovery(now=100))
        self.assertIsNone(self.adapter.begin_recovery(
            self.target, self.path.parent, is_dirty=True, now=1000))
        newer = dict(self.target, revision=4)
        self.assertIsNone(self.adapter.begin_recovery(
            newer, self.path.parent, is_dirty=True, now=129.9))
        operation = self.recovery(now=130, target=newer)
        self.finish_recovery(operation, current_revision=4)
        self.assertEqual(self.adapter._autosave_revision, 4)
        self.assertEqual(self.adapter._autosave_time, 130)

    def test_recovery_keeps_exact_prepared_bytes_and_loss_report_without_a_dialog(self):
        report = {"count": 1, "losses": [{"kind": "unsupportedObject"}]}
        expected_report = deepcopy(report)
        operation = self.recovery(report=report)
        self.assertFalse(operation.requires_loss_acceptance)
        expected_bytes = bytes(self.data)
        self.data[:] = b"a later revision"
        report["losses"].clear()
        operation.publish()
        header, data = files.read_recovery(operation.path)
        self.assertEqual(data, expected_bytes)
        self.assertEqual(header["report"], expected_report)
        result = operation.recovery_completed(self.cleanup(current_revision=4))
        self.assertEqual(result["report"], expected_report)
        self.assertEqual(result["currentRevision"], 4)
        self.assertTrue(result["isDirty"])

    def test_source_fingerprint_stays_bound_to_opened_bytes_after_external_changes(self):
        opened_stamp = self.opened()
        operation = self.recovery()
        self.path.write_bytes(b"external source edit")
        real_read = files.read_document

        def do_not_read_source(path):
            self.assertNotEqual(Path(path), self.path)
            return real_read(path)

        with patch.object(files, "read_document", side_effect=do_not_read_source):
            operation.publish()
        header, data = files.read_recovery(operation.path)
        self.assertEqual(header["sourcePath"], str(self.path))
        self.assertEqual(header["sourceStamp"], files.asdict(opened_stamp))
        self.assertEqual(self.path.read_bytes(), b"external source edit")
        self.assertEqual(data, self.data)
        operation.recovery_completed(self.cleanup())

    def test_stale_cleanup_preserves_artifact_and_leaves_new_document_untouched(self):
        operation = self.recovery()
        operation.publish()
        new_document = NativeWorkerFileStore(99)
        with self.assertRaises(files.PublicationError):
            operation.recovery_completed({"ok": False, "code": "staleDocument", "error": "replaced"})
        self.assertTrue(self.adapter.busy)
        self.assertEqual(files.read_recovery(operation.path)[1], self.data)
        operation.cancelled()
        self.assertFalse(self.adapter.busy)
        self.assertIsNone(self.adapter.begin_recovery(
            self.target, self.path.parent, is_dirty=True, now=1000))
        self.assertFalse(new_document.busy)
        self.assertIsNone(new_document.source_path)
        self.assertIsNone(new_document.source_stamp)

    def test_cleanup_rejects_mismatched_document_or_checkpoint_status(self):
        for reply in (self.cleanup(documentId=99), self.cleanup(isDirty=False),
                      self.cleanup(saved_revision=4), self.cleanup(current_revision=2)):
            with self.subTest(reply=reply):
                self.adapter = NativeWorkerFileStore(5)
                operation = self.recovery()
                operation.publish()
                with self.assertRaises(files.PublicationError):
                    operation.recovery_completed(reply)
                self.assertTrue(operation.path.exists())
                operation.cancelled()

    def test_cleanup_cannot_complete_before_a_verified_publication(self):
        operation = self.recovery()
        with self.assertRaises(RuntimeError):
            operation.recovery_completed(self.cleanup())
        operation.cancelled()
        self.assertIsNone(self.adapter._autosave_revision)
        self.assertFalse(operation.path.exists())

    def test_failed_publication_does_not_advance_duplicate_or_interval_state(self):
        operation = self.recovery()
        with patch.object(files.os, "fsync", side_effect=OSError("disk full")):
            with self.assertRaises(OSError):
                operation.publish()
        self.assertEqual(operation.cancel_command["token"], 7)
        operation.cancelled()
        self.assertFalse(operation.path.exists())
        self.assertIsNone(self.adapter._autosave_revision)
        self.assertIsNotNone(self.adapter.begin_recovery(
            self.target, self.path.parent, is_dirty=True, now=100))

    def test_header_limit_does_not_reduce_the_document_payload_allowance(self):
        operation = self.recovery()
        with patch.object(files, "MAX_BYTES", len(self.data)):
            operation.publish()
            self.assertEqual(files.read_recovery(operation.path)[1], self.data)
        self.assertGreater(operation.path.stat().st_size, len(self.data))
        operation.recovery_completed(self.cleanup())

    def test_oversized_recovery_metadata_is_rejected_before_creating_an_artifact(self):
        operation = self.recovery(report={"count": 1, "losses": ["x" * files.MAX_RECOVERY_HEADER]})
        with self.assertRaises(ValueError):
            operation.publish()
        self.assertFalse(operation.path.exists())
        self.assertEqual(operation.cancel_command["token"], 7)
        operation.cancelled()

    def test_recovery_publication_is_exclusive_and_never_overwrites_existing_files(self):
        operation = self.recovery()
        operation.path.parent.mkdir(parents=True)
        operation.path.write_bytes(b"existing artifact")
        with self.assertRaises(FileExistsError):
            operation.publish()
        self.assertEqual(operation.path.read_bytes(), b"existing artifact")
        operation.cancelled()

    def test_corrupt_truncated_and_unsupported_recovery_is_rejected(self):
        operation = self.recovery()
        operation.publish()
        valid = operation.path.read_bytes()
        header, payload = files.read_recovery(operation.path)
        broken = (b"", b"bad signature", files.MAGIC + b"\0",
                  files.MAGIC + files.struct.pack("<I", files.MAX_RECOVERY_HEADER + 1),
                  valid[:-1], valid + b"extra", valid[:len(files.MAGIC) + 5])
        for data in broken:
            with self.subTest(data=data[:20]):
                operation.path.write_bytes(data)
                with self.assertRaises(ValueError):
                    files.read_recovery(operation.path)
        changes = ({"version": 2}, {"version": True}, {"format": "hwp"},
                   {"revision": -1}, {"byteLength": True}, {"digest": "wrong"},
                   {"report": None}, {"sourcePath": "unexpected", "sourceStamp": None})
        for change in changes:
            with self.subTest(change=change):
                self.rewrite_recovery(operation.path, dict(header, **change), payload)
                with self.assertRaises(ValueError):
                    files.read_recovery(operation.path)

    def test_recovery_metadata_paths_are_only_returned_and_never_followed(self):
        operation = self.recovery()
        operation.publish()
        header, payload = files.read_recovery(operation.path)
        other = self.path.with_name("must-not-change.hwpx")
        other.write_bytes(b"private source")
        header["sourcePath"] = str(other)
        header["sourceStamp"] = {"digest": "untrusted metadata"}
        self.rewrite_recovery(operation.path, header, payload)
        with patch.object(files, "read_document", side_effect=AssertionError("source followed")):
            actual_header, actual_data = files.read_recovery(operation.path)
        self.assertEqual(actual_header["sourcePath"], str(other))
        self.assertEqual(actual_data, payload)
        self.assertEqual(other.read_bytes(), b"private source")
        self.assertTrue(other.exists())


def native_roundtrip(library):
    """Exercise the same stages with real worker completions and owned buffers."""
    from native_worker import NativeWorker

    def accepted(worker, command):
        result = worker.wait(worker.submit(command))
        if not result.get("ok"):
            raise AssertionError(result)
        return result

    with tempfile.TemporaryDirectory(prefix="rhwp-worker-save-native-") as directory:
        with NativeWorker(library) as worker, ThreadPoolExecutor(max_workers=1) as io:
            info = accepted(worker, {"op": "new", "replace": None})["info"]
            target = {"documentId": info["documentId"], "revision": info["revision"]}
            position = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
            accepted(worker, {"op": "beginEdit", "target": target,
                              "selection": {"anchor": position, "focus": position}})
            edited = accepted(worker, {"op": "replace", "target": target, "text": "가나다"})
            target["revision"] = edited["outcome"]["revision"]
            adapter = NativeWorkerFileStore(target["documentId"])
            path = Path(directory) / "document.hwpx"
            operation = adapter.begin_save(target, path)
            operation.receive_prepared(accepted(worker, operation.prepare_command), worker.copy_buffer)
            if operation.requires_loss_acceptance:
                raise AssertionError(operation.report)
            io.submit(operation.publish).result(timeout=10)
            edited = accepted(worker, {"op": "replace", "target": target, "text": " 이후 편집"})
            target["revision"] = edited["outcome"]["revision"]
            saved = operation.acknowledged(accepted(worker, operation.acknowledge_command))
            if not saved["isDirty"] or saved["currentRevision"] != target["revision"]:
                raise AssertionError(saved)
            if worker.host.read_text(path).strip() != "가나다":
                raise AssertionError("published bytes did not retain their prepared revision")

            operation = adapter.begin_save(target)
            operation.receive_prepared(accepted(worker, operation.prepare_command), worker.copy_buffer)
            io.submit(operation.publish).result(timeout=10)
            saved = operation.acknowledged(accepted(worker, operation.acknowledge_command))
            if saved["isDirty"] or worker.host.read_text(path).strip() != "가나다 이후 편집":
                raise AssertionError("replacement did not save the latest document")

            operation = adapter.begin_save(target, Path(directory) / "document.hwp")
            operation.receive_prepared(accepted(worker, operation.prepare_command), worker.copy_buffer)
            io.submit(operation.publish).result(timeout=10)
            operation.acknowledged(accepted(worker, operation.acknowledge_command))
            if worker.host.read_text(adapter.source_path).strip() != "가나다 이후 편집":
                raise AssertionError("HWP save did not reopen with its original text")

            operation = adapter.begin_save(target)
            operation.receive_prepared(accepted(worker, operation.prepare_command), worker.copy_buffer)
            adapter.source_path.write_bytes(b"external edit")
            try:
                io.submit(operation.publish).result(timeout=10)
            except files.ExternalFileChanged:
                accepted(worker, operation.cancel_command)
                operation.cancelled()
            else:
                raise AssertionError("external edit was overwritten")
            if adapter.source_path.read_bytes() != b"external edit" or adapter.busy:
                raise AssertionError("conflict cleanup changed the destination or retained the store")

            stale_path = Path(directory) / "stale.hwpx"
            operation = adapter.begin_save(target, stale_path)
            operation.receive_prepared(accepted(worker, operation.prepare_command), worker.copy_buffer)
            io.submit(operation.publish).result(timeout=10)
            fresh = accepted(worker, {"op": "new", "replace": target})["info"]
            reply = worker.wait(worker.submit(operation.acknowledge_command))
            try:
                operation.acknowledged(reply)
            except files.PublicationError:
                worker.wait(worker.submit(operation.cancel_command))
                operation.cancelled()
            else:
                raise AssertionError("old document acknowledgement was accepted")
            current = accepted(worker, {"op": "info"})["info"]
            if current != fresh or not stale_path.exists() or adapter.busy:
                raise AssertionError("stale acknowledgement changed the new document or lost its output")

            source_bytes, source_stamp = files.read_document(stale_path)
            opened = worker.wait(worker.open_bytes(source_bytes, replace={
                "documentId": fresh["documentId"], "revision": fresh["revision"]}))
            if not opened.get("ok"):
                raise AssertionError(opened)
            info = opened["info"]
            target = {"documentId": info["documentId"], "revision": info["revision"]}
            accepted(worker, {"op": "beginEdit", "target": target,
                              "selection": {"anchor": position, "focus": position}})
            edited = accepted(worker, {"op": "replace", "target": target, "text": "복구 "})
            target["revision"] = edited["outcome"]["revision"]
            adapter = NativeWorkerFileStore(target["documentId"], stale_path, source_stamp)
            before = accepted(worker, {"op": "info"})["info"]
            recovery = adapter.begin_recovery(target, Path(directory) / "recovery",
                                               is_dirty=before["isDirty"], now=100)
            if recovery is None:
                raise AssertionError("dirty document did not prepare recovery")
            recovery.receive_prepared(accepted(worker, recovery.prepare_command), worker.copy_buffer)
            io.submit(recovery.publish).result(timeout=10)
            cleanup = accepted(worker, recovery.cancel_command)
            recovery.recovery_completed(cleanup)
            if cleanup["status"]["savedRevision"] != info["revision"]:
                raise AssertionError("recovery advanced the original save checkpoint")
            if accepted(worker, {"op": "info"})["info"] != before:
                raise AssertionError("recovery changed the dirty document state")
            if stale_path.read_bytes() != source_bytes:
                raise AssertionError("recovery overwrote its source")
            header, recovered_bytes = files.read_recovery(recovery.path)
            if header["sourceStamp"] != files.asdict(source_stamp):
                raise AssertionError("recovery did not retain the opened source fingerprint")
            if adapter.begin_recovery(target, recovery.path.parent, is_dirty=True, now=200) is not None:
                raise AssertionError("unchanged revision created duplicate recovery")

            with NativeWorker(library) as restored:
                reopened = restored.wait(restored.open_recovery_bytes(recovered_bytes))
                if not reopened.get("ok"):
                    raise AssertionError(reopened)
                restored_info = reopened["info"]
                if not restored_info["isDirty"] or restored_info["revision"] != 0:
                    raise AssertionError("recovery import falsely adopted a saved checkpoint")
                restored_target = {"documentId": restored_info["documentId"],
                                   "revision": restored_info["revision"]}
                restored_store = NativeWorkerFileStore(restored_target["documentId"])
                restored_path = Path(directory) / "explicitly-restored.hwpx"
                saved = restored_store.begin_save(restored_target, restored_path)
                saved.receive_prepared(accepted(restored, saved.prepare_command), restored.copy_buffer)
                io.submit(saved.publish).result(timeout=10)
                saved.acknowledged(accepted(restored, saved.acknowledge_command))
                if restored.host.read_text(restored_path).strip() != "복구 가나다 이후 편집":
                    raise AssertionError("worker recovery reopened different document content")

            edited = accepted(worker, {"op": "replace", "target": target, "text": "추가 "})
            target["revision"] = edited["outcome"]["revision"]
            stale_recovery = adapter.begin_recovery(target, recovery.path.parent, is_dirty=True, now=131)
            stale_recovery.receive_prepared(accepted(worker, stale_recovery.prepare_command), worker.copy_buffer)
            io.submit(stale_recovery.publish).result(timeout=10)
            fresh = accepted(worker, {"op": "new", "replace": target})["info"]
            cleanup = worker.wait(worker.submit(stale_recovery.cancel_command))
            try:
                stale_recovery.recovery_completed(cleanup)
            except files.PublicationError:
                stale_recovery.cancelled()
            else:
                raise AssertionError("stale recovery cleanup was accepted")
            if accepted(worker, {"op": "info"})["info"] != fresh or adapter.busy:
                raise AssertionError("old recovery cleanup changed the new document")
            files.read_recovery(stale_recovery.path)
    print(json.dumps({"ok": True, "nativeWorkerSave": True, "savedRevisionBytes": True,
                      "laterEditDirty": True, "hwpHwpxReopen": True,
                      "externalConflict": True, "staleDocumentPublicationPreserved": True,
                      "workerRecoveryReopen": True, "recoveryPreservedDirtyCheckpoint": True,
                      "staleRecoveryPublicationPreserved": True}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--library")
    arguments, remaining = parser.parse_known_args()
    result = unittest.main(argv=[sys.argv[0], *remaining], exit=False)
    if not result.result.wasSuccessful():
        raise SystemExit(1)
    if arguments.library:
        native_roundtrip(arguments.library)
