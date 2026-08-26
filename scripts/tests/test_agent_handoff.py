"""[handoff] Agent Capability Handoff 오케스트레이터의 계약 시험.

`tools/handoff/orchestrator.py` 는 외부 에이전트를 **진짜 서브프로세스**로 띄운다.
이 시험은 결정론적 가짜 에이전트(운영체제별 래퍼 + 파이썬 구현 — repair_loop 시험과
같은 방식)로 성공/실패→재시도/timeout/스키마 위반/boundary 위반/fallback/정책 거부를
전부 실증하고, NDJSON 저널의 지문 체인 내용까지 검사한다. 진짜 rhwp 빌드가 없어도
(CI 에 바이너리가 없어도) 항상 돈다 — consistency 재검증은 가짜 rhwp 스텁으로 시험한다.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "tools" / "handoff" / "orchestrator.py"


def load():
    spec = importlib.util.spec_from_file_location("handoff_module", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


MOD = load()


# ---------------------------------------------------------------------------
# 가짜 에이전트 — stdin 으로 HandoffTask 를 받아 정해진 행동을 한다
# ---------------------------------------------------------------------------

_AGENT_PRELUDE = """
import hashlib, json, sys
from pathlib import Path

task = json.load(sys.stdin)
out = Path("out")


def write_expected():
    outputs = []
    for rel in task["expectedOutputs"]:
        p = out / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        data = ("converted:" + rel).encode("utf-8")
        p.write_bytes(data)
        outputs.append({"path": rel, "sha256": hashlib.sha256(data).hexdigest()})
    return outputs


def ok_result(outputs, tools_used=None, extra=None):
    r = {
        "handoffVersion": "1.0",
        "taskId": task["taskId"],
        "status": "ok",
        "outputs": outputs,
        "capabilities": [{"name": "demo-convert", "kind": "command",
                          "detail": "fixture agent"}],
        "toolsUsed": task["allowedTools"][:1] if tools_used is None else tools_used,
    }
    if extra:
        r.update(extra)
    return r
"""


def make_agent(tmpdir: str, name: str, body: str) -> str:
    """가짜 에이전트를 만들고 오케스트레이터에 넘길 명령 문자열을 반환한다."""
    tmp = Path(tmpdir)
    impl = tmp / f"{name}.py"
    impl.write_text(_AGENT_PRELUDE + "\n" + body + "\n", encoding="utf-8")
    if os.name == "nt":
        wrapper = tmp / f"{name}.bat"
        wrapper.write_text(
            f'@echo off\r\n"{sys.executable}" "{impl}" %*\r\nexit /b %errorlevel%\r\n',
            encoding="utf-8")
    else:
        wrapper = tmp / f"{name}.sh"
        wrapper.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{impl}" "$@"\n',
                           encoding="utf-8")
        wrapper.chmod(0o755)
    return str(wrapper)


def make_fake_rhwp(tmpdir: str, exit_code: int) -> str:
    """consistency 재검증용 가짜 rhwp — info 호출에 정해진 종료 코드를 낸다."""
    tmp = Path(tmpdir)
    impl = tmp / f"fake_rhwp_{exit_code}.py"
    impl.write_text(
        f"import sys\nsys.stdout.write('{{}}')\nsys.exit({exit_code})\n",
        encoding="utf-8")
    if os.name == "nt":
        wrapper = tmp / f"fake_rhwp_{exit_code}.bat"
        wrapper.write_text(
            f'@echo off\r\n"{sys.executable}" "{impl}" %*\r\nexit /b %errorlevel%\r\n',
            encoding="utf-8")
    else:
        wrapper = tmp / f"fake_rhwp_{exit_code}.sh"
        wrapper.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{impl}" "$@"\n',
                           encoding="utf-8")
        wrapper.chmod(0o755)
    return str(wrapper)


AGENT_SUCCESS = 'print(json.dumps(ok_result(write_expected())))'

AGENT_SUCCESS_WITHOUT_SHA256 = """
outputs = write_expected()
for output in outputs:
    output.pop("sha256")
print(json.dumps(ok_result(outputs)))
"""

AGENT_FAIL_THEN_SUCCEED_TEMPLATE = """
state_path = Path({state!r})
calls = int(state_path.read_text()) if state_path.exists() else 0
calls += 1
state_path.write_text(str(calls))
if calls == 1:
    print(json.dumps({{"handoffVersion": "1.0", "taskId": task["taskId"],
                       "status": "error", "outputs": [], "capabilities": [],
                       "toolsUsed": [], "error": {{"code": "transient"}}}}))
else:
    print(json.dumps(ok_result(write_expected())))
"""

AGENT_TIMEOUT = 'import time\ntime.sleep(15)'

AGENT_BAD_JSON = 'sys.stdout.write("정말 JSON 이 아니다")'

AGENT_SCHEMA_VIOLATION = """
print(json.dumps({"handoffVersion": "1.0", "taskId": task["taskId"],
                  "outputs": "이건 배열이 아니다"}))
"""

AGENT_WRITES_OUTSIDE_OUT = """
outputs = write_expected()
Path("stray.txt").write_text("boundary 밖 기록", encoding="utf-8")
print(json.dumps(ok_result(outputs)))
"""

AGENT_OUTPUT_ESCAPE = """
outputs = write_expected()
outputs.append({"path": "../escape.txt", "sha256": "0" * 64})
print(json.dumps(ok_result(outputs)))
"""

AGENT_UNDECLARED_OUTPUT = """
outputs = write_expected()
(out / "extra.txt").write_text("선언 안 한 산출", encoding="utf-8")
print(json.dumps(ok_result(outputs)))
"""

AGENT_TOOL_NOT_ALLOWED = """
outputs = write_expected()
print(json.dumps(ok_result(outputs, tools_used=["금지된-도구"])))
"""

AGENT_MODIFIES_INPUT = """
outputs = write_expected()
for rel in task["inputs"]:
    Path(rel).write_text("입력 사본 변조", encoding="utf-8")
print(json.dumps(ok_result(outputs)))
"""

AGENT_VERDICT = """
print(json.dumps({"handoffVersion": "1.0", "taskId": task["taskId"],
                  "status": "verdict", "outputs": [], "capabilities": [],
                  "toolsUsed": []}))
"""

AGENT_ALWAYS_ERROR = """
print(json.dumps({"handoffVersion": "1.0", "taskId": task["taskId"],
                  "status": "error", "outputs": [], "capabilities": [],
                  "toolsUsed": [], "error": {"code": "permanent"}}))
"""

AGENT_MISSING_EXPECTED = """
print(json.dumps(ok_result([])))
"""


# ---------------------------------------------------------------------------
# 공통 도우미
# ---------------------------------------------------------------------------

class HandoffCase(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory(prefix="handoff_test_")
        self.tmp = self._tmp.name
        self.addCleanup(self._tmp.cleanup)

    def write_task(self, **overrides) -> Path:
        doc = Path(self.tmp) / "input_doc.hwpx"
        if not doc.exists():
            doc.write_bytes(b"fixture-hwpx-bytes")
        task = {
            "handoffVersion": "1.0",
            "taskId": "t-demo",
            "objective": "픽스처 문서를 변환한다",
            "inputs": [str(doc)],
            "allowedTools": ["rhwp export-hwpx"],
            "timeoutSec": 30,
            "expectedOutputs": [{"path": "converted.hwpx"}],
        }
        task.update(overrides)
        p = Path(self.tmp) / "task.json"
        p.write_text(json.dumps(task, ensure_ascii=False), encoding="utf-8")
        return p

    def run_main(self, argv: list[str]) -> tuple[int, dict]:
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = MOD.main(argv)
        text = buf.getvalue()
        envelope = json.loads(text) if text.strip().startswith("{") else {}
        return rc, envelope

    def journal_records(self, work_dir: str) -> list[dict]:
        p = Path(work_dir) / "handoff.journal.ndjson"
        return [json.loads(l) for l in p.read_text(encoding="utf-8").splitlines()
                if l.strip()]


# ---------------------------------------------------------------------------
# 1. task 스키마 (정적 선검증)
# ---------------------------------------------------------------------------

class TaskSchemaTest(HandoffCase):
    def test_good_task_passes(self):
        task = json.loads(self.write_task().read_text(encoding="utf-8"))
        self.assertEqual(MOD.validate_task(task), [])

    def test_missing_fields_rejected(self):
        problems = MOD.validate_task({"handoffVersion": "1.0"})
        self.assertTrue(any("taskId" in p for p in problems))
        self.assertTrue(any("objective" in p for p in problems))
        self.assertTrue(any("timeoutSec" in p for p in problems))

    def test_expected_output_escape_rejected(self):
        task = json.loads(self.write_task(
            expectedOutputs=[{"path": "../evil.hwpx"}]).read_text(encoding="utf-8"))
        problems = MOD.validate_task(task)
        self.assertTrue(any("상대 경로" in p for p in problems))

    def test_nonexistent_input_rejected(self):
        task = json.loads(self.write_task(
            inputs=[str(Path(self.tmp) / "없는파일.hwpx")]).read_text(encoding="utf-8"))
        problems = MOD.validate_task(task)
        self.assertTrue(any("입력 파일이 없다" in p for p in problems))

    def test_bad_task_is_usage_error_exit_2(self):
        task_path = self.write_task(taskId="")
        agent = make_agent(self.tmp, "unused", AGENT_SUCCESS)
        rc, _ = self.run_main(["--task", str(task_path), "--agent", agent, "--json"])
        self.assertEqual(rc, 2)

    def test_wire_task_hides_original_paths(self):
        task = json.loads(self.write_task().read_text(encoding="utf-8"))
        wire = MOD.wire_task(task)
        self.assertEqual(wire["inputs"], ["inputs/input_doc.hwpx"])
        self.assertNotIn(self.tmp, json.dumps(wire))


# ---------------------------------------------------------------------------
# 2. 성공 경로 + 저널 실증
# ---------------------------------------------------------------------------

class SuccessPathTest(HandoffCase):
    def test_accepted_and_collected_and_journaled(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "success", AGENT_SUCCESS)
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--json"])
        self.assertEqual(rc, 0)
        self.assertEqual(env["status"], "ok")
        self.assertEqual(env["outcome"], "accepted")
        self.assertEqual(env["code"], 0)
        self.assertEqual(env["acceptedAgent"], "primary")
        # 반환 capability 는 명시적 스키마 + untrusted 표지
        self.assertEqual(env["capabilities"][0]["name"], "demo-convert")
        self.assertTrue(env["untrustedContent"])
        self.assertIn("result", env["untrustedFields"])
        # 수거는 sandbox 밖 collected/ 로, 해시와 함께
        self.assertEqual(len(env["collectedOutputs"]), 1)
        collected = Path(env["collectedOutputs"][0]["path"])
        self.assertTrue(collected.is_file())
        self.assertEqual(MOD.sha256_file(collected), env["collectedOutputs"][0]["sha256"])
        # 저널: attempt 레코드 + final 레코드, 입력·task·결과 지문 포함
        records = self.journal_records(work)
        self.assertEqual([r["event"] for r in records], ["attempt", "final"])
        att = records[0]
        self.assertEqual(att["category"], "accepted")
        self.assertEqual(att["taskSha256"], env["taskSha256"])
        self.assertIn("input_doc.hwpx", att["inputsSha256"])
        self.assertTrue(att["resultSha256"])
        self.assertEqual(att["nextAction"]["action"], "consume")
        self.assertEqual(records[1]["outcome"], "accepted")
        # 지문 체인 자기검증
        v = MOD.verify_journal(Path(work) / "handoff.journal.ndjson")
        self.assertTrue(v["chainValid"])
        self.assertEqual(v["entries"], 2)

    def test_journal_resumes_chain_across_runs(self):
        """같은 저널에 이어 쓰는 두 번째 실행이 연번·지문 체인을 끊지 않는다."""
        task_path = self.write_task()
        agent = make_agent(self.tmp, "success_resume", AGENT_SUCCESS)
        work = str(Path(self.tmp) / "work_resume")
        self.run_main(["--task", str(task_path), "--agent", agent,
                       "--work-dir", work, "--json"])
        self.run_main(["--task", str(task_path), "--agent", agent,
                       "--work-dir", work, "--json"])
        v = MOD.verify_journal(Path(work) / "handoff.journal.ndjson")
        self.assertTrue(v["chainValid"])
        self.assertEqual(v["entries"], 4)

    def test_journal_tamper_detected(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "success2", AGENT_SUCCESS)
        work = str(Path(self.tmp) / "work2")
        self.run_main(["--task", str(task_path), "--agent", agent,
                       "--work-dir", work, "--json"])
        jp = Path(work) / "handoff.journal.ndjson"
        lines = jp.read_text(encoding="utf-8").splitlines()
        first = json.loads(lines[0])
        first["category"] = "위조됨"
        lines[0] = json.dumps(first, ensure_ascii=False)
        jp.write_text("\n".join(lines) + "\n", encoding="utf-8")
        v = MOD.verify_journal(jp)
        self.assertFalse(v["chainValid"])
        self.assertEqual(v["brokenAt"], 2)
        rc, env = self.run_main(["--verify-journal", str(jp), "--json"])
        self.assertEqual(rc, 3)
        self.assertEqual(env["status"], "verdict")


# ---------------------------------------------------------------------------
# 3. 실패 → 재시도 / timeout / fallback
# ---------------------------------------------------------------------------

class RetryFallbackTest(HandoffCase):
    def test_transient_error_then_success_retries(self):
        task_path = self.write_task()
        state = str(Path(self.tmp) / "calls.txt")
        agent = make_agent(self.tmp, "flaky",
                           AGENT_FAIL_THEN_SUCCEED_TEMPLATE.format(state=state))
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--max-attempts", "3", "--json"])
        self.assertEqual(rc, 0)
        self.assertEqual(env["outcome"], "accepted")
        cats = [a["category"] for a in env["attempts"]]
        self.assertEqual(cats, ["agentError", "accepted"])
        self.assertEqual(env["attempts"][0]["nextAction"]["action"], "retry")

    def test_timeout_stops_on_no_progress(self):
        task_path = self.write_task(timeoutSec=1)
        agent = make_agent(self.tmp, "sleeper", AGENT_TIMEOUT)
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--max-attempts", "5", "--json"])
        self.assertEqual(rc, 1)
        self.assertEqual(env["status"], "error")
        self.assertEqual(env["code"], 1000)
        self.assertEqual(env["outcome"], "handoff")
        # 같은 시그니처 재발 즉시 중단 — 5회 한도라도 2회에서 멈춘다
        self.assertEqual(len(env["attempts"]), 2)
        self.assertEqual(env["nextAction"]["action"], "selfExecute")

    def test_fallback_agent_used_after_primary_exhausted(self):
        task_path = self.write_task()
        primary = make_agent(self.tmp, "always_error", AGENT_ALWAYS_ERROR)
        fallback = make_agent(self.tmp, "rescue", AGENT_SUCCESS)
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", primary,
                                 "--fallback-agent", fallback,
                                 "--work-dir", work, "--max-attempts", "3", "--json"])
        self.assertEqual(rc, 0)
        self.assertEqual(env["outcome"], "accepted")
        self.assertEqual(env["acceptedAgent"], "fallback")
        agents = [a["agent"] for a in env["attempts"]]
        # primary 는 결정적 동일 실패라 진전 판정으로 2회 만에 접는다
        self.assertEqual(agents, ["primary", "primary", "fallback"])
        self.assertEqual(env["attempts"][1]["nextAction"]["action"], "fallback")

    def test_agent_verdict_is_not_retried(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "verdict", AGENT_VERDICT)
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--max-attempts", "4", "--json"])
        self.assertEqual(rc, 3)
        self.assertEqual(env["outcome"], "handoff")
        self.assertEqual(len(env["attempts"]), 1)
        self.assertEqual(env["attempts"][0]["category"], "agentVerdict")


# ---------------------------------------------------------------------------
# 4. Security Boundary 위반 — 전부 거부(4000)이고 재시도하지 않는다
# ---------------------------------------------------------------------------

class SecurityBoundaryTest(HandoffCase):
    def _expect_security_reject(self, name: str, body: str, code: str):
        task_path = self.write_task()
        agent = make_agent(self.tmp, name, body)
        work = str(Path(self.tmp) / f"work_{name}")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--max-attempts", "3", "--json"])
        self.assertEqual(rc, 4)
        self.assertEqual(env["status"], "verdict")
        self.assertEqual(env["code"], 4000)
        self.assertEqual(env["outcome"], "rejected")
        self.assertIsNone(env["result"])
        self.assertEqual(env["collectedOutputs"], [])
        self.assertEqual(len(env["attempts"]), 1)  # boundary 위반은 재시도 금지
        att = env["attempts"][0]
        self.assertEqual(att["category"], "securityViolation")
        self.assertIn(code, [f["code"] for f in att["findings"]])
        # 저널에도 같은 판정이 남는다
        records = self.journal_records(work)
        self.assertEqual(records[0]["category"], "securityViolation")

    def test_writes_outside_out(self):
        self._expect_security_reject("stray", AGENT_WRITES_OUTSIDE_OUT, "wroteOutsideOut")

    def test_output_path_escape(self):
        self._expect_security_reject("escape", AGENT_OUTPUT_ESCAPE, "outputPathEscape")

    def test_undeclared_output(self):
        self._expect_security_reject("undeclared", AGENT_UNDECLARED_OUTPUT,
                                     "undeclaredOutput")

    def test_tool_not_allowed(self):
        self._expect_security_reject("badtool", AGENT_TOOL_NOT_ALLOWED, "toolNotAllowed")

    def test_input_copy_modified(self):
        self._expect_security_reject("tamper", AGENT_MODIFIES_INPUT, "inputModified")


# ---------------------------------------------------------------------------
# 5. 결과 스키마 위반과 completion / consistency 검증
# ---------------------------------------------------------------------------

class ResultValidationTest(HandoffCase):
    def test_schema_violation_rejected(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "badschema", AGENT_SCHEMA_VIOLATION)
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--json"])
        self.assertEqual(rc, 3)
        self.assertEqual(env["outcome"], "handoff")
        self.assertEqual(env["attempts"][0]["category"], "schemaViolation")
        codes = [f["code"] for f in env["attempts"][0]["findings"]]
        self.assertIn("badStatus", codes)
        self.assertIn("badOutputs", codes)

    def test_output_without_sha256_is_rejected(self):
        """수거한 산출물은 에이전트 선언 해시와 대조 가능한 경우에만 수용한다."""
        task_path = self.write_task()
        agent = make_agent(self.tmp, "no_sha256", AGENT_SUCCESS_WITHOUT_SHA256)
        work = str(Path(self.tmp) / "work_no_sha256")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--json"])
        self.assertEqual(rc, 3)
        self.assertEqual(env["outcome"], "handoff")
        self.assertEqual(env["attempts"][0]["category"], "schemaViolation")
        codes = [f["code"] for f in env["attempts"][0]["findings"]]
        self.assertIn("badOutputSha256", codes)

    def test_unparseable_stdout_is_runtime_error(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "garbage", AGENT_BAD_JSON)
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--json"])
        self.assertEqual(rc, 1)
        self.assertEqual(env["attempts"][0]["category"], "unparseableResult")

    def test_missing_expected_output_is_incomplete(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "lazy", AGENT_MISSING_EXPECTED)
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--work-dir", work, "--json"])
        self.assertEqual(rc, 3)
        self.assertEqual(env["attempts"][0]["category"], "incompleteResult")
        codes = [f["code"] for f in env["attempts"][0]["findings"]]
        self.assertIn("missingExpectedOutput", codes)

    def test_must_parse_reverified_with_rhwp(self):
        # 가짜 rhwp 가 exit 1 → 산출물이 열리지 않는다는 판정 (에이전트 보고 불신)
        task_path = self.write_task(
            expectedOutputs=[{"path": "converted.hwpx", "mustParse": True}])
        agent = make_agent(self.tmp, "success_mp", AGENT_SUCCESS)
        bad_bin = make_fake_rhwp(self.tmp, 1)
        work = str(Path(self.tmp) / "work_bad")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--bin", bad_bin, "--work-dir", work, "--json"])
        self.assertEqual(rc, 3)
        self.assertEqual(env["attempts"][0]["category"], "inconsistentResult")
        # 가짜 rhwp 가 exit 0 → 수용
        good_bin = make_fake_rhwp(self.tmp, 0)
        work2 = str(Path(self.tmp) / "work_good")
        rc2, env2 = self.run_main(["--task", str(task_path), "--agent", agent,
                                   "--bin", good_bin, "--work-dir", work2, "--json"])
        self.assertEqual(rc2, 0)
        self.assertEqual(env2["outcome"], "accepted")

    def test_must_parse_without_bin_is_not_accepted(self):
        task_path = self.write_task(
            expectedOutputs=[{"path": "converted.hwpx", "mustParse": True}])
        agent = make_agent(self.tmp, "success_nb", AGENT_SUCCESS)
        work = str(Path(self.tmp) / "work")
        env_backup = os.environ.pop("RHWP_BIN", None)
        try:
            rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                     "--work-dir", work, "--json"])
        finally:
            if env_backup is not None:
                os.environ["RHWP_BIN"] = env_backup
        self.assertEqual(rc, 3)
        codes = [f["code"] for f in env["attempts"][0]["findings"]]
        self.assertIn("unchecked", codes)


# ---------------------------------------------------------------------------
# 6. 수용 정책 — dar 정책 언어 재사용 (같은 언어, handoff 판정 키)
# ---------------------------------------------------------------------------

class PolicyTest(HandoffCase):
    def write_policy(self, obj) -> str:
        p = Path(self.tmp) / "policy.json"
        p.write_text(json.dumps(obj, ensure_ascii=False), encoding="utf-8")
        return str(p)

    def test_policy_rejects_accepted_result(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "success_pol", AGENT_SUCCESS)
        policy = self.write_policy({
            "kind": "admissionPolicy", "name": "fallback-만-신뢰",
            "rules": [{"id": "R1", "require": {"agent": {"eq": "fallback"}}}]})
        work = str(Path(self.tmp) / "work")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--policy", policy, "--work-dir", work, "--json"])
        self.assertEqual(rc, 4)
        self.assertEqual(env["code"], 4000)
        self.assertEqual(env["outcome"], "rejected")
        self.assertEqual(env["attempts"][0]["category"], "policyRejected")

    def test_policy_allows_matching_result(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "success_pol2", AGENT_SUCCESS)
        policy = self.write_policy({
            "kind": "admissionPolicy", "name": "검증 통과만",
            "rules": [{"id": "R1", "require": {"validated": {"eq": True},
                                               "violationCount": {"lte": 0}}}]})
        work = str(Path(self.tmp) / "work2")
        rc, env = self.run_main(["--task", str(task_path), "--agent", agent,
                                 "--policy", policy, "--work-dir", work, "--json"])
        self.assertEqual(rc, 0)
        self.assertEqual(env["outcome"], "accepted")

    def test_unknown_policy_key_is_usage_error(self):
        task_path = self.write_task()
        agent = make_agent(self.tmp, "success_pol3", AGENT_SUCCESS)
        policy = self.write_policy({
            "kind": "admissionPolicy", "name": "오타",
            "rules": [{"id": "R1", "require": {"vaildated": {"eq": True}}}]})
        rc, _ = self.run_main(["--task", str(task_path), "--agent", agent,
                               "--policy", policy, "--json"])
        self.assertEqual(rc, 2)

    def test_dar_default_judgment_keys_unchanged(self):
        """dar 의 기존 --policy 호출(인자 없이)이 그대로 동작해야 한다."""
        spec = importlib.util.spec_from_file_location(
            "dar_tx_for_handoff_test", REPO_ROOT / "tools" / "dar" / "transaction.py")
        assert spec and spec.loader
        dar = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(dar)
        pol = dar.parse_policy(json.dumps({
            "kind": "admissionPolicy", "name": "기존 계약",
            "rules": [{"id": "R1", "require": {"validated": {"eq": True}}}]}))
        allow, violations = dar.evaluate_policy(pol, {"validated": True})
        self.assertTrue(allow)
        self.assertEqual(violations, [])
        with self.assertRaises(ValueError):
            dar.parse_policy(json.dumps({
                "kind": "admissionPolicy", "name": "handoff 키는 기본 사전엔 없다",
                "rules": [{"id": "R1", "require": {"violationCount": {"lte": 0}}}]}))


if __name__ == "__main__":
    unittest.main()
