"""[#5279] gym/core/schema.py 예외 경로·카탈로그·기존 나무 계약.

성공 칸(저장소 pack 전수가 스키마를 지킨다)은 test_gym_packs 가 이미 본다.
이 파일은 그 칸을 바꾸지 않고, 예전이 죽거나 한 줄로 뭉개던 자리를 kind 로
고정한다.

커버하는 네 자리(이슈 본문):
- 필수 키 없음
- 나쁜 tier (0·6·bool·문자열·실수)
- 미등록 연산자
- 프로파일이 없는 pack 을 가리킴

새 CLI 는 없다. REGISTRY 는 읽기만 한다. audit.py / runner.py / checks.py 는
이 시험에서 고치지 않는다.
"""

from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACKS = GYM / "packs"
PROFILES = GYM / "profiles"


def load_schema():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    from gym.core import schema
    return schema


def load_checks():
    if str(REPO_ROOT) not in sys.path:
        sys.path.insert(0, str(REPO_ROOT))
    from gym.core import checks
    return checks


def _pack(**overrides):
    schema = load_schema()
    return schema.clone_minimal_pack(**overrides)


def _task(**overrides):
    schema = load_schema()
    return schema.clone_minimal_task(**overrides)


def _profile(**overrides):
    schema = load_schema()
    return schema.clone_minimal_profile(**overrides)


def _plain(fn, *args):
    errors = []
    fn(*args, errors)
    return errors


def _issues(fn, *args):
    schema = load_schema()
    collected = schema.IssueList()
    fn(*args, collected)
    return collected


def _read(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _write(path, payload):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(payload, fh, ensure_ascii=False, indent=2)
        fh.write("\n")


class CatalogTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_issue_kinds_are_unique_and_documented(self):
        kinds = self.s.SCHEMA_ISSUE_KINDS
        self.assertEqual(len(kinds), len(set(kinds)))
        self.assertGreaterEqual(len(kinds), 25)
        for kind in kinds:
            self.assertTrue(self.s.is_known_issue_kind(kind), kind)
            self.assertTrue(self.s.SCHEMA_ISSUE_HELP[kind], kind)
            self.assertTrue(self.s.describe_issue_kind(kind), kind)

    def test_unknown_kind_falls_back_to_unexpected_help(self):
        self.assertEqual(
            self.s.describe_issue_kind("not-a-kind"),
            self.s.SCHEMA_ISSUE_HELP["unexpected"],
        )

    def test_format_issue_catalog_matches_constants(self):
        catalog = self.s.format_issue_catalog()
        self.assertEqual(len(catalog), self.s.issue_kind_count())
        self.assertEqual([row[0] for row in catalog], list(self.s.SCHEMA_ISSUE_KINDS))

    def test_required_key_tuples_are_stable(self):
        self.assertEqual(
            self.s.TASK_REQUIRED,
            ("id", "tier", "title", "input", "instructions", "submit", "checks"),
        )
        self.assertEqual(
            self.s.PACK_REQUIRED,
            ("schemaVersion", "kind", "id", "title", "axis", "requires", "runner"),
        )
        self.assertEqual(
            self.s.PROFILE_REQUIRED,
            ("schemaVersion", "kind", "id", "title", "packs"),
        )
        self.assertEqual(
            self.s.RUNNER_KEYS,
            ("rhwpVersion", "rhwpCommit", "capabilitiesSha256"),
        )

    def test_submit_kinds_are_exactly_three(self):
        self.assertEqual(self.s.SUBMIT_KINDS, ("answer", "artifact", "pair"))
        self.assertTrue(self.s.is_known_submit_kind("answer"))
        self.assertTrue(self.s.is_known_submit_kind("artifact"))
        self.assertTrue(self.s.is_known_submit_kind("pair"))
        self.assertFalse(self.s.is_known_submit_kind("json"))
        self.assertFalse(self.s.is_known_submit_kind(""))

    def test_schema_version_and_kinds_unchanged(self):
        self.assertEqual(self.s.SCHEMA_VERSION, "1.0")
        self.assertEqual(self.s.PACK_KIND, "gymPack")
        self.assertEqual(self.s.PROFILE_KIND, "gymProfile")
        self.assertEqual(self.s.TIER_MIN, 1)
        self.assertEqual(self.s.TIER_MAX, 5)
        self.assertEqual(self.s.TIER_NAMES[1], "입문")
        self.assertEqual(self.s.TIER_NAMES[5], "보스")
        self.assertEqual(self.s.EDITING_AXES, ("편집", "보안"))

    def test_legacy_message_fragments_are_stable(self):
        self.assertIn("gymPack", self.s.MSG_PACK_KIND)
        self.assertIn("1.0", self.s.MSG_PACK_SCHEMA)
        self.assertIn("requires.commands", self.s.MSG_REQUIRES_EMPTY)
        self.assertIn("1~5", self.s.MSG_TIER)
        self.assertEqual(self.s.MSG_CHECKS_EMPTY, "checks 가 비었다")
        self.assertIn("gymProfile", self.s.MSG_PROFILE_KIND)
        self.assertEqual(self.s.MSG_PACKS_EMPTY, "packs 가 비었다")
        self.assertEqual(self.s.MSG_MISSING_KEY_PREFIX, "필수 키 없음: ")
        self.assertEqual(self.s.MSG_UNKNOWN_OP_PREFIX, "미등록 연산자: ")
        self.assertEqual(self.s.MSG_MISSING_PACK_PREFIX, "없는 pack 참조: ")


class HelperTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_valid_tiers_are_one_through_five(self):
        for tier in (1, 2, 3, 4, 5):
            self.assertTrue(self.s.is_valid_tier(tier), tier)

    def test_invalid_tiers_include_bool_and_text(self):
        for value in (0, 6, -1, True, False, "1", 1.0, None, [], {}, 99):
            self.assertFalse(self.s.is_valid_tier(value), value)

    def test_safe_ids_accept_pack_and_task_shapes(self):
        for ident in ("table-editing", "TB01", "T10", "core-cli", "render-tree",
                      "self-description", "casual-rides", "studio-e2e"):
            self.assertTrue(self.s.is_safe_id(ident), ident)

    def test_unsafe_ids_reject_paths_and_blanks(self):
        for ident in ("", "  ", ".", "..", "../x", "a/b", "a\\b", "C:x",
                      " spaced", "x y", "-leading", "_no", "한글"):
            self.assertFalse(self.s.is_safe_id(ident), ident)

    def test_commit_and_sha_hex(self):
        self.assertTrue(self.s.is_commit_hex("c" * 40))
        self.assertTrue(self.s.is_commit_hex("A" * 40))
        self.assertFalse(self.s.is_commit_hex("c" * 39))
        self.assertFalse(self.s.is_commit_hex("c" * 41))
        self.assertFalse(self.s.is_commit_hex("g" * 40))
        self.assertTrue(self.s.is_sha256_hex("a" * 64))
        self.assertFalse(self.s.is_sha256_hex("a" * 63))
        self.assertFalse(self.s.is_sha256_hex("z" * 64))

    def test_nonempty_str_and_list_helpers(self):
        self.assertTrue(self.s.is_nonempty_str("x"))
        self.assertFalse(self.s.is_nonempty_str("  "))
        self.assertFalse(self.s.is_nonempty_str(1))
        self.assertTrue(self.s.is_nonempty_str_list(["a", "b"]))
        self.assertFalse(self.s.is_nonempty_str_list(["a", ""]))
        self.assertFalse(self.s.is_nonempty_str_list([]))
        self.assertFalse(self.s.is_str_list(["a", 1]))
        self.assertTrue(self.s.is_mapping({}))
        self.assertFalse(self.s.is_mapping([]))

    def test_editing_axis_prefixes(self):
        self.assertTrue(self.s.is_editing_axis("편집 (표 좌표 지정)"))
        self.assertTrue(self.s.is_editing_axis("보안 (은닉)"))
        self.assertFalse(self.s.is_editing_axis("자동화"))
        self.assertFalse(self.s.is_editing_axis(""))
        self.assertFalse(self.s.is_editing_axis(None))

    def test_clone_helpers_do_not_alias(self):
        one = self.s.clone_minimal_task(id="X1")
        two = self.s.clone_minimal_task(id="X2")
        self.assertEqual(one["id"], "X1")
        self.assertEqual(two["id"], "X2")
        one["checks"].append({"name": "extra"})
        self.assertEqual(len(two["checks"]), 1)


class IssueListTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_plain_list_still_gets_legacy_strings(self):
        errors = []
        self.s.validate_task({"id": "X"}, {"id": "p"}, None, errors)
        self.assertTrue(any(item.startswith("p/X: 필수 키 없음:") for item in errors))
        self.assertTrue(all(isinstance(item, str) for item in errors))

    def test_issue_list_records_kind_and_text(self):
        issues = self.s.collect_task({"id": "X"}, {"id": "p"})
        self.assertTrue(issues.has_kind("missing-key"))
        self.assertTrue(issues.of_kind("missing-key"))
        self.assertIn("tier", issues.fields_of("missing-key"))
        self.assertTrue(issues.as_dicts())
        self.assertEqual(issues[0], issues.structured[0].as_text())

    def test_unknown_kind_coerces_to_unexpected(self):
        issue = self.s.SchemaIssue("not-real", "here", "msg")
        self.assertEqual(issue.kind, "unexpected")

    def test_preview_truncates_long_strings(self):
        long = "x" * 200
        issue = self.s.SchemaIssue("empty-field", "w", "m", field="f", got=long)
        self.assertEqual(len(issue.got), 80)
        self.assertTrue(issue.got.endswith("..."))

    def test_preview_uses_type_name_for_objects(self):
        issue = self.s.SchemaIssue("not-a-mapping", "w", "m", got={"a": 1})
        self.assertEqual(issue.got, "dict")
        issue = self.s.SchemaIssue("bad-type", "w", "m", got=list)
        self.assertEqual(issue.got, "list")

    def test_collect_wrappers_return_issue_list(self):
        pack_issues = self.s.collect_pack("nope", "demo-pack")
        self.assertTrue(pack_issues.has_kind("not-a-mapping"))
        profile_issues = self.s.collect_profile("nope", {"demo-pack"})
        self.assertTrue(profile_issues.has_kind("not-a-mapping"))


class PackMissingAndTypeTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_valid_minimal_pack_is_clean(self):
        issues = self.s.collect_pack(_pack(), "demo-pack")
        self.assertEqual(list(issues), [])

    def test_folder_name_mismatch(self):
        issues = self.s.collect_pack(_pack(), "other-pack")
        self.assertTrue(issues.has_kind("pack-id-mismatch"))
        self.assertTrue(any("폴더 이름과 다르다" in line for line in issues))

    def test_bad_kind_and_schema_version(self):
        issues = self.s.collect_pack(_pack(kind="nope", schemaVersion="2.0"), "demo-pack")
        self.assertTrue(issues.has_kind("bad-kind"))
        self.assertTrue(issues.has_kind("bad-schema-version"))
        self.assertTrue(any("gymPack" in line for line in issues))
        self.assertTrue(any("1.0" in line for line in issues))

    def test_empty_title_and_axis(self):
        issues = self.s.collect_pack(_pack(title="", axis=""), "demo-pack")
        self.assertTrue(issues.has_kind("empty-field"))
        self.assertIn("title", issues.fields_of("empty-field"))
        self.assertIn("axis", issues.fields_of("empty-field"))

    def test_non_string_title(self):
        issues = self.s.collect_pack(_pack(title=12), "demo-pack")
        self.assertTrue(issues.has_kind("bad-type"))

    def test_requires_missing_or_empty(self):
        for requires in ({}, {"commands": []}, {"commands": None}):
            body = _pack()
            body["requires"] = requires
            issues = self.s.collect_pack(body, "demo-pack")
            self.assertTrue(issues.has_kind("empty-commands"), requires)
            self.assertTrue(any("requires.commands 가 비었다" in line for line in issues))

    def test_requires_not_an_object(self):
        body = _pack()
        body["requires"] = ["info"]
        issues = self.s.collect_pack(body, "demo-pack")
        self.assertTrue(issues.has_kind("malformed-requires"))

    def test_requires_blank_command_item(self):
        body = _pack()
        body["requires"] = {"commands": ["info", "  "]}
        issues = self.s.collect_pack(body, "demo-pack")
        self.assertTrue(issues.has_kind("empty-commands"))

    def test_runner_missing_each_key(self):
        for key in self.s.RUNNER_KEYS:
            body = _pack()
            body["runner"] = dict(self.s.MINIMAL_RUNNER)
            body["runner"][key] = ""
            issues = self.s.collect_pack(body, "demo-pack")
            self.assertTrue(issues.has_kind("empty-field"), key)
            self.assertTrue(any(f"runner.{key} 가 비었다" in line for line in issues), key)

    def test_runner_not_an_object(self):
        body = _pack()
        body["runner"] = "nope"
        issues = self.s.collect_pack(body, "demo-pack")
        self.assertTrue(issues.has_kind("malformed-runner"))

    def test_runner_commit_and_digest_must_be_hex(self):
        body = _pack()
        body["runner"] = dict(self.s.MINIMAL_RUNNER)
        body["runner"]["rhwpCommit"] = "not-a-commit"
        body["runner"]["capabilitiesSha256"] = "short"
        issues = self.s.collect_pack(body, "demo-pack")
        self.assertTrue(issues.has_kind("bad-runner-identity"))
        self.assertEqual(len(issues.of_kind("bad-runner-identity")), 2)

    def test_runner_non_string_version(self):
        body = _pack()
        body["runner"] = dict(self.s.MINIMAL_RUNNER)
        body["runner"]["rhwpVersion"] = 1
        issues = self.s.collect_pack(body, "demo-pack")
        self.assertTrue(issues.has_kind("bad-type"))

    def test_unsafe_pack_id(self):
        issues = self.s.collect_pack(_pack(id="../x"), "../x")
        self.assertTrue(issues.has_kind("unsafe-id"))

    def test_non_mapping_manifest_does_not_raise(self):
        for payload in (None, [], "x", 1):
            issues = self.s.collect_pack(payload, "demo-pack")
            self.assertTrue(issues.has_kind("not-a-mapping"), payload)

    def test_plain_list_messages_keep_where_prefix(self):
        errors = _plain(self.s.validate_pack, _pack(title=""), "demo-pack")
        self.assertTrue(errors[0].startswith("demo-pack: "))


class TaskMissingKeyTests(unittest.TestCase):
    """이슈 본문: 필수 키 없음."""

    def setUp(self):
        self.s = load_schema()
        self.pack = _pack()

    def test_each_required_key_reports_missing(self):
        for key in self.s.TASK_REQUIRED:
            body = _task()
            del body[key]
            issues = self.s.collect_task(body, self.pack)
            self.assertTrue(issues.has_kind("missing-key"), key)
            self.assertIn(key, issues.fields_of("missing-key"))
            self.assertTrue(
                any(f"필수 키 없음: {key}" in line for line in issues), key)

    def test_empty_task_reports_all_required_keys(self):
        issues = self.s.collect_task({}, {"id": "p"})
        for key in self.s.TASK_REQUIRED:
            self.assertIn(key, issues.fields_of("missing-key"), key)

    def test_empty_string_fields_are_not_missing_keys(self):
        body = _task(title="  ", input="", instructions="")
        issues = self.s.collect_task(body, self.pack)
        self.assertFalse(issues.has_kind("missing-key"))
        self.assertTrue(issues.has_kind("empty-field"))
        self.assertIn("title", issues.fields_of("empty-field"))
        self.assertIn("input", issues.fields_of("empty-field"))
        self.assertIn("instructions", issues.fields_of("empty-field"))

    def test_non_string_title_input_instructions(self):
        body = _task(title=1, input=2, instructions=3)
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("bad-type"))
        self.assertGreaterEqual(len(issues.of_kind("bad-type")), 3)

    def test_empty_id_and_path_id(self):
        issues = self.s.collect_task(_task(id=""), self.pack)
        self.assertTrue(issues.has_kind("empty-field"))
        issues = self.s.collect_task(_task(id="../T01"), self.pack)
        self.assertTrue(issues.has_kind("unsafe-id"))

    def test_non_mapping_task_does_not_raise(self):
        for payload in (None, [], "task", 1):
            issues = self.s.collect_task(payload, self.pack)
            self.assertTrue(issues.has_kind("not-a-mapping"), payload)

    def test_non_mapping_pack_is_tolerated(self):
        issues = self.s.collect_task(_task(), None)
        self.assertEqual(list(issues), [])


class TaskTierTests(unittest.TestCase):
    """이슈 본문: 나쁜 tier."""

    def setUp(self):
        self.s = load_schema()
        self.pack = _pack()

    def test_tiers_one_and_five_pass(self):
        for tier in (1, 2, 3, 4, 5):
            issues = self.s.collect_task(_task(tier=tier), self.pack)
            self.assertEqual(list(issues), [], tier)

    def test_tiers_zero_and_six_keep_legacy_message(self):
        for tier in (0, 6):
            issues = self.s.collect_task(_task(tier=tier), self.pack)
            self.assertTrue(issues.has_kind("bad-tier"), tier)
            self.assertTrue(any("tier" in line for line in issues), tier)
            self.assertTrue(any(self.s.MSG_TIER in line for line in issues), tier)

    def test_bool_tier_is_rejected(self):
        for value in (True, False):
            issues = self.s.collect_task(_task(tier=value), self.pack)
            self.assertTrue(issues.has_kind("bad-tier"), value)
            self.assertTrue(any(self.s.MSG_TIER in line for line in issues), value)

    def test_string_and_float_tier_are_rejected(self):
        for value in ("1", "5", 1.0, 2.5, None, [1], {"n": 1}):
            issues = self.s.collect_task(_task(tier=value), self.pack)
            self.assertTrue(issues.has_kind("bad-tier"), value)

    def test_missing_tier_is_both_missing_key_and_bad_tier(self):
        body = _task()
        del body["tier"]
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("missing-key"))
        self.assertTrue(issues.has_kind("bad-tier"))


class TaskUnknownOpTests(unittest.TestCase):
    """이슈 본문: 미등록 연산자."""

    def setUp(self):
        self.s = load_schema()
        self.pack = _pack()

    def test_unknown_op_keeps_legacy_message(self):
        body = _task(checks=[{"name": "x", "op": "not_an_op"}])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("unknown-op"))
        self.assertTrue(any("미등록 연산자: not_an_op" in line for line in issues))

    def test_none_op_is_unknown(self):
        body = _task(checks=[{"name": "x"}])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("unknown-op"))
        self.assertTrue(any("미등록 연산자: None" in line for line in issues))

    def test_registered_file_op_without_cmd_passes(self):
        body = _task(checks=[{"name": "존재", "op": "file_exists", "file": "a.json"}])
        issues = self.s.collect_task(body, self.pack)
        self.assertEqual(list(issues), [])

    def test_registry_is_not_mutated_when_unknown_op_is_seen(self):
        checks = load_checks()
        before = set(checks.REGISTRY)
        body = _task(checks=[{"name": "x", "op": "invented_op"}])
        self.s.collect_task(body, self.pack)
        self.assertEqual(set(checks.REGISTRY), before)
        self.assertNotIn("invented_op", checks.REGISTRY)

    def test_registered_ops_helper_matches_registry_keys(self):
        checks = load_checks()
        self.assertEqual(self.s.registered_ops(), frozenset(checks.REGISTRY))
        for op in checks.REGISTRY:
            self.assertTrue(self.s.is_registered_op(op), op)
        self.assertFalse(self.s.is_registered_op("invented_op"))


class TaskCheckShapeTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()
        self.pack = _pack()

    def test_empty_checks_keeps_legacy_message(self):
        issues = self.s.collect_task(_task(checks=[]), self.pack)
        self.assertTrue(issues.has_kind("empty-checks"))
        self.assertTrue(any("checks 가 비었다" in line for line in issues))

    def test_missing_checks_also_says_empty(self):
        body = _task()
        del body["checks"]
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("empty-checks"))
        self.assertTrue(issues.has_kind("missing-key"))

    def test_checks_must_be_a_list(self):
        for payload in ({"op": "file_exists"}, "file_exists", 1):
            issues = self.s.collect_task(_task(checks=payload), self.pack)
            self.assertTrue(issues.has_kind("not-a-list"), payload)

    def test_check_item_must_be_object(self):
        body = _task(checks=["file_exists", 1, None])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("malformed-check"))
        self.assertEqual(len(issues.of_kind("malformed-check")), 3)

    def test_check_without_name(self):
        body = _task(checks=[{"op": "file_exists", "file": "a.json"}])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("malformed-check"))
        self.assertTrue(any("이름 없음" in line for line in issues))

    def test_duplicate_check_names(self):
        body = _task(checks=[
            {"name": "존재", "op": "file_exists", "file": "a.json"},
            {"name": "존재", "op": "file_exists", "file": "b.json"},
        ])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("duplicate-check-name"))

    def test_lint_check_fields_for_known_ops(self):
        missing = self.s.lint_check_fields({"op": "file_exists"})
        self.assertEqual(missing, ("file",))
        missing = self.s.lint_check_fields({"op": "csv_cell_eq", "file": "a.csv"})
        self.assertEqual(missing, ("row", "col", "value"))
        self.assertEqual(self.s.lint_check_fields("nope"), ("<not-a-mapping>",))
        self.assertEqual(self.s.lint_check_fields({"op": "file_exists", "file": "a"}), ())


class TaskSubmitTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()
        self.pack = _pack()

    def test_known_submit_kinds_pass(self):
        for kind in self.s.SUBMIT_KINDS:
            issues = self.s.collect_task(_task(submit={"kind": kind}), self.pack)
            self.assertEqual(list(issues), [], kind)

    def test_unknown_submit_kind(self):
        issues = self.s.collect_task(_task(submit={"kind": "zip"}), self.pack)
        self.assertTrue(issues.has_kind("unknown-submit-kind"))
        self.assertTrue(any("zip" in line for line in issues))

    def test_submit_must_be_object(self):
        issues = self.s.collect_task(_task(submit="answer"), self.pack)
        self.assertTrue(issues.has_kind("malformed-submit"))

    def test_empty_submit_kind(self):
        issues = self.s.collect_task(_task(submit={}), self.pack)
        self.assertTrue(issues.has_kind("empty-field"))

    def test_submit_files_must_be_nonempty_strings(self):
        issues = self.s.collect_task(
            _task(submit={"kind": "artifact", "files": []}), self.pack)
        self.assertTrue(issues.has_kind("empty-field"))
        issues = self.s.collect_task(
            _task(submit={"kind": "artifact", "files": "out.hwp"}), self.pack)
        self.assertTrue(issues.has_kind("malformed-submit"))
        issues = self.s.collect_task(
            _task(submit={"kind": "pair", "files": ["o1.hwp", ""]}), self.pack)
        self.assertTrue(issues.has_kind("malformed-submit"))

    def test_submit_files_may_be_omitted(self):
        issues = self.s.collect_task(_task(submit={"kind": "answer"}), self.pack)
        self.assertEqual(list(issues), [])


class TaskCmdTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()
        self.pack = _pack()

    def test_cli_op_requires_cmd(self):
        body = _task(checks=[{"name": "쪽수", "op": "answer_eq", "answer": "pages"}])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("missing-cmd"))
        self.assertTrue(any("answer_eq 는 cmd 가 필요하다" in line for line in issues))

    def test_file_op_rejects_cmd(self):
        body = _task(checks=[{
            "name": "존재",
            "op": "file_exists",
            "file": "a.json",
            "cmd": ["info"],
        }])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("unexpected-cmd"))
        self.assertTrue(any("CLI 를 부르지 않는데 cmd 가 있다" in line for line in issues))

    def test_cmd_must_be_string_list(self):
        body = _task(checks=[{
            "name": "쪽수",
            "op": "answer_eq",
            "answer": "pages",
            "cmd": "info {input} --json",
        }])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("malformed-cmd"))

    def test_cmd_rejects_blank_items(self):
        body = _task(checks=[{
            "name": "쪽수",
            "op": "answer_eq",
            "answer": "pages",
            "cmd": ["info", ""],
        }])
        issues = self.s.collect_task(body, self.pack)
        self.assertTrue(issues.has_kind("malformed-cmd"))

    def test_unknown_command_when_surface_is_known(self):
        body = _task(checks=[{
            "name": "쪽수",
            "op": "answer_eq",
            "answer": "pages",
            "cmd": ["not-a-cli", "{input}", "--json"],
        }])
        issues = self.s.collect_task(body, self.pack, {"info", "edit"})
        self.assertTrue(issues.has_kind("unknown-command"))
        self.assertTrue(any("CLI 에 없는 명령: not-a-cli" in line for line in issues))

    def test_known_command_passes(self):
        body = _task(checks=[{
            "name": "쪽수",
            "op": "answer_eq",
            "answer": "pages",
            "cmd": ["info", "{input}", "--json"],
        }])
        issues = self.s.collect_task(body, self.pack, {"info"})
        self.assertEqual(list(issues), [])

    def test_known_commands_none_skips_surface_check(self):
        body = _task(checks=[{
            "name": "쪽수",
            "op": "answer_eq",
            "answer": "pages",
            "cmd": ["not-a-cli", "{input}"],
        }])
        issues = self.s.collect_task(body, self.pack, None)
        self.assertFalse(issues.has_kind("unknown-command"))


class TaskGlobalScanTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_editing_axis_rejects_deep_contains(self):
        pack = _pack(axis="편집 (표 좌표 지정)")
        body = _task(checks=[{
            "name": "어딘가",
            "op": "deep_contains",
            "value": "x",
            "cmd": ["export-tables", "{input}", "--json"],
        }])
        issues = self.s.collect_task(body, pack)
        self.assertTrue(issues.has_kind("global-scan-forbidden"))
        self.assertTrue(any("전역 훑기 연산자" in line for line in issues))

    def test_security_axis_rejects_not_contains_without_reason(self):
        pack = _pack(axis="보안 (은닉)")
        body = _task(checks=[{
            "name": "지움",
            "op": "not_contains",
            "value": "x",
            "cmd": ["inspect", "{input}", "--json"],
        }])
        issues = self.s.collect_task(body, pack)
        self.assertTrue(issues.has_kind("global-scan-forbidden"))

    def test_allow_global_scan_reason_permits_it(self):
        pack = _pack(axis="편집")
        body = _task(checks=[{
            "name": "어딘가",
            "op": "deep_contains",
            "value": "x",
            "cmd": ["export-tables", "{input}", "--json"],
            "allowGlobalScan": "코퍼스 스윕 예외",
        }])
        issues = self.s.collect_task(body, pack)
        self.assertFalse(issues.has_kind("global-scan-forbidden"))

    def test_task_axis_overrides_pack_axis(self):
        pack = _pack(axis="자동화")
        body = _task(axis="편집", checks=[{
            "name": "어딘가",
            "op": "deep_contains",
            "value": "x",
            "cmd": ["info", "{input}", "--json"],
        }])
        issues = self.s.collect_task(body, pack)
        self.assertTrue(issues.has_kind("global-scan-forbidden"))

    def test_non_editing_axis_allows_global_scan(self):
        pack = _pack(axis="자동화")
        body = _task(checks=[{
            "name": "어딘가",
            "op": "deep_contains",
            "value": "x",
            "cmd": ["info", "{input}", "--json"],
        }])
        issues = self.s.collect_task(body, pack)
        self.assertFalse(issues.has_kind("global-scan-forbidden"))

    def test_global_scan_ops_are_exactly_the_registry_set(self):
        checks = load_checks()
        self.assertEqual(checks.GLOBAL_SCAN_OPS, {"deep_contains", "not_contains"})
        self.assertTrue(self.s.is_global_scan_op("deep_contains"))
        self.assertFalse(self.s.is_global_scan_op("file_exists"))


class ProfileMissingPackTests(unittest.TestCase):
    """이슈 본문: 프로파일이 없는 pack 을 가리킴."""

    def setUp(self):
        self.s = load_schema()

    def test_known_pack_passes(self):
        issues = self.s.collect_profile(_profile(), {"demo-pack"})
        self.assertEqual(list(issues), [])

    def test_missing_pack_keeps_legacy_message(self):
        issues = self.s.collect_profile(
            _profile(packs=["no-such-pack"]), {"demo-pack"})
        self.assertTrue(issues.has_kind("profile-missing-pack"))
        self.assertTrue(any("없는 pack 참조: no-such-pack" in line for line in issues))

    def test_empty_packs_keeps_legacy_message(self):
        issues = self.s.collect_profile(_profile(packs=[]), {"demo-pack"})
        self.assertTrue(issues.has_kind("empty-packs"))
        self.assertTrue(any("packs 가 비었다" in line for line in issues))

    def test_none_packs_is_empty(self):
        body = _profile()
        body["packs"] = None
        issues = self.s.collect_profile(body, {"demo-pack"})
        self.assertTrue(issues.has_kind("empty-packs"))

    def test_packs_must_be_a_list(self):
        issues = self.s.collect_profile(_profile(packs="demo-pack"), {"demo-pack"})
        self.assertTrue(issues.has_kind("not-a-list"))

    def test_duplicate_pack_entries(self):
        issues = self.s.collect_profile(
            _profile(packs=["demo-pack", "demo-pack"]), {"demo-pack"})
        self.assertTrue(issues.has_kind("duplicate-pack"))

    def test_blank_and_unsafe_pack_entries(self):
        issues = self.s.collect_profile(
            _profile(packs=["demo-pack", "  ", "../x"]), {"demo-pack"})
        self.assertTrue(issues.has_kind("empty-field"))
        self.assertTrue(issues.has_kind("unsafe-id"))

    def test_bad_profile_kind(self):
        issues = self.s.collect_profile(_profile(kind="gymPack"), {"demo-pack"})
        self.assertTrue(issues.has_kind("bad-kind"))
        self.assertTrue(any("gymProfile" in line for line in issues))

    def test_bad_profile_schema_version(self):
        issues = self.s.collect_profile(
            _profile(schemaVersion="9.9"), {"demo-pack"})
        self.assertTrue(issues.has_kind("bad-schema-version"))

    def test_unsafe_profile_id(self):
        issues = self.s.collect_profile(_profile(id="a/b"), {"demo-pack"})
        self.assertTrue(issues.has_kind("unsafe-id"))

    def test_empty_title(self):
        issues = self.s.collect_profile(_profile(title=""), {"demo-pack"})
        self.assertTrue(issues.has_kind("empty-field"))

    def test_pack_ids_none_skips_existence_check(self):
        issues = self.s.collect_profile(_profile(packs=["ghost"]), None)
        self.assertFalse(issues.has_kind("profile-missing-pack"))

    def test_non_mapping_profile_does_not_raise(self):
        issues = self.s.collect_profile(["demo-pack"], {"demo-pack"})
        self.assertTrue(issues.has_kind("not-a-mapping"))


class ExistingTreeTests(unittest.TestCase):
    """저장소에 있는 pack·과제·프로파일은 강화 뒤에도 통과해야 한다."""

    def setUp(self):
        self.s = load_schema()

    def test_every_shipped_pack_still_validates(self):
        errors = []
        for path in sorted(PACKS.glob("*/pack.json")):
            self.s.validate_pack(_read(path), str(path.parent), errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_every_shipped_task_still_validates(self):
        errors = []
        for pack_dir in sorted(p.parent for p in PACKS.glob("*/pack.json")):
            manifest = _read(pack_dir / "pack.json")
            for task_path in sorted((pack_dir / "tasks").glob("*.json")):
                self.s.validate_task(_read(task_path), manifest, None, errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_every_shipped_profile_still_validates(self):
        errors = []
        ids = {p.parent.name for p in PACKS.glob("*/pack.json")}
        for path in sorted(PROFILES.glob("*.json")):
            self.s.validate_profile(_read(path), ids, errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_validate_gym_tree_on_repo_is_clean(self):
        issues = self.s.validate_gym_tree(str(GYM))
        self.assertEqual(list(issues), [], "\n".join(issues))

    def test_shipped_tasks_have_no_unknown_ops(self):
        known = self.s.registered_ops()
        unknown = []
        for task_path in PACKS.glob("*/tasks/*.json"):
            task = _read(task_path)
            for check in task.get("checks", []):
                if check.get("op") not in known:
                    unknown.append(f"{task_path.name}:{check.get('op')}")
        self.assertEqual(unknown, [])

    def test_shipped_submit_kinds_are_known(self):
        bad = []
        for task_path in PACKS.glob("*/tasks/*.json"):
            kind = _read(task_path).get("submit", {}).get("kind")
            if kind not in self.s.SUBMIT_KINDS:
                bad.append(f"{task_path.name}:{kind}")
        self.assertEqual(bad, [])

    def test_shipped_tiers_are_one_through_five(self):
        bad = []
        for task_path in PACKS.glob("*/tasks/*.json"):
            tier = _read(task_path).get("tier")
            if not self.s.is_valid_tier(tier):
                bad.append(f"{task_path.name}:{tier}")
        self.assertEqual(bad, [])

    def test_collect_wrappers_agree_with_plain_list(self):
        manifest = _read(PACKS / "table-editing" / "pack.json")
        task = _read(PACKS / "table-editing" / "tasks" / "TB01.json")
        plain = []
        self.s.validate_task(task, manifest, None, plain)
        issues = self.s.collect_task(task, manifest)
        self.assertEqual(plain, list(issues))


class CapabilitiesTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_parse_capabilities_payload_accepts_object(self):
        raw = json.dumps({"version": "0.8.3", "commands": [{"name": "info"}]}).encode()
        payload = self.s.parse_capabilities_payload(raw)
        self.assertEqual(payload["version"], "0.8.3")
        self.assertEqual(self.s.parse_command_names(raw), {"info"})
        self.assertEqual(self.s.parse_capabilities_version(raw), "0.8.3")

    def test_parse_capabilities_payload_rejects_garbage(self):
        self.assertIsNone(self.s.parse_capabilities_payload(None))
        self.assertIsNone(self.s.parse_capabilities_payload(b"not-json"))
        self.assertIsNone(self.s.parse_capabilities_payload(b"[1, 2]"))
        self.assertIsNone(self.s.parse_capabilities_payload(b"\xff\xfe"))

    def test_parse_command_names_skips_bad_items(self):
        raw = json.dumps({
            "commands": [
                {"name": "info"},
                {"name": ""},
                "info",
                {"title": "no-name"},
                None,
            ],
        }).encode()
        self.assertEqual(self.s.parse_command_names(raw), {"info"})

    def test_parse_command_names_none_when_commands_missing(self):
        self.assertIsNone(self.s.parse_command_names(b"{}"))
        self.assertIsNone(self.s.parse_command_names(b'{"commands": "info"}'))

    def test_parse_version_non_string_is_empty(self):
        self.assertEqual(
            self.s.parse_capabilities_version(b'{"version": 1}'), "")

    def test_capabilities_digest_rejects_empty_path(self):
        with self.assertRaises(ValueError):
            self.s.capabilities_digest("")

    def test_known_commands_none_on_bad_json(self):
        with mock.patch.object(self.s, "capabilities_digest", return_value=("a" * 64, b"nope")):
            self.assertIsNone(self.s.known_commands("rhwp"))

    def test_known_commands_set_on_good_json(self):
        raw = json.dumps({"commands": [{"name": "info"}, {"name": "edit"}]}).encode()
        with mock.patch.object(self.s, "capabilities_digest", return_value=("a" * 64, raw)):
            self.assertEqual(self.s.known_commands("rhwp"), {"info", "edit"})

    def test_known_commands_none_when_item_is_not_mapping(self):
        raw = json.dumps({"commands": ["info"]}).encode()
        with mock.patch.object(self.s, "capabilities_digest", return_value=("a" * 64, raw)):
            self.assertIsNone(self.s.known_commands("rhwp"))

    def test_try_known_commands_none_on_oserror(self):
        with mock.patch.object(self.s, "known_commands", side_effect=FileNotFoundError):
            self.assertIsNone(self.s.try_known_commands("missing-bin"))

    def test_capabilities_digest_hashes_stdout(self):
        raw = b'{"commands":[]}'
        completed = mock.Mock(stdout=raw)

        def fake_run(argv, capture_output=False):
            self.assertEqual(argv[1], "capabilities")
            self.assertTrue(capture_output)
            return completed

        with mock.patch.object(self.s.subprocess, "run", side_effect=fake_run):
            digest, got = self.s.capabilities_digest("rhwp")
        self.assertEqual(got, raw)
        self.assertEqual(digest, hashlib.sha256(raw).hexdigest())

    def test_capabilities_digest_treats_none_stdout_as_empty(self):
        with mock.patch.object(self.s.subprocess, "run", return_value=mock.Mock(stdout=None)):
            digest, raw = self.s.capabilities_digest("rhwp")
        self.assertEqual(raw, b"")
        self.assertEqual(digest, hashlib.sha256(b"").hexdigest())


class RunnerIdentityTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_runner_identity_reads_version_and_commit(self):
        raw = json.dumps({"version": "0.8.3", "commands": []}).encode()
        with mock.patch.object(self.s, "capabilities_digest", return_value=("b" * 64, raw)):
            with mock.patch.object(self.s, "git_head", return_value="c" * 40):
                ident = self.s.runner_identity("rhwp", str(REPO_ROOT))
        self.assertEqual(ident["rhwpVersion"], "0.8.3")
        self.assertEqual(ident["rhwpCommit"], "c" * 40)
        self.assertEqual(ident["capabilitiesSha256"], "b" * 64)

    def test_runner_identity_bad_json_leaves_empty_version(self):
        with mock.patch.object(self.s, "capabilities_digest", return_value=("b" * 64, b"nope")):
            with mock.patch.object(self.s, "git_head", return_value=""):
                ident = self.s.runner_identity("rhwp", str(REPO_ROOT))
        self.assertEqual(ident["rhwpVersion"], "")
        self.assertEqual(ident["capabilitiesSha256"], "b" * 64)

    def test_git_head_oserror_is_empty(self):
        with mock.patch.object(self.s.subprocess, "run", side_effect=OSError):
            self.assertEqual(self.s.git_head(str(REPO_ROOT)), "")

    def test_git_head_reads_stdout(self):
        with mock.patch.object(
            self.s.subprocess, "run",
            return_value=mock.Mock(stdout=("d" * 40 + "\n").encode()),
        ):
            self.assertEqual(self.s.git_head(str(REPO_ROOT)), "d" * 40)


class TreeWalkTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def _plant(self, root, pack_id="demo-pack", task=None, profile=None, manifest=None):
        pack_dir = os.path.join(root, "packs", pack_id)
        os.makedirs(os.path.join(pack_dir, "tasks"), exist_ok=True)
        _write(os.path.join(pack_dir, "pack.json"), manifest or _pack(id=pack_id))
        _write(os.path.join(pack_dir, "tasks", "D01.json"), task or _task())
        os.makedirs(os.path.join(root, "profiles"), exist_ok=True)
        _write(
            os.path.join(root, "profiles", "demo.json"),
            profile or _profile(packs=[pack_id]),
        )
        return root

    def test_clean_tree_has_no_issues(self):
        with tempfile.TemporaryDirectory() as tmp:
            issues = self.s.validate_gym_tree(self._plant(tmp))
        self.assertEqual(list(issues), [])

    def test_tree_reports_missing_task_keys(self):
        with tempfile.TemporaryDirectory() as tmp:
            issues = self.s.validate_gym_tree(self._plant(tmp, task={"id": "D01"}))
        self.assertTrue(issues.has_kind("missing-key"))

    def test_tree_reports_bad_tier(self):
        with tempfile.TemporaryDirectory() as tmp:
            issues = self.s.validate_gym_tree(self._plant(tmp, task=_task(tier=9)))
        self.assertTrue(issues.has_kind("bad-tier"))

    def test_tree_reports_unknown_op(self):
        with tempfile.TemporaryDirectory() as tmp:
            issues = self.s.validate_gym_tree(self._plant(
                tmp, task=_task(checks=[{"name": "x", "op": "ghost_op"}])))
        self.assertTrue(issues.has_kind("unknown-op"))

    def test_tree_reports_profile_missing_pack(self):
        with tempfile.TemporaryDirectory() as tmp:
            issues = self.s.validate_gym_tree(self._plant(
                tmp, profile=_profile(packs=["ghost-pack"])))
        self.assertTrue(issues.has_kind("profile-missing-pack"))

    def test_tree_reports_broken_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._plant(tmp)
            broken = os.path.join(tmp, "packs", "demo-pack", "tasks", "D01.json")
            with open(broken, "w", encoding="utf-8") as fh:
                fh.write("{")
            issues = self.s.validate_gym_tree(tmp)
        self.assertTrue(issues.has_kind("malformed-object"))

    def test_tree_reports_json_array_root(self):
        with tempfile.TemporaryDirectory() as tmp:
            self._plant(tmp)
            path = os.path.join(tmp, "profiles", "demo.json")
            with open(path, "w", encoding="utf-8") as fh:
                fh.write("[]")
            issues = self.s.validate_gym_tree(tmp)
        self.assertTrue(issues.has_kind("not-a-mapping"))

    def test_load_json_mapping_missing_file(self):
        issues = self.s.IssueList()
        payload = self.s.load_json_mapping(
            os.path.join("no", "such", "file.json"), issues, "ghost")
        self.assertIsNone(payload)
        self.assertTrue(issues.has_kind("missing-key"))

    def test_discover_pack_ids_ignores_plain_dirs(self):
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "orphan"))
            self.assertEqual(self.s.discover_pack_ids(tmp), [])
            os.makedirs(os.path.join(tmp, "demo-pack"))
            _write(os.path.join(tmp, "demo-pack", "pack.json"), _pack())
            self.assertEqual(self.s.discover_pack_ids(tmp), ["demo-pack"])

    def test_iter_task_paths_skips_non_json(self):
        with tempfile.TemporaryDirectory() as tmp:
            tasks = os.path.join(tmp, "tasks")
            os.makedirs(tasks)
            with open(os.path.join(tasks, "notes.txt"), "w", encoding="utf-8") as fh:
                fh.write("x")
            _write(os.path.join(tasks, "D01.json"), _task())
            names = [os.path.basename(p) for p in self.s.iter_task_paths(tmp)]
        self.assertEqual(names, ["D01.json"])

    def test_iter_helpers_on_missing_dirs(self):
        missing = os.path.join("no", "such", "dir")
        self.assertEqual(self.s.discover_pack_ids(missing), [])
        self.assertEqual(list(self.s.iter_task_paths(missing)), [])
        self.assertEqual(list(self.s.iter_profile_paths(missing)), [])


class RegistryIsolationTests(unittest.TestCase):
    def test_schema_does_not_redefine_registry(self):
        schema = load_schema()
        checks = load_checks()
        self.assertFalse(hasattr(schema, "REGISTRY"))
        self.assertTrue(hasattr(checks, "REGISTRY"))
        self.assertIn("file_exists", checks.REGISTRY)
        self.assertIn("answer_eq", checks.REGISTRY)
        self.assertEqual(schema.registered_ops(), frozenset(checks.REGISTRY))

    def test_needs_cli_matches_registry_flag(self):
        schema = load_schema()
        checks = load_checks()
        for op, (_fn, needs) in checks.REGISTRY.items():
            self.assertEqual(schema.op_needs_cli(op), needs, op)

    def test_field_hints_only_name_existing_ops(self):
        schema = load_schema()
        checks = load_checks()
        extra = set(schema.CHECK_FIELD_HINTS) - set(checks.REGISTRY)
        self.assertEqual(extra, set())


class MessageCompatibilityTests(unittest.TestCase):
    """test_gym_packs / audit.py 가 기대하는 조각이 남아 있는지."""

    def setUp(self):
        self.s = load_schema()

    def test_pack_messages(self):
        errors = _plain(self.s.validate_pack, {"kind": "x"}, "folder")
        blob = "\n".join(errors)
        self.assertIn("kind 가 gymPack 가 아니다", blob)
        self.assertIn("schemaVersion 이 1.0 이 아니다", blob)
        self.assertIn("폴더 이름과 다르다", blob)
        self.assertIn("title 가 비었다", blob)
        self.assertIn("axis 가 비었다", blob)
        self.assertIn("requires.commands 가 비었다", blob)
        self.assertIn("runner.rhwpVersion 가 비었다", blob)

    def test_task_messages(self):
        errors = _plain(self.s.validate_task, {"id": "X", "tier": 0}, {"id": "p"}, None)
        blob = "\n".join(errors)
        self.assertIn("필수 키 없음: title", blob)
        self.assertIn("tier 는 1~5 정수", blob)
        self.assertIn("checks 가 비었다", blob)

    def test_unknown_op_message(self):
        errors = _plain(
            self.s.validate_task,
            _task(checks=[{"name": "c", "op": "ghost"}]),
            {"id": "p"},
            None,
        )
        self.assertTrue(any("미등록 연산자: ghost" in line for line in errors))

    def test_profile_messages(self):
        errors = _plain(
            self.s.validate_profile,
            {"id": "z", "kind": "nope", "packs": ["ghost"]},
            {"demo-pack"},
        )
        blob = "\n".join(errors)
        self.assertIn("kind 가 gymProfile 가 아니다", blob)
        self.assertIn("없는 pack 참조: ghost", blob)

    def test_profile_empty_packs_message(self):
        errors = _plain(self.s.validate_profile, {"id": "z", "kind": "gymProfile"}, set())
        self.assertTrue(any("packs 가 비었다" in line for line in errors))


class FailHelperTests(unittest.TestCase):
    def test_fail_on_none_is_noop(self):
        schema = load_schema()
        schema._fail(None, "w", "m")

    def test_fail_on_plain_list(self):
        schema = load_schema()
        errors = []
        schema._fail(errors, "here", "boom", kind="bad-tier", field="tier", got=0)
        self.assertEqual(errors, ["here: boom"])

    def test_issue_as_dict_omits_empty_optional(self):
        schema = load_schema()
        issue = schema.SchemaIssue("bad-tier", "p/X", "tier 는 1~5 정수")
        self.assertEqual(
            issue.as_dict(),
            {"kind": "bad-tier", "where": "p/X", "message": "tier 는 1~5 정수"},
        )
        issue = schema.SchemaIssue("bad-tier", "p/X", "m", field="tier", got=9)
        self.assertEqual(issue.as_dict()["field"], "tier")
        self.assertEqual(issue.as_dict()["got"], "9")


class MinimalFixtureTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_minimal_constants_round_trip(self):
        self.assertEqual(self.s.MINIMAL_PACK["kind"], "gymPack")
        self.assertEqual(self.s.MINIMAL_TASK["tier"], 1)
        self.assertEqual(self.s.MINIMAL_PROFILE["packs"], ["demo-pack"])
        self.assertEqual(len(self.s.MINIMAL_RUNNER["rhwpCommit"]), 40)
        self.assertEqual(len(self.s.MINIMAL_RUNNER["capabilitiesSha256"]), 64)

    def test_minimal_check_constants_are_registered(self):
        self.assertTrue(self.s.is_registered_op(self.s.MINIMAL_CHECK_FILE["op"]))
        self.assertTrue(self.s.is_registered_op(self.s.MINIMAL_CHECK_CLI["op"]))
        self.assertFalse(self.s.op_needs_cli(self.s.MINIMAL_CHECK_FILE["op"]))
        self.assertTrue(self.s.op_needs_cli(self.s.MINIMAL_CHECK_CLI["op"]))


class WhereLabelTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_task_where_uses_ids(self):
        self.assertEqual(self.s._task_where({"id": "p"}, {"id": "T01"}), "p/T01")
        self.assertEqual(self.s._task_where(None, None), "None/None")

    def test_profile_where(self):
        self.assertEqual(self.s._profile_where({"id": "starter"}), "profiles/starter")
        self.assertEqual(self.s._profile_where(None), "profiles/None")

    def test_axis_prefers_task_then_pack(self):
        self.assertEqual(self.s._axis_of({"axis": "편집"}, {"axis": "자동화"}), "편집")
        self.assertEqual(self.s._axis_of({}, {"axis": "보안"}), "보안")
        self.assertEqual(self.s._axis_of({}, {}), "")


class LoadJsonEdgeTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_load_json_mapping_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "x.json")
            _write(path, {"ok": True})
            payload = self.s.load_json_mapping(path)
        self.assertEqual(payload, {"ok": True})

    def test_load_json_mapping_unicode_error(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "x.json")
            with open(path, "wb") as fh:
                fh.write(b"\xff\xfe\x00{")
            issues = self.s.IssueList()
            payload = self.s.load_json_mapping(path, issues, "x")
        self.assertIsNone(payload)
        self.assertTrue(issues)

    def test_validate_gym_tree_missing_packs_dir(self):
        with tempfile.TemporaryDirectory() as tmp:
            issues = self.s.validate_gym_tree(tmp)
        self.assertEqual(list(issues), [])


class MorePackEdgeTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_empty_pack_dir_uses_fallback_where(self):
        issues = self.s.collect_pack(_pack(), "")
        self.assertTrue(issues.has_kind("pack-id-mismatch"))

    def test_pack_id_none_is_mismatch_and_unsafe(self):
        body = _pack()
        body["id"] = None
        issues = self.s.collect_pack(body, "demo-pack")
        self.assertTrue(issues.has_kind("pack-id-mismatch"))

    def test_axis_non_string(self):
        issues = self.s.collect_pack(_pack(axis=3), "demo-pack")
        self.assertTrue(issues.has_kind("bad-type"))


class MoreTaskEdgeTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()
        self.pack = _pack()

    def test_id_whitespace_only(self):
        issues = self.s.collect_task(_task(id="   "), self.pack)
        self.assertTrue(issues.has_kind("empty-field"))

    def test_pair_submit_with_three_files_passes(self):
        issues = self.s.collect_task(_task(submit={
            "kind": "pair",
            "files": ["o1.hwp", "o2.hwp", "plan.json"],
        }), self.pack)
        self.assertEqual(list(issues), [])

    def test_multiple_unknown_ops_all_reported(self):
        body = _task(checks=[
            {"name": "a", "op": "ghost_a"},
            {"name": "b", "op": "ghost_b"},
        ])
        issues = self.s.collect_task(body, self.pack)
        self.assertEqual(len(issues.of_kind("unknown-op")), 2)

    def test_cli_op_with_known_and_unknown_mix(self):
        body = _task(checks=[
            {
                "name": "ok",
                "op": "answer_eq",
                "answer": "pages",
                "cmd": ["info", "{input}"],
            },
            {
                "name": "bad",
                "op": "value_eq",
                "value": 1,
                "cmd": ["missing", "{input}"],
            },
        ])
        issues = self.s.collect_task(body, self.pack, {"info"})
        self.assertTrue(issues.has_kind("unknown-command"))
        self.assertEqual(len(issues.of_kind("unknown-command")), 1)

    def test_file_ops_do_not_need_known_commands(self):
        body = _task(checks=[
            {"name": "a", "op": "file_exists", "file": "a.json"},
            {"name": "b", "op": "same_hash", "files": ["a.json", "b.json"]},
            {"name": "c", "op": "differs_from_input", "file": "a.hwp"},
            {"name": "d", "op": "files_differ", "files": ["a.json", "b.json"]},
            {"name": "e", "op": "utf8_bom", "file": "a.json"},
        ])
        issues = self.s.collect_task(body, self.pack, set())
        self.assertEqual(list(issues), [])


class MoreProfileEdgeTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_multiple_missing_packs(self):
        issues = self.s.collect_profile(
            _profile(packs=["a", "b", "demo-pack"]), {"demo-pack"})
        self.assertEqual(len(issues.of_kind("profile-missing-pack")), 2)

    def test_title_non_string(self):
        issues = self.s.collect_profile(_profile(title=1), {"demo-pack"})
        self.assertTrue(issues.has_kind("empty-field"))

    def test_schema_version_absent_is_not_an_error(self):
        body = _profile()
        del body["schemaVersion"]
        issues = self.s.collect_profile(body, {"demo-pack"})
        self.assertFalse(issues.has_kind("bad-schema-version"))


class OpSurfaceTests(unittest.TestCase):
    def test_every_registry_op_is_either_cli_or_file(self):
        schema = load_schema()
        checks = load_checks()
        for op in checks.REGISTRY:
            needs = schema.op_needs_cli(op)
            self.assertIsInstance(needs, bool, op)

    def test_hint_table_covers_devel_ops(self):
        schema = load_schema()
        checks = load_checks()
        missing = set(checks.REGISTRY) - set(schema.CHECK_FIELD_HINTS)
        self.assertEqual(missing, set(), f"hints 빠진 op: {missing}")


class ExtraUnknownKeysAreIgnoredTests(unittest.TestCase):
    """스키마는 모르는 키를 거절하지 않는다 — 후속 필드가 들어올 자리."""

    def setUp(self):
        self.s = load_schema()

    def test_pack_extra_description_passes(self):
        body = _pack(description="설명을 더해도 된다")
        self.assertEqual(list(self.s.collect_pack(body, "demo-pack")), [])

    def test_task_extra_axis_and_notes_pass(self):
        body = _task(axis="자동화", notes="채점기가 안 보는 메모")
        self.assertEqual(list(self.s.collect_task(body, _pack())), [])

    def test_profile_extra_description_passes(self):
        body = _profile(description="코스 설명")
        self.assertEqual(list(self.s.collect_profile(body, {"demo-pack"})), [])


class HexCaseAndIdentityTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_uppercase_commit_and_digest_pass(self):
        body = _pack()
        body["runner"] = {
            "rhwpVersion": "0.8.3",
            "rhwpCommit": "C" * 40,
            "capabilitiesSha256": "B" * 64,
        }
        self.assertEqual(list(self.s.collect_pack(body, "demo-pack")), [])

    def test_mixed_case_hex_passes(self):
        self.assertTrue(self.s.is_commit_hex("AbCdEf09" * 5))
        self.assertTrue(self.s.is_sha256_hex("Ab" * 32))

    def test_runner_identity_non_string_version_becomes_empty(self):
        raw = json.dumps({"version": 12, "commands": []}).encode()
        with mock.patch.object(self.s, "capabilities_digest", return_value=("b" * 64, raw)):
            with mock.patch.object(self.s, "git_head", return_value=""):
                ident = self.s.runner_identity("rhwp", str(REPO_ROOT))
        self.assertEqual(ident["rhwpVersion"], "")

    def test_git_head_empty_stdout(self):
        with mock.patch.object(
            self.s.subprocess, "run", return_value=mock.Mock(stdout=b""),
        ):
            self.assertEqual(self.s.git_head(str(REPO_ROOT)), "")


class TreeKnownCommandsTests(unittest.TestCase):
    def setUp(self):
        self.s = load_schema()

    def test_tree_flags_unknown_command_when_surface_given(self):
        with tempfile.TemporaryDirectory() as tmp:
            pack_dir = os.path.join(tmp, "packs", "demo-pack")
            os.makedirs(os.path.join(pack_dir, "tasks"), exist_ok=True)
            _write(os.path.join(pack_dir, "pack.json"), _pack())
            _write(os.path.join(pack_dir, "tasks", "D01.json"), _task(checks=[{
                "name": "쪽수",
                "op": "answer_eq",
                "answer": "pages",
                "cmd": ["missing-bin-cmd", "{input}"],
            }]))
            os.makedirs(os.path.join(tmp, "profiles"), exist_ok=True)
            _write(os.path.join(tmp, "profiles", "demo.json"), _profile())
            issues = self.s.validate_gym_tree(tmp, known_commands={"info"})
        self.assertTrue(issues.has_kind("unknown-command"))

    def test_tree_skips_command_surface_when_none(self):
        with tempfile.TemporaryDirectory() as tmp:
            pack_dir = os.path.join(tmp, "packs", "demo-pack")
            os.makedirs(os.path.join(pack_dir, "tasks"), exist_ok=True)
            _write(os.path.join(pack_dir, "pack.json"), _pack())
            _write(os.path.join(pack_dir, "tasks", "D01.json"), _task(checks=[{
                "name": "쪽수",
                "op": "answer_eq",
                "answer": "pages",
                "cmd": ["missing-bin-cmd", "{input}"],
            }]))
            issues = self.s.validate_gym_tree(tmp, known_commands=None)
        self.assertFalse(issues.has_kind("unknown-command"))


class IssueListKindFilterTests(unittest.TestCase):
    def test_of_kind_empty_when_absent(self):
        schema = load_schema()
        issues = schema.collect_task(_task(), _pack())
        self.assertEqual(issues.of_kind("profile-missing-pack"), [])
        self.assertFalse(issues.has_kind("profile-missing-pack"))

    def test_as_dicts_round_trip_fields(self):
        schema = load_schema()
        body = _task()
        del body["title"]
        issues = schema.collect_task(body, _pack())
        rows = [row for row in issues.as_dicts() if row["kind"] == "missing-key"]
        self.assertTrue(any(row.get("field") == "title" for row in rows))
        self.assertTrue(all("where" in row and "message" in row for row in rows))


if __name__ == "__main__":
    unittest.main()
