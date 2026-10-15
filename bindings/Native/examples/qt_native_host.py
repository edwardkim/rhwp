#!/usr/bin/env python3
"""Small Qt/native worker reference window. No WASM, DOM or GUI-thread layout."""
import argparse
from collections import deque
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import sys

from PySide6.QtCore import QEvent, QMimeData, QPointF, QRectF, Qt, QTimer
from PySide6.QtGui import QColor, QImage, QPainter, QPen
from PySide6.QtWidgets import QApplication, QFileDialog, QGridLayout, QMainWindow, QScrollBar, QWidget

sys.path.insert(0, str(Path(__file__).resolve().parent))
from native_host import write_new_file
from native_worker import NativeWorker


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
        for page in self.window.view.get("visible", []):
            bounds = page["pageRect"]
            painter.fillRect(QRectF(bounds["x"], bounds["y"], bounds["width"], bounds["height"]), Qt.GlobalColor.white)
            frame = self.window.frames.get(page["page"])
            if frame:
                painter.drawImage(frame[0], frame[1])
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
        self.loading_pages = False
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
        for label, handler in [("열기", self.open_dialog), ("새 파일로 저장", self.save_dialog),
                               ("실행 취소", lambda: self.command("undo")),
                               ("다시 실행", lambda: self.command("redo")),
                               ("확대", lambda: self.zoom_at(1.1)), ("축소", lambda: self.zoom_at(1 / 1.1))]:
            toolbar.addAction(label).triggered.connect(handler)
        self.resize(900, 700)
        self.setWindowTitle("rhwp Native · Qt reference")
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.tick)
        self.timer.start(8)
        if input_path:
            self.open_path(input_path)
        else:
            self.enqueue([lambda _: {"op": "new", "replace": None}], self.opened)

    def target(self):
        return {"documentId": self.info["documentId"], "revision": self.info["revision"]}

    def fail(self, message):
        self.errors.append(str(message))
        self.statusBar().showMessage(str(message))

    def enqueue(self, steps, done=None, refresh=False):
        if self.closing:
            return False
        if len(self.actions) + (self.job is not None) >= self.MAX_ACTIONS:
            self.fail("입력 큐가 가득 찼습니다. 진행 중인 작업은 취소되지 않았습니다.")
            return False
        self.actions.append([deque(steps), done, refresh, None])
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
                        self.worker.copy_buffer(result)
                    self.fail("예상하지 못한 worker completion")
                    continue
                callback = self.pending[1]
                self.pending = None
                callback(result)
            self.pump()
        except Exception as error:
            self.pending = None
            self.job = None
            self.fail(error)

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
        if self.pending is not None:
            return
        if self.job is None and self.actions:
            self.job = self.actions.popleft()
        if self.job is not None:
            steps, done, refresh, last = self.job
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

    def step_done(self, result):
        if not result.get("ok") or result.get("kind") == "skipped":
            self.fail(result.get("error", result.get("reason", "native request failed")))
            self.job = None
            return
        self.absorb(result)
        self.job[3] = result

    def opened(self, result):
        self.frames.clear()
        self.pages.clear()
        self.loading_pages = False
        self.context.clear()
        self.preedit = ""
        self.enqueue([lambda _: {"op": "beginEdit", "selection": {"anchor": body(), "focus": body()}}],
                     refresh=True)
        self.refresh()

    def refresh(self):
        if not self.info or self.closing:
            return
        def got_info(result):
            count = self.info["pageCount"]
            if count > self.MAX_PAGES:
                self.fail(f"이 예제는 최대 {self.MAX_PAGES}쪽까지 표시합니다.")
                return
            if len(self.pages) != count and not self.loading_pages:
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

    def load_page_size(self, page, count):
        if page >= count:
            self.loading_pages = False
            self.rebuild_view()
            return
        def loaded(result):
            self.pages.append(result["size"])
            self.load_page_size(page + 1, count)
        self.enqueue([lambda _: {"op": "pageSize", "page": page}], loaded)

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
        self.canvas.update()
        QApplication.inputMethod().update(Qt.InputMethodQuery.ImCursorRectangle)

    def set_rects(self, rects):
        self.rects = rects
        self.canvas.update()

    def rebuild_view(self):
        if not hasattr(self, "canvas") or not self.pages:
            return
        self.state["config"].update(pages=self.pages, viewport={"width": max(1, self.canvas.width()),
            "height": max(1, self.canvas.height())}, deviceScale=self.canvas.devicePixelRatioF())
        self.apply_view(action="state")

    def apply_view(self, **command):
        try:
            self.view = self.worker.host.host_request(op="view", state=self.state, command=command)
            self.state = self.view["state"]
            self.epoch += 1
            self.frames.clear()
            for bar, axis in ((self.horizontal, "x"), (self.vertical, "y")):
                bar.blockSignals(True)
                bar.setRange(0, int(self.view["scrollLimit"][axis] + 0.5))
                bar.setValue(int(self.state["scroll"][axis]))
                bar.blockSignals(False)
            self.canvas.update()
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
        return self.page_rect(self.caret["page"], self.caret) if self.caret and self.pages else QRectF(0, 0, 1, 20)

    def render_next(self):
        if not self.info or not self.view:
            return
        visible = next((item for item in self.view["visible"] if item["page"] not in self.frames), None)
        if visible is None:
            return
        epoch, target = self.epoch, self.target()
        generation = self.info["fontGeneration"]
        dpr, scale = self.state["config"]["deviceScale"], visible["rasterScale"]
        def rendered(result):
            if "buffer" in result:
                pixels = self.worker.copy_buffer(result)  # copy + release, on GUI thread
                if result.get("ok") and epoch == self.epoch and target == self.target() and generation == self.info["fontGeneration"]:
                    image = QImage(pixels, result["width"], result["height"], result["stride"],
                                   QImage.Format.Format_RGBA8888_Premultiplied).copy()
                    image.setDevicePixelRatio(dpr)
                    origin = self.mapped(visible["page"], result["originX"] / scale, result["originY"] / scale)
                    self.frames[visible["page"]] = (origin, image)
                    self.frame_count += 1
                    self.canvas.update()
            elif not result.get("ok"):
                self.fail(result.get("error", "render failed"))
                self.frames[visible["page"]] = None  # do not spin on the same error
        self.send({"op": "renderRegion", "fontGeneration": generation, "page": visible["page"],
                   "region": visible["pageRegion"], "scale": scale}, rendered)

    def click(self, point, extend):
        if not self.info or self.preedit:
            return
        hit = self.worker.host.host_request(op="view", state=self.state,
            command={"action": "pageAt", "x": point.x(), "y": point.y()})["query"]
        if hit is None:
            return
        def select(reply):
            position = reply["hit"]["position"]
            if not extend or self.drag_anchor is None:
                self.drag_anchor = position
            return {"op": "setSelection" if self.info.get("selection") else "beginEdit",
                    "selection": {"anchor": self.drag_anchor, "focus": position}}
        self.enqueue([lambda _: {"op": "hitTest", "page": hit["page"], **hit["point"]}, select], refresh=True)

    def command(self, op, **fields):
        def request(_):
            values = dict(fields)
            if op == "executeCommand" and values["command"]["command"] in ("moveUp", "moveDown"):
                values["command"] = dict(values["command"], preferredX=self.preferred_x)
            return {"op": op, **values}
        self.enqueue([request], refresh=True)

    def key(self, event):
        control = bool(event.modifiers() & Qt.KeyboardModifier.ControlModifier)
        shift = bool(event.modifiers() & Qt.KeyboardModifier.ShiftModifier)
        key = event.key()
        if control:
            if key == Qt.Key.Key_C:
                self.enqueue([lambda _: {"op": "copySelection"}], self.copied)
            elif key == Qt.Key.Key_X:
                self.enqueue([lambda _: {"op": "cutSelection"}], self.copied, refresh=True)
            elif key == Qt.Key.Key_V:
                mime = QApplication.clipboard().mimeData()
                self.command("pasteHtml" if mime.hasHtml() else "pasteText", **({"html": mime.html()} if mime.hasHtml() else {"text": mime.text()}))
            elif key == Qt.Key.Key_Z:
                self.command("redo" if shift else "undo")
            elif key == Qt.Key.Key_Y:
                self.command("redo")
            elif key == Qt.Key.Key_A:
                self.command("executeCommand", command={"command": "selectAll"})
            elif key == Qt.Key.Key_S:
                self.save_dialog()
            elif key == Qt.Key.Key_O:
                self.open_dialog()
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

    def open_dialog(self):
        path, _ = QFileDialog.getOpenFileName(self, "HWP/HWPX 열기", "", "문서 (*.hwp *.hwpx)")
        if path:
            self.open_path(path)

    def open_path(self, path):
        def read():
            with Path(path).open("rb") as file:
                data = file.read(128 * 1024 * 1024 + 1)
            if len(data) > 128 * 1024 * 1024:
                raise ValueError("문서 크기가 128 MiB를 넘습니다.")
            return data
        self.io_jobs.append((self.io.submit(read), lambda future: self.enqueue([lambda _: {"_open": future.result()}], self.opened)))

    def save_dialog(self):
        path = str(self.output_path) if self.output_path else ""
        path, _ = QFileDialog.getSaveFileName(self, "새 HWPX 파일로 저장", path, "HWPX (*.hwpx)")
        if path:
            self.save_new(path)

    def save_new(self, path):
        def prepared(result):
            token, document_id = result["token"], self.info["documentId"]
            try:
                data = self.worker.copy_buffer(result)
            except Exception:
                self.enqueue([lambda _: {"op": "cancelSave", "documentId": document_id, "token": token}])
                raise
            def persisted(future):
                try:
                    future.result()
                except Exception as error:
                    self.fail(error)
                    self.enqueue([lambda _: {"op": "cancelSave", "documentId": document_id, "token": token}])
                    return
                def acknowledged(reply):
                    self.saved_path = Path(path)
                    self.info["isDirty"] = reply["isDirty"]
                    self.statusBar().showMessage(f"저장됨: {path}")
                self.enqueue([lambda _: {"op": "acknowledgeSave", "documentId": document_id, "token": token}], acknowledged)
            self.io_jobs.append((self.io.submit(write_new_file, path, data), persisted))
        self.enqueue([lambda _: {"op": "prepareSave", "format": "hwpx"}], prepared)

    def closeEvent(self, event):
        if self.finished:
            event.accept()
            return
        event.ignore()
        if self.io_jobs:
            self.statusBar().showMessage("파일 작업 완료 후 창을 닫아 주세요.")
            return
        if not self.closing:
            self.closing = True
            self.actions.clear()
            self.worker.close()
            self.statusBar().showMessage("진행 중인 native 작업의 종료를 기다립니다.")


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
