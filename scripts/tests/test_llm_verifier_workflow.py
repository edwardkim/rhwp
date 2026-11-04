"""LLM verifier의 선택적 CI 제외와 나머지 도구의 실행 계약을 고정한다."""

from __future__ import annotations

import re
import shlex
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = REPO_ROOT / ".github/workflows/ci.yml"
EXCLUDED_PACKAGE = "llm-verifier-verdict-protocol"


def step_body(workflow: str, name: str) -> str:
    marker = f"      - name: {name}\n"
    if marker not in workflow:
        raise AssertionError(f"workflow에 {name} step이 없다")
    body = workflow.split(marker, maxsplit=1)[1]
    boundary = re.search(r"(?m)^(?:      - |  [A-Za-z0-9_-]+:)", body)
    return body[: boundary.start()] if boundary else body


def cargo_commands(workflow: str) -> list[list[str]]:
    # YAML block scalar의 shell continuation을 실제 명령 인자로 복원한다.
    joined = re.sub(r"\\\n\s*", " ", workflow)
    return [
        shlex.split(match.group(1))
        for match in re.finditer(r"(?m)^\s*(?:run: )?(cargo [^\n]+)", joined)
    ]


def excluded_packages(command: list[str]) -> list[str]:
    return [command[index + 1] for index, arg in enumerate(command) if arg == "--exclude"]


class LlmVerifierWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ci = CI_WORKFLOW.read_text(encoding="utf-8")

    def test_lint_runs_all_standalone_verifier_contracts(self) -> None:
        lint_job = re.search(
            r"(?ms)^  lint:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)", self.ci
        )
        self.assertIsNotNone(lint_job, "ci.yml 에 lint job 이 없다")
        body = lint_job.group(0) if lint_job else ""
        self.assertIn("- name: Validate LLM verifier tool contracts", body)
        self.assertIn("export PYTHONPATH=tools/llm_verifier", body)
        contracts = step_body(body, "Validate LLM verifier tool contracts")
        self.assertNotIn("if:", contracts)
        self.assertNotIn("continue-on-error:", contracts)
        for command in [
            "tools/llm_verifier/third_party_replay/tests",
            "tools/llm_verifier/shadow_agree/tests",
            "tools/llm_verifier/untrusted_sandbox/tests",
            "tools/llm_verifier/lineage_chain/tests",
            "tools/llm_verifier/abstain/tests",
            "cargo test --manifest-path tools/llm_verifier/repeat_eval/Cargo.toml",
            "cargo test --manifest-path tools/llm_verifier/tool_version_gate/Cargo.toml",
        ]:
            self.assertIn(command, contracts)

    def test_internal_crates_exclude_only_the_verdict_protocol_in_addition_to_existing_tools(self) -> None:
        body = step_body(self.ci, "Test internal Rust crates")
        commands = cargo_commands(body)
        self.assertEqual(len(commands), 1)
        command = commands[0]
        self.assertEqual(command[:3], ["cargo", "test", "--workspace"])
        self.assertIn("--lib", command)
        self.assertCountEqual(
            excluded_packages(command),
            ["rhwp", "rhwp-subsecond", "rhwp-native-ffi", "batch-convert", EXCLUDED_PACKAGE],
        )
        self.assertNotIn("if:", body)
        self.assertNotIn("continue-on-error:", body)

    def test_workspace_build_and_clippy_exclude_only_the_verdict_protocol(self) -> None:
        body = step_body(self.ci, "Check workspace members (FFI bindings, tools)")
        commands = cargo_commands(body)
        self.assertEqual([command[:2] for command in commands], [["cargo", "build"], ["cargo", "clippy"]])
        for command in commands:
            self.assertIn("--workspace", command)
            self.assertEqual(excluded_packages(command), [EXCLUDED_PACKAGE])
        self.assertIn("--all-targets", commands[1])
        self.assertEqual(commands[1][-3:], ["--", "-D", "warnings"])
        self.assertNotIn("if:", body)
        self.assertNotIn("continue-on-error:", body)

    def test_all_workflow_workspace_build_and_test_commands_keep_the_exclusion(self) -> None:
        checked = 0
        for path in sorted((REPO_ROOT / ".github/workflows").glob("*.yml")):
            for command in cargo_commands(path.read_text(encoding="utf-8")):
                if "--workspace" in command:
                    with self.subTest(workflow=path.name, command=command):
                        self.assertIn(EXCLUDED_PACKAGE, excluded_packages(command))
                    checked += 1
        self.assertEqual(checked, 3)

    def test_workflows_do_not_invoke_the_verdict_protocol_directly(self) -> None:
        for path in sorted((REPO_ROOT / ".github").rglob("*.yml")):
            for command in cargo_commands(path.read_text(encoding="utf-8")):
                with self.subTest(workflow=str(path.relative_to(REPO_ROOT)), command=command):
                    selected = [
                        arg for index, arg in enumerate(command)
                        if index == 0 or command[index - 1] != "--exclude"
                    ]
                    self.assertNotIn(EXCLUDED_PACKAGE, selected)
                    self.assertFalse(any("verdict_protocol/Cargo.toml" in arg for arg in selected))

    def test_product_clippy_and_archive_package_selection_are_preserved(self) -> None:
        self.assertIn("run: cargo clippy -- -D warnings", step_body(self.ci, "Clippy"))
        self.assertIn(
            "cargo clippy -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings",
            step_body(self.ci, "Clippy (WASM32)"),
        )
        archive = (REPO_ROOT / ".github/workflows/build-nextest-archives.yml").read_text(encoding="utf-8")
        commands = cargo_commands(archive)
        nextest = [command for command in commands if command[:2] == ["cargo", "nextest"]]
        self.assertEqual(len(nextest), 2)
        for command in nextest:
            self.assertEqual(command[command.index("--package") + 1], "rhwp")


if __name__ == "__main__":
    unittest.main()
