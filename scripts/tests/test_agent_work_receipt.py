"""[#5308] rhwp-work-receipt 스킬 고도화 계약.

실사용 에이전트가 기존 replay / audit / lineage 만으로 노동을 증명하도록
문서·픽스처·워크스루가 같은 단어를 쓰는지 파일만으로 고정한다.

새 CLI 를 시험하지 않는다. gym/ 을 열지 않는다.
다른 에이전트 스킬 본문을 요구하거나 바꾸지 않는다. 바이너리·네트워크를 부르지 않는다.

정본: .agents/skills/rhwp-work-receipt/
작업 기록: mydocs/working/archives/agent_work_receipt.md
"""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".agents" / "skills" / "rhwp-work-receipt"
SKILL_MD = SKILL / "SKILL.md"
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
PLANS = FIXTURES / "plans"
ENVS = FIXTURES / "envelopes"
CAPS = FIXTURES / "capsules"
CATALOG = FIXTURES / "catalog.json"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_work_receipt.md"

REFERENCE_NAMES = (
    "README.md",
    "replay-attest.md",
    "capsule-chain.md",
    "audit-accounting.md",
    "lineage-chronicle.md",
    "exit-codes.md",
    "pitfalls.md",
    "decision-tree.md",
    "envelope-field-catalog.md",
    "recipe-index.md",
)

COMMANDS = ("replay", "audit", "lineage")
HASHES = ("inputSha256", "planSha256", "outputSha256")
AXES = ("parentOk", "lineageOk", "reproduced")

SIBLING_SKILLS = (
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-safe-edit",
)

REF_TOKENS = {
    "replay-attest.md": (
        "inputSha256",
        "planSha256",
        "outputSha256",
        "--expect-output-sha256",
        "mode",
        "attest",
        "verify",
        "toolVersion",
        "임시",
    ),
    "capsule-chain.md": (
        "--capsule",
        "--parent",
        "workCapsule",
        "불변",
        "캡슐 파일",
        "같은 파일",
        "planText",
    ),
    "audit-accounting.md": (
        "reproducedRate",
        "*.capsule.json",
        "비재귀",
        "total",
        "failed",
        "exit 3",
        "exit 2",
    ),
    "lineage-chronicle.md": (
        "parentOk",
        "lineageOk",
        "reproduced",
        "brokenAt",
        "--deep",
        "부모 산출",
        "자식 입력",
    ),
    "exit-codes.md": (
        "exit 3",
        "exit 1",
        "exit 2",
        "판정",
        "IO",
        "사용법",
        "0바이트",
    ),
    "pitfalls.md": (
        "toolVersion",
        "누가",
        "서명",
        "attribution",
        "gym",
    ),
}

SKILL_TOKENS = (
    "inputSha256",
    "planSha256",
    "outputSha256",
    "--expect-output-sha256",
    "--capsule",
    "--parent",
    "reproducedRate",
    "parentOk",
    "lineageOk",
    "brokenAt",
    "exit 3",
    "toolVersion",
    "새 CLI",
    "gym",
    "references/replay-attest.md",
    "references/capsule-chain.md",
    "references/audit-accounting.md",
    "references/lineage-chronicle.md",
    "fixtures/catalog.json",
)

WORKING_TOKENS = (
    "#5308",
    "rhwp-work-receipt",
    "replay",
    "audit",
    "lineage",
    "exit 3",
    "gym",
    "5000",
)

INVENTED = (
    "rhwp receipt",
    "rhwp prove",
    "rhwp work-receipt",
    "--recursive",
    "--expect-input-sha256",
)

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path):
    return json.loads(read(path))


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class SkillLayoutTests(unittest.TestCase):
    def test_skill_root_exists(self):
        self.assertTrue(SKILL.is_dir(), SKILL)
        self.assertTrue(SKILL_MD.is_file(), SKILL_MD)
        self.assertTrue(REFS.is_dir(), REFS)
        self.assertTrue(EXAMPLES.is_dir(), EXAMPLES)
        self.assertTrue(FIXTURES.is_dir(), FIXTURES)
        self.assertTrue(CATALOG.is_file(), CATALOG)
        self.assertTrue(WORKING.is_file(), WORKING)

    def test_references_present(self):
        names = sorted(p.name for p in REFS.glob("*.md"))
        self.assertEqual(sorted(REFERENCE_NAMES), names)

    def test_sibling_skills_exist_untouched_by_this_suite(self):
        for name in SIBLING_SKILLS:
            path = REPO / ".agents" / "skills" / name / "SKILL.md"
            self.assertTrue(path.is_file(), path)

    def test_does_not_live_under_gym(self):
        self.assertNotIn("gym", SKILL.parts)
        self.assertFalse((REPO / "gym" / "docs" / "agent_work_receipt.md").exists())


class FrontmatterTests(unittest.TestCase):
    def test_skill_frontmatter(self):
        body = read(SKILL_MD)
        self.assertTrue(body.startswith("---\n"), "frontmatter 시작")
        end = body.find("\n---\n", 4)
        self.assertGreater(end, 0, "frontmatter 종료")
        fm = body[4:end]
        self.assertIn("name: rhwp-work-receipt", fm)
        self.assertIn("description:", fm)

    def test_skill_tokens(self):
        body = read(SKILL_MD)
        for tok in SKILL_TOKENS:
            self.assertIn(tok, body, tok)

    def test_skill_does_not_invent_cli(self):
        body = read(SKILL_MD)
        self.assertIn("새 CLI", body)
        for bad in INVENTED:
            # 금지 안내로 등장할 수는 있다. 호출 예로 쓰면 안 된다.
            if bad in body:
                self.assertRegex(
                    body,
                    rf"(만들지 않|발명하지 않|없다|금지).{{0,80}}{re.escape(bad)}|{re.escape(bad)}.{{0,80}}(만들지 않|발명하지 않|없다|금지)",
                    f"{bad} 가 금지 안내 없이 등장",
                )


class ReferenceTokenTests(unittest.TestCase):
    def test_each_reference_has_contract_tokens(self):
        for name, tokens in REF_TOKENS.items():
            body = read(REFS / name)
            for tok in tokens:
                self.assertIn(tok, body, f"{name} 에 {tok!r} 없음")

    def test_exit_doc_orders_judgment_vs_crash(self):
        body = read(REFS / "exit-codes.md")
        self.assertIn("도구 크래시가 아니다", body)
        self.assertIn("exit 3", body)
        self.assertIn("exit 1", body)
        self.assertIn("exit 2", body)

    def test_audit_is_non_recursive(self):
        body = read(REFS / "audit-accounting.md")
        self.assertIn("비재귀", body)
        self.assertIn("*.capsule.json", body)
        self.assertNotRegex(body, r"rhwp audit .*--recursive")

    def test_parent_path_relative_to_capsule_file(self):
        body = read(REFS / "capsule-chain.md")
        self.assertIn("캡슐 파일", body)
        self.assertIn("cwd", body)


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(CATALOG)

    def test_catalog_header(self):
        self.assertEqual(self.cat["catalogVersion"], "1.0")
        self.assertEqual(self.cat["skill"], "rhwp-work-receipt")
        self.assertEqual(self.cat["issue"], 5308)
        self.assertEqual(tuple(self.cat["commands"]), COMMANDS)
        self.assertEqual(self.cat["attributionClaim"], False)
        self.assertEqual(self.cat["signatureClaim"], False)
        self.assertEqual(self.cat["auditRecursive"], False)
        self.assertEqual(self.cat["auditGlob"], "*.capsule.json")

    def test_catalog_lists_match_files(self):
        for rel in self.cat["plans"]["valid"]:
            self.assertTrue((PLANS / rel).is_file(), rel)
        for rel in self.cat["plans"]["invalid"]:
            self.assertTrue((PLANS / rel).is_file(), rel)
        for rel in self.cat["envelopes"]:
            self.assertTrue((ENVS / rel).is_file(), rel)
        for rel in self.cat["examples"]:
            self.assertTrue((EXAMPLES / rel).is_file(), rel)
        for rel in self.cat["references"]:
            self.assertTrue((REFS / rel).is_file(), rel)
        for rel in self.cat["transcripts"]:
            self.assertTrue((FIXTURES / "transcripts" / rel).is_file(), rel)

    def test_no_stray_plan_files(self):
        listed = set(self.cat["plans"]["valid"]) | set(self.cat["plans"]["invalid"])
        actual = {p.name for p in PLANS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_no_stray_envelope_files(self):
        listed = set(self.cat["envelopes"])
        actual = {p.name for p in ENVS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_example_readme_lists_same_files(self):
        readme = read(EXAMPLES / "README.md")
        for rel in self.cat["examples"]:
            self.assertIn(rel, readme, rel)

    def test_hashes_and_axes(self):
        self.assertEqual(tuple(self.cat["hashes"]), HASHES)
        self.assertEqual(tuple(self.cat["lineageAxes"]), AXES)
        self.assertEqual(self.cat["exits"]["judgment"], 3)
        self.assertEqual(self.cat["exits"]["io"], 1)
        self.assertEqual(self.cat["exits"]["usage"], 2)


class PlanFixtureTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(CATALOG)

    def test_valid_plans_have_required_keys(self):
        for name in self.cat["plans"]["valid"]:
            plan = load_json(PLANS / name)
            self.assertEqual(plan.get("planVersion"), "1.0", name)
            self.assertIsInstance(plan.get("input"), str, name)
            self.assertIsInstance(plan.get("output"), str, name)
            steps = plan.get("steps")
            self.assertIsInstance(steps, list, name)
            self.assertGreaterEqual(len(steps), 1, name)

    def test_invalid_plans_break_a_known_rule(self):
        checks = {
            "invalid_missing_input.json": lambda p: "input" not in p,
            "invalid_empty_steps.json": lambda p: p.get("steps") == [],
            "invalid_missing_plan_version.json": lambda p: "planVersion" not in p,
            "invalid_unknown_action.json": lambda p: p["steps"][0]["action"]
            == "insert_image",
            "invalid_camel_action.json": lambda p: p["steps"][0]["action"]
            == "replaceText",
            "invalid_wrong_keys_source_op.json": lambda p: "input" not in p
            and "source" in p,
            "invalid_numeric_plan_version.json": lambda p: not isinstance(
                p.get("planVersion"), str
            ),
        }
        self.assertEqual(set(self.cat["plans"]["invalid"]), set(checks))
        for name, pred in checks.items():
            self.assertTrue(pred(load_json(PLANS / name)), name)


class EnvelopeTests(unittest.TestCase):
    def test_every_envelope_has_exit_meta(self):
        for path in ENVS.glob("*.json"):
            env = load_json(path)
            meta = env["_skillMeta"]
            self.assertIn(meta["exit"], (0, 1, 2, 3), path.name)
            self.assertIn(meta["command"], COMMANDS + ("run",), path.name)
            self.assertTrue(meta["branch"], path.name)

    def test_attest_has_three_hashes_and_null_reproduced(self):
        env = load_json(ENVS / "replay_attest.json")
        self.assertEqual(env["mode"], "attest")
        for key in HASHES:
            self.assertRegex(env[key], SHA256_RE, key)
        self.assertIsNone(env["reproduced"])
        self.assertEqual(env["_skillMeta"]["exit"], 0)

    def test_verify_mismatch_is_exit_3(self):
        env = load_json(ENVS / "replay_verify_mismatch.json")
        self.assertEqual(env["mode"], "verify")
        self.assertIs(env["reproduced"], False)
        self.assertEqual(env["_skillMeta"]["exit"], 3)

    def test_audit_rate_formula(self):
        env = load_json(ENVS / "audit_mixed.json")
        total = env["total"]
        reproduced = env["reproduced"]
        self.assertAlmostEqual(env["reproducedRate"], reproduced / total)
        self.assertEqual(env["_skillMeta"]["exit"], 3)

    def test_audit_empty_is_usage(self):
        env = load_json(ENVS / "audit_empty.json")
        self.assertEqual(env["_skillMeta"]["exit"], 2)

    def test_lineage_axes_on_two_link(self):
        env = load_json(ENVS / "lineage_two_link.json")
        self.assertTrue(env["valid"])
        self.assertIsNone(env["brokenAt"])
        head = env["links"][0]
        self.assertIs(head["parentOk"], True)
        self.assertIs(head["lineageOk"], True)
        self.assertIsNone(head["reproduced"])

    def test_lineage_deep_fills_reproduced(self):
        env = load_json(ENVS / "lineage_deep.json")
        self.assertTrue(all(link["reproduced"] is True for link in env["links"]))

    def test_lineage_missing_head_is_io(self):
        env = load_json(ENVS / "lineage_missing_head.json")
        self.assertEqual(env["_skillMeta"]["exit"], 1)

    def test_silent_stdout_flag_on_usage_and_io(self):
        for name in (
            "replay_usage.json",
            "replay_io.json",
            "audit_empty.json",
            "lineage_usage.json",
            "lineage_missing_head.json",
        ):
            env = load_json(ENVS / name)
            self.assertTrue(env["_skillMeta"]["stdoutSilentOnFail"], name)


class CapsuleTests(unittest.TestCase):
    def test_root_capsules_are_self_consistent(self):
        for path in CAPS.glob("*.capsule.json"):
            if path.name.startswith("tamper_") or path.name.startswith("toolversion_"):
                continue
            cap = load_json(path)
            self.assertEqual(cap["kind"], "workCapsule", path.name)
            plan_sha = sha256_hex(cap["planText"].encode("utf-8"))
            self.assertEqual(cap["receipt"]["planSha256"], plan_sha, path.name)
            parsed = json.loads(cap["planText"])
            self.assertEqual(cap["plan"], parsed, path.name)
            self.assertEqual(cap["receipt"]["steps"], len(cap["plan"]["steps"]), path.name)
            for key in HASHES:
                self.assertRegex(cap["receipt"][key], SHA256_RE, f"{path.name}:{key}")

    def test_child_parent_is_relative_and_lineage_ok(self):
        index = load_json(FIXTURES / "capsule_index.json")
        by_file = {row["file"]: row for row in index["roots"] + [
            {
                "file": c["file"],
                "outputSha256": c["outputSha256"],
                "inputSha256": c["inputSha256"],
            }
            for c in index["children"]
        ]}
        # rebuild roots map
        roots = {r["file"]: r for r in index["roots"]}
        for child in index["children"]:
            self.assertTrue(child["parentPathRelativeToCapsuleFile"])
            self.assertFalse(Path(child["parent"]).is_absolute())
            parent = roots[child["parent"]]
            cap = load_json(CAPS / child["file"])
            self.assertEqual(cap["parent"]["capsule"], child["parent"])
            self.assertEqual(cap["parent"]["sha256"], parent["fileSha256"])
            self.assertEqual(cap["receipt"]["inputSha256"], parent["outputSha256"])
            self.assertTrue(child["lineageOk"])

    def test_tamper_output_breaks_receipt_hash(self):
        cap = load_json(CAPS / "tamper_output_sha.capsule.json")
        self.assertEqual(cap["receipt"]["outputSha256"], "0" * 64)

    def test_plan_vs_text_tamper(self):
        cap = load_json(CAPS / "tamper_plan_vs_text.capsule.json")
        self.assertNotEqual(cap["plan"], json.loads(cap["planText"]))


class LayoutTests(unittest.TestCase):
    def _count_top(self, root: Path) -> int:
        return sum(
            1
            for p in root.iterdir()
            if p.is_file() and p.name.endswith(".capsule.json")
        )

    def test_all_ok_three_top_level(self):
        root = FIXTURES / "audit-layouts" / "all-ok"
        self.assertEqual(self._count_top(root), 3)
        nested = list(root.rglob("*.capsule.json"))
        self.assertEqual(len(nested), 3)

    def test_nested_ignored_counts_one(self):
        root = FIXTURES / "audit-layouts" / "nested-ignored"
        self.assertEqual(self._count_top(root), 1)
        self.assertTrue((root / "nested" / "hidden.capsule.json").is_file())

    def test_empty_has_zero(self):
        root = FIXTURES / "audit-layouts" / "empty"
        self.assertEqual(self._count_top(root), 0)

    def test_mixed_ext_ignores_decoys(self):
        root = FIXTURES / "audit-layouts" / "mixed-ext"
        self.assertEqual(self._count_top(root), 1)
        self.assertTrue((root / "notes.json").is_file())
        self.assertTrue((root / "keep.capsule.json.bak").is_file())

    def test_relative_subdir_parent_field(self):
        child = load_json(
            FIXTURES / "lineage-layouts" / "relative-subdir" / "child" / "b.capsule.json"
        )
        self.assertEqual(child["parent"]["capsule"], "../root/a.capsule.json")
        parent = FIXTURES / "lineage-layouts" / "relative-subdir" / "root" / "a.capsule.json"
        self.assertEqual(child["parent"]["sha256"], sha256_hex(parent.read_bytes()))

    def test_broken_lineage_input_differs(self):
        child = load_json(
            FIXTURES / "lineage-layouts" / "lineage-broken" / "b.capsule.json"
        )
        parent = load_json(
            FIXTURES / "lineage-layouts" / "lineage-broken" / "a.capsule.json"
        )
        self.assertNotEqual(
            child["receipt"]["inputSha256"], parent["receipt"]["outputSha256"]
        )

    def test_layout_index_matches_dirs(self):
        idx = load_json(FIXTURES / "layout_index.json")
        for row in idx["layouts"]:
            root = REPO / row["root"] if not row["root"].startswith("fixtures/") else FIXTURES.parent / row["root"]
            # catalog stores skill-relative fixtures/…
            root = SKILL / row["root"]
            self.assertTrue(root.is_dir(), row["id"])


class ScenarioTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(FIXTURES / "scenario_catalog.json")

    def test_enough_scenarios(self):
        self.assertGreaterEqual(self.cat["count"], 80)
        self.assertEqual(self.cat["count"], len(self.cat["scenarios"]))

    def test_commands_are_known(self):
        for sc in self.cat["scenarios"]:
            cmd = sc.get("command")
            if cmd is None:
                self.assertTrue(sc.get("refuse") or sc.get("family") in {"boundary", "routing", "pitfall"})
                continue
            self.assertIn(cmd, COMMANDS + ("run",), sc["id"])

    def test_families_cover_ladder(self):
        families = set(self.cat["families"])
        for need in (
            "replay-attest",
            "replay-verify",
            "capsule",
            "audit",
            "lineage",
            "pitfall",
            "exit",
        ):
            self.assertIn(need, families)

    def test_no_gym_command(self):
        blob = json.dumps(self.cat, ensure_ascii=False)
        self.assertNotIn("rhwp gym", blob)


class HashVectorTests(unittest.TestCase):
    def test_plan_vectors_match_digest(self):
        data = load_json(FIXTURES / "hash-vectors" / "vectors.json")
        self.assertEqual(data["alg"], "SHA-256")
        checked = 0
        for vec in data["vectors"]:
            if "payloadUtf8" in vec:
                self.assertEqual(vec["sha256"], sha256_hex(vec["payloadUtf8"].encode("utf-8")), vec["id"])
                checked += 1
            if vec.get("kind") == "expect-output-sha256" and vec.get("valid"):
                self.assertRegex(vec["value"].lower(), SHA256_RE)
        self.assertGreaterEqual(checked, 20)

    def test_invalid_expect_hex_marked(self):
        data = load_json(FIXTURES / "hash-vectors" / "vectors.json")
        bad = [v for v in data["vectors"] if v.get("valid") is False]
        self.assertGreaterEqual(len(bad), 2)
        self.assertTrue(all(v.get("exit") == 2 for v in bad))


class TranscriptTests(unittest.TestCase):
    def test_argv_starts_with_rhwp(self):
        for path in (FIXTURES / "transcripts").glob("*.json"):
            obj = load_json(path)
            if "argv" in obj:
                self.assertEqual(obj["argv"][0], "rhwp", path.name)
                self.assertIn(obj["argv"][1], COMMANDS + ("run",), path.name)
            if "steps" in obj:
                for step in obj["steps"]:
                    self.assertEqual(step["argv"][0], "rhwp")

    def test_verify_mismatch_transcript_exit_3(self):
        t = load_json(FIXTURES / "transcripts" / "verify_mismatch.json")
        self.assertEqual(t["exit"], 3)
        self.assertTrue(t["judgmentNotCrash"])


class WorkingDocTests(unittest.TestCase):
    def test_working_tokens(self):
        body = read(WORKING)
        for tok in WORKING_TOKENS:
            self.assertIn(tok, body, tok)
        self.assertIn("새 CLI", body)

    def test_unix_newlines_on_skill_docs(self):
        for path in [SKILL_MD, WORKING, CATALOG]:
            raw = path.read_bytes()
            self.assertNotIn(b"\r\n", raw, path)


class LinkTests(unittest.TestCase):
    LINK = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")

    def test_relative_links_exist(self):
        missing = []
        for path in list(REFS.glob("*.md")) + list(EXAMPLES.glob("*.md")) + [SKILL_MD]:
            text = read(path)
            for _label, href in self.LINK.findall(text):
                if href.startswith(("http://", "https://", "#", "mailto:")):
                    continue
                target = href.split("#", 1)[0]
                if not target:
                    continue
                dest = (path.parent / target).resolve()
                if not dest.exists():
                    missing.append(f"{path.name} -> {href}")
        self.assertEqual(missing, [])


if __name__ == "__main__":
    unittest.main()
