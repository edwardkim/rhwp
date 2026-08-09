#!/usr/bin/env python3
"""``pi_map_hash.py``의 입력·실패·TSV 식별자 계약 회귀 시험."""
from __future__ import annotations

import csv
import importlib.util
import io
import stat
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("pi_map_hash.py")
SPEC = importlib.util.spec_from_file_location("pi_map_hash", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
PI_MAP_HASH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PI_MAP_HASH)


class PiMapHashTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.root = Path(self.tempdir.name)
        self.exe = self.root / "fake_rhwp.py"
        self.exe.write_text(
            "#!/usr/bin/env python3\n"
            "import sys\n"
            "from pathlib import Path\n"
            "name = Path(sys.argv[-1]).name\n"
            "if name == 'bad.hwp':\n"
            "    print('fixture failure', file=sys.stderr)\n"
            "    raise SystemExit(7)\n"
            "if name == 'no-pages.hwp':\n"
            "    print('unrelated diagnostic')\n"
            "    raise SystemExit(0)\n"
            "print('=== 페이지 1 (global_idx=0, section=0, page_num=1) ===')\n"
            "print('  FullParagraph pi=2')\n"
            "print('=== 페이지 2 (global_idx=1, section=0, page_num=2) ===')\n"
            "print('  FullParagraph pi=3')\n",
            encoding="utf-8",
        )
        self.exe.chmod(self.exe.stat().st_mode | stat.S_IXUSR)

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    def write_chunk(self, lines: list[str]) -> Path:
        chunks = self.root / "chunks"
        chunks.mkdir()
        (chunks / "chunk_001.txt").write_text("\n".join(lines), encoding="utf-8")
        return chunks

    def test_success_keeps_full_path_for_same_basename(self) -> None:
        chunks = self.write_chunk(["/left/same.hwp", "/right/same.hwp"])
        output = self.root / "map.tsv"

        self.assertEqual(
            PI_MAP_HASH.main([str(chunks), str(output), "--exe", str(self.exe), "--jobs", "2"]),
            0,
        )

        with output.open(encoding="utf-8", newline="") as handle:
            rows = list(csv.DictReader(handle, delimiter="\t"))
        self.assertEqual([row["doc"] for row in rows], ["/left/same.hwp", "/right/same.hwp"])
        self.assertTrue(all(row["pi_hash"] for row in rows))
        self.assertEqual([row["pages"] for row in rows], ["2", "2"])

    def test_failure_preserves_existing_output(self) -> None:
        chunks = self.write_chunk(["/ok.hwp", "/bad.hwp"])
        output = self.root / "map.tsv"
        output.write_text("previous result\n", encoding="utf-8")
        stderr = io.StringIO()

        with redirect_stderr(stderr):
            result = PI_MAP_HASH.main([str(chunks), str(output), "--exe", str(self.exe)])

        self.assertEqual(result, 1)
        self.assertEqual(output.read_text(encoding="utf-8"), "previous result\n")
        self.assertIn("bad.hwp", stderr.getvalue())
        self.assertIn("기존 TSV는 유지", stderr.getvalue())

    def test_success_without_page_map_is_failure(self) -> None:
        digest, pages, error = PI_MAP_HASH.run_one(str(self.exe), "/no-pages.hwp", 10)
        self.assertIsNone(digest)
        self.assertIsNone(pages)
        self.assertIn("페이지 지도", error or "")

    def test_empty_chunks_are_rejected(self) -> None:
        chunks = self.root / "chunks"
        chunks.mkdir()
        with self.assertRaisesRegex(ValueError, "chunk_\\*\\.txt"):
            PI_MAP_HASH.load_files(chunks)


if __name__ == "__main__":
    unittest.main()
