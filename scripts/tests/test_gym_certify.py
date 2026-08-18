"""[certify] gym 능력 인증서 계약 — 벤치마크 지문 결정성 + 위조 탐지.

핵심: 인증서는 '재현 = 증명'이다. 벤치마크 지문(전 pack 정의 sha256)이 결정적이고,
재현 core(지문·바이너리 신원·정확도·커버리지·축)가 인증 시점과 하나라도 다르면
위조로 판정한다. 바이너리 없이 순수 로직만 시험한다(재현은 _run_report 를 목킹).

예외 칸(#5275): 없는 스코어카드, 깨진 JSON, 미가용 pack 은 스택이 아니라
kind 로 남긴다. 새 CLI 플래그는 없다. 예전 성공 칸의 문구는 그대로다.
"""

from __future__ import annotations

import copy
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "certify.py"
DOCS = REPO_ROOT / "gym" / "docs" / "certify_report.md"


def load():
    spec = importlib.util.spec_from_file_location("gym_certify", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


FIXED_REPORT = {
    "kind": "gymCapabilityReport",
    "runner": {"rhwpVersion": "0.8.4", "rhwpCommit": "deadbeef" * 5,
               "capabilitiesSha256": "c" * 64},
    "accuracy": {"score": 224, "max": 224, "percent": 100},
    "coverage": {"percent": 82, "covered": 42, "agentFacingTotal": 51},
    "axisProfile": [{"axis": "편집", "score": 37, "max": 37, "percent": 100}],
    "packsScored": 13,
}


def _write(path, payload):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        if isinstance(payload, str):
            fh.write(payload)
        else:
            json.dump(payload, fh, ensure_ascii=False)


class CertifyTests(unittest.TestCase):
    def test_fingerprint_is_deterministic_and_change_sensitive(self):
        cov = load()
        with tempfile.TemporaryDirectory() as d:
            pd = os.path.join(d, "packs", "p1")
            os.makedirs(os.path.join(pd, "tasks"))
            Path(pd, "pack.json").write_text('{"id":"p1"}', encoding="utf-8")
            Path(pd, "tasks", "A01.json").write_text('{"id":"A01"}', encoding="utf-8")
            fp1 = cov.benchmark_fingerprint(d)
            fp2 = cov.benchmark_fingerprint(d)
            self.assertEqual(fp1, fp2)                       # 결정적
            self.assertEqual(len(fp1), 64)                   # sha256 hex
            Path(pd, "tasks", "A01.json").write_text('{"id":"A01","x":1}', encoding="utf-8")
            self.assertNotEqual(fp1, cov.benchmark_fingerprint(d))  # 변조에 민감

    def test_fingerprint_covers_pack_assets_and_measurement_protocol(self):
        cov = load()
        with tempfile.TemporaryDirectory() as d:
            pd = os.path.join(d, "packs", "p1", "assets")
            os.makedirs(pd)
            tools = os.path.join(d, "tools")
            os.makedirs(tools)
            asset = os.path.join(pd, "input.csv")
            protocol = os.path.join(tools, "build_baseline.py")
            Path(asset).write_text("a,b\n1,2\n", encoding="utf-8")
            Path(protocol).write_text("print('v1')\n", encoding="utf-8")
            fp1 = cov.benchmark_fingerprint(d)
            Path(asset).write_text("a,b\n3,4\n", encoding="utf-8")
            self.assertNotEqual(fp1, cov.benchmark_fingerprint(d))
            fp2 = cov.benchmark_fingerprint(d)
            Path(protocol).write_text("print('v2')\n", encoding="utf-8")
            self.assertNotEqual(fp2, cov.benchmark_fingerprint(d))

    def test_reproducible_core_excludes_volatile_metadata(self):
        cov = load()
        core = cov.reproducible_core(FIXED_REPORT, "FP")
        self.assertEqual(core["benchmarkFingerprint"], "FP")
        self.assertEqual(core["capabilitiesSha256"], "c" * 64)
        self.assertEqual(core["accuracy"], FIXED_REPORT["accuracy"])
        # 변동 메타(git commit·agent)는 재현 core 에 없다.
        self.assertNotIn("rhwpCommit", core)
        self.assertNotIn("agent", core)

    def test_verify_passes_for_genuine_certificate(self):
        cov = load()
        cov.benchmark_fingerprint = lambda root: "FP"
        cov._run_report = lambda bin_path: copy.deepcopy(FIXED_REPORT)
        cert = {"kind": cov.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(FIXED_REPORT)}
        ok, diffs = cov.verify(cert, "anybin")
        self.assertTrue(ok, diffs)

    def test_verify_detects_accuracy_tamper(self):
        cov = load()
        cov.benchmark_fingerprint = lambda root: "FP"
        cov._run_report = lambda bin_path: copy.deepcopy(FIXED_REPORT)
        cert = {"kind": cov.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(FIXED_REPORT)}
        cert["report"]["accuracy"]["percent"] = 999  # 위조
        ok, diffs = cov.verify(cert, "anybin")
        self.assertFalse(ok)
        self.assertTrue(any("정확도" in d for d in diffs))

    def test_verify_detects_shrunk_benchmark(self):
        cov = load()
        cov.benchmark_fingerprint = lambda root: "REAL_FP"
        cov._run_report = lambda bin_path: copy.deepcopy(FIXED_REPORT)
        # 인증서가 다른(축소된) 벤치마크 지문을 주장 → 재현 지문과 불일치.
        cert = {"kind": cov.CERT_KIND, "benchmarkFingerprint": "FAKE_FP",
                "report": copy.deepcopy(FIXED_REPORT)}
        ok, diffs = cov.verify(cert, "anybin")
        self.assertFalse(ok)
        self.assertTrue(any("벤치마크 지문" in d for d in diffs))


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_exception_kinds_are_unique_and_documented(self):
        kinds = self.m.EXCEPTION_KINDS
        self.assertEqual(len(kinds), len(set(kinds)))
        for kind in kinds:
            self.assertIn(kind, self.m.EXCEPTION_KIND_HELP)
            self.assertTrue(self.m.EXCEPTION_KIND_HELP[kind])
            self.assertTrue(self.m.is_known_exception_kind(kind))
            self.assertTrue(self.m.describe_exception_kind(kind))

    def test_docs_backtick_every_exception_kind(self):
        text = DOCS.read_text(encoding="utf-8")
        for kind in self.m.EXCEPTION_KINDS:
            self.assertIn(f"`{kind}`", text, msg=kind)

    def test_docs_mentions_cert_and_report_kinds(self):
        text = DOCS.read_text(encoding="utf-8")
        self.assertIn("`gymCapabilityCertificate`", text)
        self.assertIn("`gymCapabilityReport`", text)

    def test_cli_flags_unchanged(self):
        names = self.m.cli_flag_names()
        self.assertEqual(tuple(sorted(names)), tuple(sorted(self.m.CERT_CLI_FLAGS)))

    def test_no_new_cli_flags(self):
        self.assertNotIn("--json", self.m.CERT_CLI_FLAGS)
        self.assertNotIn("--strict", self.m.CERT_CLI_FLAGS)
        self.assertNotIn("--limit", self.m.CERT_CLI_FLAGS)

    def test_reproducible_core_keys(self):
        for key in ("benchmarkFingerprint", "capabilitiesSha256", "accuracy",
                    "coverage", "axisProfile"):
            self.assertIn(key, self.m.REPRODUCIBLE_CORE_KEYS)

    def test_unknown_kind_not_known(self):
        self.assertFalse(self.m.is_known_exception_kind(""))
        self.assertFalse(self.m.is_known_exception_kind(None))
        self.assertEqual(
            self.m.describe_exception_kind("nope"),
            self.m.EXCEPTION_KIND_HELP["unexpected"],
        )


class FingerprintHelperTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_empty_root_is_empty_fingerprint(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertTrue(self.m.fingerprint_is_empty(d))
            self.assertEqual(self.m.fingerprint_entry_count(d), 0)
            self.assertEqual(self.m.fingerprint_rel_paths(d), [])
            self.assertEqual(len(self.m.benchmark_fingerprint(d)), 64)

    def test_skips_pyc_and_pycache(self):
        with tempfile.TemporaryDirectory() as d:
            tools = os.path.join(d, "tools")
            cache = os.path.join(tools, "__pycache__")
            os.makedirs(cache)
            Path(tools, "keep.py").write_text("print(1)\n", encoding="utf-8")
            Path(tools, "skip.pyc").write_bytes(b"\x00\x01")
            Path(cache, "keep.cpython-313.pyc").write_bytes(b"\x00")
            paths = self.m.fingerprint_rel_paths(d)
            self.assertEqual(paths, ["tools/keep.py"])

    def test_includes_score_report_certify_files(self):
        with tempfile.TemporaryDirectory() as d:
            for name in ("score.py", "report.py", "certify.py"):
                Path(d, name).write_text(f"# {name}\n", encoding="utf-8")
            paths = set(self.m.fingerprint_rel_paths(d))
            self.assertEqual(paths, {"score.py", "report.py", "certify.py"})

    def test_hash_same_bytes_different_relpath_differs(self):
        a = self.m.hash_fingerprint_entries([("a.json", b"x")])
        b = self.m.hash_fingerprint_entries([("b.json", b"x")])
        self.assertNotEqual(a, b)

    def test_should_skip_pyc_only(self):
        self.assertTrue(self.m.should_skip_fingerprint_name("x.pyc"))
        self.assertFalse(self.m.should_skip_fingerprint_name("x.py"))
        self.assertFalse(self.m.should_skip_fingerprint_name("pack.json"))


class LoadCertTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_missing_cert_file(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "no-cert.json")
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m.load_cert(path)
            self.assertEqual(ctx.exception.kind, "missing-cert")

    def test_empty_path_is_missing_cert(self):
        with self.assertRaises(self.m.CertifyError) as ctx:
            self.m.load_cert("")
        self.assertEqual(ctx.exception.kind, "missing-cert")

    def test_bad_json_cert(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "bad.json")
            _write(path, "{")
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m.load_cert(path)
            self.assertEqual(ctx.exception.kind, "malformed-json")

    def test_array_cert_is_malformed_cert(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "arr.json")
            _write(path, [1, 2])
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m.load_cert(path)
            self.assertEqual(ctx.exception.kind, "malformed-cert")

    def test_empty_file_is_malformed_json(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "empty.json")
            _write(path, "")
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m.load_cert(path)
            self.assertEqual(ctx.exception.kind, "malformed-json")

    def test_directory_is_malformed_cert(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m.load_cert(d)
            self.assertEqual(ctx.exception.kind, "malformed-cert")

    def test_valid_cert_loads(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "c.json")
            _write(path, {"kind": "gymCapabilityCertificate", "report": {}})
            data = self.m.load_cert(path)
            self.assertEqual(data["kind"], "gymCapabilityCertificate")


class ClassifyReportFailureTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_missing_scorecard_marker(self):
        err = self.m.classify_report_failure(
            2, "", "missing-scorecard: score.py 가 scorecard.json 을 남기지 않았다",
        )
        self.assertEqual(err.kind, "missing-scorecard")

    def test_missing_scorecard_korean_phrase(self):
        err = self.m.classify_report_failure(
            2, "", "scorecard.json 파일이 없다",
        )
        self.assertEqual(err.kind, "missing-scorecard")

    def test_malformed_json_marker(self):
        err = self.m.classify_report_failure(2, "", "malformed-json: 파싱 실패")
        self.assertEqual(err.kind, "malformed-json")

    def test_generic_nonzero_is_report_tool_failed(self):
        err = self.m.classify_report_failure(3, "", "하위 도구 실패: score.py")
        self.assertEqual(err.kind, "report-tool-failed")

    def test_permission_marker(self):
        err = self.m.classify_report_failure(2, "", "permission: 권한 없음")
        self.assertEqual(err.kind, "permission")

    def test_load_report_json_rejects_array(self):
        with self.assertRaises(self.m.CertifyError) as ctx:
            self.m.load_report_json("[1]")
        self.assertEqual(ctx.exception.kind, "malformed-report")

    def test_load_report_json_rejects_bad_text(self):
        with self.assertRaises(self.m.CertifyError) as ctx:
            self.m.load_report_json("{")
        self.assertEqual(ctx.exception.kind, "malformed-json")

    def test_load_report_json_accepts_object(self):
        data = self.m.load_report_json('{"kind":"gymCapabilityReport"}')
        self.assertEqual(data["kind"], "gymCapabilityReport")

    def test_run_report_empty_bin(self):
        with self.assertRaises(self.m.CertifyError) as ctx:
            self.m._run_report("")
        self.assertEqual(ctx.exception.kind, "missing-bin")

    def test_run_report_classifies_missing_scorecard(self):
        fake = mock.Mock()
        fake.returncode = 2
        fake.stdout = b""
        fake.stderr = "missing-scorecard: 없다".encode("utf-8")
        with mock.patch.object(self.m.subprocess, "run", return_value=fake):
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m._run_report("bin")
        self.assertEqual(ctx.exception.kind, "missing-scorecard")

    def test_run_report_empty_stdout_is_malformed_report(self):
        fake = mock.Mock()
        fake.returncode = 0
        fake.stdout = b"  "
        fake.stderr = b""
        with mock.patch.object(self.m.subprocess, "run", return_value=fake):
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m._run_report("bin")
        self.assertEqual(ctx.exception.kind, "malformed-report")

    def test_run_report_bad_stdout_json(self):
        fake = mock.Mock()
        fake.returncode = 0
        fake.stdout = b"not-json"
        fake.stderr = b""
        with mock.patch.object(self.m.subprocess, "run", return_value=fake):
            with self.assertRaises(self.m.CertifyError) as ctx:
                self.m._run_report("bin")
        self.assertEqual(ctx.exception.kind, "malformed-json")


class UnavailablePackCertTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_extract_unavailable(self):
        report = {**FIXED_REPORT, "packsUnavailable": ["sec", "xc"]}
        self.assertEqual(self.m.extract_unavailable(report), ["sec", "xc"])
        self.assertEqual(self.m.extract_unavailable({}), [])
        self.assertEqual(self.m.extract_unavailable(None), [])
        self.assertEqual(self.m.extract_unavailable({"packsUnavailable": "x"}), [])

    def test_certify_attaches_unavailable_exceptions(self):
        report = {**copy.deepcopy(FIXED_REPORT), "packsUnavailable": ["d"]}
        self.m._run_report = lambda bin_path: report
        self.m.benchmark_fingerprint = lambda root: "FP"
        self.m.fingerprint_is_empty = lambda root: False
        cert = self.m.certify("bin")
        self.assertEqual(cert["unavailablePacks"], ["d"])
        kinds = [e["kind"] for e in cert["exceptions"]]
        self.assertIn("unavailable-pack", kinds)
        self.assertTrue(cert["trusted"])
        self.assertEqual(cert["kind"], self.m.CERT_KIND)
        self.assertEqual(cert["proof"], self.m.PROOF_TEXT)

    def test_verify_detects_unavailable_set_change(self):
        claimed_report = {**copy.deepcopy(FIXED_REPORT), "packsUnavailable": ["d"]}
        fresh_report = {**copy.deepcopy(FIXED_REPORT), "packsUnavailable": ["e"]}
        self.m.benchmark_fingerprint = lambda root: "FP"
        self.m._run_report = lambda bin_path: copy.deepcopy(fresh_report)
        cert = {"kind": self.m.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": claimed_report}
        ok, diffs = self.m.verify(cert, "bin")
        self.assertFalse(ok)
        self.assertTrue(any("미가용 pack" in d for d in diffs))

    def test_verify_same_unavailable_passes(self):
        report = {**copy.deepcopy(FIXED_REPORT), "packsUnavailable": ["d"]}
        self.m.benchmark_fingerprint = lambda root: "FP"
        self.m._run_report = lambda bin_path: copy.deepcopy(report)
        cert = {"kind": self.m.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(report)}
        ok, diffs = self.m.verify(cert, "bin")
        self.assertTrue(ok, diffs)

    def test_certified_at_is_not_in_core(self):
        self.m._run_report = lambda bin_path: copy.deepcopy(FIXED_REPORT)
        self.m.benchmark_fingerprint = lambda root: "FP"
        self.m.fingerprint_is_empty = lambda root: False
        cert = self.m.certify("bin", measured_at="2026-08-18")
        self.assertEqual(cert["certifiedAt"], "2026-08-18")
        core = self.m.reproducible_core(cert["report"], cert["benchmarkFingerprint"])
        self.assertNotIn("certifiedAt", core)


class VerifyDefensiveTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_verify_wrong_kind_keeps_legacy_message(self):
        ok, diffs = self.m.verify({"kind": "other", "report": {}}, "bin")
        self.assertFalse(ok)
        self.assertTrue(any("kind 가" in d for d in diffs))

    def test_verify_non_object_cert(self):
        ok, diffs = self.m.verify([1, 2], "bin")  # type: ignore[arg-type]
        self.assertFalse(ok)
        self.assertTrue(any("객체가 아니다" in d for d in diffs))

    def test_verify_missing_report(self):
        ok, diffs = self.m.verify({"kind": self.m.CERT_KIND}, "bin")
        self.assertFalse(ok)
        self.assertTrue(any("report" in d for d in diffs))

    def test_verify_report_not_object(self):
        ok, diffs = self.m.verify(
            {"kind": self.m.CERT_KIND, "report": [1], "benchmarkFingerprint": "FP"},
            "bin",
        )
        self.assertFalse(ok)
        self.assertTrue(any("report" in d for d in diffs))

    def test_verify_report_error_becomes_false(self):
        def boom(_bin):
            raise self.m.CertifyError("missing-scorecard", "없다")

        self.m._run_report = boom
        cert = {"kind": self.m.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(FIXED_REPORT)}
        ok, diffs = self.m.verify(cert, "bin")
        self.assertFalse(ok)
        self.assertTrue(any("missing-scorecard" in d for d in diffs))

    def test_verify_detects_binary_identity_change(self):
        self.m.benchmark_fingerprint = lambda root: "FP"
        fresh = copy.deepcopy(FIXED_REPORT)
        fresh["runner"]["capabilitiesSha256"] = "d" * 64
        self.m._run_report = lambda bin_path: fresh
        cert = {"kind": self.m.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(FIXED_REPORT)}
        ok, diffs = self.m.verify(cert, "bin")
        self.assertFalse(ok)
        self.assertTrue(any("capabilitiesSha256" in d for d in diffs))

    def test_verify_detects_coverage_change(self):
        self.m.benchmark_fingerprint = lambda root: "FP"
        fresh = copy.deepcopy(FIXED_REPORT)
        fresh["coverage"]["percent"] = 1
        self.m._run_report = lambda bin_path: fresh
        cert = {"kind": self.m.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(FIXED_REPORT)}
        ok, diffs = self.m.verify(cert, "bin")
        self.assertFalse(ok)
        self.assertTrue(any("커버리지" in d for d in diffs))

    def test_verify_detects_axis_change(self):
        self.m.benchmark_fingerprint = lambda root: "FP"
        fresh = copy.deepcopy(FIXED_REPORT)
        fresh["axisProfile"] = []
        self.m._run_report = lambda bin_path: fresh
        cert = {"kind": self.m.CERT_KIND, "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(FIXED_REPORT)}
        ok, diffs = self.m.verify(cert, "bin")
        self.assertFalse(ok)
        self.assertTrue(any("축별" in d for d in diffs))

    def test_reproducible_core_accepts_none_report(self):
        core = self.m.reproducible_core(None, "FP")  # type: ignore[arg-type]
        self.assertEqual(core["benchmarkFingerprint"], "FP")
        self.assertIsNone(core["capabilitiesSha256"])
        self.assertEqual(core["coverage"], {
            "percent": None, "covered": None, "agentFacingTotal": None,
        })

    def test_validate_cert_empty_fingerprint(self):
        diffs = self.m.validate_cert({
            "kind": self.m.CERT_KIND,
            "report": {},
            "benchmarkFingerprint": "",
        })
        self.assertTrue(any("benchmarkFingerprint" in d for d in diffs))


class MainCliTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_main_verify_missing_cert_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--bin", "b", "--verify", os.path.join(d, "no.json")])
            self.assertEqual(code, 2)
            self.assertIn("missing-cert", buf.getvalue())

    def test_main_verify_bad_json_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "bad.json")
            _write(path, "{")
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--bin", "b", "--verify", path])
            self.assertEqual(code, 2)
            self.assertIn("malformed-json", buf.getvalue())

    def test_main_verify_array_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "arr.json")
            _write(path, [1])
            buf = io.StringIO()
            with mock.patch.object(sys, "stderr", buf):
                code = self.m.main(["--bin", "b", "--verify", path])
            self.assertEqual(code, 2)
            self.assertIn("malformed-cert", buf.getvalue())

    def test_main_verify_wrong_kind_exit_1(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "c.json")
            _write(path, {"kind": "nope", "report": {}})
            buf = io.StringIO()
            with mock.patch.object(sys, "stdout", buf):
                code = self.m.main(["--bin", "b", "--verify", path])
            self.assertEqual(code, 1)
            self.assertIn("kind 가", buf.getvalue())

    def test_main_verify_pass(self):
        self.m.benchmark_fingerprint = lambda root: "FP"
        self.m._run_report = lambda bin_path: copy.deepcopy(FIXED_REPORT)
        # Rebind on the already-loaded module used by main via import? main is on self.m
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "c.json")
            _write(path, {
                "kind": self.m.CERT_KIND,
                "benchmarkFingerprint": "FP",
                "report": copy.deepcopy(FIXED_REPORT),
            })
            buf = io.StringIO()
            with mock.patch.object(sys, "stdout", buf):
                code = self.m.main(["--bin", "b", "--verify", path])
            self.assertEqual(code, 0, buf.getvalue())
            self.assertIn("통과", buf.getvalue())

    def test_main_issue_stdout_json(self):
        self.m._run_report = lambda bin_path: copy.deepcopy(FIXED_REPORT)
        self.m.benchmark_fingerprint = lambda root: "a" * 64
        self.m.fingerprint_is_empty = lambda root: False
        buf = io.StringIO()
        with mock.patch.object(sys, "stdout", buf):
            code = self.m.main(["--bin", "b"])
        self.assertEqual(code, 0)
        payload = json.loads(buf.getvalue())
        self.assertEqual(payload["kind"], self.m.CERT_KIND)
        self.assertEqual(payload["report"]["accuracy"]["percent"], 100)

    def test_main_issue_out_file(self):
        self.m._run_report = lambda bin_path: {
            **copy.deepcopy(FIXED_REPORT), "packsUnavailable": ["d"],
        }
        self.m.benchmark_fingerprint = lambda root: "b" * 64
        self.m.fingerprint_is_empty = lambda root: False
        with tempfile.TemporaryDirectory() as d:
            out = os.path.join(d, "cert.json")
            code = self.m.main(["--bin", "b", "--out", out, "--at", "now"])
            self.assertEqual(code, 0)
            payload = json.loads(Path(out).read_text(encoding="utf-8"))
            self.assertEqual(payload["certifiedAt"], "now")
            self.assertEqual(payload["unavailablePacks"], ["d"])

    def test_main_issue_missing_scorecard_exit_2(self):
        def boom(_bin):
            raise self.m.CertifyError("missing-scorecard", "없다")

        self.m._run_report = boom
        buf = io.StringIO()
        with mock.patch.object(sys, "stderr", buf):
            code = self.m.main(["--bin", "b"])
        self.assertEqual(code, 2)
        self.assertIn("missing-scorecard", buf.getvalue())

    def test_dump_cert_json_newline(self):
        text = self.m.dump_cert_json({"kind": self.m.CERT_KIND})
        self.assertTrue(text.endswith("\n"))


class FatalCatchableTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_fatal_not_catchable(self):
        for exc in (KeyboardInterrupt(), SystemExit(1), MemoryError(), GeneratorExit()):
            self.assertTrue(self.m.is_fatal_exception(exc))
            self.assertFalse(self.m.is_catchable_exception(exc))

    def test_certify_error_is_catchable(self):
        self.assertTrue(self.m.is_catchable_exception(
            self.m.CertifyError("missing-cert", "x"),
        ))

    def test_classify_os_error_roles(self):
        self.assertEqual(
            self.m.classify_os_error(FileNotFoundError("x"), role="cert"),
            "missing-cert",
        )
        self.assertEqual(
            self.m.classify_os_error(FileNotFoundError("x"), role="bin"),
            "missing-bin",
        )
        self.assertEqual(
            self.m.classify_os_error(json.JSONDecodeError("m", "d", 0), role="cert"),
            "malformed-json",
        )

    def test_wrap_reraises_fatal(self):
        with self.assertRaises(SystemExit):
            self.m.wrap_exception(SystemExit(2))

    def test_exception_record_pack(self):
        rec = self.m.exception_record("unavailable-pack", "부재", pack="d")
        self.assertEqual(rec["pack"], "d")
        rec2 = self.m.exception_record("??", "x")
        self.assertEqual(rec2["kind"], "unexpected")

    def test_certify_error_unknown_kind(self):
        err = self.m.CertifyError("nope", "x")
        self.assertEqual(err.kind, "unexpected")


class CompareHelperTests(unittest.TestCase):
    def setUp(self):
        self.m = load()

    def test_compare_core_all_match(self):
        core = {"benchmarkFingerprint": "a", "capabilitiesSha256": "b",
                "accuracy": 1, "coverage": 2, "axisProfile": 3}
        self.assertEqual(self.m.compare_core(core, core), [])

    def test_compare_unavailable_order_independent(self):
        self.assertEqual(self.m.compare_unavailable(["b", "a"], ["a", "b"]), [])
        diffs = self.m.compare_unavailable(["a"], [])
        self.assertEqual(len(diffs), 1)

    def test_mapping_or_empty(self):
        self.assertEqual(self.m.mapping_or_empty(None), {})
        self.assertEqual(self.m.mapping_or_empty([1]), {})
        self.assertEqual(self.m.mapping_or_empty({"a": 1}), {"a": 1})

    def test_format_issue_summary(self):
        cert = {"benchmarkFingerprint": "abcdef1234567890",
                "report": {"accuracy": {"percent": 62}}}
        text = self.m.format_issue_summary(cert)
        self.assertIn("62%", text)
        self.assertIn("abcdef123456", text)


if __name__ == "__main__":
    unittest.main()
