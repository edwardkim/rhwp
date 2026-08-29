#!/usr/bin/env python
"""오라클 자체의 계약 — 한글도 rhwp 도 없이 돈다.

파싱과 판정 로직이 조용히 썩으면 오라클이 **틀린 표**를 자신 있게 낸다. 그쪽이
아무 표도 없는 것보다 나쁘다. 여기서 막는다.

  python tools/hangul_rotation_oracle/test_oracle.py
"""
from __future__ import annotations

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from oracle import TRANSFORM, Transform, verdict_of  # noqa: E402


class TestTransformParsing(unittest.TestCase):
    """`rhwp dump` 변환 줄 파싱 — mydocs/manual/dump_command.md 의 형식과 함께 움직인다."""

    LINE = (
        "        변환: 뒤집기=(false,true), 회전=34, "
        "flip=0x26080000, rotateImage=false"
    )

    def test_parses_all_five_fields(self) -> None:
        found = TRANSFORM.findall(self.LINE)
        self.assertEqual(len(found), 1)
        h, v, angle, flip, ri = found[0]
        self.assertEqual((h, v, angle, flip, ri), ("false", "true", "34", "26080000", "false"))

    def test_parses_negative_angle(self) -> None:
        line = "  변환: 뒤집기=(false,false), 회전=-90, flip=0x00000000, rotateImage=true"
        self.assertEqual(TRANSFORM.findall(line)[0][2], "-90")

    def test_ignores_old_format_line(self) -> None:
        """flip 필드가 없는 종전 형식은 매치되지 않아야 한다 — 조용한 0 표본 방지."""
        old = "    변환: 뒤집기=(false,false), 회전=67"
        self.assertEqual(TRANSFORM.findall(old), [])


class TestTransform(unittest.TestCase):
    def test_rotated_uses_modulo(self) -> None:
        self.assertFalse(Transform(False, False, 360, 0, False).rotated)
        self.assertFalse(Transform(False, False, 0, 0, False).rotated)
        self.assertTrue(Transform(False, False, 34, 0, False).rotated)

    def test_bit19_reads_the_documented_mask(self) -> None:
        self.assertTrue(Transform(False, False, 0, 0x0008_0000, False).bit19)
        self.assertFalse(Transform(False, False, 0, 0x0004_0000, False).bit19)


class TestVerdict(unittest.TestCase):
    def test_identical_is_no_change(self) -> None:
        t = Transform(False, False, 34, 0x2608_0000, False)
        self.assertEqual(verdict_of(t, Transform(False, False, 34, 0x2608_0000, False)), "무변화")

    def test_names_the_bit_that_moved(self) -> None:
        """한글 2024 실측 사례 — 저장 시 bit16 이 추가된다."""
        before = Transform(False, False, 34, 0x2608_0000, False)
        after = Transform(False, False, 34, 0x2609_0000, False)
        verdict = verdict_of(before, after)
        self.assertIn("bit16", verdict)
        self.assertIn("꺼짐->켜짐", verdict)
        self.assertNotIn("bit19", verdict, "움직이지 않은 비트는 언급하지 않는다")

    def test_reports_angle_and_rotate_image(self) -> None:
        before = Transform(False, False, 34, 0, False)
        after = Transform(False, False, 0, 0, True)
        verdict = verdict_of(before, after)
        self.assertIn("각도 34->0", verdict)
        self.assertIn("rotateImage 0->1", verdict)


if __name__ == "__main__":
    unittest.main()
