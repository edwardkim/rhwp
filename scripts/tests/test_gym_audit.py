"""[audit] gym 정합 감사 계약 — 전-저장소 정합 불변식.

CI 강제: 실제 저장소의 전 pack 이 "그 방식"(해결 가능·고유·정합)을 지킨다. 비정합
pack(짝 기준풀이 없음·과제 ID 전역 충돌)이 들어오면 이 테스트가 red 로 막는다.
바이너리 없이 순수 파일 검사만 시험한다.

확대 계약 (#5277):
- 없는 packs 루트·읽기 실패·객체가 아닌 JSON 에서도 도구가 죽지 않는다.
- 위반은 코드(ISSUE_CODES)로 접히고, packs[].issues 의 한글 문구는 원 계약을 지킨다.
- 빠진 pack.json, 고아 기준풀이, 전역/pack 안 ID 충돌, 나쁜 스키마를 각각 잡는다.
- CLI 는 `--json` 만. 새 플래그·새 pack 은 없다.
- 치명 예외(KeyboardInterrupt · SystemExit · MemoryError · GeneratorExit)는 삼키지 않는다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "audit.py"

LEGACY_REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "packCount",
    "packs",
    "taskIdCollisions",
    "issueCount",
)

REQUIRED_CODES = (
    "missing-pack-json",
    "orphan-reference",
    "task-id-collision",
    "bad-schema",
    "missing-reference",
    "pack-json-parse",
    "task-id-duplicate-in-pack",
    "task-filename-id-mismatch",
    "reference-id-mismatch",
    "empty-pack",
    "missing-packs-root",
    "task-not-object",
    "pack-json-not-object",
)


def load():
    spec = importlib.util.spec_from_file_location("gym_audit", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _manifest(pid):
    return {
        "schemaVersion": "1.0",
        "kind": "gymPack",
        "id": pid,
        "title": "t",
        "axis": "조회 (x)",
        "requires": {"commands": ["info"]},
        "runner": {
            "rhwpVersion": "0.8.4",
            "rhwpCommit": "a" * 40,
            "capabilitiesSha256": "b" * 64,
        },
    }


def _task(task_id, **overrides):
    body = {
        "id": task_id,
        "tier": 2,
        "title": "t",
        "input": "samples/x.hwp",
        "instructions": "i",
        "submit": {"kind": "answer"},
        "checks": [{
            "op": "answer_eq",
            "answer": "p",
            "cmd": ["info", "{input}", "--json"],
            "path": "pageCount",
        }],
    }
    body.update(overrides)
    return body


def _ref(task_id):
    return {
        "id": task_id,
        "steps": [{
            "answer": {
                "p": {"cmd": ["info", "{input}", "--json"], "path": "pageCount"},
            },
        }],
    }


def _write_json(path, obj):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(obj, fh, ensure_ascii=False)


def _write_text(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def _write_bytes(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as fh:
        fh.write(data)


def _write_pack(root, pid, task_id, with_ref=True, task=None, manifest=None, ref=None):
    pd = os.path.join(root, "packs", pid)
    os.makedirs(os.path.join(pd, "tasks"), exist_ok=True)
    os.makedirs(os.path.join(pd, "reference"), exist_ok=True)
    _write_json(os.path.join(pd, "pack.json"), manifest if manifest is not None else _manifest(pid))
    _write_json(os.path.join(pd, "tasks", f"{task_id}.json"), task if task is not None else _task(task_id))
    if with_ref:
        _write_json(
            os.path.join(pd, "reference", f"{task_id}.json"),
            ref if ref is not None else _ref(task_id),
        )
    return pd


def _codes(report):
    return [i.get("code") for i in report.get("issues") or [] if isinstance(i, dict)]


def _pack_issue_text(report):
    return [i for p in report.get("packs") or [] for i in p.get("issues") or []]


class LoadMixin(unittest.TestCase):
    def setUp(self):
        self.mod = load()


class AuditTests(unittest.TestCase):
    """원 계약 다섯 자리 — 문구와 키를 깨면 CI 가 막는다."""

    def test_real_repo_all_packs_conform(self):
        report = load().audit(str(REPO_ROOT / "gym"))
        self.assertTrue(
            report["ok"],
            f"gym 정합 위반: {report['packs']} · 충돌 {report['taskIdCollisions']}")
        self.assertGreaterEqual(report["packCount"], 10)

    def test_missing_reference_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            r = load().audit(d)
            self.assertFalse(r["ok"])
            self.assertTrue(any("기준풀이" in i for p in r["packs"] for i in p["issues"]))

    def test_orphan_reference_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=True)
            os.remove(os.path.join(d, "packs", "p1", "tasks", "X01.json"))
            r = load().audit(d)
            self.assertFalse(r["ok"])
            self.assertTrue(any("고아" in i for p in r["packs"] for i in p["issues"]))

    def test_task_id_collision_across_packs_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "DUP", with_ref=True)
            _write_pack(d, "p2", "DUP", with_ref=True)
            r = load().audit(d)
            self.assertFalse(r["ok"])
            self.assertIn("DUP", r["taskIdCollisions"])

    def test_clean_fixture_passes(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "A01", with_ref=True)
            _write_pack(d, "p2", "B01", with_ref=True)
            self.assertTrue(load().audit(d)["ok"])


class CatalogContractTests(LoadMixin):
    def test_kind_and_schema_stay_on_v1(self):
        self.assertEqual(self.mod.REPORT_KIND, "gymAudit")
        self.assertEqual(self.mod.SCHEMA_VERSION, "1.0")

    def test_cli_flags_are_unchanged(self):
        args = self.mod.parse_args([])
        self.assertFalse(args.json)
        args = self.mod.parse_args(["--json"])
        self.assertTrue(args.json)
        with self.assertRaises(SystemExit):
            self.mod.parse_args(["--pack", "core-cli"])
        with self.assertRaises(SystemExit):
            self.mod.parse_args(["--out", "x.json"])
        with self.assertRaises(SystemExit):
            self.mod.parse_args(["--strict"])
        with self.assertRaises(SystemExit):
            self.mod.parse_args(["--root", "x"])

    def test_issue_codes_cover_the_issued_paths(self):
        codes = self.mod.issue_codes()
        self.assertEqual(codes, self.mod.ISSUE_CODES)
        self.assertEqual(codes, self.mod.catalog_ids())
        for key in REQUIRED_CODES:
            self.assertIn(key, codes, key)

    def test_every_code_has_family_and_text(self):
        for code in self.mod.ISSUE_CODES:
            self.assertIn(code, self.mod.ISSUE_FAMILY, code)
            self.assertIn(self.mod.ISSUE_FAMILY[code], self.mod.ISSUE_FAMILIES, code)
            self.assertIn(code, self.mod.ISSUE_TEXT, code)
            self.assertTrue(self.mod.ISSUE_TEXT[code], code)
            self.assertTrue(self.mod.is_known_code(code))
            self.assertTrue(self.mod.is_blocking_code(code))
            self.assertEqual(self.mod.issue_family(code), self.mod.ISSUE_FAMILY[code])

    def test_unknown_code_folds_to_tool_family(self):
        self.assertEqual(self.mod.issue_family("not-a-code"), "tool")
        self.assertEqual(self.mod.issue_family(""), "tool")
        self.assertFalse(self.mod.is_known_code("not-a-code"))
        rec = self.mod.make_issue("not-a-code", message="x")
        self.assertEqual(rec["code"], "unexpected")
        self.assertEqual(rec["family"], "tool")

    def test_exits_are_the_audit_contract(self):
        self.assertEqual(self.mod.EXIT_OK, 0)
        self.assertEqual(self.mod.EXIT_VIOLATION, 1)
        self.assertEqual(self.mod.EXIT_TOOL, 2)

    def test_empty_report_has_every_key(self):
        report = self.mod.empty_report()
        for key in self.mod.REPORT_KEYS:
            self.assertIn(key, report, key)
        for key in LEGACY_REPORT_KEYS:
            self.assertIn(key, report, key)
        self.assertEqual(self.mod.validate_report(report), [])
        self.assertEqual(report["kind"], "gymAudit")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertTrue(report["ok"])
        self.assertEqual(report["exit"], 0)

    def test_legacy_load_still_exists(self):
        self.assertTrue(callable(self.mod._load))


class ExceptionKindTests(LoadMixin):
    def test_fatal_exceptions_are_marked(self):
        self.assertTrue(self.mod.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(self.mod.is_fatal_exception(SystemExit(1)))
        self.assertTrue(self.mod.is_fatal_exception(MemoryError()))
        self.assertTrue(self.mod.is_fatal_exception(GeneratorExit()))
        self.assertFalse(self.mod.is_fatal_exception(FileNotFoundError("x")))
        self.assertFalse(self.mod.is_fatal_exception(ValueError("x")))
        self.assertFalse(self.mod.is_fatal_exception(json.JSONDecodeError("bad", "x", 0)))

    def test_file_not_found_depends_on_context(self):
        exc = FileNotFoundError("nope")
        self.assertEqual(self.mod.exception_kind(exc, "packs-root"), "missing-packs-root")
        self.assertEqual(self.mod.exception_kind(exc, "pack-json"), "missing-pack-json")
        self.assertEqual(self.mod.exception_kind(exc, "listdir-tasks"), "missing-tasks-dir")
        self.assertEqual(self.mod.exception_kind(exc, "listdir-reference"), "missing-reference-dir")

    def test_json_decode_depends_on_context(self):
        exc = json.JSONDecodeError("bad", "x", 0)
        self.assertEqual(self.mod.exception_kind(exc, "pack-json"), "pack-json-parse")
        self.assertEqual(self.mod.exception_kind(exc, "task"), "task-parse")
        self.assertEqual(self.mod.exception_kind(exc, "reference"), "reference-parse")

    def test_permission_depends_on_context(self):
        exc = PermissionError("p")
        self.assertEqual(self.mod.exception_kind(exc, "packs-root"), "unlistable-packs")
        self.assertEqual(self.mod.exception_kind(exc, "listdir-tasks"), "unlistable-tasks")
        self.assertEqual(self.mod.exception_kind(exc, "listdir-reference"), "unlistable-reference")
        self.assertEqual(self.mod.exception_kind(exc, "pack-json"), "pack-json-unreadable")
        self.assertEqual(self.mod.exception_kind(exc, "task"), "task-unreadable")
        self.assertEqual(self.mod.exception_kind(exc, "reference"), "reference-unreadable")

    def test_not_a_directory_depends_on_context(self):
        exc = NotADirectoryError("n")
        self.assertEqual(self.mod.exception_kind(exc, "packs-root"), "packs-not-dir")
        self.assertEqual(self.mod.exception_kind(exc, "listdir-tasks"), "tasks-not-dir")
        self.assertEqual(self.mod.exception_kind(exc, "listdir-reference"), "reference-not-dir")

    def test_unicode_and_type_errors(self):
        dec = UnicodeDecodeError("utf-8", b"\xff", 0, 1, "bad")
        self.assertEqual(self.mod.exception_kind(dec, "pack-json"), "pack-json-unreadable")
        self.assertEqual(self.mod.exception_kind(dec, "task"), "task-unreadable")
        self.assertEqual(self.mod.exception_kind(TypeError("t"), "task"), "task-not-object")
        self.assertEqual(self.mod.exception_kind(TypeError("t"), "pack-json"), "pack-json-not-object")
        self.assertEqual(self.mod.exception_kind(TypeError("t"), "reference"), "reference-not-object")
        self.assertEqual(self.mod.exception_kind(TypeError("t"), "schema"), "bad-schema")
        self.assertEqual(self.mod.exception_kind(RuntimeError("r"), "audit"), "unexpected")
        self.assertEqual(self.mod.exception_kind(None, "audit"), "unexpected")

    def test_exception_record_does_not_raise(self):
        rec = self.mod.exception_record(ValueError("boom"), context="pack-json", path="p/pack.json")
        self.assertEqual(rec["context"], "pack-json")
        self.assertEqual(rec["kind"], "pack-json-parse")
        self.assertEqual(rec["error"], "ValueError")
        self.assertIn("boom", rec["head"])
        self.assertEqual(rec["path"], "p/pack.json")
        rec_none = self.mod.exception_record(None)
        self.assertEqual(rec_none["error"], "NoneType")

    def test_truncate_head_bounds(self):
        self.assertEqual(self.mod.truncate_head(None), "")
        self.assertEqual(self.mod.truncate_head("abc"), "abc")
        self.assertEqual(self.mod.truncate_head("abcdef", 3), "abc")
        self.assertEqual(self.mod.truncate_head("abcdef", 0), "")
        self.assertEqual(self.mod.truncate_head("abcdef", -1), "")
        self.assertEqual(self.mod.truncate_head(12), "12")


class PureHelperTests(LoadMixin):
    def test_is_json_name(self):
        self.assertTrue(self.mod.is_json_name("X01.json"))
        self.assertFalse(self.mod.is_json_name("X01.JSON"))
        self.assertFalse(self.mod.is_json_name("X01.txt"))
        self.assertFalse(self.mod.is_json_name(".hidden.json"))
        self.assertFalse(self.mod.is_json_name(""))
        self.assertFalse(self.mod.is_json_name(None))
        self.assertFalse(self.mod.is_json_name("."))
        self.assertFalse(self.mod.is_json_name(".."))

    def test_stem_and_json_filename(self):
        self.assertEqual(self.mod.stem_of("X01.json"), "X01")
        self.assertEqual(self.mod.stem_of("X01"), "X01")
        self.assertEqual(self.mod.stem_of(None), "")
        self.assertEqual(self.mod.json_filename("X01"), "X01.json")
        self.assertEqual(self.mod.json_filename(""), "")
        self.assertEqual(self.mod.json_filename(None), "")

    def test_posix_rel_uses_forward_slashes(self):
        self.assertEqual(self.mod.posix_rel("p1", "tasks", "X01.json"), "p1/tasks/X01.json")
        self.assertEqual(self.mod.posix_rel("p1\\tasks", "X01.json"), "p1/tasks/X01.json")
        self.assertEqual(self.mod.posix_rel("", None, "a"), "a")

    def test_pair_names(self):
        paired = self.mod.pair_names(["A.json", "B.json"], ["B.json", "C.json"])
        self.assertEqual(paired["paired"], ["B.json"])
        self.assertEqual(paired["missing_refs"], ["A.json"])
        self.assertEqual(paired["orphans"], ["C.json"])
        empty = self.mod.pair_names(None, None)
        self.assertEqual(empty["paired"], [])
        self.assertEqual(empty["missing_refs"], [])
        self.assertEqual(empty["orphans"], [])

    def test_detect_in_pack_duplicates(self):
        found = self.mod.detect_in_pack_duplicates({
            "DUP": ["A.json", "B.json"],
            "OK": ["OK.json"],
            "": ["empty.json", "other.json"],
        })
        self.assertEqual(found, {"DUP": ["A.json", "B.json"]})
        self.assertEqual(self.mod.detect_in_pack_duplicates({}), {})
        self.assertEqual(self.mod.detect_in_pack_duplicates(None), {})

    def test_detect_global_collisions_ignores_same_pack(self):
        collisions = self.mod.detect_global_collisions({
            "DUP": ["p1", "p2"],
            "SAME": ["p1", "p1"],
            "OK": ["p3"],
            "": ["p1", "p2"],
        })
        self.assertEqual(collisions, {"DUP": ["p1", "p2"]})
        self.assertNotIn("SAME", collisions)

    def test_distinct_preserve(self):
        self.assertEqual(self.mod.distinct_preserve(["a", "b", "a", "c"]), ["a", "b", "c"])
        self.assertEqual(self.mod.distinct_preserve(None), [])

    def test_json_names_from_filters(self):
        names = self.mod.json_names_from(["B.json", "notes.txt", "A.json", ".skip.json", "C.JSON"])
        self.assertEqual(names, ["A.json", "B.json"])
        self.assertEqual(self.mod.json_names_from(None), [])

    def test_classify_schema_message(self):
        cases = [
            ("core: kind 가 gymPack 가 아니다", "kind"),
            ("core: schemaVersion 이 1.0 이 아니다", "schemaVersion"),
            ("core: pack id(x) 가 폴더 이름과 다르다", "pack-id"),
            ("core: title 가 비었다", "title"),
            ("core: axis 가 비었다", "axis"),
            ("core: requires.commands 가 비었다", "requires"),
            ("core: runner.rhwpVersion 가 비었다", "runner"),
            ("p/X: 필수 키 없음: checks", "task-required"),
            ("p/X: tier 는 1~5 정수", "tier"),
            ("p/X: checks 가 비었다", "checks-empty"),
            ("p/X: 미등록 연산자: nope", "unknown-op"),
            ("p/X: 편집 과제에 전역 훑기 연산자(deep_contains)", "global-scan"),
            ("p/X: answer_eq 는 cmd 가 필요하다", "missing-cmd"),
            ("p/X: same_hash 는 CLI 를 부르지 않는데 cmd 가 있다", "unexpected-cmd"),
            ("이상한 문장", "other"),
        ]
        for message, tag in cases:
            self.assertEqual(self.mod.classify_schema_message(message), tag, message)

    def test_make_issue_and_format_message(self):
        rec = self.mod.make_issue("orphan-reference", pack="p1", path="p1/reference/X.json")
        self.assertEqual(rec["code"], "orphan-reference")
        self.assertEqual(rec["family"], "pairing")
        self.assertIn("고아", rec["message"] + self.mod.ISSUE_TEXT["orphan-reference"])
        self.assertIn("고아", rec["message"])
        self.assertEqual(
            self.mod.format_issue_message("missing-pack-json"),
            "pack.json 이 없다",
        )
        self.assertIn("x", self.mod.format_issue_message("bad-schema", detail="x"))
        self.assertIn("스키마 위반", self.mod.format_issue_message("bad-schema", detail="x"))

    def test_pack_issue_line_keeps_legacy_korean(self):
        self.assertEqual(self.mod.pack_issue_line("missing-pack-json"), "pack.json 이 없다")
        self.assertIn("기준풀이", self.mod.pack_issue_line("missing-reference", name="X01.json"))
        self.assertIn("고아", self.mod.pack_issue_line("orphan-reference", name="X01.json"))
        self.assertIn("파싱 실패", self.mod.pack_issue_line("pack-json-parse", detail="boom"))
        mismatch = self.mod.pack_issue_line(
            "reference-id-mismatch", name="X01.json", tid="A", rid="B")
        self.assertIn("A", mismatch)
        self.assertIn("B", mismatch)

    def test_count_by(self):
        items = [{"code": "a", "family": "x"}, {"code": "a", "family": "y"}, {"code": "b", "family": "x"}]
        self.assertEqual(self.mod.count_by(items, "code"), {"a": 2, "b": 1})
        self.assertEqual(self.mod.count_by(items, "family"), {"x": 2, "y": 1})
        self.assertEqual(self.mod.count_by(None, "code"), {})

    def test_resolve_exit(self):
        ok = self.mod.empty_report()
        self.assertEqual(self.mod.resolve_exit(ok), 0)
        bad = dict(ok)
        bad["ok"] = False
        bad["issueCount"] = 1
        self.assertEqual(self.mod.resolve_exit(bad), 1)
        tool = dict(ok)
        tool["ok"] = False
        tool["toolFailed"] = True
        self.assertEqual(self.mod.resolve_exit(tool), 2)
        missing = dict(ok)
        missing["ok"] = False
        missing["missingPacksRoot"] = True
        self.assertEqual(self.mod.resolve_exit(missing), 2)
        self.assertEqual(self.mod.resolve_exit("nope"), 2)

    def test_validate_report_rejects_broken(self):
        self.assertIn("객체가 아니다", self.mod.validate_report("x")[0])
        broken = self.mod.empty_report()
        del broken["ok"]
        self.assertTrue(any("키 없음" in p for p in self.mod.validate_report(broken)))
        wrong = self.mod.empty_report()
        wrong["kind"] = "nope"
        self.assertTrue(any("kind" in p for p in self.mod.validate_report(wrong)))
        flip = self.mod.empty_report()
        flip["ok"] = False
        flip["issueCount"] = 0
        flip["toolFailed"] = False
        self.assertTrue(any("ok" in p for p in self.mod.validate_report(flip)))

    def test_path_kind(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertEqual(self.mod.path_kind(d), "dir")
            f = os.path.join(d, "f.txt")
            _write_text(f, "x")
            self.assertEqual(self.mod.path_kind(f), "file")
            self.assertEqual(self.mod.path_kind(os.path.join(d, "missing")), "missing")
            self.assertEqual(self.mod.path_kind(""), "missing")

    def test_load_object_rejects_array(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "a.json")
            _write_text(path, "[1, 2]")
            obj, err = self.mod.load_object(path, context="task")
            self.assertIsNone(obj)
            self.assertEqual(err["kind"], "task-not-object")

    def test_load_object_rejects_invalid_json(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "a.json")
            _write_text(path, "{")
            obj, err = self.mod.load_object(path, context="pack-json")
            self.assertIsNone(obj)
            self.assertEqual(err["kind"], "pack-json-parse")

    def test_load_json_safe_missing(self):
        obj, err = self.mod.load_json_safe(os.path.join("definitely", "missing.json"), context="task")
        self.assertIsNone(obj)
        self.assertIsNotNone(err)
        self.assertIn(err["kind"], ("task-unreadable", "missing-packs-root", "task-parse"))

    def test_list_dir_safe_missing(self):
        names, err = self.mod.list_dir_safe(os.path.join("definitely", "missing"), context="listdir-tasks")
        self.assertEqual(names, [])
        self.assertIsNotNone(err)
        self.assertEqual(err["kind"], "missing-tasks-dir")

    def test_list_dir_safe_ok(self):
        with tempfile.TemporaryDirectory() as d:
            _write_text(os.path.join(d, "a.json"), "{}")
            names, err = self.mod.list_dir_safe(d, context="listdir-tasks")
            self.assertIsNone(err)
            self.assertIn("a.json", names)


class MissingPackJsonTests(LoadMixin):
    def test_directory_without_manifest_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "packs", "ghost", "tasks"))
            r = self.mod.audit(d)
            self.assertFalse(r["ok"])
            self.assertIn("missing-pack-json", _codes(r))
            self.assertTrue(any("pack.json 이 없다" in i for i in _pack_issue_text(r)))
            self.assertEqual(r["exit"], 1)
            self.assertFalse(r["toolFailed"])

    def test_missing_manifest_does_not_scan_sibling_tasks(self):
        with tempfile.TemporaryDirectory() as d:
            pd = os.path.join(d, "packs", "ghost")
            os.makedirs(os.path.join(pd, "tasks"))
            _write_json(os.path.join(pd, "tasks", "X01.json"), _task("X01"))
            r = self.mod.audit(d)
            self.assertIn("missing-pack-json", _codes(r))
            self.assertNotIn("missing-reference", _codes(r))
            self.assertEqual(r["taskCount"], 0)

    def test_manifest_as_directory_is_unreadable(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "packs", "ghost", "pack.json"))
            r = self.mod.audit(d)
            self.assertFalse(r["ok"])
            self.assertIn("pack-json-unreadable", _codes(r))


class OrphanAndPairingTests(LoadMixin):
    def test_orphan_has_structured_code(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=True)
            os.remove(os.path.join(d, "packs", "p1", "tasks", "X01.json"))
            r = self.mod.audit(d)
            self.assertIn("orphan-reference", _codes(r))
            self.assertIn("empty-pack", _codes(r))
            self.assertTrue(any("고아" in i for i in _pack_issue_text(r)))
            self.assertIn("p1", r["emptyPacks"])

    def test_missing_reference_has_structured_code(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            r = self.mod.audit(d)
            self.assertIn("missing-reference", _codes(r))
            self.assertTrue(any("해결 가능성" in i for i in _pack_issue_text(r)))

    def test_reference_id_mismatch(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=True, ref=_ref("OTHER"))
            r = self.mod.audit(d)
            self.assertFalse(r["ok"])
            self.assertIn("reference-id-mismatch", _codes(r))
            self.assertTrue(any("OTHER" in i and "X01" in i for i in _pack_issue_text(r)))

    def test_pairing_is_by_filename_not_id(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=True, task=_task("Y99"), ref=_ref("Y99"))
            r = self.mod.audit(d)
            self.assertIn("task-filename-id-mismatch", _codes(r))
            self.assertNotIn("missing-reference", _codes(r))
            self.assertNotIn("orphan-reference", _codes(r))

    def test_non_json_files_are_ignored(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=True)
            _write_text(os.path.join(d, "packs", "p1", "tasks", "notes.txt"), "nope")
            _write_text(os.path.join(d, "packs", "p1", "reference", "readme.md"), "nope")
            _write_text(os.path.join(d, "packs", "p1", "tasks", "X02.JSON"), "{}")
            r = self.mod.audit(d)
            self.assertTrue(r["ok"], r["packs"])
            self.assertEqual(r["taskCount"], 1)

    def test_two_missing_refs_are_two_issues(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            _write_json(os.path.join(d, "packs", "p1", "tasks", "X02.json"), _task("X02"))
            r = self.mod.audit(d)
            self.assertEqual(_codes(r).count("missing-reference"), 2)


class DuplicateIdTests(LoadMixin):
    def test_cross_pack_collision_is_global(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "DUP")
            _write_pack(d, "p2", "DUP")
            r = self.mod.audit(d)
            self.assertEqual(r["taskIdCollisions"]["DUP"], ["p1", "p2"])
            self.assertIn("task-id-collision", _codes(r))
            self.assertGreaterEqual(r["issueCount"], 1)

    def test_in_pack_duplicate_is_not_global_collision(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "A01")
            _write_json(os.path.join(d, "packs", "p1", "tasks", "A01b.json"), _task("A01"))
            _write_json(os.path.join(d, "packs", "p1", "reference", "A01b.json"), _ref("A01"))
            r = self.mod.audit(d)
            self.assertNotIn("A01", r["taskIdCollisions"])
            self.assertIn("task-id-duplicate-in-pack", _codes(r))
            self.assertTrue(any("여러 파일" in i for i in _pack_issue_text(r)))

    def test_three_pack_collision_preserves_order(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "aa", "Z")
            _write_pack(d, "mm", "Z")
            _write_pack(d, "zz", "Z")
            r = self.mod.audit(d)
            self.assertEqual(r["taskIdCollisions"]["Z"], ["aa", "mm", "zz"])

    def test_distinct_ids_do_not_collide(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "A01")
            _write_pack(d, "p2", "B01")
            _write_pack(d, "p3", "C01")
            r = self.mod.audit(d)
            self.assertEqual(r["taskIdCollisions"], {})
            self.assertTrue(r["ok"])


class BadSchemaTests(LoadMixin):
    def test_wrong_kind(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["kind"] = "notPack"
            _write_pack(d, "p1", "X01", manifest=man)
            r = self.mod.audit(d)
            self.assertIn("bad-schema", _codes(r))
            self.assertTrue(any(i.get("schemaTag") == "kind" for i in r["issues"] if i.get("code") == "bad-schema"))

    def test_wrong_schema_version(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["schemaVersion"] = "9.9"
            _write_pack(d, "p1", "X01", manifest=man)
            r = self.mod.audit(d)
            self.assertTrue(any(i.get("schemaTag") == "schemaVersion" for i in r["issues"]))

    def test_pack_id_mismatch_folder(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("other")
            _write_pack(d, "p1", "X01", manifest=man)
            r = self.mod.audit(d)
            self.assertTrue(any(i.get("schemaTag") == "pack-id" for i in r["issues"]))

    def test_empty_title_and_axis(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["title"] = ""
            man["axis"] = ""
            _write_pack(d, "p1", "X01", manifest=man)
            tags = {i.get("schemaTag") for i in r_issues(self.mod.audit(d))}
            self.assertIn("title", tags)
            self.assertIn("axis", tags)

    def test_empty_requires_commands(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["requires"] = {"commands": []}
            _write_pack(d, "p1", "X01", manifest=man)
            self.assertTrue(any(i.get("schemaTag") == "requires" for i in r_issues(self.mod.audit(d))))

    def test_missing_runner_fields(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["runner"] = {}
            _write_pack(d, "p1", "X01", manifest=man)
            tags = [i.get("schemaTag") for i in r_issues(self.mod.audit(d))]
            self.assertGreaterEqual(tags.count("runner"), 1)

    def test_task_missing_required_key(self):
        with tempfile.TemporaryDirectory() as d:
            task = _task("X01")
            del task["checks"]
            _write_pack(d, "p1", "X01", task=task)
            r = self.mod.audit(d)
            tags = {i.get("schemaTag") for i in r["issues"] if i.get("code") == "bad-schema"}
            self.assertTrue("task-required" in tags or "checks-empty" in tags)

    def test_task_bad_tier(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", task=_task("X01", tier=9))
            r = self.mod.audit(d)
            self.assertTrue(any(i.get("schemaTag") == "tier" for i in r["issues"]))

    def test_task_unknown_operator(self):
        with tempfile.TemporaryDirectory() as d:
            task = _task("X01")
            task["checks"] = [{"op": "not_a_real_op", "cmd": ["info"]}]
            _write_pack(d, "p1", "X01", task=task)
            r = self.mod.audit(d)
            self.assertTrue(any(i.get("schemaTag") == "unknown-op" for i in r["issues"]))

    def test_editing_global_scan_is_schema(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["axis"] = "편집 (좌표 지정)"
            task = _task("X01")
            task["checks"] = [{
                "op": "deep_contains",
                "cmd": ["export-text", "{input}", "--json"],
                "needle": "x",
            }]
            _write_pack(d, "p1", "X01", manifest=man, task=task)
            r = self.mod.audit(d)
            self.assertTrue(any(i.get("schemaTag") == "global-scan" for i in r["issues"]))

    def test_cli_op_missing_cmd(self):
        with tempfile.TemporaryDirectory() as d:
            task = _task("X01")
            task["checks"] = [{"op": "answer_eq", "answer": "p", "path": "pageCount"}]
            _write_pack(d, "p1", "X01", task=task)
            r = self.mod.audit(d)
            self.assertTrue(any(i.get("schemaTag") == "missing-cmd" for i in r["issues"]))

    def test_file_op_with_cmd(self):
        with tempfile.TemporaryDirectory() as d:
            task = _task("X01")
            task["checks"] = [{"op": "file_exists", "cmd": ["info"]}]
            _write_pack(d, "p1", "X01", task=task)
            r = self.mod.audit(d)
            self.assertTrue(any(i.get("schemaTag") == "unexpected-cmd" for i in r["issues"]))

    def test_empty_task_id(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", task=_task(""), ref=_ref(""))
            r = self.mod.audit(d)
            self.assertIn("task-empty-id", _codes(r))

    def test_schema_failures_keep_legacy_packs_issues(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["kind"] = "x"
            _write_pack(d, "p1", "X01", manifest=man)
            r = self.mod.audit(d)
            self.assertTrue(_pack_issue_text(r))
            self.assertTrue(any("kind" in i for i in _pack_issue_text(r)))


def r_issues(report):
    return [i for i in report.get("issues") or [] if isinstance(i, dict)]


class ParseAndShapeTests(LoadMixin):
    def test_pack_json_invalid(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_text(os.path.join(d, "packs", "p1", "pack.json"), "{")
            r = self.mod.audit(d)
            self.assertIn("pack-json-parse", _codes(r))
            self.assertTrue(any("파싱 실패" in i for i in _pack_issue_text(r)))

    def test_pack_json_array(self):
        with tempfile.TemporaryDirectory() as d:
            pd = os.path.join(d, "packs", "p1")
            os.makedirs(os.path.join(pd, "tasks"))
            _write_text(os.path.join(pd, "pack.json"), "[1]")
            r = self.mod.audit(d)
            self.assertIn("pack-json-not-object", _codes(r))

    def test_pack_json_number(self):
        with tempfile.TemporaryDirectory() as d:
            pd = os.path.join(d, "packs", "p1")
            os.makedirs(pd)
            _write_text(os.path.join(pd, "pack.json"), "3")
            r = self.mod.audit(d)
            self.assertIn("pack-json-not-object", _codes(r))

    def test_task_invalid_json(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_text(os.path.join(d, "packs", "p1", "tasks", "X01.json"), "{")
            r = self.mod.audit(d)
            self.assertIn("task-parse", _codes(r))
            self.assertTrue(any("tasks/X01.json 파싱 실패" in i for i in _pack_issue_text(r)))

    def test_task_array(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_text(os.path.join(d, "packs", "p1", "tasks", "X01.json"), "[]")
            r = self.mod.audit(d)
            self.assertIn("task-not-object", _codes(r))

    def test_reference_invalid_json(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_text(os.path.join(d, "packs", "p1", "reference", "X01.json"), "{")
            r = self.mod.audit(d)
            self.assertIn("reference-parse", _codes(r))

    def test_reference_array(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_text(os.path.join(d, "packs", "p1", "reference", "X01.json"), "[]")
            r = self.mod.audit(d)
            self.assertIn("reference-not-object", _codes(r))

    def test_non_utf8_pack_json(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_bytes(os.path.join(d, "packs", "p1", "pack.json"), b"\xff\xfe{")
            r = self.mod.audit(d)
            self.assertFalse(r["ok"])
            codes = set(_codes(r))
            self.assertTrue(codes & {"pack-json-parse", "pack-json-unreadable"})

    def test_audit_does_not_raise_on_broken_json(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_text(os.path.join(d, "packs", "p1", "tasks", "X01.json"), "not-json")
            try:
                r = self.mod.audit(d)
            except Exception as exc:  # noqa: BLE001
                self.fail(f"audit raised {exc}")
            self.assertFalse(r["ok"])


class RootExceptionTests(LoadMixin):
    def test_missing_packs_root_is_tool_failure(self):
        with tempfile.TemporaryDirectory() as d:
            r = self.mod.audit(os.path.join(d, "no-such-gym"))
            self.assertTrue(r["missingPacksRoot"])
            self.assertTrue(r["toolFailed"])
            self.assertFalse(r["ok"])
            self.assertEqual(r["exit"], 2)
            self.assertIn("missing-packs-root", _codes(r))
            self.assertEqual(r["packCount"], 0)

    def test_packs_as_file_is_tool_failure(self):
        with tempfile.TemporaryDirectory() as d:
            _write_text(os.path.join(d, "packs"), "not-a-dir")
            r = self.mod.audit(d)
            self.assertTrue(r["toolFailed"])
            self.assertEqual(r["exit"], 2)
            self.assertIn("packs-not-dir", _codes(r))

    def test_empty_packs_dir_is_ok_zero(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "packs"))
            r = self.mod.audit(d)
            self.assertTrue(r["ok"])
            self.assertEqual(r["packCount"], 0)
            self.assertEqual(r["issueCount"], 0)
            self.assertEqual(r["exit"], 0)
            self.assertEqual(self.mod.validate_report(r), [])

    def test_loose_file_in_packs_is_ignored(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "A01")
            _write_text(os.path.join(d, "packs", "notes.txt"), "ignore")
            r = self.mod.audit(d)
            self.assertTrue(r["ok"])
            self.assertEqual(r["packCount"], 1)

    def test_tasks_as_file(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            tasks = os.path.join(d, "packs", "p1", "tasks")
            for name in os.listdir(tasks):
                os.remove(os.path.join(tasks, name))
            os.rmdir(tasks)
            _write_text(tasks, "file")
            r = self.mod.audit(d)
            self.assertIn("tasks-not-dir", _codes(r))

    def test_reference_as_file(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            ref = os.path.join(d, "packs", "p1", "reference")
            for name in os.listdir(ref):
                os.remove(os.path.join(ref, name))
            os.rmdir(ref)
            _write_text(ref, "file")
            r = self.mod.audit(d)
            self.assertIn("reference-not-dir", _codes(r))

    def test_missing_tasks_dir_with_orphan_ref(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            tasks = os.path.join(d, "packs", "p1", "tasks")
            for name in os.listdir(tasks):
                os.remove(os.path.join(tasks, name))
            os.rmdir(tasks)
            r = self.mod.audit(d)
            self.assertIn("orphan-reference", _codes(r))
            self.assertIn("empty-pack", _codes(r))

    def test_audit_none_root_does_not_raise(self):
        r = self.mod.audit(None)
        self.assertFalse(r["ok"])
        self.assertTrue(r["toolFailed"] or r["missingPacksRoot"])


class ReportShapeTests(LoadMixin):
    def test_clean_report_validates(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "A01")
            _write_pack(d, "p2", "B01")
            r = self.mod.audit(d)
            self.assertEqual(self.mod.validate_report(r), [])
            self.assertTrue(r["ok"])
            self.assertEqual(r["packCount"], 2)
            self.assertEqual(r["taskCount"], 2)
            self.assertEqual(r["referenceCount"], 2)
            self.assertEqual(set(r["okPacks"]), {"p1", "p2"})
            self.assertEqual(r["packs"], [])
            self.assertEqual(r["taskIdCollisions"], {})
            self.assertEqual(r["issueCount"], 0)
            self.assertEqual(r["exit"], 0)

    def test_dirty_packs_only_lists_offenders(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "clean", "A01")
            _write_pack(d, "dirty", "B01", with_ref=False)
            r = self.mod.audit(d)
            ids = [p["id"] for p in r["packs"]]
            self.assertEqual(ids, ["dirty"])
            self.assertIn("clean", r["okPacks"])
            self.assertNotIn("dirty", r["okPacks"])

    def test_issue_count_is_pack_lines_plus_collisions(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "DUP")
            _write_pack(d, "p2", "DUP")
            r = self.mod.audit(d)
            pack_lines = sum(len(p["issues"]) for p in r["packs"])
            self.assertEqual(r["issueCount"], pack_lines + len(r["taskIdCollisions"]))

    def test_issue_counts_by_code_and_family(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            r = self.mod.audit(d)
            self.assertGreaterEqual(r["issueCountsByCode"].get("missing-reference", 0), 1)
            self.assertGreaterEqual(r["issueCountsByFamily"].get("pairing", 0), 1)

    def test_human_report_ok(self):
        text = self.mod.format_human_report(self.mod.empty_report())
        self.assertIn("전부 통과", text)
        self.assertIn("위반 0", text)

    def test_human_report_violations(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            r = self.mod.audit(d)
            text = self.mod.format_human_report(r)
            self.assertIn("위반", text)
            self.assertIn("[p1]", text)
            self.assertIn("기준풀이", text)

    def test_human_report_collision(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "DUP")
            _write_pack(d, "p2", "DUP")
            text = self.mod.format_human_report(self.mod.audit(d))
            self.assertIn("[전역]", text)
            self.assertIn("DUP", text)
            self.assertIn("p1", text)
            self.assertIn("p2", text)

    def test_human_report_tool_failure(self):
        r = self.mod.empty_report()
        r["ok"] = False
        r["toolFailed"] = True
        r["missingPacksRoot"] = True
        r["toolErrors"] = [{"head": "packs 루트가 없다", "kind": "missing-packs-root"}]
        text = self.mod.format_human_report(r)
        self.assertIn("도구 실패", text)

    def test_human_report_broken_input(self):
        self.assertIn("손상", self.mod.format_human_report("nope"))

    def test_json_report_is_object(self):
        raw = self.mod.format_json_report(self.mod.empty_report())
        parsed = json.loads(raw)
        self.assertEqual(parsed["kind"], "gymAudit")
        self.assertTrue(raw.endswith("\n"))

    def test_real_repo_report_validates_and_counts(self):
        r = self.mod.audit(str(REPO_ROOT / "gym"))
        self.assertEqual(self.mod.validate_report(r), [])
        self.assertGreaterEqual(r["taskCount"], 80)
        self.assertEqual(r["taskCount"], r["referenceCount"])
        self.assertGreaterEqual(len(r["okPacks"]), 10)
        self.assertFalse(r["toolFailed"])
        self.assertFalse(r["missingPacksRoot"])


class CliTests(LoadMixin):
    def test_main_json_ok(self):
        buf = io.StringIO()
        with mock.patch.object(self.mod, "GYM_ROOT", str(REPO_ROOT / "gym")):
            with mock.patch.object(sys, "stdout", buf):
                code = self.mod.main(["--json"])
        self.assertEqual(code, 0)
        parsed = json.loads(buf.getvalue())
        self.assertTrue(parsed["ok"])
        self.assertEqual(parsed["kind"], "gymAudit")

    def test_main_human_ok(self):
        buf = io.StringIO()
        with mock.patch.object(self.mod, "GYM_ROOT", str(REPO_ROOT / "gym")):
            with mock.patch.object(sys, "stdout", buf):
                code = self.mod.main([])
        self.assertEqual(code, 0)
        self.assertIn("전부 통과", buf.getvalue())

    def test_main_json_violation_exit_1(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            buf = io.StringIO()
            with mock.patch.object(self.mod, "GYM_ROOT", d):
                with mock.patch.object(sys, "stdout", buf):
                    code = self.mod.main(["--json"])
            self.assertEqual(code, 1)
            parsed = json.loads(buf.getvalue())
            self.assertFalse(parsed["ok"])
            self.assertIn("missing-reference", [i["code"] for i in parsed["issues"]])

    def test_main_human_violation_exit_1(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            buf = io.StringIO()
            with mock.patch.object(self.mod, "GYM_ROOT", d):
                with mock.patch.object(sys, "stdout", buf):
                    code = self.mod.main([])
            self.assertEqual(code, 1)
            self.assertIn("위반", buf.getvalue())

    def test_main_missing_root_exit_2(self):
        with tempfile.TemporaryDirectory() as d:
            buf = io.StringIO()
            with mock.patch.object(self.mod, "GYM_ROOT", os.path.join(d, "missing")):
                with mock.patch.object(sys, "stdout", buf):
                    code = self.mod.main(["--json"])
            self.assertEqual(code, 2)
            parsed = json.loads(buf.getvalue())
            self.assertTrue(parsed["toolFailed"])

    def test_main_unknown_flag_exits(self):
        with self.assertRaises(SystemExit):
            self.mod.main(["--nope"])


class FatalAndFoldTests(LoadMixin):
    def test_audit_one_pack_does_not_swallow_keyboard_interrupt(self):
        with tempfile.TemporaryDirectory() as d:
            pd = _write_pack(d, "p1", "X01")

            def boom(*_a, **_k):
                raise KeyboardInterrupt()

            with mock.patch.object(self.mod, "load_object", side_effect=boom):
                with self.assertRaises(KeyboardInterrupt):
                    self.mod.audit_one_pack("p1", pd, {})

    def test_audit_does_not_swallow_system_exit(self):
        def boom(*_a, **_k):
            raise SystemExit(3)

        with mock.patch.object(self.mod, "path_kind", side_effect=boom):
            with self.assertRaises(SystemExit):
                self.mod.audit("whatever")

    def test_run_validate_pack_folds_nonfatal(self):
        def boom(*_a, **_k):
            raise RuntimeError("schema-down")

        with mock.patch.object(self.mod.schema, "validate_pack", side_effect=boom):
            msgs = self.mod.run_validate_pack(_manifest("p1"), "p1")
        self.assertTrue(any("schema-down" in m for m in msgs))

    def test_run_validate_task_folds_nonfatal(self):
        def boom(*_a, **_k):
            raise TypeError("task-down")

        with mock.patch.object(self.mod.schema, "validate_task", side_effect=boom):
            msgs = self.mod.run_validate_task(_task("X01"), _manifest("p1"))
        self.assertTrue(any("task-down" in m for m in msgs))

    def test_run_validate_pack_reraises_fatal(self):
        def boom(*_a, **_k):
            raise MemoryError()

        with mock.patch.object(self.mod.schema, "validate_pack", side_effect=boom):
            with self.assertRaises(MemoryError):
                self.mod.run_validate_pack(_manifest("p1"), "p1")

    def test_list_dir_safe_does_not_swallow_keyboard(self):
        def boom(_path):
            raise KeyboardInterrupt()

        with mock.patch.object(os, "listdir", side_effect=boom):
            with self.assertRaises(KeyboardInterrupt):
                self.mod.list_dir_safe("x", context="packs-root")

    def test_load_json_safe_does_not_swallow_generator_exit(self):
        class Boom(io.StringIO):
            def read(self, *a, **k):
                raise GeneratorExit()

        real_open = open

        def fake_open(path, *a, **k):
            if str(path).endswith(".json"):
                return Boom("{}")
            return real_open(path, *a, **k)

        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "a.json")
            _write_text(path, "{}")
            with mock.patch("builtins.open", side_effect=fake_open):
                with self.assertRaises(GeneratorExit):
                    self.mod.load_json_safe(path, context="pack-json")


class MixedPackTests(LoadMixin):
    def test_one_clean_one_missing_json_one_orphan_one_dup(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "clean", "C01")
            os.makedirs(os.path.join(d, "packs", "ghost"))
            _write_pack(d, "orphan", "O01")
            os.remove(os.path.join(d, "packs", "orphan", "tasks", "O01.json"))
            _write_pack(d, "left", "SHARE")
            _write_pack(d, "right", "SHARE")
            r = self.mod.audit(d)
            self.assertFalse(r["ok"])
            codes = set(_codes(r))
            self.assertIn("missing-pack-json", codes)
            self.assertIn("orphan-reference", codes)
            self.assertIn("task-id-collision", codes)
            self.assertIn("clean", r["okPacks"])
            self.assertIn("SHARE", r["taskIdCollisions"])
            self.assertEqual(r["exit"], 1)

    def test_bad_schema_and_missing_ref_together(self):
        with tempfile.TemporaryDirectory() as d:
            man = _manifest("p1")
            man["kind"] = "x"
            _write_pack(d, "p1", "X01", with_ref=False, manifest=man)
            r = self.mod.audit(d)
            codes = set(_codes(r))
            self.assertIn("bad-schema", codes)
            self.assertIn("missing-reference", codes)

    def test_ok_packs_sorted_like_directory_order(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "b-pack", "B01")
            _write_pack(d, "a-pack", "A01")
            r = self.mod.audit(d)
            self.assertEqual(r["okPacks"], ["a-pack", "b-pack"])

    def test_readme_next_to_pack_is_not_an_issue(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01")
            _write_text(os.path.join(d, "packs", "p1", "README.md"), "# hi")
            os.makedirs(os.path.join(d, "packs", "p1", "assets"), exist_ok=True)
            _write_text(os.path.join(d, "packs", "p1", "assets", "x.csv"), "a,b")
            r = self.mod.audit(d)
            self.assertTrue(r["ok"], r["packs"])


class SchemaWrapAndLegacyLoadTests(LoadMixin):
    def test_legacy_load_reads_object(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "x.json")
            _write_json(path, {"a": 1})
            self.assertEqual(self.mod._load(path), {"a": 1})

    def test_legacy_load_raises_on_invalid(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "x.json")
            _write_text(path, "{")
            with self.assertRaises((ValueError, json.JSONDecodeError)):
                self.mod._load(path)

    def test_append_issue_writes_both_surfaces(self):
        pack_issues = []
        structured = []
        rec = self.mod.append_issue(
            pack_issues, structured, "orphan-reference", "p1",
            path="p1/reference/X01.json", name="X01.json",
        )
        self.assertEqual(len(pack_issues), 1)
        self.assertEqual(len(structured), 1)
        self.assertIn("고아", pack_issues[0])
        self.assertEqual(rec["code"], "orphan-reference")
        self.assertEqual(structured[0]["pack"], "p1")


class DoDCoverageTests(LoadMixin):
    """이슈 #5277 가 명시한 네 자리 — 빠진 pack.json · 고아 · 중복 ID · 나쁜 스키마."""

    def test_four_issued_faults_are_independent(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "packs", "no-manifest"))
            _write_pack(d, "orphan-pack", "O01")
            os.remove(os.path.join(d, "packs", "orphan-pack", "tasks", "O01.json"))
            _write_pack(d, "dup-a", "SAME")
            _write_pack(d, "dup-b", "SAME")
            man = _manifest("bad-schema")
            man["schemaVersion"] = "0.0"
            _write_pack(d, "bad-schema", "S01", manifest=man)
            r = self.mod.audit(d)
            codes = set(_codes(r))
            self.assertIn("missing-pack-json", codes)
            self.assertIn("orphan-reference", codes)
            self.assertIn("task-id-collision", codes)
            self.assertIn("bad-schema", codes)
            self.assertFalse(r["ok"])
            self.assertEqual(r["exit"], 1)
            self.assertIn("SAME", r["taskIdCollisions"])

    def test_four_issued_faults_have_korean_pack_lines(self):
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "packs", "ghost"))
            _write_pack(d, "orphan-pack", "O01")
            os.remove(os.path.join(d, "packs", "orphan-pack", "tasks", "O01.json"))
            man = _manifest("bad")
            man["kind"] = "x"
            _write_pack(d, "bad", "S01", manifest=man)
            _write_pack(d, "a", "ID")
            _write_pack(d, "b", "ID")
            lines = "\n".join(_pack_issue_text(self.mod.audit(d)))
            self.assertIn("pack.json 이 없다", lines)
            self.assertIn("고아", lines)
            self.assertIn("kind", lines)

    def test_module_doc_forbids_new_cli(self):
        text = Path(TOOL).read_text(encoding="utf-8")
        self.assertIn("새 플래그는 없다", text)
        self.assertIn("schema.validate_pack", text)
        self.assertIn("고아", text)


class RealRepoHonestyTests(LoadMixin):
    def test_real_repo_has_no_collision_map(self):
        r = self.mod.audit(str(REPO_ROOT / "gym"))
        self.assertEqual(r["taskIdCollisions"], {})
        self.assertEqual(r["emptyPacks"], [])
        self.assertEqual(r["issueCountsByCode"], {})

    def test_every_real_pack_is_in_ok_or_dirty(self):
        r = self.mod.audit(str(REPO_ROOT / "gym"))
        listed = {name for name in os.listdir(REPO_ROOT / "gym" / "packs")
                  if (REPO_ROOT / "gym" / "packs" / name).is_dir()}
        self.assertEqual(set(r["okPacks"]) | {p["id"] for p in r["packs"]}, listed)
        self.assertEqual(len(listed), r["packCount"])

    def test_audit_py_is_the_only_cli_entry(self):
        src = Path(TOOL).read_text(encoding="utf-8")
        self.assertIn('ap.add_argument("--json"', src)
        self.assertEqual(src.count("add_argument"), 1)


if __name__ == "__main__":
    unittest.main()
