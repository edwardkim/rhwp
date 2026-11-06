#!/usr/bin/env python3
"""Qt offscreen adapter checks; --library adds actual native render/IME/save flow."""
import argparse
import hashlib
import copy
from concurrent.futures import Future
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from PySide6.QtCore import QEvent, QMimeData, QPointF, Qt
from PySide6.QtGui import QImage, QInputMethodEvent, QKeyEvent
from PySide6.QtWidgets import QApplication
from qt_native_host import NativeWindow, scalar_from_utf16, utf16_length
from native_file_store import FileStamp


class IdleWorker:
    def __init__(self):
        self.requests = []
        self.finished = True
        self.released = False
    def submit(self, command):
        self.requests.append(command)
        return len(self.requests)
    def poll(self):
        return None
    def close(self):
        pass
    def is_finished(self):
        return self.finished
    def release(self):
        self.released = True
    def open_bytes(self, data, replace=None):
        self.requests.append({"op": "open", "data": data, "replace": replace})
        return len(self.requests)
    def copy_buffer(self, response):
        return b"serialized document"


class DeferredIO:
    """Hold filesystem completions so tests can order actual GUI races."""
    def __init__(self):
        self.futures = []
        self.tasks = []
    def submit(self, function, *args):
        future = Future()
        self.futures.append(future)
        self.tasks.append((function, args))
        return future
    def run(self, index):
        function, args = self.tasks[index]
        try:
            self.futures[index].set_result(function(*args))
        except BaseException as error:
            self.futures[index].set_exception(error)


class TileWorker(IdleWorker):
    def __init__(self):
        super().__init__()
        self.host = self
        self.cancellations, self.released_buffers, self.copied_buffers = [], [], []
        self.next_view = None
    def cancel(self, request_id):
        self.cancellations.append(request_id)
        return "cancelled"
    def call(self, operation, buffer):
        assert operation == "buffer_release"
        self.released_buffers.append(buffer)
    def host_request(self, **request):
        assert request["op"] == "view"
        return self.next_view
    def copy_buffer(self, response):
        self.copied_buffers.append(response["buffer"])
        self.released_buffers.append(response["buffer"])
        return response.get("pixels", b"\xff\0\0\xff" * (response["width"] * response["height"]))


class AdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.app = QApplication.instance() or QApplication([])

    def idle_window(self, worker=None):
        window = NativeWindow(None, worker=worker or IdleWorker())
        window.timer.stop()
        window.actions.clear()
        self.addCleanup(window.close)
        self.addCleanup(window.io.shutdown, wait=True)
        self.addCleanup(setattr, window, "finished", True)
        return window

    def lifecycle_window(self, dirty=True):
        window = self.idle_window()
        window.info = {"documentId": 7, "revision": 3, "fontGeneration": 0,
                       "pageCount": 1, "isDirty": dirty, "compositionId": None,
                       "canUndo": dirty, "canRedo": False}
        window.source_path = Path("original.hwpx")
        window.refresh = lambda: None
        window.update_document_ui()
        return window

    def complete_step(self, window, **reply):
        self.assertIsNotNone(window.pending)
        if "token" in reply:
            reply = {"kind": "preparedSave", "documentId": window.info["documentId"],
                     "revision": window.info["revision"], "byteLength": len(b"serialized document"),
                     "report": {"count": 0}, **reply}
        elif "isDirty" in reply and "info" not in reply:
            operation = window.save_operation
            reply = {"kind": "saveStatus", "documentId": window.info["documentId"],
                     "status": {"savedRevision": operation.revision, "revision": window.info["revision"],
                                "dirty": reply["isDirty"]}, **reply}
        window.pending = None
        window.step_done({"ok": True, **reply})
        window.pump()

    @staticmethod
    def stamp(data=b"serialized document"):
        return FileStamp(hashlib.sha256(data).hexdigest(), len(data), 1, 1, 1)

    def complete_publication(self, io, index=0):
        # Exercise the operation's real stage transitions with a controlled file
        # publisher. The file-store tests separately cover actual filesystem I/O.
        with patch("native_worker_file_store.publish_document", return_value=self.stamp()):
            io.run(index)
    def tiled_window(self):
        worker = TileWorker()
        window = self.idle_window(worker)
        window.info = {"documentId": 7, "revision": 3, "fontGeneration": 0,
                       "pageCount": 1, "isDirty": False}
        window.pages = [{"width": 1024, "height": 768}]
        window.state["config"].update(pages=window.pages, viewport={"width": 400, "height": 400})
        window.view = {"state": copy.deepcopy(window.state),
                       "scrollLimit": {"x": 624, "y": 368},
                       "visible": [{"page": 0, "rasterScale": 1,
                                    "pageRegion": {"x": 0, "y": 0, "width": 400, "height": 400},
                                    "pageRect": {"x": 0, "y": 0, "width": 1024, "height": 768}}]}
        window.page_geometry_key = window.geometry_key()
        window.refresh = lambda: None
        window.reset_tiles()
        window.sync_visible_tiles()
        return window

    def complete_tile(self, window, **changes):
        _, callback = window.pending
        window.pending = None
        reply = {"ok": True, "kind": "frame", "buffer": 99,
                 "revision": 3, "fontGeneration": 0,
                 "originX": 0, "originY": 0, "width": 512, "height": 512,
                 "stride": 512 * 4, "byteLength": 512 * 512 * 4}
        callback({**reply, **changes})

    def test_scroll_reuses_client_tiles_and_keeps_images_out_of_frame_metadata(self):
        window = self.tiled_window()
        window.render_next()
        self.assertEqual(window.worker.requests[-1]["op"], "renderTiledRegion")
        self.complete_tile(window)
        key = window.visible_tiles[0].key(window.tile_identity)
        image = window.tile_images.get(key).image
        moved = copy.deepcopy(window.view)
        moved["state"]["scroll"]["x"] = 100
        moved["visible"][0]["pageRegion"].update(x=100, width=400)
        moved["visible"][0]["pageRect"]["x"] = -100
        window.worker.next_view = moved
        window.set_scroll(x=100)
        self.assertTrue(window.frames)
        self.assertTrue(all(value is True for value in window.frames.values()))
        self.assertIs(window.tile_images.get(key).image, image)
        window.render_next()
        self.assertEqual(len([r for r in window.worker.requests if r["op"] == "renderTiledRegion"]), 1)
        self.assertEqual(window.worker.copied_buffers, [99])

    def test_overlay_selection_changes_do_not_invalidate_raster_tiles(self):
        window = self.tiled_window()
        window.render_next()
        self.complete_tile(window)
        retained, identity = window.tile_images.retained_bytes, window.tile_identity
        position = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        window.absorb({"outcome": {"selection": {"anchor": position, "focus": position}}})
        window.set_rects([])
        self.assertEqual(window.tile_identity, identity)
        self.assertEqual(window.tile_images.retained_bytes, retained)
        self.assertTrue(window.frames)

    def test_stale_view_revision_font_and_native_metadata_are_released_without_copy(self):
        for change in ("view", "revision", "font", "resultFont"):
            with self.subTest(change=change):
                window = self.tiled_window()
                window.render_next()
                if change == "view":
                    window.epoch += 1
                elif change == "revision":
                    window.absorb({"info": {**window.info, "revision": 4}})
                elif change == "font":
                    window.absorb({"info": {**window.info, "fontGeneration": 1}})
                self.complete_tile(window, **({"fontGeneration": 1} if change == "resultFont" else {}))
                self.assertEqual(window.worker.copied_buffers, [])
                self.assertEqual(window.worker.released_buffers, [99])
                self.assertEqual(len(window.tile_images), 0)
                self.assertEqual(window.errors, [])

    def test_pending_render_cancellation_keeps_edit_fifo_until_completion(self):
        window = self.tiled_window()
        window.render_next()
        pending_id = window.pending[0]
        moved = copy.deepcopy(window.view)
        moved["state"]["scroll"]["x"] = 100
        moved["visible"][0]["pageRegion"]["x"] = 100
        moved["visible"][0]["pageRect"]["x"] = -100
        window.worker.next_view = moved
        window.set_scroll(x=100)
        window.command("replace", text="after render")
        window.pump()
        self.assertEqual(window.pending[0], pending_id)
        self.assertEqual(window.worker.cancellations, [pending_id])
        self.assertEqual([r["op"] for r in window.worker.requests], ["renderTiledRegion"])
        self.complete_tile(window, kind="cancelled")
        window.pump()
        self.assertEqual([r["op"] for r in window.worker.requests], ["renderTiledRegion", "replace"])
        self.assertEqual(window.worker.released_buffers, [99])
        self.assertEqual(window.errors, [])

    def test_skipped_render_is_normal_cancellation_and_does_not_poison_tile(self):
        window = self.tiled_window()
        window.render_next()
        self.complete_tile(window, kind="skipped", reason="supersededRender")
        self.assertFalse(window.tile_failures)
        self.assertEqual(window.errors, [])
        self.assertEqual(window.worker.copied_buffers, [])
        self.assertEqual(window.worker.released_buffers, [99])

    def test_invalid_tile_buffer_metadata_is_released_before_copy_and_not_retried(self):
        window = self.tiled_window()
        window.render_next()
        self.complete_tile(window, width=100_000, byteLength=2**63)
        window.render_next()
        self.assertEqual(window.worker.copied_buffers, [])
        self.assertEqual(window.worker.released_buffers, [99])
        self.assertEqual(len([r for r in window.worker.requests if r["op"] == "renderTiledRegion"]), 1)
        self.assertEqual(len(window.errors), 1)

    def test_tile_paint_uses_actual_origin_and_clips_to_canonical_grid(self):
        window = self.tiled_window()
        window.view["visible"][0]["pageRegion"].update(x=512, width=400)
        window.view["visible"][0]["pageRect"]["x"] = -512
        window.sync_visible_tiles()
        window.render_next()
        row = b"\0\xff\0\xff" + b"\xff\0\0\xff" * 513
        self.complete_tile(window, originX=511, width=514, stride=514 * 4,
                           byteLength=514 * 512 * 4, pixels=row * 512)
        window.worker.next_view = copy.deepcopy(window.view)
        window.show()
        self.app.processEvents()
        painted = window.canvas.grab().toImage()
        self.assertEqual(painted.pixelColor(0, 20).name(), "#ff0000")
        self.assertEqual(window.tile_images.retained_bytes, 514 * 512 * 4)

    def test_dirty_close_cancel_keeps_document_and_worker(self):
        window = self.lifecycle_window()
        choices = []
        window.confirm_unsaved_changes = lambda: choices.append("asked") or "cancel"
        window.close()
        self.assertEqual(choices, [])  # Wait for the accepted-input FIFO barrier.
        window.pump()
        self.complete_step(window, info=dict(window.info))
        self.assertEqual(choices, ["asked"])
        self.assertFalse(window.closing)
        self.assertIsNone(window.intent)
        self.assertEqual(window.info["documentId"], 7)
        self.assertTrue(window.info["isDirty"])
        self.assertFalse(window.worker.released)

    def test_transition_inspects_edits_already_accepted_by_the_queue(self):
        window = self.lifecycle_window(dirty=False)
        window.command("replace", text="accepted before close")
        choices = []
        window.confirm_unsaved_changes = lambda: choices.append("dirty") or "cancel"
        window.close()
        window.pump()
        self.assertEqual(window.worker.requests[-1]["op"], "replace")
        self.complete_step(window, outcome={"revision": 4})
        self.assertEqual(window.worker.requests[-1]["op"], "info")
        self.complete_step(window, info={**window.info, "isDirty": True})
        self.assertEqual(choices, ["dirty"])
        self.assertFalse(window.closing)

    def test_discard_new_document_drops_actions_accepted_for_old_generation(self):
        window = self.lifecycle_window()
        window.confirm_unsaved_changes = lambda: "discard"
        window.new_document()
        window.pump()
        self.complete_step(window, info=dict(window.info))
        self.complete_step(window, info=dict(window.info))
        self.assertEqual(window.worker.requests[-1], {"op": "new", "replace": {"documentId": 7, "revision": 3}})
        window.command("replace", text="belongs to the old document")
        self.complete_step(window, info={**window.info, "documentId": 8, "revision": 0, "isDirty": False})
        self.assertEqual(window.document_generation, 1)
        self.assertIsNone(window.source_path)
        self.assertNotIn("replace", [request["op"] for request in window.worker.requests])
        self.assertEqual(window.worker.requests[-1]["op"], "beginEdit")

    def test_late_open_read_cannot_replace_a_newer_open_intent(self):
        window = self.lifecycle_window(dirty=False)
        io = DeferredIO()
        window.io = io
        window.open_path("old.hwpx")
        window.pump()
        self.complete_step(window, info=dict(window.info))
        window.open_path("latest.hwpx")
        window.pump()
        self.complete_step(window, info=dict(window.info))
        io.futures[1].set_result((b"latest document", self.stamp(b"latest document")))
        window.tick()
        self.complete_step(window, info=dict(window.info))
        self.assertEqual(window.worker.requests[-1]["data"], b"latest document")
        self.complete_step(window, info={**window.info, "documentId": 8, "revision": 0})
        io.futures[0].set_exception(OSError("obsolete read failed"))
        window.tick()
        self.assertEqual([r["data"] for r in window.worker.requests if r["op"] == "open"], [b"latest document"])
        self.assertEqual(window.source_path, Path("latest.hwpx"))
        self.assertEqual(window.file_store.source_stamp, self.stamp(b"latest document"))
        self.assertEqual(window.errors, [])

    def test_edit_during_open_read_keeps_current_document(self):
        window = self.lifecycle_window(dirty=False)
        io = DeferredIO()
        window.io = io
        window.open_path("other.hwpx")
        window.pump()
        self.complete_step(window, info=dict(window.info))
        window.command("replace", text="arrived while reading")
        window.pump()
        self.complete_step(window, outcome={"revision": 4})
        io.futures[0].set_result((b"other document", self.stamp(b"other document")))
        window.tick()
        self.complete_step(window, info={**window.info, "isDirty": True})
        self.assertFalse(any(r["op"] == "open" for r in window.worker.requests))
        self.assertIsNone(window.intent)
        self.assertEqual(window.source_path, Path("original.hwpx"))
        self.assertTrue(window.info["isDirty"])

    def test_cancelled_save_dialog_preserves_dirty_document_on_new(self):
        window = self.lifecycle_window()
        window.confirm_unsaved_changes = lambda: "save"
        with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=("", "")):
            window.new_document()
            window.pump()
            self.complete_step(window, info=dict(window.info))
        self.assertIsNone(window.intent)
        self.assertFalse(window.saving)
        self.assertTrue(window.info["isDirty"])
        self.assertFalse(any(r["op"] in ("new", "prepareSave") for r in window.worker.requests))

    def test_save_write_failure_cancels_token_and_keeps_close_pending_document(self):
        window = self.lifecycle_window()
        io = DeferredIO()
        window.io = io
        window.confirm_unsaved_changes = lambda: "save"
        with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=("new.hwpx", "")):
            window.close()
            window.pump()
            self.complete_step(window, info=dict(window.info))
        self.assertEqual(window.worker.requests[-1]["op"], "prepareSave")
        self.complete_step(window, token=12, buffer=99)
        io.futures[0].set_exception(OSError("disk full"))
        window.tick()
        self.assertEqual(window.worker.requests[-1]["op"], "cancelSave")
        self.assertTrue(window.saving)  # Retain the operation until token cleanup.
        self.complete_step(window)
        self.assertIsNone(window.intent)
        self.assertFalse(window.closing)
        self.assertFalse(window.saving)
        self.assertTrue(window.info["isDirty"])
        self.assertEqual(window.source_path, Path("original.hwpx"))

    def test_successful_save_with_later_edit_does_not_replace_document(self):
        window = self.lifecycle_window()
        io = DeferredIO()
        window.io = io
        window.confirm_unsaved_changes = lambda: "save"
        with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=("new.hwpx", "")):
            window.new_document()
            window.pump()
            self.complete_step(window, info=dict(window.info))
        self.complete_step(window, token=12, buffer=99)
        window.command("replace", text="later edit")
        window.pump()
        self.complete_step(window, outcome={"revision": 4})
        self.complete_publication(io)
        window.tick()
        self.complete_step(window, isDirty=True)
        self.complete_step(window, info={**window.info, "isDirty": True})
        self.assertIsNone(window.intent)
        self.assertFalse(window.saving)
        self.assertEqual(window.source_path, Path("new.hwpx").absolute())
        self.assertEqual(window.saved_path, Path("new.hwpx").absolute())
        self.assertFalse(any(r["op"] == "new" for r in window.worker.requests))
        self.assertTrue(window.info["isDirty"])

    def test_save_before_close_waits_for_ack_and_fresh_clean_info(self):
        window = self.lifecycle_window()
        io = DeferredIO()
        window.io = io
        window.confirm_unsaved_changes = lambda: "save"
        with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=("new.hwpx", "")):
            window.close()
            window.pump()
            self.complete_step(window, info=dict(window.info))
        self.complete_step(window, token=12, buffer=99)
        self.complete_publication(io)
        window.tick()
        self.assertFalse(window.closing)
        self.complete_step(window, isDirty=False)
        self.assertFalse(window.closing)
        self.complete_step(window, info={**window.info, "isDirty": False})
        self.assertTrue(window.closing)
        self.assertIsNone(window.pending)
        self.assertFalse(window.errors)

    def test_save_ack_failure_preserves_document_and_published_path(self):
        window = self.lifecycle_window()
        io = DeferredIO()
        window.io = io
        window.confirm_unsaved_changes = lambda: "save"
        with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=("new.hwpx", "")):
            window.close()
            window.pump()
            self.complete_step(window, info=dict(window.info))
        self.complete_step(window, token=12, buffer=99)
        self.complete_publication(io)
        window.tick()
        window.pending = None
        window.step_done({"ok": False, "error": "ack failed after publication"})
        window.pump()
        self.assertTrue(window.saving)
        self.assertEqual(window.worker.requests[-1]["op"], "cancelSave")
        self.complete_step(window)
        self.assertFalse(window.closing)
        self.assertFalse(window.saving)
        self.assertIsNone(window.intent)
        self.assertTrue(window.info["isDirty"])
        self.assertEqual(window.source_path, Path("original.hwpx"))
        self.assertFalse(any(r["op"] == "new" for r in window.worker.requests))

    def test_failed_save_executor_cancels_token_and_unblocks_transition(self):
        class FailedIO:
            def submit(self, *_args):
                raise RuntimeError("file executor is unavailable")
        window = self.lifecycle_window()
        window.io = FailedIO()
        window.confirm_unsaved_changes = lambda: "save"
        with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=("new.hwpx", "")):
            window.new_document()
            window.pump()
            self.complete_step(window, info=dict(window.info))
        self.complete_step(window, token=12, buffer=99)
        self.assertEqual(window.worker.requests[-1]["op"], "cancelSave")
        self.assertTrue(window.saving)
        self.complete_step(window)
        self.assertFalse(window.saving)
        self.assertIsNone(window.intent)
        self.assertTrue(window.info["isDirty"])

    def test_save_submission_exception_clears_busy_without_discarding_document(self):
        window = self.lifecycle_window()
        def rejected(_request):
            raise RuntimeError("worker submission rejected")
        window.worker.submit = rejected
        window.save_new("new.hwpx")
        self.assertTrue(window.saving)
        window.tick()
        self.assertFalse(window.saving)
        self.assertFalse(window.closing)
        self.assertEqual(window.info["documentId"], 7)
        self.assertTrue(window.info["isDirty"])
        self.assertEqual(window.errors, ["worker submission rejected"])

    def test_document_title_and_actions_follow_completed_state(self):
        window = self.lifecycle_window()
        self.assertIn("original.hwpx *", window.windowTitle())
        self.assertTrue(window.document_actions["undo"].isEnabled())
        self.assertFalse(window.document_actions["redo"].isEnabled())
        window.open_path("other.hwpx")
        self.assertFalse(window.document_actions["new"].isEnabled())
        self.assertFalse(window.document_actions["save"].isEnabled())
        self.assertIn("작업 중", window.document_status.text())
        window.abort_transition(window.intent)
        window.absorb({"info": {**window.info, "isDirty": False, "canUndo": False, "canRedo": True}})
        self.assertNotIn("*", window.windowTitle())
        self.assertFalse(window.document_actions["undo"].isEnabled())
        self.assertTrue(window.document_actions["redo"].isEnabled())

    def test_save_binds_the_revision_after_earlier_accepted_edits(self):
        window = self.lifecycle_window()
        window.command("replace", text="accepted before Save")
        window.save_new("new.hwpx")
        window.pump()
        self.complete_step(window, outcome={"revision": 4})
        self.assertEqual(window.worker.requests[-1], {
            "op": "prepareSave", "target": {"documentId": 7, "revision": 4}, "format": "hwpx"})

    def test_opened_source_save_uses_the_original_stamp_without_a_save_dialog(self):
        from native_worker_file_store import NativeWorkerFileStore
        window = self.lifecycle_window()
        stamp = self.stamp(b"opened bytes")
        window.file_store = NativeWorkerFileStore(7, window.source_path, stamp)
        with patch("qt_native_host.QFileDialog.getSaveFileName", side_effect=AssertionError("unexpected dialog")):
            self.assertTrue(window.save_document())
        window.pump()
        self.assertEqual(window.file_store.source_stamp, stamp)
        self.assertEqual(window.worker.requests[-1]["format"], "hwpx")

    def test_content_loss_rejection_releases_token_before_unblocking(self):
        window = self.lifecycle_window()
        io = DeferredIO()
        window.io = io
        window.confirm_content_loss = lambda _report: False
        window.save_new("new.hwp")
        window.pump()
        self.complete_step(window, token=12, buffer=99, report={"count": 1, "items": [{"reason": "unsupported"}]})
        self.assertEqual(window.worker.requests[-1], {"op": "cancelSave", "documentId": 7, "token": 12})
        self.assertTrue(window.saving)
        self.assertEqual(io.tasks, [])
        self.complete_step(window)
        self.assertFalse(window.saving)
        self.assertFalse(window.file_store.busy)
        self.assertTrue(window.info["isDirty"])

    def test_save_cleanup_has_a_reserved_slot_when_input_queue_is_full(self):
        window = self.lifecycle_window()
        io = DeferredIO()
        window.io = io
        window.save_new("new.hwpx")
        window.pump()
        self.complete_step(window, token=12, buffer=99)
        for _ in range(window.MAX_ACTIONS):
            self.assertTrue(window.enqueue([lambda _: {"op": "replace", "text": "queued edit"}]))
        io.futures[0].set_exception(OSError("disk full"))
        window.tick()
        self.assertEqual(window.worker.requests[-1]["op"], "cancelSave")
        self.assertEqual(len(window.actions), window.MAX_ACTIONS)
        self.assertTrue(window.file_store.busy)
        self.complete_step(window)
        self.assertFalse(window.file_store.busy)
        self.assertFalse(window.saving)
        self.assertEqual(window.worker.requests[-1]["op"], "replace")

    def test_copy_failure_cancels_matching_prepared_token(self):
        window = self.lifecycle_window()
        def cannot_copy(_response):
            raise MemoryError("cannot copy prepared save")
        window.worker.copy_buffer = cannot_copy
        window.save_new("new.hwpx")
        window.pump()
        self.complete_step(window, token=12, buffer=99)
        self.assertTrue(window.saving)
        self.assertEqual(window.worker.requests[-1], {"op": "cancelSave", "documentId": 7, "token": 12})
        self.complete_step(window)
        self.assertFalse(window.saving)
        self.assertFalse(window.file_store.busy)

    def test_save_as_observes_existing_destination_before_overwrite_approval(self):
        window = self.lifecycle_window()
        io = DeferredIO()
        window.io = io
        seen = []
        window.confirm_overwrite = lambda path: seen.append(path) or False
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "existing.hwp"
            destination.write_bytes(b"preserve destination")
            with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=(str(destination), "")):
                self.assertTrue(window.save_dialog())
            self.assertEqual(seen, [])
            self.assertTrue(window.saving)
            io.run(0)
            window.tick()
            self.assertEqual(seen, [destination])
            self.assertFalse(window.saving)
            self.assertFalse(any(r["op"] == "prepareSave" for r in window.worker.requests))
            self.assertEqual(destination.read_bytes(), b"preserve destination")

    def test_exclusive_save_new_never_replaces_the_opened_source(self):
        window = self.lifecycle_window()
        with tempfile.TemporaryDirectory() as directory:
            destination = Path(directory) / "existing.hwpx"
            destination.write_bytes(b"preserve opened source")
            self.assertFalse(window.save_new(destination))
            self.assertEqual(window.worker.requests, [])
            self.assertEqual(destination.read_bytes(), b"preserve opened source")

    def test_unrelated_failed_edit_does_not_cancel_page_geometry_load(self):
        worker = IdleWorker()
        window = self.idle_window(worker)
        window.info = {"documentId": 7, "revision": 0}
        published = []
        window.rebuild_view = lambda: published.append(list(window.pages))
        window.loading_pages = True
        window.load_page_size(0, 2)
        window.pump()
        window.enqueue([lambda _: {"op": "replace", "text": "\t"}], refresh=True)
        window.pending = None
        window.step_done({"ok": True, "size": {"width": 100, "height": 200}})
        window.pump()
        window.pending = None
        window.step_done({"ok": False, "error": "unsupported edit"})
        window.pump()
        self.assertEqual([request["op"] for request in worker.requests],
                         ["pageSize", "replace", "pageSize"])
        self.assertTrue(window.loading_pages)
        window.pending = None
        window.step_done({"ok": True, "size": {"width": 120, "height": 240}})
        window.pump()
        self.assertFalse(window.loading_pages)
        self.assertEqual(published, [[{"width": 100, "height": 200}, {"width": 120, "height": 240}]])
        self.assertEqual(window.errors, ["unsupported edit"])

    def test_click_waits_for_completed_page_geometry(self):
        class ViewHost:
            def __init__(self):
                self.calls = []
            def host_request(self, **request):
                self.calls.append(request)
                return {"query": {"page": 0, "point": {"x": 10, "y": 20}}}
        worker = IdleWorker()
        worker.host = ViewHost()
        window = self.idle_window(worker)
        window.info = {"documentId": 7, "revision": 1}
        window.view = {"visible": [{"page": 1}]}
        window.loading_pages = True
        window.click(QPointF(10, 20), False)
        self.assertEqual(worker.host.calls, [])
        self.assertFalse(window.actions)
        window.loading_pages = False
        window.pages = [{"width": 100, "height": 200}]
        window.view = {"visible": [{"page": 0}]}
        window.page_geometry_key = (7, 1, None)
        window.click(QPointF(10, 20), False)
        window.pump()
        self.assertEqual(len(worker.host.calls), 1)
        self.assertEqual(worker.requests, [{"op": "hitTest", "page": 0, "x": 10, "y": 20,
                                          "target": {"documentId": 7, "revision": 1}}])

    def test_queued_click_does_not_apply_old_geometry_to_a_completed_edit(self):
        class ViewHost:
            def __init__(self):
                self.calls = []
            def host_request(self, **request):
                self.calls.append(request)
                return {"query": {"page": 1, "point": {"x": 10, "y": 20}}}
        worker = IdleWorker()
        worker.host = ViewHost()
        window = self.idle_window(worker)
        window.info = {"documentId": 7, "revision": 1, "fontGeneration": 0, "pageCount": 2}
        window.pages = [{"width": 100, "height": 200}] * 2
        window.view = {"visible": [{"page": 1}]}
        window.page_geometry_key = (7, 1, 0)
        window.command("undo")
        window.pump()
        window.click(QPointF(10, 20), False)
        self.assertEqual([request["op"] for request in worker.requests], ["undo"])
        window.pending = None
        window.step_done({"ok": True, "outcome": {"revision": 2}})
        # Even before the queued refresh runs, new clicks cannot use the old view.
        window.click(QPointF(10, 20), True)
        window.pump()
        self.assertNotIn("hitTest", [request["op"] for request in worker.requests])
        self.assertNotIn("setSelection", [request["op"] for request in worker.requests])
        self.assertEqual(len(worker.host.calls), 1)
        self.assertEqual(window.errors, [])

    def test_queued_click_rechecks_document_font_and_view_generations(self):
        class ViewHost:
            def host_request(self, **request):
                return {"query": {"page": 0, "point": {"x": 10, "y": 20}}}
        for change in ("documentId", "fontGeneration", "view"):
            with self.subTest(change=change):
                worker = IdleWorker()
                worker.host = ViewHost()
                window = self.idle_window(worker)
                window.info = {"documentId": 7, "revision": 1, "fontGeneration": 0}
                window.pages = [{"width": 100, "height": 200}]
                window.view = {"visible": [{"page": 0}]}
                window.page_geometry_key = (7, 1, 0)
                window.click(QPointF(10, 20), False)
                if change == "view":
                    window.epoch += 1
                else:
                    window.absorb({"ok": True, "info": {**window.info, change: 8}})
                window.pump()
                self.assertNotIn("hitTest", [request["op"] for request in worker.requests])
                self.assertEqual(window.errors, [])

    def test_revision_change_reloads_geometry_even_if_page_count_is_unchanged(self):
        worker = IdleWorker()
        window = self.idle_window(worker)
        window.info = {"documentId": 7, "revision": 1, "fontGeneration": 0, "pageCount": 2}
        window.pages = [{"width": 100, "height": 200}] * 2
        window.view = {"visible": []}
        window.page_geometry_key = (7, 1, 0)
        window.absorb({"ok": True, "outcome": {"revision": 2}})
        window.refresh()
        window.pump()
        window.pending = None
        window.step_done({"ok": True, "info": dict(window.info)})
        window.pump()
        self.assertTrue(window.loading_pages)
        self.assertEqual(window.pages, [])
        self.assertEqual([request["op"] for request in worker.requests], ["info", "pageSize"])

    def test_altgr_text_is_inserted_while_control_shortcuts_still_work(self):
        worker = IdleWorker()
        window = self.idle_window(worker)
        window.info = {"documentId": 7, "revision": 0}
        control, alt = Qt.KeyboardModifier.ControlModifier, Qt.KeyboardModifier.AltModifier
        events = [(Qt.Key.Key_Q, "@", control | alt),
                  (Qt.Key.Key_E, "€", control | alt),
                  (Qt.Key.Key_C, "ç", control | alt),
                  (Qt.Key.Key_7, "{", Qt.KeyboardModifier.GroupSwitchModifier)]
        for key, text, modifiers in events:
            event = QKeyEvent(QEvent.Type.KeyPress, key, modifiers, text)
            window.key(event)
            self.assertTrue(event.isAccepted())
        window.key(QKeyEvent(QEvent.Type.KeyPress, Qt.Key.Key_C, control))
        for _ in range(len(events)):
            window.pump()
            window.pending = None
            window.step_done({"ok": True, "outcome": {"revision": window.info["revision"] + 1}})
        window.pump()
        self.assertEqual([(request["op"], request.get("text")) for request in worker.requests],
                         [("replace", text) for _, text, _ in events] + [("copySelection", None)])

    def test_empty_clipboard_paste_is_an_accepted_no_op(self):
        clipboard = self.app.clipboard()
        self.addCleanup(clipboard.clear)
        for contents in ("cleared", "empty", "unsupported"):
            with self.subTest(contents=contents):
                worker = IdleWorker()
                window = self.idle_window(worker)
                window.info = {"documentId": 7, "revision": 1}
                if contents == "cleared":
                    clipboard.clear()
                    self.assertIsNone(clipboard.mimeData())
                else:
                    mime = QMimeData()
                    if contents == "unsupported":
                        mime.setData("application/x-rhwp-test", b"unsupported")
                    clipboard.setMimeData(mime)
                event = QKeyEvent(QEvent.Type.KeyPress, Qt.Key.Key_V, Qt.KeyboardModifier.ControlModifier)
                window.key(event)
                self.assertTrue(event.isAccepted())
                self.assertFalse(window.actions)
                self.assertEqual(worker.requests, [])
                self.assertEqual(window.errors, [])

    def test_clipboard_paste_preserves_text_and_html_payloads(self):
        clipboard = self.app.clipboard()
        self.addCleanup(clipboard.clear)
        for html in (None, "<p>가😀</p>"):
            with self.subTest(html=html):
                worker = IdleWorker()
                window = self.idle_window(worker)
                window.info = {"documentId": 7, "revision": 1}
                mime = QMimeData()
                mime.setText("가😀")
                if html is not None:
                    mime.setHtml(html)
                clipboard.setMimeData(mime)
                event = QKeyEvent(QEvent.Type.KeyPress, Qt.Key.Key_V, Qt.KeyboardModifier.ControlModifier)
                window.key(event)
                window.pump()
                expected = {"op": "pasteHtml", "html": html} if html is not None else {
                    "op": "pasteText", "text": "가😀"}
                self.assertEqual(worker.requests, [{**expected, "target": {"documentId": 7, "revision": 1}}])
                self.assertTrue(event.isAccepted())

    def test_utf16_non_bmp_and_surrogate_rejection(self):
        self.assertEqual(utf16_length("가😀각"), 4)
        self.assertEqual(scalar_from_utf16("가😀각", 3), 2)
        with self.assertRaises(ValueError):
            scalar_from_utf16("가😀각", 2)
        with self.assertRaises(ValueError):
            scalar_from_utf16("가😀각", -1)

    def test_qimage_detaches_premultiplied_rgba_memory(self):
        pixels = bytearray([120, 60, 30, 255] * 4)
        image = QImage(pixels, 2, 2, 8, QImage.Format.Format_RGBA8888_Premultiplied).copy()
        pixels[:] = b"\0" * len(pixels)
        self.assertEqual(image.pixelColor(0, 0).getRgb(), (120, 60, 30, 255))
        image.setDevicePixelRatio(2)
        self.assertEqual(image.deviceIndependentSize().width(), 1)

    def test_widget_queries_use_utf16_and_replacement_offsets_are_rejected(self):
        worker = IdleWorker()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        window.context = {"text": "가😀각", "cursorScalar": 2, "anchorScalar": 1,
                          "selectedText": "😀"}
        self.assertEqual(window.canvas.inputMethodQuery(Qt.InputMethodQuery.ImCursorPosition), 3)
        self.assertEqual(window.canvas.inputMethodQuery(Qt.InputMethodQuery.ImAnchorPosition), 1)
        self.assertEqual(window.canvas.inputMethodQuery(Qt.InputMethodQuery.ImCurrentSelection), "😀")
        event = QInputMethodEvent("", [])
        event.setCommitString("x", -1, 1)
        window.canvas.inputMethodEvent(event)
        self.assertFalse(event.isAccepted())
        self.assertFalse(window.actions)
        self.assertFalse(worker.requests)
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_actual_widget_paint_accepts_owned_frame_and_page_rect(self):
        window = self.tiled_window()
        window.render_next()
        self.complete_tile(window)
        window.worker.next_view = copy.deepcopy(window.view)
        window.show()
        self.app.processEvents()
        self.assertGreater(window.paint_count, 0)
        self.assertEqual(window.canvas.grab().toImage().pixelColor(10, 20).name(), "#ff0000")

    def test_next_queued_edit_uses_completed_revision(self):
        worker = IdleWorker()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        window.info = {"documentId": 7, "revision": 0}
        window.enqueue([lambda _: {"op": "replace", "text": "a"},
                        lambda _: {"op": "replace", "text": "b"}])
        window.pump()
        self.assertEqual(worker.requests[0]["target"]["revision"], 0)
        window.pending = None
        window.step_done({"ok": True, "outcome": {"revision": 1}})
        window.pump()
        self.assertEqual(worker.requests[1]["target"]["revision"], 1)
        self.assertEqual([item["text"] for item in worker.requests], ["a", "b"])
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_begin_edit_info_response_enables_editor_before_next_command(self):
        worker = IdleWorker()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        window.info = {"documentId": 7, "revision": 0}
        position = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        selection = {"anchor": position, "focus": position}
        window.enqueue([lambda _: {"op": "beginEdit", "selection": selection},
                        lambda _: {"op": "replace", "text": "각"}])
        window.pump()
        window.pending = None
        window.step_done({"ok": True, "kind": "info", "info": {
            "documentId": 7, "revision": 0, "editing": True,
            "selection": selection, "compositionId": None}})
        window.pump()
        self.assertTrue(window.info["editing"])
        self.assertEqual(window.info["selection"], selection)
        self.assertEqual(worker.requests[1]["target"], {"documentId": 7, "revision": 0})
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_close_retains_worker_until_native_call_has_finished(self):
        worker = IdleWorker()
        worker.finished = False
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.close()
        window.tick()
        self.assertTrue(window.closing)
        self.assertFalse(window.finished)
        self.assertFalse(worker.released)
        worker.finished = True
        window.tick()
        self.assertTrue(worker.released)
        self.assertTrue(window.finished)

    def test_info_request_has_no_revision_target(self):
        worker = IdleWorker()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        window.info = {"documentId": 7, "revision": 2}
        window.send({"op": "info"}, lambda _: None)
        self.assertEqual(worker.requests, [{"op": "info"}])
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_preedit_cancel_drops_queued_old_underline_range(self):
        worker = IdleWorker()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        window.refresh = lambda: None
        position = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        window.info = {"documentId": 7, "revision": 0, "compositionId": None,
                       "selection": {"anchor": position, "focus": position}}
        window.ime("", "ㄱ", 1)
        window.ime("", "", 0)
        window.pump()
        for revision, composition_id in ((0, 1), (1, 1), (2, None)):
            window.pending = None
            window.step_done({"ok": True, "outcome": {"revision": revision, "compositionId": composition_id}})
            window.pump()
        self.assertEqual([request["op"] for request in worker.requests],
                         ["beginComposition", "updateComposition", "cancelComposition"])
        self.assertEqual(window.preedit, "")
        self.assertFalse(window.preedit_rects)
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_surrounding_window_removes_partial_long_preedit(self):
        window = NativeWindow(None, worker=IdleWorker())
        window.timer.stop()
        window.actions.clear()
        window.preedit = "가" * 5000
        window.preedit_start = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        window.got_context({"context": {
            "text": "가" * 2048 + "😀뒤", "absoluteStartScalar": 2952,
            "cursorScalar": 2048, "anchorScalar": 2048, "selectedText": "",
            "position": {"section": 0, "paragraph": 0, "scalarOffset": 5000, "cellPath": []}}})
        self.assertEqual(window.canvas.inputMethodQuery(Qt.InputMethodQuery.ImSurroundingText), "😀뒤")
        self.assertEqual(window.canvas.inputMethodQuery(Qt.InputMethodQuery.ImCursorPosition), 0)
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_pending_actions_are_bounded_without_claiming_cancellation(self):
        worker = IdleWorker()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        for _ in range(window.MAX_ACTIONS):
            self.assertTrue(window.enqueue([lambda _: {"op": "info"}]))
        self.assertFalse(window.enqueue([lambda _: {"op": "info"}]))
        self.assertEqual(len(window.actions), 64)
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_opened_document_drops_previous_interaction_geometry(self):
        window = NativeWindow(None, worker=IdleWorker())
        window.timer.stop()
        window.actions.clear()
        window.refresh = lambda: None
        window.caret = {"page": 4}
        window.rects = [{"page": 4}]
        window.preedit_rects = [{"page": 4}]
        window.preedit_start = {"paragraph": 9}
        window.drag_anchor = {"paragraph": 9}
        window.preferred_x = 123
        window.view = {"visible": [{"page": 4}]}
        window.info = {"documentId": 7, "revision": 0}
        window.opened({})
        self.assertIsNone(window.caret)
        self.assertEqual(window.rects, [])
        self.assertEqual(window.preedit_rects, [])
        self.assertIsNone(window.preedit_start)
        self.assertIsNone(window.drag_anchor)
        self.assertIsNone(window.preferred_x)
        self.assertEqual(window.view, {})
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_page_geometry_query_does_not_cross_an_edit_revision(self):
        worker = IdleWorker()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        window.refresh = lambda: None
        window.info = {"documentId": 7, "revision": 0, "pageCount": 2}
        window.loading_pages = True
        window.load_page_size(0, 2)
        window.pump()
        window.enqueue([lambda _: {"op": "replace", "text": "x"}])
        window.pending = None
        window.step_done({"ok": True, "size": {"width": 100, "height": 200}})
        window.pump()
        window.pending = None
        window.step_done({"ok": True, "outcome": {"revision": 1}})
        window.pump()
        self.assertEqual([request["op"] for request in worker.requests], ["pageSize", "replace"])
        self.assertFalse(window.loading_pages)
        self.assertEqual(window.pages, [])
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_page_geometry_failure_allows_a_later_reload(self):
        window = NativeWindow(None, worker=IdleWorker())
        window.timer.stop()
        window.actions.clear()
        window.info = {"documentId": 7, "revision": 0}
        window.loading_pages = True
        window.load_page_size(0, 2)
        window.pump()
        window.pending = None
        window.step_done({"ok": False, "error": "geometry unavailable"})
        self.assertFalse(window.loading_pages)
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_page_geometry_is_published_only_after_the_complete_batch(self):
        window = NativeWindow(None, worker=IdleWorker())
        window.timer.stop()
        window.actions.clear()
        window.info = {"documentId": 7, "revision": 0}
        window.loading_pages = True
        published = []
        window.rebuild_view = lambda: published.append(list(window.pages))
        window.load_page_size(0, 2)
        window.pump()
        window.pending = None
        window.step_done({"ok": True, "size": {"width": 100, "height": 200}})
        window.pump()
        self.assertEqual(window.pages, [])
        self.assertEqual(published, [])
        window.pending = None
        window.step_done({"ok": True, "size": {"width": 120, "height": 240}})
        window.pump()
        self.assertEqual(window.pages, [{"width": 100, "height": 200}, {"width": 120, "height": 240}])
        self.assertEqual(published, [window.pages])
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_ime_caret_query_during_geometry_load_uses_no_partial_view(self):
        class NotReadyHost:
            def host_request(self, **request):
                raise AssertionError("partially loaded view must not be queried")
        worker = IdleWorker()
        worker.host = NotReadyHost()
        window = NativeWindow(None, worker=worker)
        window.timer.stop()
        window.actions.clear()
        window.loading_pages = True
        window.pages = [{"width": 100, "height": 200}]
        window.caret = {"page": 0, "x": 10, "y": 20, "height": 16}
        # This query must wait until the native view is complete.
        area = window.canvas.inputMethodQuery(Qt.InputMethodQuery.ImCursorRectangle)
        self.assertTrue(area.isValid())
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

    def test_completed_edit_drops_geometry_from_the_previous_revision(self):
        window = NativeWindow(None, worker=IdleWorker())
        window.timer.stop()
        window.actions.clear()
        window.info = {"documentId": 7, "revision": 1}
        window.caret = {"page": 2}
        window.rects = [{"page": 2}]
        window.preedit_rects = [{"page": 2}]
        window.absorb({"ok": True, "outcome": {"revision": 2}})
        self.assertIsNone(window.caret)
        self.assertEqual(window.rects, [])
        self.assertEqual(window.preedit_rects, [])
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()


def pump(app, predicate, timeout=45):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        app.processEvents()
        if predicate():
            return
        time.sleep(0.002)
    raise TimeoutError("Qt/native flow remains pending; no cancellation was inferred")


def actual_smoke(library, screenshot=None):
    app = QApplication.instance() or QApplication([])
    window = NativeWindow(library)
    window.show()
    window.canvas.setFocus()
    def idle():
        return not window.actions and window.job is None and window.pending is None and not window.io_jobs
    try:
        pump(app, lambda: bool(window.frames) and bool(window.info.get("selection")) and idle())
        assert window.paint_count > 0 and window.frame_count > 0
        assert window.raster_cache_stats["tileEntries"] > 0
        assert window.raster_cache_stats["retainedBytes"] <= 32 * 1024 * 1024
        copied = window.frame_count
        window.scroll(0, 8)
        pump(app, lambda: idle() and bool(window.frames))
        window.scroll(0, -8)
        pump(app, lambda: idle() and bool(window.frames))
        assert window.frame_count == copied, "overlapping scroll recopied retained client tiles"
        cancelled = window.render_cancellations
        window.zoom_at(1.3)
        window.pump()
        assert window.pending_render is not None
        window.zoom_at(1 / 1.3)
        pump(app, lambda: idle() and bool(window.frames))
        assert window.render_cancellations > cancelled
        assert window.frame_count == copied, "stale zoom frame replaced a reusable viewport"
        assert window.tile_images.retained_bytes <= window.tile_images.max_bytes
        for text in ("ㄱ", "가", "각"):
            QApplication.sendEvent(window.canvas, QInputMethodEvent(text, []))
            pump(app, lambda: idle() and window.preedit == text)
            assert window.info["compositionId"] is not None
        commit = QInputMethodEvent("", [])
        commit.setCommitString("각")
        QApplication.sendEvent(window.canvas, commit)
        pump(app, lambda: idle() and window.info.get("compositionId") is None)
        assert window.context["text"] == "각"
        for key, expected in ((Qt.Key.Key_Left, 0), (Qt.Key.Key_Right, 1)):
            QApplication.sendEvent(window.canvas, QKeyEvent(QEvent.Type.KeyPress, key, Qt.KeyboardModifier.NoModifier))
            pump(app, lambda: idle() and window.info["selection"]["focus"]["scalarOffset"] == expected)
        window.command("undo")
        pump(app, lambda: idle() and window.context.get("text") == "")
        window.command("redo")
        pump(app, lambda: idle() and window.context.get("text") == "각")
        window.command("pasteText", text="\n".join(f"페이지 {index}" for index in range(110)))
        pump(app, lambda: idle() and window.info.get("pageCount", 0) > 1
             and len(window.pages) == window.info["pageCount"] and bool(window.frames))
        caret = window.caret_rect()
        assert 0 <= caret.top() < caret.bottom() <= window.canvas.height(), (
            "off-screen caret after multi-page paste", caret, window.state)
        assert window.state["scroll"]["y"] > 0
        window.command("undo")
        pump(app, lambda: idle() and window.context.get("text") == "각"
             and window.info.get("pageCount") == 1 and len(window.pages) == 1 and bool(window.frames))
        control = Qt.KeyboardModifier.ControlModifier
        for key in (Qt.Key.Key_A, Qt.Key.Key_C):
            QApplication.sendEvent(window.canvas, QKeyEvent(QEvent.Type.KeyPress, key, control))
            pump(app, idle)
        assert app.clipboard().text() == "각"
        QApplication.sendEvent(window.canvas, QKeyEvent(QEvent.Type.KeyPress, Qt.Key.Key_X, control))
        pump(app, lambda: idle() and window.context.get("text") == "")
        QApplication.sendEvent(window.canvas, QKeyEvent(QEvent.Type.KeyPress, Qt.Key.Key_V, control))
        pump(app, idle)
        assert window.context.get("text") == "각", (window.context, window.errors, app.clipboard().mimeData().html())
        window.command("undo")
        pump(app, lambda: idle() and window.context.get("text") == "")
        window.command("redo")
        pump(app, lambda: idle() and window.context.get("text") == "각")
        window.zoom_at(1.25)
        pump(app, lambda: bool(window.frames) and idle())
        if screenshot is not None:
            if screenshot.exists():
                raise FileExistsError(screenshot)
            if not window.grab().save(str(screenshot), "PNG"):
                raise RuntimeError("Qt window screenshot could not be saved")
        window.confirm_unsaved_changes = lambda: "cancel"
        original_document = window.info["documentId"]
        window.close()
        pump(app, lambda: window.intent is None and idle())
        assert not window.closing and window.info["documentId"] == original_document
        assert window.info["isDirty"] and window.context["text"] == "각"
        with tempfile.TemporaryDirectory(prefix="rhwp-native-window-") as directory:
            output = Path(directory) / "saved.hwpx"
            window.save_new(output)
            pump(app, lambda: window.saved_path == output and idle())
            assert window.worker.host.read_text(output).strip() == "각"
            document_id = window.info["documentId"]
            window.open_path(output)
            pump(app, lambda: window.info["documentId"] != document_id and idle() and bool(window.frames))
            assert not window.info["isDirty"]
            assert window.source_path == output and output.name in window.windowTitle()
            assert not window.errors, window.errors
            original_bytes = output.read_bytes()
            window.command("replace", text="충돌 시 유지할 편집")
            pump(app, lambda: idle() and window.info["isDirty"])
            output.write_bytes(b"external edit")
            window.save_document()
            pump(app, lambda: not window.saving and idle())
            assert output.read_bytes() == b"external edit"
            assert window.info["isDirty"] and not window.file_store.busy
            assert "충돌 시 유지할 편집" in window.context["text"]
            assert any("changed" in message for message in window.errors), window.errors
            output.write_bytes(original_bytes)
            window.errors.clear()
            document_id = window.info["documentId"]
            window.confirm_unsaved_changes = lambda: "discard"
            window.open_path(output)
            pump(app, lambda: window.info["documentId"] != document_id and idle() and bool(window.frames))
            window.command("replace", text="저장할 내용")
            pump(app, lambda: idle() and window.info["isDirty"])
            saved_text = window.context["text"]
            document_id, revision = window.info["documentId"], window.info["revision"]
            original_io, deferred = window.io, DeferredIO()
            window.io = deferred
            window.confirm_unsaved_changes = lambda: "save"
            saved_before_edit = output  # Save now atomically replaces the opened source.
            try:
                with patch("qt_native_host.QFileDialog.getSaveFileName", return_value=(str(saved_before_edit), "")):
                    window.new_document()
                    pump(app, lambda: bool(deferred.futures) and bool(window.io_jobs))
                window.command("replace", text="추가 변경")
                pump(app, lambda: window.info["revision"] > revision and not window.actions
                     and window.job is None and window.pending is None)
                deferred.run(0)
                pump(app, lambda: window.intent is None and not window.saving and idle())
                assert window.info["documentId"] == document_id and window.info["isDirty"]
                assert window.worker.host.read_text(saved_before_edit).strip() == saved_text.strip()
                assert "추가 변경" in window.context["text"]
                assert window.errors == ["저장 중 추가 변경이 있어 문서 교체 또는 종료를 보류했습니다."], window.errors
            finally:
                window.io = original_io
            window.confirm_unsaved_changes = lambda: "discard"
            window.new_document()
            pump(app, lambda: window.info["documentId"] != document_id and idle() and bool(window.frames))
            assert window.source_path is None and window.context["text"] == ""
        print('{"ok":true,"qtOffscreen":true,"nativeRender":true,"syntheticKoreanIME":true,"clipboard":true,"multiPage":true,"caretVisible":true,"saveReopen":true,"dirtyCloseCancel":true,"saveEditRacePreserved":true,"newDocumentDiscard":true,"externalConflictPreserved":true,"osIMEVerified":false,"tiledViewport":true,"clientTileReuse":true,"renderCancellation":true}')
    finally:
        pump(app, lambda: not window.io_jobs)
        window.confirm_unsaved_changes = lambda: "discard"
        window.close()
        pump(app, lambda: window.finished)
        app.clipboard().clear()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", type=Path)
    parser.add_argument("--screenshot", type=Path)
    args = parser.parse_args()
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(AdapterTests))
    if not result.wasSuccessful():
        raise SystemExit(1)
    if args.library:
        actual_smoke(args.library, args.screenshot)
