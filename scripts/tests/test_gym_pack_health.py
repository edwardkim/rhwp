"""[pack_health] gym pack 건강 감사 계약 — 지시·검사 이름·힌트·제출 형식.

audit.py 는 스키마·기준풀이 짝만 본다. 이 가드는 빈/짧은 instructions, 중복·누락
check.name, 과제 id/title 공백, 빈 reference.steps, 미지 submit.kind, 힌트 정답
노출, pack.json 신원, tier/input 위생, 연산자 필수 필드, 제출 경로, 고아
reference 를 픽스처로 고정한다. 실제 저장소 pack 은 고치지 않는다.
"""

from __future__ import annotations

import importlib.util
import io
import json
import os
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "pack_health.py"

CLEAN_INSTRUCTIONS = (
    "입력 문서의 총 쪽수를 알아내 answer.json 에 제출하라. "
    "힌트: rhwp info <문서> --json."
)

DEFAULT_CHECKS = [
    {
        "name": "쪽수 일치",
        "op": "answer_eq",
        "answer": "pages",
        "cmd": ["info", "{input}", "--json"],
        "path": "pageCount",
    }
]


def _cli_check(name="쪽수 일치", op="answer_eq", **extra):
    row = {
        "name": name,
        "op": op,
        "cmd": extra.pop("cmd", ["info", "{input}", "--json"]),
    }
    row.update(extra)
    return row


def load():
    spec = importlib.util.spec_from_file_location("gym_pack_health", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _task(
    tid="A01",
    title="쪽수 세기",
    instructions=CLEAN_INSTRUCTIONS,
    submit=None,
    checks=None,
    **extra,
):
    doc = {
        "id": tid,
        "tier": 1,
        "title": title,
        "input": "samples/x.hwp",
        "instructions": instructions,
        "submit": submit if submit is not None else {"kind": "answer"},
        "checks": checks if checks is not None else [dict(DEFAULT_CHECKS[0])],
    }
    doc.update(extra)
    return doc


def _ref(tid="A01", steps=None):
    return {
        "id": tid,
        "steps": steps
        if steps is not None
        else [{"answer": {"pages": {"cmd": ["info", "{input}", "--json"], "path": "pageCount"}}}],
    }


def _write_json(path, doc):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=2)
        fh.write("\n")


def _default_manifest(pid):
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


def _write_pack(root, pid, tasks, refs=None, manifest=True):
    pack_dir = os.path.join(root, "packs", pid)
    os.makedirs(os.path.join(pack_dir, "tasks"), exist_ok=True)
    os.makedirs(os.path.join(pack_dir, "reference"), exist_ok=True)
    if manifest:
        doc = dict(_default_manifest(pid))
        if isinstance(manifest, dict):
            doc.update(manifest)
            if "id" not in manifest:
                doc["id"] = pid
        _write_json(os.path.join(pack_dir, "pack.json"), doc)
    for task in tasks:
        tid = task.get("id") or "X"
        name = extra_name(task)
        _write_json(os.path.join(pack_dir, "tasks", name), task)
        if refs is not False:
            ref_doc = None
            if isinstance(refs, dict):
                ref_doc = refs.get(name) or refs.get(tid)
            if ref_doc is None and refs is not False:
                ref_doc = _ref(tid if isinstance(tid, str) else "X")
            if ref_doc is not None:
                _write_json(os.path.join(pack_dir, "reference", name), ref_doc)
    return pack_dir


def extra_name(task):
    """파일명. 기본은 id.json. `_filename` 으로 덮어쓸 수 있다."""
    override = task.pop("_filename", None) if isinstance(task, dict) else None
    if override:
        return override
    tid = task.get("id") if isinstance(task, dict) else None
    if isinstance(tid, str) and tid.strip() and not any(ch.isspace() for ch in tid):
        return f"{tid}.json"
    return "TASK.json"


def codes_of(report, pack_id=None):
    packs = report.get("packs") or []
    if pack_id is not None:
        packs = [p for p in packs if p["id"] == pack_id]
    return [item["code"] for p in packs for item in p.get("issues") or []]


def messages_of(report, pack_id=None):
    packs = report.get("packs") or []
    if pack_id is not None:
        packs = [p for p in packs if p["id"] == pack_id]
    return [item["message"] for p in packs for item in p.get("issues") or []]


class EnvelopeTests(unittest.TestCase):
    def test_clean_fixture_is_ok(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()])
            report = mod.audit(tmp)
        self.assertEqual(report["kind"], "gymPackHealth")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertTrue(report["ok"])
        self.assertEqual(report["issueCount"], 0)
        self.assertEqual(report["packCount"], 1)
        self.assertEqual(report["taskCount"], 1)
        self.assertEqual(report["errorCount"], 0)
        self.assertEqual(report["codes"], {})

    def test_default_exit_zero_even_with_issues(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            report = mod.audit(tmp)
            self.assertFalse(report["ok"])
            self.assertGreater(report["issueCount"], 0)
            self.assertEqual(mod.exit_status(report, strict=False), 0)
            self.assertEqual(mod.exit_status(report, strict=True), 1)


class InstructionTests(unittest.TestCase):
    def test_empty_instructions(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="   ")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_INSTRUCTIONS, codes_of(report, "p1"))

    def test_missing_instructions_key(self):
        mod = load()
        task = _task()
        del task["instructions"]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [task])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_INSTRUCTIONS, codes_of(report, "p1"))

    def test_short_instructions_under_20(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="열아홉글자짜리지시문임.")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_SHORT_INSTRUCTIONS, codes_of(report, "p1"))
        self.assertTrue(any("< 20" in m for m in messages_of(report, "p1")))

    def test_min_chars_override(self):
        mod = load()
        text = "스무글자가넘는지시문입니다!"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            ok = mod.audit(tmp, min_chars=5)
            tight = mod.audit(tmp, min_chars=80)
        self.assertTrue(ok["ok"])
        self.assertIn(mod.CODE_SHORT_INSTRUCTIONS, codes_of(tight, "p1"))

    def test_non_string_instructions(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=["리스트"])])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INSTRUCTIONS_TYPE, codes_of(report, "p1"))


class CheckNameTests(unittest.TestCase):
    def test_missing_check_name(self):
        mod = load()
        checks = [{"op": "answer_eq", "answer": "pages"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_MISSING_CHECK_NAME, codes_of(report, "p1"))

    def test_empty_check_name(self):
        mod = load()
        checks = [{"name": "  ", "op": "answer_eq"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_CHECK_NAME, codes_of(report, "p1"))

    def test_duplicate_check_name_inside_task(self):
        mod = load()
        checks = [
            {"name": "쪽수 일치", "op": "answer_eq"},
            {"name": "쪽수 일치", "op": "value_eq"},
        ]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_DUPLICATE_CHECK_NAME, codes_of(report, "p1"))
        self.assertTrue(any("2번" in m for m in messages_of(report, "p1")))

    def test_same_name_in_different_tasks_is_ok(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [
                    _task("A01", checks=[_cli_check("쪽수 일치", answer="pages")]),
                    _task("A02", checks=[_cli_check("쪽수 일치", answer="pages")]),
                ],
            )
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_DUPLICATE_CHECK_NAME, codes_of(report, "p1"))
        self.assertTrue(report["ok"])


class IdentityTests(unittest.TestCase):
    def test_task_id_leading_whitespace(self):
        mod = load()
        task = _task(tid=" A01")
        task["_filename"] = "A01.json"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [task])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_TASK_ID_WHITESPACE, codes_of(report, "p1"))

    def test_task_id_internal_whitespace(self):
        mod = load()
        task = _task(tid="A 01")
        task["_filename"] = "A01.json"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [task])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_TASK_ID_WHITESPACE, codes_of(report, "p1"))

    def test_title_edge_whitespace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(title=" 쪽수 세기 ")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_TASK_TITLE_WHITESPACE, codes_of(report, "p1"))

    def test_title_internal_space_is_ok(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(title="쪽수 세기")])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_TASK_TITLE_WHITESPACE, codes_of(report, "p1"))
        self.assertTrue(report["ok"])

    def test_id_filename_mismatch(self):
        mod = load()
        task = _task(tid="A01")
        task["_filename"] = "B01.json"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [task], refs={"B01.json": _ref("A01")})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_ID_FILENAME_MISMATCH, codes_of(report, "p1"))


class ReferenceStepTests(unittest.TestCase):
    def test_missing_reference_is_not_this_tools_job(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], refs=False)
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_EMPTY_REFERENCE_STEPS, codes_of(report, "p1"))
        self.assertTrue(report["ok"])

    def test_empty_steps_list(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], refs={"A01.json": _ref("A01", steps=[])})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_REFERENCE_STEPS, codes_of(report, "p1"))

    def test_missing_steps_key(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], refs={"A01.json": {"id": "A01"}})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_REFERENCE_STEPS, codes_of(report, "p1"))

    def test_null_steps(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], refs={"A01.json": {"id": "A01", "steps": None}})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_REFERENCE_STEPS, codes_of(report, "p1"))

    def test_vacuous_step_objects(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task()],
                refs={"A01.json": _ref("A01", steps=[{}, {"run": None}])},
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_REFERENCE_STEPS, codes_of(report, "p1"))

    def test_steps_wrong_type(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], refs={"A01.json": {"id": "A01", "steps": "run"}})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_REFERENCE_STEPS_TYPE, codes_of(report, "p1"))


class SubmitKindTests(unittest.TestCase):
    def test_unknown_submit_kind(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"kind": "folder"})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_UNKNOWN_SUBMIT_KIND, codes_of(report, "p1"))
        self.assertTrue(any("folder" in m for m in messages_of(report, "p1")))

    def test_known_kinds_pass(self):
        mod = load()
        cases = (
            ("answer", {"kind": "answer"}),
            ("artifact", {"kind": "artifact", "files": ["out.hwp"]}),
            ("pair", {"kind": "pair", "files": ["o1.hwp", "o2.hwp", "plan.json"]}),
        )
        with tempfile.TemporaryDirectory() as tmp:
            for tid, submit in cases:
                _write_pack(tmp, tid, [_task(tid=tid, submit=submit)])
            report = mod.audit(tmp)
        self.assertTrue(report["ok"], report["packs"])

    def test_missing_submit_kind(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"files": ["a"]})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_MISSING_SUBMIT_KIND, codes_of(report, "p1"))

    def test_artifact_without_files_is_warning(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"kind": "artifact"})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_ARTIFACT_WITHOUT_FILES, codes_of(report, "p1"))
        self.assertGreater(report["warningCount"], 0)


class HintHealthTests(unittest.TestCase):
    def test_command_hint_is_ok(self):
        mod = load()
        body, hint = mod.split_hint(CLEAN_INSTRUCTIONS)
        self.assertIsNotNone(hint)
        self.assertIn("쪽수", body)
        self.assertIn("rhwp info", hint)

    def test_hint_answer_json_dump(self):
        mod = load()
        text = CLEAN_INSTRUCTIONS.split("힌트:")[0] + '힌트: {"pages": 4}'
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))

    def test_hint_placeholder_json_is_ok(self):
        mod = load()
        text = '입력 문서의 총 쪽수를 알아내 제출하라. 힌트: {"pages": "<수>"}'
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))
        self.assertTrue(report["ok"])

    def test_hint_cli_syntax_with_placeholder_key_is_ok(self):
        """fill-fields 의 `{"<필드이름>": "홍길동"}` 는 문법 예시이지 정답 봉투가 아니다."""
        mod = load()
        text = (
            "첫 필드에 '홍길동' 을 채우라. "
            '힌트: rhwp edit fill-fields --data \'{"<필드이름>": "홍길동"}\'.'
        )
        checks = [_cli_check("값", op="value_eq", value="홍길동")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text, checks=checks)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))
        self.assertNotIn(mod.CODE_HINT_EMBEDS_VALUE, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_format_token_inside_command_is_ok(self):
        mod = load()
        text = "입력을 HWPX 로 변환하라. 힌트: rhwp export-hwpx <입력> conv.hwpx --verify."
        checks = [_cli_check("형식", op="value_eq", value="hwpx")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text, checks=checks)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_EMBEDS_VALUE, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_hint_spoiler_phrase(self):
        mod = load()
        text = "쪽수를 세어 제출하라. 힌트: 답은 4 이다"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_HINT_SPOILER, codes_of(report, "p1"))

    def test_anti_spoiler_warning_is_not_a_spoiler(self):
        mod = load()
        text = "쪽수를 세어 제출하라. 힌트: 정답 숫자를 과제에 박제하지 마라."
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_SPOILER, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_path_index_is_not_an_answer_array(self):
        mod = load()
        text = "첫 필드 값을 읽어 제출하라. 힌트: fields[0].value 를 읽는다."
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_standalone_answer_array_is_still_rejected(self):
        mod = load()
        text = "배열 답을 제출하라. 힌트: [4]"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))

    def test_data_argument_is_task_input_not_answer_dump(self):
        mod = load()
        text = (
            "없는 필드의 거부 결과를 제출하라. "
            "힌트: rhwp edit fill-fields --data '{\"없는필드\": \"입력값\"}' --dry-run."
        )
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_json_command_example_repeating_body_is_not_answer_dump(self):
        mod = load()
        text = (
            "전화번호에 02-720-8899를 넣어라. "
            "힌트: --data '{\"전화번호\": \"02-720-8899\"}'."
        )
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_hint_embeds_check_value(self):
        mod = load()
        text = "첫 칸을 고치라. 힌트: 셀에 계획실행을 넣으면 된다"
        checks = [{"name": "칸", "op": "cell_text_eq", "value": "계획실행"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text, checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_HINT_EMBEDS_VALUE, codes_of(report, "p1"))

    def test_expected_literal_used_as_command_argument_is_not_output_spoiler(self):
        mod = load()
        digest = "0" * 64
        text = f"기대 해시 에코를 확인하라. 힌트: replay --expect-output-sha256 {digest}."
        checks = [_cli_check("에코", op="value_eq", value=digest, cmd=["replay", digest])]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text, checks=checks)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_EMBEDS_VALUE, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_expected_value_in_body_not_hint_is_ok(self):
        """과제가 '이 값으로 바꿔라'고 본문에 쓰는 것은 힌트 유출이 아니다."""
        mod = load()
        text = "첫 칸을 '계획실행' 으로 바꿔라. 힌트: rhwp run --plan-json."
        checks = [
            _cli_check(
                "칸",
                op="cell_text_eq",
                value="계획실행",
                table=0,
                row=0,
                col=0,
                cmd=["export-tables", "{file:out.hwp}", "--json"],
            )
        ]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text, checks=checks)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_EMBEDS_VALUE, codes_of(report, "p1"))
        self.assertTrue(report["ok"])

    def test_extract_json_fragments_skips_broken_braces(self):
        mod = load()
        bits = mod.extract_json_fragments('앞 {깨짐 그리고 {"ok": 1} 뒤')
        self.assertEqual(bits, [{"ok": 1}])


class StructureTests(unittest.TestCase):
    def test_missing_pack_json(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest=False)
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_MISSING_PACK, codes_of(report, "p1"))

    def test_parse_error_on_task(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()])
            bad = os.path.join(tmp, "packs", "p1", "tasks", "Z99.json")
            with open(bad, "w", encoding="utf-8", newline="\n") as fh:
                fh.write("{not json\n")
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PARSE_ERROR, codes_of(report, "p1"))

    def test_missing_packs_dir_is_scan_error(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            report = mod.audit(tmp)
        self.assertFalse(report["ok"])
        self.assertIn("scanError", report)
        self.assertEqual(mod.exit_status(report, strict=False), 1)

    def test_duplicate_task_id_in_pack(self):
        mod = load()
        a = _task(tid="A01")
        b = _task(tid="A01")
        a["_filename"] = "A01.json"
        b["_filename"] = "A01b.json"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [a, b],
                refs={"A01.json": _ref("A01"), "A01b.json": _ref("A01")},
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_DUPLICATE_TASK_ID, codes_of(report, "p1"))

    def test_issues_are_grouped_per_pack(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "alpha", [_task("A01", instructions="짧다")])
            _write_pack(tmp, "beta", [_task("B01")])
            report = mod.audit(tmp)
        by_id = {p["id"]: p for p in report["packs"]}
        self.assertGreater(by_id["alpha"]["issueCount"], 0)
        self.assertEqual(by_id["beta"]["issueCount"], 0)
        self.assertEqual(report["packCount"], 2)

    def test_pack_filter(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "alpha", [_task("A01", instructions="짧다")])
            _write_pack(tmp, "beta", [_task("B01", instructions="짧다")])
            report = mod.audit(tmp, pack_ids=["beta"])
        self.assertEqual(report["packCount"], 1)
        self.assertEqual(report["packs"][0]["id"], "beta")

    def test_unknown_pack_id_is_reported(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()])
            report = mod.audit(tmp, pack_ids=["nope"])
        self.assertIn(mod.CODE_MISSING_PACK, codes_of(report, "nope"))


class RenderAndCliTests(unittest.TestCase):
    def test_human_render_lists_pack_and_code(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            text = mod.render_report(mod.audit(tmp))
        self.assertIn("이슈", text)
        self.assertIn("short_instructions", text)
        self.assertIn("p1/A01", text)

    def test_clean_render(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()])
            text = mod.render_report(mod.audit(tmp))
        self.assertIn("이슈 0", text)

    def test_cli_json_default_exit_zero(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["--root", tmp, "--json"])
            payload = json.loads(buf.getvalue())
        self.assertEqual(code, 0)
        self.assertEqual(payload["kind"], "gymPackHealth")
        self.assertFalse(payload["ok"])

    def test_cli_strict_exits_one(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["--root", tmp, "--strict"])
        self.assertEqual(code, 1)
        self.assertIn("short_instructions", buf.getvalue())

    def test_cli_strict_clean_is_zero(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()])
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(["--root", tmp, "--strict", "--json"])
        self.assertEqual(code, 0)
        self.assertTrue(json.loads(buf.getvalue())["ok"])

    def test_cli_bad_min_instructions(self):
        mod = load()
        err = io.StringIO()
        with redirect_stderr(err):
            code = mod.main(["--min-instructions", "0"])
        self.assertEqual(code, 2)
        self.assertIn("min-instructions", err.getvalue())


class RealRepoTests(unittest.TestCase):
    def test_real_gym_scan_does_not_crash(self):
        """현재 트리 스캔은 예외 없이 봉투를 낸다. 이슈가 있어도 기본 종료는 0."""
        mod = load()
        report = mod.audit(str(REPO_ROOT / "gym"))
        self.assertEqual(report["kind"], "gymPackHealth")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertGreaterEqual(report["packCount"], 10)
        self.assertGreaterEqual(report["taskCount"], 40)
        self.assertIn("packs", report)
        self.assertEqual(mod.exit_status(report, strict=False), 0)
        # --json 자기시험: 현재 트리가 깨끗하지 않아도 도구는 관측만 한다.
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = mod.main(["--root", str(REPO_ROOT / "gym"), "--json"])
        self.assertEqual(code, 0)
        again = json.loads(buf.getvalue())
        self.assertEqual(again["kind"], "gymPackHealth")
        self.assertEqual(again["packCount"], report["packCount"])


class UtilityTests(unittest.TestCase):
    def test_instruction_length_strips(self):
        mod = load()
        self.assertEqual(mod.instruction_length("  열글자입니다  "), 6)

    def test_has_whitespace_helpers(self):
        mod = load()
        self.assertTrue(mod.has_edge_whitespace(" A01"))
        self.assertTrue(mod.has_any_whitespace("A 01"))
        self.assertFalse(mod.has_any_whitespace("A01"))
        self.assertFalse(mod.has_edge_whitespace("쪽수 세기"))

    def test_fragment_placeholder_is_not_answer(self):
        mod = load()
        self.assertFalse(mod.fragment_looks_like_answer({"pages": "<수>"}))
        self.assertFalse(mod.fragment_looks_like_answer({"<필드이름>": "홍길동"}))
        self.assertTrue(mod.fragment_looks_like_answer({"pages": 4}))
        self.assertTrue(mod.is_placeholder_value("{input}"))

    def test_token_appears_bare_ignores_command_suffix(self):
        mod = load()
        self.assertFalse(mod.token_appears_bare("rhwp export-hwpx conv.hwpx", "hwpx"))
        self.assertTrue(mod.token_appears_bare("형식은 hwpx 다", "hwpx"))
        self.assertTrue(mod.token_appears_bare("셀에 계획실행을 넣으면", "계획실행"))


def _issues(report, pack_id=None):
    packs = report.get("packs") or []
    if pack_id is not None:
        packs = [p for p in packs if p["id"] == pack_id]
    return [item for p in packs for item in p.get("issues") or []]


class ManifestHealthTests(unittest.TestCase):
    def test_pack_kind_must_be_gym_pack(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"kind": "gymProfile"})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_KIND, codes_of(report, "p1"))

    def test_pack_schema_version(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"schemaVersion": "2.0"})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_SCHEMA_VERSION, codes_of(report, "p1"))

    def test_pack_id_must_match_folder(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"id": "other"})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_ID_MISMATCH, codes_of(report, "p1"))

    def test_pack_empty_title_and_axis(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"title": "  ", "axis": ""})
            report = mod.audit(tmp)
        found = codes_of(report, "p1")
        self.assertIn(mod.CODE_PACK_EMPTY_TITLE, found)
        self.assertIn(mod.CODE_PACK_EMPTY_AXIS, found)

    def test_pack_title_axis_whitespace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"title": " 조회 ", "axis": " 편집 "})
            report = mod.audit(tmp)
        found = codes_of(report, "p1")
        self.assertIn(mod.CODE_PACK_TITLE_WHITESPACE, found)
        self.assertIn(mod.CODE_PACK_AXIS_WHITESPACE, found)

    def test_pack_missing_requires_commands(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"requires": {}})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_MISSING_REQUIRES, codes_of(report, "p1"))

    def test_pack_empty_commands(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"requires": {"commands": []}})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_EMPTY_COMMANDS, codes_of(report, "p1"))

    def test_pack_command_type(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"requires": {"commands": [1, ""]}})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_COMMAND_TYPE, codes_of(report, "p1"))

    def test_pack_missing_runner(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], manifest={"runner": {}})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_MISSING_RUNNER, codes_of(report, "p1"))

    def test_pack_missing_runner_field(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task()],
                manifest={"runner": {"rhwpVersion": "0.8.4", "rhwpCommit": "a" * 40}},
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_MISSING_RUNNER_FIELD, codes_of(report, "p1"))
        self.assertTrue(any("capabilitiesSha256" in m for m in messages_of(report, "p1")))

    def test_pack_json_array_is_type_error(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            pack_dir = _write_pack(tmp, "p1", [_task()])
            _write_json(os.path.join(pack_dir, "pack.json"), ["not", "object"])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PACK_TYPE, codes_of(report, "p1"))


class InputAndTierTests(unittest.TestCase):
    def test_missing_tier(self):
        mod = load()
        task = _task()
        del task["tier"]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [task])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_MISSING_TIER, codes_of(report, "p1"))

    def test_tier_bool_is_not_int(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(tier=True)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_TIER_TYPE, codes_of(report, "p1"))

    def test_tier_string(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(tier="2")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_TIER_TYPE, codes_of(report, "p1"))

    def test_tier_out_of_range(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(tier=6)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_TIER_RANGE, codes_of(report, "p1"))

    def test_tier_zero(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(tier=0)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_TIER_RANGE, codes_of(report, "p1"))

    def test_valid_tiers_1_to_5(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            for n in range(1, 6):
                _write_pack(tmp, f"p{n}", [_task(tid=f"A0{n}", tier=n)])
            report = mod.audit(tmp)
        self.assertTrue(report["ok"], report["packs"])

    def test_missing_input(self):
        mod = load()
        task = _task()
        del task["input"]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [task])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_MISSING_INPUT, codes_of(report, "p1"))

    def test_empty_input(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(input="  ")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_INPUT, codes_of(report, "p1"))

    def test_input_not_string(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(input=["a.hwp", "b.hwp"])])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INPUT_TYPE, codes_of(report, "p1"))

    def test_input_edge_whitespace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(input=" samples/x.hwp ")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INPUT_WHITESPACE, codes_of(report, "p1"))

    def test_input_absolute_posix(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(input="/tmp/x.hwp")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INPUT_ABSOLUTE, codes_of(report, "p1"))

    def test_input_absolute_windows(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(input="C:\\docs\\x.hwp")])
            report = mod.audit(tmp)
        found = codes_of(report, "p1")
        self.assertIn(mod.CODE_INPUT_ABSOLUTE, found)
        self.assertIn(mod.CODE_INPUT_BACKSLASH, found)

    def test_input_parent_traversal(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(input="../secret/x.hwp")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INPUT_PARENT, codes_of(report, "p1"))

    def test_title_too_short_respects_min(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(title="가")])
            ok = mod.audit(tmp, min_title=1)
            tight = mod.audit(tmp, min_title=3)
        self.assertTrue(ok["ok"])
        self.assertIn(mod.CODE_TITLE_TOO_SHORT, codes_of(tight, "p1"))


class CheckContractTests(unittest.TestCase):
    def test_check_name_edge_whitespace(self):
        mod = load()
        checks = [_cli_check(" 쪽수 일치 ", answer="pages")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_NAME_WHITESPACE, codes_of(report, "p1"))

    def test_missing_check_op(self):
        mod = load()
        checks = [{"name": "쪽수 일치", "answer": "pages"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_MISSING_CHECK_OP, codes_of(report, "p1"))

    def test_empty_check_op(self):
        mod = load()
        checks = [{"name": "쪽수 일치", "op": "  "}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_CHECK_OP, codes_of(report, "p1"))

    def test_unknown_check_op(self):
        mod = load()
        checks = [_cli_check("쪽수", op="looks_ok", answer="pages")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_UNKNOWN_CHECK_OP, codes_of(report, "p1"))
        self.assertTrue(any("looks_ok" in m for m in messages_of(report, "p1")))

    def test_answer_eq_requires_answer(self):
        mod = load()
        checks = [_cli_check("쪽수", op="answer_eq")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_MISSING_ANSWER, codes_of(report, "p1"))

    def test_value_eq_requires_value(self):
        mod = load()
        checks = [_cli_check("값", op="value_eq")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_MISSING_VALUE, codes_of(report, "p1"))

    def test_value_in_requires_plural_values(self):
        mod = load()
        valid = [_cli_check("집합", op="value_in", values=["a", "b"])]
        missing = [_cli_check("집합", op="value_in")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "ok", [_task(checks=valid)])
            _write_pack(tmp, "bad", [_task(checks=missing)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_CHECK_MISSING_VALUE, codes_of(report, "ok"))
        self.assertIn(mod.CODE_CHECK_MISSING_VALUE, codes_of(report, "bad"))

    def test_file_op_requires_file(self):
        mod = load()
        checks = [{"name": "존재", "op": "file_exists"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_MISSING_FILE, codes_of(report, "p1"))

    def test_file_op_rejects_absolute_and_backslash(self):
        mod = load()
        checks = [{"name": "존재", "op": "file_exists", "file": "C:\\tmp\\out.hwp"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        found = codes_of(report, "p1")
        self.assertIn(mod.CODE_CHECK_FILE_ABSOLUTE, found)
        self.assertIn(mod.CODE_CHECK_FILE_BACKSLASH, found)

    def test_same_hash_requires_two_files(self):
        mod = load()
        checks = [{"name": "동일", "op": "same_hash", "files": ["a.hwp"]}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_FILES_SHORT, codes_of(report, "p1"))

    def test_same_hash_missing_files(self):
        mod = load()
        checks = [{"name": "동일", "op": "same_hash"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_MISSING_FILES, codes_of(report, "p1"))

    def test_cli_op_requires_cmd(self):
        mod = load()
        checks = [{"name": "쪽수", "op": "answer_eq", "answer": "pages"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_MISSING_CMD, codes_of(report, "p1"))

    def test_cli_cmd_must_be_list(self):
        mod = load()
        checks = [_cli_check("쪽수", answer="pages", cmd="info")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_CMD_TYPE, codes_of(report, "p1"))

    def test_cli_cmd_empty_list(self):
        mod = load()
        checks = [_cli_check("쪽수", answer="pages", cmd=[])]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_CMD_EMPTY, codes_of(report, "p1"))

    def test_cli_cmd_item_must_be_string(self):
        mod = load()
        checks = [_cli_check("쪽수", answer="pages", cmd=["info", 3])]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_CMD_ITEM_TYPE, codes_of(report, "p1"))

    def test_file_op_rejects_cmd(self):
        mod = load()
        checks = [
            {
                "name": "존재",
                "op": "file_exists",
                "file": "out.hwp",
                "cmd": ["info", "{file:out.hwp}"],
            }
        ]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CHECK_UNEXPECTED_CMD, codes_of(report, "p1"))

    def test_cell_text_requires_coords(self):
        mod = load()
        checks = [_cli_check("칸", op="cell_text_eq", value="계획실행")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CELL_MISSING_COORD, codes_of(report, "p1"))

    def test_cell_text_bool_coord_rejected(self):
        mod = load()
        checks = [
            _cli_check(
                "칸",
                op="cell_text_eq",
                value="계획실행",
                table=True,
                row=0,
                col=0,
            )
        ]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CELL_MISSING_COORD, codes_of(report, "p1"))

    def test_csv_cell_requires_coords(self):
        mod = load()
        checks = [{"name": "셀", "op": "csv_cell_eq", "file": "out.csv", "value": "a"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_CSV_MISSING_COORD, codes_of(report, "p1"))

    def test_global_scan_on_editing_axis_needs_allow(self):
        mod = load()
        checks = [_cli_check("훑기", op="deep_contains", value="비밀")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task(checks=checks, axis="편집 (표 좌표 지정)")],
                manifest={"axis": "편집 (표 좌표 지정)"},
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_GLOBAL_SCAN_UNDECLARED, codes_of(report, "p1"))

    def test_global_scan_allowed_with_reason(self):
        mod = load()
        checks = [
            _cli_check(
                "훑기",
                op="deep_contains",
                value="비밀",
                allowGlobalScan="은닉 문구가 좌표를 갖지 않는다",
            )
        ]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task(checks=checks, axis="편집 (표 좌표 지정)")],
                manifest={"axis": "편집 (표 좌표 지정)"},
            )
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_GLOBAL_SCAN_UNDECLARED, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_global_scan_on_inquiry_axis_is_ok(self):
        mod = load()
        checks = [_cli_check("훑기", op="not_contains", value="TODO")]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(checks=checks)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_GLOBAL_SCAN_UNDECLARED, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_well_formed_file_and_pair_ops_pass(self):
        mod = load()
        checks = [
            {"name": "존재", "op": "file_exists", "file": "out.hwp"},
            {"name": "원본과 다름", "op": "differs_from_input", "file": "out.hwp"},
            {"name": "동일", "op": "same_hash", "files": ["o1.hwp", "o2.hwp"]},
            {
                "name": "칸",
                "op": "csv_cell_eq",
                "file": "t.csv",
                "row": 1,
                "col": 0,
                "value": "앞칸",
            },
        ]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task(submit={"kind": "artifact", "files": ["out.hwp", "t.csv"]}, checks=checks)],
            )
            report = mod.audit(tmp)
        self.assertTrue(report["ok"], report["packs"])


class SubmitPathTests(unittest.TestCase):
    def test_submit_files_must_be_list(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"kind": "answer", "files": "answer.json"})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_SUBMIT_FILES_TYPE, codes_of(report, "p1"))

    def test_submit_file_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"kind": "artifact", "files": ["", "out.hwp"]})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_SUBMIT_FILE_EMPTY, codes_of(report, "p1"))

    def test_submit_file_not_string(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"kind": "artifact", "files": [1]})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_SUBMIT_FILE_TYPE, codes_of(report, "p1"))

    def test_submit_file_whitespace(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"kind": "artifact", "files": [" out.hwp "]})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_SUBMIT_FILE_WHITESPACE, codes_of(report, "p1"))

    def test_submit_file_absolute_and_backslash(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task(submit={"kind": "artifact", "files": ["D:\\out\\edited.hwp"]})],
            )
            report = mod.audit(tmp)
        found = codes_of(report, "p1")
        self.assertIn(mod.CODE_SUBMIT_FILE_ABSOLUTE, found)
        self.assertIn(mod.CODE_SUBMIT_FILE_BACKSLASH, found)

    def test_submit_file_duplicate(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task(submit={"kind": "artifact", "files": ["out.hwp", "out.hwp"]})],
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_SUBMIT_FILE_DUPLICATE, codes_of(report, "p1"))

    def test_pair_without_files_still_warning(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(submit={"kind": "pair", "files": ["only.hwp"]})])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_PAIR_WITHOUT_FILES, codes_of(report, "p1"))
        self.assertGreater(report["warningCount"], 0)


class InstructionQualityTests(unittest.TestCase):
    def test_hint_only_instructions(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="힌트: rhwp info <문서> --json.")])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INSTRUCTIONS_HINT_ONLY, codes_of(report, "p1"))

    def test_empty_hint_after_marker(self):
        mod = load()
        text = "입력 문서의 총 쪽수를 알아내 answer.json 에 제출하라. 힌트:"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_HINT, codes_of(report, "p1"))
        self.assertGreater(report["warningCount"], 0)

    def test_duplicate_hint_marker(self):
        mod = load()
        text = (
            "입력 문서의 총 쪽수를 알아내 제출하라. "
            "힌트: rhwp info. 힌트: 다시 적지 마라."
        )
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_DUPLICATE_HINT_MARKER, codes_of(report, "p1"))

    def test_parenthetical_hint_is_not_duplicate_tail(self):
        """T06 처럼 본문 `(힌트: export-text)` 는 꼬리 마커가 아니다."""
        mod = load()
        text = (
            "입력 문서에서 문구를 스스로 찾아(힌트: export-text) "
            "그 문구를 바꾼 산출물을 제출하라. 힌트: rhwp edit replace-text."
        )
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_DUPLICATE_HINT_MARKER, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])
        body, hint = mod.split_hint(text)
        self.assertIn("export-text", body)
        self.assertIsNotNone(hint)
        self.assertIn("replace-text", hint)

    def test_todo_placeholder(self):
        mod = load()
        text = "TODO: 여기에 쪽수 세기 지시문을 작성한다. 충분히 길게."
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INSTRUCTIONS_TODO, codes_of(report, "p1"))

    def test_fixme_placeholder(self):
        mod = load()
        text = "FIXME 자리표를 지우고 실제 과제를 적어라. 스무 글자는 넘긴다."
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INSTRUCTIONS_TODO, codes_of(report, "p1"))

    def test_control_character(self):
        mod = load()
        text = "입력 문서의 총 쪽수를 알아내 제출하라.\x00숨은문자"
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_INSTRUCTIONS_CONTROL_CHAR, codes_of(report, "p1"))

    def test_newline_and_tab_are_not_control_issues(self):
        mod = load()
        text = "입력 문서의 총 쪽수를 알아내 제출하라.\n힌트:\trhwp info <문서> --json."
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_INSTRUCTIONS_CONTROL_CHAR, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])


class ReferenceDetailTests(unittest.TestCase):
    def test_reference_id_mismatch(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task()], refs={"A01.json": _ref("B99")})
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_REFERENCE_ID_MISMATCH, codes_of(report, "p1"))

    def test_reference_step_not_object(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task()],
                refs={"A01.json": {"id": "A01", "steps": ["run info"]}},
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_REFERENCE_STEP_TYPE, codes_of(report, "p1"))

    def test_reference_run_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task()],
                refs={
                    "A01.json": {
                        "id": "A01",
                        "steps": [{"run": []}, {"run": ["info", "{input}", "--json"]}],
                    }
                },
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_REFERENCE_RUN_EMPTY, codes_of(report, "p1"))

    def test_reference_answer_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task()],
                refs={
                    "A01.json": {
                        "id": "A01",
                        "steps": [{"write_json": {"file": "out.json"}, "answer": {}}],
                    }
                },
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_REFERENCE_ANSWER_EMPTY, codes_of(report, "p1"))

    def test_reference_cmd_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(
                tmp,
                "p1",
                [_task()],
                refs={
                    "A01.json": {
                        "id": "A01",
                        "steps": [{"answer": {"pages": {"cmd": [], "path": "pageCount"}}}],
                    }
                },
            )
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_REFERENCE_CMD_EMPTY, codes_of(report, "p1"))

    def test_orphan_reference(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            pack_dir = _write_pack(tmp, "p1", [_task()])
            _write_json(os.path.join(pack_dir, "reference", "Z99.json"), _ref("Z99"))
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_ORPHAN_REFERENCE, codes_of(report, "p1"))

    def test_empty_pack_is_warning(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            pack_dir = _write_pack(tmp, "p1", [_task()])
            os.remove(os.path.join(pack_dir, "tasks", "A01.json"))
            os.remove(os.path.join(pack_dir, "reference", "A01.json"))
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_EMPTY_PACK, codes_of(report, "p1"))
        self.assertGreater(report["warningCount"], 0)


class CatalogAndCliExtraTests(unittest.TestCase):
    def test_catalog_covers_all_code_constants(self):
        mod = load()
        codes = set(mod.catalog_codes())
        exported = [
            value
            for name, value in vars(mod).items()
            if name.startswith("CODE_") and isinstance(value, str)
        ]
        missing = [item for item in exported if item not in codes]
        self.assertEqual(missing, [], missing)
        self.assertGreaterEqual(len(codes), 70)

    def test_catalog_rows_have_severity_and_layer(self):
        mod = load()
        for code, severity, layer, summary in mod.ISSUE_CATALOG:
            self.assertIn(severity, (mod.SEVERITY_ERROR, mod.SEVERITY_WARNING), code)
            self.assertTrue(layer, code)
            self.assertTrue(summary, code)

    def test_cli_codes_lists_catalog(self):
        mod = load()
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = mod.main(["--codes"])
        text = buf.getvalue()
        self.assertEqual(code, 0)
        self.assertIn("empty_instructions", text)
        self.assertIn("unknown_check_op", text)
        self.assertIn("pack_id_mismatch", text)
        self.assertIn("severity", text)

    def test_exclude_code_drops_issue(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            full = mod.audit(tmp)
            trimmed = mod.audit(tmp, exclude_codes=[mod.CODE_SHORT_INSTRUCTIONS])
        self.assertIn(mod.CODE_SHORT_INSTRUCTIONS, codes_of(full, "p1"))
        self.assertNotIn(mod.CODE_SHORT_INSTRUCTIONS, codes_of(trimmed, "p1"))
        self.assertIn("excludedCodes", trimmed)

    def test_cli_exclude_and_json(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = mod.main(
                    ["--root", tmp, "--json", "--exclude", "short_instructions"]
                )
            payload = json.loads(buf.getvalue())
        self.assertEqual(code, 0)
        self.assertNotIn("short_instructions", payload.get("codes") or {})

    def test_cli_bad_min_title(self):
        mod = load()
        err = io.StringIO()
        with redirect_stderr(err):
            code = mod.main(["--min-title", "0"])
        self.assertEqual(code, 2)
        self.assertIn("min-title", err.getvalue())

    def test_render_includes_severity_and_code_summary(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            text = mod.render_report(mod.audit(tmp))
        self.assertIn("error", text)
        self.assertIn("코드 집계", text)
        self.assertIn("short_instructions", text)

    def test_path_helpers(self):
        mod = load()
        self.assertTrue(mod.is_absolute_path("/tmp/a.hwp"))
        self.assertTrue(mod.is_absolute_path("D:\\a.hwp"))
        self.assertFalse(mod.is_absolute_path("samples/a.hwp"))
        self.assertTrue(mod.has_backslash("samples\\a.hwp"))
        self.assertTrue(mod.has_parent_traversal("foo/../bar.hwp"))
        self.assertFalse(mod.has_parent_traversal("foo/bar.hwp"))
        self.assertTrue(mod.is_nonneg_int(0))
        self.assertFalse(mod.is_nonneg_int(True))
        self.assertFalse(mod.is_nonneg_int(-1))

    def test_editing_axis_prefixes(self):
        mod = load()
        self.assertTrue(mod.pack_axis_is_editing("편집 (표 좌표 지정)"))
        self.assertTrue(mod.pack_axis_is_editing("보안 (은닉)"))
        self.assertFalse(mod.pack_axis_is_editing("조회 (x)"))
        self.assertEqual(mod.effective_axis({"axis": "조사"}, "편집"), "조사")
        self.assertEqual(mod.effective_axis({}, "편집 (표)"), "편집 (표)")

    def test_known_ops_include_registry(self):
        mod = load()
        mod.reset_ops_cache()
        all_ops, cli_ops, file_ops, global_ops = mod.known_ops_bundle()
        self.assertIn("answer_eq", cli_ops)
        self.assertIn("file_exists", file_ops)
        self.assertIn("deep_contains", global_ops)
        self.assertTrue(all_ops >= (cli_ops | file_ops))

    def test_scan_error_render(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            text = mod.render_report(mod.audit(tmp))
        self.assertIn("스캔 실패", text)


class RealRepoHealthGateTests(unittest.TestCase):
    def test_current_tree_stays_clean(self):
        """새 규칙은 현재 gym/packs 를 실패로 뒤집지 않는다."""
        mod = load()
        report = mod.audit(str(REPO_ROOT / "gym"))
        self.assertEqual(report["kind"], "gymPackHealth")
        self.assertGreaterEqual(report["packCount"], 10)
        self.assertEqual(
            report["issueCount"],
            0,
            report.get("codes"),
        )
        self.assertTrue(report["ok"], report.get("codes"))

    def test_issue_rows_have_stable_keys(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions="짧다")])
            rows = _issues(mod.audit(tmp), "p1")
        self.assertGreater(len(rows), 0)
        for row in rows:
            self.assertIn("code", row)
            self.assertIn("severity", row)
            self.assertIn("message", row)
            self.assertIn("where", row)
            self.assertIn("task", row)


if __name__ == "__main__":
    unittest.main()
