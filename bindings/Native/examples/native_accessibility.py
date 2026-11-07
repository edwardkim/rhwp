"""Revision-bound semantic snapshots for a read-only Qt text interface.

Qt queries read completed host data only. Missing character geometry is queued
for the existing native selection-rect API and is never inferred from carets.
"""

from bisect import bisect_left, bisect_right
from collections import OrderedDict
import copy
from dataclasses import dataclass
import weakref


MAX_NODES = 256
MAX_TEXT_BYTES = 64 * 1024
MAX_CHARACTER_RECTS = 64
MAX_GEOMETRY_PENDING = 8
SNAPSHOT_REQUEST = {"geometry": True, "maxNodes": MAX_NODES, "maxTextBytes": MAX_TEXT_BYTES,
                    "maxDepth": 8, "maxGeometryQueries": 64, "maxSelectionRects": 64}


def utf16_boundaries(text):
    offsets = [0]
    for character in text:
        offsets.append(offsets[-1] + (2 if ord(character) > 0xffff else 1))
    return tuple(offsets)


def position_address(position):
    return (position["section"], position["paragraph"],
            tuple((entry["control"], entry["cell"], entry["paragraph"]) for entry in position["cellPath"]))


def selection_identity(selection):
    if selection is None:
        return None
    return tuple((position_address(selection[key]), selection[key]["scalarOffset"]) for key in ("anchor", "focus"))


@dataclass(frozen=True)
class SemanticNode:
    text: str
    start: dict
    end: dict
    role: str
    complete: bool
    offset: int
    boundaries: tuple
    geometry: object


class SemanticSnapshot:
    def __init__(self, snapshot):
        raw_nodes = snapshot["nodes"]
        if len(raw_nodes) > MAX_NODES:
            raise ValueError("accessibility node budget exceeded")
        self.nodes, offset, total_bytes = [], 0, 0
        self.addresses = {}
        for raw in raw_nodes:
            text, start, end = raw["text"], raw["start"], raw["end"]
            total_bytes += len(text.encode("utf-8"))
            if total_bytes > MAX_TEXT_BYTES:
                raise ValueError("accessibility text budget exceeded")
            if (position_address(start) != position_address(end)
                    or end["scalarOffset"] - start["scalarOffset"] != len(text)):
                raise ValueError("accessibility node text/address mismatch")
            boundaries = utf16_boundaries(text)
            node = SemanticNode(text, start, end, raw["role"], raw["textComplete"], offset, boundaries, raw.get("geometry"))
            address = position_address(start)
            if address in self.addresses:
                raise ValueError("duplicate accessibility paragraph address")
            self.nodes.append(node)
            self.addresses[address] = node
            offset += boundaries[-1] + 1  # Explicit structural paragraph separator.
        self.text = "\n".join(node.text for node in self.nodes)
        self.boundaries = utf16_boundaries(self.text)
        self.starts = tuple(node.offset for node in self.nodes)
        self.selection = snapshot["selection"]
        self.selection_geometry = snapshot.get("selectionGeometry")
        if self.selection_geometry and len(self.selection_geometry["rectangles"]) > MAX_CHARACTER_RECTS:
            raise ValueError("accessibility selection geometry budget exceeded")
        self.limitations = tuple(snapshot["limitations"])
        self.complete = bool(snapshot["complete"])
        self.truncated = bool(snapshot["truncated"])
        self.scope = snapshot["scope"]

    @property
    def character_count(self):
        return self.boundaries[-1]

    def scalar_at(self, offset):
        if isinstance(offset, bool) or not isinstance(offset, int):
            raise ValueError("text offset must be an integer UTF-16 boundary")
        index = bisect_left(self.boundaries, offset)
        if index >= len(self.boundaries) or self.boundaries[index] != offset:
            raise ValueError("text offset is outside a Unicode scalar boundary")
        return index

    def text_range(self, start, end):
        if end == -1:
            end = self.character_count
        left, right = self.scalar_at(start), self.scalar_at(end)
        if right < left:
            raise ValueError("text range is reversed")
        return self.text[left:right]

    def offset_for_position(self, position):
        if position is None:
            return None
        node = self.addresses.get(position_address(position))
        if node is None:
            return None
        relative = position["scalarOffset"] - node.start["scalarOffset"]
        return node.offset + node.boundaries[relative] if 0 <= relative < len(node.boundaries) else None

    def position_for_offset(self, offset):
        self.scalar_at(offset)
        index = bisect_right(self.starts, offset) - 1
        if index < 0:
            raise ValueError("text position has no native paragraph")
        node = self.nodes[index]
        relative = bisect_left(node.boundaries, offset - node.offset)
        if relative >= len(node.boundaries) or node.boundaries[relative] != offset - node.offset:
            raise ValueError("paragraph separator has no native scalar address")
        return {**node.start, "scalarOffset": node.start["scalarOffset"] + relative}

    def selection_offsets(self):
        if self.selection is None:
            return None
        anchor, focus = (self.offset_for_position(self.selection[key]) for key in ("anchor", "focus"))
        return None if anchor is None or focus is None else (anchor, focus)


class AccessibilityState:
    def __init__(self):
        self.key = None
        self.snapshot = None
        self.closed = False
        self.pending = None
        self.failed_snapshot = False
        self.snapshot_needed = False
        self.serial = 0
        self.character_rects = OrderedDict()
        self.geometry_wanted = OrderedDict()
        self.changed = lambda _kind, _previous: None

    def invalidate(self, key):
        if self.closed or key == self.key:
            return
        previous = self.snapshot
        same_document = previous is not None and key is not None and self.key[:4] == key[:4]
        self.key, self.pending = key, None
        self.snapshot = copy.copy(previous) if same_document else None
        if same_document:
            # Text/geometry remain valid at the same revision. Hide the old
            # selection until its asynchronous replacement is available, without
            # announcing that the document text was deleted on every caret move.
            self.snapshot.selection = None
        self.failed_snapshot = False
        self.snapshot_needed = key is not None
        if not same_document:
            self.character_rects.clear()
        self.geometry_wanted.clear()
        self.changed("selection" if same_document else "snapshot", previous)

    def close(self):
        previous = self.snapshot
        self.closed = True
        self.key = self.snapshot = self.pending = None
        self.snapshot_needed = False
        self.character_rects.clear()
        self.geometry_wanted.clear()
        self.changed("closed", previous)

    def request_character(self, offset):
        if self.snapshot is None or offset in self.character_rects or offset in self.geometry_wanted:
            return
        if self.pending is not None and self.pending[2:] == ("geometry", offset):
            return
        try:
            index = self.snapshot.scalar_at(offset)
            if index == len(self.snapshot.text):
                return
            end = self.snapshot.boundaries[index + 1]
            selection = {"anchor": self.snapshot.position_for_offset(offset),
                         "focus": self.snapshot.position_for_offset(end)}
        except (ValueError, KeyError):
            return
        if len(self.geometry_wanted) < MAX_GEOMETRY_PENDING:
            self.geometry_wanted[offset] = selection

    def take_request(self):
        if self.closed or self.key is None or self.pending is not None:
            return None
        kind, offset = "snapshot", None
        if self.snapshot_needed:
            if self.failed_snapshot:
                return None
            command = {"op": "accessibility", "request": dict(SNAPSHOT_REQUEST)}
        elif self.geometry_wanted:
            offset, selection = self.geometry_wanted.popitem(last=False)
            kind = "geometry"
            command = {"op": "selectionRects", "selection": selection}
        else:
            return None
        self.serial += 1
        ticket = (self.serial, self.key, kind, offset)
        self.pending = ticket
        command["target"] = {"documentId": self.key[1], "revision": self.key[2]}
        return ticket, command

    def current(self, ticket):
        return not self.closed and self.pending == ticket and self.key == ticket[1]

    def accept(self, ticket, result):
        if not self.current(ticket):
            return False
        if ticket[2] == "snapshot":
            snapshot = result["snapshot"]
            if (result["target"] != {"documentId": self.key[1], "revision": self.key[2]}
                    or snapshot["revision"] != self.key[2] or snapshot["fontGeneration"] != self.key[3]
                    or selection_identity(snapshot["selection"]) != self.key[4]):
                self.failed_snapshot = True
                self.pending = None
                return False
            previous = self.snapshot
            self.snapshot = SemanticSnapshot(snapshot)
            self.snapshot_needed = False
            selection, geometry = self.snapshot.selection_offsets(), self.snapshot.selection_geometry
            if selection is not None and geometry and geometry.get("status") == "complete":
                start, end = sorted(selection)
                index = self.snapshot.scalar_at(start)
                if index < len(self.snapshot.text) and self.snapshot.boundaries[index + 1] == end:
                    rectangles = geometry["rectangles"]
                    self.character_rects[start] = tuple(rectangles)
                    while len(self.character_rects) > MAX_CHARACTER_RECTS:
                        self.character_rects.popitem(last=False)
            self.pending = None
            self.changed("snapshot", previous)
        else:
            rectangles = result["rects"]
            if len(rectangles) > MAX_CHARACTER_RECTS:
                raise ValueError("accessibility character geometry budget exceeded")
            self.character_rects[ticket[3]] = tuple(rectangles)
            while len(self.character_rects) > MAX_CHARACTER_RECTS:
                self.character_rects.popitem(last=False)
            self.pending = None
            self.changed("geometry", None)
        return True

    def failed(self, ticket):
        if not self.current(ticket):
            return
        self.pending = None
        if ticket[2] == "snapshot":
            self.failed_snapshot = True
        else:
            self.character_rects[ticket[3]] = ()
            while len(self.character_rects) > MAX_CHARACTER_RECTS:
                self.character_rects.popitem(last=False)
        self.changed("geometry", None)


_qt_document_type = None
_qt_factory = None
_qt_interface_for = None
_qt_bindings = weakref.WeakKeyDictionary()


def ensure_qt_accessibility_factory():
    global _qt_document_type, _qt_factory, _qt_interface_for
    if _qt_factory is not None:
        return
    from PySide6.QtCore import QRect, QTextBoundaryFinder
    from PySide6.QtGui import QAccessible, QAccessibleTextInterface
    from PySide6.QtWidgets import QAccessibleWidget
    from shiboken6 import getCppPointer

    class AccessibleDocument(QAccessibleWidget, QAccessibleTextInterface):
        def __init__(self, widget, binding):
            QAccessibleWidget.__init__(self, widget, QAccessible.Role.Document)
            QAccessibleTextInterface.__init__(self)
            self.binding = weakref.ref(binding) if binding is not None else lambda: None

        def interface_cast(self, kind):
            return self if kind == QAccessible.InterfaceType.TextInterface else super().interface_cast(kind)

        def model(self):
            binding = self.binding()
            return binding.state.snapshot if binding is not None else None

        def state(self):
            state = super().state()
            binding = self.binding()
            state.readOnly = True
            state.selectableText = True
            state.busy = bool(binding and binding.state.pending and binding.state.snapshot is None)
            state.invalid = binding is None or binding.state.closed
            return state

        def text(self, *arguments):
            model = self.model()
            if len(arguments) == 2:
                try:
                    return model.text_range(*arguments) if model is not None else ""
                except ValueError:
                    return ""
            kind = arguments[0]
            if kind == QAccessible.Text.Name:
                return "rhwp 문서 내용"
            if kind == QAccessible.Text.Value:
                return model.text if model is not None else ""
            if kind == QAccessible.Text.Description:
                if model is None:
                    return "문서 내용 snapshot 준비 중 또는 사용할 수 없음"
                limits = ", ".join(model.limitations) or "없음"
                binding = self.binding()
                return (f"범위 {model.scope}; complete={model.complete}; truncated={model.truncated}; 제한 {limits}; "
                        f"selectionRefreshPending={bool(binding and binding.state.snapshot_needed)}; "
                        "접근성 편집·선택 변경·스크롤과 조판 line/word/sentence 경계 미지원; "
                        "문자 geometry는 요청 후 갱신하며 캐시된 위치만 hit-test 지원")
            return super().text(kind)

        def characterCount(self):
            model = self.model()
            return model.character_count if model is not None else 0

        def cursorPosition(self):
            model = self.model()
            selection = model.selection_offsets() if model is not None else None
            return selection[1] if selection is not None else -1

        def selectionCount(self):
            model = self.model()
            selection = model.selection_offsets() if model is not None else None
            return int(selection is not None and selection[0] != selection[1])

        def selection(self, index):
            model = self.model()
            selection = model.selection_offsets() if model is not None else None
            return tuple(sorted(selection)) if index == 0 and selection is not None else (-1, -1)

        def characterRect(self, offset):
            binding = self.binding()
            if binding is None:
                return QRect()
            binding.state.request_character(offset)
            return binding.character_rect(offset)

        def offsetAtPoint(self, point):
            binding = self.binding()
            if binding is not None:
                for offset in binding.state.character_rects:
                    if binding.character_rect(offset).contains(point):
                        return offset
            return -1

        def attributes(self, offset):
            model = self.model()
            if model is not None:
                try:
                    model.scalar_at(offset)
                    index = bisect_right(model.starts, offset) - 1
                    node = model.nodes[index]
                    status = node.geometry.get("status", "unavailable") if node.geometry else "notRequested"
                    return (f"rhwp-role:{node.role};rhwp-text-complete:{str(node.complete).lower()};rhwp-geometry-status:{status};",
                            node.offset, node.offset + node.boundaries[-1])
                except (ValueError, IndexError):
                    pass
            return ("", -1, -1)

        def boundary_offset(self, offset):
            if offset == -2:
                return self.cursorPosition()
            return self.characterCount() if offset == -1 else offset

        def boundary_text(self, mode, offset, boundary):
            offset = self.boundary_offset(offset)
            model = self.model()
            unavailable = ("", -1, -1)
            if model is None or not 0 <= offset <= model.character_count:
                return unavailable
            if boundary == QAccessible.TextBoundaryType.NoBoundary:
                return (model.text, 0, model.character_count) if mode == "at" else unavailable
            if boundary == QAccessible.TextBoundaryType.ParagraphBoundary:
                boundaries = (*model.starts, model.character_count)
            elif boundary == QAccessible.TextBoundaryType.CharBoundary:
                # PySide 6.10.2 cannot base-call QAccessibleTextInterface's
                # default out-parameter methods. Use Qt's public grapheme
                # finder, which reports UTF-16 positions, without native I/O.
                boundaries = getattr(model, "qt_graphemes", None)
                if boundaries is None:
                    finder = QTextBoundaryFinder(QTextBoundaryFinder.BoundaryType.Grapheme, model.text)
                    positions = [0]
                    for _ in model.boundaries:
                        position = finder.toNextBoundary()
                        if position < 0:
                            break
                        if position <= positions[-1]:
                            return unavailable
                        positions.append(position)
                    boundaries = model.qt_graphemes = tuple(positions)
            else:
                return unavailable
            index = bisect_right(boundaries, offset) - 1
            if mode == "before":
                index -= 1
            elif mode == "after":
                index += 1
            if not 0 <= index < len(boundaries) - 1:
                return unavailable
            start, end = boundaries[index:index + 2]
            return (model.text_range(start, end), start, end)

        def textAtOffset(self, offset, boundary):
            return self.boundary_text("at", offset, boundary)

        def textBeforeOffset(self, offset, boundary):
            return self.boundary_text("before", offset, boundary)

        def textAfterOffset(self, offset, boundary):
            return self.boundary_text("after", offset, boundary)

        # This adapter deliberately exposes snapshot reads. Native keyboard/IME
        # editing remains separate; no inaccessible offset is silently retargeted.
        def addSelection(self, _start, _end): pass
        def setSelection(self, _index, _start, _end): pass
        def removeSelection(self, _index): pass
        def setCursorPosition(self, _position): pass
        def scrollToSubstring(self, _start, _end): pass

    _qt_document_type = AccessibleDocument
    def interface_for(widget, binding=None):
        # Qt may query one QObject during construction under several distinct
        # QMetaObjects. Its cache owns/deletes each corresponding interface;
        # returning the same pointer for two metaobjects registers it twice.
        # Keep one wrapper per actual metaobject, not per factory class-name
        # argument (which can also be a superclass fallback).
        interfaces = getattr(widget, "_rhwp_accessibility_interfaces", None)
        if interfaces is None:
            interfaces = widget._rhwp_accessibility_interfaces = {}
        key = getCppPointer(widget.metaObject())[0]
        interface = interfaces.get(key)
        if interface is None:
            interface = interfaces[key] = _qt_document_type(widget, binding)
        elif binding is not None:
            interface.binding = weakref.ref(binding)
        widget._rhwp_accessibility_interface = interface
        return interface

    _qt_interface_for = interface_for
    def factory(_name, widget):
        reference = _qt_bindings.get(widget)
        binding = reference() if reference is not None else None
        # A platform bridge may ask during construction before binding. Attach
        # the provider later, preserving each metaobject's cached interface.
        if binding is not None or getattr(widget, "_rhwp_accessibility_document", False):
            return interface_for(widget, binding)
        return None
    _qt_factory = factory
    QAccessible.installFactory(_qt_factory)


class QtAccessibilityBinding:
    def __init__(self, widget, state, map_rectangle):
        ensure_qt_accessibility_factory()
        self.widget = weakref.ref(widget)
        self.state = state
        self.map_rectangle = (weakref.WeakMethod(map_rectangle) if getattr(map_rectangle, "__self__", None) is not None
                              else lambda: map_rectangle)
        self.interface = _qt_interface_for(widget, self)
        for interface in widget._rhwp_accessibility_interfaces.values():
            interface.binding = weakref.ref(self)
        _qt_bindings[widget] = weakref.ref(self)
        reference = weakref.ref(self)
        state.changed = lambda kind, previous: reference().notify(kind, previous) if reference() is not None else None

    def character_rect(self, offset):
        from PySide6.QtCore import QRect
        output = QRect()
        mapper = self.map_rectangle()
        if mapper is None:
            return output
        for rectangle in self.state.character_rects.get(offset, ()):
            mapped = mapper(rectangle)
            if mapped is not None:
                output = output.united(mapped)
        return output

    def notify(self, kind, previous=None):
        from PySide6.QtGui import QAccessible, QAccessibleEvent, QAccessibleTextSelectionEvent, QAccessibleTextUpdateEvent
        from shiboken6 import isValid
        widget = self.widget()
        if widget is None or not isValid(widget):
            return
        if kind == "closed":
            QAccessible.updateAccessibility(QAccessibleEvent(widget, QAccessible.Event.ObjectHide))
        elif kind == "snapshot":
            text = self.state.snapshot.text if self.state.snapshot is not None else ""
            previous_text = previous.text if previous is not None else ""
            if previous_text != text:
                QAccessible.updateAccessibility(QAccessibleTextUpdateEvent(widget, 0, previous_text, text))
            selection = self.state.snapshot.selection_offsets() if self.state.snapshot is not None else None
            if selection is not None:
                QAccessible.updateAccessibility(QAccessibleTextSelectionEvent(widget, *sorted(selection)))
        else:
            QAccessible.updateAccessibility(QAccessibleEvent(widget, QAccessible.Event.LocationChanged))
