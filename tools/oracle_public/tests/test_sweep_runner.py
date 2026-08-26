#!/usr/bin/env python3
"""sweep_runner.py 단위 시험 — tiny fixture 만. 409 PDF 전수 불필요.

실행 (저장소 루트 또는 이 파일):
    python tools/oracle_public/tests/test_sweep_runner.py
    python -m unittest tools.oracle_public.tests.test_sweep_runner
"""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

TESTS_DIR = Path(__file__).resolve().parent
TOOL_DIR = TESTS_DIR.parent
MODULE_PATH = TOOL_DIR / "sweep_runner.py"

SPEC = importlib.util.spec_from_file_location("sweep_runner", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
sr = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = sr
SPEC.loader.exec_module(sr)


def _touch(path: Path, text: str = "x") -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


def _mini_tree(root: Path) -> None:
    _touch(root / "samples" / "exam_kor.hwp")
    _touch(root / "samples" / "exam_kor.hwpx")
    _touch(root / "samples" / "lonely.hwp")
    _touch(root / "samples" / "hwp3-sample.hwp")
    _touch(root / "samples" / "hwp3-sample-hwpx.hwpx")
    _touch(root / "samples" / "편람.hwp")
    _touch(root / "samples" / "편람.hwpx")
    _touch(root / "samples" / "3-09월_교육_통합_2022.hwp")
    _touch(root / "samples" / "basic" / "calendar_year.hwp")
    _touch(root / "samples" / "readme.txt")
    sr.write_minimal_pdf(root / "pdf" / "exam_kor-2020.pdf", 1)
    sr.write_minimal_pdf(root / "pdf" / "exam_kor-2022.pdf", 2)
    sr.write_minimal_pdf(root / "pdf-2020" / "exam_kor-2020.pdf", 1)
    sr.write_minimal_pdf(root / "pdf" / "hwp3-sample-hwpx-2022.pdf", 1)
    sr.write_minimal_pdf(root / "pdf" / "편람-hwp-2020.pdf", 1)
    sr.write_minimal_pdf(root / "pdf" / "편람-hwpx-2020.pdf", 1)
    sr.write_minimal_pdf(root / "pdf" / "3-09월_교육_통합_2022.pdf", 3)
    sr.write_minimal_pdf(root / "pdf" / "basic" / "calendar_year-2022.pdf", 1)
    sr.write_minimal_pdf(root / "pdf" / "unused-2022.pdf", 1)
    sr.write_minimal_pdf(root / "pdf-large" / "hwpx" / "lonely.pdf", 1)


class FakeDump:
    def __init__(self, mapping: dict[str, int] | int, rc: int = 0) -> None:
        self.mapping = mapping
        self.rc = rc
        self.calls: list[list[str]] = []

    def __call__(self, cmd, **_: object) -> subprocess.CompletedProcess[str]:
        self.calls.append(list(cmd))
        name = Path(cmd[2]).name if len(cmd) > 2 else ""
        if isinstance(self.mapping, int):
            n = self.mapping
        else:
            n = self.mapping[name]
        return subprocess.CompletedProcess(
            list(cmd), self.rc, json.dumps({"pageCount": n, "pages": []}), ""
        )


class ParseSuffixTests(unittest.TestCase):
    def test_plain_year(self) -> None:
        info = sr.parse_oracle_suffix("exam_kor", "exam_kor-2022.pdf")
        self.assertEqual(info, {"year": "2022", "variant": "2022", "fmt": None})

    def test_hwp_variant_does_not_steal_hwpx_stem(self) -> None:
        stolen = sr.parse_oracle_suffix("hwp3-sample", "hwp3-sample-hwpx-2022.pdf")
        self.assertIsNotNone(stolen)
        assert stolen is not None
        self.assertEqual(stolen["fmt"], "hwpx")
        own = sr.parse_oracle_suffix("hwp3-sample-hwpx", "hwp3-sample-hwpx-2022.pdf")
        self.assertEqual(own, {"year": "2022", "variant": "2022", "fmt": None})

    def test_exact_stem_with_year(self) -> None:
        info = sr.parse_oracle_suffix(
            "3-09월_교육_통합_2022", "3-09월_교육_통합_2022.pdf"
        )
        self.assertEqual(info, {"year": "2022", "variant": "exact", "fmt": None})

    def test_exact_stem_without_year_rejected(self) -> None:
        self.assertIsNone(sr.parse_oracle_suffix("lonely", "lonely.pdf"))


class PairingTests(unittest.TestCase):
    def test_discover_mini_tree(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _mini_tree(repo)
            pairs, unmatched = sr.discover_pairs(repo)
            samples = {item.sample for item in pairs}
            self.assertIn("samples/exam_kor.hwp", samples)
            self.assertIn("samples/편람.hwp", samples)
            self.assertIn("samples/hwp3-sample-hwpx.hwpx", samples)
            self.assertIn("samples/lonely.hwp", set(unmatched))
            self.assertIn("samples/hwp3-sample.hwp", set(unmatched))
            self.assertNotIn("samples/readme.txt", set(unmatched))
            hwp_pdfs = [p.pdf for p in pairs if p.sample == "samples/편람.hwp"]
            hwpx_pdfs = [p.pdf for p in pairs if p.sample == "samples/편람.hwpx"]
            self.assertEqual(hwp_pdfs, ["pdf/편람-hwp-2020.pdf"])
            self.assertEqual(hwpx_pdfs, ["pdf/편람-hwpx-2020.pdf"])

    def test_manifest_m01_schema(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "a.hwp")
            sr.write_minimal_pdf(repo / "pdf" / "a-2024.pdf", 2)
            man = repo / "oracle_pairs.json"
            man.write_text(
                json.dumps(
                    {
                        "schemaVersion": "1.0",
                        "targetPairCount": 269,
                        "pairCount": 409,
                        "pairs": [
                            {
                                "id": "samples/a.hwp::pdf/a-2024.pdf",
                                "sample": "samples/a.hwp",
                                "pdf": "pdf/a-2024.pdf",
                                "stem": "a",
                                "hancomVersion": "2024",
                                "sourceFormat": "hwp",
                                "oracleRoot": "pdf",
                            }
                        ],
                        "unmatched": [{"sample": "samples/missing.hwp"}],
                    }
                ),
                encoding="utf-8",
            )
            pairs, unmatched, raw = sr.load_manifest(man, repo)
            self.assertEqual(len(pairs), 1)
            self.assertEqual(pairs[0].sample, "samples/a.hwp")
            self.assertEqual(pairs[0].hancom_version, "2024")
            self.assertEqual(unmatched, ["samples/missing.hwp"])
            self.assertEqual(raw["pairCount"], 409)
            self.assertNotEqual(raw["pairCount"], sr.REFERENCE_TARGET_PAIR_COUNT)


class PdfCountTests(unittest.TestCase):
    def test_minimal_pdf_count(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "n.pdf"
            sr.write_minimal_pdf(path, 3)
            self.assertEqual(sr.pdf_page_count(path), 3)

    def test_flate_object_stream_count(self) -> None:
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
            self.assertEqual(sr.pdf_page_count(path), 6)


class DumpPagesParseTests(unittest.TestCase):
    def test_json_page_count(self) -> None:
        self.assertEqual(sr.parse_dump_pages_count('{"pageCount": 7, "pages": []}'), 7)

    def test_text_total_line(self) -> None:
        text = "문서 로드: sample.hwp (12페이지)\n=== 페이지 1 ===\n"
        self.assertEqual(sr.parse_dump_pages_count(text), 12)

    def test_empty_raises(self) -> None:
        with self.assertRaises(sr.SweepError):
            sr.parse_dump_pages_count("")


class CompareTests(unittest.TestCase):
    def test_match_mismatch_topn_default_exit_zero(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "tiny.hwp")
            match_pdf = repo / "pdf" / "tiny-2022.pdf"
            miss_pdf = repo / "pdf" / "tiny-2024.pdf"
            worse_pdf = repo / "pdf" / "tiny-2020.pdf"
            sr.write_minimal_pdf(match_pdf, 2)
            sr.write_minimal_pdf(miss_pdf, 5)
            sr.write_minimal_pdf(worse_pdf, 9)
            fake = FakeDump(2)
            pairs = [
                sr.Pair(sample="samples/tiny.hwp", pdf="pdf/tiny-2022.pdf", stem="tiny"),
                sr.Pair(sample="samples/tiny.hwp", pdf="pdf/tiny-2024.pdf", stem="tiny"),
                sr.Pair(sample="samples/tiny.hwp", pdf="pdf/tiny-2020.pdf", stem="tiny"),
            ]
            report = sr.run_sweep(
                repo=repo,
                pairs=pairs,
                unmatched=["samples/lonely.hwp"],
                rhwp=Path("rhwp-fake"),
                mode="cheap",
                strict=False,
                pair_source="test",
                top_n=1,
                runner=fake,
            )
            self.assertEqual(
                [row.verdict for row in report.rows],
                ["MATCH", "MISMATCH", "MISMATCH"],
            )
            self.assertEqual(report.rows[1].page_delta, -3)
            self.assertIn("sweep_runner.py --pair", report.rows[1].repro)
            self.assertGreater(report.rows[1].pdf_bytes or 0, 0)
            self.assertGreater(report.rows[1].sample_bytes or 0, 0)
            self.assertEqual(sr.exit_code(report), 0)
            top = report.top_failures()
            self.assertEqual(len(top), 1)
            self.assertEqual(top[0].pdf, "pdf/tiny-2020.pdf")
            self.assertGreater(top[0].score, report.rows[1].score)
            self.assertEqual(len(fake.calls), 1)
            self.assertTrue(doc.is_file())
            strict = sr.Report(strict=True, rows=report.rows, top_n=1)
            self.assertEqual(sr.exit_code(strict), 1)

    def test_missing_rhwp_is_error_but_default_exit_zero(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "x.hwp")
            sr.write_minimal_pdf(repo / "pdf" / "x-2022.pdf", 1)
            report = sr.run_sweep(
                repo=repo,
                pairs=[sr.Pair(sample="samples/x.hwp", pdf="pdf/x-2022.pdf", stem="x")],
                unmatched=[],
                rhwp=None,
                mode="cheap",
                strict=False,
                pair_source="test",
                top_n=10,
            )
            self.assertEqual(report.rows[0].verdict, "ERROR")
            self.assertIn("rhwp", report.rows[0].note)
            self.assertEqual(sr.exit_code(report), 0)

    def test_export_pdf_size_threshold_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            work = repo / "work"
            _touch(repo / "samples" / "z.hwp", "sample-bytes")
            pdf = repo / "pdf" / "z-2022.pdf"
            sr.write_minimal_pdf(pdf, 1)

            class FakeExport(FakeDump):
                def __call__(self, cmd, **kwargs: object) -> subprocess.CompletedProcess[str]:
                    self.calls.append(list(cmd))
                    if len(cmd) > 1 and cmd[1] == "export-pdf":
                        out = Path(cmd[cmd.index("-o") + 1])
                        out.parent.mkdir(parents=True, exist_ok=True)
                        out.write_bytes(b"%PDF-1.4\n" + b"X" * 4000)
                        return subprocess.CompletedProcess(list(cmd), 0, "", "")
                    return super().__call__(cmd, **kwargs)

            fake = FakeExport(1)
            report = sr.run_sweep(
                repo=repo,
                pairs=[sr.Pair(sample="samples/z.hwp", pdf="pdf/z-2022.pdf", stem="z")],
                unmatched=[],
                rhwp=Path("rhwp-fake"),
                mode="cheap",
                strict=False,
                pair_source="test",
                top_n=5,
                export_pdf=True,
                size_threshold=0.1,
                runner=fake,
                work_dir=work,
            )
            self.assertEqual(report.rows[0].verdict, "MISMATCH")
            self.assertIsNotNone(report.rows[0].size_ratio)
            self.assertIn("size_ratio", report.rows[0].note)
            self.assertIn("pages+size", report.rows[0].metric)


class ReportFormatTests(unittest.TestCase):
    def test_human_report_lists_repro_and_honest_counts(self) -> None:
        row = sr.Row(
            sample="samples/foo.hwp",
            pdf="pdf/foo-2022.pdf",
            stem="foo",
            verdict="MISMATCH",
            score=1_000_000.0,
            metric="pages",
            rhwp_pages=4,
            pdf_pages=5,
            page_delta=-1,
            pdf_bytes=1234,
            note="pages rhwp=4 pdf=5 delta=-1",
            repro=sr.repro_command("samples/foo.hwp", "pdf/foo-2022.pdf"),
        )
        text = sr.format_human(
            sr.Report(rows=[row], unmatched=["samples/none.hwp"], pair_count=409, top_n=10)
        )
        self.assertIn("REPRO python tools/oracle_public/sweep_runner.py --pair samples/foo.hwp pdf/foo-2022.pdf", text)
        self.assertIn("UNMATCHED samples/none.hwp", text)
        self.assertIn("판정은 데이터다", text)
        self.assertIn("measuredDevel=409", text)
        self.assertIn("targetPairCount=269(참고, 실측 아님)", text)
        self.assertIn("TOP 1 FAILURES", text)
        payload = sr.Report(rows=[row], pair_count=409).to_json()
        self.assertEqual(payload["measuredDevelPairCount"], 409)
        self.assertEqual(payload["targetPairCount"], 269)
        self.assertIn("실측이 아니다", payload["targetPairCountNote"])
        self.assertEqual(payload["topFailures"][0]["repro"], row.repro)


class CliTests(unittest.TestCase):
    def test_pair_json_default_zero_on_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "z.hwp")
            pdf = repo / "pdf" / "z-2022.pdf"
            sr.write_minimal_pdf(pdf, 3)
            fake = FakeDump(1)
            with patch.object(sr, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(sr.subprocess, "run", fake):
                    with patch.object(sys, "stdout") as stdout:
                        stdout.write = lambda *_a, **_k: None
                        rc = sr.main(
                            [
                                "--repo-root",
                                str(repo),
                                "--pair",
                                "samples/z.hwp",
                                "pdf/z-2022.pdf",
                                "--json",
                            ]
                        )
            self.assertEqual(rc, 0)

    def test_strict_cli_nonzero_on_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "z.hwp")
            sr.write_minimal_pdf(repo / "pdf" / "z-2022.pdf", 3)
            fake = FakeDump(1)
            buf: list[str] = []
            with patch.object(sr, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(sr.subprocess, "run", fake):
                    with patch.object(sys, "stdout") as stdout:
                        stdout.write = buf.append
                        rc = sr.main(
                            [
                                "--repo-root",
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
            self.assertIn("sweep_runner.py --pair", joined)

    def test_manifest_cli_does_not_scan_real_pdf_tree(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "fx" / "mini.hwp")
            pdf = repo / "fx" / "mini-2024.pdf"
            sr.write_minimal_pdf(pdf, 2)
            man = repo / "fx" / "pairs.json"
            man.write_text(
                json.dumps(
                    {
                        "schemaVersion": "1.0",
                        "pairCount": 409,
                        "targetPairCount": 269,
                        "pairs": [
                            {
                                "sample": "fx/mini.hwp",
                                "pdf": "fx/mini-2024.pdf",
                                "stem": "mini",
                                "hancomVersion": "2024",
                                "sourceFormat": "hwp",
                                "oracleRoot": "fx",
                            }
                        ],
                        "unmatched": [],
                    }
                ),
                encoding="utf-8",
            )
            fake = FakeDump(2)
            out = repo / "sweep.json"
            with patch.object(sr, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(sr.subprocess, "run", fake):
                    with patch.object(sys, "stdout") as stdout:
                        stdout.write = lambda *_a, **_k: None
                        rc = sr.main(
                            [
                                "--repo-root",
                                str(repo),
                                "--manifest",
                                str(man),
                                "--json",
                                "-o",
                                str(out),
                                "--top",
                                "3",
                            ]
                        )
            self.assertEqual(rc, 0)
            self.assertEqual(len(fake.calls), 1)
            payload = json.loads(out.read_text(encoding="utf-8"))
            self.assertEqual(payload["pairCount"], 409)
            self.assertEqual(payload["measuredDevelPairCount"], 409)
            self.assertEqual(payload["summary"]["pairs"], 1)
            self.assertEqual(payload["summary"]["match"], 1)

    def test_limit_and_top(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "a.hwp")
            _touch(repo / "samples" / "b.hwp")
            sr.write_minimal_pdf(repo / "pdf" / "a-2022.pdf", 2)
            sr.write_minimal_pdf(repo / "pdf" / "b-2022.pdf", 4)
            man = repo / "pairs.json"
            man.write_text(
                json.dumps(
                    {
                        "schemaVersion": "1.0",
                        "pairCount": 409,
                        "pairs": [
                            {"sample": "samples/a.hwp", "pdf": "pdf/a-2022.pdf"},
                            {"sample": "samples/b.hwp", "pdf": "pdf/b-2022.pdf"},
                        ],
                        "unmatched": [],
                    }
                ),
                encoding="utf-8",
            )
            fake = FakeDump({"a.hwp": 2, "b.hwp": 1})
            with patch.object(sr, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(sr.subprocess, "run", fake):
                    with patch.object(sys, "stdout") as stdout:
                        stdout.write = lambda *_a, **_k: None
                        rc = sr.main(
                            [
                                "--repo-root",
                                str(repo),
                                "--manifest",
                                str(man),
                                "--limit",
                                "1",
                                "--json",
                            ]
                        )
            self.assertEqual(rc, 0)
            self.assertEqual(len(fake.calls), 1)

    def test_missing_manifest_is_usage_error(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            rc = sr.main(
                [
                    "--repo-root",
                    td,
                    "--manifest",
                    str(Path(td) / "nope.json"),
                ]
            )
            self.assertEqual(rc, 2)

    def test_does_not_claim_269_when_resolver_measured_409(self) -> None:
        self.assertEqual(sr.MEASURED_DEVEL_PAIR_COUNT, 409)
        self.assertEqual(sr.REFERENCE_TARGET_PAIR_COUNT, 269)
        self.assertNotEqual(sr.MEASURED_DEVEL_PAIR_COUNT, sr.REFERENCE_TARGET_PAIR_COUNT)
        self.assertIn("409", sr.__doc__ or "")
        self.assertIn("visual_sweep.py", sr.__doc__ or "")


class OptionalModeTests(unittest.TestCase):
    def test_export_svg_page_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            work = repo / "work"
            _touch(repo / "samples" / "s.hwp")
            sr.write_minimal_pdf(repo / "pdf" / "s-2022.pdf", 2)

            class FakeSvg(FakeDump):
                def __call__(self, cmd, **kwargs: object) -> subprocess.CompletedProcess[str]:
                    self.calls.append(list(cmd))
                    if len(cmd) > 1 and cmd[1] == "export-svg":
                        out = Path(cmd[cmd.index("-o") + 1])
                        out.mkdir(parents=True, exist_ok=True)
                        (out / "page-0.svg").write_text("<svg/>", encoding="utf-8")
                        return subprocess.CompletedProcess(list(cmd), 0, "", "")
                    return super().__call__(cmd, **kwargs)

            fake = FakeSvg(2)
            report = sr.run_sweep(
                repo=repo,
                pairs=[sr.Pair(sample="samples/s.hwp", pdf="pdf/s-2022.pdf", stem="s")],
                unmatched=[],
                rhwp=Path("rhwp-fake"),
                mode="export-svg",
                strict=False,
                pair_source="test",
                top_n=5,
                runner=fake,
                work_dir=work,
            )
            self.assertEqual(report.rows[0].svg_pages, 1)
            self.assertEqual(report.rows[0].verdict, "MISMATCH")
            self.assertIn("export-svg", report.rows[0].repro)


if __name__ == "__main__":
    unittest.main()
