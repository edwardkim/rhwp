"""[#5339] rhwp-handoff 세션 인수인계 스킬 계약.

실사용 에이전트가 tools/handoff/orchestrator.py 와 replay --capsule/--parent
만으로 세션을 넘기도록 문서·픽스처·워크스루가 같은 단어를 쓰는지 파일만으로
고정한다.

새 CLI 를 시험하지 않는다. gym/ 을 열지 않는다.
다른 에이전트 스킬 본문을 요구하거나 바꾸지 않는다.
기존 scripts/tests/test_agent_handoff.py (오케스트레이터 본체) 를 수정하지 않는다.
바이너리·네트워크를 부르지 않는다.

정본: .claude/skills/rhwp-handoff/
작업 기록: mydocs/working/agent_handoff.md
"""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-handoff"
SKILL_MD = SKILL / "SKILL.md"
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
ENVS = FIXTURES / "envelopes"
CAPS = FIXTURES / "capsules"
CATALOG = FIXTURES / "catalog.json"
WORKING = REPO / "mydocs" / "working" / "agent_handoff.md"
ORCH = REPO / "tools" / "handoff" / "orchestrator.py"
EXISTING_ORCH_TEST = REPO / "scripts" / "tests" / "test_agent_handoff.py"

REFERENCE_NAMES = (
    "README.md",
    "when-to-handoff.md",
    "orchestrator-protocol.md",
    "artifacts.md",
    "result-json.md",
    "journal-chain.md",
    "incoming-agent.md",
    "capsule-parent-chain.md",
    "work-receipt-boundary.md",
    "working-doc-handoff.md",
    "isolation-worktree.md",
    "staging-named-files.md",
    "no-documentcore.md",
    "exception-index.md",
    "exception-missing-capsule.md",
    "exception-parent-hash.md",
    "exception-dirty-worktree.md",
    "exception-disk-full.md",
    "exit-codes.md",
    "pitfalls.md",
    "decision-tree.md",
    "recipe-index.md",
    "envelope-field-catalog.md",
)

TRIGGERS = ("context_budget", "session_interrupt", "seat_refill")
EXCEPTIONS = (
    "missing_capsule",
    "parent_hash_mismatch",
    "dirty_named_worktree",
    "disk_full",
)
HASHES = ("inputSha256", "planSha256", "outputSha256")
NEXT_ACTIONS = ("consume", "retry", "fallback", "selfExecute")
COMMANDS = ("python", "rhwp", "git", "read")

SIBLING_SKILLS = (
    "rhwp-onboarding",
    "rhwp-mcp-session",
    "rhwp-provenance",
    "rhwp-doc-triage",
    "rhwp-safe-edit",
    "rhwp-work-receipt",
)

REF_TOKENS = {
    "when-to-handoff.md": TRIGGERS + ("넘기지",),
    "orchestrator-protocol.md": (
        "tools/handoff/orchestrator.py",
        "HandoffTask",
        "HandoffResult",
        "exit 3",
        "exit 4",
        "untrustedContent",
    ),
    "artifacts.md": ("result.json", "session.capsule.json", "collected/"),
    "result-json.md": ("outcome", "nextAction", "collectedOutputs", "consume"),
    "journal-chain.md": ("prevSha256", "--verify-journal", "brokenAt"),
    "incoming-agent.md": ("result.json", "capsule", "working doc"),
    "capsule-parent-chain.md": ("--parent", "--capsule", "캡슐 파일", "workCapsule"),
    "work-receipt-boundary.md": ("단건", "rhwp-work-receipt", "재작성"),
    "isolation-worktree.md": (
        r"C:\Users\swsz9\rhwp",
        "rhwp-handoff",
        "rhwp-desk",
        "never",
    ),
    "staging-named-files.md": ("git add -A", "이름"),
    "no-documentcore.md": ("DocumentCore", "발명"),
    "exception-missing-capsule.md": ("missing_capsule", "날조"),
    "exception-parent-hash.md": ("parent.sha256", "--parent"),
    "exception-dirty-worktree.md": ("reset", "isolation"),
    "exception-disk-full.md": ("ENOSPC", "추가"),
    "exit-codes.md": ("exit 3", "exit 1", "exit 2", "exit 4", "크래시가 아니다", "0바이트"),
    "pitfalls.md": ("gym", "attribution", "rhwp handoff"),
}

SKILL_TOKENS = (
    "context budget",
    "session interrupt",
    "seat refill",
    "tools/handoff/orchestrator.py",
    "result.json",
    "--capsule",
    "--parent",
    "git add -A",
    "DocumentCore",
    "새 CLI",
    "gym",
    "rhwp-work-receipt",
    "missing capsule",
    "disk full",
    "fixtures/catalog.json",
    "references/when-to-handoff.md",
    "references/orchestrator-protocol.md",
    "references/capsule-parent-chain.md",
    "references/exception-index.md",
)

WORKING_TOKENS = (
    "#5339",
    "rhwp-handoff",
    "orchestrator.py",
    "replay",
    "audit",
    "lineage",
    "exit",
    "gym",
    "5000",
    "새 CLI",
    "DocumentCore",
    "git add -A",
)

INVENTED = (
    "rhwp handoff",
    "rhwp session-resume",
    "rhwp session-chain",
)

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
LINK = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")


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
        self.assertTrue(ORCH.is_file(), ORCH)
        self.assertTrue(EXISTING_ORCH_TEST.is_file(), EXISTING_ORCH_TEST)

    def test_references_present(self):
        names = sorted(p.name for p in REFS.glob("*.md"))
        self.assertEqual(sorted(REFERENCE_NAMES), names)

    def test_sibling_skills_exist_untouched_by_this_suite(self):
        for name in SIBLING_SKILLS:
            path = REPO / ".claude" / "skills" / name / "SKILL.md"
            self.assertTrue(path.is_file(), path)

    def test_does_not_live_under_gym(self):
        self.assertNotIn("gym", SKILL.parts)
        self.assertFalse((REPO / "gym" / "docs" / "agent_handoff.md").exists())

    def test_does_not_rewrite_work_receipt_or_orchestrator_test(self):
        # 이 시험은 파일 존재만 본다. 본문 해시를 고정하지 않는다.
        self.assertTrue((REPO / ".claude" / "skills" / "rhwp-work-receipt" / "SKILL.md").is_file())
        self.assertTrue(EXISTING_ORCH_TEST.is_file())


class FrontmatterTests(unittest.TestCase):
    def test_skill_frontmatter(self):
        body = read(SKILL_MD)
        self.assertTrue(body.startswith("---\n"), "frontmatter 시작")
        end = body.find("\n---\n", 4)
        self.assertGreater(end, 0, "frontmatter 종료")
        fm = body[4:end]
        self.assertIn("name: rhwp-handoff", fm)
        self.assertIn("description:", fm)

    def test_skill_tokens(self):
        body = read(SKILL_MD)
        for tok in SKILL_TOKENS:
            self.assertIn(tok, body, tok)

    def test_skill_does_not_invent_cli(self):
        body = read(SKILL_MD)
        self.assertIn("새 CLI", body)
        for bad in INVENTED:
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
        self.assertIn("크래시가 아니다", body)
        self.assertIn("exit 3", body)
        self.assertIn("exit 4", body)

    def test_three_triggers_only(self):
        body = read(REFS / "when-to-handoff.md")
        for t in TRIGGERS:
            self.assertIn(t, body)
        self.assertIn("세 문자열만", body)

    def test_parent_path_relative_to_capsule_file(self):
        body = read(REFS / "capsule-parent-chain.md")
        self.assertIn("캡슐 파일", body)
        self.assertIn("cwd", body)

    def test_work_receipt_is_pointer(self):
        body = read(REFS / "work-receipt-boundary.md")
        self.assertIn("다시 쓰", body)
        self.assertIn("rhwp-work-receipt", body)


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(CATALOG)

    def test_catalog_header(self):
        self.assertEqual(self.cat["catalogVersion"], "1.0")
        self.assertEqual(self.cat["skill"], "rhwp-handoff")
        self.assertEqual(self.cat["issue"], 5339)
        self.assertEqual(tuple(self.cat["triggers"]), TRIGGERS)
        self.assertEqual(tuple(self.cat["exceptions"]), EXCEPTIONS)
        self.assertEqual(tuple(self.cat["commands"]), COMMANDS)
        self.assertEqual(self.cat["receiptSkill"], "rhwp-work-receipt")
        self.assertTrue(self.cat["receiptIsSingleJobProof"])
        self.assertTrue(self.cat["sessionHandoffIsNotReceipt"])
        self.assertTrue(self.cat["noNewCli"])
        self.assertTrue(self.cat["neverGitAddA"])
        self.assertTrue(self.cat["neverStealNamedWorktrees"])
        self.assertTrue(self.cat["neverInventDocumentCore"])
        self.assertFalse(self.cat["attributionClaim"])
        self.assertFalse(self.cat["gym"])
        self.assertEqual(self.cat["orchestrator"], "tools/handoff/orchestrator.py")
        self.assertEqual(self.cat["hardGate"], "cargo fmt --all -- --check")
        self.assertEqual(self.cat["staleFmt"], "cargo fmt --check")
        self.assertEqual(self.cat["newlineStyle"], "Unix")
        self.assertEqual(self.cat["base"], "devel")

    def test_catalog_lists_match_files(self):
        for rel in self.cat["envelopes"]:
            self.assertTrue((ENVS / rel).is_file(), rel)
        for rel in self.cat["examples"]:
            self.assertTrue((EXAMPLES / rel).is_file(), rel)
        for rel in self.cat["references"]:
            self.assertTrue((REFS / rel).is_file(), rel)
        for rel in self.cat["transcripts"]:
            self.assertTrue((FIXTURES / "transcripts" / rel).is_file(), rel)
        for rel in self.cat["tasks"]["valid"]:
            self.assertTrue((FIXTURES / "tasks" / rel).is_file(), rel)
        for rel in self.cat["tasks"]["invalid"]:
            self.assertTrue((FIXTURES / "tasks" / rel).is_file(), rel)

    def test_no_stray_envelope_files(self):
        listed = set(self.cat["envelopes"])
        actual = {p.name for p in ENVS.glob("*.json")}
        self.assertEqual(listed, actual)

    def test_example_readme_lists_same_files(self):
        readme = read(EXAMPLES / "README.md")
        for rel in self.cat["examples"]:
            if rel == "README.md":
                continue
            self.assertIn(rel, readme, rel)

    def test_hashes_and_actions(self):
        self.assertEqual(tuple(self.cat["hashes"]), HASHES)
        self.assertEqual(tuple(self.cat["nextActions"]), NEXT_ACTIONS)
        self.assertEqual(self.cat["exits"]["judgment"], 3)
        self.assertEqual(self.cat["exits"]["policy"], 4)


class EnvelopeTests(unittest.TestCase):
    def test_known_exits_and_commands(self):
        seen = set()
        for path in ENVS.glob("*.json"):
            env = load_json(path)
            code = env["_skillMeta"]["exit"]
            self.assertIn(code, (0, 1, 2, 3, 4), path.name)
            seen.add(code)
            cmd = env["_skillMeta"]["command"]
            self.assertIn(cmd, COMMANDS + ("cargo",), path.name)
            self.assertEqual(env["_skillMeta"]["hardGate"], "cargo fmt --all -- --check")
            self.assertTrue(env["_skillMeta"]["neverGitAddA"])
            self.assertTrue(env["_skillMeta"]["neverInventDocumentCore"])
        self.assertTrue({1, 2, 3}.issubset(seen), seen)

    def test_accepted_envelope_shape(self):
        env = load_json(ENVS / "orch_accepted.json")
        self.assertEqual(env["protocol"], "DAP/1.0")
        self.assertEqual(env["operation"], "agent.handoff")
        self.assertEqual(env["tool"], "rhwp-handoff-orchestrator")
        self.assertEqual(env["outcome"], "accepted")
        self.assertEqual(env["nextAction"]["action"], "consume")
        self.assertTrue(env["untrustedContent"])
        self.assertEqual(env["_skillMeta"]["exit"], 0)

    def test_boundary_is_rejected_once(self):
        env = load_json(ENVS / "orch_boundary.json")
        self.assertEqual(env["code"], 4000)
        self.assertEqual(env["outcome"], "rejected")
        self.assertEqual(len(env["attempts"]), 1)
        self.assertEqual(env["attempts"][0]["category"], "securityViolation")
        self.assertEqual(env["_skillMeta"]["exit"], 4)

    def test_git_add_a_and_documentcore_rejected(self):
        add = load_json(ENVS / "git_add_a_rejected.json")
        self.assertEqual(add["command"], "git add -A")
        self.assertTrue(add["rejected"])
        core = load_json(ENVS / "documentcore_invented.json")
        self.assertTrue(core["rejected"])
        self.assertIn("DocumentCore", core["command"])

    def test_verify_journal_is_not_head(self):
        env = load_json(ENVS / "verify_journal_ok.json")
        self.assertEqual(env["operation"], "agent.handoff.verifyJournal")
        self.assertTrue(env["_skillMeta"]["notLastResult"])

    def test_exception_exits(self):
        self.assertEqual(load_json(ENVS / "missing_capsule.json")["_skillMeta"]["exit"], 1)
        self.assertEqual(load_json(ENVS / "parent_hash_mismatch.json")["_skillMeta"]["exit"], 3)
        self.assertEqual(load_json(ENVS / "dirty_named_worktree.json")["_skillMeta"]["exit"], 2)
        self.assertEqual(load_json(ENVS / "disk_full.json")["_skillMeta"]["exit"], 1)


class CapsuleTests(unittest.TestCase):
    def test_root_and_children_are_self_consistent(self):
        for path in CAPS.glob("s*.capsule.json"):
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
        roots = {r["file"]: r for r in index["roots"]}
        # later children can also be parents
        known = dict(roots)
        for child in index["children"]:
            self.assertTrue(child["parentPathRelativeToCapsuleFile"], child["file"])
            self.assertFalse(Path(child["parent"]).is_absolute())
            parent = known[child["parent"]]
            cap = load_json(CAPS / child["file"])
            self.assertEqual(cap["parent"]["capsule"], child["parent"])
            self.assertEqual(cap["parent"]["sha256"], parent["fileSha256"])
            self.assertEqual(cap["receipt"]["inputSha256"], parent["outputSha256"])
            self.assertTrue(child["lineageOk"])
            known[child["file"]] = child

    def test_tamper_parent_sha_is_zeros(self):
        cap = load_json(CAPS / "tamper_parent_sha.capsule.json")
        self.assertEqual(cap["parent"]["sha256"], "0" * 64)
        parent = load_json(CAPS / "s03.capsule.json")
        self.assertNotEqual(
            cap["receipt"]["inputSha256"], parent["receipt"]["outputSha256"]
        )

    def test_plan_vs_text_tamper(self):
        cap = load_json(CAPS / "tamper_plan_vs_text.capsule.json")
        self.assertNotEqual(cap["plan"], json.loads(cap["planText"]))

    def test_enough_session_capsules(self):
        good = list(CAPS.glob("s*.capsule.json"))
        self.assertGreaterEqual(len(good), 24)


class JournalTests(unittest.TestCase):
    def test_ok_journal_chain(self):
        raw = (FIXTURES / "journals" / "ok.ndjson").read_text(encoding="utf-8")
        lines = [l for l in raw.splitlines() if l.strip()]
        self.assertEqual(len(lines), 2)
        prev = None
        for i, line in enumerate(lines, start=1):
            rec = json.loads(line)
            self.assertEqual(rec["seq"], i)
            self.assertEqual(rec["prevSha256"], prev)
            prev = sha256_hex(line.encode("utf-8"))
        self.assertEqual(json.loads(lines[0])["event"], "attempt")
        self.assertEqual(json.loads(lines[1])["event"], "final")

    def test_resume_has_four_entries(self):
        raw = (FIXTURES / "journals" / "resume_two_runs.ndjson").read_text(encoding="utf-8")
        lines = [l for l in raw.splitlines() if l.strip()]
        self.assertEqual(len(lines), 4)
        self.assertEqual(json.loads(lines[-1])["seq"], 4)

    def test_index_marks_tampered(self):
        idx = load_json(FIXTURES / "journals" / "index.json")
        self.assertFalse(idx["verify"]["tampered.ndjson"]["chainValid"])
        self.assertEqual(idx["verify"]["tampered.ndjson"]["brokenAt"], 2)
        self.assertEqual(idx["verify"]["tampered.ndjson"]["exit"], 3)


class LayoutTests(unittest.TestCase):
    def test_missing_capsule_layout_has_no_capsule(self):
        root = FIXTURES / "layouts" / "missing-capsule"
        caps = list(root.glob("*.capsule.json"))
        self.assertEqual(caps, [])
        self.assertTrue((root / "result.json").is_file())

    def test_complete_bundle_has_three_heads(self):
        root = FIXTURES / "layouts" / "complete-bundle"
        self.assertTrue((root / "result.json").is_file())
        self.assertTrue((root / "session.capsule.json").is_file())
        self.assertTrue((root / "WORKING.md").is_file())
        self.assertTrue((root / "handoff.journal.ndjson").is_file())

    def test_forbidden_worktree_registry(self):
        reg = load_json(FIXTURES / "layouts" / "forbidden-worktrees" / "registry.json")
        forbidden = reg["forbidden"]
        for needle in (
            r"C:\Users\swsz9\rhwp",
            r"C:\Users\swsz9\rhwp-handoff",
            r"C:\Users\swsz9\rhwp-scaffold-final",
            r"C:\Users\swsz9\rhwp-doc-repro",
        ):
            self.assertTrue(any(needle in p or p == needle for p in forbidden), needle)
        self.assertTrue(any("rhwp-desk" in p for p in forbidden))
        self.assertEqual(reg["rule"], "never steal named worktrees")

    def test_layout_index_matches_dirs(self):
        idx = load_json(FIXTURES / "layout_index.json")
        for row in idx["layouts"]:
            root = SKILL / row["root"]
            self.assertTrue(root.is_dir(), row["id"])


class IncomingTests(unittest.TestCase):
    def test_read_order(self):
        obj = load_json(FIXTURES / "incoming" / "read-order.json")
        self.assertEqual(
            obj["order"],
            ["last result.json", "last *.capsule.json", "last working doc"],
        )

    def test_first_turn_is_read(self):
        obj = load_json(FIXTURES / "incoming" / "first-turn.json")
        self.assertEqual(obj["firstTurn"], "read-only")
        self.assertEqual(obj["maxCommandsFirstTurn"], 1)
        for bad in ("DocumentCore", "git add -A", "rhwp handoff"):
            self.assertIn(bad, obj["forbiddenFirstTurn"])


class TaskFixtureTests(unittest.TestCase):
    def test_valid_tasks(self):
        cat = load_json(CATALOG)
        for name in cat["tasks"]["valid"]:
            task = load_json(FIXTURES / "tasks" / name)
            self.assertEqual(task["handoffVersion"], "1.0", name)
            self.assertTrue(task["taskId"], name)
            self.assertGreater(task["timeoutSec"], 0, name)
            self.assertIsInstance(task["expectedOutputs"], list, name)

    def test_invalid_tasks_break_a_known_rule(self):
        empty = load_json(FIXTURES / "tasks" / "invalid_empty_task_id.json")
        self.assertEqual(empty["task"]["taskId"], "")
        esc = load_json(FIXTURES / "tasks" / "invalid_escape_output.json")
        self.assertIn("..", esc["task"]["expectedOutputs"][0]["path"])
        ver = load_json(FIXTURES / "tasks" / "invalid_version.json")
        self.assertNotEqual(ver["task"]["handoffVersion"], "1.0")
        z = load_json(FIXTURES / "tasks" / "invalid_timeout.json")
        self.assertEqual(z["task"]["timeoutSec"], 0)


class ScenarioTests(unittest.TestCase):
    def setUp(self):
        self.cat = load_json(FIXTURES / "scenario_catalog.json")

    def test_enough_scenarios(self):
        self.assertGreaterEqual(self.cat["count"], 80)
        self.assertEqual(self.cat["count"], len(self.cat["scenarios"]))

    def test_commands_are_known(self):
        allowed = set(COMMANDS + ("cargo", "run", "replay", "audit", "lineage"))
        for sc in self.cat["scenarios"]:
            cmd = sc.get("command")
            if cmd is None:
                self.assertTrue(sc.get("refuse") or sc.get("family") in {
                    "boundary", "pitfall", "receipt-pointer", "trigger", "exception"
                }, sc["id"])
                continue
            self.assertIn(cmd, allowed, sc["id"])

    def test_no_gym_command(self):
        blob = json.dumps(self.cat, ensure_ascii=False)
        self.assertNotIn("rhwp gym", blob)
        invented_calls = [
            sc
            for sc in self.cat["scenarios"]
            if sc.get("command") == "rhwp" and "handoff" in sc.get("title", "") and not sc.get("refuse")
        ]
        self.assertEqual(invented_calls, [])

    def test_hard_gate_on_each_row(self):
        for sc in self.cat["scenarios"]:
            self.assertEqual(sc["hardGate"], "cargo fmt --all -- --check", sc["id"])
            self.assertTrue(sc["noNewCli"], sc["id"])


class HashVectorTests(unittest.TestCase):
    def test_plan_vectors_match_digest(self):
        data = load_json(FIXTURES / "hash-vectors" / "vectors.json")
        self.assertEqual(data["alg"], "SHA-256")
        checked = 0
        for vec in data["vectors"]:
            if "payloadUtf8" in vec:
                self.assertEqual(
                    vec["sha256"],
                    sha256_hex(vec["payloadUtf8"].encode("utf-8")),
                    vec["id"],
                )
                checked += 1
        self.assertGreaterEqual(checked, 20)

    def test_invalid_expect_hex_marked(self):
        data = load_json(FIXTURES / "hash-vectors" / "vectors.json")
        bad = [v for v in data["vectors"] if v.get("valid") is False]
        self.assertGreaterEqual(len(bad), 2)
        self.assertTrue(all(v.get("exit") == 2 for v in bad))


class TranscriptTests(unittest.TestCase):
    def test_argv_starts_with_known(self):
        allowed0 = {"python", "rhwp", "git", "read"}
        allowed_rhwp = {"replay", "run", "lineage", "audit", "batch", "csv-to-table", "edit", "explore"}
        for path in (FIXTURES / "transcripts").glob("*.json"):
            obj = load_json(path)
            for step in obj["steps"]:
                argv = step["argv"]
                self.assertIn(argv[0], allowed0, path.name)
                if argv[0] == "rhwp":
                    self.assertIn(argv[1], allowed_rhwp, path.name)
                if argv[0] == "python":
                    self.assertEqual(argv[1], "tools/handoff/orchestrator.py", path.name)

    def test_git_add_a_transcript_exit_2(self):
        t = load_json(FIXTURES / "transcripts" / "T12_git_add_a.json")
        self.assertEqual(t["exit"], 2)
        self.assertEqual(t["steps"][0]["argv"], ["git", "add", "-A"])


class ResultTests(unittest.TestCase):
    def test_verify_journal_result_is_not_handoff_op(self):
        obj = load_json(FIXTURES / "results" / "verify_journal_only.json")
        self.assertEqual(obj["operation"], "agent.handoff.verifyJournal")
        self.assertNotIn("collectedOutputs", obj)

    def test_accepted_result_has_collected(self):
        obj = load_json(FIXTURES / "results" / "accepted_consume.json")
        self.assertEqual(obj["nextAction"]["action"], "consume")
        self.assertTrue(obj["collectedOutputs"])


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
    def test_relative_links_exist(self):
        missing = []
        for path in list(REFS.glob("*.md")) + list(EXAMPLES.glob("*.md")) + [SKILL_MD]:
            text = read(path)
            for _label, href in LINK.findall(text):
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
