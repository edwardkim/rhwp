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

    def test_advisory_remains_manual_and_non_required(self) -> None:
        active_triggers = self.workflow.split("permissions:\n", maxsplit=1)[0]
        self.assertIn("on:\n  workflow_dispatch:\n", active_triggers)
        self.assertNotIn("\non:\n  pull_request:\n", active_triggers)
        self.assertIn("continue-on-error: true", self.workflow)
        self.assertIn("이 잡은 advisory 이며 required check 가 아닙니다.", self.workflow)


if __name__ == "__main__":
    unittest.main()
