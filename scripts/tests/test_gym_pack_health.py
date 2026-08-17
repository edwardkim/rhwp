"""[pack_health] gym pack 건강 감사 계약 — 지시·검사 이름·힌트·제출 형식.

audit.py 는 스키마·기준풀이 짝만 본다. 이 가드는 빈/짧은 instructions, 중복·누락
check.name, 과제 id/title 공백, 빈 reference.steps, 미지 submit.kind, 힌트 정답
노출을 픽스처로 고정한다. 실제 저장소 pack 은 고치지 않는다.
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
        "checks": checks
        if checks is not None
        else [{"name": "쪽수 일치", "op": "answer_eq", "answer": "pages"}],
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


def _write_pack(root, pid, tasks, refs=None, manifest=True):
    pack_dir = os.path.join(root, "packs", pid)
    os.makedirs(os.path.join(pack_dir, "tasks"), exist_ok=True)
    os.makedirs(os.path.join(pack_dir, "reference"), exist_ok=True)
    if manifest:
        _write_json(
            os.path.join(pack_dir, "pack.json"),
            {
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
            },
        )
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
                    _task("A01", checks=[{"name": "쪽수 일치", "op": "answer_eq"}]),
                    _task("A02", checks=[{"name": "쪽수 일치", "op": "answer_eq"}]),
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
        checks = [{"name": "값", "op": "value_eq", "value": "홍길동"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text, checks=checks)])
            report = mod.audit(tmp)
        self.assertNotIn(mod.CODE_HINT_ANSWER_DUMP, codes_of(report, "p1"))
        self.assertNotIn(mod.CODE_HINT_EMBEDS_VALUE, codes_of(report, "p1"))
        self.assertTrue(report["ok"], report["packs"])

    def test_format_token_inside_command_is_ok(self):
        mod = load()
        text = "입력을 HWPX 로 변환하라. 힌트: rhwp export-hwpx <입력> conv.hwpx --verify."
        checks = [{"name": "형식", "op": "value_eq", "value": "hwpx"}]
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

    def test_hint_embeds_check_value(self):
        mod = load()
        text = "첫 칸을 고치라. 힌트: 셀에 계획실행을 넣으면 된다"
        checks = [{"name": "칸", "op": "cell_text_eq", "value": "계획실행"}]
        with tempfile.TemporaryDirectory() as tmp:
            _write_pack(tmp, "p1", [_task(instructions=text, checks=checks)])
            report = mod.audit(tmp)
        self.assertIn(mod.CODE_HINT_EMBEDS_VALUE, codes_of(report, "p1"))

    def test_expected_value_in_body_not_hint_is_ok(self):
        """과제가 '이 값으로 바꿔라'고 본문에 쓰는 것은 힌트 유출이 아니다."""
        mod = load()
        text = "첫 칸을 '계획실행' 으로 바꿔라. 힌트: rhwp run --plan-json."
        checks = [{"name": "칸", "op": "cell_text_eq", "value": "계획실행"}]
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


if __name__ == "__main__":
    unittest.main()
