"""[#5207] 라이브 오라클 프로브 계약 — 다중 자리표·결정성 실패·부재 보고.

핵심 불변식:
1. 한 문자열의 여러 `{sub:이름}` 은 모두 치환된다(첫 하나만 바꾸면 안 된다).
2. 두 번 계산이 어긋나면 결정성 프로브는 실패한다.
3. 산출물이 없으면 status=absent 이고 통과로 위장하지 않는다.
4. `--json` 은 팩 픽스처 없이 kind=gymOracleProbe / schemaVersion=1.0 을 낸다.

바이너리·pack 없이 순수 함수와 임시 디렉터리만 시험한다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "oracle_probe.py"
BASELINE = REPO_ROOT / "gym" / "tools" / "build_baseline.py"


def load(name="gym_oracle_probe"):
    spec = importlib.util.spec_from_file_location(name, TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_baseline():
    spec = importlib.util.spec_from_file_location("gym_build_baseline_for_probe", BASELINE)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class PlaceholderTests(unittest.TestCase):
    def test_multiple_sub_placeholders_all_resolve(self):
        mod = load("gym_oracle_probe_multi_sub")
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'
            out = mod.probe_placeholders(token, {"input": "in.hwp"}, sub_dir)
            self.assertTrue(out["ok"], out)
            self.assertNotIn("{sub:", out["resolved"])
            self.assertEqual(out["leftover"], [])
            self.assertIn("o1.hwp", out["resolved"])
            self.assertIn("o2.hwp", out["resolved"])
            self.assertEqual(out["names"], ["o1.hwp", "o2.hwp"])

    def test_resolve_matches_build_baseline(self):
        probe = load("gym_oracle_probe_vs_baseline")
        baseline = load_baseline()
        task = {"input": "samples/in.hwp"}
        with tempfile.TemporaryDirectory() as sub_dir:
            tokens = (
                "{input}",
                '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}',
                "{sub:capsules/child.hwp}",
                "plain-literal",
                "keep {input} embedded",
            )
            for token in tokens:
                self.assertEqual(
                    probe.resolve_placeholders(token, task, sub_dir),
                    baseline.resolve(token, task, sub_dir),
                    token,
                )

    def test_cmd_list_resolves_every_token(self):
        mod = load("gym_oracle_probe_cmd")
        with tempfile.TemporaryDirectory() as sub_dir:
            cmd = ["search", "{sub:edited.hwp}", "--json", "--", "규제"]
            report = mod.probe_cmd_placeholders(cmd, {"input": "in.hwp"}, sub_dir)
            self.assertTrue(report["ok"], report)
            resolved = [t["resolved"] for t in report["tokens"]]
            self.assertTrue(any(str(item).endswith("edited.hwp") for item in resolved))
            self.assertTrue(all("{sub:" not in str(item) for item in resolved))


class DeterminismTests(unittest.TestCase):
    def test_stable_compute_passes(self):
        mod = load("gym_oracle_probe_det_ok")
        report = mod.probe_determinism(lambda: {"pageCount": 3, "kind": "info"})
        self.assertTrue(report["ok"])
        self.assertTrue(report["equal"])
        self.assertEqual(report["runs"], 2)

    def test_determinism_fail_on_drift(self):
        mod = load("gym_oracle_probe_det_fail")
        state = {"i": 0}

        def drift():
            state["i"] += 1
            return {"n": state["i"]}

        report = mod.probe_determinism(drift)
        self.assertFalse(report["ok"])
        self.assertFalse(report["equal"])
        self.assertNotEqual(report["first"], report["second"])

    def test_key_order_and_numeric_string_are_equal(self):
        mod = load("gym_oracle_probe_norm")
        seq = [{"b": 1, "a": "2"}, {"a": 2, "b": "1"}]
        report = mod.probe_determinism(lambda: seq.pop(0))
        self.assertTrue(report["ok"], report)

    def test_exception_is_not_a_pass(self):
        mod = load("gym_oracle_probe_exc")
        report = mod.probe_determinism(lambda: (_ for _ in ()).throw(RuntimeError("boom")))
        self.assertFalse(report["ok"])
        self.assertIn("RuntimeError", report.get("error", ""))


class MissingArtifactTests(unittest.TestCase):
    def test_missing_file_is_absent_not_pass(self):
        mod = load("gym_oracle_probe_missing")
        with tempfile.TemporaryDirectory() as sub_dir:
            path = os.path.join(sub_dir, "never-written.svg")
            report = mod.probe_missing_artifact(path)
            self.assertFalse(report["ok"])
            self.assertFalse(report["present"])
            self.assertEqual(report["status"], "absent")

    def test_present_file_passes(self):
        mod = load("gym_oracle_probe_present")
        with tempfile.TemporaryDirectory() as sub_dir:
            path = os.path.join(sub_dir, "answer.json")
            Path(path).write_text("{}\n", encoding="utf-8")
            report = mod.probe_missing_artifact(path)
            self.assertTrue(report["ok"])
            self.assertTrue(report["present"])
            self.assertEqual(report["status"], "present")

    def test_directory_is_not_a_file_and_not_pass(self):
        mod = load("gym_oracle_probe_dir")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_missing_artifact(sub_dir)
            self.assertFalse(report["ok"])
            self.assertEqual(report["status"], "not-a-file")


class EnvelopeAndCliTests(unittest.TestCase):
    def test_json_structural_self_check_has_schema(self):
        proc = subprocess.run(
            [sys.executable, str(TOOL), "--json"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))
        report = json.loads(proc.stdout.decode("utf-8"))
        self.assertEqual(report["kind"], "gymOracleProbe")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertEqual(report["mode"], "structural")
        self.assertTrue(report["ok"], report)
        self.assertFalse(report["probes"]["missingArtifact"]["ok"])
        self.assertEqual(report["probes"]["missingArtifact"]["status"], "absent")

    def test_selftest_covers_required_probes(self):
        proc = subprocess.run(
            [sys.executable, str(TOOL), "--json", "--selftest"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))
        report = json.loads(proc.stdout.decode("utf-8"))
        names = {c["name"] for c in report["checks"]}
        self.assertIn("placeholders-multi-sub", names)
        self.assertIn("determinism-drift-detected", names)
        self.assertIn("artifact-absent-is-not-pass", names)
        self.assertTrue(report["ok"], report.get("failed"))

    def test_run_helper_returns_envelope(self):
        mod = load("gym_oracle_probe_run")
        buf = io.StringIO()
        old = sys.stdout
        sys.stdout = buf
        try:
            code = mod.run(["--json"])
        finally:
            sys.stdout = old
        self.assertEqual(code, 0)
        report = json.loads(buf.getvalue())
        self.assertEqual(report["kind"], "gymOracleProbe")
        self.assertEqual(report["schemaVersion"], "1.0")


if __name__ == "__main__":
    unittest.main()
