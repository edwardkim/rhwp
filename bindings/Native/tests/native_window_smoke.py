#!/usr/bin/env python3
"""Qt offscreen adapter checks; --library adds actual native render/IME/save flow."""
import argparse
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from PySide6.QtCore import QEvent, QMimeData, QPointF, Qt
from PySide6.QtGui import QImage, QInputMethodEvent, QKeyEvent
from PySide6.QtWidgets import QApplication
from qt_native_host import NativeWindow, scalar_from_utf16, utf16_length


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
        window = NativeWindow(None, worker=IdleWorker())
        window.timer.stop()
        window.actions.clear()
        window.view = {"visible": [{"page": 0, "pageRect": {"x": 0, "y": 0, "width": 100, "height": 100}}]}
        image = QImage(2, 2, QImage.Format.Format_RGBA8888_Premultiplied)
        image.fill(Qt.GlobalColor.white)
        window.frames = {0: (QPointF(0, 0), image)}
        window.show()
        self.app.processEvents()
        self.assertGreater(window.paint_count, 0)
        window.finished = True
        window.io.shutdown(wait=True)
        window.close()

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
        with tempfile.TemporaryDirectory(prefix="rhwp-native-window-") as directory:
            output = Path(directory) / "saved.hwpx"
            window.save_new(output)
            pump(app, lambda: window.saved_path == output and idle())
            assert window.worker.host.read_text(output).strip() == "각"
            document_id = window.info["documentId"]
            window.open_path(output)
            pump(app, lambda: window.info["documentId"] != document_id and idle() and bool(window.frames))
            assert not window.info["isDirty"]
        assert not window.errors, window.errors
        print('{"ok":true,"qtOffscreen":true,"nativeRender":true,"syntheticKoreanIME":true,"clipboard":true,"multiPage":true,"caretVisible":true,"saveReopen":true,"osIMEVerified":false}')
    finally:
        pump(app, lambda: not window.io_jobs)
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
