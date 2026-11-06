#!/usr/bin/env python3
"""Snapshot controller/Qt fallback checks; --library adds actual xcb GL flow."""
import argparse
from copy import deepcopy
import json
import os
from pathlib import Path
import sys
import time
from types import SimpleNamespace
import unittest

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_host import NativeCallError
from qt_gpu_view import GpuViewportController, device_failure, transient


class Host:
    def __init__(self):
        self.configurations = []
        self.released = []
        self.error = None

    def host_request(self, **request):
        assert request["op"] == "presentationConfigure"
        if self.error:
            raise NativeCallError(self.error)
        self.configurations.append(deepcopy(request["stamp"]))
        return {"ok": True}

    def presentation_request(self, **request):
        assert request["op"] == "snapshotRelease"
        self.released.append(request["snapshot"])


class Window:
    def __init__(self):
        self.worker = SimpleNamespace(host=Host(), worker=5)
        self.info = {"documentId": 7, "revision": 3, "fontGeneration": 2}
        self.document_generation = 1
        self.epoch = 1
        self.loading_pages = False
        self.state = {"config": {"deviceScale": 1.5, "zoom": 1.25}}
        self.view = {"visible": [{"page": 0, "pageRect": {"x": 12, "y": -7}}]}
        self.canvas = SimpleNamespace(update=lambda: None)
        self.requests = []
        self.errors, self.device_errors = [], []
        self.refreshes = 0

    def send(self, command, done):
        self.requests.append((command, done))

    def fail(self, error):
        self.errors.append(str(error))

    def gpu_failed(self, error):
        self.device_errors.append(str(error))

    def refresh(self):
        self.refreshes += 1

    def update_raster_status(self):
        pass


def snapshot_reply(snapshot=19, **changes):
    info = {"documentId": 7, "revision": 3, "fontGeneration": 2,
            "page": 0, "profile": "screen", "current": True, **changes}
    return {"ok": True, "kind": "renderSnapshot", "documentId": info["documentId"],
            "snapshot": snapshot, "info": info}


def target(**changes):
    return {"contextId": 30, "deviceGeneration": 1, "fbo": 9, "width": 750, "height": 600,
            "format": "rgba8", "origin": "bottomLeft", "sampleCount": 0, "stencilBits": 8, **changes}


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.window = Window()
        self.controller = GpuViewportController(self.window)
        self.host = self.window.worker.host

    def export(self, reply=None):
        self.controller.render_next()
        command, done = self.window.requests[-1]
        done(snapshot_reply() if reply is None else reply)
        return command

    def test_export_binds_exact_source_and_never_requests_pixels(self):
        command = self.export()
        self.assertEqual(command, {"op": "exportRenderSnapshot", "target": {"documentId": 7, "revision": 3},
            "fontGeneration": 2, "page": 0, "profile": "screen"})
        stamp, placements = self.controller.frame_arguments(target())
        self.assertEqual(set(stamp), {"documentId", "revision", "fontGeneration", "viewGeneration",
                                     "deviceGeneration", "deviceScale"})
        self.assertEqual(placements, [{"snapshot": 19, "scale": 1.875,
            "originX": 18, "originY": -10.5, "backgroundColor": 0xffffff}])
        self.assertFalse(self.host.released)

    def test_view_and_new_fbo_reuse_snapshot_but_advance_frame_stamp(self):
        self.export()
        before = self.controller.frame_arguments(target())[0]
        self.window.epoch += 1
        self.window.state["config"].update(deviceScale=2, zoom=2)
        self.window.view["visible"][0]["pageRect"]["y"] = -40
        after, placements = self.controller.frame_arguments(target(fbo=10, width=1000))
        self.assertGreater(after["viewGeneration"], before["viewGeneration"])
        self.assertEqual(placements[0]["snapshot"], 19)
        self.assertEqual(placements[0]["scale"], 4)
        self.controller.render_next()
        self.assertEqual(len(self.window.requests), 1)
        self.assertFalse(self.host.released)

    def test_device_recreation_reuses_immutable_snapshot(self):
        self.export()
        before = self.controller.frame_arguments(target())[0]
        self.controller.device_generation += 1
        after, _ = self.controller.frame_arguments(target(contextId=31, deviceGeneration=2))
        self.assertGreater(after["viewGeneration"], before["viewGeneration"])
        self.assertEqual(self.controller.snapshots, {0: 19})

    def test_source_revision_font_and_document_generation_release(self):
        for field in ("revision", "fontGeneration", "documentId", "document_generation"):
            with self.subTest(field=field):
                self.setUp()
                self.export()
                if field == "document_generation":
                    self.window.document_generation += 1
                else:
                    self.window.info[field] += 1
                self.controller.sync_source()
                self.assertEqual(self.host.released, [19])
                self.assertFalse(self.controller.snapshots)

    def test_offscreen_pages_release_but_overlap_reuses(self):
        self.export()
        self.window.view["visible"].append({"page": 1, "pageRect": {"x": 0, "y": 100}})
        self.export(snapshot_reply(20, page=1))
        self.window.view["visible"].pop(0)
        self.controller.sync_source()
        self.assertEqual(self.host.released, [19])
        self.assertEqual(self.controller.snapshots, {1: 20})

    def test_wait_for_complete_visible_set(self):
        self.window.view["visible"].append({"page": 1, "pageRect": {"x": 0, "y": 100}})
        self.export()
        self.assertIsNone(self.controller.frame_arguments(target()))
        self.export(snapshot_reply(20, page=1))
        self.assertEqual(len(self.controller.frame_arguments(target())[1]), 2)

    def test_late_snapshot_from_replaced_document_is_released(self):
        self.controller.render_next()
        self.window.info["documentId"] = 8
        self.window.requests[0][1](snapshot_reply())
        self.assertEqual(self.host.released, [19])
        self.assertFalse(self.controller.snapshots)

    def test_close_releases_owned_and_late_export_exactly_once(self):
        self.export()
        self.window.view["visible"].append({"page": 1, "pageRect": {"x": 0, "y": 100}})
        self.controller.render_next()
        self.controller.close()
        self.controller.close()
        self.window.requests[-1][1](snapshot_reply(20, page=1))
        self.assertEqual(self.host.released, [19, 20])
        self.assertIsNone(self.controller.frame_arguments(target()))

    def test_snapshot_metadata_failure_is_released_and_diagnostic(self):
        for changes in ({"revision": 2}, {"page": 1}, {"fontGeneration": 4},
                        {"profile": "print"}):
            with self.subTest(changes=changes):
                self.setUp()
                self.export(snapshot_reply(**changes))
                self.assertEqual(self.host.released, [19])
                self.assertTrue(self.window.errors)
                self.assertFalse(self.window.device_errors)

    def test_export_that_became_noncurrent_releases_and_refreshes(self):
        self.export(snapshot_reply(current=False))
        self.assertEqual(self.host.released, [19])
        self.assertEqual(self.window.refreshes, 1)
        self.assertFalse(self.window.device_errors or self.window.errors)

    def test_empty_view_does_not_submit_an_empty_native_frame(self):
        self.window.view["visible"] = []
        self.assertIsNone(self.controller.frame_arguments(target()))

    def test_stale_source_waits_for_refresh_without_cpu_fallback_or_spin(self):
        self.host.error = {"code": "invalidArgument", "error": "stale presentation source document"}
        self.controller.render_next()
        self.controller.render_next()
        self.assertEqual(self.window.refreshes, 1)
        self.assertFalse(self.window.device_errors or self.window.errors or self.window.requests)
        self.host.error = None
        self.window.info["revision"] += 1
        self.controller.render_next()
        self.assertEqual(len(self.window.requests), 1)

    def test_stale_export_releases_handle_and_refreshes(self):
        self.export({"ok": True, "kind": "skipped", "reason": "staleFrame", "snapshot": 19})
        self.assertEqual(self.host.released, [19])
        self.assertEqual(self.window.refreshes, 1)
        self.assertFalse(self.window.device_errors)

    def test_strict_error_and_budget_limit_keep_previous_frame(self):
        for code in ("documentError", "limitExceeded", "invalidArgument"):
            with self.subTest(code=code):
                self.setUp()
                self.export()
                self.controller.frame_arguments(target())
                previous = self.controller.last_frame = {"sentinel": "last good frame"}
                self.controller.diagnose(NativeCallError({"code": code, "error": "strict replay failed"}))
                self.assertIs(self.controller.last_frame, previous)
                self.assertIsNone(self.controller.frame_arguments(target()))
                self.assertTrue(self.window.errors)
                self.assertFalse(self.window.device_errors)
                self.assertFalse(self.host.released)

    def test_only_gpu_unsupported_triggers_device_fallback(self):
        self.controller.diagnose(NativeCallError({"code": "unsupported", "error": "GL lost"}))
        self.assertTrue(self.window.device_errors)
        self.assertFalse(self.window.errors)
        self.assertTrue(transient({"code": "staleFrame", "retryable": True}))
        self.assertFalse(device_failure({"code": "documentError"}))
        self.assertTrue(device_failure({"code": "invalidArgument",
            "error": "matching host GL context is not current"}))

    def test_visible_budget_rejected_before_export(self):
        self.window.view["visible"] = [{"page": page} for page in range(17)]
        self.controller.render_next()
        self.assertFalse(self.window.requests)
        self.assertTrue(self.window.errors)
        self.assertFalse(self.window.device_errors)


class QtFallbackTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        from PySide6.QtWidgets import QApplication
        cls.app = QApplication.instance() or QApplication([])

    def idle_window(self):
        from native_window_smoke import IdleWorker
        from qt_native_host import NativeWindow
        window = NativeWindow(None, worker=IdleWorker())
        window.timer.stop()
        window.actions.clear()
        self.addCleanup(window.io.shutdown, wait=True)
        self.addCleanup(window.close)
        self.addCleanup(setattr, window, "finished", True)
        return window

    def test_auto_uses_cpu_on_offscreen_or_absent_native_capability(self):
        from native_window_smoke import IdleWorker
        from qt_native_host import Canvas, CanvasInputMixin, NativeWindow
        window = NativeWindow(None, worker=IdleWorker(), renderer="auto")
        try:
            self.assertEqual(window.renderer, "cpu")
            self.assertIsInstance(window.canvas, (Canvas, CanvasInputMixin))
            self.assertTrue(window.canvas.rhwp_native_canvas)
            self.assertTrue(window.renderer_error)
        finally:
            window.finished = True
            window.timer.stop()
            window.close()
            window.io.shutdown(wait=True)

    def test_explicit_gl_reports_unavailable_instead_of_cpu(self):
        from native_window_smoke import IdleWorker
        from qt_native_host import NativeWindow
        with self.assertRaisesRegex(RuntimeError, "GL renderer unavailable"):
            NativeWindow(None, worker=IdleWorker(), renderer="gl")

    def test_late_device_failure_replaces_canvas_on_event_loop_and_keeps_source(self):
        from qt_native_host import Canvas
        window = self.idle_window()
        original = window.canvas
        disposed = []
        original.dispose_gpu = lambda: disposed.append(True)
        window.renderer_requested = "auto"
        window.renderer = "gl"
        window.gpu = SimpleNamespace()
        window.info = {"documentId": 7, "revision": 3, "fontGeneration": 2}
        window.gpu_failed("device lost")
        self.assertIs(window.canvas, original)
        self.app.processEvents()
        self.assertEqual(disposed, [True])
        self.assertIsInstance(window.canvas, Canvas)
        self.assertIsNot(window.canvas, original)
        self.assertIsNone(window.gpu)
        self.assertEqual(window.renderer, "cpu")
        self.assertEqual(window.info["revision"], 3)
        self.assertIn("GL → CPU", window.errors[-1])

    def test_explicit_runtime_gl_failure_releases_without_silent_cpu_switch(self):
        window = self.idle_window()
        original = window.canvas
        disposed = []
        original.dispose_gpu = lambda: disposed.append(True)
        window.renderer_requested = window.renderer = "gl"
        window.gpu_failed("device lost")
        self.app.processEvents()
        self.assertEqual(disposed, [True])
        self.assertIs(window.canvas, original)
        self.assertEqual(window.renderer, "gl")
        self.assertIn("GL renderer failed", window.errors[-1])


def pump(app, condition, timeout=30):
    deadline = time.monotonic() + timeout
    while not condition():
        if time.monotonic() >= deadline:
            raise AssertionError("Qt GL condition timed out")
        app.processEvents()
        time.sleep(0.002)


def actual_smoke(library, diagnostic_readback=False):
    from PySide6.QtWidgets import QApplication
    from qt_native_host import NativeWindow
    app = QApplication.instance() or QApplication([])
    if app.platformName() != "xcb":
        raise RuntimeError("actual GL check requires QT_QPA_PLATFORM=xcb and a running X server")
    window = NativeWindow(library, renderer="gl")
    window.show()
    try:
        pump(app, lambda: window.gpu.last_frame is not None or bool(window.errors))
        assert not window.errors, window.errors
        assert window.gpu.last_info["wholeFrameReadbacks"] == 0
        initial_exports = window.gpu.exports
        initial_snapshots = dict(window.gpu.snapshots)
        frame = window.frame_count
        window.zoom_at(1.2)
        pump(app, lambda: window.frame_count > frame or bool(window.errors))
        assert not window.errors, window.errors
        assert window.gpu.snapshots == initial_snapshots and window.gpu.exports == initial_exports
        frame = window.frame_count
        window.resize(1000, 720)
        pump(app, lambda: window.frame_count > frame or bool(window.errors))
        assert not window.errors, window.errors
        # This optional test readback is explicitly outside normal presentation.
        if diagnostic_readback:
            image = window.canvas.grabFramebuffer()
            assert not image.isNull()
        window.command("pasteText", text="Native GL 한글 😀")
        pump(app, lambda: window.gpu.source[2] == window.info["revision"] and
             window.info["revision"] > 0 and window.gpu.exports > initial_exports and
             window.gpu.last_frame["stamp"]["revision"] == window.info["revision"] or bool(window.errors))
        assert not window.errors, window.errors
        assert window.gpu.last_info["wholeFrameReadbacks"] == 0
        print(json.dumps({"renderer": "gl", "driver": window.canvas.driver,
            "frames": window.frame_count, "exports": window.gpu.exports,
            "wholeFrameReadbacks": 0, "diagnosticReadback": diagnostic_readback}, ensure_ascii=False))
    finally:
        window.shutdown_worker()
        pump(app, lambda: window.finished)
        assert not window.gpu.snapshots


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library")
    parser.add_argument("--diagnostic-readback", action="store_true")
    args = parser.parse_args()
    result = unittest.TextTestRunner().run(unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__]))
    if not result.wasSuccessful():
        return 1
    if args.library:
        actual_smoke(args.library, args.diagnostic_readback)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
