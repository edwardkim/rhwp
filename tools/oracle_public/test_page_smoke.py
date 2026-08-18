#!/usr/bin/env python3
"""page_smoke.py 단위 시험 — tiny fixture 만. 269 PDF 전수 불필요.

실행 (저장소 루트 또는 이 디렉터리):
    python -m unittest tools.oracle_public.test_page_smoke
    python -m unittest tools/oracle_public/test_page_smoke.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import page_smoke as ps  # noqa: E402


def _touch(path: Path, text: str = "x") -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


class FakeDump:
    def __init__(self, mapping: dict[str, int] | int, rc: int = 0) -> None:
        self.mapping = mapping
        self.rc = rc
        self.calls: list[list[str]] = []

    def __call__(self, cmd, **_: object) -> subprocess.CompletedProcess[str]:
        self.calls.append(list(cmd))
        doc = Path(cmd[2]).name
        if isinstance(self.mapping, int):
            n = self.mapping
        else:
            n = self.mapping[doc]
        return subprocess.CompletedProcess(
            cmd, self.rc, json.dumps({"pageCount": n, "pages": []}), ""
        )


class PdfCountTests(unittest.TestCase):
    def test_minimal_pdf_count_matches_requested_pages(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "n.pdf"
            ps.write_minimal_pdf(path, 3)
            self.assertEqual(ps.pdf_page_count(path), 3)

    def test_one_page_pdf(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "one.pdf"
            ps.write_minimal_pdf(path, 1)
            self.assertEqual(ps.pdf_page_count(path), 1)

    def test_rejects_non_pdf(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "nope.txt"
            path.write_text("hello", encoding="utf-8")
            with self.assertRaises(ps.PageSmokeError):
                ps.pdf_page_count(path)

    def test_flate_object_stream_count(self) -> None:
        """한컴 PDF 처럼 /Count 가 Flate 스트림 안에만 있는 경우."""
        import zlib

        inner = b"<< /Type /Pages /Kids [3 0 R] /Count 6 >>"
        compressed = zlib.compress(inner)
        pdf = (
            b"%PDF-1.5\n%\xe2\xe3\xcf\xd3\n"
            b"1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n"
            b"2 0 obj << /Filter /FlateDecode /Length "
            + str(len(compressed)).encode("ascii")
            + b" >> stream\n"
            + compressed
            + b"\nendstream\nendobj\n"
            b"trailer << /Root 1 0 R >>\n%%EOF\n"
        )
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "flate.pdf"
            path.write_bytes(pdf)
            self.assertEqual(ps.pdf_page_count(path), 6)


class DumpPagesParseTests(unittest.TestCase):
    def test_json_page_count(self) -> None:
        self.assertEqual(ps.parse_dump_pages_count('{"pageCount": 7, "pages": []}'), 7)

    def test_json_with_provenance_fields(self) -> None:
        blob = json.dumps(
            {"pageCount": 4, "untrustedContent": True, "untrustedFields": ["pages"]}
        )
        self.assertEqual(ps.parse_dump_pages_count(blob), 4)

    def test_text_total_line(self) -> None:
        text = "문서 로드: sample.hwp (12페이지)\n=== 페이지 1 (global_idx=0, section=0, page_num=1) ===\n"
        self.assertEqual(ps.parse_dump_pages_count(text), 12)

    def test_header_fallback(self) -> None:
        text = "=== 페이지 1 (global_idx=0, section=0, page_num=1) ===\n=== 페이지 2 (global_idx=1, section=0, page_num=2) ===\n"
        self.assertEqual(ps.parse_dump_pages_count(text), 2)

    def test_empty_raises(self) -> None:
        with self.assertRaises(ps.PageSmokeError):
            ps.parse_dump_pages_count("")


class PairingTests(unittest.TestCase):
    def test_glob_exact_and_version_suffix(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "foo.hwp")
            _touch(repo / "samples" / "bar.hwp")
            ps.write_minimal_pdf(repo / "pdf" / "foo.pdf", 1)
            ps.write_minimal_pdf(repo / "pdf" / "foo-2022.pdf", 1)
            ps.write_minimal_pdf(repo / "pdf-2020" / "foo-2020.pdf", 1)
            pairs, unpaired = ps.discover_pairs(repo)
            pdfs = sorted(p.pdf.name for p in pairs)
            self.assertEqual(pdfs, ["foo-2020.pdf", "foo-2022.pdf", "foo.pdf"])
            self.assertEqual(unpaired, ["samples/bar.hwp"])
            self.assertTrue(all(p.doc.name == "foo.hwp" for p in pairs))

    def test_nested_pdf_large_subdir(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "issue2006" / "report.hwpx")
            ps.write_minimal_pdf(repo / "pdf-large" / "issue2006" / "report-2022.pdf", 2)
            pairs, unpaired = ps.discover_pairs(repo)
            self.assertEqual(len(pairs), 1)
            self.assertEqual(unpaired, [])
            self.assertEqual(pairs[0].pdf.name, "report-2022.pdf")

    def test_manifest_roundtrip_and_m01_keys(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "a.hwp")
            pdf = repo / "pdf" / "a-2024.pdf"
            ps.write_minimal_pdf(pdf, 2)
            man = repo / "pairs.json"
            man.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "pairs": [{"sample": "samples/a.hwp", "pdf": "pdf/a-2024.pdf"}],
                        "unpaired": ["samples/missing.hwp"],
                    }
                ),
                encoding="utf-8",
            )
            pairs, unpaired = ps.load_manifest(man, repo)
            self.assertEqual(len(pairs), 1)
            self.assertEqual(pairs[0].doc, doc)
            self.assertEqual(pairs[0].pdf, pdf)
            self.assertEqual(unpaired, ["samples/missing.hwp"])
            out = repo / "out.json"
            ps.write_manifest(out, pairs, repo, unpaired)
            again, unpaired2 = ps.load_manifest(out, repo)
            self.assertEqual(len(again), 1)
            self.assertEqual(unpaired2, ["samples/missing.hwp"])


class CompareTests(unittest.TestCase):
    def test_match_and_mismatch_are_data(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "tiny.hwp")
            match_pdf = repo / "pdf" / "tiny-2022.pdf"
            miss_pdf = repo / "pdf" / "tiny-2024.pdf"
            ps.write_minimal_pdf(match_pdf, 2)
            ps.write_minimal_pdf(miss_pdf, 5)
            fake = FakeDump(2)
            pairs = [
                ps.Pair(doc=doc, pdf=match_pdf, stem="tiny"),
                ps.Pair(doc=doc, pdf=miss_pdf, stem="tiny"),
            ]
            report = ps.run_smoke(
                repo=repo,
                pairs=pairs,
                unpaired=[],
                rhwp=Path("rhwp-fake"),
                strict=False,
                pair_source="test",
                runner=fake,
            )
            self.assertEqual([r.verdict for r in report.rows], ["MATCH", "MISMATCH"])
            self.assertEqual(report.rows[1].delta, -3)
            self.assertIn("page_smoke.py --pair", report.rows[1].repro)
            self.assertEqual(ps.exit_code(report), 0)
            strict = ps.Report(strict=True, rows=report.rows)
            self.assertEqual(ps.exit_code(strict), 1)
            # 같은 문서는 dump-pages 한 번만.
            self.assertEqual(len(fake.calls), 1)

    def test_dump_error_is_error_row_and_strict_fails(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "bad.hwp")
            pdf = repo / "pdf" / "bad-2022.pdf"
            ps.write_minimal_pdf(pdf, 1)

            def boom(cmd, **_: object) -> subprocess.CompletedProcess[str]:
                return subprocess.CompletedProcess(cmd, 1, "", "boom")

            report = ps.run_smoke(
                repo=repo,
                pairs=[ps.Pair(doc=doc, pdf=pdf, stem="bad")],
                unpaired=[],
                rhwp=Path("rhwp-fake"),
                strict=True,
                pair_source="test",
                runner=boom,
            )
            self.assertEqual(report.rows[0].verdict, "ERROR")
            self.assertEqual(report.rows[0].pdf_pages, 1)
            self.assertEqual(ps.exit_code(report), 1)

    def test_missing_rhwp_is_error_but_default_exit_zero(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "x.hwp")
            pdf = repo / "pdf" / "x-2022.pdf"
            ps.write_minimal_pdf(pdf, 1)
            report = ps.run_smoke(
                repo=repo,
                pairs=[ps.Pair(doc=doc, pdf=pdf, stem="x")],
                unpaired=["samples/lonely.hwp"],
                rhwp=None,
                strict=False,
                pair_source="test",
            )
            self.assertEqual(report.rows[0].verdict, "ERROR")
            self.assertIn("rhwp", report.rows[0].note)
            self.assertEqual(report.unpaired, ["samples/lonely.hwp"])
            self.assertEqual(ps.exit_code(report), 0)


class CliTests(unittest.TestCase):
    def test_pdf_count_flag(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            pdf = Path(td) / "p.pdf"
            ps.write_minimal_pdf(pdf, 4)
            rc = ps.main(["--pdf-count", str(pdf)])
            self.assertEqual(rc, 0)

    def test_pair_repro_json_default_zero_on_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "z.hwp")
            pdf = repo / "pdf" / "z-2022.pdf"
            ps.write_minimal_pdf(pdf, 3)
            fake = FakeDump(1)
            with patch.object(ps, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(ps.subprocess, "run", fake):
                    rc = ps.main(
                        [
                            "--repo",
                            str(repo),
                            "--pair",
                            str(doc),
                            str(pdf),
                            "--json",
                        ]
                    )
            self.assertEqual(rc, 0)

    def test_strict_cli_nonzero_on_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "z.hwp")
            pdf = repo / "pdf" / "z-2022.pdf"
            ps.write_minimal_pdf(pdf, 3)
            fake = FakeDump(1)
            buf: list[str] = []

            def capture(text: str) -> None:
                buf.append(text)

            with patch.object(ps, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(ps.subprocess, "run", fake):
                    with patch.object(sys, "stdout") as stdout:
                        stdout.write = capture
                        rc = ps.main(
                            [
                                "--repo",
                                str(repo),
                                "--pair",
                                "samples/z.hwp",
                                "pdf/z-2022.pdf",
                                "--strict",
                            ]
                        )
            self.assertEqual(rc, 1)
            joined = "".join(buf)
            self.assertIn("MISMATCH", joined)
            self.assertIn("page_smoke.py --pair", joined)

    def test_manifest_cli_does_not_scan_real_pdf_tree(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "fx" / "mini.hwp")
            pdf = repo / "fx" / "mini-2024.pdf"
            ps.write_minimal_pdf(pdf, 2)
            man = repo / "fx" / "pairs.json"
            ps.write_manifest(
                man,
                [ps.Pair(doc=doc, pdf=pdf, stem="mini")],
                repo,
            )
            fake = FakeDump(2)
            with patch.object(ps, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(ps.subprocess, "run", fake):
                    rc = ps.main(
                        [
                            "--repo",
                            str(repo),
                            "--manifest",
                            str(man),
                            "--json",
                        ]
                    )
            self.assertEqual(rc, 0)
            self.assertEqual(len(fake.calls), 1)

    def test_human_report_lists_repro_for_mismatch(self) -> None:
        row = ps.Row(
            doc="samples/foo.hwp",
            pdf="pdf/foo-2022.pdf",
            stem="foo",
            rhwp_pages=4,
            pdf_pages=5,
            delta=-1,
            verdict="MISMATCH",
            note="rhwp=4 pdf=5 delta=-1",
            repro=ps.repro_command("samples/foo.hwp", "pdf/foo-2022.pdf"),
        )
        text = ps.format_human(ps.Report(rows=[row], unpaired=["samples/none.hwp"]))
        self.assertIn("# REPRO python tools/oracle_public/page_smoke.py --pair samples/foo.hwp pdf/foo-2022.pdf", text)
        self.assertIn("# UNPAIRED samples/none.hwp", text)
        self.assertIn("판정은 데이터다", text)


if __name__ == "__main__":
    unittest.main()
