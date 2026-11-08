"""Publication recovery decisions use public immutable version checksums."""

import gzip
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

from scripts.crates_io_publish_preflight import (
    CheckError, audit_archives, dependency_order, disposition, output_path, public_json, registry_state,
)


class PublishPreflightTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.sha = "a" * 40
        self.policy = {"packages": [{"published_name": "leaf", "version": "0.1.0",
                                     "local_dependencies": []}]}
        self.manifest = ('[package]\nname="leaf"\nversion="0.1.0"\n'
                         'publish=["crates-io"]\nrust-version="1.93.1"\n')
        self.vcs = {"git": {"sha1": self.sha}}

    def archive(self, extra=None):
        path = self.directory / "leaf-0.1.0.crate"
        files = {"leaf-0.1.0/Cargo.toml": self.manifest,
                 "leaf-0.1.0/.cargo_vcs_info.json": json.dumps(self.vcs)}
        files.update(extra or {})
        with tarfile.open(path, "w:gz") as archive:
            for name, content in files.items():
                raw = content.encode()
                member = tarfile.TarInfo(name)
                member.size = len(raw)
                archive.addfile(member, io.BytesIO(raw))
        self.row = {"name": "leaf", "version": "0.1.0", "bytes": path.stat().st_size,
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        return {"source_sha": self.sha, "packages": [self.row]}

    def audit(self, ledger):
        return audit_archives(ledger, self.policy, self.directory)

    def test_immutable_archive_identity(self):
        self.assertEqual(self.audit(self.archive())["leaf"], self.row)

    def test_archive_tampering(self):
        ledger = self.archive()
        (self.directory / "leaf-0.1.0.crate").write_bytes(b"changed")
        with self.assertRaisesRegex(CheckError, "size differs"):
            self.audit(ledger)

    def replace_archive_bytes(self, ledger, transform):
        path = self.directory / "leaf-0.1.0.crate"
        raw = transform(path.read_bytes())
        path.write_bytes(raw)
        ledger["packages"][0].update(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())

    def test_stale_output_tail_is_rejected_even_with_matching_checksum(self):
        ledger = self.archive()
        self.replace_archive_bytes(ledger, lambda raw: raw + b"\x04=previous archive tail")
        with self.assertRaisesRegex(CheckError, "trailing gzip"):
            self.audit(ledger)

    def test_second_gzip_member_is_not_part_of_a_single_crate(self):
        ledger = self.archive()
        self.replace_archive_bytes(ledger, lambda raw: raw + gzip.compress(b"old stream"))
        with self.assertRaisesRegex(CheckError, "trailing gzip"):
            self.audit(ledger)

    def test_gzip_footer_is_required_even_when_tar_entries_are_complete(self):
        ledger = self.archive()
        self.replace_archive_bytes(ledger, lambda raw: raw[:-8])
        with self.assertRaisesRegex(CheckError, "incomplete gzip"):
            self.audit(ledger)

    def test_gzip_checksum_is_verified_beyond_the_tar_end(self):
        ledger = self.archive()
        self.replace_archive_bytes(ledger, lambda raw: raw[:-8] + bytes([raw[-8] ^ 1]) + raw[-7:])
        with self.assertRaisesRegex(CheckError, "invalid gzip"):
            self.audit(ledger)

    def test_equal_size_wrong_digest(self):
        ledger = self.archive()
        ledger["packages"][0]["sha256"] = "0" * 64
        with self.assertRaisesRegex(CheckError, "checksum differs"):
            self.audit(ledger)

    def test_source_mismatch_and_dirty(self):
        for vcs in [{"git": {"sha1": "b" * 40}},
                    {"git": {"sha1": self.sha, "dirty": True}}]:
            self.vcs = vcs
            with self.assertRaisesRegex(CheckError, "VCS source differs"):
                self.audit(self.archive())

    def test_wrong_version_and_msrv_and_guard(self):
        for before, after in [('version="0.1.0"', 'version="0.2.0"'),
                              ('1.93.1', '1.88'),
                              ('publish=["crates-io"]', 'publish=false')]:
            original = self.manifest
            self.manifest = original.replace(before, after)
            with self.assertRaises(CheckError):
                self.audit(self.archive())
            self.manifest = original

    def test_package_set_mismatch(self):
        ledger = self.archive()
        ledger["packages"].append(self.row)
        with self.assertRaisesRegex(CheckError, "count differs"):
            self.audit(ledger)

    def test_path_escape(self):
        with self.assertRaisesRegex(CheckError, "unsafe archive entry"):
            self.audit(self.archive({"leaf-0.1.0/../escape": "bad"}))

    def test_dependency_order_and_missing_cycle(self):
        specs = [{"published_name": "parent", "local_dependencies": ["leaf"]},
                 self.policy["packages"][0]]
        self.assertEqual(dependency_order(specs), ["leaf", "parent"])
        specs[1]["local_dependencies"] = ["parent"]
        with self.assertRaisesRegex(CheckError, "cyclic or unknown"):
            dependency_order(specs)

    def state(self, **changes):
        result = {"status": "exists", "owners": ["edwardkim"],
                  "versions": [{"version": "0.1.0", "checksum": self.row["sha256"],
                                "yanked": False}]}
        result.update(changes)
        return result

    def test_first_publication_does_not_claim_ownership(self):
        self.archive()
        self.assertEqual(disposition(self.row, {"status": "not-found"}, "edwardkim"),
                         "first-publication-pending-approval")

    def test_recovery_skips_only_matching_existing_version(self):
        self.archive()
        self.assertEqual(disposition(self.row, self.state(), "edwardkim"), "already-present")
        self.assertEqual(disposition(self.row, self.state(versions=[]), "edwardkim"),
                         "new-version-pending-approval")

    def test_foreign_owner_checksum_yank_never_skipped(self):
        self.archive()
        cases = [self.state(owners=["other"]),
                 self.state(versions=[{"version": "0.1.0", "checksum": "0" * 64, "yanked": False}]),
                 self.state(versions=[{"version": "0.1.0", "checksum": self.row["sha256"], "yanked": True}])]
        for state in cases:
            with self.assertRaises(CheckError):
                disposition(self.row, state, "edwardkim")

    def test_http_failure_does_not_mean_available(self):
        for code in [403, 429, 500]:
            with patch("urllib.request.urlopen", side_effect=urllib.error.HTTPError(
                    "https://crates.io", code, "failed", None, None)):
                with self.assertRaisesRegex(CheckError, str(code)):
                    public_json("https://crates.io")
        with patch("urllib.request.urlopen", side_effect=urllib.error.HTTPError(
                "https://crates.io", 404, "missing", None, None)):
            self.assertIsNone(public_json("https://crates.io"))

    def test_missing_owners_is_failure(self):
        with patch("scripts.crates_io_publish_preflight.public_json", side_effect=[{}, None]):
            with self.assertRaisesRegex(CheckError, "owners unavailable"):
                registry_state("leaf")

    def test_output_parent_symlink_cannot_escape(self):
        root = self.directory / "repo"
        (root / "output").mkdir(parents=True)
        outside = self.directory / "outside"
        outside.mkdir()
        (root / "output/redirect").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(CheckError, "output must"):
            output_path(root, root / "output/redirect/report.json")
        self.assertEqual(output_path(root, root / "output/report.json"), root / "output/report.json")


if __name__ == "__main__":
    unittest.main()
