"""[#5263] gym 휴게실·PARK 입문 계약 — 링크·프로파일·채점 불가침.

이 시험이 지키는 것:

- `gym/tutorial/` · `gym/docs/tutorial.md` · `mydocs/working/gym_tutorial.md`
  · `gym/PARK.md` · `gym/INVITE.md` 가 실재하고 서로를 가리킨다.
- 프로파일 일곱 이름과 packs 묶음이 `gym/profiles/*.json` 과 같다.
- 입문존 CR01~CR04 의 명령·답 키·입력이 안내에 그대로 있다.
- `gym/core/checks.py` 의 `REGISTRY` 가 검토된 서른네 이름 그대로다.
  휴게실 작업이 채점 논리를 바꾸면 이 시험이 실패한다.

바이너리 없이 순수 파일 검사다. 새 pack · 새 과제 JSON 을 만들지 않는다.
"""

from __future__ import annotations

import json
import re
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
TUTORIAL = GYM / "tutorial"
PROFILES = GYM / "profiles"
PACKS = GYM / "packs"
CHECKS_PY = GYM / "core" / "checks.py"
GYM_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "gym-release-gate.yml"

INLINE_LINK_RE = re.compile(
    r"!?\[[^\]]*\]\(\s*(?:<([^>]+)>|([^)\s]+))(?:\s+[^)]*)?\s*\)"
)

PROFILE_IDS = (
    "family",
    "starter",
    "editor",
    "publisher",
    "operator",
    "boss",
    "maintainer",
)

PROFILE_PACKS = {
    "family": ("casual-rides",),
    "starter": ("core-cli", "self-description"),
    "editor": ("core-cli", "text-editing", "table-editing", "objects-media"),
    "publisher": ("serialization", "layout-rendering", "security"),
    "operator": ("corpus-diagnostics", "automation"),
    "boss": ("expert-challenges",),
}

CASUAL_RIDES = (
    {
        "id": "CR01",
        "cmd": "info",
        "input": "samples/table-001.hwp",
        "oracle": "pageCount",
        "answer": "pages",
    },
    {
        "id": "CR02",
        "cmd": "explain",
        "input": "samples/table-001.hwp",
        "oracle": "paragraphCount",
        "answer": "paragraphs",
    },
    {
        "id": "CR03",
        "cmd": "export-tables",
        "input": "samples/table-001.hwp",
        "oracle": "tableCount",
        "answer": "tables",
    },
    {
        "id": "CR04",
        "cmd": "search",
        "input": "samples/table-001.hwp",
        "oracle": "matchCount",
        "answer": "hits",
    },
)

TUTORIAL_PAGES = (
    "README.md",
    "01-admission.md",
    "02-cr01-carousel.md",
    "03-cr02-ferris.md",
    "04-cr03-circus.md",
    "05-cr04-ringtoss.md",
    "06-profiles.md",
    "07-starter-path.md",
    "08-editor-path.md",
    "09-publisher-path.md",
    "10-operator-path.md",
    "11-boss-path.md",
    "12-leaderboard.md",
    "13-invite.md",
    "14-submissions.md",
    "15-scoring-honesty.md",
    "16-unavailable.md",
    "17-faq.md",
    "18-troubleshooting.md",
    "19-windows.md",
    "20-checklist.md",
)

REQUIRED_DOCS = (
    GYM / "PARK.md",
    GYM / "INVITE.md",
    GYM / "docs" / "tutorial.md",
    REPO_ROOT / "mydocs" / "working" / "gym_tutorial.md",
)

FILE_OPERATOR_SNAPSHOT = {
    "same_hash",
    "differs_from_input",
    "file_exists",
    "files_differ",
    "xml_root_eq",
    "json_value_eq",
    "csv_cell_eq",
    "utf8_bom",
    "json_len_eq",
    "csv_row_count_eq",
    "ndjson_count_eq",
    "ndjson_field_eq",
    "json_keys_contain",
    "text_line_eq",
    "json_type_eq",
    "json_len_ge",
    "json_array_item_eq",
    "csv_col_count_eq",
    "csv_header_eq",
    "csv_row_eq",
    "ndjson_keys_contain",
    "ndjson_len_eq",
    "text_line_count_eq",
    "text_line_contains",
}

CLI_OPERATOR_SNAPSHOT = {
    "answer_eq",
    "len_answer_eq",
    "len_ge",
    "value_eq",
    "value_ge",
    "value_in",
    "deep_contains",
    "not_contains",
    "cell_text_eq",
    "text_file_envelope_eq",
}

REGISTRY_SNAPSHOT = FILE_OPERATOR_SNAPSHOT | CLI_OPERATOR_SNAPSHOT

FORBIDDEN_PROFILE_FLAGS = (
    "--profile Family",
    "--profile FAMILY",
    "--profile casual",
    "--profile beginner",
    "--profile expert",
    "--profile guest",
    "--profile kiddie",
    "--profile admin",
)

SCORING_MUTATION_PHRASES = (
    "checks.py 를 고친다",
    "checks.py를 고친다",
    "REGISTRY 에 연산자를 추가",
    "새 채점 연산자를 추가한다",
    "unavailable 을 0점",
    "unavailable을 0점",
)


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def _tutorial_text() -> str:
    parts = [_read(TUTORIAL / name) for name in TUTORIAL_PAGES]
    parts.append(_read(GYM / "docs" / "tutorial.md"))
    parts.append(_read(GYM / "PARK.md"))
    parts.append(_read(GYM / "INVITE.md"))
    return "\n".join(parts)


def iter_md_links(text: str):
    """마크다운 인라인 링크의 (줄번호, 대상)을 낸다. 1-based."""
    for lineno, line in enumerate(text.splitlines(), 1):
        for match in INLINE_LINK_RE.finditer(line):
            dest = match.group(1) or match.group(2) or ""
            dest = dest.strip()
            if dest:
                yield lineno, dest


def is_internal_file_link(dest: str) -> bool:
    if dest.startswith(("http://", "https://", "mailto:", "irc:")):
        return False
    if dest.startswith("#"):
        return False
    if dest.startswith("//"):
        return False
    return True


def dest_path(dest: str) -> str:
    return dest.split("#", 1)[0].split("?", 1)[0]


def broken_relative_links(source: Path, text: str | None = None) -> list[str]:
    """source 기준 상대 링크 중 디스크에 없는 대상을 모은다."""
    text = _read(source) if text is None else text
    problems = []
    for lineno, dest in iter_md_links(text):
        if not is_internal_file_link(dest):
            continue
        rel = dest_path(dest)
        if not rel:
            continue
        resolved = (source.parent / rel).resolve()
        try:
            resolved.relative_to(REPO_ROOT.resolve())
        except ValueError:
            problems.append(f"{source.relative_to(REPO_ROOT)}:{lineno} 저장소 밖 {dest}")
            continue
        if not resolved.exists():
            problems.append(
                f"{source.relative_to(REPO_ROOT)}:{lineno} 없는 파일 {dest} → {resolved}"
            )
    return problems


def load_checks():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    from gym.core import checks  # noqa: WPS433

    return checks


def pack_ids() -> list[str]:
    return sorted(p.name for p in PACKS.iterdir() if (p / "pack.json").is_file())


class RequiredFilesTests(unittest.TestCase):
    def test_tutorial_pages_exist(self):
        missing = [name for name in TUTORIAL_PAGES if not (TUTORIAL / name).is_file()]
        self.assertEqual(missing, [], f"휴게실 페이지 없음: {missing}")

    def test_required_docs_exist(self):
        missing = [str(p.relative_to(REPO_ROOT)) for p in REQUIRED_DOCS if not p.is_file()]
        self.assertEqual(missing, [], f"필수 문서 없음: {missing}")

    def test_tutorial_readme_is_not_the_only_page(self):
        pages = [p for p in TUTORIAL.glob("*.md") if p.is_file()]
        self.assertGreaterEqual(len(pages), 20, "휴게실이 다시 한 장으로 줄었다")

    def test_docs_tutorial_declares_canonical(self):
        text = _read(GYM / "docs" / "tutorial.md")
        self.assertIn("canonical: gym/docs/tutorial.md", text)
        self.assertIn("scripts/tests/test_gym_tutorial.py", text)

    def test_working_note_points_at_issue(self):
        text = _read(REPO_ROOT / "mydocs" / "working" / "gym_tutorial.md")
        self.assertIn("#5263", text)
        self.assertIn("feat/gym-tutorial-park-docs", text)
        self.assertIn("audit.py", text)


class ProfileNameTests(unittest.TestCase):
    def test_seven_profile_files_exist(self):
        for pid in PROFILE_IDS:
            path = PROFILES / f"{pid}.json"
            self.assertTrue(path.is_file(), f"프로파일 파일 없음: {path}")

    def test_profile_ids_match_filenames(self):
        for pid in PROFILE_IDS:
            data = _load_json(PROFILES / f"{pid}.json")
            self.assertEqual(data["id"], pid)
            self.assertEqual(data["kind"], "gymProfile")
            self.assertTrue(data["packs"], pid)

    def test_named_profile_pack_lists_match_json(self):
        for pid, expected in PROFILE_PACKS.items():
            data = _load_json(PROFILES / f"{pid}.json")
            self.assertEqual(tuple(data["packs"]), expected, pid)

    def test_maintainer_covers_every_pack(self):
        data = _load_json(PROFILES / "maintainer.json")
        self.assertEqual(sorted(data["packs"]), pack_ids())

    def test_no_extra_profile_files(self):
        on_disk = sorted(p.stem for p in PROFILES.glob("*.json"))
        self.assertEqual(on_disk, sorted(PROFILE_IDS))

    def test_hub_documents_list_all_profile_ids(self):
        hubs = (
            _read(TUTORIAL / "README.md"),
            _read(TUTORIAL / "06-profiles.md"),
            _read(GYM / "docs" / "tutorial.md"),
            _read(GYM / "PARK.md"),
        )
        for text in hubs:
            for pid in PROFILE_IDS:
                self.assertIn(pid, text, f"{pid} 가 허브에서 빠졌다")

    def test_readme_has_profile_cli_flags(self):
        text = _read(TUTORIAL / "README.md")
        for pid in PROFILE_IDS:
            self.assertIn(f"--profile {pid}", text, pid)

    def test_forbidden_profile_flags_are_not_commands(self):
        """오타 프로파일을 실행 예로 주면 입구에서 넘어진다.

        '쓰지 마라'는 설명에 철자가 등장할 수는 있다. `--profile Family`
        처럼 플래그 형태로 권하면 안 된다.
        """
        command_blobs = []
        for name in TUTORIAL_PAGES:
            text = _read(TUTORIAL / name)
            command_blobs.extend(re.findall(r"```(?:bash|powershell)?\n(.*?)```", text, re.S))
        joined = "\n".join(command_blobs)
        for flag in FORBIDDEN_PROFILE_FLAGS:
            self.assertNotIn(flag, joined, f"실행 예에 금지 플래그: {flag}")

    def test_family_selects_only_casual_rides(self):
        data = _load_json(PROFILES / "family.json")
        self.assertEqual(data["packs"], ["casual-rides"])

    def test_boss_selects_only_expert_challenges(self):
        data = _load_json(PROFILES / "boss.json")
        self.assertEqual(data["packs"], ["expert-challenges"])


class TutorialLinkTests(unittest.TestCase):
    def test_all_tutorial_relative_links_resolve(self):
        problems = []
        for name in TUTORIAL_PAGES:
            problems.extend(broken_relative_links(TUTORIAL / name))
        self.assertEqual(problems, [], "\n".join(problems))

    def test_park_invite_docs_links_resolve(self):
        problems = []
        for path in (
            GYM / "PARK.md",
            GYM / "INVITE.md",
            GYM / "docs" / "tutorial.md",
            REPO_ROOT / "mydocs" / "working" / "gym_tutorial.md",
        ):
            problems.extend(broken_relative_links(path))
        self.assertEqual(problems, [], "\n".join(problems))

    def test_readme_indexes_every_tutorial_page(self):
        text = _read(TUTORIAL / "README.md")
        for name in TUTORIAL_PAGES:
            if name == "README.md":
                continue
            self.assertIn(f"]({name})", text, f"색인에 {name} 링크 없음")

    def test_park_points_at_tutorial_and_invite_and_docs(self):
        text = _read(GYM / "PARK.md")
        self.assertIn("tutorial/README.md", text)
        self.assertIn("INVITE.md", text)
        self.assertIn("docs/tutorial.md", text)

    def test_invite_points_at_tutorial_and_park(self):
        text = _read(GYM / "INVITE.md")
        self.assertIn("tutorial/README.md", text)
        self.assertIn("PARK.md", text)

    def test_tutorial_readme_points_at_park_invite_docs(self):
        text = _read(TUTORIAL / "README.md")
        self.assertIn("../PARK.md", text)
        self.assertIn("../INVITE.md", text)
        self.assertIn("../docs/tutorial.md", text)

    def test_docs_point_at_working_note(self):
        text = _read(GYM / "docs" / "tutorial.md")
        self.assertIn("mydocs/working/gym_tutorial.md", text)

    def test_broken_link_helper_flags_missing_target(self):
        fake = "# x\n\n[없음](does-not-exist-5263.md)\n"
        problems = broken_relative_links(TUTORIAL / "README.md", fake)
        self.assertTrue(any("does-not-exist-5263.md" in p for p in problems), problems)

    def test_broken_link_helper_accepts_existing_relative(self):
        ok = "# x\n\n[지도](../PARK.md)\n"
        self.assertEqual(broken_relative_links(TUTORIAL / "README.md", ok), [])

    def test_external_links_are_ignored_by_file_checker(self):
        text = "# x\n\n[웹](https://example.com/nope)\n"
        self.assertEqual(broken_relative_links(TUTORIAL / "README.md", text), [])


class CasualRideContractTests(unittest.TestCase):
    def test_task_json_still_matches_tutorial_table(self):
        """안내가 과제 JSON 과 어긋나면 방문자가 틀린 키를 낸다.

        이 시험은 과제 JSON 을 고치지 않는다. 읽어서 안내와 같은지만 본다.
        """
        for ride in CASUAL_RIDES:
            task = _load_json(PACKS / "casual-rides" / "tasks" / f"{ride['id']}.json")
            self.assertEqual(task["id"], ride["id"])
            self.assertEqual(task["tier"], 1)
            self.assertEqual(task["input"], ride["input"])
            self.assertEqual(task["submit"]["kind"], "answer")
            check = task["checks"][0]
            self.assertEqual(check["op"], "answer_eq")
            self.assertEqual(check["answer"], ride["answer"])
            self.assertEqual(check["path"], ride["oracle"])
            self.assertEqual(check["cmd"][0], ride["cmd"])

    def test_tutorial_repeats_command_oracle_and_key(self):
        text = _tutorial_text()
        for ride in CASUAL_RIDES:
            self.assertIn(ride["id"], text)
            self.assertIn(ride["cmd"], text)
            self.assertIn(ride["oracle"], text)
            self.assertIn(f"`{ride['answer']}`", text)
            self.assertIn(ride["input"], text)

    def test_casual_requires_the_four_read_commands(self):
        manifest = _load_json(PACKS / "casual-rides" / "pack.json")
        self.assertEqual(
            set(manifest["requires"]["commands"]),
            {"info", "explain", "export-tables", "search"},
        )

    def test_cr_pages_exist_for_each_ride(self):
        mapping = {
            "CR01": "02-cr01-carousel.md",
            "CR02": "03-cr02-ferris.md",
            "CR03": "04-cr03-circus.md",
            "CR04": "05-cr04-ringtoss.md",
        }
        for tid, name in mapping.items():
            text = _read(TUTORIAL / name)
            self.assertIn(tid, text)
            self.assertIn(f"gym/packs/casual-rides/tasks/{tid}.json", text)

    def test_tutorial_does_not_invent_cr05(self):
        """다른 열린 PR 의 CR05+ 와 싸우지 않는다. 안내는 네 개만 잠근다."""
        readme = _read(TUTORIAL / "README.md")
        self.assertNotIn("CR05", readme)
        self.assertIn("CR01", readme)
        self.assertIn("CR04", readme)


class ScoringUntouchedTests(unittest.TestCase):
    def test_registry_matches_devel_snapshot(self):
        checks = load_checks()
        self.assertEqual(set(checks.REGISTRY), REGISTRY_SNAPSHOT)

    def test_registry_has_thirty_four_operators(self):
        checks = load_checks()
        self.assertEqual(len(checks.REGISTRY), 34)
        self.assertEqual(len(set(checks.REGISTRY)), 34)

    def test_global_scan_ops_unchanged(self):
        checks = load_checks()
        self.assertEqual(checks.GLOBAL_SCAN_OPS, {"deep_contains", "not_contains"})

    def test_needs_cli_partition_unchanged(self):
        checks = load_checks()
        file_ops = {name for name, (_fn, cli) in checks.REGISTRY.items() if not cli}
        cli_ops = {name for name, (_fn, cli) in checks.REGISTRY.items() if cli}
        self.assertEqual(file_ops, FILE_OPERATOR_SNAPSHOT)
        self.assertEqual(cli_ops, CLI_OPERATOR_SNAPSHOT)

    def test_honesty_page_lists_every_registry_name(self):
        text = _read(TUTORIAL / "15-scoring-honesty.md")
        for name in sorted(REGISTRY_SNAPSHOT):
            self.assertIn(f"`{name}`", text, name)

    def test_docs_snapshot_lists_every_registry_name(self):
        text = _read(GYM / "docs" / "tutorial.md")
        for name in sorted(REGISTRY_SNAPSHOT):
            self.assertIn(name, text, name)

    def test_tutorial_markdown_does_not_define_op_functions(self):
        for name in TUTORIAL_PAGES:
            text = _read(TUTORIAL / name)
            self.assertNotRegex(text, r"^def op_", f"{name} 이 연산자를 구현한다")

    def test_guides_do_not_claim_to_edit_checks(self):
        text = _tutorial_text()
        for phrase in SCORING_MUTATION_PHRASES:
            self.assertNotIn(phrase, text, phrase)

    def test_guides_say_checks_are_out_of_scope(self):
        text = _tutorial_text()
        self.assertIn("gym/core/checks.py", text)
        self.assertRegex(text, r"바꾸지 않는다|고치지 않는다|불가침")

    def test_checks_py_still_lives_at_canonical_path(self):
        self.assertTrue(CHECKS_PY.is_file())
        source = _read(CHECKS_PY)
        self.assertIn("REGISTRY = {", source)
        self.assertIn('GLOBAL_SCAN_OPS = {"deep_contains", "not_contains"}', source)
        for name in REGISTRY_SNAPSHOT:
            self.assertIn(f'"{name}"', source, name)


class HonestyClauseTests(unittest.TestCase):
    def test_park_keeps_four_honesty_claims(self):
        text = _read(GYM / "PARK.md")
        self.assertIn("장식", text)
        self.assertIn("라이브 오라클", text)
        self.assertIn("unavailable", text)
        self.assertIn("총점은 편의값", text)

    def test_honesty_page_repeats_live_oracle(self):
        text = _read(TUTORIAL / "15-scoring-honesty.md")
        self.assertIn("라이브", text)
        self.assertIn("골든", text)
        self.assertIn("unavailable", text)
        self.assertIn("총점은 편의값", text)

    def test_unavailable_page_rejects_zero_disguise(self):
        text = _read(TUTORIAL / "16-unavailable.md")
        self.assertIn("0점이 아닌 부재", text)
        self.assertIn("missingCommands", text)
        self.assertIn("packsScored", text)

    def test_park_still_has_mermaid_map(self):
        text = _read(GYM / "PARK.md")
        self.assertIn("```mermaid", text)
        self.assertIn("casual-rides", text)
        self.assertIn("expert-challenges", text)


class AdmissionContractTests(unittest.TestCase):
    def test_admission_kind_and_allow_rule_are_documented(self):
        blobs = (
            _read(TUTORIAL / "01-admission.md"),
            _read(GYM / "docs" / "tutorial.md"),
        )
        joined = "\n".join(blobs)
        self.assertIn("gymAdmission", joined)
        self.assertIn("packsScored", joined)
        self.assertIn("allow", joined)
        self.assertIn("deny", joined)

    def test_admission_is_not_perfect_score(self):
        text = _read(TUTORIAL / "01-admission.md")
        self.assertIn("만점", text)
        self.assertIn("입장 거부", text)

    def test_runner_allow_rule_unchanged(self):
        source = _read(GYM / "core" / "runner.py")
        self.assertIn('"verdict": "allow" if scored >= 1 else "deny",', source)


class StarterPathContractTests(unittest.TestCase):
    def test_starter_first_tasks_still_exist(self):
        self.assertTrue((PACKS / "core-cli" / "tasks" / "T01.json").is_file())
        self.assertTrue((PACKS / "core-cli" / "tasks" / "T02.json").is_file())
        self.assertTrue((PACKS / "self-description" / "tasks" / "SD01.json").is_file())

    def test_starter_guide_names_those_tasks(self):
        text = _read(TUTORIAL / "07-starter-path.md")
        self.assertIn("T01", text)
        self.assertIn("T02", text)
        self.assertIn("SD01", text)
        self.assertIn("pages", text)
        self.assertIn("matchCount", text)
        self.assertIn("commands", text)

    def test_t01_is_still_pagecount_on_a_different_sample(self):
        task = _load_json(PACKS / "core-cli" / "tasks" / "T01.json")
        self.assertEqual(task["checks"][0]["path"], "pageCount")
        self.assertNotEqual(task["input"], "samples/table-001.hwp")

    def test_editor_guide_names_existing_first_tasks(self):
        text = _read(TUTORIAL / "08-editor-path.md")
        for tid in ("TE01", "TB01", "OM01"):
            self.assertIn(tid, text)
            self.assertTrue((PACKS / {
                "TE01": "text-editing",
                "TB01": "table-editing",
                "OM01": "objects-media",
            }[tid] / "tasks" / f"{tid}.json").is_file())

    def test_publisher_guide_names_existing_first_tasks(self):
        text = _read(TUTORIAL / "09-publisher-path.md")
        for tid, pack in (("SR01", "serialization"), ("LR01", "layout-rendering"),
                          ("SE01", "security")):
            self.assertIn(tid, text)
            self.assertTrue((PACKS / pack / "tasks" / f"{tid}.json").is_file())

    def test_operator_and_boss_guides_name_existing_first_tasks(self):
        op = _read(TUTORIAL / "10-operator-path.md")
        boss = _read(TUTORIAL / "11-boss-path.md")
        self.assertIn("CD01", op)
        self.assertIn("AU01", op)
        self.assertIn("XC01", boss)
        self.assertTrue((PACKS / "corpus-diagnostics" / "tasks" / "CD01.json").is_file())
        self.assertTrue((PACKS / "automation" / "tasks" / "AU01.json").is_file())
        self.assertTrue((PACKS / "expert-challenges" / "tasks" / "XC01.json").is_file())


class WindowsAndInviteTests(unittest.TestCase):
    def test_windows_page_has_bom_free_utf8(self):
        text = _read(TUTORIAL / "19-windows.md")
        self.assertIn("UTF8Encoding", text)
        self.assertIn("without BOM", text)
        self.assertIn("New-Item", text)
        self.assertIn("--profile family", text)

    def test_windows_page_keeps_answer_keys(self):
        text = _read(TUTORIAL / "19-windows.md")
        for key in ("pages", "paragraphs", "tables", "hits"):
            self.assertIn(key, text)

    def test_invite_guide_and_canonical_share_fingerprint_keys(self):
        invite = _read(GYM / "INVITE.md")
        lounge = _read(TUTORIAL / "13-invite.md")
        for key in (
            "members",
            "ledgerEntries",
            "merkleRoot",
            "workorderSha256",
            "ledgerSnapshotSha256",
        ):
            self.assertIn(key, invite, key)
            self.assertIn(key, lounge, key)

    def test_invite_is_guidance_not_permission(self):
        text = _read(GYM / "INVITE.md") + _read(TUTORIAL / "13-invite.md")
        self.assertIn("권한이 아니라 안내", text)
        self.assertIn("attest", text)


class CiWiringTests(unittest.TestCase):
    def test_gym_workflow_invokes_tutorial_contract(self):
        text = _read(GYM_WORKFLOW)
        self.assertIn("scripts.tests.test_gym_tutorial", text)

    def test_gym_workflow_invokes_audit_and_packs(self):
        text = _read(GYM_WORKFLOW)
        self.assertIn("scripts.tests.test_gym_audit", text)
        self.assertIn("scripts.tests.test_gym_packs", text)


class NegativeGuardTests(unittest.TestCase):
    def test_missing_profile_id_is_caught_by_hub_assertion_shape(self):
        """허브에서 이름 하나가 빠지면 프로파일 시험이 실패해야 한다."""
        text = _read(TUTORIAL / "06-profiles.md")
        for pid in PROFILE_IDS:
            self.assertIn(f"`{pid}`", text)

    def test_link_checker_reports_line_number(self):
        fake = "한 줄\n두 줄\n[깨짐](missing-page.md)\n"
        problems = broken_relative_links(TUTORIAL / "README.md", fake)
        self.assertTrue(any(":3 " in p or ":3" in p for p in problems), problems)

    def test_registry_snapshot_rejects_added_operator(self):
        checks = load_checks()
        mutated = set(checks.REGISTRY) | {"brand_new_op_5263"}
        self.assertNotEqual(mutated, REGISTRY_SNAPSHOT)

    def test_registry_snapshot_rejects_removed_operator(self):
        checks = load_checks()
        mutated = set(checks.REGISTRY) - {"answer_eq"}
        self.assertNotEqual(mutated, REGISTRY_SNAPSHOT)


class FrontMatterTests(unittest.TestCase):
    def test_tutorial_pages_declare_readme_canonical(self):
        for name in TUTORIAL_PAGES:
            text = _read(TUTORIAL / name)
            self.assertTrue(text.startswith("---\n"), name)
            self.assertIn("kind: guide", text)
            self.assertIn("status: active", text)
            self.assertIn("canonical: gym/tutorial/README.md", text)

    def test_docs_and_working_have_front_matter(self):
        docs = _read(GYM / "docs" / "tutorial.md")
        working = _read(REPO_ROOT / "mydocs" / "working" / "gym_tutorial.md")
        self.assertIn("kind: guide", docs)
        self.assertIn("kind: working", working)
        self.assertIn("last_verified: 2026-08-18", docs)
        self.assertIn("last_verified: 2026-08-18", working)


class ScopeGuardTests(unittest.TestCase):
    """이 기둥이 pack 과제 JSON 을 늘리지 않았는지 느슨히 확인한다.

    다른 PR 이 devel 에 합쳐지면 과제 수는 늘어날 수 있다. 여기서
    잠그는 것은 '이 시험 파일이 새 과제를 전제로 쓰지 않는다'와
    '입문 안내가 CR01~CR04 만 가리킨다'다.
    """

    def test_casual_rides_on_this_tree_still_has_four_committed_tasks(self):
        tasks = sorted((PACKS / "casual-rides" / "tasks").glob("CR*.json"))
        # devel 기준 4개. 다른 PR 이 합쳐지면 늘어날 수 있어 하한만 둔다.
        self.assertGreaterEqual(len(tasks), 4)
        ids = {p.stem for p in tasks}
        self.assertTrue({"CR01", "CR02", "CR03", "CR04"}.issubset(ids))

    def test_working_note_forbids_pack_json_edits(self):
        text = _read(REPO_ROOT / "mydocs" / "working" / "gym_tutorial.md")
        self.assertIn("pack 과제 JSON", text)
        self.assertIn("고치지 않는다", text)
        self.assertIn("git add -A", text)

    def test_docs_forbid_new_operator_and_new_pack(self):
        text = _read(GYM / "docs" / "tutorial.md")
        self.assertIn("새 pack", text)
        self.assertIn("새 채점 연산자", text)
        self.assertIn("cargo fmt --all", text)


if __name__ == "__main__":
    unittest.main()
