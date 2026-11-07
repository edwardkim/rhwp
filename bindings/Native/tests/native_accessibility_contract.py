"""Relational semantic transport/UTF-16 contracts; no visual layout baseline.

2026-11-07 KST: geometry is supplied by native queries, never reconstructed
from paragraph carets or text metrics. Qt API probes live in native_window_smoke.
"""

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_accessibility import AccessibilityState, MAX_GEOMETRY_PENDING, SemanticSnapshot, selection_identity


def position(paragraph=0, offset=0, cell_path=None):
    return {"section": 0, "paragraph": paragraph, "scalarOffset": offset, "cellPath": cell_path or []}


def response(text="a😀b", revision=3, generation=0, nodes=None, selection=None):
    if nodes is None:
        nodes = [{"key": "3/body/0", "role": "bodyParagraph", "start": position(),
                  "end": position(offset=len(text)), "text": text, "textComplete": True, "geometry": None}]
    return {"ok": True, "target": {"documentId": 7, "revision": revision}, "snapshot": {
        "revision": revision, "fontGeneration": generation, "scope": "bodyAndTableCellText",
        "range": None, "nodes": nodes,
        "selection": selection or {"anchor": position(offset=1), "focus": position(offset=min(2, len(text)))},
        "selectionGeometry": None, "complete": True, "truncated": False, "limitations": []}}


class AccessibilityContracts(unittest.TestCase):
    def ready(self, packet=None):
        packet = packet or response()
        state = AccessibilityState()
        state.invalidate((1, 7, 3, 0, selection_identity(packet["snapshot"]["selection"])))
        ticket, command = state.take_request()
        self.assertEqual(command["op"], "accessibility")
        self.assertEqual(command["target"], {"documentId": 7, "revision": 3})
        self.assertTrue(state.accept(ticket, packet))
        return state

    def test_qt_offsets_are_utf16_while_native_addresses_are_scalars(self):
        model = self.ready().snapshot
        self.assertEqual(model.character_count, 4)
        self.assertEqual(model.text_range(1, 3), "😀")
        self.assertEqual(model.text_range(0, -1), "a😀b")
        self.assertEqual(model.position_for_offset(3), position(offset=2))
        self.assertEqual(model.offset_for_position(position(offset=2)), 3)
        self.assertEqual(model.selection_offsets(), (1, 3))
        with self.assertRaises(ValueError):
            model.text_range(1, 2)
        with self.assertRaises(ValueError):
            model.position_for_offset(2)

    def test_structural_separators_preserve_body_and_table_addresses(self):
        cell = [{"control": 0, "cell": 2, "paragraph": 0}]
        nodes = [response(text="body")["snapshot"]["nodes"][0],
                 {"key": "cell", "role": "tableCellParagraph", "text": "표😀", "textComplete": True,
                  "start": position(paragraph=1, cell_path=cell),
                  "end": position(paragraph=1, offset=2, cell_path=cell), "geometry": None}]
        model = SemanticSnapshot(response(nodes=nodes)["snapshot"])
        self.assertEqual(model.text, "body\n표😀")
        self.assertEqual(model.offset_for_position(position(paragraph=1, offset=2, cell_path=cell)), 8)
        self.assertEqual(model.position_for_offset(5), position(paragraph=1, cell_path=cell))
        self.assertEqual(model.position_for_offset(4), position(offset=4))

    def test_cursor_outside_bounded_snapshot_stays_unknown(self):
        packet = response()
        packet["snapshot"]["selection"]["focus"] = position(paragraph=100, offset=0)
        model = SemanticSnapshot(packet["snapshot"])
        self.assertIsNone(model.selection_offsets())
        self.assertIsNone(model.offset_for_position(position(offset=100)))

    def test_node_and_text_budgets_and_address_mismatch_are_rejected(self):
        packet = response()
        for nodes in ([packet["snapshot"]["nodes"][0]] * 257,
                      response(text="x" * (64 * 1024 + 1))["snapshot"]["nodes"]):
            with self.assertRaises(ValueError):
                SemanticSnapshot({**packet["snapshot"], "nodes": nodes})
        packet["snapshot"]["nodes"][0]["end"] = position(offset=4)
        with self.assertRaises(ValueError):
            SemanticSnapshot(packet["snapshot"])

    def test_stale_document_revision_font_and_selection_tickets_are_rejected(self):
        for key in ((2, 7, 3, 0, "selection"), (1, 8, 3, 0, "selection"),
                    (1, 7, 4, 0, "selection"), (1, 7, 3, 1, "selection"), (1, 7, 3, 0, "new selection")):
            state = AccessibilityState()
            state.invalidate((1, 7, 3, 0, "selection"))
            ticket, _ = state.take_request()
            state.invalidate(key)
            self.assertFalse(state.accept(ticket, response()))
            self.assertIsNone(state.snapshot)

    def test_wrong_native_response_metadata_does_not_publish_or_spin(self):
        for packet in (response(revision=4), response(generation=1)):
            state = AccessibilityState()
            state.invalidate((1, 7, 3, 0, selection_identity(response()["snapshot"]["selection"])))
            ticket, _ = state.take_request()
            self.assertFalse(state.accept(ticket, packet))
            self.assertIsNone(state.snapshot)
            self.assertIsNone(state.take_request())

    def test_same_revision_snapshot_with_wrong_selection_is_rejected(self):
        state = AccessibilityState()
        packet = response()
        state.invalidate((1, 7, 3, 0, selection_identity(packet["snapshot"]["selection"])))
        ticket, _ = state.take_request()
        packet["snapshot"]["selection"]["focus"] = position(offset=3)
        self.assertFalse(state.accept(ticket, packet))
        self.assertIsNone(state.snapshot)

    def test_caret_move_keeps_revision_text_and_refreshes_selection_asynchronously(self):
        state = self.ready()
        changed = response(selection={"anchor": position(offset=3), "focus": position(offset=3)})
        key = (*state.key[:4], selection_identity(changed["snapshot"]["selection"]))
        state.invalidate(key)
        self.assertEqual(state.snapshot.text, "a😀b")
        self.assertIsNone(state.snapshot.selection_offsets())
        ticket, command = state.take_request()
        self.assertEqual(command["op"], "accessibility")
        self.assertTrue(state.accept(ticket, changed))
        self.assertEqual(state.snapshot.selection_offsets(), (4, 4))

    def test_character_queries_queue_bounded_coalesced_native_geometry(self):
        state = self.ready(response(text="x" * 100))
        for offset in range(100):
            state.request_character(offset)
        self.assertEqual(len(state.geometry_wanted), MAX_GEOMETRY_PENDING)
        ticket, command = state.take_request()
        self.assertEqual(command["op"], "selectionRects")
        self.assertEqual(command["selection"], {"anchor": position(), "focus": position(offset=1)})
        state.request_character(0)
        self.assertNotIn(0, state.geometry_wanted)
        rects = [{"page": 0, "bounds": {"x": 10, "y": 20, "width": 8, "height": 12}}]
        self.assertTrue(state.accept(ticket, {"rects": rects}))
        self.assertEqual(state.character_rects[0], tuple(rects))

    def test_single_character_selection_uses_snapshot_geometry_without_an_extra_query(self):
        packet = response()
        rect = {"page": 0, "bounds": {"x": 10, "y": 20, "width": 8, "height": 12}}
        packet["snapshot"]["selectionGeometry"] = {"status": "complete", "focusCaret": None, "rectangles": [rect]}
        state = self.ready(packet)
        state.request_character(1)
        self.assertEqual(state.character_rects[1], (rect,))
        self.assertIsNone(state.take_request())

    def test_surrogate_midpoint_never_queues_character_geometry(self):
        state = self.ready()
        state.request_character(2)
        self.assertFalse(state.geometry_wanted)
        state.request_character(1)
        _, command = state.take_request()
        self.assertEqual(command["selection"]["focus"]["scalarOffset"], 2)

    def test_invalidation_and_close_drop_geometry_and_reject_late_results(self):
        state = self.ready()
        state.request_character(0)
        ticket, _ = state.take_request()
        state.close()
        self.assertFalse(state.accept(ticket, {"rects": []}))
        self.assertIsNone(state.snapshot)
        self.assertIsNone(state.take_request())
        self.assertFalse(state.character_rects)

    def test_failed_geometry_requests_are_negatively_cached_and_bounded(self):
        state = self.ready(response(text="x" * 100))
        for offset in range(100):
            state.request_character(offset)
            ticket, _ = state.take_request()
            state.failed(ticket)
        self.assertEqual(len(state.character_rects), 64)
        state.request_character(99)
        self.assertIsNone(state.take_request())

    def test_limitations_and_partial_scope_are_preserved(self):
        packet = response()
        packet["snapshot"].update(complete=False, truncated=True, limitations=["nodeLimit", "headersFooters"])
        model = self.ready(packet).snapshot
        self.assertFalse(model.complete)
        self.assertTrue(model.truncated)
        self.assertEqual(model.limitations, ("nodeLimit", "headersFooters"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
