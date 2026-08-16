"""[repair_loop] 검증 실패 → 진단 → 수리 → 재검증 루프의 계약 시험.

`tools/repair_loop/loop.py` 는 rhwp CLI 판정 명령(`verify`/`edit --verify`/
`render-diff`/`ir-diff`/`run`)을 서브프로세스로 그대로 부른다. 진단(diagnose)과
수리(repair) 함수는 실제 바이너리 없이 순수 함수로 시험하고, 오케스트레이션의
안전장치(max_attempts·진전 판정·loop detection)는 실행 순서를 미리 정해 둔
가짜 rhwp 스텁(운영체제별 래퍼 + 파이썬 구현)으로 결정론적으로 시험한다 — 진짜 rhwp 빌드가
없어도(CI에 바이너리가 없어도) 이 시험은 항상 돈다.
"""

from __future__ import annotations

import importlib.util
import json
import os
import shlex
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "tools" / "repair_loop" / "loop.py"


def load():
    spec = importlib.util.spec_from_file_location("repair_loop_module", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# ---------------------------------------------------------------------------
# 가짜 rhwp 스텁 — 호출 순번에 따라 미리 정해 둔 (종료 코드, JSON) 을 낸다.
# ---------------------------------------------------------------------------

_IMPL_TEMPLATE = """
import json, os, sys

STATE_PATH = {state_path!r}
RESP_PATH = {resp_path!r}

state = {{"calls": 0}}
if os.path.exists(STATE_PATH):
    with open(STATE_PATH, "r", encoding="utf-8") as fh:
        state = json.load(fh)
state["calls"] += 1
with open(STATE_PATH, "w", encoding="utf-8") as fh:
    json.dump(state, fh)

with open(RESP_PATH, "r", encoding="utf-8") as fh:
    responses = json.load(fh)
idx = min(state["calls"] - 1, len(responses) - 1)
resp = responses[idx]
sys.stdout.write(json.dumps(resp["envelope"]))
sys.exit(resp["returncode"])
"""


def make_fake_bin(tmpdir: str, responses: list[dict]) -> str:
    """`responses` 는 호출될 때마다 순서대로 소비되는 [{"returncode","envelope"}].
    마지막 항목보다 더 호출되면 마지막 항목을 반복해 낸다."""
    tmp = Path(tmpdir)
    resp_path = tmp / "responses.json"
    state_path = tmp / "state.json"
    impl_path = tmp / "fake_rhwp_impl.py"

    resp_path.write_text(json.dumps(responses, ensure_ascii=False), encoding="utf-8")
    if state_path.exists():
        state_path.unlink()
    impl_path.write_text(
        _IMPL_TEMPLATE.format(state_path=str(state_path), resp_path=str(resp_path)),
        encoding="utf-8",
    )
    if os.name == "nt":
        wrapper_path = tmp / "fake_rhwp.bat"
        wrapper_path.write_text(
            f'@echo off\r\n"{sys.executable}" "{impl_path}" %*\r\nexit /b %errorlevel%\r\n',
            encoding="utf-8",
        )
    else:
        wrapper_path = tmp / "fake_rhwp"
        wrapper_path.write_text(
            "#!/bin/sh\n"
            f"exec {shlex.quote(sys.executable)} {shlex.quote(str(impl_path))} \"$@\"\n",
            encoding="utf-8",
        )
        wrapper_path.chmod(0o755)
    return str(wrapper_path)


# ---------------------------------------------------------------------------
# diagnose() — 판정 종류 분류 (순수 함수, 서브프로세스 없음)
# ---------------------------------------------------------------------------

class DiagnoseTests(unittest.TestCase):
    def setUp(self):
        self.mod = load()

    def test_verify_pass(self):
        d = self.mod.diagnose("verify", {"returncode": 0, "envelope": {"verdict": "pass"},
                                        "timedOut": False})
        self.assertEqual(d["category"], "pass")

    def test_verify_expect_mismatch(self):
        env = {"verdict": "fail", "failCount": 1,
               "expectations": [{"kind": "minPages", "expected": 99, "actual": 3, "pass": False}]}
        d = self.mod.diagnose("verify", {"returncode": 3, "envelope": env, "timedOut": False})
        self.assertEqual(d["category"], "expectMismatch")
        self.assertEqual(d["detail"]["failingKinds"], ["minPages"])

    def test_edit_verify_reparse_mismatch(self):
        env = {"verify": {"identical": False, "diffCount": 4}}
        d = self.mod.diagnose("edit-verify", {"returncode": 3, "envelope": env, "timedOut": False})
        self.assertEqual(d["category"], "editReparseMismatch")
        self.assertEqual(d["detail"]["diffCount"], 4)

    def test_render_diff_roundtrip_over(self):
        env = {"mode": "roundtrip", "status": "OVER", "regression": True,
               "via": "hwpx", "maxDisp": 3.7, "threshold": 1.0}
        d = self.mod.diagnose("render-diff", {"returncode": 3, "envelope": env, "timedOut": False})
        self.assertEqual(d["category"], "renderDisplacementOver")
        self.assertEqual(d["detail"]["via"], "hwpx")

    def test_render_diff_pair_struct_mismatch_not_repairable(self):
        env = {"mode": "pair", "status": "STRUCT_MISMATCH", "regression": True, "via": None}
        d = self.mod.diagnose("render-diff", {"returncode": 3, "envelope": env, "timedOut": False})
        self.assertEqual(d["category"], "renderPairMismatch")
        self.assertNotIn(d["category"], self.mod.REPAIRS)

    def test_ir_diff_structural(self):
        env = {"identical": False, "diffCount": 12, "categories": {"cc": 12}}
        d = self.mod.diagnose("ir-diff", {"returncode": 3, "envelope": env, "timedOut": False})
        self.assertEqual(d["category"], "irStructuralDiff")
        self.assertEqual(d["detail"]["diffCount"], 12)

    def test_run_precondition_failed(self):
        env = {"invalid": [{"code": "preconditionFailed", "expected": "aaa", "actual": "bbb",
                            "reason": "stale"}]}
        d = self.mod.diagnose("run", {"returncode": 2, "envelope": env, "timedOut": False})
        self.assertEqual(d["category"], "casStalePrecondition")
        self.assertEqual(d["detail"]["actual"], "bbb")
        self.assertIn("casStalePrecondition", self.mod.REPAIRS)

    def test_run_other_invalid_not_repairable(self):
        env = {"invalid": [{"code": "fieldNotFound"}]}
        d = self.mod.diagnose("run", {"returncode": 2, "envelope": env, "timedOut": False})
        self.assertEqual(d["category"], "planInvalid")
        self.assertNotIn(d["category"], self.mod.REPAIRS)

    def test_timeout_and_unparseable(self):
        t = self.mod.diagnose("run", {"timedOut": True, "returncode": None, "envelope": None,
                                      "stderr": ""})
        self.assertEqual(t["category"], "timeout")
        u = self.mod.diagnose("run", {"timedOut": False, "returncode": 1, "envelope": None,
                                      "stderr": "boom"})
        self.assertEqual(u["category"], "unparseable")

    def test_detect_kind(self):
        self.assertEqual(self.mod.detect_kind(["verify", "x.hwp", "--json"]), "verify")
        self.assertEqual(self.mod.detect_kind(["render-diff", "x.hwp", "--json"]), "render-diff")
        self.assertEqual(self.mod.detect_kind(["run", "plan.json", "--json"]), "run")
        self.assertEqual(
            self.mod.detect_kind(["edit", "fill-fields", "x.hwp", "--verify", "--json"]),
            "edit-verify")
        self.assertEqual(self.mod.detect_kind(["edit", "fill-fields", "x.hwp", "--json"]),
                          "unknown")
        self.assertEqual(self.mod.detect_kind([]), "unknown")


# ---------------------------------------------------------------------------
# repair 함수 — CAS 재계획 · render-diff via 전환
# ---------------------------------------------------------------------------

class RepairFunctionTests(unittest.TestCase):
    def setUp(self):
        self.mod = load()

    def test_cas_repair_rewrites_plan_copy_not_original(self):
        with tempfile.TemporaryDirectory() as d:
            plan_path = Path(d) / "plan.json"
            plan = {"planVersion": "1.0", "input": "samples/field-01.hwp", "output": "out.hwp",
                    "steps": [{"action": "fill_fields", "data": {"a": "b"}}],
                    "preconditions": {"inputSha256": "stale" * 12 + "----"}}
            original_text = json.dumps(plan, ensure_ascii=False)
            plan_path.write_text(original_text, encoding="utf-8")

            diagnosis = {"category": "casStalePrecondition",
                         "detail": {"expected": "stale", "actual": "fresh" * 12 + "1234"}}
            ctx = {"check_args": ["run", str(plan_path), "--json"], "diagnosis": diagnosis,
                   "work_dir": Path(d) / "work", "attempt": 1, "state": {}}
            result = self.mod._repair_cas_stale_precondition(ctx)
            self.assertIsNotNone(result)
            # 원본 계획 파일은 그대로다.
            self.assertEqual(plan_path.read_text(encoding="utf-8"), original_text)
            # 새 인자는 사본을 가리키고, 사본의 해시는 갱신됐다.
            new_plan_path = Path(result.new_args[1])
            self.assertNotEqual(new_plan_path, plan_path)
            new_plan = json.loads(new_plan_path.read_text(encoding="utf-8"))
            self.assertEqual(new_plan["preconditions"]["inputSha256"], diagnosis["detail"]["actual"])

    def test_cas_repair_inline_plan_json(self):
        plan = {"planVersion": "1.0", "input": "x.hwp", "output": "o.hwp",
                "steps": [{"action": "fill_fields", "data": {"a": "b"}}],
                "preconditions": {"inputSha256": "old"}}
        args = ["run", "--plan-json", json.dumps(plan), "--json"]
        diagnosis = {"category": "casStalePrecondition", "detail": {"actual": "new-hash"}}
        with tempfile.TemporaryDirectory() as d:
            ctx = {"check_args": args, "diagnosis": diagnosis, "work_dir": Path(d),
                   "attempt": 1, "state": {}}
            result = self.mod._repair_cas_stale_precondition(ctx)
            self.assertIsNotNone(result)
            rewritten = json.loads(result.new_args[2])
            self.assertEqual(rewritten["preconditions"]["inputSha256"], "new-hash")

    def test_cas_repair_no_preconditions_object_is_unrepairable(self):
        with tempfile.TemporaryDirectory() as d:
            plan_path = Path(d) / "plan.json"
            plan = {"planVersion": "1.0", "input": "x.hwp", "output": "o.hwp",
                    "steps": [{"action": "fill_fields", "data": {"a": "b"}}]}
            plan_path.write_text(json.dumps(plan), encoding="utf-8")
            diagnosis = {"category": "casStalePrecondition", "detail": {"actual": "new-hash"}}
            ctx = {"check_args": ["run", str(plan_path), "--json"], "diagnosis": diagnosis,
                   "work_dir": Path(d) / "work", "attempt": 1, "state": {}}
            self.assertIsNone(self.mod._repair_cas_stale_precondition(ctx))

    def test_render_via_toggle_switches_and_stops_after_both_tried(self):
        ctx = {"check_args": ["render-diff", "x.hwp", "--json"],
              "diagnosis": {"detail": {"via": "hwpx"}}, "work_dir": Path("."), "attempt": 1,
              "state": {}}
        first = self.mod._repair_render_via_toggle(ctx)
        self.assertIsNotNone(first)
        self.assertIn("hwp", first.new_args)
        self.assertEqual(first.new_args[-2:], ["--via", "hwp"])

        # 두 번째: 이제 hwp 도 실패했다고 가정 — hwpx 는 이미 시도했으니 더 없다.
        ctx2 = {"check_args": first.new_args, "diagnosis": {"detail": {"via": "hwp"}},
                "work_dir": Path("."), "attempt": 2, "state": ctx["state"]}
        second = self.mod._repair_render_via_toggle(ctx2)
        self.assertIsNone(second)


# ---------------------------------------------------------------------------
# orchestrate() — 안전장치 3종을 가짜 스텁으로 결정론적으로 시험
# ---------------------------------------------------------------------------

class OrchestrateTests(unittest.TestCase):
    def setUp(self):
        self.mod = load()
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.work_dir = Path(self._tmp.name) / "work"
        # casStalePrecondition 수리기는 실제로 존재하는 계획 파일을 읽어야 한다 —
        # orchestrate 시험은 실제 rhwp 를 부르지 않으므로 여기서 직접 마련한다.
        self.plan_path = Path(self._tmp.name) / "plan.json"
        self.plan_path.write_text(json.dumps({
            "planVersion": "1.0", "input": "x.hwp", "output": "o.hwp",
            "steps": [{"action": "fill_fields", "data": {"a": "b"}}],
            "preconditions": {"inputSha256": "stale"},
        }), encoding="utf-8")

    def test_repair_then_pass(self):
        """1차 casStalePrecondition 실패 → 수리 적용 → 2차 통과."""
        bin_path = make_fake_bin(self._tmp.name, [
            {"returncode": 2, "envelope": {"invalid": [{"code": "preconditionFailed",
                                                          "expected": "a", "actual": "b"}]}},
            {"returncode": 0, "envelope": {"ok": True}},
        ])
        summary = self.mod.orchestrate(bin_path, ["run", str(self.plan_path), "--json"], "run",
                                       max_attempts=5, timeout=10.0, work_dir=self.work_dir)
        self.assertEqual(summary["outcome"], "pass")
        self.assertIsNone(summary["haltReason"])
        self.assertEqual(len(summary["attempts"]), 2)
        self.assertIsNotNone(summary["attempts"][0]["repairApplied"])

    def test_no_repair_strategy_halts_on_first_attempt(self):
        """수리기가 없는 진단 종류는 시도를 소모하지 않고 즉시 인계한다."""
        bin_path = make_fake_bin(self._tmp.name, [
            {"returncode": 3, "envelope": {"verdict": "fail", "failCount": 1,
                                           "expectations": [{"kind": "minPages", "expected": 99,
                                                             "actual": 3, "pass": False}]}},
        ])
        summary = self.mod.orchestrate(bin_path, ["verify", "x.hwp", "--expect-min-pages", "99",
                                                   "--json"], "verify", max_attempts=5,
                                       timeout=10.0, work_dir=self.work_dir)
        self.assertEqual(summary["outcome"], "handoff")
        self.assertEqual(summary["haltReason"], "noRepairStrategy")
        self.assertEqual(len(summary["attempts"]), 1)

    def test_no_progress_loop_detection_stops_before_max_attempts(self):
        """수리를 적용해도 진단 시그니처가 반복되면 max_attempts 전에 멈춘다."""
        same_failure = {"returncode": 2, "envelope": {"invalid": [{"code": "preconditionFailed",
                                                                    "expected": "a", "actual": "b"}]}}
        bin_path = make_fake_bin(self._tmp.name, [same_failure])  # 항상 같은 실패를 반복
        summary = self.mod.orchestrate(bin_path, ["run", str(self.plan_path), "--json"], "run",
                                       max_attempts=10, timeout=10.0, work_dir=self.work_dir)
        self.assertEqual(summary["outcome"], "handoff")
        self.assertEqual(summary["haltReason"], "noProgress")
        # 10번까지 갈 수 있었지만 2번째 반복(시그니처 재발)에서 멈춰야 한다.
        self.assertEqual(len(summary["attempts"]), 2)

    def test_max_attempts_exhausted_when_signature_keeps_changing(self):
        """수리를 계속 적용해도 매번 다른(진전은 있는) 실패면 max_attempts 에서 멈춘다."""
        responses = [
            {"returncode": 2, "envelope": {"invalid": [{"code": "preconditionFailed",
                                                         "expected": "a", "actual": f"actual-{i}"}]}}
            for i in range(10)
        ]
        bin_path = make_fake_bin(self._tmp.name, responses)
        summary = self.mod.orchestrate(bin_path, ["run", str(self.plan_path), "--json"], "run",
                                       max_attempts=3, timeout=10.0, work_dir=self.work_dir)
        self.assertEqual(summary["outcome"], "handoff")
        self.assertEqual(summary["haltReason"], "maxAttemptsReached")
        self.assertEqual(len(summary["attempts"]), 3)
        # 절대 무한루프가 아니었다 — 정확히 상한만큼만 돌았다.
        for a in summary["attempts"]:
            self.assertIsNotNone(a["repairApplied"])


if __name__ == "__main__":
    unittest.main()
