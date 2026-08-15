"""[certify] gym 능력 인증서 계약 — 벤치마크 지문 결정성 + 위조 탐지.

핵심: 인증서는 '재현 = 증명'이다. 벤치마크 지문(전 pack 정의 sha256)이 결정적이고,
재현 core(지문·바이너리 신원·정확도·커버리지·축)가 인증 시점과 하나라도 다르면
위조로 판정한다. 바이너리 없이 순수 로직만 시험한다(재현은 _run_report 를 목킹).
"""

from __future__ import annotations

import copy
import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "certify.py"


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


class CertifyTests(unittest.TestCase):
    def test_fingerprint_is_deterministic_and_change_sensitive(self):
        cov = load()
        with tempfile.TemporaryDirectory() as d:
            pd = os.path.join(d, "packs", "p1")
            os.makedirs(os.path.join(pd, "tasks"))
            open(os.path.join(pd, "pack.json"), "w").write('{"id":"p1"}')
            open(os.path.join(pd, "tasks", "A01.json"), "w").write('{"id":"A01"}')
            fp1 = cov.benchmark_fingerprint(d)
            fp2 = cov.benchmark_fingerprint(d)
            self.assertEqual(fp1, fp2)                       # 결정적
            self.assertEqual(len(fp1), 64)                   # sha256 hex
            open(os.path.join(pd, "tasks", "A01.json"), "w").write('{"id":"A01","x":1}')
            self.assertNotEqual(fp1, cov.benchmark_fingerprint(d))  # 변조에 민감

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


if __name__ == "__main__":
    unittest.main()
