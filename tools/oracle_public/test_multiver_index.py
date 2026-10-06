#!/usr/bin/env python3
"""multiver_index.py 가드 테스트 — 실 코퍼스 불요.

픽스처 PDF 는 테스트가 임시 디렉터리에 직접 만든다. 쪽수 불일치는 실제로
다른 page count 를 가진 PDF 를 만들어 재고, LFS 포인터는 쪽수를 넣지 않는다.
픽셀 차이 필드는 색인에 등장하지 않아야 한다.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import multiver_index as mx  # noqa: E402


def write_minimal_pdf(path: Path, page_count: int) -> None:
    if page_count < 1:
        raise ValueError("page_count >= 1")
    catalog = b"<< /Type /Catalog /Pages 2 0 R >>"
    page_nums = list(range(3, 3 + page_count))
    kids = " ".join(f"{n} 0 R" for n in page_nums)
    pages = f"<< /Type /Pages /Kids [{kids}] /Count {page_count} >>".encode()
    page_objs = [
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>" for _ in page_nums
    ]
    body = [catalog, pages, *page_objs]
    out = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for i, data in enumerate(body, start=1):
        offsets.append(len(out))
        out.extend(f"{i} 0 obj\n".encode())
        out.extend(data)
        out.extend(b"\nendobj\n")
    xref_pos = len(out)
    out.extend(f"xref\n0 {len(offsets)}\n".encode())
    out.extend(b"0000000000 65535 f \n")
    for off in offsets[1:]:
        out.extend(f"{off:010d} 00000 n \n".encode())
    out.extend(
        f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n".encode()
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(bytes(out))


def write_lfs_pointer(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "version https://git-lfs.github.com/spec/v1\n"
        "oid sha256:" + ("ab" * 32) + "\n"
        "size 99\n",
        encoding="ascii",
        newline="\n",
    )


class ParseOracleNameTests(unittest.TestCase):
    def test_plain_version_suffix(self) -> None:
        stem, ver, variants = mx.parse_oracle_name("exam_kor-2022")
        self.assertEqual(stem, "exam_kor")
        self.assertEqual(ver, "2022")
        self.assertEqual(variants, [])

    def test_date_stem_keeps_leading_2010(self) -> None:
        stem, ver, variants = mx.parse_oracle_name("2010-01-06-2022")
        self.assertEqual(stem, "2010-01-06")
        self.assertEqual(ver, "2022")
        self.assertEqual(variants, [])

    def test_underscore_year_stays_in_stem(self) -> None:
        stem, ver, variants = mx.parse_oracle_name("3-09월_교육_통합_2022")
        self.assertEqual(stem, "3-09월_교육_통합_2022")
        self.assertIsNone(ver)
        self.assertEqual(variants, [])

    def test_variant_after_version(self) -> None:
        stem, ver, variants = mx.parse_oracle_name("편람-2010-kopub")
        self.assertEqual(stem, "편람")
        self.assertEqual(ver, "2010")
        self.assertEqual(variants, ["kopub"])

    def test_variant_before_version(self) -> None:
        stem, ver, variants = mx.parse_oracle_name("편람-hwp-kopub-2020")
        self.assertEqual(stem, "편람")
        self.assertEqual(ver, "2020")
        self.assertEqual(variants, ["hwp", "kopub"])

    def test_dual_year_keeps_source_year_in_stem(self) -> None:
        stem, ver, variants = mx.parse_oracle_name("hwp3-sample16-hwp5-2018-2020")
        self.assertEqual(stem, "hwp3-sample16-hwp5-2018")
        self.assertEqual(ver, "2020")
        self.assertEqual(variants, [])


class MeasureTests(unittest.TestCase):
    def test_pypdf_reads_minimal_page_count(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "n.pdf"
            write_minimal_pdf(path, 3)
            pages, status = mx.measure_page_count(path)
        self.assertEqual(pages, 3)
        self.assertEqual(status, "pypdf")

    def test_lfs_pointer_is_unmeasured(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "big.pdf"
            write_lfs_pointer(path)
            pages, status = mx.measure_page_count(path)
        self.assertIsNone(pages)
        self.assertEqual(status, "lfs_pointer")

    def test_checked_in_lfs_fixture_is_unmeasured(self) -> None:
        fixture = Path(__file__).with_name("fixtures") / "lfs_pointer.pdf"
        self.assertTrue(fixture.is_file(), fixture)
        pages, status = mx.measure_page_count(fixture)
        self.assertIsNone(pages)
        self.assertEqual(status, "lfs_pointer")

    def test_not_pdf_is_unmeasured(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "x.pdf"
            path.write_bytes(b"not a pdf")
            pages, status = mx.measure_page_count(path)
        self.assertIsNone(pages)
        self.assertEqual(status, "not_pdf")


class IndexFixtureTests(unittest.TestCase):
    def _tree(self, root: Path) -> None:
        write_minimal_pdf(root / "pdf" / "same-2018.pdf", 1)
        write_minimal_pdf(root / "pdf" / "same-2020.pdf", 1)
        write_minimal_pdf(root / "pdf" / "same-2022.pdf", 2)
        write_minimal_pdf(root / "pdf" / "agree-2020.pdf", 1)
        write_minimal_pdf(root / "pdf" / "agree-2022.pdf", 1)
        write_minimal_pdf(root / "pdf" / "solo-2022.pdf", 1)
        write_minimal_pdf(root / "pdf" / "nover.pdf", 4)
        write_minimal_pdf(root / "pdf" / "big-2022.pdf", 5)
        write_lfs_pointer(root / "pdf" / "orphan.pdf")

    def test_incorporation_lists_the_single_oracle_tree(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._tree(root)
            index = mx.build_index(root)
        self.assertEqual(index["trees"]["pdf"]["file_count"], 9)
        paths = [r["path"] for r in index["incorporation"]["pdf"]]
        self.assertIn("pdf/same-2020.pdf", paths)
        self.assertIn("pdf/big-2022.pdf", paths)
        self.assertIn("pdf/orphan.pdf", paths)

    def test_disagreement_is_measured_not_invented(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._tree(root)
            index = mx.build_index(root)
        disagree = {s["stem"]: s for s in index["disagreements"]}
        self.assertIn("same", disagree)
        rec = disagree["same"]
        self.assertEqual(rec["kind"], "page_count_disagree")
        self.assertEqual(rec["measured_page_counts"]["2018"], [1])
        self.assertEqual(rec["measured_page_counts"]["2022"], [2])
        self.assertEqual(rec["measured_page_counts"]["2020"], [1])
        self.assertEqual(rec["min_pages"], 1)
        self.assertEqual(rec["max_pages"], 2)
        self.assertEqual(index["counts"]["page_count_disagree"], 1)

    def test_agree_same_page_count(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._tree(root)
            index = mx.build_index(root)
        agrees = {s["stem"]: s for s in index["multiver_stems"] if s["kind"] == "page_count_agree"}
        self.assertIn("agree", agrees)
        self.assertEqual(agrees["agree"]["min_pages"], 1)
        self.assertEqual(agrees["agree"]["max_pages"], 1)

    def test_directory_does_not_infer_a_version(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._tree(root)
            index = mx.build_index(root)
        nover = next(r for r in index["files"] if r["path"] == "pdf/nover.pdf")
        self.assertIsNone(nover["hangul_version"])
        self.assertEqual(nover["version_source"], "unknown")
        self.assertFalse(nover["version_inferred"])
        self.assertEqual(nover["page_count"], 4)

    def test_unversioned_lfs_pointer_stays_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._tree(root)
            index = mx.build_index(root)
        orphan = next(r for r in index["files"] if r["path"] == "pdf/orphan.pdf")
        self.assertIsNone(orphan["hangul_version"])
        self.assertEqual(orphan["version_source"], "unknown")
        self.assertIsNone(orphan["page_count"])
        self.assertEqual(orphan["page_count_status"], "lfs_pointer")

    def test_pixel_diff_is_marked_out_of_scope(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self._tree(root)
            index = mx.build_index(root)
            blob = json.dumps(index)
        self.assertEqual(index["pixel_diff"], "out_of_scope")
        self.assertEqual(index["metric"], "pypdf_page_count")
        self.assertNotIn("pixel_delta", blob)
        self.assertNotIn("ssimulacra", blob.lower())

    def test_write_reports_roundtrip(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            out = Path(tmp) / "reports"
            self._tree(root)
            index = mx.build_index(root)
            written = mx.write_reports(index, out)
            names = {p.name for p in written}
            self.assertEqual(
                names,
                {
                    "incorporation_manifest.json",
                    "multiver_disagreements.json",
                    "multiver_index.md",
                },
            )
            manifest = json.loads((out / "incorporation_manifest.json").read_text(encoding="utf-8"))
            dis = json.loads((out / "multiver_disagreements.json").read_text(encoding="utf-8"))
            md = (out / "multiver_index.md").read_text(encoding="utf-8")
        self.assertEqual(manifest["counts"]["page_count_disagree"], 1)
        self.assertEqual(len(dis["disagreements"]), 1)
        self.assertIn("same", md)
        self.assertIn("out_of_scope", md)

    def test_single_default_tree_is_present(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_minimal_pdf(root / "pdf" / "a-2022.pdf", 1)
            index = mx.build_index(root)
        self.assertTrue(index["trees"]["pdf"]["present"])
        self.assertEqual(set(index["trees"]), {"pdf"})
        self.assertEqual(index["counts"]["files"], 1)


class CliTests(unittest.TestCase):
    def test_usage_error_on_empty_trees(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            code = mx.main(["--root", tmp, "--trees", ""])
        self.assertEqual(code, 2)

    def test_cli_writes_reports(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            out = Path(tmp) / "out"
            write_minimal_pdf(root / "pdf" / "x-2018.pdf", 1)
            write_minimal_pdf(root / "pdf" / "x-2022.pdf", 2)
            code = mx.main(
                ["--root", str(root), "--write-reports", "--out-dir", str(out), "--json"]
            )
            self.assertEqual(code, 0)
            self.assertTrue((out / "multiver_disagreements.json").is_file())


if __name__ == "__main__":
    unittest.main()
