#!/usr/bin/env python3
"""Small Qt/native worker reference window. No WASM, DOM or GUI-thread layout."""
import argparse
import json
from collections import deque
from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
from dataclasses import dataclass
import math
from pathlib import Path
import sys
import time

from PySide6.QtCore import QEvent, QMimeData, QPoint, QPointF, QRectF, Qt, QTimer
from PySide6.QtGui import QAction, QColor, QImage, QKeySequence, QPainter, QPen
from PySide6.QtWidgets import QApplication, QFileDialog, QGridLayout, QLabel, QMainWindow, QMessageBox, QScrollBar, QWidget

sys.path.insert(0, str(Path(__file__).resolve().parent))
from native_file_store import read_document, read_recovery
from native_worker import NativeWorker
from native_worker_file_store import NativeWorkerFileStore
from native_viewport_tiles import NATIVE_CACHE_BYTES, TileImageCache, frame_byte_cost, plan_visible_tiles
from native_accessibility import AccessibilityState, QtAccessibilityBinding, ensure_qt_accessibility_factory, selection_identity


def utf16_length(text):
    return len(text.encode("utf-16-le")) // 2


def scalar_from_utf16(text, offset):
    """Reject an IME offset in the middle of a surrogate pair."""
    units = 0
    for index, char in enumerate(text):
        if units == offset:
            return index
        units += utf16_length(char)
    if units == offset:
        return len(text)
    raise ValueError("IME offset is not a Unicode scalar boundary")


def body():
    return {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}


def altgr_modifiers(modifiers):
    return bool(modifiers & Qt.KeyboardModifier.GroupSwitchModifier) or bool(
        modifiers & Qt.KeyboardModifier.ControlModifier and modifiers & Qt.KeyboardModifier.AltModifier)


@dataclass
class NativeAction:
    steps: deque
    generation: int
    document_id: object
    done: object = None
    refresh: bool = False
    last_result: object = None
    page_load_token: object = None
    failed: object = None
    replacement: bool = False


@dataclass
class CachedTile:
    image: QImage
    origin_x: int
    origin_y: int


class CanvasInputMixin:
    """Shared input/IME and overlays; contains no QObject base or renderer."""

    rhwp_native_canvas = True
    _rhwp_accessibility_document = True

    def __init__(self, window):
        super().__init__(window)
        self.window = window
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setAttribute(Qt.WidgetAttribute.WA_InputMethodEnabled)
        self.setMinimumSize(200, 150)

    def paint_overlays(self, painter):
        if self.window.view and self.window.pages and not self.window.loading_pages:
            painter.setPen(Qt.PenStyle.NoPen)
            painter.setBrush(QColor(50, 120, 230, 75))
            for rect in self.window.rects:
                painter.drawRect(self.window.page_rect(rect["page"], rect["bounds"]))
            if self.window.caret and self.hasFocus() and (not self.window.preedit or self.window.preedit_cursor_visible):
                painter.setPen(QPen(QColor("#202020"), 1))
                caret = self.window.caret_rect()
                painter.drawLine(caret.topLeft(), caret.bottomLeft())
            for rect in self.window.preedit_rects:
                area = self.window.page_rect(rect["page"], rect["bounds"])
                painter.setPen(QPen(self.window.preedit_color, 1, Qt.PenStyle.DotLine))
                painter.drawLine(area.bottomLeft(), area.bottomRight())

    def resizeEvent(self, event):
        self.window.rebuild_view()
        super().resizeEvent(event)

    def event(self, event):
        if event.type() == QEvent.Type.ShortcutOverride and (
                altgr_modifiers(event.modifiers()) or self.window.is_composing()):
            # Give the focused editor first refusal before a window QAction can
            # consume AltGr text or a composition-cancelling Undo/Escape.
            event.accept()
            return True
        if event.type() == QEvent.Type.DevicePixelRatioChange:
            self.window.rebuild_view()
        return super().event(event)

    def wheelEvent(self, event):
        if event.modifiers() & Qt.KeyboardModifier.ControlModifier:
            factor = 1.1 if event.angleDelta().y() > 0 else 1 / 1.1
            self.window.zoom_at(factor, event.position())
        else:
            self.window.scroll(0, -event.angleDelta().y() / 2)
        event.accept()

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.setFocus()
            self.window.start_drag(event.position())

    def mouseMoveEvent(self, event):
        if event.buttons() & Qt.MouseButton.LeftButton:
            self.window.drag_to(event.position())

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.window.end_drag(event.position())

    def keyPressEvent(self, event):
        self.window.key(event)

    def inputMethodEvent(self, event):
        if event.replacementStart() or event.replacementLength():
            self.window.fail("이 예제는 IME 주변 문자열 치환 offset을 지원하지 않습니다.")
            event.ignore()
            return
        cursor = len(event.preeditString())
        cursor_visible = True
        color = QColor("#202020")
        try:
            for attribute in event.attributes():
                if attribute.type == event.AttributeType.Cursor:
                    cursor = scalar_from_utf16(event.preeditString(), attribute.start)
                    cursor_visible = attribute.length != 0
                elif attribute.type == event.AttributeType.TextFormat:
                    fmt = attribute.value
                    if hasattr(fmt, "underlineColor") and fmt.underlineColor().isValid():
                        color = fmt.underlineColor()
        except ValueError as error:
            self.window.fail(str(error))
            event.ignore()
            return
        self.window.preedit_color = color
        if self.window.ime(event.commitString(), event.preeditString(), cursor, cursor_visible):
            event.accept()
        else:
            event.ignore()

    def inputMethodQuery(self, query):
        context = self.window.context
        text = context.get("text", "")
        cursor = context.get("cursorScalar", 0)
        anchor = context.get("anchorScalar")
        values = {
            Qt.InputMethodQuery.ImEnabled: bool(self.window.info.get("selection")),
            Qt.InputMethodQuery.ImCursorRectangle: self.window.caret_rect(),
            Qt.InputMethodQuery.ImSurroundingText: text,
            Qt.InputMethodQuery.ImCursorPosition: utf16_length(text[:cursor]),
            Qt.InputMethodQuery.ImAnchorPosition: utf16_length(text[:cursor if anchor is None else anchor]),
            Qt.InputMethodQuery.ImCurrentSelection: context.get("selectedText", ""),
            Qt.InputMethodQuery.ImAbsolutePosition: context.get("absoluteStartScalar", 0) + cursor,
        }
        # Absolute position outside this bounded context cannot be converted to
        # UTF-16 correctly. Leave that optional query to Qt's default instead.
        values.pop(Qt.InputMethodQuery.ImAbsolutePosition)
        return values[query] if query in values else super().inputMethodQuery(query)

    def focusOutEvent(self, event):
        self.window.stop_drag()
        if event.reason() not in (Qt.FocusReason.PopupFocusReason, Qt.FocusReason.MenuBarFocusReason):
            self.window.cancel_composition(reset=True)
        super().focusOutEvent(event)


class Canvas(CanvasInputMixin, QWidget):
    def paintEvent(self, event):
        painter = QPainter(self)
        painter.fillRect(self.rect(), QColor("#55575a"))
        page_rects = {}
        for page in self.window.view.get("visible", []):
            bounds = page["pageRect"]
            page_rects[page["page"]] = bounds
            painter.fillRect(QRectF(bounds["x"], bounds["y"], bounds["width"], bounds["height"]), Qt.GlobalColor.white)
        for tile in self.window.visible_tiles:
            frame = self.window.tile_images.get(tile.key(self.window.tile_identity))
            bounds = page_rects.get(tile.page)
            if frame is not None and bounds is not None:
                painter.save()
                factor = self.window.state["config"]["zoom"] / tile.scale
                origin = QPointF(bounds["x"] + frame.origin_x * factor, bounds["y"] + frame.origin_y * factor)
                painter.setClipRect(QRectF(bounds["x"], bounds["y"], bounds["width"], bounds["height"]),
                                    Qt.ClipOperation.IntersectClip)
                painter.setClipRect(QRectF(bounds["x"] + tile.x * factor, bounds["y"] + tile.y * factor,
                    tile.width * factor, tile.height * factor), Qt.ClipOperation.IntersectClip)
                painter.drawImage(origin, frame.image)
                painter.restore()
        self.paint_overlays(painter)
        painter.end()
        self.window.paint_count += 1


class NativeWindow(QMainWindow):
    MAX_ACTIONS = 64
    MAX_PAGES = 512
    MAX_DRAG_ACTIONS = 4
    MAX_DRAG_SCROLL = 32
    STANDARD_ACTIONS = (
        ("new", "새 문서", QKeySequence.StandardKey.New, "file"),
        ("open", "열기", QKeySequence.StandardKey.Open, "file"),
        ("save", "저장", QKeySequence.StandardKey.Save, "file"),
        ("saveAs", "다른 이름으로 저장", QKeySequence.StandardKey.SaveAs, "file"),
        ("undo", "실행 취소", QKeySequence.StandardKey.Undo, "edit"),
        ("redo", "다시 실행", QKeySequence.StandardKey.Redo, "edit"),
        ("cut", "잘라내기", QKeySequence.StandardKey.Cut, "edit"),
        ("copy", "복사", QKeySequence.StandardKey.Copy, "edit"),
        ("paste", "붙여넣기", QKeySequence.StandardKey.Paste, "edit"),
        ("selectAll", "전체 선택", QKeySequence.StandardKey.SelectAll, "edit"),
        ("zoomIn", "확대", QKeySequence.StandardKey.ZoomIn, "view"),
        ("zoomOut", "축소", QKeySequence.StandardKey.ZoomOut, "view"),
    )

    def __init__(self, library, input_path=None, output_path=None, worker=None, renderer="cpu",
                 recovery_dir=None, recovery_interval=30.0):
        super().__init__()
        if renderer not in ("cpu", "auto", "gl"):
            raise ValueError("renderer must be cpu, auto or gl")
        if not 1 <= recovery_interval <= 86400:
            raise ValueError("recovery interval must be between 1 and 86400 seconds")
        self.recovery_dir = Path(recovery_dir).absolute() if recovery_dir else None
        self.recovery_interval = recovery_interval
        self.next_recovery_attempt = 0.0
        self.recovery_path = None
        ensure_qt_accessibility_factory()
        self.worker = worker or NativeWorker(library)
        self.renderer_requested = renderer
        self.renderer = "cpu"
        self.renderer_error = None
        self.gpu = None
        self.gpu_failure_timer = QTimer(self)
        self.gpu_failure_timer.setSingleShot(True)
        self.gpu_failure_timer.timeout.connect(self.apply_gpu_failure)
        self.output_path = Path(output_path) if output_path else None
        self.info, self.context, self.view = {}, {}, {}
        self.pages, self.frames, self.rects, self.preedit_rects = [], {}, [], []
        self.tile_images = TileImageCache()
        self.tile_identity = None
        self.visible_tiles = ()
        self.tile_failures = set()
        self.pending_render = None
        self.render_cancel_requested = False
        self.render_cancellations = 0
        self.raster_cache_stats = {}
        self.cache_stats_epoch = -1
        self.loading_pages = False
        self.page_load_token = None
        self.page_geometry_key = None
        self.caret = None
        self.actions = deque()
        self.pending = None
        self.job = None
        self.epoch = 0
        self.drag_anchor = None
        self.drag_token = None
        self.drag_point = None
        self.drag_document = None
        self.drag_hit_pending = False
        self.drag_timer = QTimer(self)
        self.drag_timer.setInterval(40)
        self.drag_timer.timeout.connect(self.autoscroll_drag)
        self.preedit = ""
        self.ime_pending = 0
        self.preedit_start = None
        self.preedit_cursor = 0
        self.preedit_cursor_visible = True
        self.preedit_color = QColor("#202020")
        self.preferred_x = None
        self.errors = []
        self.paint_count = self.frame_count = 0
        self.saved_path = None
        self.source_path = None
        self.file_store = None
        self.save_operation = None
        self.document_generation = 0
        self.intent = None
        self.intent_serial = 0
        self.transition_phase = None
        self.saving = False
        self.closing = self.finished = False
        self.io = ThreadPoolExecutor(max_workers=1, thread_name_prefix="rhwp-file-io")
        self.io_jobs = []
        self.state = {"config": {"pages": [], "viewport": {"width": 800, "height": 600},
                                 "zoom": 1, "deviceScale": 1, "pageGap": 18},
                      "scroll": {"x": 0, "y": 0}}
        self.semantic = AccessibilityState()
        self.canvas = Canvas(self)
        if renderer != "cpu":
            try:
                from qt_gpu_view import create_gpu_canvas
                gpu_canvas = create_gpu_canvas(self, CanvasInputMixin)
                self.canvas.deleteLater()
                self.canvas = gpu_canvas
                self.renderer = "gl"
            except Exception as error:
                self.renderer_error = str(error)
                if renderer == "gl":
                    self.io.shutdown(wait=False)
                    if worker is None:
                        self.worker.close()
                        self.worker.wait_closed()
                    raise RuntimeError(f"GL renderer unavailable: {error}") from error
        self.semantic_binding = QtAccessibilityBinding(self.canvas, self.semantic, self.map_accessible_rectangle)
        self.vertical = QScrollBar(Qt.Orientation.Vertical)
        self.horizontal = QScrollBar(Qt.Orientation.Horizontal)
        central = QWidget()
        layout = QGridLayout(central)
        self.canvas_layout = layout
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self.canvas, 0, 0)
        layout.addWidget(self.vertical, 0, 1)
        layout.addWidget(self.horizontal, 1, 0)
        self.setCentralWidget(central)
        self.vertical.valueChanged.connect(lambda value: self.set_scroll(y=value))
        self.horizontal.valueChanged.connect(lambda value: self.set_scroll(x=value))
        toolbar = self.addToolBar("문서")
        self.document_actions = {}
        menus = {name: self.menuBar().addMenu(label) for name, label in
                 (("file", "파일"), ("edit", "편집"), ("view", "보기"))}
        for name, label, standard, menu in self.STANDARD_ACTIONS:
            action = QAction(label, self)
            action.setShortcuts(QKeySequence.keyBindings(standard))
            action.triggered.connect(lambda _checked=False, name=name: self.invoke_action(name))
            self.addAction(action)
            menus[menu].addAction(action)
            if name in ("new", "open", "save", "undo", "redo", "zoomIn", "zoomOut"):
                toolbar.addAction(action)
            self.document_actions[name] = action
        recovery_action = QAction("복구 파일 열기", self)
        recovery_action.triggered.connect(lambda _checked=False: self.invoke_action("recovery"))
        menus["file"].addAction(recovery_action)
        toolbar.addAction(recovery_action)
        self.document_actions["recovery"] = recovery_action
        self.document_status = QLabel()
        self.statusBar().addPermanentWidget(self.document_status)
        self.raster_status = QLabel()
        self.statusBar().addPermanentWidget(self.raster_status)
        self.resize(900, 700)
        self.update_document_ui()
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.tick)
        self.timer.start(8)
        if input_path:
            self.open_path(input_path)
        else:
            self.enqueue([lambda _: {"op": "new", "replace": None}], self.opened)
        if self.renderer_error:
            self.statusBar().showMessage(f"GL → CPU: {self.renderer_error}")

    def gpu_failed(self, error):
        """Defer widget replacement until paintGL/aboutToBeDestroyed returns."""
        self.renderer_error = str(error)
        self.gpu_failure_timer.start(0)

    def apply_gpu_failure(self):
        if self.closing or self.renderer != "gl":
            return
        if self.renderer_requested == "gl":
            self.canvas.dispose_gpu()
            self.fail(f"GL renderer failed: {self.renderer_error}")
            return
        previous = self.canvas
        focused = previous.hasFocus()
        previous.dispose_gpu()
        self.gpu = None
        self.canvas = Canvas(self)
        self.semantic_binding = QtAccessibilityBinding(self.canvas, self.semantic, self.map_accessible_rectangle)
        self.canvas_layout.replaceWidget(previous, self.canvas)
        previous.hide()
        previous.deleteLater()
        self.renderer = "cpu"
        self.reset_tiles()
        self.rebuild_view()
        self.canvas.show()
        if focused:
            self.canvas.setFocus()
        self.fail(f"GL → CPU: {self.renderer_error}")

    def target(self):
        return {"documentId": self.info["documentId"], "revision": self.info["revision"]}

    def geometry_key(self):
        return tuple(self.info.get(key) for key in ("documentId", "revision", "fontGeneration"))

    def fail(self, message):
        self.errors.append(str(message))
        self.statusBar().showMessage(str(message))

    def update_document_ui(self):
        name = self.source_path.name if self.source_path else "제목 없음"
        dirty = bool(self.info.get("isDirty"))
        busy = self.closing or self.intent is not None or self.saving
        self.setWindowTitle(f"{name}{' *' if dirty else ''} · rhwp Native · Qt reference")
        self.document_status.setText(" · ".join(filter(None, (
            "수정됨" if dirty else "저장된 상태", "작업 중" if busy else None))))
        available = not self.closing and self.intent is None
        self.document_actions["new"].setEnabled(available and not self.saving)
        self.document_actions["open"].setEnabled(available and not self.saving)
        self.document_actions["recovery"].setEnabled(available and not self.saving)
        for name in ("save", "saveAs"):
            self.document_actions[name].setEnabled(available and bool(self.info) and not self.saving)
        for name, capability in (("undo", "canUndo"), ("redo", "canRedo")):
            self.document_actions[name].setEnabled(available and (
                bool(self.info.get(capability)) or (name == "undo" and self.is_composing())))
        for name in ("copy", "cut", "paste", "selectAll"):
            self.document_actions[name].setEnabled(available and bool(self.info))
        for name in ("zoomIn", "zoomOut"):
            self.document_actions[name].setEnabled(available and bool(self.pages))
        # Standalone Save As permits later edits. A pending replacement/close
        # freezes new UI input while its FIFO barrier checks the accepted edits.
        self.canvas.setEnabled(available and bool(self.info))

    def enqueue(self, steps, done=None, refresh=False, page_load_token=None, failed=None, replacement=False,
                save_control=False):
        if self.closing:
            return False
        # A single save owns one reserved cleanup/ack slot. Its completion cannot
        # be starved by the ordinary bounded input queue. Edit order is unchanged.
        if len(self.actions) + (self.job is not None) >= self.MAX_ACTIONS + int(save_control):
            self.fail("입력 큐가 가득 찼습니다. 진행 중인 작업은 취소되지 않았습니다.")
            return False
        self.cancel_render()
        action = NativeAction(deque(steps), self.document_generation,
            self.info.get("documentId"), done=done, refresh=refresh,
            page_load_token=page_load_token, failed=failed, replacement=replacement)
        (self.actions.appendleft if save_control else self.actions.append)(action)
        return True

    def tick(self):
        if self.closing:
            if self.worker.is_finished():
                self.worker.release()
                self.finished = True
                self.timer.stop()
                self.io.shutdown(wait=False)
                self.close()
            return
        try:
            for future, callback in tuple(self.io_jobs):
                if future.done():
                    self.io_jobs.remove((future, callback))
                    callback(future)
            for _ in range(16):
                result = self.worker.poll()
                if result is None:
                    break
                if self.pending is None or result["requestId"] != self.pending[0]:
                    if "buffer" in result:
                        self.release_frame(result)
                    if "snapshot" in result:
                        self.worker.host.presentation_request(op="snapshotRelease", snapshot=result["snapshot"])
                    self.fail("예상하지 못한 worker completion")
                    continue
                callback = self.pending[1]
                self.pending = None
                callback(result)
            self.maybe_recover()
            self.pump()
            if self.pending is None and self.job is None and not self.actions:
                self.request_semantics()
            self.update_document_ui()
        except Exception as error:
            failed = self.job.failed if self.job is not None else None
            self.pending = None
            self.job = None
            self.fail(error)
            if failed:
                failed({"ok": False, "error": str(error)})
            if self.intent is not None:
                self.abort_transition(self.intent)

    def send(self, request, callback):
        request = dict(request)
        if "_open" in request:
            request_id = self.worker.open_bytes(request["_open"], replace=self.target() if self.info else None)
        elif "_openRecovery" in request:
            request_id = self.worker.open_recovery_bytes(request["_openRecovery"],
                replace=self.target() if self.info else None)
        else:
            if request["op"] not in ("new", "info", "acknowledgeSave", "cancelSave"):
                request.setdefault("target", self.target())
            request_id = self.worker.submit(request)
        self.pending = (request_id, callback)

    def pump(self):
        if self.closing or self.pending is not None:
            return
        if self.job is None and self.actions:
            self.job = self.actions.popleft()
        if self.job is not None and (self.job.generation != self.document_generation or
                (self.job.document_id is not None and self.job.document_id != self.info.get("documentId"))):
            discarded = self.job
            self.job = None
            if discarded.failed:
                discarded.failed({"ok": False, "kind": "skipped", "error": "document replaced before queued action"})
            if discarded.refresh:
                self.refresh()
            return self.pump()
        if self.job is not None:
            steps, done, refresh, last = self.job.steps, self.job.done, self.job.refresh, self.job.last_result
            if steps:
                request = steps.popleft()(last)
                if request is None:
                    return self.pump()
                self.send(request, self.step_done)
            else:
                self.job = None
                if done:
                    done(last)
                if refresh:
                    self.refresh()
                self.pump()
            return
        self.render_next()

    def absorb(self, result):
        geometry_keys = ("documentId", "revision", "fontGeneration", "selection", "compositionId")
        previous = tuple(self.info.get(key) for key in geometry_keys)
        if result.get("info"):
            self.info = result["info"]
        if "preferredX" in result:
            self.preferred_x = result["preferredX"]
        outcome = result.get("outcome")
        if outcome:
            if "outcome" in outcome:
                self.preferred_x = outcome.get("preferredX")
                outcome = outcome["outcome"]
            elif "preferredX" not in result:
                self.preferred_x = None
            for key in ("revision", "selection", "compositionId"):
                if key in outcome:
                    self.info[key] = outcome[key]
        if previous != tuple(self.info.get(key) for key in geometry_keys):
            self.caret = None
            self.context.clear()
            self.rects.clear()
            self.preedit_rects.clear()
            QApplication.inputMethod().update(Qt.InputMethodQuery.ImQueryAll)
        if previous[:3] != self.geometry_key():
            self.stop_drag()
            self.reset_tiles()
        self.update_document_ui()
        self.semantic.invalidate(self.accessibility_key())

    def step_done(self, result):
        if not result.get("ok") or result.get("kind") == "skipped":
            self.fail(result.get("error", result.get("reason", "native request failed")))
            if (self.loading_pages and self.page_load_token is not None
                    and self.job.page_load_token is self.page_load_token):
                self.loading_pages = False
                self.page_load_token = None
                self.pages.clear()
            failed = self.job.failed
            self.job = None
            if failed:
                failed({**result, "_completionReceived": True})
            return
        self.absorb(result)
        if self.job.replacement:
            self.job.document_id = self.info.get("documentId")
        self.job.last_result = result

    def opened(self, result, path=None, source_stamp=None):
        self.stop_drag()
        self.document_generation += 1
        self.semantic.invalidate(self.accessibility_key())
        self.reset_tiles()
        self.source_path = Path(path) if path is not None else None
        self.saved_path = None
        self.recovery_path = None
        self.file_store = NativeWorkerFileStore(self.info["documentId"],
            self.source_path if source_stamp is not None else None, source_stamp)
        # Actions were accepted for the previous document, including those whose
        # target is filled lazily. They must never migrate to its replacement.
        self.actions.clear()
        self.frames.clear()
        self.pages.clear()
        self.loading_pages = False
        self.page_load_token = None
        self.page_geometry_key = None
        self.view = {}
        self.state["config"]["pages"] = []
        self.state["scroll"] = {"x": 0, "y": 0}
        self.context.clear()
        self.preedit = ""
        self.ime_pending = 0
        self.preedit_start = None
        self.preedit_cursor = 0
        self.preedit_rects.clear()
        self.caret = None
        self.rects.clear()
        self.drag_anchor = None
        self.preferred_x = None
        self.update_document_ui()
        self.enqueue([lambda _: {"op": "beginEdit", "selection": {"anchor": body(), "focus": body()}}],
                     refresh=True)
        self.refresh()
        self.enqueue([lambda _: {"op": "setRasterCacheLimit", "maxBytes": NATIVE_CACHE_BYTES}])

    def refresh(self):
        if not self.info or self.closing or self.intent is not None:
            return
        def got_info(result):
            count = self.info["pageCount"]
            if count > self.MAX_PAGES:
                self.fail(f"이 예제는 최대 {self.MAX_PAGES}쪽까지 표시합니다.")
                return
            if (len(self.pages) != count or self.page_geometry_key != self.geometry_key()) and not self.loading_pages:
                self.loading_pages = True
                self.pages = []
                self.load_page_size(0, count)
            elif not self.loading_pages:
                self.rebuild_view()
            if self.info.get("selection"):
                self.enqueue_input_query(lambda: {"op": "imeContext"}, self.got_context)
                self.enqueue_input_query(lambda: {"op": "caretRect", "position": self.ime_caret_position()},
                                         self.got_caret)
                self.enqueue_input_query(lambda: {"op": "selectionRects", "selection": self.info["selection"]},
                                         lambda reply: self.set_rects(reply["rects"]))
        self.enqueue([lambda _: {"op": "info"}], got_info)

    def input_geometry_key(self):
        return (self.document_generation, self.geometry_key(), deepcopy(self.info.get("selection")),
                self.info.get("compositionId"), deepcopy(self.preedit_start), self.preedit_cursor)

    def enqueue_input_query(self, request, done):
        expected = []
        def query(_):
            expected[:] = [self.input_geometry_key()]
            return request()
        def received(reply):
            if reply is not None and expected and expected[0] == self.input_geometry_key():
                done(reply)
        return self.enqueue([query], received)

    def load_page_size(self, page, count, load=None):
        if load is None:
            load = (object(), self.geometry_key(), [])
            self.page_load_token = load[0]
        token, target, sizes = load
        if token is not self.page_load_token:
            return
        if target != self.geometry_key():
            self.loading_pages = False
            self.page_load_token = None
            self.pages.clear()
            self.refresh()
            return
        if page >= count:
            self.pages = sizes
            self.page_geometry_key = target
            self.loading_pages = False
            self.page_load_token = None
            self.rebuild_view()
            return
        def loaded(result):
            if token is not self.page_load_token:
                return
            if target != self.geometry_key():
                self.load_page_size(page, count, load)
                return
            sizes.append(result["size"])
            self.load_page_size(page + 1, count, load)
        def request(_):
            if token is self.page_load_token and target == self.geometry_key():
                return {"op": "pageSize", "page": page}
        if not self.enqueue([request], loaded, page_load_token=token):
            self.loading_pages = False
            self.page_load_token = None
            self.pages.clear()

    def got_context(self, result):
        self.context = dict(result["context"])
        # Qt's surrounding document excludes its separate preedit string. Native
        # preview is real temporary document text, so remove it from this query.
        position = self.context.get("position", {})
        if self.preedit and self.preedit_start and all(position.get(key) == self.preedit_start[key] for key in ("section", "paragraph")):
            start = self.preedit_start["scalarOffset"] - self.context.get("absoluteStartScalar", 0)
            text = self.context["text"]
            left, right = max(0, start), min(len(text), start + len(self.preedit))
            if left <= right and text[left:right] == self.preedit[left - start:right - start]:
                self.context.update(text=text[:left] + text[right:],
                                    cursorScalar=left, anchorScalar=left, selectedText="")
        QApplication.inputMethod().update(Qt.InputMethodQuery.ImQueryAll)

    def ime_caret_position(self):
        if self.preedit and self.preedit_start:
            position = dict(self.preedit_start)
            position["scalarOffset"] += self.preedit_cursor
            return position
        return self.info["selection"]["focus"]

    def got_caret(self, result):
        self.caret = result["caret"]
        self.ensure_caret_visible()
        self.canvas.update()
        QApplication.inputMethod().update(Qt.InputMethodQuery.ImCursorRectangle)

    def ensure_caret_visible(self):
        if self.caret and self.pages and self.view and not self.loading_pages and self.drag_token is None:
            self.apply_view(action="ensureCaret", page=self.caret["page"], margin=12,
                            rect={"x": self.caret["x"], "y": self.caret["y"],
                                  "width": self.caret.get("width", 0), "height": self.caret["height"]})

    def set_rects(self, rects):
        self.rects = rects
        self.canvas.update()

    def rebuild_view(self):
        if not hasattr(self, "canvas") or not self.pages:
            return
        self.state["config"].update(pages=self.pages, viewport={"width": max(1, self.canvas.width()),
            "height": max(1, self.canvas.height())}, deviceScale=self.canvas.devicePixelRatioF())
        self.apply_view(action="state")
        self.ensure_caret_visible()

    def apply_view(self, **command):
        try:
            view = self.worker.host.host_request(op="view", state=self.state, command=command)
            self.cancel_render()
            self.view = view
            self.state = self.view["state"]
            self.epoch += 1
            self.tile_failures.clear()
            self.sync_visible_tiles()
            for bar, axis in ((self.horizontal, "x"), (self.vertical, "y")):
                bar.blockSignals(True)
                bar.setRange(0, int(self.view["scrollLimit"][axis] + 0.5))
                bar.setValue(int(self.state["scroll"][axis]))
                bar.blockSignals(False)
            self.canvas.update()
            QApplication.inputMethod().update(Qt.InputMethodQuery.ImCursorRectangle)
            self.semantic_binding.notify("geometry")
        except Exception as error:
            self.fail(error)

    def set_scroll(self, x=None, y=None):
        if self.pages:
            self.apply_view(action="scroll", x=self.state["scroll"]["x"] if x is None else x,
                            y=self.state["scroll"]["y"] if y is None else y)

    def scroll(self, x, y):
        self.set_scroll(self.state["scroll"]["x"] + x, self.state["scroll"]["y"] + y)

    def zoom_at(self, factor, point=None):
        if not self.pages:
            return
        point = point or QPointF(self.canvas.width() / 2, self.canvas.height() / 2)
        zoom = max(0.25, min(4.0, self.state["config"]["zoom"] * factor))
        self.apply_view(action="zoomAt", zoom=zoom, x=point.x(), y=point.y())

    def mapped(self, page, x, y):
        result = self.worker.host.host_request(op="view", state=self.state,
            command={"action": "pointInViewport", "page": page, "x": x, "y": y})["query"]
        return QPointF(result["x"], result["y"])

    def page_rect(self, page, bounds):
        point = self.mapped(page, bounds["x"], bounds["y"])
        zoom = self.state["config"]["zoom"]
        return QRectF(point.x(), point.y(), bounds.get("width", 1) * zoom, bounds["height"] * zoom)

    def caret_rect(self):
        ready = self.caret and self.pages and self.view and not self.loading_pages
        return self.page_rect(self.caret["page"], self.caret) if ready else QRectF(0, 0, 1, 20)

    def current_tile_identity(self):
        return (self.document_generation, *self.geometry_key())

    def accessibility_key(self):
        if not self.info:
            return None
        return (self.document_generation, *self.geometry_key(),
                selection_identity(self.info.get("selection")))

    def request_semantics(self):
        if (self.closing or self.intent is not None or not self.pages or self.loading_pages
                or self.page_geometry_key != self.geometry_key()):
            return
        self.semantic.invalidate(self.accessibility_key())
        request = self.semantic.take_request()
        if request is None:
            return
        ticket, command = request
        def received(result):
            if result is None:
                return
            try:
                self.semantic.accept(ticket, result)
            except (ValueError, KeyError, TypeError) as error:
                self.semantic.failed(ticket)
                self.fail(error)
        accepted = self.enqueue([lambda _: command if self.semantic.current(ticket) else None], received,
                                failed=lambda _: self.semantic.failed(ticket))
        if not accepted:
            self.semantic.failed(ticket)

    def map_accessible_rectangle(self, rectangle):
        # Consume the completed NativeView placement. Accessibility reads must
        # never call mapped()/host_request or derive new document layout.
        if (self.loading_pages or self.page_geometry_key != self.geometry_key()
                or self.semantic.key != self.accessibility_key()):
            return None
        page = next((item for item in self.view.get("visible", []) if item["page"] == rectangle.get("page")), None)
        bounds = rectangle.get("bounds", {})
        values = [bounds.get(key) for key in ("x", "y", "width", "height")]
        if page is None or any(not isinstance(value, (int, float)) or not math.isfinite(value) for value in values):
            return None
        if values[2] <= 0 or values[3] <= 0:
            return None
        zoom, placed = self.state["config"]["zoom"], page["pageRect"]
        local = QRectF(placed["x"] + values[0] * zoom, placed["y"] + values[1] * zoom,
                       values[2] * zoom, values[3] * zoom).toAlignedRect()
        local = local.intersected(self.canvas.rect())
        if local.isEmpty():
            return None
        return local.translated(self.canvas.mapToGlobal(QPoint(0, 0)))

    def reset_tiles(self):
        self.cancel_render()
        if self.gpu is not None:
            self.gpu.sync_source()
        self.tile_identity = self.current_tile_identity()
        self.tile_images.clear()
        self.visible_tiles = ()
        self.frames.clear()
        self.tile_failures.clear()
        self.raster_cache_stats = {}
        self.cache_stats_epoch = -1
        self.update_raster_status()

    def sync_visible_tiles(self):
        if self.tile_identity != self.current_tile_identity():
            self.reset_tiles()
        if self.gpu is not None:
            self.visible_tiles = ()
            self.frames.clear()
            self.gpu.sync_source()
            self.update_raster_status()
            return
        if not self.pages or self.loading_pages:
            self.visible_tiles = ()
            self.frames.clear()
            return
        try:
            self.visible_tiles = plan_visible_tiles(self.pages, self.view.get("visible", []),
                                                    self.state["config"]["deviceScale"])
        except (ValueError, KeyError, TypeError) as error:
            self.visible_tiles = ()
            self.frames.clear()
            self.fail(error)
            return
        # This map is visibility metadata only. QImages are owned solely by the
        # byte/entry-bounded LRU, so eviction cannot leave hidden image owners.
        self.frames = {tile.key(self.tile_identity): True for tile in self.visible_tiles
                       if tile.key(self.tile_identity) in self.tile_images}
        self.update_raster_status()

    def update_raster_status(self):
        if hasattr(self, "raster_status"):
            if self.gpu is not None:
                self.raster_status.setText(f"GL · snapshot {len(self.gpu.snapshots)}개")
                return
            native = self.raster_cache_stats.get("retainedBytes", 0)
            self.raster_status.setText(f"타일 {len(self.tile_images)}개 · 화면 {self.tile_images.retained_bytes // 1024} KiB"
                                      f" · Native {native // 1024} KiB")

    def cancel_render(self):
        if self.pending_render is None or self.render_cancel_requested:
            return
        # The pending completion retains the FIFO slot until it actually ends.
        # P77 can stop only at tile boundaries; an active Skia call may finish.
        self.render_cancel_requested = True
        try:
            self.worker.cancel(self.pending_render)
            self.render_cancellations += 1
        except Exception as error:
            self.fail(error)

    def release_frame(self, result):
        if "buffer" in result:
            self.worker.host.call("buffer_release", result["buffer"])

    def query_raster_cache_stats(self):
        def received(result):
            self.raster_cache_stats = dict(result["stats"])
            self.update_raster_status()
        self.enqueue([lambda _: {"op": "rasterCacheStats"}], received)

    def render_next(self):
        if self.closing or not self.info or not self.view or not self.pages or self.loading_pages:
            return
        if self.gpu is not None:
            self.gpu.render_next()
            return
        tile = next((item for item in self.visible_tiles if item.key(self.tile_identity) not in self.tile_images
                     and item.key(self.tile_identity) not in self.tile_failures), None)
        if tile is None:
            if self.visible_tiles and self.cache_stats_epoch != self.epoch:
                self.cache_stats_epoch = self.epoch
                self.query_raster_cache_stats()
            return
        epoch, target, identity = self.epoch, self.target(), self.tile_identity
        key = tile.key(identity)
        def rendered(result):
            self.pending_render = None
            self.render_cancel_requested = False
            current = (epoch == self.epoch and target == self.target() and identity == self.tile_identity
                       and identity == self.current_tile_identity())
            if not current or result.get("kind") in ("cancelled", "skipped"):
                self.release_frame(result)
                return
            if not result.get("ok") or "buffer" not in result:
                self.release_frame(result)
                self.fail(result.get("error", "tiled render failed"))
                self.tile_failures.add(key)
                return
            consumed = False
            try:
                if result.get("revision") != target["revision"] or result.get("fontGeneration") != identity[-1]:
                    # A stale native result is never copied, even if its buffer
                    # was already published before a cancellation took effect.
                    self.tile_failures.add(key)
                    return
                frame_byte_cost(tile, result)
                consumed = True  # copy_buffer releases in its own finally.
                pixels = self.worker.copy_buffer(result)
                image = QImage(pixels, result["width"], result["height"], result["stride"],
                               QImage.Format.Format_RGBA8888_Premultiplied).copy()
                if image.isNull():
                    raise ValueError("tile QImage allocation failed")
                image.setDevicePixelRatio(tile.device_scale)
                if not self.tile_images.put(key, CachedTile(image, result["originX"], result["originY"]), image.sizeInBytes()):
                    raise ValueError("tile image exceeds the client cache budget")
                self.frames[key] = True
                self.frame_count += 1
                self.update_raster_status()
                self.canvas.update()
            except Exception as error:
                self.tile_failures.add(key)
                self.fail(error)
            finally:
                if not consumed:
                    self.release_frame(result)
        self.send({"op": "renderTiledRegion", "fontGeneration": identity[-1], "page": tile.page,
                   "region": tile.region, "scale": tile.scale}, rendered)
        self.pending_render = self.pending[0]

    def click(self, point, extend, *, drag_token=None, done=None):
        if (not self.info or self.is_composing() or self.loading_pages or not self.pages or not self.view
                or self.page_geometry_key != self.geometry_key()):
            return False
        geometry, epoch = self.page_geometry_key, self.epoch
        hit = self.worker.host.host_request(op="view", state=self.state,
            command={"action": "pageAt", "x": point.x(), "y": point.y()})["query"]
        if hit is None:
            return False
        def current_geometry():
            return (geometry == self.page_geometry_key == self.geometry_key()
                    and epoch == self.epoch and not self.loading_pages and not self.is_composing()
                    and (drag_token is None or drag_token is self.drag_token))
        def hit_test(_):
            if current_geometry():
                return {"op": "hitTest", "page": hit["page"], **hit["point"]}
        def select(reply):
            if reply is None or not current_geometry():
                return
            position = reply["hit"]["position"]
            if not extend or self.drag_anchor is None:
                self.drag_anchor = position
            return {"op": "setSelection" if self.info.get("selection") else "beginEdit",
                    "selection": {"anchor": self.drag_anchor, "focus": position}}
        return self.enqueue([hit_test, select], done=done, refresh=True, failed=done)

    def stop_drag(self):
        self.drag_timer.stop()
        self.drag_token = self.drag_point = self.drag_document = None
        self.drag_anchor = None
        self.drag_hit_pending = False

    def start_drag(self, point):
        self.stop_drag()
        self.drag_token = object()
        self.drag_document = (self.document_generation, self.geometry_key())
        self.drag_point = QPointF(point)
        if not self.queue_drag_hit(point, False):
            self.stop_drag()

    def outside_viewport(self, point):
        return not QRectF(self.canvas.rect()).contains(point)

    def drag_to(self, point):
        if self.drag_token is None or self.drag_point is None:
            return
        self.drag_point = QPointF(point)
        if self.outside_viewport(point):
            self.drag_timer.start()
        else:
            if self.queue_drag_hit(point, True):
                self.drag_timer.stop()
            else:
                self.drag_timer.start()

    def queue_drag_hit(self, point, extend):
        if self.drag_hit_pending or len(self.actions) + (self.job is not None) >= self.MAX_DRAG_ACTIONS:
            return False
        token = self.drag_token
        requested = QPointF(point)
        def received(reply):
            if token is self.drag_token:
                self.drag_hit_pending = False
                if self.drag_anchor is None or (reply is not None and not reply.get("ok", True)):
                    self.stop_drag()
                    return
                if (self.drag_point is not None and self.drag_point != requested
                        and not self.outside_viewport(self.drag_point)):
                    self.drag_to(self.drag_point)
                elif self.drag_point is not None and not self.outside_viewport(self.drag_point):
                    self.drag_timer.stop()
        self.drag_hit_pending = True
        accepted = self.click(point, extend, drag_token=token, done=received)
        if not accepted:
            self.drag_hit_pending = False
        return accepted

    def drag_hit_point(self, point):
        # Viewport edges may lie in page margins or inter-page gaps. Reuse the
        # native view's visible page rectangles to query the nearest visible page.
        viewport = QRectF(self.canvas.rect())
        candidates = []
        for item in self.view.get("visible", []):
            bounds = item["pageRect"]
            area = QRectF(bounds["x"], bounds["y"], bounds["width"], bounds["height"]).intersected(viewport)
            if area.isEmpty():
                continue
            x = max(area.left(), min(math.nextafter(area.right(), area.left()), point.x()))
            y = max(area.top(), min(math.nextafter(area.bottom(), area.top()), point.y()))
            nearest = QPointF(x, y)
            distance = (nearest.x() - point.x()) ** 2 + (nearest.y() - point.y()) ** 2
            candidates.append((distance, nearest))
        return min(candidates, key=lambda item: item[0])[1] if candidates else None

    def autoscroll_drag(self):
        if (self.drag_point is None or self.drag_document != (self.document_generation, self.geometry_key())
                or self.closing or self.intent is not None or self.is_composing()):
            self.stop_drag()
            return
        if (self.drag_hit_pending or self.drag_anchor is None or self.loading_pages
                or len(self.actions) + (self.job is not None) >= self.MAX_DRAG_ACTIONS):
            return
        point = self.drag_point
        def step(value, extent):
            distance = value if value < 0 else max(0, value - extent)
            return max(-self.MAX_DRAG_SCROLL, min(self.MAX_DRAG_SCROLL, distance))
        dx, dy = step(point.x(), self.canvas.width()), step(point.y(), self.canvas.height())
        if dx or dy:
            self.scroll(dx, dy)
        hit = self.drag_hit_point(point)
        if hit is not None:
            if self.queue_drag_hit(hit, True) and not self.outside_viewport(point):
                self.drag_timer.stop()

    def end_drag(self, point):
        token = self.drag_token
        self.drag_timer.stop()
        self.drag_point = None
        def finished(_=None):
            if token is self.drag_token:
                self.stop_drag()
        hit = self.drag_hit_point(point)
        if token is None or hit is None or not self.click(hit, True, drag_token=token, done=finished):
            finished()

    def command(self, op, **fields):
        def request(_):
            values = dict(fields)
            if op == "executeCommand" and values["command"]["command"] in ("moveUp", "moveDown"):
                values["command"] = dict(values["command"], preferredX=self.preferred_x)
            return {"op": op, **values}
        self.enqueue([request], refresh=True)

    def is_composing(self):
        return bool(self.preedit or self.ime_pending or self.info.get("compositionId") is not None)

    def invoke_action(self, name):
        if self.closing or self.intent is not None:
            return False
        if self.is_composing():
            if name == "undo":
                self.cancel_composition(reset=True)
                return True
            if name not in ("zoomIn", "zoomOut"):
                self.statusBar().showMessage("한글 조합을 확정하거나 취소한 뒤 다시 시도해 주세요.")
                return False
        handlers = {
            "new": self.new_document, "open": self.open_dialog,
            "recovery": self.open_recovery_dialog,
            "save": lambda: getattr(self, "save_document", self.save_dialog)(),
            "saveAs": self.save_dialog,
            "undo": lambda: self.command("undo"), "redo": lambda: self.command("redo"),
            "copy": lambda: self.enqueue([lambda _: {"op": "copySelection"}], self.copied),
            "cut": lambda: self.enqueue([lambda _: {"op": "cutSelection"}], self.copied, refresh=True),
            "paste": self.paste_clipboard,
            "selectAll": lambda: self.command("executeCommand", command={"command": "selectAll"}),
            "zoomIn": lambda: self.zoom_at(1.1), "zoomOut": lambda: self.zoom_at(1 / 1.1),
        }
        handlers[name]()
        return True

    def paste_clipboard(self):
        mime = QApplication.clipboard().mimeData()
        if mime is not None:
            if mime.hasHtml():
                self.command("pasteHtml", html=mime.html())
            elif mime.hasText():
                self.command("pasteText", text=mime.text())

    def key(self, event):
        modifiers = event.modifiers()
        altgr = altgr_modifiers(modifiers)
        shift = bool(modifiers & Qt.KeyboardModifier.ShiftModifier)
        key = event.key()
        if not altgr:
            for name, _label, standard, _menu in self.STANDARD_ACTIONS:
                if event.matches(standard):
                    self.update_document_ui()
                    self.document_actions[name].trigger()
                    event.accept()
                    return
            if key == Qt.Key.Key_Escape and self.is_composing():
                self.cancel_composition(reset=True)
                event.accept()
                return
            if modifiers & (Qt.KeyboardModifier.ControlModifier | Qt.KeyboardModifier.MetaModifier | Qt.KeyboardModifier.AltModifier):
                event.ignore()
                return
        names = {Qt.Key.Key_Left: "moveLeft", Qt.Key.Key_Right: "moveRight", Qt.Key.Key_Up: "moveUp",
                 Qt.Key.Key_Down: "moveDown", Qt.Key.Key_Home: "home", Qt.Key.Key_End: "end",
                 Qt.Key.Key_Backspace: "deleteBackward", Qt.Key.Key_Delete: "deleteForward",
                 Qt.Key.Key_Return: "splitParagraph", Qt.Key.Key_Enter: "splitParagraph"}
        if key in names and not self.is_composing():
            command = {"command": names[key]}
            if key in (Qt.Key.Key_Left, Qt.Key.Key_Right, Qt.Key.Key_Up, Qt.Key.Key_Down, Qt.Key.Key_Home, Qt.Key.Key_End):
                command["extendSelection"] = shift
            if key in (Qt.Key.Key_Up, Qt.Key.Key_Down):
                command["preferredX"] = self.preferred_x
            self.command("executeCommand", command=command)
        elif event.text() and not self.is_composing():
            self.command("replace", text=event.text())
        else:
            event.ignore()
            return
        event.accept()

    def copied(self, result):
        payload = result["payload"]
        mime = QMimeData()
        mime.setText(payload["plainText"])
        if payload.get("html") is not None:
            mime.setHtml(payload["html"])
        QApplication.clipboard().setMimeData(mime)

    def ime(self, committed, preedit, cursor, cursor_visible=True):
        generation = self.document_generation
        def begin(_):
            if self.info.get("compositionId") is None:
                selection = self.info["selection"]
                self.preedit_start = min((selection["anchor"], selection["focus"]),
                                        key=lambda p: (p["section"], p["paragraph"], p["scalarOffset"]))
                return {"op": "beginComposition"}
        def update(text):
            return lambda _: {"op": "updateComposition", "compositionId": self.info["compositionId"], "text": text}
        def finish(_):
            return {"op": "commitComposition", "compositionId": self.info["compositionId"]}
        steps = []
        if committed:
            steps += [begin, update(committed), finish]
        if preedit:
            steps += [begin, update(preedit)]
        elif not committed:
            steps += [lambda _: {"op": "cancelComposition", "compositionId": self.info["compositionId"]}
                      if self.info.get("compositionId") is not None else None]
        def done(_):
            if generation != self.document_generation:
                return
            self.ime_pending = max(0, self.ime_pending - 1)
            self.preedit, self.preedit_cursor = preedit, cursor
            self.preedit_cursor_visible = cursor_visible
            self.preedit_rects = []
            if preedit and self.preedit_start:
                anchor = dict(self.preedit_start)
                focus = dict(anchor)
                focus["scalarOffset"] += len(preedit)
                target, composition_id = self.target(), self.info.get("compositionId")
                def current_preedit(_):
                    if target == self.target() and composition_id == self.info.get("compositionId"):
                        return {"op": "selectionRects", "selection": {"anchor": anchor, "focus": focus}}
                self.enqueue([current_preedit],
                             lambda reply: self.set_preedit_rects(reply["rects"]) if reply else None)
            self.refresh()
            self.update_document_ui()
        def failed(_):
            if generation == self.document_generation:
                self.ime_pending = max(0, self.ime_pending - 1)
                self.update_document_ui()
        accepted = self.enqueue(steps, done, failed=failed)
        if accepted:
            self.ime_pending += 1
            self.update_document_ui()
        return accepted

    def set_preedit_rects(self, rects):
        self.preedit_rects = rects
        self.canvas.update()

    def cancel_composition(self, *, reset=False):
        if reset:
            QApplication.inputMethod().reset()
        if self.info and not self.closing:
            self.enqueue([lambda _: {"op": "cancelComposition", "compositionId": self.info["compositionId"]}
                          if self.info.get("compositionId") is not None else None],
                         lambda _: self.clear_preedit(), refresh=True)

    def clear_preedit(self):
        self.preedit = ""
        self.preedit_start = None
        self.preedit_cursor = 0
        self.preedit_rects = []
        self.caret = None
        self.context.clear()
        self.canvas.update()
        QApplication.inputMethod().update(Qt.InputMethodQuery.ImQueryAll)

    def confirm_unsaved_changes(self):
        answer = QMessageBox.question(self, "변경 내용 저장", "현재 문서의 변경 내용을 저장할까요?",
            QMessageBox.StandardButton.Save | QMessageBox.StandardButton.Discard | QMessageBox.StandardButton.Cancel,
            QMessageBox.StandardButton.Cancel)
        return {QMessageBox.StandardButton.Save: "save", QMessageBox.StandardButton.Discard: "discard"}.get(answer, "cancel")

    def abort_transition(self, intent, message=None):
        if self.intent is not intent:
            return
        self.intent = None
        self.transition_phase = None
        if message:
            self.fail(message)
        self.update_document_ui()
        self.refresh()

    def request_transition(self, operation, path=None):
        self.stop_drag()
        if self.closing or self.saving:
            self.statusBar().showMessage("진행 중인 저장 작업을 마친 뒤 다시 시도해 주세요.")
            return False
        if self.intent is not None and self.transition_phase != "reading":
            return False
        self.intent_serial += 1
        intent = {"serial": self.intent_serial, "operation": operation, "path": path}
        self.intent = intent
        self.transition_phase = "settling"
        self.update_document_ui()
        # Earlier commands are already accepted input. Inspect their completed
        # state before deciding whether this transition would discard changes.
        if self.info or self.pending is not None or self.actions:
            accepted = self.enqueue([lambda _: {"op": "info"}], lambda _: self.guard_transition(intent),
                                    failed=lambda _: self.abort_transition(intent))
            if not accepted:
                self.abort_transition(intent)
            return accepted
        self.guard_transition(intent)
        return True

    def guard_transition(self, intent):
        if self.intent is not intent:
            return
        if self.info.get("compositionId") is not None:
            self.abort_transition(intent, "한글 조합을 확정하거나 취소한 뒤 다시 시도해 주세요.")
            return
        if self.info.get("isDirty"):
            self.transition_phase = "prompting"
            choice = self.confirm_unsaved_changes()
            if self.intent is not intent:
                return
            if choice == "cancel":
                self.abort_transition(intent)
                return
            if choice == "save":
                self.transition_phase = "saving"
                self.save_document(on_saved=lambda _: self.saved_before_transition(intent),
                                   on_failed=lambda _: self.abort_transition(intent))
                return
        self.perform_transition(intent)

    def saved_before_transition(self, intent):
        if self.intent is not intent:
            return
        if self.info.get("isDirty"):
            self.abort_transition(intent, "저장 중 추가 변경이 있어 문서 교체 또는 종료를 보류했습니다.")
            return
        self.perform_transition(intent)

    def perform_transition(self, intent):
        if self.intent is not intent:
            return
        intent["target"] = self.target() if self.info else None
        if intent["operation"] == "close":
            if self.io_jobs:
                self.abort_transition(intent, "파일 작업 완료 후 창을 닫아 주세요.")
                return
            self.intent = None
            self.transition_phase = None
            self.shutdown_worker()
        elif intent["operation"] == "new":
            self.replace_document(intent)
        else:
            self.transition_phase = "reading"
            path = Path(intent["path"])
            def loaded(future):
                if self.intent is not intent:
                    return
                try:
                    first, second = future.result()
                    if intent["operation"] == "recovery":
                        intent["recovery_header"], data = first, second
                        if not self.confirm_recovery(intent["recovery_header"]):
                            self.abort_transition(intent)
                            return
                    else:
                        data, intent["source_stamp"] = first, second
                except Exception as error:
                    self.abort_transition(intent, str(error))
                    return
                self.replace_document(intent, data)
            reader = read_recovery if intent["operation"] == "recovery" else read_document
            self.io_jobs.append((self.io.submit(reader, path), loaded))

    def replace_document(self, intent, data=None):
        def request(_):
            if self.intent is not intent:
                return None
            current = self.target() if self.info else None
            if intent["target"] != current or self.info.get("compositionId") is not None:
                self.abort_transition(intent, "대기 중 문서가 변경되어 문서 교체를 보류했습니다.")
                return None
            self.transition_phase = "replacing"
            if data is None:
                return {"op": "new", "replace": current}
            return {"_openRecovery" if intent["operation"] == "recovery" else "_open": data}
        def replaced(result):
            if self.intent is intent:
                self.intent = None
                self.transition_phase = None
                recovery = intent["operation"] == "recovery"
                self.opened(result, None if recovery else intent["path"], intent.get("source_stamp"))
                if recovery:
                    self.statusBar().showMessage("복구 내용을 저장되지 않은 새 문서로 열었습니다.")
        # The read may have overlapped accepted programmatic edits. A fresh info
        # barrier makes that race visible instead of retargeting the replacement.
        steps = [lambda _: {"op": "info"}] if self.info else []
        if not self.enqueue(steps + [request], replaced,
                            failed=lambda _: self.abort_transition(intent), replacement=True):
            self.abort_transition(intent)

    def new_document(self):
        return self.request_transition("new")

    def open_dialog(self):
        path, _ = QFileDialog.getOpenFileName(self, "HWP/HWPX 열기", "", "문서 (*.hwp *.hwpx)")
        if path:
            self.open_path(path)

    def open_path(self, path):
        return self.request_transition("open", Path(path))

    def open_recovery_dialog(self):
        path, _ = QFileDialog.getOpenFileName(self, "복구 파일 열기", str(self.recovery_dir or ""),
                                             "rhwp 복구 파일 (*.rhwp-recovery)")
        if path:
            return self.open_recovery_path(path)

    def open_recovery_path(self, path):
        return self.request_transition("recovery", Path(path))

    def confirm_recovery(self, header):
        report = header["report"]
        dialog = QMessageBox(QMessageBox.Icon.Question, "복구 내용 확인",
            f"복구한 내용을 새 문서로 열까요? 보존하지 못한 항목: {report['count']}개.\n"
            "원본 파일은 복구 메타데이터만으로 덮어쓰지 않습니다.",
            QMessageBox.StandardButton.Open | QMessageBox.StandardButton.Cancel, self)
        dialog.setDefaultButton(QMessageBox.StandardButton.Cancel)
        dialog.setDetailedText(json.dumps(header, ensure_ascii=False, indent=2)[:65536])
        return dialog.exec() == QMessageBox.StandardButton.Open

    def maybe_recover(self, now=None):
        now = time.monotonic() if now is None else now
        if (self.recovery_dir is None or now < self.next_recovery_attempt or not self.info
                or self.closing or self.saving or self.intent is not None or self.io_jobs
                or self.pending is not None or self.job is not None or self.actions
                or self.info.get("compositionId") is not None or not self.info.get("isDirty")):
            return False
        # Back off failures as well as successes; never retry filesystem work at
        # the 8 ms GUI poll frequency. Native store also deduplicates revisions.
        self.next_recovery_attempt = now + self.recovery_interval
        return self._save_destination(self.recovery_dir, recovery=True, recovery_now=now)

    def confirm_content_loss(self, report):
        dialog = QMessageBox(QMessageBox.Icon.Warning, "저장 내용 확인",
            f"선택한 형식에서 보존할 수 없는 항목이 {report['count']}개 있습니다. 계속 저장하시겠습니까?",
            QMessageBox.StandardButton.Save | QMessageBox.StandardButton.Cancel, self)
        dialog.setDefaultButton(QMessageBox.StandardButton.Cancel)
        dialog.setDetailedText(json.dumps(report, ensure_ascii=False, indent=2)[:65536])
        return dialog.exec() == QMessageBox.StandardButton.Save

    def confirm_overwrite(self, path):
        return QMessageBox.question(self, "기존 파일 교체", f"{path.name} 파일을 교체하시겠습니까?",
            QMessageBox.StandardButton.Save | QMessageBox.StandardButton.Cancel,
            QMessageBox.StandardButton.Cancel) == QMessageBox.StandardButton.Save

    def save_document(self, checked=False, *, on_saved=None, on_failed=None):
        if self.file_store is not None and self.file_store.source_path is not None:
            return self._save_destination(None, on_saved=on_saved, on_failed=on_failed)
        return self.save_dialog(on_saved=on_saved, on_failed=on_failed)

    def save_dialog(self, checked=False, *, on_saved=None, on_failed=None):
        if self.saving or self.closing:
            return False
        default = self.output_path or self.source_path
        path, _ = QFileDialog.getSaveFileName(self, "다른 이름으로 저장", str(default) if default else "",
            "한글 문서 (*.hwp *.hwpx)", options=QFileDialog.Option.DontConfirmOverwrite)
        if not path:
            if on_failed:
                on_failed(None)
            return False
        destination = Path(path).absolute()
        if (self.file_store is not None and destination == self.file_store.source_path) or not destination.exists():
            return self._save_destination(destination, on_saved=on_saved, on_failed=on_failed)
        # Observe the destination on the I/O executor before asking approval.
        # Publication will check this exact stamp again, preserving later edits.
        self.saving = True
        generation, document_id = self.document_generation, self.info.get("documentId")
        self.update_document_ui()
        def rejected(error=None):
            self.saving = False
            self.update_document_ui()
            if error:
                self.fail(error)
            if on_failed:
                on_failed(error)
        def observed(future):
            try:
                _, stamp = future.result()
                if generation != self.document_generation or document_id != self.info.get("documentId"):
                    return rejected("파일 확인 중 문서가 교체되어 저장을 보류했습니다.")
                if not self.confirm_overwrite(destination):
                    return rejected()
                self._save_destination(destination, destination_stamp=stamp, already_busy=True,
                                       on_saved=on_saved, on_failed=on_failed)
            except Exception as error:
                rejected(error)
        try:
            self.io_jobs.append((self.io.submit(read_document, destination), observed))
        except Exception as error:
            rejected(error)
            return False
        return True

    def save_new(self, path, *, on_saved=None, on_failed=None):
        """Exclusive new-file API used by CLI/reference smoke callers."""
        if Path(path).exists() or Path(path).is_symlink():
            error = FileExistsError(path)
            self.fail(error)
            if on_failed:
                on_failed(error)
            return False
        return self._save_destination(path, on_saved=on_saved, on_failed=on_failed)

    def _save_destination(self, path, *, destination_stamp=None, already_busy=False, on_saved=None, on_failed=None,
                          recovery=False, recovery_now=None):
        if self.closing or (self.saving and not already_busy) or not self.info:
            return False
        self.saving = True
        generation = self.document_generation
        self.update_document_ui()
        operation = None

        def finish(error=None, result=None):
            self.saving = False
            self.save_operation = None
            self.update_document_ui()
            callback = on_failed if error is not None else on_saved
            if callback:
                callback(error if error is not None else result)

        def failed(error, cleanup_received=False):
            if isinstance(error, dict):
                error = RuntimeError(error.get("error", "native save request failed"))
            if operation is not None:
                error = operation.failure(error)
            if error is not None and (not self.errors or self.errors[-1] != str(error)):
                self.fail(error)
            cleanup = operation.cancel_command if operation is not None and not cleanup_received else None
            def cleaned(_reply=None):
                if operation is not None:
                    operation.cancelled()
                finish(error or RuntimeError("저장을 취소했습니다."))
            if cleanup is not None:
                def cleanup_failed(reply):
                    if reply.get("_completionReceived"):
                        cleaned(reply)
                    else:
                        self.fail("저장 토큰 해제를 전송하지 못했습니다. 문서를 유지합니다.")
                if not self.enqueue([lambda _: cleanup], cleaned, failed=cleanup_failed, save_control=True):
                    self.fail("저장 토큰 정리를 큐에 넣지 못했습니다. 문서를 유지합니다.")
            else:
                cleaned()

        def request(_):
            nonlocal operation
            if self.file_store is None or self.file_store.document_id != self.info["documentId"]:
                self.file_store = NativeWorkerFileStore(self.info["documentId"])
            # This runs after previously accepted edits, so bind the completed
            # revision rather than the state observed when Save was clicked.
            if recovery:
                operation = self.file_store.begin_recovery(self.target(), path,
                    is_dirty=bool(self.info.get("isDirty")), now=recovery_now, interval=self.recovery_interval)
            else:
                operation = self.file_store.begin_save(self.target(), path, destination_stamp=destination_stamp)
            self.save_operation = operation
            return operation.prepare_command if operation is not None else None

        def prepare_failed(reply):
            if operation is not None:
                try:
                    operation.receive_prepared(reply, self.worker.copy_buffer)
                except Exception as error:
                    failed(error)
                    return
            failed(reply)

        def prepared(reply):
            if operation is None:
                finish()
                return
            try:
                operation.receive_prepared(reply, self.worker.copy_buffer)
                if operation.requires_loss_acceptance:
                    if not self.confirm_content_loss(operation.report):
                        failed(RuntimeError("내용 손실 보고를 승인하지 않아 저장을 취소했습니다."))
                        return
                    operation.accept_loss()
                future = self.io.submit(operation.publish)
            except Exception as error:
                failed(error)
                return

            def persisted(future):
                try:
                    future.result()
                    command = operation.cancel_command if recovery else operation.acknowledge_command
                except Exception as error:
                    failed(error)
                    return

                def acknowledged(reply):
                    try:
                        result = operation.recovery_completed(reply) if recovery else operation.acknowledged(reply)
                    except Exception as error:
                        failed(error, cleanup_received=recovery)
                        return
                    if generation != self.document_generation or operation.document_id != self.info.get("documentId"):
                        finish(RuntimeError("이전 문서 파일은 저장했으며 현재 문서는 유지합니다."))
                        return
                    if recovery:
                        self.recovery_path = operation.path
                        self.statusBar().showMessage(f"자동 복구 파일: {operation.path}")
                        finish(result=result)
                        return
                    self.saved_path = self.source_path = operation.path
                    self.info["isDirty"] = result["isDirty"]
                    self.statusBar().showMessage(f"저장됨: {operation.path}")
                    # Remain behind accepted edits when inspecting final dirty
                    # state; this barrier must not use the priority save slot.
                    def inspected(reply):
                        finish(result=reply)
                    def inspect_failed(error):
                        if isinstance(error, dict):
                            error = error.get("error", "저장 후 문서 상태를 확인하지 못했습니다.")
                        self.fail(error)
                        finish(error)
                    if not self.enqueue([lambda _: {"op": "info"}], inspected, failed=inspect_failed):
                        inspect_failed("파일은 저장했으나 문서 상태 확인을 큐에 넣지 못했습니다.")

                def control_failed(reply):
                    failed(reply, cleanup_received=recovery and reply.get("_completionReceived", False))
                if not self.enqueue([lambda _: command], acknowledged, failed=control_failed, save_control=True):
                    failed(RuntimeError("저장 완료 확인을 큐에 넣지 못했습니다."))
            self.io_jobs.append((future, persisted))

        accepted = self.enqueue([request], prepared, failed=prepare_failed)
        if not accepted:
            failed(RuntimeError("저장을 큐에 넣지 못했습니다."))
        return accepted

    def shutdown_worker(self):
        self.stop_drag()
        self.semantic.close()
        self.closing = True
        self.gpu_failure_timer.stop()
        if self.gpu is not None:
            self.canvas.dispose_gpu()
        self.actions.clear()
        self.worker.close()
        self.statusBar().showMessage("진행 중인 native 작업의 종료를 기다립니다.")
        self.update_document_ui()

    def closeEvent(self, event):
        self.stop_drag()
        if self.finished:
            event.accept()
            return
        event.ignore()
        if not self.closing:
            if not self.info and self.pending is None and not self.io_jobs:
                self.shutdown_worker()
            else:
                self.request_transition("close")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--renderer", choices=("cpu", "auto", "gl"), default="cpu")
    parser.add_argument("--recovery-dir", type=Path, help="명시적으로 지정한 폴더에 자동 복구 파일 보관")
    parser.add_argument("--recovery-interval", type=float, default=30.0)
    args = parser.parse_args()
    app = QApplication(sys.argv[:1])
    window = NativeWindow(args.library, args.input, args.output, renderer=args.renderer,
        recovery_dir=args.recovery_dir, recovery_interval=args.recovery_interval)
    window.show()
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
