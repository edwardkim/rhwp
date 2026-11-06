#!/usr/bin/env python3
"""Small Qt/native worker reference window. No WASM, DOM or GUI-thread layout."""
import argparse
import json
from collections import deque
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
import sys

from PySide6.QtCore import QEvent, QMimeData, QPointF, QRectF, Qt, QTimer
from PySide6.QtGui import QColor, QImage, QPainter, QPen
from PySide6.QtWidgets import QApplication, QFileDialog, QGridLayout, QLabel, QMainWindow, QMessageBox, QScrollBar, QWidget

sys.path.insert(0, str(Path(__file__).resolve().parent))
from native_file_store import read_document
from native_worker import NativeWorker
from native_worker_file_store import NativeWorkerFileStore
from native_viewport_tiles import NATIVE_CACHE_BYTES, TileImageCache, frame_byte_cost, plan_visible_tiles


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


class Canvas(QWidget):
    def __init__(self, window):
        super().__init__(window)
        self.window = window
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)
        self.setAttribute(Qt.WidgetAttribute.WA_InputMethodEnabled)
        self.setMinimumSize(200, 150)

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
        painter.end()
        self.window.paint_count += 1

    def resizeEvent(self, event):
        self.window.rebuild_view()
        super().resizeEvent(event)

    def event(self, event):
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
            self.window.click(event.position(), False)

    def mouseMoveEvent(self, event):
        if event.buttons() & Qt.MouseButton.LeftButton and len(self.window.actions) < 4:
            self.window.click(event.position(), True)

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.window.click(event.position(), True)

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
        self.window.ime(event.commitString(), event.preeditString(), cursor, cursor_visible)
        event.accept()

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
        self.window.cancel_composition()
        super().focusOutEvent(event)


class NativeWindow(QMainWindow):
    MAX_ACTIONS = 64
    MAX_PAGES = 512

    def __init__(self, library, input_path=None, output_path=None, worker=None):
        super().__init__()
        self.worker = worker or NativeWorker(library)
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
        self.preedit = ""
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
        self.canvas = Canvas(self)
        self.vertical = QScrollBar(Qt.Orientation.Vertical)
        self.horizontal = QScrollBar(Qt.Orientation.Horizontal)
        central = QWidget()
        layout = QGridLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self.canvas, 0, 0)
        layout.addWidget(self.vertical, 0, 1)
        layout.addWidget(self.horizontal, 1, 0)
        self.setCentralWidget(central)
        self.vertical.valueChanged.connect(lambda value: self.set_scroll(y=value))
        self.horizontal.valueChanged.connect(lambda value: self.set_scroll(x=value))
        toolbar = self.addToolBar("문서")
        self.document_actions = {}
        for name, label, handler in [("new", "새 문서", self.new_document), ("open", "열기", self.open_dialog),
                                     ("save", "저장", self.save_document),
                                     ("saveAs", "다른 이름으로 저장", self.save_dialog),
                                     ("undo", "실행 취소", lambda: self.command("undo")),
                                     ("redo", "다시 실행", lambda: self.command("redo")),
                                     ("zoomIn", "확대", lambda: self.zoom_at(1.1)),
                                     ("zoomOut", "축소", lambda: self.zoom_at(1 / 1.1))]:
            action = toolbar.addAction(label)
            action.triggered.connect(handler)
            self.document_actions[name] = action
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
        for name in ("save", "saveAs"):
            self.document_actions[name].setEnabled(available and bool(self.info) and not self.saving)
        for name, capability in (("undo", "canUndo"), ("redo", "canRedo")):
            self.document_actions[name].setEnabled(available and bool(self.info.get(capability)))
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
                    self.fail("예상하지 못한 worker completion")
                    continue
                callback = self.pending[1]
                self.pending = None
                callback(result)
            self.pump()
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
        geometry_keys = ("documentId", "revision", "fontGeneration", "selection")
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
            self.rects.clear()
            self.preedit_rects.clear()
        if previous[:3] != self.geometry_key():
            self.drag_anchor = None
            self.reset_tiles()
        self.update_document_ui()

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
                failed(result)
            return
        self.absorb(result)
        if self.job.replacement:
            self.job.document_id = self.info.get("documentId")
        self.job.last_result = result

    def opened(self, result, path=None, source_stamp=None):
        self.document_generation += 1
        self.reset_tiles()
        self.source_path = Path(path) if path is not None else None
        self.saved_path = None
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
                self.enqueue([lambda _: {"op": "imeContext"}], self.got_context)
                self.enqueue([lambda _: {"op": "caretRect", "position": self.ime_caret_position()}], self.got_caret)
                self.enqueue([lambda _: {"op": "selectionRects", "selection": self.info["selection"]}],
                             lambda reply: self.set_rects(reply["rects"]))
        self.enqueue([lambda _: {"op": "info"}], got_info)

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
        if self.caret and self.pages and self.view and not self.loading_pages:
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

    def reset_tiles(self):
        self.cancel_render()
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

    def click(self, point, extend):
        if (not self.info or self.preedit or self.loading_pages or not self.pages or not self.view
                or self.page_geometry_key != self.geometry_key()):
            return
        geometry, epoch = self.page_geometry_key, self.epoch
        hit = self.worker.host.host_request(op="view", state=self.state,
            command={"action": "pageAt", "x": point.x(), "y": point.y()})["query"]
        if hit is None:
            return
        def current_geometry():
            return (geometry == self.page_geometry_key == self.geometry_key()
                    and epoch == self.epoch and not self.loading_pages and not self.preedit)
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
        self.enqueue([hit_test, select], refresh=True)

    def command(self, op, **fields):
        def request(_):
            values = dict(fields)
            if op == "executeCommand" and values["command"]["command"] in ("moveUp", "moveDown"):
                values["command"] = dict(values["command"], preferredX=self.preferred_x)
            return {"op": op, **values}
        self.enqueue([request], refresh=True)

    def key(self, event):
        modifiers = event.modifiers()
        control = bool(modifiers & Qt.KeyboardModifier.ControlModifier) and not bool(
            modifiers & (Qt.KeyboardModifier.AltModifier | Qt.KeyboardModifier.GroupSwitchModifier))
        shift = bool(modifiers & Qt.KeyboardModifier.ShiftModifier)
        key = event.key()
        if control:
            if key == Qt.Key.Key_C:
                self.enqueue([lambda _: {"op": "copySelection"}], self.copied)
            elif key == Qt.Key.Key_X:
                self.enqueue([lambda _: {"op": "cutSelection"}], self.copied, refresh=True)
            elif key == Qt.Key.Key_V:
                mime = QApplication.clipboard().mimeData()
                if mime is not None:
                    if mime.hasHtml():
                        self.command("pasteHtml", html=mime.html())
                    elif mime.hasText():
                        self.command("pasteText", text=mime.text())
            elif key == Qt.Key.Key_Z:
                self.command("redo" if shift else "undo")
            elif key == Qt.Key.Key_Y:
                self.command("redo")
            elif key == Qt.Key.Key_A:
                self.command("executeCommand", command={"command": "selectAll"})
            elif key == Qt.Key.Key_S:
                self.save_document()
            elif key == Qt.Key.Key_O:
                self.open_dialog()
            elif key == Qt.Key.Key_N:
                self.new_document()
            else:
                event.ignore()
                return
        else:
            names = {Qt.Key.Key_Left: "moveLeft", Qt.Key.Key_Right: "moveRight", Qt.Key.Key_Up: "moveUp",
                     Qt.Key.Key_Down: "moveDown", Qt.Key.Key_Home: "home", Qt.Key.Key_End: "end",
                     Qt.Key.Key_Backspace: "deleteBackward", Qt.Key.Key_Delete: "deleteForward",
                     Qt.Key.Key_Return: "splitParagraph", Qt.Key.Key_Enter: "splitParagraph"}
            if key in names:
                command = {"command": names[key]}
                if key in (Qt.Key.Key_Left, Qt.Key.Key_Right, Qt.Key.Key_Up, Qt.Key.Key_Down, Qt.Key.Key_Home, Qt.Key.Key_End):
                    command["extendSelection"] = shift
                if key in (Qt.Key.Key_Up, Qt.Key.Key_Down):
                    command["preferredX"] = self.preferred_x
                self.command("executeCommand", command=command)
            elif event.text() and not self.preedit:
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
        self.enqueue(steps, done)

    def set_preedit_rects(self, rects):
        self.preedit_rects = rects
        self.canvas.update()

    def cancel_composition(self):
        if self.info and not self.closing:
            self.enqueue([lambda _: {"op": "cancelComposition", "compositionId": self.info["compositionId"]}
                          if self.info.get("compositionId") is not None else None],
                         lambda _: self.clear_preedit(), refresh=True)

    def clear_preedit(self):
        self.preedit = ""
        self.preedit_rects = []

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
                    data, intent["source_stamp"] = future.result()
                except Exception as error:
                    self.abort_transition(intent, str(error))
                    return
                self.replace_document(intent, data)
            self.io_jobs.append((self.io.submit(read_document, path), loaded))

    def replace_document(self, intent, data=None):
        def request(_):
            if self.intent is not intent:
                return None
            current = self.target() if self.info else None
            if intent["target"] != current or self.info.get("compositionId") is not None:
                self.abort_transition(intent, "대기 중 문서가 변경되어 문서 교체를 보류했습니다.")
                return None
            self.transition_phase = "replacing"
            return {"op": "new", "replace": current} if data is None else {"_open": data}
        def replaced(result):
            if self.intent is intent:
                self.intent = None
                self.transition_phase = None
                self.opened(result, intent["path"], intent.get("source_stamp"))
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

    def _save_destination(self, path, *, destination_stamp=None, already_busy=False, on_saved=None, on_failed=None):
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

        def failed(error):
            if isinstance(error, dict):
                error = RuntimeError(error.get("error", "native save request failed"))
            if operation is not None:
                error = operation.failure(error)
            if error is not None and (not self.errors or self.errors[-1] != str(error)):
                self.fail(error)
            cleanup = operation.cancel_command if operation is not None else None
            def cleaned(_reply=None):
                if operation is not None:
                    operation.cancelled()
                finish(error or RuntimeError("저장을 취소했습니다."))
            if cleanup is not None:
                if not self.enqueue([lambda _: cleanup], cleaned, failed=cleaned, save_control=True):
                    self.fail("저장 토큰 정리를 큐에 넣지 못했습니다. 문서를 유지합니다.")
            else:
                cleaned()

        def request(_):
            nonlocal operation
            if self.file_store is None or self.file_store.document_id != self.info["documentId"]:
                self.file_store = NativeWorkerFileStore(self.info["documentId"])
            # This runs after previously accepted edits, so bind the completed
            # revision rather than the state observed when Save was clicked.
            operation = self.file_store.begin_save(self.target(), path, destination_stamp=destination_stamp)
            self.save_operation = operation
            return operation.prepare_command

        def prepare_failed(reply):
            if operation is not None:
                try:
                    operation.receive_prepared(reply, self.worker.copy_buffer)
                except Exception as error:
                    failed(error)
                    return
            failed(reply)

        def prepared(reply):
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
                    command = operation.acknowledge_command
                except Exception as error:
                    failed(error)
                    return

                def acknowledged(reply):
                    try:
                        result = operation.acknowledged(reply)
                    except Exception as error:
                        failed(error)
                        return
                    if generation != self.document_generation or operation.document_id != self.info.get("documentId"):
                        finish(RuntimeError("이전 문서 파일은 저장했으며 현재 문서는 유지합니다."))
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

                if not self.enqueue([lambda _: command], acknowledged, failed=failed, save_control=True):
                    failed(RuntimeError("저장 완료 확인을 큐에 넣지 못했습니다."))
            self.io_jobs.append((future, persisted))

        accepted = self.enqueue([request], prepared, failed=prepare_failed)
        if not accepted:
            failed(RuntimeError("저장을 큐에 넣지 못했습니다."))
        return accepted

    def shutdown_worker(self):
        self.closing = True
        self.actions.clear()
        self.worker.close()
        self.statusBar().showMessage("진행 중인 native 작업의 종료를 기다립니다.")
        self.update_document_ui()

    def closeEvent(self, event):
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
    args = parser.parse_args()
    app = QApplication(sys.argv[:1])
    window = NativeWindow(args.library, args.input, args.output)
    window.show()
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
