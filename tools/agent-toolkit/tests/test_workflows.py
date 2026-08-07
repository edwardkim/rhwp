#!/usr/bin/env python3
"""agent-toolkit 워크플로 회귀 테스트 — 실제 rhwp 바이너리·samples fixture 로 실행.

    RHWP_BIN=<rhwp 경로> python tools/agent-toolkit/tests/test_workflows.py

워크플로마다 성공 케이스(산출물 생성 + 테스트 자체의 독립 재독)와
실패 케이스(없는 파일 → exit 2, notFound/표 없음/부분 실패 → exit 1,
동일성 다름 → exit 3)를 검증한다. 표준 라이브러리만 사용하며,
테스트 사이 의존이 없도록 공용 fixture 는 모듈 셋업에서 한 번 만든다.
"""

import csv
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TESTS_DIR = Path(__file__).resolve().parent
TOOLKIT_DIR = TESTS_DIR.parent
WORKFLOWS = TOOLKIT_DIR / "workflows"
REPO_ROOT = TOOLKIT_DIR.parent.parent
SAMPLES = REPO_ROOT / "samples"

FIELD_DOC = SAMPLES / "field-01.hwp"  # 누름틀 11개, 표 0개, 3쪽
TABLE_DOC = SAMPLES / "21868765_별표2_보건소_분장사무.hwp"  # 표 1개(147x3), "보건" 16건

RHWP_BIN = None   # setUpModule 에서 해석
WORKDIR = None
ARCHIVE_DIR = None   # 정상 문서 2건
BROKEN_DIR = None    # 정상 1건 + 깨진 hwp 1건
MODIFIED_DOC = None  # rhwp 로 직접 만든 변조본 (distribution_verify 의 "다름" 상대)


def resolve_rhwp_for_tests():
    candidate = os.environ.get("RHWP_BIN") or "rhwp"
    if os.path.sep in candidate or (os.altsep and os.altsep in candidate):
        return candidate if Path(candidate).is_file() else None
    return shutil.which(candidate)


def setUpModule():
    global RHWP_BIN, WORKDIR, ARCHIVE_DIR, BROKEN_DIR, MODIFIED_DOC
    RHWP_BIN = resolve_rhwp_for_tests()
    if not RHWP_BIN:
        print(
            "오류: rhwp 바이너리를 찾을 수 없습니다 — RHWP_BIN=<경로> 로 지정하세요",
            file=sys.stderr,
        )
        sys.exit(2)
    for fixture in (FIELD_DOC, TABLE_DOC):
        if not fixture.is_file():
            print(f"오류: fixture 가 없습니다: {fixture}", file=sys.stderr)
            sys.exit(2)
    WORKDIR = Path(tempfile.mkdtemp(prefix="agent_toolkit_test_"))

    ARCHIVE_DIR = WORKDIR / "archive"
    ARCHIVE_DIR.mkdir()
    shutil.copy(TABLE_DOC, ARCHIVE_DIR / TABLE_DOC.name)
    shutil.copy(FIELD_DOC, ARCHIVE_DIR / FIELD_DOC.name)

    BROKEN_DIR = WORKDIR / "archive_broken"
    BROKEN_DIR.mkdir()
    shutil.copy(TABLE_DOC, BROKEN_DIR / TABLE_DOC.name)
    (BROKEN_DIR / "corrupt.hwp").write_bytes(b"garbage-not-hwp")

    # 변조본은 워크플로가 아니라 rhwp 를 직접 불러 만든다 (테스트 독립성)
    MODIFIED_DOC = WORKDIR / "modified.hwp"
    data = WORKDIR / "modify.json"
    data.write_text(
        json.dumps({"회사명": "다른회사"}, ensure_ascii=False), encoding="utf-8"
    )
    result = subprocess.run(
        [
            RHWP_BIN, "edit", "fill-fields", str(FIELD_DOC),
            "--data", "@" + str(data), "-o", str(MODIFIED_DOC), "--json",
        ],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
        check=False,
    )
    if result.returncode != 0 or not MODIFIED_DOC.is_file():
        print(f"오류: 변조본 fixture 생성 실패: {result.stderr}", file=sys.stderr)
        sys.exit(2)


def tearDownModule():
    if WORKDIR and WORKDIR.exists():
        shutil.rmtree(WORKDIR, ignore_errors=True)


def run_workflow(script, *args):
    """워크플로 스크립트 실행 → CompletedProcess (UTF-8 캡처)."""
    env = dict(os.environ)
    env["RHWP_BIN"] = RHWP_BIN
    env["PYTHONIOENCODING"] = "utf-8"
    return subprocess.run(
        [sys.executable, str(WORKFLOWS / script), *[str(a) for a in args]],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
        cwd=str(REPO_ROOT),
        check=False,
    )


def run_rhwp_json(*args):
    """테스트 자체의 독립 재독용 rhwp 호출 (워크플로를 거치지 않는다)."""
    result = subprocess.run(
        [RHWP_BIN, *args, "--json"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    return json.loads(result.stdout), result.returncode


def write_json(path, obj):
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(obj, fh, ensure_ascii=False)
    return path


class TestFormFilling(unittest.TestCase):
    def test_success_creates_output_and_verifies(self):
        values = write_json(
            WORKDIR / "values.json", {"회사명": "한국수자원공사", "작성자": "홍길동"}
        )
        out = WORKDIR / "filled.hwp"
        r = run_workflow("form_filling.py", FIELD_DOC, values, "-o", out, "--json")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue(out.is_file(), "성공인데 산출물이 없다")
        summary = json.loads(r.stdout)
        self.assertEqual(summary["filledCount"], 2)
        self.assertTrue(summary["rereadVerified"])
        # 워크플로를 믿지 않는 독립 재독 — rhwp fields 로 직접 값 확인
        reread, code = run_rhwp_json("fields", str(out))
        self.assertEqual(code, 0)
        by_name = {f["name"]: f["value"] for f in reread["fields"] if f["value"]}
        self.assertEqual(by_name.get("회사명"), "한국수자원공사")
        self.assertEqual(by_name.get("작성자"), "홍길동")

    def test_occurrence_index_targets_nth_field(self):
        values = write_json(WORKDIR / "occ.json", {"목차1[1]": "재독항목"})
        out = WORKDIR / "occ.hwp"
        r = run_workflow("form_filling.py", FIELD_DOC, values, "-o", out)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertTrue(out.is_file())

    def test_not_found_fails_and_removes_output(self):
        values = write_json(WORKDIR / "nf.json", {"없는칸": "x"})
        out = WORKDIR / "nf.hwp"
        r = run_workflow("form_filling.py", FIELD_DOC, values, "-o", out)
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("notFound", r.stderr)
        self.assertFalse(out.exists(), "실패했는데 산출물이 남았다")

    def test_existing_output_is_preserved(self):
        values = write_json(WORKDIR / "existing_values.json", {"회사명": "새 값"})
        out = WORKDIR / "existing.hwp"
        original = b"must-not-overwrite"
        out.write_bytes(original)
        r = run_workflow("form_filling.py", FIELD_DOC, values, "-o", out)
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertEqual(out.read_bytes(), original)

    def test_ambiguous_name_fails(self):
        values = write_json(WORKDIR / "amb.json", {"목차1": "모호"})
        out = WORKDIR / "amb.hwp"
        r = run_workflow("form_filling.py", FIELD_DOC, values, "-o", out)
        self.assertEqual(r.returncode, 1)
        self.assertIn("ambiguous", r.stderr)
        self.assertFalse(out.exists())

    def test_missing_template_is_usage_error(self):
        values = write_json(WORKDIR / "v.json", {"회사명": "x"})
        r = run_workflow(
            "form_filling.py", SAMPLES / "no-such.hwp", values, "-o", WORKDIR / "x.hwp"
        )
        self.assertEqual(r.returncode, 2)

    def test_malformed_values_json_is_usage_error(self):
        bad = WORKDIR / "bad.json"
        bad.write_text("not json", encoding="utf-8")
        r = run_workflow("form_filling.py", FIELD_DOC, bad, "-o", WORKDIR / "x.hwp")
        self.assertEqual(r.returncode, 2)


class TestTableHarvest(unittest.TestCase):
    def test_success_writes_csv_matching_grid(self):
        out_dir = WORKDIR / "tables"
        r = run_workflow("table_harvest.py", TABLE_DOC, "-o", out_dir, "--bom", "--json")
        self.assertEqual(r.returncode, 0, r.stderr)
        summary = json.loads(r.stdout)
        self.assertEqual(summary["tableCount"], 1)
        csv_path = out_dir / "table0.csv"
        self.assertTrue(csv_path.is_file())
        # 워크플로를 믿지 않는 독립 재독 — CSV 격자를 직접 센다
        with io.open(csv_path, encoding="utf-8-sig", newline="") as fh:
            rows = list(csv.reader(fh))
        self.assertEqual(len(rows), 147)
        self.assertTrue(all(len(row) == 3 for row in rows))

    def test_document_without_tables_fails(self):
        out_dir = WORKDIR / "tables_zero"
        r = run_workflow("table_harvest.py", FIELD_DOC, "-o", out_dir)
        self.assertEqual(r.returncode, 1)
        self.assertIn("tableCount=0", r.stderr)
        self.assertFalse(list(out_dir.glob("*.csv")), "실패인데 CSV 가 남았다")

    def test_bad_table_index_fails(self):
        r = run_workflow(
            "table_harvest.py", TABLE_DOC, "-o", WORKDIR / "t999", "--table", "999"
        )
        self.assertEqual(r.returncode, 1)

    def test_existing_csv_is_preserved(self):
        out_dir = WORKDIR / "tables_existing"
        out_dir.mkdir()
        csv_path = out_dir / "table0.csv"
        original = b"must-not-overwrite\n"
        csv_path.write_bytes(original)
        r = run_workflow("table_harvest.py", TABLE_DOC, "-o", out_dir)
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertEqual(csv_path.read_bytes(), original)

    def test_missing_document_is_usage_error(self):
        r = run_workflow(
            "table_harvest.py", SAMPLES / "no-such.hwp", "-o", WORKDIR / "tx"
        )
        self.assertEqual(r.returncode, 2)


class TestArchiveSearch(unittest.TestCase):
    def test_success_reports_file_page_coordinates(self):
        report = WORKDIR / "search_report.json"
        r = run_workflow("archive_search.py", ARCHIVE_DIR, "--query", "보건", "-o", report)
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads(report.read_text(encoding="utf-8"))
        self.assertEqual(data["scannedCount"], 2)
        self.assertEqual(data["matchedFileCount"], 1)
        self.assertEqual(data["totalMatchCount"], 16)
        hit = data["files"][0]
        self.assertIn(TABLE_DOC.name, hit["source"])
        self.assertTrue(all("page" in m and "charOffset" in m for m in hit["matches"]))

    def test_zero_matches_is_success(self):
        r = run_workflow(
            "archive_search.py", ARCHIVE_DIR, "--query", "이런문구는없다없다", "--json"
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads(r.stdout)
        self.assertEqual(data["matchedFileCount"], 0)

    def test_partial_failure_exits_1_but_keeps_results(self):
        report = WORKDIR / "search_report_partial.json"
        r = run_workflow("archive_search.py", BROKEN_DIR, "--query", "보건", "-o", report)
        self.assertEqual(r.returncode, 1)
        data = json.loads(report.read_text(encoding="utf-8"))
        self.assertEqual(len(data["errors"]), 1)
        self.assertIn("corrupt.hwp", data["errors"][0]["source"])
        self.assertEqual(data["matchedFileCount"], 1, "성공분 결과가 유실됐다")
        self.assertEqual(data["exit"], 1)
        self.assertEqual(data["batch"]["exitCode"], 1)

    def test_existing_report_is_preserved(self):
        report = WORKDIR / "search_existing.json"
        original = b"must-not-overwrite\n"
        report.write_bytes(original)
        r = run_workflow("archive_search.py", ARCHIVE_DIR, "--query", "보건", "-o", report)
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertEqual(report.read_bytes(), original)

    def test_missing_directory_is_usage_error(self):
        r = run_workflow("archive_search.py", WORKDIR / "no-such-dir", "--query", "보건")
        self.assertEqual(r.returncode, 2)


class TestBulkSweep(unittest.TestCase):
    def test_success_writes_ndjson_and_summary(self):
        out_dir = WORKDIR / "sweep_ok"
        r = run_workflow("bulk_sweep.py", ARCHIVE_DIR, "-o", out_dir, "--json")
        self.assertEqual(r.returncode, 0, r.stderr)
        for name in ("info.ndjson", "export_text.ndjson", "export_structure.ndjson",
                     "summary.json"):
            self.assertTrue((out_dir / name).is_file(), f"{name} 이 없다")
        # 독립 재독 — info.ndjson 레코드 수와 pageCount 존재 확인
        records = [
            json.loads(line)
            for line in (out_dir / "info.ndjson").read_text(encoding="utf-8").splitlines()
            if line.strip()
        ]
        self.assertEqual(len(records), 2)
        self.assertTrue(all(isinstance(rec.get("pageCount"), int) for rec in records))

    def test_min_pages_filter_narrows_targets(self):
        out_dir = WORKDIR / "sweep_filtered"
        r = run_workflow(
            "bulk_sweep.py", ARCHIVE_DIR, "-o", out_dir, "--min-pages", "4", "--json"
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        summary = json.loads(r.stdout)
        # 분장사무 4쪽만 통과, field-01 은 3쪽이라 걸러진다
        self.assertEqual(summary["inputCount"], 2)
        self.assertEqual(summary["targetCount"], 1)

    def test_partial_failure_exits_1_and_lists_failed(self):
        out_dir = WORKDIR / "sweep_partial"
        r = run_workflow("bulk_sweep.py", BROKEN_DIR, "-o", out_dir)
        self.assertEqual(r.returncode, 1)
        summary = json.loads((out_dir / "summary.json").read_text(encoding="utf-8"))
        self.assertTrue(any("corrupt.hwp" in s for s in summary["failedSources"]))
        self.assertTrue((out_dir / "info.ndjson").is_file(), "성공분 NDJSON 이 없다")

    def test_unknown_task_is_usage_error(self):
        r = run_workflow(
            "bulk_sweep.py", ARCHIVE_DIR, "-o", WORKDIR / "sx", "--tasks", "render-all"
        )
        self.assertEqual(r.returncode, 2)

    def test_existing_summary_input_is_preserved(self):
        out_dir = WORKDIR / "sweep_existing"
        out_dir.mkdir()
        info_path = out_dir / "info.ndjson"
        original = b"must-not-overwrite\n"
        info_path.write_bytes(original)
        r = run_workflow("bulk_sweep.py", ARCHIVE_DIR, "-o", out_dir)
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertEqual(info_path.read_bytes(), original)

    @unittest.skipUnless(
        os.name == "posix" and Path("/bin/false").is_file(),
        "POSIX false executable is required",
    )
    def test_batch_process_failure_exits_1(self):
        out_dir = WORKDIR / "sweep_batch_process_failure"
        r = run_workflow(
            "bulk_sweep.py",
            FIELD_DOC,
            "-o",
            out_dir,
            "--rhwp-bin",
            "/bin/false",
            "--json",
        )
        self.assertEqual(r.returncode, 1, r.stderr)
        summary = json.loads(r.stdout)
        self.assertEqual(summary["exit"], 1)
        self.assertEqual(summary["batchFailures"], [{"task": "info", "exitCode": 1}])


class TestDistributionVerify(unittest.TestCase):
    def test_identical_document_exits_0(self):
        report = WORKDIR / "dv_same.json"
        r = run_workflow(
            "distribution_verify.py", FIELD_DOC, FIELD_DOC, "-o", report, "--json"
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads(report.read_text(encoding="utf-8"))
        self.assertEqual(data["verdict"], "identical")
        self.assertEqual(data["geometry"]["maxDisp"], 0.0)

    def test_modified_document_exits_3(self):
        r = run_workflow("distribution_verify.py", FIELD_DOC, MODIFIED_DOC, "--json")
        self.assertEqual(r.returncode, 3, r.stderr)
        data = json.loads(r.stdout)
        self.assertEqual(data["verdict"], "different")

    def test_missing_file_is_usage_error(self):
        r = run_workflow("distribution_verify.py", FIELD_DOC, SAMPLES / "no-such.hwp")
        self.assertEqual(r.returncode, 2)

    def test_existing_report_is_preserved(self):
        report = WORKDIR / "distribution_existing.json"
        original = b"must-not-overwrite\n"
        report.write_bytes(original)
        r = run_workflow(
            "distribution_verify.py", FIELD_DOC, FIELD_DOC, "-o", report
        )
        self.assertEqual(r.returncode, 2, r.stderr)
        self.assertEqual(report.read_bytes(), original)


if __name__ == "__main__":
    unittest.main(verbosity=2)
