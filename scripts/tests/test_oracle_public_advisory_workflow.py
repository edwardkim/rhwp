from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


WORKFLOW_PATH = (
    Path(__file__).resolve().parents[2]
    / ".github"
    / "workflows"
    / "oracle-public-advisory.yml"
)
DOCUMENT_COMMANDS_PATH = (
    Path(__file__).resolve().parents[2]
    / "src"
    / "document_core"
    / "commands"
    / "document.rs"
)
BLANK_TEMPLATE_PATH = Path(__file__).resolve().parents[2] / "saved" / "blank2010.hwp"
MCP_SERVE_PATH = Path(__file__).resolve().parents[2] / "src" / "mcp_serve.rs"
MCP_DOCUMENT_PATHS = (
    "mydocs/manual/agent_knowledge_map.md",
    "mydocs/manual/agent_troubleshooting_guide.md",
    "mydocs/manual/recipes/01_fill_form_and_submit.md",
    "mydocs/manual/recipes/02_table_csv_roundtrip.md",
    "mydocs/manual/recipes/03_redact_before_sharing.md",
    "mydocs/manual/recipes/04_safety_check_untrusted_doc.md",
    "mydocs/manual/recipes/05_mail_merge_batch_fill.md",
    "mydocs/manual/recipes/06_visual_regression_before_after.md",
    "mydocs/tech/agent_roadmap/atlas_r1_r200.md",
    "gym/README.md",
)


class OraclePublicAdvisoryWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def gate_script(self) -> str:
        step = self.workflow.split(
            "      - name: Gate on real oracle PDFs\n", maxsplit=1
        )[1].split("      - name: Skip note\n", maxsplit=1)[0]
        embedded = step.split("          python3 - <<'PY'\n", maxsplit=1)[1].split(
            "\n          PY", maxsplit=1
        )[0]
        return textwrap.dedent(embedded)

    def test_gate_embedded_python_executes_and_writes_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pdf_root = root / "pdf"
            pdf_root.mkdir()
            (pdf_root / "example-2024.pdf").write_bytes(b"%PDF-1.4\n%%EOF\n")
            runner = root / "tools" / "oracle_public" / "page_smoke.py"
            runner.parent.mkdir(parents=True)
            runner.write_text("# fixture\n", encoding="utf-8")
            output = root / "github-output.txt"
            env = os.environ.copy()
            env["GITHUB_OUTPUT"] = str(output)

            completed = subprocess.run(
                [sys.executable, "-c", self.gate_script()],
                cwd=root,
                env=env,
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            fields = dict(
                line.split("=", maxsplit=1)
                for line in output.read_text(encoding="utf-8").splitlines()
            )
            self.assertEqual("1", fields["real_pdfs"])
            self.assertEqual("0", fields["lfs_pointers"])
            self.assertEqual("true", fields["runner"])
            self.assertEqual("ok", fields["reason"])
            self.assertEqual("true", fields["should_run"])

    def test_checkout_and_compare_use_only_the_canonical_pdf_root(self) -> None:
        self.assertNotIn("include_large:", self.workflow)
        self.assertNotIn("pdf-2020", self.workflow)
        self.assertNotIn("pdf-large", self.workflow)
        self.assertIn('roots = [Path("pdf")]', self.workflow)
        self.assertIn("--pdf-dirs pdf", self.workflow)

    def test_sparse_checkout_includes_out_of_tree_production_inputs(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        document_source = DOCUMENT_COMMANDS_PATH.read_text(encoding="utf-8")
        self.assertIn(
            'include_bytes!("../../../saved/blank2010.hwp")',
            document_source,
        )
        self.assertTrue(BLANK_TEMPLATE_PATH.is_file())

        mcp_source = MCP_SERVE_PATH.read_text(encoding="utf-8")
        self.assertIn('include_str!("../llms.txt")', mcp_source)
        self.assertTrue((repo_root / "llms.txt").is_file())
        for relative in MCP_DOCUMENT_PATHS:
            self.assertIn(f'include_str!("../{relative}")', mcp_source)
            self.assertTrue((repo_root / relative).is_file(), relative)

        checkout = self.workflow.split(
            "      - name: Check out sparse oracle tree\n", maxsplit=1
        )[1].split("      - name: Gate on real oracle PDFs\n", maxsplit=1)[0]
        checkout_entries = {line.strip() for line in checkout.splitlines()}
        required_entries = {"saved/blank2010.hwp", *MCP_DOCUMENT_PATHS}
        self.assertTrue(required_entries.issubset(checkout_entries))

    def test_advisory_has_narrow_bootstrap_and_remains_non_required(self) -> None:
        active_triggers = self.workflow.split("permissions:\n", maxsplit=1)[0]
        self.assertIn(
            "on:\n"
            "  # workflow_dispatch identity가 아직 기본 브랜치에 등록되지 않은 후보도\n"
            "  # workflow 파일 자체를 바꾼 신뢰 push에서 한 번 실실행한다.\n"
            "  push:\n"
            "    branches:\n"
            "      - devel\n"
            "      - 'task_m100_*'\n"
            "    paths:\n"
            "      - '.github/workflows/oracle-public-advisory.yml'\n"
            "  workflow_dispatch:\n",
            active_triggers,
        )
        self.assertNotIn("\n  pull_request:\n", active_triggers)
        self.assertNotIn("\n      - main\n", active_triggers)
        self.assertIn("TOP_N: ${{ inputs.top_n || '10' }}", self.workflow)
        self.assertIn("LIMIT: ${{ inputs.limit || '0' }}", self.workflow)
        self.assertIn("continue-on-error: true", self.workflow)
        self.assertIn("이 잡은 advisory 이며 required check 가 아닙니다.", self.workflow)


if __name__ == "__main__":
    unittest.main()
