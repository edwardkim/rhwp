from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "check_markdown_links.py"
SPEC = importlib.util.spec_from_file_location("check_markdown_links", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"검사기 모듈을 불러올 수 없습니다: {MODULE_PATH}")
CHECKER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHECKER
SPEC.loader.exec_module(CHECKER)


class CapabilityRegistryFormatTests(unittest.TestCase):
    def test_registration_id_accepts_positive_issue_number_without_leading_zero(self) -> None:
        for value in ("CAP-1", "CAP-3398", "LEGACY-d86c935bc"):
            with self.subTest(value=value):
                self.assertIsNotNone(CHECKER.CAPABILITY_ID_RE.fullmatch(value))

    def test_registration_id_rejects_zero_and_leading_zero(self) -> None:
        for value in ("CAP-0", "CAP-01", "CAP-", "LEGACY-D86C935BC"):
            with self.subTest(value=value):
                self.assertIsNone(CHECKER.CAPABILITY_ID_RE.fullmatch(value))

    def test_capability_slug_requires_canonical_kebab_case(self) -> None:
        for value in ("bug-hunter", "rhwp-cli", "a1"):
            with self.subTest(value=value):
                self.assertIsNotNone(CHECKER.CAPABILITY_SLUG_RE.fullmatch(value))
        for value in ("bug-hunter-", "bug--hunter", "Bug-hunter", "-bug-hunter"):
            with self.subTest(value=value):
                self.assertIsNone(CHECKER.CAPABILITY_SLUG_RE.fullmatch(value))

    def test_table_cells_keeps_escaped_pipe_inside_cell(self) -> None:
        row = (
            "| `CAP-3398` | `bug-hunter` | `export-svg \\| export-pdf` 대조 | "
            "[권위](manual.md) | — | — | active · maintainers |"
        )

        cells = CHECKER.table_cells(row)

        self.assertIsNotNone(cells)
        self.assertEqual(len(cells), 7)
        self.assertEqual(cells[2], "`export-svg \\| export-pdf` 대조")

    def test_inline_code_filename_is_not_interpreted_as_a_link(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            document = Path(directory) / "report.md"
            document.write_text(
                "`admrul_downloads\\3190263_[별지](문서).hwp` [실제 링크](guide.md)\n",
                encoding="utf-8",
            )

            self.assertEqual(CHECKER.destinations_in_markdown(document), [(1, "guide.md")])


if __name__ == "__main__":
    unittest.main()
