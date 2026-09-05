"""자동화 도구의 입력·capability·원본 보존 계약 회귀 테스트."""

from __future__ import annotations

import base64
import contextlib
import importlib.util
import io
import json
import shutil
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch


REPO_ROOT = Path(__file__).resolve().parents[2]


def load_tool(relative: str):
    path = REPO_ROOT / relative
    name = "automation_tool_" + relative.replace("/", "_").replace(".", "_")
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


class GhNoCloneContracts(unittest.TestCase):
    def test_subcommand_repo_and_full_log_are_parsed(self):
        tool = load_tool("tools/gh_noclone.py")
        read = tool.build_parser().parse_args(
            ["read", "README.md", "--ref", "devel", "--repo", "owner/repo"]
        )
        log = tool.build_parser().parse_args(["ci-log", "123"])
        self.assertEqual(read.repo, "owner/repo")
        self.assertFalse(log.failed_only)

    def test_contents_reads_use_get_even_with_ref(self):
        tool = load_tool("tools/gh_noclone.py")
        response = SimpleNamespace(
            returncode=0,
            stdout=base64.b64encode(b"ok\n").decode("ascii"),
            stderr="",
        )
        args = SimpleNamespace(repo="owner/repo", path="README.md", ref="devel", out=None)
        with patch.object(tool, "run_gh", return_value=response) as mocked:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(tool.cmd_read(args), 0)
        self.assertIn("GET", mocked.call_args.args[0])


class FileSafetyContracts(unittest.TestCase):
    def test_minimizer_rejects_output_equal_to_input(self):
        tool = load_tool("tools/crash_minimizer.py")
        with tempfile.TemporaryDirectory() as td:
            source = Path(td) / "source.hwpx"
            with zipfile.ZipFile(source, "w") as archive:
                archive.writestr("mimetype", "application/hwp+zip")
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                code = tool.main([str(source), "--oracle", "not-used {doc}", "-o", str(source)])
        self.assertEqual(code, 2)
        self.assertIn("덮어쓸 수 없다", stderr.getvalue())

    def test_sparse_apply_requires_existing_sparse_checkout(self):
        tool = load_tool("tools/sparse_clone_hint.py")
        with patch.object(tool, "get_current", return_value=None):
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(tool.main(["--task", "parser", "--apply"]), 2)


if __name__ == "__main__":
    unittest.main()
