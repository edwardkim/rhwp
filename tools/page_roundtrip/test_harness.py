#!/usr/bin/env python3
"""page_roundtrip 하네스 단위 시험 — 가짜 rhwp, 실문서 전수 불필요.

실행 (저장소 루트):
    python -m unittest tools.page_roundtrip.test_harness
    python -m unittest tools/page_roundtrip/test_harness.py
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
import harness as prt  # noqa: E402


def _touch(path: Path, text: str = "x") -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


class FakeRhwp:
    """export-hwpx/convert --verify-pages --json 을 흉내 낸다."""

    def __init__(self, mapping: dict[str, tuple[int, int]] | tuple[int, int], rc: int | None = None) -> None:
        self.mapping = mapping
        self.rc = rc
        self.calls: list[list[str]] = []

    def __call__(self, cmd, **_: object) -> subprocess.CompletedProcess[str]:
        self.calls.append(list(cmd))
        # [rhwp, export-hwpx|convert, input, output, --verify-pages, --json]
        name = Path(cmd[2]).name
        if isinstance(self.mapping, tuple):
            before, after = self.mapping
        else:
            before, after = self.mapping[name]
        identical = before == after
        payload = {
            "schemaVersion": 1,
            "verifyPages": {"before": before, "after": after, "identical": identical},
            "untrustedContent": False,
        }
        if self.rc is not None:
            rc = self.rc
        else:
            rc = 0 if identical else 4
        return subprocess.CompletedProcess(cmd, rc, json.dumps(payload), "")


class ClassifyTests(unittest.TestCase):
    def test_four_way_verdict(self) -> None:
        self.assertEqual(prt.classify(True, cataloged=False), "MATCH")
        self.assertEqual(prt.classify(False, cataloged=False), "MISMATCH")
        self.assertEqual(prt.classify(False, cataloged=True), "EXPECTED_FAIL")
        self.assertEqual(prt.classify(True, cataloged=True), "UNEXPECTED_PASS")

    def test_error_and_missing_are_not_skips(self) -> None:
        self.assertEqual(prt.classify(None, cataloged=True, error=True), "ERROR")
        self.assertEqual(prt.classify(None, cataloged=True, missing=True), "CATALOG_MISSING")


class ParseTests(unittest.TestCase):
    def test_json_envelope(self) -> None:
        blob = json.dumps({"verifyPages": {"before": 64, "after": 65, "identical": False}})
        self.assertEqual(prt.parse_verify_pages(blob, ""), (64, 65))

    def test_json_with_noise_prefix(self) -> None:
        blob = '저장 완료\n{"verifyPages": {"before": 4, "after": 4, "identical": true}}'
        self.assertEqual(prt.parse_verify_pages(blob, ""), (4, 4))

    def test_text_fail_on_stderr(self) -> None:
        err = "검증 실패(--verify-pages): 변환 전 35쪽, 재파싱 후 36쪽"
        self.assertEqual(prt.parse_verify_pages("", err), (35, 36))

    def test_text_pass(self) -> None:
        self.assertEqual(prt.parse_verify_pages("검증 통과(--verify-pages): 2쪽", ""), (2, 2))

    def test_empty(self) -> None:
        self.assertIsNone(prt.parse_verify_pages("", ""))


class CatalogTests(unittest.TestCase):
    def test_shipped_catalog_lists_m05_issues(self) -> None:
        entries = prt.load_catalog(HERE / "catalog.json")
        issues = {e.issue for e in entries}
        self.assertEqual(issues, {3518, 3521, 3737, 4056, 5128})
        self.assertNotIn(4882, issues)
        self.assertTrue(all(e.route == "hwpx" for e in entries))
        docs = {e.doc for e in entries}
        self.assertIn("samples/hwp3-sample16.hwp", docs)
        self.assertIn("samples/issue-505-equations.hwp", docs)
        self.assertFalse(any("중간진도보고서" in e.doc for e in entries))
        self.assertTrue(any("revision1.3" in e.doc for e in entries))

    def test_ci_subset_includes_cataloged_fixture(self) -> None:
        docs = prt.load_manifest(HERE / "fixtures" / "ci-subset.json", HERE.parents[1])
        rels = [p.as_posix().replace("\\", "/") for p in docs]
        joined = "\n".join(rels)
        self.assertIn("issue-505-equations.hwp", joined)
        self.assertGreaterEqual(len(docs), 2)


class RunTests(unittest.TestCase):
    def test_match_mismatch_expected_fail_unexpected_pass(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            ok = _touch(repo / "samples" / "ok.hwp")
            bad = _touch(repo / "samples" / "bad.hwp")
            known = _touch(repo / "samples" / "known.hwp")
            stale = _touch(repo / "samples" / "stale.hwp")
            catalog = [
                prt.CatalogEntry("samples/known.hwp", "hwpx", 3518, "known"),
                prt.CatalogEntry("samples/stale.hwp", "hwpx", 4056, "stale"),
            ]
            fake = FakeRhwp({"ok.hwp": (1, 1), "bad.hwp": (2, 3), "known.hwp": (4, 1), "stale.hwp": (5, 5)})
            report = prt.run_harness(
                repo=repo,
                docs=[ok, bad, known, stale],
                routes=["hwpx"],
                catalog=catalog,
                rhwp=Path("rhwp-fake"),
                strict=False,
                source="test",
                runner=fake,
            )
            by_doc = {r.doc: r.verdict for r in report.rows}
            self.assertEqual(by_doc["samples/ok.hwp"], "MATCH")
            self.assertEqual(by_doc["samples/bad.hwp"], "MISMATCH")
            self.assertEqual(by_doc["samples/known.hwp"], "EXPECTED_FAIL")
            self.assertEqual(by_doc["samples/stale.hwp"], "UNEXPECTED_PASS")
            self.assertEqual(report.rows[1].equal, False)
            self.assertEqual(report.rows[2].issue, 3518)
            self.assertIn("harness.py --file", report.rows[1].repro)
            self.assertEqual(prt.exit_code(report), 0)
            states = {c.doc: c.state for c in report.catalog}
            self.assertEqual(states["samples/known.hwp"], "run")
            self.assertEqual(states["samples/stale.hwp"], "run")

    def test_catalog_not_silently_skipped_on_limit(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            first = _touch(repo / "samples" / "a.hwp")
            _touch(repo / "samples" / "known.hwp")
            catalog = [prt.CatalogEntry("samples/known.hwp", "hwpx", 3518, "known")]
            fake = FakeRhwp((1, 1))
            report = prt.run_harness(
                repo=repo,
                docs=[first],
                routes=["hwpx"],
                catalog=catalog,
                rhwp=Path("rhwp-fake"),
                strict=False,
                source="limit",
                runner=fake,
            )
            self.assertEqual(len(report.rows), 1)
            self.assertEqual(report.rows[0].doc, "samples/a.hwp")
            self.assertEqual(report.catalog[0].state, "held")
            self.assertEqual(report.catalog[0].doc, "samples/known.hwp")
            self.assertEqual(report.summary["catalog_held"], 1)

    def test_full_sweep_missing_catalog_is_a_row(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            present = _touch(repo / "samples" / "ok.hwp")
            catalog = [prt.CatalogEntry("samples/ghost.hwp", "hwpx", 3518, "ghost")]
            fake = FakeRhwp((1, 1))
            report = prt.run_harness(
                repo=repo,
                docs=[present],
                routes=["hwpx"],
                catalog=catalog,
                rhwp=Path("rhwp-fake"),
                strict=True,
                source="glob:samples",
                runner=fake,
                require_missing_rows=True,
            )
            verdicts = [r.verdict for r in report.rows]
            self.assertIn("CATALOG_MISSING", verdicts)
            self.assertEqual(prt.exit_code(report), 1)

    def test_strict_fails_on_new_violation_not_expected_fail(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            known = _touch(repo / "samples" / "known.hwp")
            catalog = [prt.CatalogEntry("samples/known.hwp", "hwpx", 3518, "known")]
            fake = FakeRhwp((10, 11))
            report = prt.run_harness(
                repo=repo,
                docs=[known],
                routes=["hwpx"],
                catalog=catalog,
                rhwp=Path("rhwp-fake"),
                strict=True,
                source="test",
                runner=fake,
            )
            self.assertEqual(report.rows[0].verdict, "EXPECTED_FAIL")
            self.assertEqual(prt.exit_code(report), 0)

            newbie = _touch(repo / "samples" / "new.hwp")
            report2 = prt.run_harness(
                repo=repo,
                docs=[newbie],
                routes=["hwpx"],
                catalog=catalog,
                rhwp=Path("rhwp-fake"),
                strict=True,
                source="test",
                runner=FakeRhwp((1, 2)),
            )
            self.assertEqual(report2.rows[0].verdict, "MISMATCH")
            self.assertEqual(prt.exit_code(report2), 1)

    def test_both_routes_emit_two_rows(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "x.hwpx")
            fake = FakeRhwp((3, 3))
            report = prt.run_harness(
                repo=repo,
                docs=[doc],
                routes=["hwpx", "hwp"],
                catalog=[],
                rhwp=Path("rhwp-fake"),
                strict=False,
                source="test",
                runner=fake,
            )
            self.assertEqual([r.route for r in report.rows], ["hwpx", "hwp"])
            self.assertEqual(len(fake.calls), 2)
            self.assertEqual(fake.calls[0][1], "export-hwpx")
            self.assertEqual(fake.calls[1][1], "convert")

    def test_missing_rhwp_is_error_row_default_exit_zero(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "x.hwp")
            report = prt.run_harness(
                repo=repo,
                docs=[doc],
                routes=["hwpx"],
                catalog=[],
                rhwp=None,
                strict=False,
                source="test",
            )
            self.assertEqual(report.rows[0].verdict, "ERROR")
            self.assertIn("rhwp", report.rows[0].note)
            self.assertEqual(prt.exit_code(report), 0)


class CliTests(unittest.TestCase):
    def test_file_json_default_zero_on_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            doc = _touch(repo / "samples" / "z.hwp")
            cat = repo / "cat.json"
            cat.write_text(json.dumps({"entries": []}), encoding="utf-8")
            fake = FakeRhwp((1, 2))
            with patch.object(prt, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(prt.subprocess, "run", fake):
                    with patch.object(sys, "stdout"):
                        rc = prt.main(
                            [
                                "--repo",
                                str(repo),
                                "--catalog",
                                str(cat),
                                "--file",
                                "samples/z.hwp",
                                "--json",
                            ]
                        )
            self.assertEqual(rc, 0)
            self.assertEqual(len(fake.calls), 1)

    def test_strict_cli_nonzero_on_new_violation(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "z.hwp")
            cat = repo / "cat.json"
            cat.write_text(json.dumps({"entries": []}), encoding="utf-8")
            fake = FakeRhwp((1, 2))
            buf: list[str] = []

            def capture(text: str) -> None:
                buf.append(text)

            with patch.object(prt, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(prt.subprocess, "run", fake):
                    with patch.object(sys, "stdout") as stdout:
                        stdout.write = capture
                        rc = prt.main(
                            [
                                "--repo",
                                str(repo),
                                "--catalog",
                                str(cat),
                                "--file",
                                "samples/z.hwp",
                                "--strict",
                            ]
                        )
            self.assertEqual(rc, 1)
            joined = "".join(buf)
            self.assertIn("MISMATCH", joined)
            self.assertIn("harness.py --file", joined)

    def test_limit_does_not_drop_catalog_section(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "a.hwp")
            _touch(repo / "samples" / "b.hwp")
            _touch(repo / "samples" / "known.hwp")
            cat = repo / "cat.json"
            cat.write_text(
                json.dumps(
                    {
                        "entries": [
                            {"doc": "samples/known.hwp", "route": "hwpx", "issue": 3518}
                        ]
                    }
                ),
                encoding="utf-8",
            )
            fake = FakeRhwp((1, 1))
            with patch.object(prt, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(prt.subprocess, "run", fake):
                    with patch.object(sys, "stdout"):
                        rc = prt.main(
                            [
                                "--repo",
                                str(repo),
                                "--catalog",
                                str(cat),
                                "--docs",
                                "samples",
                                "--limit",
                                "1",
                                "--json",
                            ]
                        )
            self.assertEqual(rc, 0)
            self.assertEqual(len(fake.calls), 1)

    def test_human_report_lists_repro_and_catalog(self) -> None:
        row = prt.Row(
            doc="samples/foo.hwp",
            route="hwpx",
            pages_before=4,
            pages_after=5,
            equal=False,
            verdict="MISMATCH",
            note="before=4 after=5",
            repro=prt.repro_command("samples/foo.hwp", "hwpx"),
        )
        report = prt.Report(
            rows=[row],
            catalog=[
                prt.CatalogStatus(
                    "samples/known.hwp", "hwpx", 3518, "known", "held", ""
                )
            ],
        )
        text = prt.format_human(report)
        self.assertIn("# REPRO python tools/page_roundtrip/harness.py --file samples/foo.hwp --route hwpx", text)
        self.assertIn("# CATALOG held\tsamples/known.hwp\thwpx\t#3518", text)
        self.assertIn("판정은 데이터다", text)

    def test_transcript_dir_writes_jsonl(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            repo = Path(td)
            _touch(repo / "samples" / "z.hwp")
            cat = repo / "cat.json"
            cat.write_text(json.dumps({"entries": []}), encoding="utf-8")
            out = repo / "tr"
            fake = FakeRhwp((215, 215))
            with patch.object(prt, "find_rhwp", return_value=Path("rhwp-fake")):
                with patch.object(prt.subprocess, "run", fake):
                    with patch.object(sys, "stdout"):
                        rc = prt.main(
                            [
                                "--repo",
                                str(repo),
                                "--catalog",
                                str(cat),
                                "--file",
                                "samples/z.hwp",
                                "--transcript-dir",
                                str(out),
                            ]
                        )
            self.assertEqual(rc, 0)
            files = list(out.glob("*.jsonl"))
            self.assertEqual(len(files), 1)
            text = files[0].read_text(encoding="utf-8")
            self.assertIn("215", text)
            self.assertIn("MATCH", text)


if __name__ == "__main__":
    unittest.main()
