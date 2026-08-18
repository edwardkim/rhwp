#!/usr/bin/env python3
"""M-rcpt work_receipt 픽스처·계약 시험."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

TOOL_DIR = Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

import catalog as cat  # noqa: E402
import contracts as C  # noqa: E402
import fatten_work_receipt as fatten  # noqa: E402


class ContractClassifyTests(unittest.TestCase):
    def test_attest_is_exit_0_mode_attest(self) -> None:
        code, mode = C.classify_replay(
            has_plan=True,
            plan_parse_ok=True,
            has_input=True,
            sign_key=False,
            capsule=False,
            same_file=False,
            expect=None,
            reproduced=None,
        )
        self.assertEqual((code, mode), (0, "attest"))

    def test_verify_match_and_mismatch(self) -> None:
        ok, mode = C.classify_replay(
            has_plan=True,
            plan_parse_ok=True,
            has_input=True,
            sign_key=False,
            capsule=False,
            same_file=False,
            expect="a" * 64,
            reproduced=True,
        )
        self.assertEqual((ok, mode), (0, "verify"))
        bad, mode = C.classify_replay(
            has_plan=True,
            plan_parse_ok=True,
            has_input=True,
            sign_key=False,
            capsule=False,
            same_file=False,
            expect=C.ZERO64,
            reproduced=False,
        )
        self.assertEqual((bad, mode), (3, "verify"))

    def test_expect_not_hex_is_usage_before_engine(self) -> None:
        code, mode = C.classify_replay(
            has_plan=True,
            plan_parse_ok=True,
            has_input=True,
            sign_key=False,
            capsule=False,
            same_file=False,
            expect="xyz",
            reproduced=None,
        )
        self.assertEqual((code, mode), (2, "usage"))
        self.assertEqual(C.classify_expect_hash("g" * 64)[0], 2)
        self.assertEqual(C.classify_expect_hash("A" * 64)[0], 0)

    def test_sign_key_requires_capsule(self) -> None:
        code, mode = C.classify_replay(
            has_plan=True,
            plan_parse_ok=True,
            has_input=True,
            sign_key=True,
            capsule=False,
            same_file=False,
            expect=None,
            reproduced=None,
        )
        self.assertEqual((code, mode), (2, "usage"))

    def test_same_file_parent_rejected(self) -> None:
        code, _ = C.classify_replay(
            has_plan=True,
            plan_parse_ok=True,
            has_input=True,
            sign_key=False,
            capsule=True,
            same_file=True,
            expect=None,
            reproduced=None,
        )
        self.assertEqual(code, 2)

    def test_io_and_engine(self) -> None:
        self.assertEqual(
            C.classify_replay(
                has_plan=True,
                plan_parse_ok=True,
                has_input=True,
                sign_key=False,
                capsule=False,
                same_file=False,
                expect=None,
                reproduced=None,
                io_error=True,
            )[0],
            1,
        )
        self.assertEqual(
            C.classify_replay(
                has_plan=True,
                plan_parse_ok=True,
                has_input=True,
                sign_key=False,
                capsule=False,
                same_file=False,
                expect=None,
                reproduced=None,
                engine_fail=True,
            )[0],
            1,
        )

    def test_audit_empty_is_usage_not_rate_zero(self) -> None:
        self.assertEqual(C.classify_audit(dir_exists=True, total=0, failed=0), 2)
        with self.assertRaises(ValueError):
            C.audit_rate(0, 0)
        self.assertEqual(C.audit_rate(2, 3), 2 / 3)
        self.assertEqual(C.classify_audit(dir_exists=True, total=3, failed=1), 3)
        self.assertEqual(C.classify_audit(dir_exists=False, total=0, failed=0), 1)

    def test_lineage_io_vs_judgment(self) -> None:
        self.assertEqual(
            C.classify_lineage(has_head_arg=False, head_readable=False, valid=False),
            2,
        )
        self.assertEqual(
            C.classify_lineage(has_head_arg=True, head_readable=False, valid=False),
            1,
        )
        self.assertEqual(
            C.classify_lineage(has_head_arg=True, head_readable=True, valid=False),
            3,
        )
        self.assertEqual(
            C.classify_lineage(has_head_arg=True, head_readable=True, valid=True),
            0,
        )

    def test_chronicle_invariant(self) -> None:
        self.assertTrue(C.lineage_ok("aa", "aa"))
        self.assertFalse(C.lineage_ok("aa", "bb"))
        self.assertTrue(C.parent_ok("aa", "aa"))
        self.assertFalse(C.parent_ok("aa", "bb"))


class CapsuleValidationTests(unittest.TestCase):
    def _ok_capsule(self) -> dict:
        scenario = cat.scenario_by_id("notice_year")
        return fatten.make_capsule(
            scenario, parent=None, parent_bytes=None, parent_rel=None
        )

    def test_good_capsule_returns_plan_and_steps(self) -> None:
        capsule = self._ok_capsule()
        result = C.validated_capsule_plan(capsule)
        self.assertIsInstance(result, tuple)
        plan, steps = result
        self.assertEqual(steps, 1)
        self.assertEqual(plan["steps"][0]["action"], "replace_text")

    def test_plan_vs_text(self) -> None:
        capsule = fatten.apply_tamper(self._ok_capsule(), "plan")
        self.assertEqual(C.validated_capsule_plan(capsule), C.NEEDLE["audit_plan_vs_text"])

    def test_plan_text_sha(self) -> None:
        capsule = fatten.apply_tamper(self._ok_capsule(), "plan_text")
        self.assertEqual(C.validated_capsule_plan(capsule), C.NEEDLE["audit_plan_text_sha"])

    def test_steps(self) -> None:
        capsule = fatten.apply_tamper(self._ok_capsule(), "steps")
        self.assertEqual(C.validated_capsule_plan(capsule), C.NEEDLE["audit_steps"])

    def test_missing_plan_text(self) -> None:
        capsule = fatten.apply_tamper(self._ok_capsule(), "missing_plan_text")
        self.assertEqual(
            C.validated_capsule_plan(capsule), C.NEEDLE["audit_plan_text_missing"]
        )

    def test_missing_plan_sha(self) -> None:
        capsule = fatten.apply_tamper(self._ok_capsule(), "missing_plan_sha")
        err = C.validated_capsule_plan(capsule)
        self.assertIsInstance(err, str)
        self.assertIn("planSha256", err)

    def test_plan_sha_is_utf8_of_plan_text(self) -> None:
        capsule = self._ok_capsule()
        self.assertEqual(
            capsule["receipt"]["planSha256"], C.sha256_hex(capsule["planText"])
        )
        self.assertTrue(C.is_sha256_hex(capsule["receipt"]["inputSha256"]))
        self.assertTrue(C.is_sha256_hex(capsule["receipt"]["outputSha256"]))


class CatalogIntegrityTests(unittest.TestCase):
    def test_scenario_idents_unique(self) -> None:
        idents = [s.ident for s in cat.SCENARIOS]
        self.assertEqual(len(idents), len(set(idents)))
        self.assertGreaterEqual(len(idents), 64)

    def test_exception_idents_unique(self) -> None:
        idents = [e.ident for e in cat.EXCEPTIONS]
        self.assertEqual(len(idents), len(set(idents)))
        self.assertGreaterEqual(len(idents), 30)

    def test_every_plan_action_appears(self) -> None:
        seen = {step["action"] for s in cat.SCENARIOS for step in s.steps}
        self.assertEqual(seen, set(C.PLAN_ACTIONS))

    def test_replace_find_never_empty(self) -> None:
        for scenario in cat.SCENARIOS:
            for step in scenario.steps:
                if step["action"] == "replace_text":
                    self.assertGreater(len(step["find"]), 0, scenario.ident)

    def test_set_cell_is_single_line(self) -> None:
        for scenario in cat.SCENARIOS:
            for step in scenario.steps:
                if step["action"] == "set_cell":
                    self.assertNotRegex(step["text"], r"[\r\n\t]", scenario.ident)

    def test_exception_needles_are_known(self) -> None:
        known = set(C.NEEDLE.values())
        for spec in cat.EXCEPTIONS:
            if spec.needle:
                self.assertTrue(
                    any(spec.needle == n or spec.needle in n or n in spec.needle for n in known)
                    or spec.ident.endswith("mismatch")
                    or spec.ident.endswith("rate")
                    or spec.ident.endswith("tamper")
                    or spec.ident.endswith("invariant"),
                    spec.ident,
                )

    def test_audit_rate_matches_members(self) -> None:
        for layout in cat.AUDIT_LAYOUTS:
            self.assertEqual(layout.total, len(layout.members), layout.ident)
            self.assertLessEqual(layout.reproduced, layout.total)
            if layout.total == 0:
                self.assertEqual(layout.exit, 2)
            elif layout.reproduced < layout.total:
                self.assertEqual(layout.exit, 3)
            else:
                self.assertEqual(layout.exit, 0)

    def test_lineage_exit_matches_valid(self) -> None:
        for topo in cat.LINEAGE:
            if topo.ident == "usage":
                self.assertEqual(topo.exit, 2)
            elif topo.ident == "head-missing":
                self.assertEqual(topo.exit, 1)
            elif topo.valid:
                self.assertEqual(topo.exit, 0)
            else:
                self.assertEqual(topo.exit, 3)


class GeneratedFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        cls.bundle = fatten.build(cls.root)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_writes_schemas_and_index(self) -> None:
        for name in (
            "schema/replay_case.v1.json",
            "schema/exception_envelope.v1.json",
            "schema/work_capsule.v1.json",
            "fixtures/index.json",
            "WORKING.md",
            "README.md",
            "reports/fatten_summary.json",
        ):
            self.assertTrue((self.root / name).is_file(), name)

    def test_replay_case_roundtrip(self) -> None:
        path = self.root / "fixtures/replay/cases/notice_year.json"
        doc = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(doc["kind"], "workReceiptReplayCase")
        self.assertEqual(doc["expected"]["attest"]["exit"], 0)
        self.assertEqual(doc["expected"]["verify_ok"]["reproduced"], True)
        self.assertEqual(doc["expected"]["verify_mismatch"]["exit"], 3)
        self.assertFalse(doc["expected"]["attest"]["userOutputCreated"])
        self.assertEqual(doc["planSha256"], C.sha256_hex(doc["planText"]))
        for key in C.REPLAY_REQUIRED:
            self.assertIn(key, doc["receipt"], key)

    def test_every_scenario_has_case_and_capsule(self) -> None:
        for scenario in cat.SCENARIOS:
            case = self.root / f"fixtures/replay/cases/{scenario.ident}.json"
            cap = self.root / f"fixtures/capsules/{scenario.ident}.capsule.json"
            self.assertTrue(case.is_file(), case)
            self.assertTrue(cap.is_file(), cap)
            capsule = json.loads(cap.read_text(encoding="utf-8"))
            self.assertEqual(capsule["kind"], "workCapsule")
            self.assertIsNone(capsule["parent"])

    def test_chain_chronicle_holds(self) -> None:
        parent = json.loads(
            (self.root / "fixtures/capsules/notice_year.capsule.json").read_text(
                encoding="utf-8"
            )
        )
        child = json.loads(
            (self.root / "fixtures/capsules/chain_child.capsule.json").read_text(
                encoding="utf-8"
            )
        )
        parent_bytes = (self.root / "fixtures/capsules/notice_year.capsule.json").read_bytes()
        # File may be pretty-printed; hash the on-disk bytes that child recorded.
        self.assertTrue(C.is_sha256_hex(child["parent"]["sha256"]))
        self.assertEqual(
            child["receipt"]["inputSha256"], parent["receipt"]["outputSha256"]
        )
        self.assertTrue(
            C.lineage_ok(parent["receipt"]["outputSha256"], child["receipt"]["inputSha256"])
        )
        # Child records hash of the capsules/ copy as generated (pretty JSON + NL).
        self.assertEqual(child["parent"]["sha256"], C.sha256_hex(parent_bytes))

    def test_exception_matrix_covers_three_commands(self) -> None:
        commands = {e.command for e in cat.EXCEPTIONS}
        self.assertEqual(commands, {"replay", "audit", "lineage"})
        for spec in cat.EXCEPTIONS:
            path = (
                self.root
                / f"fixtures/exceptions/{spec.command}/{spec.ident}.json"
            )
            doc = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(doc["exit"], spec.exit)
            self.assertEqual(doc["argv"], spec.argv)
            if spec.exit == 2 and spec.stdout_bytes == 0:
                self.assertTrue(doc["stdoutSilent"])

    def test_audit_layouts_match_catalog_exit(self) -> None:
        for layout in cat.AUDIT_LAYOUTS:
            path = self.root / f"fixtures/audit-layouts/{layout.ident}/layout.json"
            doc = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(doc["exit"], layout.exit, layout.ident)
            self.assertEqual(doc["envelope"]["total"], layout.total)
            self.assertEqual(doc["envelope"]["reproduced"], layout.reproduced)
            self.assertFalse(doc["recursive"])
            if layout.total:
                self.assertAlmostEqual(
                    doc["envelope"]["reproducedRate"],
                    layout.reproduced / layout.total,
                )
            for member in layout.members:
                self.assertTrue(
                    (self.root / f"fixtures/audit-layouts/{layout.ident}/{member}.capsule.json").exists(),
                    member,
                )

    def test_nested_ignored_is_not_in_total(self) -> None:
        doc = json.loads(
            (self.root / "fixtures/audit-layouts/nested-ignored/layout.json").read_text(
                encoding="utf-8"
            )
        )
        self.assertEqual(doc["envelope"]["total"], 1)
        hidden = (
            self.root
            / "fixtures/audit-layouts/nested-ignored/nested/hidden.capsule.json"
        )
        self.assertTrue(hidden.is_file())

    def test_lineage_topologies(self) -> None:
        for topo in cat.LINEAGE:
            path = self.root / f"fixtures/lineage/{topo.ident}/topology.json"
            doc = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(doc["valid"], topo.valid, topo.ident)
            self.assertEqual(doc["exit"], topo.exit, topo.ident)
            self.assertEqual(doc["envelope"]["depth"], topo.depth)
            for key in C.LINEAGE_REQUIRED:
                self.assertIn(key, doc["envelope"], key)

    def test_hash_vectors_include_every_plan(self) -> None:
        index = json.loads(
            (self.root / "fixtures/hash-vectors/index.json").read_text(encoding="utf-8")
        )
        plans = (self.root / "fixtures/hash-vectors/plans.tsv").read_text(encoding="utf-8")
        for scenario in cat.SCENARIOS:
            self.assertIn(f"plan:{scenario.ident}", plans)
        empty = next(v for v in index["meta"] if v["ident"] == "empty_bytes")
        self.assertEqual(empty["sha256"], C.sha256_hex(b""))

    def test_docs_name_existing_cli_only(self) -> None:
        text = (self.root / "WORKING.md").read_text(encoding="utf-8")
        self.assertIn("rhwp replay", text)
        self.assertIn("audit", text)
        self.assertIn("lineage", text)
        self.assertIn("새 CLI 는 없다", text)
        self.assertNotIn("gym/", text)
        argv = json.loads(
            (self.root / "fixtures/argv/catalog.json").read_text(encoding="utf-8")
        )
        self.assertIn("--recursive", argv["notInvented"])

    def test_needles_locked_to_main_rs(self) -> None:
        self.assertIn("64자리 16진이어야 합니다", C.NEEDLE["replay_expect_not_hex"])
        self.assertIn("같은 기존 파일", C.NEEDLE["replay_same_file"])
        self.assertIn("*.capsule.json 이 없습니다", C.NEEDLE["audit_empty"])
        self.assertIn("parent 필드 없음", C.NEEDLE["lineage_parent_field"])

    def test_summary_counts(self) -> None:
        summary = json.loads(
            (self.root / "reports/fatten_summary.json").read_text(encoding="utf-8")
        )
        self.assertEqual(summary["replayCases"], len(cat.SCENARIOS))
        self.assertEqual(summary["exceptions"], len(cat.EXCEPTIONS))
        self.assertEqual(summary["issue"], 5478)
        self.assertNotIn("gym", summary["allowedCommands"])


class DiskFixtureFreshnessTests(unittest.TestCase):
    """The committed tree must match a live generator run."""

    def test_disk_index_present_after_generate(self) -> None:
        index = TOOL_DIR / "fixtures" / "index.json"
        if not index.is_file():
            self.skipTest("generator has not been run on disk yet")
        data = json.loads(index.read_text(encoding="utf-8"))
        self.assertEqual(data["issue"], 5478)
        self.assertEqual(len(data["replay"]), len(cat.SCENARIOS))
        for ident in data["replay"]:
            self.assertTrue(
                (TOOL_DIR / "fixtures/replay/cases" / f"{ident}.json").is_file()
            )


if __name__ == "__main__":
    unittest.main()
