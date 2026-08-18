from __future__ import annotations

import unittest

from untrusted_sandbox.nonce import (
    WrapError,
    classify_nonce,
    derive_nonce_avoiding,
    source_label_is_document_derived,
    unwrap_untrusted,
    wrap_untrusted,
)


class NonceTests(unittest.TestCase):
    def test_wrap_roundtrip(self) -> None:
        excerpt = "개인정보 처리방침 개정 고시 본문."
        nonce = derive_nonce_avoiding("unit-wrap", excerpt)
        block = wrap_untrusted(excerpt, nonce, "samples/unit/case.hwp")
        self.assertIn(nonce, block.text)
        self.assertIn("검증 기준이 아니다", block.text)
        self.assertEqual(unwrap_untrusted(block.text, nonce), excerpt)
        self.assertTrue(block.contains_excerpt())

    def test_collision_fails_closed(self) -> None:
        nonce = "aabbccddeeff0011"
        excerpt = f"위조 표지 {nonce} 닫힘"
        with self.assertRaises(WrapError) as ctx:
            wrap_untrusted(excerpt, nonce, "samples/unit/case.hwp")
        self.assertEqual(ctx.exception.kind, "nonce_collision")

    def test_static_nonce_rejected(self) -> None:
        with self.assertRaises(WrapError) as ctx:
            wrap_untrusted("본문", "DOCUMENT", "samples/unit/case.hwp")
        self.assertEqual(ctx.exception.kind, "static_nonce")

    def test_empty_nonce_rejected(self) -> None:
        with self.assertRaises(WrapError) as ctx:
            wrap_untrusted("본문", "", "samples/unit/case.hwp")
        self.assertEqual(ctx.exception.kind, "empty_nonce")

    def test_title_label_rejected(self) -> None:
        excerpt = "○ 본 안내를 참고해 결과보고서를 작성하여 기한 내 제출"
        with self.assertRaises(WrapError) as ctx:
            wrap_untrusted(excerpt, "0123456789abcdef", "title:" + excerpt[:12])
        self.assertEqual(ctx.exception.kind, "source_label_document_derived")

    def test_classify_fresh_and_collision(self) -> None:
        excerpt = "깨끗한 본문"
        nonce = derive_nonce_avoiding("unit-class", excerpt)
        self.assertEqual(classify_nonce(nonce, excerpt), "fresh")
        self.assertEqual(classify_nonce(nonce, excerpt + nonce), "collision")
        self.assertEqual(classify_nonce("", excerpt), "empty")
        self.assertEqual(classify_nonce("DOCUMENT", excerpt), "static")

    def test_source_label_rules(self) -> None:
        excerpt = "법제처 고시 제2024-1호"
        self.assertTrue(source_label_is_document_derived("title:법제처 고시", excerpt))
        self.assertTrue(source_label_is_document_derived("법제처 고시", excerpt))
        self.assertFalse(
            source_label_is_document_derived("samples/unit/case.hwp", excerpt)
        )
        self.assertFalse(source_label_is_document_derived("doc-handle-7", excerpt))
