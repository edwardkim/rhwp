from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "check_pdf_repository_policy.py"
SPEC = importlib.util.spec_from_file_location("check_pdf_repository_policy", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
POLICY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = POLICY
SPEC.loader.exec_module(POLICY)


class PdfRepositoryPolicyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        subprocess.run(["git", "config", "user.email", "policy@example.invalid"], cwd=self.root, check=True)
        subprocess.run(["git", "config", "user.name", "Policy Test"], cwd=self.root, check=True)
        (self.root / "pdf").mkdir()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_pdf(self, relative: str = "pdf/example-2024.pdf") -> Path:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\n%%EOF\n")
        return path

    def rules(self) -> set[str]:
        _files, violations = POLICY.evaluate(self.root)
        return {item.rule for item in violations}

    def test_accepts_real_pdf_as_plain_git_blob(self) -> None:
        path = self.write_pdf()
        subprocess.run(["git", "add", path.relative_to(self.root).as_posix()], cwd=self.root, check=True)
        files, violations = POLICY.evaluate(self.root)
        self.assertEqual([path], files)
        self.assertEqual([], violations)

    def test_rejects_retired_top_level_oracle_root(self) -> None:
        self.write_pdf()
        (self.root / "pdf-large").mkdir()
        self.assertIn("retired-root", self.rules())

    def test_rejects_worktree_lfs_pointer(self) -> None:
        path = self.root / "pdf" / "pointer.pdf"
        path.write_bytes(
            POLICY.LFS_POINTER_HEADER
            + b"oid sha256:" + b"0" * 64 + b"\nsize 42\n"
        )
        self.assertIn("lfs-pointer", self.rules())

    def test_rejects_non_pdf_bytes(self) -> None:
        (self.root / "pdf" / "broken.pdf").write_text("not a PDF", encoding="utf-8")
        self.assertIn("pdf-magic", self.rules())

    def test_rejects_hydrated_worktree_with_pointer_in_index(self) -> None:
        path = self.root / "pdf" / "indexed.pdf"
        path.write_bytes(
            POLICY.LFS_POINTER_HEADER
            + b"oid sha256:" + b"1" * 64 + b"\nsize 42\n"
        )
        subprocess.run(["git", "add", "pdf/indexed.pdf"], cwd=self.root, check=True)
        path.write_bytes(b"%PDF-1.4\n%%EOF\n")
        self.assertIn("lfs-index-pointer", self.rules())

    def test_rejects_lfs_attribute(self) -> None:
        self.write_pdf()
        (self.root / ".gitattributes").write_text(
            "pdf/**/*.pdf filter=lfs diff=lfs merge=lfs -text\n",
            encoding="utf-8",
        )
        self.assertIn("lfs-attribute", self.rules())

    def test_rejects_exactly_fifty_mib(self) -> None:
        path = self.write_pdf()
        with path.open("r+b") as stream:
            stream.truncate(POLICY.MAX_PDF_BYTES_EXCLUSIVE)
        self.assertIn("size-limit", self.rules())


if __name__ == "__main__":
    unittest.main()
