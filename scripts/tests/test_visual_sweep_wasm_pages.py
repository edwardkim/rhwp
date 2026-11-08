"""선택 쪽 내보내기가 전체 쪽수·글꼴 공급·누락 검사를 보존하는지 확인한다."""

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.tests.test_visual_sweep import SWEEP


class WasmSelectedPagesTests(unittest.TestCase):
    def exporter(self, command, **kwargs):
        if command[0] == "node":
            output = Path(command[command.index("--out") + 1])
            for name in ("raw_svg", "render_tree"):
                (output / name).mkdir(parents=True)
            for page in (1, 2, 3):
                (output / f"raw_svg/wasm_{page:03}.svg").write_text(
                    f'<svg><text x="4" font-family="Remote">WASM{page}</text></svg>',
                )
                (output / f"render_tree/tree_{page:03}.json").write_text(
                    json.dumps({"owner": "WASM", "page": page}),
                )
            (output / "manifest.json").write_text(
                '{"pageCount":3,"layoutGeneration":"2022"}',
            )
        else:
            output = Path(command[command.index("-o") + 1])
            (output / "native_001.svg").write_text('<svg><text>Native</text></svg>')
            # 선택 쪽과 Native 쪽 소속이 달라도 같은 문서의 글꼴 별칭을 공급한다.
            (output / "native_003.svg").write_text(
                '<svg><style>@font-face {font-family:"Remote";src:local("Face")}</style></svg>',
            )
            (output / "export-provenance.json").write_text('{"keep":true}')
        return subprocess.CompletedProcess(command, 0, "", "")

    def export(self, root, pages, exporter=None):
        with patch.object(SWEEP, "run", side_effect=exporter or self.exporter):
            SWEEP.export_wasm_target(
                root, root / "source.hwp", root / "pkg", "rhwp", root / "out",
                selected_pages=pages,
            )

    def test_selected_output_keeps_full_count_and_document_wide_font_policy(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.export(root, [2])
            base = root / "out"
            self.assertEqual([p.name for p in (base / "svg").glob("*.svg")], ["wasm_002.svg"])
            svg = (base / "svg/wasm_002.svg").read_text()
            self.assertIn('@font-face {font-family:"Remote";src:local("Face")}', svg)
            self.assertIn('<text x="4" font-family="Remote">WASM2</text>', svg)
            self.assertEqual(json.loads((base / "render_tree/tree_002.json").read_text()),
                             {"owner": "WASM", "page": 2})
            metadata = json.loads((base / "wasm-export-complete.json").read_text())
            self.assertEqual(metadata["pageCount"], 3)
            self.assertEqual(len(list((base / "wasm/raw_svg").glob("*.svg"))), 3)
            self.assertEqual(len(list((base / "wasm/render_tree").glob("*.json"))), 3)
            self.assertEqual(list((base / "font_policy").glob("*.svg")), [])
            self.assertTrue((base / "font_policy/export-provenance.json").is_file())
            ledger = json.loads((base / "font-policy-sources.json").read_text())
            self.assertEqual(len(ledger["removed_svg"]), 2)
            self.assertTrue(SWEEP.wasm_export_has_pages(base, [2]))
            self.assertFalse(SWEEP.wasm_export_has_pages(base, [1]))
            self.assertFalse(SWEEP.wasm_export_has_pages(base, None))

    def test_all_pages_and_reuse_require_both_svg_and_tree(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.export(root, None)
            base = root / "out"
            self.assertEqual(len(list((base / "svg").glob("*.svg"))), 3)
            self.assertTrue(SWEEP.wasm_export_has_pages(base, None))
            (base / "render_tree/tree_002.json").unlink()
            self.assertFalse(SWEEP.wasm_export_has_pages(base, None))
            self.assertFalse(SWEEP.wasm_export_has_pages(base, [2]))
            self.assertTrue(SWEEP.wasm_export_has_pages(base, [1]))

    def test_unselected_raw_svg_or_tree_missing_still_fails(self):
        for missing in ("raw_svg/wasm_003.svg", "render_tree/tree_003.json"):
            with self.subTest(missing=missing), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)

                def incomplete(command, **kwargs):
                    result = self.exporter(command, **kwargs)
                    if command[0] == "node":
                        (root / "out/wasm" / missing).unlink()
                    return result

                with self.assertRaisesRegex(SystemExit, "누락"):
                    self.export(root, [2], incomplete)
                self.assertFalse((root / "out/wasm-export-complete.json").exists())

    def test_out_of_range_page_cannot_publish_completion(self):
        for pages in ([0], [4], []):
            with self.subTest(pages=pages), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                with self.assertRaisesRegex(SystemExit, "범위"):
                    self.export(root, pages)
                self.assertFalse((root / "out/wasm-export-complete.json").exists())


if __name__ == "__main__":
    unittest.main()
