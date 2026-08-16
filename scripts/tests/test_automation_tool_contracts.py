"""자동화 도구의 입력·capability·원본 보존 계약 회귀 테스트."""

from __future__ import annotations

import base64
import contextlib
import importlib.util
import io
import json
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


class CapabilityContracts(unittest.TestCase):
    def test_fde_does_not_descend_ladder_when_capabilities_fail(self):
        tool = load_tool("tools/fde/triage.py")
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            doc = root / "input.hwp"
            doc.write_bytes(b"HWP Document File" + b"\0" * 32)
            binary = root / "rhwp"
            binary.write_text("placeholder", encoding="utf-8")
            output = root / "ticket.json"
            calls = []

            def failed_capability(*_args):
                calls.append(1)
                return {"command": "capabilities --json", "ok": False, "exitCode": 1}

            with patch.object(tool, "run_step", side_effect=failed_capability):
                self.assertEqual(tool.main([str(doc), "--bin", str(binary), "-o", str(output)]), 0)
            ticket = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(len(calls), 1)
        self.assertEqual(ticket["route"], "workaround")
        self.assertEqual(len(ticket["steps"]), 1)

    def test_chief_refuses_goal_when_capabilities_are_unknown(self):
        tool = load_tool("tools/chief/service_loop.py")
        chief = tool.Chief.__new__(tool.Chief)
        chief.available = None
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            result = chief.handle("export-text", root / "input.hwp", {}, root / "out", root)
        self.assertEqual(result["status"], "needs-agent")
        self.assertIn("capabilities", result["reason"])

    def test_engagement_stops_when_capabilities_cannot_be_read(self):
        tool = load_tool("tools/strategist/engagement.py")
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            corpus = root / "corpus"
            corpus.mkdir()
            (corpus / "input.hwp").write_bytes(b"sample")
            binary = root / "rhwp"
            binary.write_text("placeholder", encoding="utf-8")
            engagement = root / "engagement.json"
            engagement.write_text(json.dumps({
                "objective": "검토", "corpus": "corpus", "questions": ["무엇인가"],
            }), encoding="utf-8")
            out = root / "out"
            args = SimpleNamespace(engagement=str(engagement), bin=str(binary), out=str(out), timeout=1)
            with patch.object(tool, "advertised_commands", return_value=None):
                self.assertEqual(tool.run_engagement(args), 1)
            self.assertFalse((out / "corpus_map.json").exists())


class ChiefQueueContracts(unittest.TestCase):
    def test_request_paths_cannot_escape_request_directory(self):
        tool = load_tool("tools/chief/service_loop.py")
        with tempfile.TemporaryDirectory() as td:
            request = Path(td) / "request"
            request.mkdir()
            inside = request / "nested" / "input.hwp"
            inside.parent.mkdir()
            inside.write_bytes(b"doc")
            self.assertEqual(tool.resolve_request_file(request, "nested/input.hwp"), inside.resolve())
            self.assertIsNone(tool.resolve_request_file(request, "../outside.hwp"))
            self.assertIsNone(tool.resolve_request_file(request, "/etc/passwd"))

    def test_malformed_request_is_marked_complete_without_crashing_watch_loop(self):
        tool = load_tool("tools/chief/service_loop.py")
        with tempfile.TemporaryDirectory() as td:
            request = Path(td) / "request"
            request.mkdir()
            (request / "request.json").write_text("[not-an-object]", encoding="utf-8")
            result = tool.process_request(object(), request)
            persisted = json.loads((request / "result.json").read_text(encoding="utf-8"))
        self.assertEqual(result["status"], "failed")
        self.assertEqual(persisted["status"], "failed")


class DatpContracts(unittest.TestCase):
    def test_replace_text_proposal_requires_string_find_and_replace(self):
        tool = load_tool("tools/dar/transaction.py")
        self.assertIsNotNone(tool.validate_proposal("replace-text", {"find": "a"}))
        self.assertIsNotNone(tool.validate_proposal("replace-text", ["a", "b"]))
        self.assertIsNone(tool.validate_proposal("replace-text", {"find": "a", "replace": "b"}))

    def test_commit_never_overwrites_transaction_input(self):
        tool = load_tool("tools/dar/transaction.py")
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            source = root / "source.hwpx"
            source.write_bytes(b"original")
            tx = tool.Tx.create(root / "txs", source, "rhwp", "test")
            working = tx.path / "working.hwpx"
            working.write_bytes(b"changed")
            tx.state.update({
                "workingOutput": str(working), "validated": True,
                "proposal": {"op": "replace-text", "params": {"find": "a", "replace": "b"}},
                "operationSha256": "operation",
            })
            tx.save()
            args = SimpleNamespace(request_id="req", output=str(source), overwrite=True)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(tool.op_commit(args, tx), 2)
            self.assertEqual(source.read_bytes(), b"original")
            self.assertFalse((tx.path / "receipt.json").exists())

    def test_verify_records_terminal_state_from_receipt_hashes(self):
        tool = load_tool("tools/dar/transaction.py")
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            source = root / "source.hwpx"
            output = root / "output.hwpx"
            source.write_bytes(b"source")
            output.write_bytes(b"output")
            tx = tool.Tx.create(root / "txs", source, "rhwp", "test")
            tx.state.update({"current": "COMMIT", "history": ["BEGIN", "COMMIT"], "committed": True})
            tx.save()
            receipt = {
                "transactionId": tx.state["transactionId"], "input": str(source),
                "inputSha256": tool.sha256_file(source), "output": str(output),
                "outputSha256": tool.sha256_file(output),
                "proposal": {"op": "replace-text", "params": {"find": "a", "replace": "b"}},
            }
            (tx.path / "receipt.json").write_text(json.dumps(receipt), encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(tool.op_verify(SimpleNamespace(request_id="req"), tx), 0)
            self.assertEqual(tx.state["current"], "VERIFY")

    def test_malformed_receipt_proposal_returns_envelope_error(self):
        tool = load_tool("tools/dar/transaction.py")
        with tempfile.TemporaryDirectory() as td:
            receipt = Path(td) / "receipt.json"
            receipt.write_text(json.dumps({
                "transactionId": "tx_test", "input": "input.hwpx", "inputSha256": "a" * 64,
                "output": "output.hwpx", "outputSha256": "b" * 64,
                "proposal": {"op": "replace-text"},
            }), encoding="utf-8")
            loaded, error = tool.load_receipt(receipt, "req")
        self.assertIsNone(loaded)
        self.assertEqual(error["code"], 2000)
        self.assertIn("proposal", error["record"]["reason"])


if __name__ == "__main__":
    unittest.main()
