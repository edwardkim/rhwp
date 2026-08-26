"""Alignment, table fields, probe axes."""

from __future__ import annotations

import unittest

from hwp5_inventory.catalog import ctrl_id
from hwp5_inventory.model import (
    build_index_diff,
    build_lcs_diff,
    is_table_candidate,
    make_item,
    pack_table_ctrl_payload,
    pack_table_payload,
    read_u16,
    read_u32,
    table_field_rows,
    table_probe_axes,
)


class PayloadPackTests(unittest.TestCase):
    def test_table_ctrl_fourcc_and_margins(self) -> None:
        payload = pack_table_ctrl_payload(
            common_attr=1,
            x=10,
            y=20,
            width=30,
            height=40,
            out_margin=(80, 81, 82, 83),
        )
        self.assertEqual(read_u32(payload, 0), ctrl_id("tbl "))
        self.assertEqual(read_u16(payload, 0x1C), 80)
        self.assertEqual(read_u16(payload, 0x1E), 81)
        self.assertEqual(read_u16(payload, 0x20), 82)
        self.assertEqual(read_u16(payload, 0x22), 83)

    def test_table_record_attr_and_tail(self) -> None:
        payload = pack_table_payload(table_attr=3, rows=2, cols=4, tail=b"\xab\xcd")
        self.assertEqual(read_u32(payload, 0), 3)
        self.assertEqual(read_u16(payload, 4), 2)
        self.assertEqual(read_u16(payload, 6), 4)
        self.assertEqual(payload[0x16:], b"\xab\xcd")


class AlignmentTests(unittest.TestCase):
    def _pair(self, *, generated_margin=80):
        oracle = make_item(
            sample="s",
            source_path="o.hwp",
            stream_path="/BodyText/Section0",
            record_index=0,
            tag_name="CTRL_HEADER",
            level=0,
            payload=pack_table_ctrl_payload(
                common_attr=1, x=0, y=0, width=1, height=1, out_margin=(80, 80, 80, 80)
            ),
            control_fourcc="tbl ",
        )
        generated = make_item(
            sample="s",
            source_path="g.hwp",
            stream_path="/BodyText/Section0",
            record_index=0,
            tag_name="CTRL_HEADER",
            level=0,
            payload=pack_table_ctrl_payload(
                common_attr=1,
                x=0,
                y=0,
                width=1,
                height=1,
                out_margin=(generated_margin,) * 4,
            ),
            control_fourcc="tbl ",
        )
        return oracle, generated

    def test_index_payload_changed_keeps_uid(self) -> None:
        oracle, generated = self._pair(generated_margin=0)
        items, stats = build_index_diff([oracle], [generated])
        self.assertEqual(stats.changed, 1)
        self.assertEqual(items[0].diff_kind, "payload_changed")
        self.assertEqual(items[0].key, oracle.record_uid)

    def test_index_missing_and_extra(self) -> None:
        oracle, generated = self._pair()
        generated.record_uid = "BodyText.Section0#99"
        generated.record_index = 99
        items, stats = build_index_diff([oracle], [generated])
        kinds = sorted(item.diff_kind for item in items)
        self.assertEqual(kinds, ["extra", "missing"])
        self.assertEqual(stats.missing, 1)
        self.assertEqual(stats.extra, 1)

    def test_lcs_pairs_same_signature(self) -> None:
        oracle, generated = self._pair(generated_margin=0)
        generated.record_uid = "BodyText.Section0#7"
        generated.record_index = 7
        items, stats = build_lcs_diff([oracle], [generated])
        self.assertEqual(stats.changed, 1)
        self.assertEqual(items[0].diff_kind, "changed")
        self.assertIn("payload_hash", items[0].changed_fields)

    def test_lcs_detects_insertion(self) -> None:
        first = make_item(
            sample="s",
            source_path="o.hwp",
            stream_path="/BodyText/Section0",
            record_index=0,
            tag_name="PARA_HEADER",
            level=0,
            payload=b"\x01\x00",
        )
        second = make_item(
            sample="s",
            source_path="o.hwp",
            stream_path="/BodyText/Section0",
            record_index=1,
            tag_name="PARA_TEXT",
            level=1,
            payload=b"ab",
        )
        inserted = make_item(
            sample="s",
            source_path="g.hwp",
            stream_path="/BodyText/Section0",
            record_index=0,
            tag_name="CTRL_HEADER",
            level=0,
            payload=b" osgxxxx",
            control_fourcc="gso ",
        )
        items, stats = build_lcs_diff([first, second], [inserted, first, second])
        self.assertEqual(stats.extra, 1)
        self.assertEqual(stats.matched, 2)
        self.assertEqual(items[0].diff_kind, "extra")


class TableFieldTests(unittest.TestCase):
    def test_outer_margin_rows_are_diff(self) -> None:
        oracle = make_item(
            sample="s",
            source_path="o.hwp",
            stream_path="/BodyText/Section0",
            record_index=0,
            tag_name="CTRL_HEADER",
            level=0,
            payload=pack_table_ctrl_payload(
                common_attr=1, x=0, y=0, width=1, height=1, out_margin=(80, 80, 40, 40)
            ),
            control_fourcc="tbl ",
        )
        generated = make_item(
            sample="s",
            source_path="g.hwp",
            stream_path="/BodyText/Section0",
            record_index=0,
            tag_name="CTRL_HEADER",
            level=0,
            payload=pack_table_ctrl_payload(
                common_attr=1, x=0, y=0, width=1, height=1, out_margin=(0, 0, 0, 0)
            ),
            control_fourcc="tbl ",
        )
        rows = {row.field_name: row for row in table_field_rows(oracle, generated)}
        self.assertEqual(rows["out_margin_left"].status, "diff")
        self.assertEqual(rows["ctrl_id"].status, "same")
        items, _stats = build_lcs_diff([oracle], [generated])
        self.assertTrue(is_table_candidate(items[0]))
        axes = table_probe_axes(items, [oracle], [generated])
        names = {axis.name: axis for axis in axes}
        self.assertEqual(len(names["ctrl_outer_margin"].rows), 1)
        self.assertEqual(len(names["table_attr"].rows), 0)

    def test_table_tail_axis(self) -> None:
        oracle = make_item(
            sample="s",
            source_path="o.hwp",
            stream_path="/BodyText/Section0",
            record_index=1,
            tag_name="TABLE",
            level=1,
            payload=pack_table_payload(table_attr=1, rows=1, cols=1, tail=b"\xff\xee"),
        )
        generated = make_item(
            sample="s",
            source_path="g.hwp",
            stream_path="/BodyText/Section0",
            record_index=1,
            tag_name="TABLE",
            level=1,
            payload=pack_table_payload(table_attr=1, rows=1, cols=1, tail=b"\x00\x00"),
        )
        items, _stats = build_lcs_diff([oracle], [generated])
        axes = {axis.name: axis for axis in table_probe_axes(items, [oracle], [generated])}
        self.assertEqual(len(axes["table_tail"].rows), 1)
        self.assertEqual(axes["table_tail"].rows[0].fields, ["table_tail_full"])


if __name__ == "__main__":
    unittest.main()
