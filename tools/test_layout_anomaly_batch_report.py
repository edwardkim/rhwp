#!/usr/bin/env python3
"""layout_anomaly_batch_report.py 단위 시험 — samples/ 전수 불필요.

실행 (저장소 루트):
    python tools/test_layout_anomaly_batch_report.py
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

MODULE_PATH = Path(__file__).resolve().parent / "layout_anomaly_batch_report.py"
SPEC = importlib.util.spec_from_file_location("layout_anomaly_batch_report", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
br = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = br
SPEC.loader.exec_module(br)


def _touch(path: Path, text: str = "x") -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


def _envelope(**kwargs: object) -> dict:
    payload = {
        "schemaVersion": "1.0",
        "source": "samples/a.hwp",
        "pageCount": 2,
        "overflowCount": 0,
        "overlapCount": 0,
        "emptyPageCount": 0,
        "hasSignal": False,
        "pages": [],
        "untrustedContent": True,
        "untrustedFields": ["source"],
    }
    payload.update(kwargs)
    return payload


class WalkDocsTests(unittest.TestCase):
    def test_sorts_posix_and_skips_non_docs(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            samples = root / "samples"
            _touch(samples / "z-last.hwpx")
            _touch(samples / "a-first.hwp")
            _touch(samples / "mid" / "nested.hwp")
            _touch(samples / "readme.txt")
            _touch(samples / ".hidden" / "skip.hwp")
            _touch(samples / "note.pdf")
            files = br.walk_docs(samples, root)
            rels = [br.posix_rel(p, root) for p in files]
            self.assertEqual(
                rels,
                [
                    "samples/a-first.hwp",
                    "samples/mid/nested.hwp",
                    "samples/z-last.hwpx",
                ],
            )


class ParseEnvelopeTests(unittest.TestCase):
    def test_required_counts(self) -> None:
        parsed = br.parse_envelope(
            _envelope(overflowCount=3, overlapCount=1, emptyPageCount=2, hasSignal=True)
        )
        self.assertEqual(parsed["overflow"], 3)
        self.assertEqual(parsed["overlap"], 1)
        self.assertEqual(parsed["empty_page"], 2)
        self.assertTrue(parsed["has_signal"])
        self.assertIsNone(parsed["off_canvas"])
        self.assertIsNone(parsed["text_overlap"])
        self.assertFalse(parsed["has_off_canvas_field"])
        self.assertFalse(parsed["has_text_overlap_field"])

    def test_optional_off_canvas_and_text_overlap(self) -> None:
        parsed = br.parse_envelope(
            _envelope(offCanvasCount=4, textOverlapCount=5, overflowCount=0)
        )
        self.assertEqual(parsed["off_canvas"], 4)
        self.assertEqual(parsed["text_overlap"], 5)
        self.assertTrue(parsed["has_off_canvas_field"])
        self.assertTrue(parsed["has_text_overlap_field"])

    def test_missing_required_raises(self) -> None:
        with self.assertRaises(ValueError):
            br.parse_envelope({"pageCount": 1})

    def test_extract_json_ignores_log_prefix(self) -> None:
        blob = "warn: skip\n" + json.dumps(_envelope(overflowCount=1)) + "\n"
        payload = br.extract_json_object(blob)
        self.assertEqual(payload["overflowCount"], 1)


class FakeRhwp:
    def __init__(self, mapping: dict[str, dict], rc: int = 0) -> None:
        self.mapping = mapping
        self.rc = rc
        self.calls: list[list[str]] = []

    def __call__(self, cmd, **kwargs: object) -> subprocess.CompletedProcess[str]:
        self.calls.append(list(cmd))
        timeout = kwargs.get("timeout")
        if cmd[1] == "--version":
            return subprocess.CompletedProcess(list(cmd), 0, "rhwp v0-test", "")
        if cmd[1] == "layout-anomaly" and "--json" not in cmd:
            return subprocess.CompletedProcess(
                list(cmd), 2, "", "사용법: rhwp layout-anomaly <파일.hwp|파일.hwpx>"
            )
        name = Path(cmd[2]).name
        if name == "slow.hwp" and timeout is not None and float(timeout) < 10:
            raise subprocess.TimeoutExpired(list(cmd), timeout)
        if name == "broken.hwp":
            return subprocess.CompletedProcess(list(cmd), 1, "", "오류: 문서 로드 실패")
        payload = self.mapping.get(name) or _envelope()
        return subprocess.CompletedProcess(
            list(cmd), self.rc, json.dumps(payload), ""
        )


class RunOneTests(unittest.TestCase):
    def test_success_anomaly(self) -> None:
        fake = FakeRhwp(
            {
                "hit.hwp": _envelope(
                    overflowCount=2, overlapCount=0, emptyPageCount=1, hasSignal=True
                )
            }
        )
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            path = _touch(repo / "samples" / "hit.hwp")
            rhwp = _touch(repo / "rhwp.exe")
            with patch.object(br, "run_cmd", fake):
                row = br.run_one(rhwp, path, repo, 30.0)
        self.assertEqual(row.status, "ANOMALY")
        self.assertEqual(row.overflow, 2)
        self.assertEqual(row.empty_page, 1)
        self.assertTrue(row.has_signal)

    def test_load_error(self) -> None:
        fake = FakeRhwp({})
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            path = _touch(repo / "samples" / "broken.hwp")
            rhwp = _touch(repo / "rhwp.exe")
            with patch.object(br, "run_cmd", fake):
                row = br.run_one(rhwp, path, repo, 30.0)
        self.assertEqual(row.status, "ERROR")
        self.assertIn("로드", row.error)

    def test_oserror_retries_then_succeeds(self) -> None:
        class Flaky:
            def __init__(self) -> None:
                self.n = 0

            def __call__(self, cmd, **kwargs: object) -> subprocess.CompletedProcess[str]:
                self.n += 1
                if cmd[1] == "layout-anomaly" and "--json" in cmd and self.n < 3:
                    raise OSError(2, "지정된 파일을 찾을 수 없습니다")
                return subprocess.CompletedProcess(
                    list(cmd), 0, json.dumps(_envelope(overflowCount=1, hasSignal=True)), ""
                )

        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            path = _touch(repo / "samples" / "hit.hwp")
            rhwp = _touch(repo / "rhwp.exe")
            with patch.object(br, "run_cmd", Flaky()):
                row = br.run_one(rhwp, path, repo, 30.0)
        self.assertEqual(row.status, "ANOMALY")
        self.assertEqual(row.overflow, 1)

    def test_timeout(self) -> None:
        fake = FakeRhwp({})
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            path = _touch(repo / "samples" / "slow.hwp")
            rhwp = _touch(repo / "rhwp.exe")
            with patch.object(br, "run_cmd", fake):
                row = br.run_one(rhwp, path, repo, 1.0)
        self.assertEqual(row.status, "TIMEOUT")


class ReportTests(unittest.TestCase):
    def test_summary_and_tsv_and_optional_null(self) -> None:
        rows = [
            br.FileRow("samples/a.hwp", "ANOMALY", 3, 5, 1, 0, None, None, True, 10),
            br.FileRow("samples/b.hwp", "CLEAN", 1, 0, 0, 0, None, None, False, 4),
            br.FileRow("samples/c.hwp", "ANOMALY", 8, 2, 4, 1, None, None, True, 20),
            br.FileRow("samples/d.hwp", "ERROR", 0, 0, 0, 0, None, None, False, 1, "boom"),
        ]
        report = br.Report(
            rows=rows,
            binary_path="target/release/rhwp.exe",
            binary_version="rhwp v0-test",
            supports_batch=False,
            supports_off_canvas=False,
            supports_text_overlap=False,
            git_commit="deadbeef",
            git_branch="feat/m02-8-anomaly-report",
            root="samples",
            file_count=4,
            limit=None,
            timeout_sec=180.0,
            jobs=1,
            top_n=2,
            started_at="2026-08-18T00:00:00Z",
            finished_at="2026-08-18T00:01:00Z",
            notes=["--batch 없음"],
        )
        summary = report.summary()
        self.assertEqual(summary["scanned"], 4)
        self.assertEqual(summary["anomaly"], 2)
        self.assertEqual(summary["error"], 1)
        self.assertEqual(summary["overflowCount"], 7)
        self.assertEqual(summary["overlapCount"], 5)
        self.assertEqual(summary["emptyPageCount"], 1)
        self.assertIsNone(summary["offCanvasCount"])
        self.assertIsNone(summary["textOverlapCount"])
        top = report.top_offenders()
        self.assertEqual(top["overflow"][0]["path"], "samples/a.hwp")
        self.assertEqual(top["overlap"][0]["path"], "samples/c.hwp")
        tsv = br.format_tsv(rows)
        self.assertIn("path\tstatus\tpage_count\toverflow", tsv)
        self.assertIn("samples/a.hwp\tANOMALY\t3\t5\t1\t0\t\t\t1\t10\t", tsv)
        md = br.format_markdown(report)
        self.assertIn("미지원 (devel / #5389 미병합)", md)
        self.assertIn("미지원 (devel / #5379 미병합)", md)
        payload = report.to_json()
        self.assertEqual(payload["claimId"], "M02-8")
        self.assertEqual(payload["cliContract"], br.CLI_CONTRACT)
        self.assertFalse(payload["binary"]["supportsBatch"])

    def test_optional_counts_when_supported(self) -> None:
        rows = [
            br.FileRow("samples/a.hwp", "ANOMALY", 1, 0, 0, 0, 3, 2, True, 5),
        ]
        report = br.Report(
            rows=rows,
            binary_path="rhwp",
            binary_version="x",
            supports_batch=False,
            supports_off_canvas=True,
            supports_text_overlap=True,
            git_commit="x",
            git_branch="x",
            root="samples",
            file_count=1,
            limit=None,
            timeout_sec=1.0,
            jobs=1,
            top_n=5,
            started_at="t",
            finished_at="t",
        )
        summary = report.summary()
        self.assertEqual(summary["offCanvasCount"], 3)
        self.assertEqual(summary["textOverlapCount"], 2)
        self.assertEqual(report.top_offenders()["offCanvas"][0]["score"], 3)

    def test_resume_skips_clean_and_anomaly(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            _touch(repo / "Cargo.toml")
            _touch(repo / "samples" / "a.hwp")
            _touch(repo / "samples" / "b.hwp")
            prev = repo / "prev.json"
            prev.write_text(
                json.dumps(
                    {
                        "rows": [
                            {
                                "path": "samples/a.hwp",
                                "status": "CLEAN",
                                "pageCount": 1,
                                "overflow": 0,
                                "overlap": 0,
                                "emptyPage": 0,
                                "hasSignal": False,
                                "elapsedMs": 3,
                            },
                            {
                                "path": "samples/b.hwp",
                                "status": "ERROR",
                                "error": "boom",
                            },
                        ]
                    }
                ),
                encoding="utf-8",
            )
            out = repo / "out.json"
            rc = br.main(
                [
                    "--repo-root",
                    str(repo),
                    "--root",
                    str(repo / "samples"),
                    "--resume-json",
                    str(prev),
                    "--collect-only",
                    "--json-out",
                    str(out),
                    "--quiet",
                ]
            )
            self.assertEqual(rc, 0)
            payload = json.loads(out.read_text(encoding="utf-8"))
            by_path = {row["path"]: row for row in payload["rows"]}
            self.assertEqual(by_path["samples/a.hwp"]["status"], "CLEAN")
            self.assertEqual(by_path["samples/b.hwp"]["status"], "SKIP")
            self.assertTrue(any("resume" in n for n in payload["notes"]))

    def test_collect_only_cli(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            _touch(repo / "Cargo.toml")
            _touch(repo / "samples" / "a.hwp")
            _touch(repo / "samples" / "b.hwpx")
            json_out = repo / "out.json"
            rc = br.main(
                [
                    "--repo-root",
                    str(repo),
                    "--root",
                    str(repo / "samples"),
                    "--collect-only",
                    "--json-out",
                    str(json_out),
                    "--quiet",
                ]
            )
            self.assertEqual(rc, 0)
            payload = json.loads(json_out.read_text(encoding="utf-8"))
            self.assertEqual(payload["summary"]["scanned"], 2)
            self.assertEqual(payload["rows"][0]["status"], "SKIP")

    def test_skip_does_not_claim_optional_unsupported(self) -> None:
        rows = [br.FileRow("samples/a.hwp", "SKIP", error="collect-only")]
        self.assertIsNone(br.infer_optional_support(rows, "off_canvas"))
        self.assertIsNone(br.infer_optional_support(rows, "text_overlap"))

    def test_strict_errors_exit_1(self) -> None:
        report = br.Report(
            rows=[br.FileRow("samples/a.hwp", "ERROR", error="x")],
            binary_path="rhwp",
            binary_version="x",
            supports_batch=False,
            supports_off_canvas=False,
            supports_text_overlap=False,
            git_commit="x",
            git_branch="x",
            root="samples",
            file_count=1,
            limit=None,
            timeout_sec=1.0,
            jobs=1,
            top_n=1,
            started_at="t",
            finished_at="t",
            strict=True,
        )
        self.assertEqual(br.exit_code(report), 1)


if __name__ == "__main__":
    unittest.main()
