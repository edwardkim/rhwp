"""[discriminate] gym 판별력 감사 계약 — 음성 대조 구성 + false-pass 색출.

핵심: 각 과제는 '일 안 한 제출'(음성 대조)을 거부해야 판별력이 있다. 음성 대조
종류는 세 가지로 고정한다.

- wrong-answer: 모든 답 키에 WRONG_SENTINEL
- input-copy: artifact 산출 자리에 입력 바이트를 무편집 복사
- garbage: 같은 자리에 GARBAGE_BYTES (1KiB 초과 synthetic)

음성이 통과하면 약한 오라클(false-pass, SWE-Bench 59.4% 결함과 같은 계열)로
잡는다. 채점은 목킹해 바이너리 없이 시험한다. 새 CLI/pack 은 없다.
"""

from __future__ import annotations

import argparse
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "discriminate.py"
SAMPLE = "samples/2010-01-06.hwp"  # REPO_ROOT 기준 실재 입력


def load():
    spec = importlib.util.spec_from_file_location("gym_discriminate", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _artifact_task():
    return {"id": "T", "input": SAMPLE, "submit": {"kind": "artifact", "files": ["out.svg"]},
            "checks": [{"op": "answer_eq", "answer": "pages",
                        "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}


def _answer_task(tid="A", keys=None):
    keys = list(keys or ["pages"])
    checks = [{"op": "answer_eq", "answer": k, "cmd": ["info", "{input}", "--json"],
               "path": "pageCount"} for k in keys]
    return {"id": tid, "input": SAMPLE, "submit": {"kind": "answer", "files": ["answer.json"]},
            "checks": checks}


def _pair_task():
    return {"id": "P", "input": SAMPLE, "submit": {"kind": "pair", "files": ["a.hwp", "b.hwp"]},
            "checks": [{"op": "files_differ", "files": ["a.hwp", "b.hwp"]}]}


def _temp_gym(root, task, pack="p1", name=None):
    tid = task.get("id", "T")
    name = name or f"{tid}.json"
    td = os.path.join(root, "gym", "packs", pack, "tasks")
    os.makedirs(td, exist_ok=True)
    with open(os.path.join(td, name), "w", encoding="utf-8") as fh:
        json.dump(task, fh, ensure_ascii=False)
    return os.path.join(root, "gym")


def _write(path, text="", binary=None):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    if binary is not None:
        with open(path, "wb") as fh:
            fh.write(binary)
    else:
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(text)


def _read(path, binary=False):
    mode = "rb" if binary else "r"
    kw = {} if binary else {"encoding": "utf-8"}
    with open(path, mode, **kw) as fh:
        return fh.read()


class DiscriminateTests(unittest.TestCase):
    """기존 계약 — 음성 구성과 false-pass 집계는 이 다섯 시험이 지킨다."""

    def test_build_negative_wrong_answer_and_noop_copy(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(_artifact_task(), d)
            sub = os.path.join(d, "T")
            with open(os.path.join(sub, "answer.json"), encoding="utf-8") as fh:
                ans = json.load(fh)
            self.assertEqual(ans["pages"], mod.WRONG_SENTINEL)         # 오답 대조
            with open(os.path.join(mod.REPO_ROOT, SAMPLE), "rb") as fh:
                real = fh.read()
            with open(os.path.join(sub, "out.svg"), "rb") as fh:
                self.assertEqual(fh.read(), real)  # 무편집 복사

    def test_flags_false_pass_when_negative_passes(self):
        mod = load()
        mod.runner.score_task = lambda task, pack_dir, bin_path: {"pass": True}  # 음성이 통과=약한 오라클
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _artifact_task())
            r = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertFalse(r["ok"])
            self.assertIn("p1/T", r["falsePass"])

    def test_clean_when_negative_rejected(self):
        mod = load()
        mod.runner.score_task = lambda task, pack_dir, bin_path: {"pass": False}  # 음성 거부=판별력 있음
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _artifact_task())
            r = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertTrue(r["ok"])
            self.assertEqual(r["falsePass"], [])
            self.assertEqual(r["discriminating"], 1)

    def test_artifact_garbage_false_pass_is_reported_separately(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _artifact_task())

            def score(_task, pack_dir, _bin_path):
                return {"pass": "garbage" in pack_dir}

            mod.runner.score_task = score
            r = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertFalse(r["ok"])
            self.assertEqual(r["taskCount"], 1)
            self.assertEqual(r["controlCount"], 2)
            self.assertEqual(r["falsePass"], ["p1/T"])
            self.assertEqual(r["falsePassControls"], ["p1/T (garbage)"])

    def test_build_garbage_negative_writes_non_input_bytes(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(_artifact_task(), d, artifact_mode="garbage")
            with open(os.path.join(d, "T", "out.svg"), "rb") as fh:
                data = fh.read()
            self.assertEqual(data, mod.GARBAGE_BYTES)


class CatalogContractTests(unittest.TestCase):
    """음성 대조 종류 세 가지를 시험이 고정한다."""

    def test_control_kinds_are_exactly_three(self):
        mod = load()
        self.assertEqual(
            mod.CONTROL_KINDS,
            (mod.CONTROL_WRONG_ANSWER, mod.CONTROL_INPUT_COPY, mod.CONTROL_GARBAGE),
        )
        self.assertEqual(mod.CONTROL_KINDS, ("wrong-answer", "input-copy", "garbage"))
        self.assertEqual(len(mod.CONTROL_KINDS), 3)
        self.assertEqual(len(set(mod.CONTROL_KINDS)), 3)

    def test_catalog_rows_match_ids_and_must_fail(self):
        mod = load()
        self.assertEqual(len(mod.CONTROL_CATALOG), 3)
        ids = [row["id"] for row in mod.CONTROL_CATALOG]
        self.assertEqual(ids, list(mod.CONTROL_KINDS))
        for row in mod.CONTROL_CATALOG:
            self.assertTrue(row["mustFail"])
            self.assertIn(row["submit"], ("answer", "artifact"))
            self.assertTrue(row["writes"])
            self.assertTrue(row["payload"])
            self.assertTrue(row["why"])

    def test_answer_controls_only_sentinel(self):
        mod = load()
        self.assertEqual(mod.ANSWER_CONTROLS, ("wrong-answer",))
        self.assertTrue(mod.is_answer_control("wrong-answer"))
        self.assertFalse(mod.is_answer_control("garbage"))
        self.assertFalse(mod.is_answer_control("input-copy"))

    def test_artifact_controls_are_copy_and_garbage(self):
        mod = load()
        self.assertEqual(mod.ARTIFACT_CONTROLS, ("input-copy", "garbage"))
        self.assertTrue(mod.is_artifact_control("input-copy"))
        self.assertTrue(mod.is_artifact_control("garbage"))
        self.assertFalse(mod.is_artifact_control("wrong-answer"))

    def test_control_spec_and_unknown(self):
        mod = load()
        spec = mod.control_spec("garbage")
        self.assertEqual(spec["payload"], "GARBAGE_BYTES")
        self.assertIsNone(mod.control_spec("truncate"))
        self.assertTrue(mod.is_known_control("wrong-answer"))
        self.assertFalse(mod.is_known_control("missing-file"))
        self.assertEqual(mod.control_ids(), mod.CONTROL_KINDS)

    def test_report_kind_and_required_keys(self):
        mod = load()
        self.assertEqual(mod.REPORT_KIND, "gymDiscrimination")
        self.assertEqual(mod.SCHEMA_VERSION, "1.0")
        for key in (
            "kind", "schemaVersion", "ok", "taskCount", "controlCount",
            "discriminating", "falsePass", "falsePassControls",
        ):
            self.assertIn(key, mod.REPORT_KEYS)

    def test_exit_codes_are_zero_and_one(self):
        mod = load()
        self.assertEqual(mod.EXIT_OK, 0)
        self.assertEqual(mod.EXIT_FALSE_PASS, 1)
        self.assertEqual(mod.exit_code({"ok": True}), 0)
        self.assertEqual(mod.exit_code({"ok": False}), 1)
        self.assertEqual(mod.exit_code({}), 1)

    def test_negative_dirname_constant(self):
        mod = load()
        self.assertEqual(mod.NEGATIVE_DIRNAME, "_negative_control")
        self.assertTrue(mod.default_neg_root(mod.GYM_ROOT).endswith("_negative_control"))


class SentinelContractTests(unittest.TestCase):
    def test_sentinel_text_is_stable(self):
        mod = load()
        self.assertEqual(mod.WRONG_SENTINEL, "__NEGATIVE_CONTROL_definitely_wrong__")
        self.assertTrue(mod.WRONG_SENTINEL.startswith("__NEGATIVE_CONTROL_"))
        self.assertIn("definitely_wrong", mod.WRONG_SENTINEL)

    def test_sentinel_is_not_a_typical_gold_value(self):
        mod = load()
        for value in (0, 1, -1, 0.0, True, False, None, "", "0", "pages", "ok",
                      [], {}, "true", "false", "null"):
            self.assertFalse(mod.is_wrong_sentinel(value), msg=repr(value))
        self.assertTrue(mod.is_wrong_sentinel(mod.WRONG_SENTINEL))

    def test_sentinel_answers_sorts_keys(self):
        mod = load()
        payload = mod.sentinel_answers({"z", "a", "m"})
        self.assertEqual(list(payload.keys()), ["a", "m", "z"])
        self.assertTrue(all(v == mod.WRONG_SENTINEL for v in payload.values()))

    def test_sentinel_answers_empty(self):
        mod = load()
        self.assertEqual(mod.sentinel_answers([]), {})
        self.assertEqual(mod.sentinel_answers(set()), {})

    def test_answer_keys_collects_only_nonempty_strings(self):
        mod = load()
        task = {
            "checks": [
                {"answer": "pages"},
                {"answer": ""},
                {"answer": 3},
                {"op": "file_exists"},
                "skip",
                {"answer": "loss"},
                {"answer": "pages"},
            ]
        }
        self.assertEqual(mod.answer_keys(task), {"pages", "loss"})

    def test_answer_keys_tolerates_broken_task(self):
        mod = load()
        self.assertEqual(mod.answer_keys({}), set())
        self.assertEqual(mod.answer_keys({"checks": None}), set())
        self.assertEqual(mod.answer_keys({"checks": "x"}), set())
        self.assertEqual(mod.answer_keys(None), set())  # type: ignore[arg-type]


class GarbageContractTests(unittest.TestCase):
    def test_garbage_is_marker_times_repeat(self):
        mod = load()
        self.assertEqual(mod.GARBAGE_BYTES, mod.GARBAGE_MARKER * mod.GARBAGE_REPEAT)
        self.assertEqual(mod.GARBAGE_REPEAT, 64)
        self.assertTrue(mod.GARBAGE_MARKER.startswith(b"RHWP_GYM_GARBAGE_NEGATIVE_CONTROL"))
        self.assertTrue(mod.GARBAGE_MARKER.endswith(b"\x00"))

    def test_garbage_exceeds_one_kib(self):
        mod = load()
        self.assertGreaterEqual(mod.garbage_size(), mod.GARBAGE_MIN_SIZE)
        self.assertGreaterEqual(mod.GARBAGE_MIN_SIZE, 1024)
        self.assertTrue(mod.garbage_meets_minimum())
        self.assertEqual(mod.garbage_size(), len(mod.GARBAGE_BYTES))

    def test_garbage_payload_predicate(self):
        mod = load()
        self.assertTrue(mod.is_garbage_payload(mod.GARBAGE_BYTES))
        self.assertFalse(mod.is_garbage_payload(mod.GARBAGE_BYTES + b"x"))
        self.assertFalse(mod.is_garbage_payload(mod.GARBAGE_BYTES[:-1]))
        self.assertFalse(mod.is_garbage_payload(b""))
        self.assertFalse(mod.is_garbage_payload(mod.GARBAGE_MARKER))

    def test_garbage_is_not_valid_utf8_document(self):
        mod = load()
        # 널 바이트가 있어 텍스트 문서로 쓸 수 없다. UTF-8 디코드 자체는 된다.
        self.assertIn(b"\x00", mod.GARBAGE_BYTES)
        text = mod.GARBAGE_BYTES.decode("utf-8")
        self.assertIn("\x00", text)
        self.assertTrue(text.startswith("RHWP_GYM_GARBAGE_NEGATIVE_CONTROL"))

    def test_garbage_is_not_json_or_xml(self):
        mod = load()
        with self.assertRaises(ValueError):
            json.loads(mod.GARBAGE_BYTES)
        self.assertFalse(mod.GARBAGE_BYTES.lstrip().startswith(b"{"))
        self.assertFalse(mod.GARBAGE_BYTES.lstrip().startswith(b"<"))
        self.assertFalse(mod.GARBAGE_BYTES.lstrip().startswith(b"%PDF"))


class PathSafetyTests(unittest.TestCase):
    def test_normalize_rel_accepts_nested_posix(self):
        mod = load()
        self.assertEqual(mod.normalize_rel("out.svg"), "out.svg")
        self.assertEqual(mod.normalize_rel("a/b/c.svg"), "a/b/c.svg")
        self.assertEqual(mod.normalize_rel("./a/./b"), "a/b")
        self.assertEqual(mod.normalize_rel("a\\b\\c"), "a/b/c")

    def test_normalize_rel_rejects_escape(self):
        mod = load()
        for rel in ("../x", "a/../b", "/abs", "C:/abs", "C:\\abs", "~/.ssh",
                    "//unc/share", "", ".", "..", None, 3, "  "):
            self.assertIsNone(mod.normalize_rel(rel), msg=repr(rel))

    def test_unsafe_rel_reason_catalog(self):
        mod = load()
        self.assertEqual(mod.unsafe_rel_reason(None), "not-str")
        self.assertEqual(mod.unsafe_rel_reason(""), "empty")
        self.assertEqual(mod.unsafe_rel_reason("/abs"), "absolute")
        self.assertEqual(mod.unsafe_rel_reason("C:/x"), "drive")
        self.assertEqual(mod.unsafe_rel_reason("../x"), "parent")
        self.assertEqual(mod.unsafe_rel_reason("//unc/x"), "unc")
        self.assertEqual(mod.unsafe_rel_reason("~/x"), "home")
        self.assertIsNone(mod.unsafe_rel_reason("out.svg"))
        for reason in ("empty", "not-str", "absolute", "drive", "parent", "unc", "home"):
            self.assertIn(reason, mod.UNSAFE_REL_REASONS)

    def test_is_safe_rel_and_join_sub(self):
        mod = load()
        self.assertTrue(mod.is_safe_rel("nested/out.svg"))
        self.assertFalse(mod.is_safe_rel("../out.svg"))
        with tempfile.TemporaryDirectory() as d:
            path = mod.join_sub(d, "a/b.txt")
            self.assertTrue(path.startswith(d))
            self.assertTrue(path.endswith(os.path.join("a", "b.txt")))
            with self.assertRaises(ValueError):
                mod.join_sub(d, "../escape")


class SubmitShapeTests(unittest.TestCase):
    def test_submit_kind_variants(self):
        mod = load()
        self.assertEqual(mod.submit_kind(_artifact_task()), "artifact")
        self.assertEqual(mod.submit_kind(_answer_task()), "answer")
        self.assertEqual(mod.submit_kind(_pair_task()), "pair")
        self.assertEqual(mod.submit_kind({}), "")
        self.assertEqual(mod.submit_kind({"submit": "x"}), "")
        self.assertTrue(mod.is_artifact_task(_artifact_task()))
        self.assertTrue(mod.is_answer_task(_answer_task()))
        self.assertTrue(mod.is_pair_task(_pair_task()))
        self.assertFalse(mod.is_artifact_task(_answer_task()))

    def test_controls_for_locks_three_kinds_to_submit(self):
        mod = load()
        self.assertEqual(mod.controls_for(_artifact_task()), ("input-copy", "garbage"))
        self.assertEqual(mod.controls_for(_answer_task()), ("wrong-answer",))
        self.assertEqual(mod.controls_for(_pair_task()), ("wrong-answer",))
        self.assertEqual(mod.controls_for({}), ("wrong-answer",))
        self.assertEqual(mod.controls_for({"submit": {"kind": "artifact"}}),
                         ("input-copy", "garbage"))

    def test_submit_files_dedup_and_skip_unsafe(self):
        mod = load()
        task = {"submit": {"kind": "artifact", "files": [
            "out.svg", "out.svg", "../x", "nested/a.bin", "", None, 1,
        ]}}
        self.assertEqual(mod.submit_files(task), ["out.svg", "nested/a.bin"])
        self.assertEqual(mod.submit_files({}), [])
        self.assertEqual(mod.submit_files({"submit": {"files": "out.svg"}}), [])


class BuildNegativeAnswerTests(unittest.TestCase):
    def test_answer_task_writes_only_sentinel_json(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(_answer_task(keys=["pages", "loss"]), d)
            path = os.path.join(d, "A", "answer.json")
            self.assertTrue(os.path.isfile(path))
            payload = json.loads(_read(path))
            self.assertEqual(payload, {"loss": mod.WRONG_SENTINEL, "pages": mod.WRONG_SENTINEL})
            self.assertEqual(built["answerKeys"], ["loss", "pages"])
            self.assertEqual(built["files"], [])
            self.assertFalse(os.path.exists(os.path.join(d, "A", "out.svg")))

    def test_answer_task_without_keys_makes_empty_dir(self):
        mod = load()
        task = {"id": "Z", "submit": {"kind": "answer"}, "checks": [{"op": "file_exists", "file": "x"}]}
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(task, d)
            self.assertIsNone(built["answerPath"])
            self.assertEqual(os.listdir(os.path.join(d, "Z")), [])

    def test_answer_json_is_utf8_without_ascii_escape(self):
        mod = load()
        task = _answer_task(keys=["한글키"])
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(task, d)
            raw = _read(os.path.join(d, "A", "answer.json"))
            self.assertIn("한글키", raw)
            self.assertNotIn("\\u", raw)

    def test_rebuild_replaces_previous_submission(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            _write(os.path.join(d, "T", "stale.txt"), "old")
            mod.build_negative(_answer_task(tid="T"), d)
            self.assertFalse(os.path.exists(os.path.join(d, "T", "stale.txt")))
            self.assertTrue(os.path.isfile(os.path.join(d, "T", "answer.json")))


class BuildNegativeCopyTests(unittest.TestCase):
    def test_input_copy_matches_repo_sample_bytes(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(_artifact_task(), d, artifact_mode="input-copy")
            src = os.path.join(mod.REPO_ROOT, SAMPLE)
            dst = os.path.join(d, "T", "out.svg")
            self.assertEqual(_read(dst, binary=True), _read(src, binary=True))
            self.assertEqual(built["files"][0]["action"], "copied")
            self.assertNotEqual(_read(dst, binary=True), mod.GARBAGE_BYTES)

    def test_input_copy_does_not_mutate_source(self):
        mod = load()
        src = os.path.join(mod.REPO_ROOT, SAMPLE)
        before = _read(src, binary=True)
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(_artifact_task(), d, artifact_mode="input-copy")
        self.assertEqual(_read(src, binary=True), before)

    def test_nested_artifact_path_is_created(self):
        mod = load()
        task = {"id": "N", "input": SAMPLE, "submit": {"kind": "artifact", "files": ["deep/out.svg"]},
                "checks": []}
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(task, d, artifact_mode="input-copy")
            self.assertTrue(os.path.isfile(os.path.join(d, "N", "deep", "out.svg")))

    def test_multiple_artifact_files_all_copied(self):
        mod = load()
        task = {"id": "M", "input": SAMPLE,
                "submit": {"kind": "artifact", "files": ["a.bin", "b.bin"]}, "checks": []}
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(task, d, artifact_mode="input-copy")
            src = _read(os.path.join(mod.REPO_ROOT, SAMPLE), binary=True)
            self.assertEqual(_read(os.path.join(d, "M", "a.bin"), binary=True), src)
            self.assertEqual(_read(os.path.join(d, "M", "b.bin"), binary=True), src)
            self.assertEqual([row["action"] for row in built["files"]], ["copied", "copied"])

    def test_missing_input_skips_copy_without_raising(self):
        mod = load()
        task = {"id": "X", "input": "samples/no-such-file-for-discriminate.hwp",
                "submit": {"kind": "artifact", "files": ["out.bin"]}, "checks": []}
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(task, d, artifact_mode="input-copy")
            self.assertEqual(built["files"][0]["action"], "skipped")
            self.assertFalse(os.path.exists(os.path.join(d, "X", "out.bin")))
            self.assertTrue(any("원본 없음" in e for e in built["errors"]))

    def test_path_traversal_file_is_rejected(self):
        mod = load()
        task = {"id": "E", "input": SAMPLE,
                "submit": {"kind": "artifact", "files": ["../escape.bin", "ok.bin"]},
                "checks": []}
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(task, d, artifact_mode="input-copy")
            self.assertFalse(os.path.exists(os.path.join(d, "escape.bin")))
            actions = {row["rel"]: row["action"] for row in built["files"]}
            self.assertEqual(actions["../escape.bin"], "rejected")
            self.assertEqual(actions["ok.bin"], "copied")
            self.assertTrue(os.path.isfile(os.path.join(d, "E", "ok.bin")))

    def test_unknown_artifact_mode_raises(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(ValueError) as ctx:
                mod.build_negative(_artifact_task(), d, artifact_mode="truncate")
            self.assertIn("truncate", str(ctx.exception))

    def test_wrong_answer_mode_on_artifact_skips_files(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(_artifact_task(), d, artifact_mode="wrong-answer")
            self.assertTrue(os.path.isfile(os.path.join(d, "T", "answer.json")))
            self.assertFalse(os.path.exists(os.path.join(d, "T", "out.svg")))
            self.assertEqual(built["files"], [])


class BuildNegativeGarbageTests(unittest.TestCase):
    def test_garbage_bytes_exact_and_not_input(self):
        mod = load()
        src = _read(os.path.join(mod.REPO_ROOT, SAMPLE), binary=True)
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(_artifact_task(), d, artifact_mode="garbage")
            data = _read(os.path.join(d, "T", "out.svg"), binary=True)
            self.assertEqual(data, mod.GARBAGE_BYTES)
            self.assertNotEqual(data, src)
            self.assertGreaterEqual(len(data), 1024)
            self.assertEqual(built["files"][0]["action"], "garbage")

    def test_garbage_does_not_need_source_file(self):
        mod = load()
        task = {"id": "G", "input": "samples/missing-for-garbage.hwp",
                "submit": {"kind": "artifact", "files": ["out.bin"]}, "checks": []}
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(task, d, artifact_mode="garbage")
            self.assertEqual(built["files"][0]["action"], "garbage")
            self.assertEqual(_read(os.path.join(d, "G", "out.bin"), binary=True), mod.GARBAGE_BYTES)

    def test_garbage_on_nested_and_multiple_files(self):
        mod = load()
        task = {"id": "G2", "input": SAMPLE,
                "submit": {"kind": "artifact", "files": ["x/a.bin", "y/b.bin"]}, "checks": []}
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(task, d, artifact_mode="garbage")
            self.assertEqual(_read(os.path.join(d, "G2", "x", "a.bin"), binary=True), mod.GARBAGE_BYTES)
            self.assertEqual(_read(os.path.join(d, "G2", "y", "b.bin"), binary=True), mod.GARBAGE_BYTES)

    def test_garbage_rejects_traversal_like_copy(self):
        mod = load()
        task = {"id": "G3", "input": SAMPLE,
                "submit": {"kind": "artifact", "files": ["../nope.bin"]}, "checks": []}
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(task, d, artifact_mode="garbage")
            self.assertEqual(built["files"][0]["action"], "rejected")
            self.assertFalse(os.path.exists(os.path.join(d, "nope.bin")))


class ScoreClassifyTests(unittest.TestCase):
    def test_score_is_pass_only_true_pass(self):
        mod = load()
        self.assertTrue(mod.score_is_pass({"pass": True}))
        self.assertFalse(mod.score_is_pass({"pass": False}))
        self.assertFalse(mod.score_is_pass({}))
        self.assertFalse(mod.score_is_pass(None))
        self.assertFalse(mod.score_is_pass("pass"))
        self.assertTrue(mod.score_is_pass({"pass": 1}))

    def test_score_discriminates_is_negation(self):
        mod = load()
        self.assertFalse(mod.score_discriminates({"pass": True}))
        self.assertTrue(mod.score_discriminates({"pass": False}))
        self.assertTrue(mod.score_discriminates({}))

    def test_normalize_score_non_dict(self):
        mod = load()
        got = mod.normalize_score("x")
        self.assertFalse(got["pass"])
        self.assertIn("dict", got["error"])
        wrapped = mod.normalize_score({"pass": True, "id": "T", "error": "e"})
        self.assertEqual(wrapped, {"pass": True, "error": "e", "id": "T"})

    def test_score_task_safe_folds_value_error(self):
        mod = load()

        def boom(*_a, **_k):
            raise ValueError("broken oracle")

        got = mod.score_task_safe({}, "p", "bin", score_fn=boom)
        self.assertFalse(got["pass"])
        self.assertIn("ValueError", got["error"])

    def test_score_task_safe_reraises_keyboardinterrupt(self):
        mod = load()

        def boom(*_a, **_k):
            raise KeyboardInterrupt

        with self.assertRaises(KeyboardInterrupt):
            mod.score_task_safe({}, "p", "bin", score_fn=boom)

    def test_score_task_safe_reraises_systemexit(self):
        mod = load()

        def boom(*_a, **_k):
            raise SystemExit(2)

        with self.assertRaises(SystemExit):
            mod.score_task_safe({}, "p", "bin", score_fn=boom)

    def test_is_fatal_exception(self):
        mod = load()
        self.assertTrue(mod.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(mod.is_fatal_exception(SystemExit()))
        self.assertTrue(mod.is_fatal_exception(MemoryError()))
        self.assertTrue(mod.is_fatal_exception(GeneratorExit()))
        self.assertFalse(mod.is_fatal_exception(ValueError("x")))
        self.assertFalse(mod.is_fatal_exception(OSError("x")))


class LabelAndAggregateTests(unittest.TestCase):
    def test_false_pass_labels(self):
        mod = load()
        self.assertEqual(mod.false_pass_label("core-cli", "T01"), "core-cli/T01")
        self.assertEqual(mod.false_pass_control_label("p", "T", "garbage"), "p/T (garbage)")
        self.assertEqual(mod.parse_false_pass_label("p1/T"), ("p1", "T"))
        self.assertIsNone(mod.parse_false_pass_label("nope"))
        self.assertIsNone(mod.parse_false_pass_label(""))

    def test_aggregate_empty_is_ok(self):
        mod = load()
        report = mod.aggregate_rows([], 0)
        self.assertTrue(report["ok"])
        self.assertEqual(report["taskCount"], 0)
        self.assertEqual(report["controlCount"], 0)
        self.assertEqual(report["discriminating"], 0)
        self.assertEqual(report["kind"], "gymDiscrimination")
        self.assertEqual(report["schemaVersion"], "1.0")

    def test_aggregate_dedups_task_but_keeps_control_rows(self):
        mod = load()
        rows = [
            {"pack": "p", "task": "T", "control": "input-copy", "discriminates": True},
            {"pack": "p", "task": "T", "control": "garbage", "discriminates": False},
        ]
        report = mod.aggregate_rows(rows, 1)
        self.assertFalse(report["ok"])
        self.assertEqual(report["falsePass"], ["p/T"])
        self.assertEqual(report["falsePassControls"], ["p/T (garbage)"])
        self.assertEqual(report["discriminating"], 0)
        self.assertEqual(report["controlCount"], 2)

    def test_unique_keep_order(self):
        mod = load()
        self.assertEqual(mod.unique_keep_order(["a", "b", "a", "c", "b"]), ["a", "b", "c"])


class ValidateReportTests(unittest.TestCase):
    def _good(self, mod):
        return {
            "kind": "gymDiscrimination",
            "schemaVersion": "1.0",
            "ok": True,
            "taskCount": 1,
            "controlCount": 1,
            "discriminating": 1,
            "falsePass": [],
            "falsePassControls": [],
            "results": [{"pack": "p", "task": "T", "control": "wrong-answer", "discriminates": True}],
            "controlKinds": list(mod.CONTROL_KINDS),
        }

    def test_good_report_has_no_issues(self):
        mod = load()
        self.assertEqual(mod.validate_report(self._good(mod)), [])

    def test_ok_true_with_false_pass_is_issue(self):
        mod = load()
        report = self._good(mod)
        report["ok"] = True
        report["falsePass"] = ["p/T"]
        report["discriminating"] = 0
        issues = mod.validate_report(report)
        self.assertTrue(any("ok 가 참인데" in i for i in issues))

    def test_ok_false_without_false_pass_is_issue(self):
        mod = load()
        report = self._good(mod)
        report["ok"] = False
        issues = mod.validate_report(report)
        self.assertTrue(any("ok 가 거짓인데" in i for i in issues))

    def test_unknown_control_in_results(self):
        mod = load()
        report = self._good(mod)
        report["results"][0]["control"] = "truncate"
        issues = mod.validate_report(report)
        self.assertTrue(any("미지 대조" in i for i in issues))

    def test_missing_required_key(self):
        mod = load()
        report = self._good(mod)
        del report["falsePass"]
        issues = mod.validate_report(report)
        self.assertTrue(any("필수 키 없음: falsePass" in i for i in issues))

    def test_non_dict_report(self):
        mod = load()
        self.assertEqual(mod.validate_report(None), ["보고가 dict 가 아니다"])

    def test_control_count_must_match_results(self):
        mod = load()
        report = self._good(mod)
        report["controlCount"] = 9
        issues = mod.validate_report(report)
        self.assertTrue(any("controlCount" in i for i in issues))

    def test_discriminating_must_match_arithmetic(self):
        mod = load()
        report = self._good(mod)
        report["discriminating"] = 99
        issues = mod.validate_report(report)
        self.assertTrue(any("discriminating" in i for i in issues))

    def test_bad_false_pass_control_label(self):
        mod = load()
        report = self._good(mod)
        report["ok"] = False
        report["falsePass"] = ["p/T"]
        report["falsePassControls"] = ["p/T garbage"]
        report["discriminating"] = 0
        issues = mod.validate_report(report)
        self.assertTrue(any("falsePassControls 라벨" in i for i in issues))

    def test_control_kinds_must_match_catalog(self):
        mod = load()
        report = self._good(mod)
        report["controlKinds"] = ["wrong-answer"]
        issues = mod.validate_report(report)
        self.assertTrue(any("controlKinds" in i for i in issues))


class DiscriminateDiscoveryTests(unittest.TestCase):
    def test_empty_gym_is_ok(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            os.makedirs(os.path.join(gym, "packs"))
            report = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertTrue(report["ok"])
            self.assertEqual(report["taskCount"], 0)
            self.assertEqual(report["controlCount"], 0)
            self.assertEqual(mod.validate_report(report), [])

    def test_missing_packs_dir_is_empty_ok(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            os.makedirs(gym)
            report = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertTrue(report["ok"])
            self.assertEqual(report["taskCount"], 0)

    def test_skips_non_json_and_malformed(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _answer_task())
            tasks = os.path.join(gym, "packs", "p1", "tasks")
            _write(os.path.join(tasks, "README.md"), "no")
            _write(os.path.join(tasks, "broken.json"), "{")
            _write(os.path.join(tasks, "array.json"), "[1]")
            _write(os.path.join(tasks, "noid.json"), '{"checks":[]}')
            report = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertEqual(report["taskCount"], 1)
            self.assertGreaterEqual(len(report["loadErrors"]), 3)
            self.assertTrue(report["ok"])

    def test_pack_without_tasks_dir_is_ignored(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            os.makedirs(os.path.join(gym, "packs", "empty"))
            _write(os.path.join(gym, "packs", "empty", "pack.json"), "{}")
            report = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertEqual(report["taskCount"], 0)

    def test_iter_pack_and_task_names_are_sorted(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            for name in ("zeta", "alpha"):
                os.makedirs(os.path.join(d, name, "tasks"))
            self.assertEqual(mod.iter_pack_ids(d), ["alpha", "zeta"])
            tasks = os.path.join(d, "alpha", "tasks")
            _write(os.path.join(tasks, "B.json"), "{}")
            _write(os.path.join(tasks, "A.json"), "{}")
            _write(os.path.join(tasks, "A.txt"), "x")
            self.assertEqual(mod.iter_task_names(tasks), ["A.json", "B.json"])

    def test_iter_pack_ids_missing_dir(self):
        mod = load()
        self.assertEqual(mod.iter_pack_ids(os.path.join("no", "such", "packs")), [])
        self.assertEqual(mod.iter_task_names(os.path.join("no", "such", "tasks")), [])

    def test_load_task_errors(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            bad = os.path.join(d, "x.json")
            _write(bad, "{")
            task, err = mod.load_task(bad)
            self.assertIsNone(task)
            self.assertIn("파싱 실패", err)
            arr = os.path.join(d, "a.json")
            _write(arr, "[1]")
            task, err = mod.load_task(arr)
            self.assertIsNone(task)
            self.assertIn("객체가 아니다", err)

    def test_task_id_of(self):
        mod = load()
        self.assertEqual(mod.task_id_of({"id": "T01"}), "T01")
        self.assertIsNone(mod.task_id_of({"id": "  "}))
        self.assertIsNone(mod.task_id_of({"id": 3}))
        self.assertIsNone(mod.task_id_of({}))


class DiscriminateMatrixTests(unittest.TestCase):
    def test_answer_task_runs_only_wrong_answer_control(self):
        mod = load()
        seen = []

        def score(_task, pack_dir, _bin):
            seen.append(pack_dir.replace("\\", "/"))
            return {"pass": False}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _answer_task())
            report = mod.discriminate("bin", gym, os.path.join(d, "neg"))
        self.assertEqual(report["controlCount"], 1)
        self.assertEqual(report["results"][0]["control"], "wrong-answer")
        self.assertEqual(len(seen), 1)
        self.assertIn("/wrong-answer/", seen[0])
        self.assertTrue(report["ok"])

    def test_artifact_task_runs_copy_then_garbage(self):
        mod = load()
        order = []

        def score(_task, pack_dir, _bin):
            norm = pack_dir.replace("\\", "/")
            if "/input-copy/" in norm:
                order.append("input-copy")
            if "/garbage/" in norm:
                order.append("garbage")
            return {"pass": False}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _artifact_task())
            report = mod.discriminate("bin", gym, os.path.join(d, "neg"))
        self.assertEqual(order, ["input-copy", "garbage"])
        self.assertEqual([row["control"] for row in report["results"]],
                         ["input-copy", "garbage"])
        self.assertEqual(report["controlCount"], 2)

    def test_copy_only_false_pass(self):
        mod = load()

        def score(_task, pack_dir, _bin):
            return {"pass": "input-copy" in pack_dir}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            report = mod.discriminate("bin", _temp_gym(d, _artifact_task()), os.path.join(d, "neg"))
        self.assertEqual(report["falsePass"], ["p1/T"])
        self.assertEqual(report["falsePassControls"], ["p1/T (input-copy)"])
        self.assertFalse(report["ok"])

    def test_both_artifact_controls_false_pass_dedup_task(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            report = mod.discriminate("bin", _temp_gym(d, _artifact_task()), os.path.join(d, "neg"))
        self.assertEqual(report["falsePass"], ["p1/T"])
        self.assertEqual(
            report["falsePassControls"],
            ["p1/T (input-copy)", "p1/T (garbage)"],
        )
        self.assertEqual(report["discriminating"], 0)

    def test_answer_sentinel_false_pass(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            report = mod.discriminate("bin", _temp_gym(d, _answer_task()), os.path.join(d, "neg"))
        self.assertEqual(report["falsePass"], ["p1/A"])
        self.assertEqual(report["falsePassControls"], ["p1/A (wrong-answer)"])

    def test_mixed_packs_keep_sorted_order(self):
        mod = load()
        seen = []

        def score(task, pack_dir, _bin):
            seen.append((task["id"], pack_dir.replace("\\", "/")))
            return {"pass": False}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            _temp_gym(d, _answer_task(tid="B"), pack="zeta")
            _temp_gym(d, _artifact_task(), pack="alpha")
            gym = os.path.join(d, "gym")
            report = mod.discriminate("bin", gym, os.path.join(d, "neg"))
        self.assertEqual(report["taskCount"], 2)
        self.assertEqual(report["controlCount"], 3)
        self.assertEqual([row["pack"] for row in report["results"]], ["alpha", "alpha", "zeta"])
        self.assertEqual(seen[0][0], "T")
        self.assertEqual(seen[-1][0], "B")

    def test_score_exception_is_not_false_pass(self):
        mod = load()

        def boom(*_a, **_k):
            raise RuntimeError("oracle down")

        mod.runner.score_task = boom
        with tempfile.TemporaryDirectory() as d:
            report = mod.discriminate("bin", _temp_gym(d, _answer_task()), os.path.join(d, "neg"))
        self.assertTrue(report["ok"])
        self.assertEqual(report["falsePass"], [])
        self.assertTrue(report["scoreErrors"])
        self.assertFalse(report["results"][0]["discriminates"] is True and report["results"][0].get("error") is None)
        self.assertTrue(report["results"][0]["discriminates"])

    def test_pair_task_uses_answer_control_only(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            report = mod.discriminate("bin", _temp_gym(d, _pair_task()), os.path.join(d, "neg"))
        self.assertEqual(report["controlCount"], 1)
        self.assertEqual(report["results"][0]["control"], "wrong-answer")

    def test_injected_score_fn_overrides_runner(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            report = mod.discriminate(
                "bin",
                _temp_gym(d, _answer_task()),
                os.path.join(d, "neg"),
                score_fn=lambda *a, **k: {"pass": False},
            )
        self.assertTrue(report["ok"])


class BuildThenScoreIntegrationTests(unittest.TestCase):
    def test_answer_control_writes_sentinel_before_score(self):
        mod = load()
        seen = {}

        def score(_task, pack_dir, _bin):
            path = os.path.join(pack_dir, "A", "answer.json")
            seen["payload"] = json.loads(_read(path))
            return {"pass": False}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            mod.discriminate("bin", _temp_gym(d, _answer_task()), os.path.join(d, "neg"))
        self.assertEqual(seen["payload"]["pages"], mod.WRONG_SENTINEL)

    def test_copy_control_material_is_input_bytes(self):
        mod = load()
        src = _read(os.path.join(load().REPO_ROOT, SAMPLE), binary=True)
        seen = {}

        def score(_task, pack_dir, _bin):
            if "input-copy" in pack_dir:
                seen["copy"] = _read(os.path.join(pack_dir, "T", "out.svg"), binary=True)
            return {"pass": False}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            mod.discriminate("bin", _temp_gym(d, _artifact_task()), os.path.join(d, "neg"))
        self.assertEqual(seen["copy"], src)

    def test_garbage_control_material_is_garbage_bytes(self):
        mod = load()
        seen = {}

        def score(_task, pack_dir, _bin):
            if "garbage" in pack_dir:
                seen["g"] = _read(os.path.join(pack_dir, "T", "out.svg"), binary=True)
            return {"pass": False}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            mod.discriminate("bin", _temp_gym(d, _artifact_task()), os.path.join(d, "neg"))
        self.assertEqual(seen["g"], load().GARBAGE_BYTES)

    def test_copy_and_garbage_are_different_bytes(self):
        mod = load()
        blobs = {}

        def score(_task, pack_dir, _bin):
            path = os.path.join(pack_dir, "T", "out.svg")
            if not os.path.isfile(path):
                return {"pass": False}
            norm = pack_dir.replace("\\", "/")
            if "/input-copy/" in norm:
                blobs["input-copy"] = _read(path, binary=True)
            elif "/garbage/" in norm:
                blobs["garbage"] = _read(path, binary=True)
            return {"pass": False}

        mod.runner.score_task = score
        with tempfile.TemporaryDirectory() as d:
            mod.discriminate("bin", _temp_gym(d, _artifact_task()), os.path.join(d, "neg"))
        self.assertIn("input-copy", blobs)
        self.assertIn("garbage", blobs)
        self.assertNotEqual(blobs["input-copy"], blobs["garbage"])


class HumanReportTests(unittest.TestCase):
    def test_ok_message(self):
        mod = load()
        text = mod.format_human_report({"ok": True, "taskCount": 12, "falsePass": []})
        self.assertIn("12 과제", text)
        self.assertIn("약한 오라클 0", text)
        self.assertTrue(text.endswith("\n"))

    def test_false_pass_message_lists_tasks(self):
        mod = load()
        text = mod.format_human_report({
            "ok": False,
            "falsePass": ["p/T", "q/U"],
            "falsePassControls": ["p/T (garbage)"],
        })
        self.assertIn("2건", text)
        self.assertIn("  - p/T", text)
        self.assertIn("  - q/U", text)
        self.assertIn("p/T (garbage)", text)
        self.assertIn("일 안 한 제출이 통과한다", text)

    def test_human_lines_match_join(self):
        mod = load()
        report = {"ok": True, "taskCount": 1}
        self.assertEqual(mod.format_human_report(report), "\n".join(mod.human_lines(report)) + "\n")


class CliMainTests(unittest.TestCase):
    def test_parse_args_requires_bin(self):
        mod = load()
        with self.assertRaises(SystemExit):
            mod.parse_args([])
        ns = mod.parse_args(["--bin", "target/debug/rhwp"])
        self.assertEqual(ns.bin, "target/debug/rhwp")
        self.assertFalse(ns.json)
        ns = mod.parse_args(["--bin", "rhwp", "--json"])
        self.assertTrue(ns.json)

    def test_parse_args_rejects_unknown_flag(self):
        mod = load()
        with self.assertRaises(SystemExit):
            mod.parse_args(["--bin", "rhwp", "--pack", "x"])

    def test_emit_json_and_human(self):
        mod = load()
        report = {
            "kind": "gymDiscrimination", "schemaVersion": "1.0", "ok": True,
            "taskCount": 0, "controlCount": 0, "discriminating": 0,
            "falsePass": [], "falsePassControls": [],
        }
        buf = io.StringIO()
        mod.emit_report(report, True, stream=buf)
        parsed = json.loads(buf.getvalue())
        self.assertEqual(parsed["kind"], "gymDiscrimination")
        buf = io.StringIO()
        mod.emit_report(report, False, stream=buf)
        self.assertIn("약한 오라클 0", buf.getvalue())

    def test_main_json_uses_run_audit(self):
        mod = load()
        fake = {
            "kind": "gymDiscrimination", "schemaVersion": "1.0", "ok": True,
            "taskCount": 2, "controlCount": 3, "discriminating": 2,
            "falsePass": [], "falsePassControls": [],
        }
        buf = io.StringIO()
        with mock.patch.object(mod.runner, "find_bin", return_value="rhwp"):
            with mock.patch.object(mod, "run_audit", return_value=fake):
                with mock.patch.object(sys, "stdout", buf):
                    code = mod.main(["--bin", "rhwp", "--json"])
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(buf.getvalue())["taskCount"], 2)

    def test_main_returns_one_on_false_pass(self):
        mod = load()
        fake = {"ok": False, "taskCount": 1, "falsePass": ["p/T"], "falsePassControls": []}
        buf = io.StringIO()
        with mock.patch.object(mod.runner, "find_bin", return_value="rhwp"):
            with mock.patch.object(mod, "run_audit", return_value=fake):
                with mock.patch.object(sys, "stdout", buf):
                    code = mod.main(["--bin", "rhwp"])
        self.assertEqual(code, 1)
        self.assertIn("p/T", buf.getvalue())

    def test_run_audit_clears_neg_root(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _answer_task())
            neg = os.path.join(d, "neg")
            stale = os.path.join(neg, "stale.txt")
            _write(stale, "old")
            mod.runner.score_task = lambda *a, **k: {"pass": False}
            report = mod.run_audit("bin", gym_root=gym, neg_root=neg)
            self.assertTrue(report["ok"])
            self.assertFalse(os.path.isfile(stale))

    def test_prepare_neg_root_recreates(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            target = os.path.join(d, "neg")
            _write(os.path.join(target, "x"), "1")
            got = mod.prepare_neg_root(target)
            self.assertEqual(got, target)
            self.assertTrue(os.path.isdir(target))
            self.assertEqual(os.listdir(target), [])


class ResolveInputAndWriteHelpersTests(unittest.TestCase):
    def test_resolve_input_path_relative_and_abs(self):
        mod = load()
        rel = mod.resolve_input_path({"input": SAMPLE})
        self.assertTrue(rel.endswith(SAMPLE.replace("/", os.sep)))
        self.assertTrue(os.path.isfile(rel))
        self.assertEqual(mod.resolve_input_path({}), "")
        self.assertEqual(mod.resolve_input_path({"input": 3}), "")
        with tempfile.TemporaryDirectory() as d:
            abs_in = os.path.join(d, "in.bin")
            _write(abs_in, binary=b"abc")
            self.assertEqual(mod.resolve_input_path({"input": abs_in}), abs_in)

    def test_write_json_and_bytes_create_parents(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            jp = os.path.join(d, "a", "b.json")
            bp = os.path.join(d, "a", "c.bin")
            mod.write_json(jp, {"k": "한글"})
            mod.write_bytes(bp, b"\x00\x01")
            self.assertEqual(json.loads(_read(jp)), {"k": "한글"})
            self.assertEqual(_read(bp, binary=True), b"\x00\x01")

    def test_copy_file_overwrites(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            src = os.path.join(d, "src.bin")
            dst = os.path.join(d, "sub", "dst.bin")
            _write(src, binary=b"one")
            _write(dst, binary=b"two")
            mod.copy_file(src, dst)
            self.assertEqual(_read(dst, binary=True), b"one")

    def test_write_answer_file_none_when_no_keys(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            self.assertIsNone(mod.write_answer_file(d, []))
            self.assertFalse(os.path.exists(os.path.join(d, "answer.json")))
            path = mod.write_answer_file(d, ["k"])
            self.assertTrue(os.path.isfile(path))


class RunOneControlTests(unittest.TestCase):
    def test_run_one_control_copy_row(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            row, build_err, score_err = mod.run_one_control(
                _artifact_task(), "p1", "input-copy", d, "bin",
                score_fn=lambda *a, **k: {"pass": False},
            )
        self.assertTrue(row["discriminates"])
        self.assertEqual(row["control"], "input-copy")
        self.assertEqual(build_err, [])
        self.assertEqual(score_err, [])

    def test_run_one_control_records_score_error(self):
        mod = load()

        def boom(*_a, **_k):
            raise RuntimeError("x")

        with tempfile.TemporaryDirectory() as d:
            row, _b, score_err = mod.run_one_control(
                _answer_task(), "p1", "wrong-answer", d, "bin", score_fn=boom,
            )
        self.assertTrue(score_err)
        self.assertIn("RuntimeError", score_err[0])
        self.assertTrue(row["discriminates"])

    def test_make_result_row_extra(self):
        mod = load()
        row = mod.make_result_row("p", "T", "garbage", False, {"error": "e"})
        self.assertEqual(row["pack"], "p")
        self.assertEqual(row["error"], "e")
        self.assertFalse(row["discriminates"])


class EmptyReportAndDiscoverTests(unittest.TestCase):
    def test_empty_report_validates(self):
        mod = load()
        report = mod.empty_report()
        self.assertEqual(mod.validate_report(report), [])
        self.assertEqual(report["controlKinds"], list(mod.CONTROL_KINDS))

    def test_discover_task_entries_reads_temp_gym(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _answer_task())
            entries, errs, tools = mod.discover_task_entries(gym)
        self.assertEqual(len(entries), 1)
        self.assertEqual(entries[0]["pack"], "p1")
        self.assertEqual(entries[0]["task"]["id"], "A")
        self.assertEqual(errs, [])
        self.assertEqual(tools, [])

    def test_control_pack_dir_layout(self):
        mod = load()
        path = mod.control_pack_dir("/neg", "garbage", "core-cli")
        self.assertEqual(os.path.normpath(path), os.path.normpath(os.path.join("/neg", "garbage", "core-cli")))

    def test_packs_dir_of(self):
        mod = load()
        self.assertTrue(mod.packs_dir_of("/tmp/gym").replace("\\", "/").endswith("gym/packs"))


class ThreeControlKindTableTests(unittest.TestCase):
    """이슈 #5255 DoD — 음성 대조 종류를 표로 고정."""

    CASES = (
        ("wrong-answer", "answer", "sentinel", True),
        ("input-copy", "artifact", "input-bytes", True),
        ("garbage", "artifact", "garbage-bytes", True),
    )

    def test_table_ids_are_the_only_kinds(self):
        mod = load()
        self.assertEqual(tuple(c[0] for c in self.CASES), mod.CONTROL_KINDS)

    def test_table_must_fail_every_kind(self):
        for _cid, _submit, _payload, must_fail in self.CASES:
            self.assertTrue(must_fail)

    def test_answer_kind_payload_is_sentinel_not_copy(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(_answer_task(), d, artifact_mode="wrong-answer")
            payload = json.loads(_read(os.path.join(d, "A", "answer.json")))
        self.assertEqual(payload["pages"], mod.WRONG_SENTINEL)
        self.assertNotEqual(payload["pages"], SAMPLE)

    def test_copy_kind_payload_is_input_not_garbage(self):
        mod = load()
        src = _read(os.path.join(mod.REPO_ROOT, SAMPLE), binary=True)
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(_artifact_task(), d, artifact_mode="input-copy")
            data = _read(os.path.join(d, "T", "out.svg"), binary=True)
        self.assertEqual(data, src)
        self.assertNotEqual(data, mod.GARBAGE_BYTES)

    def test_garbage_kind_payload_is_garbage_not_input(self):
        mod = load()
        src = _read(os.path.join(mod.REPO_ROOT, SAMPLE), binary=True)
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(_artifact_task(), d, artifact_mode="garbage")
            data = _read(os.path.join(d, "T", "out.svg"), binary=True)
        self.assertEqual(data, mod.GARBAGE_BYTES)
        self.assertNotEqual(data, src)

    def test_live_report_mentions_only_catalog_controls(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            _temp_gym(d, _answer_task(), pack="ans")
            _temp_gym(d, _artifact_task(), pack="art")
            report = mod.discriminate("bin", os.path.join(d, "gym"), os.path.join(d, "neg"))
        kinds = {row["control"] for row in report["results"]}
        self.assertTrue(kinds.issubset(set(mod.CONTROL_KINDS)))
        self.assertEqual(kinds, {"wrong-answer", "input-copy", "garbage"})
        self.assertEqual(mod.validate_report(report), [])


class ArgparseSurfaceTests(unittest.TestCase):
    def test_no_new_cli_flags(self):
        mod = load()
        parser = argparse.ArgumentParser()
        # 재사용하지 않고 parse_args 의 플래그만 본다.
        ns = mod.parse_args(["--bin", "x"])
        self.assertEqual(sorted(vars(ns).keys()), ["bin", "json"])

    def test_help_mentions_false_pass(self):
        mod = load()
        buf = io.StringIO()
        with mock.patch.object(sys, "stdout", buf):
            with self.assertRaises(SystemExit):
                mod.parse_args(["--help"])
        self.assertIn("false-pass", buf.getvalue().lower())


class RepoLayoutSmokeTests(unittest.TestCase):
    def test_tool_lives_in_gym_tools(self):
        self.assertTrue(TOOL.is_file())
        self.assertEqual(TOOL.name, "discriminate.py")

    def test_module_constants_point_at_repo(self):
        mod = load()
        self.assertTrue(os.path.isdir(mod.GYM_ROOT))
        self.assertTrue(os.path.isdir(mod.REPO_ROOT))
        self.assertTrue(os.path.isfile(os.path.join(mod.REPO_ROOT, SAMPLE)))
        self.assertTrue(mod.GYM_ROOT.endswith("gym") or mod.GYM_ROOT.replace("\\", "/").endswith("/gym"))

    def test_docs_were_added_next_to_tool_contract(self):
        root = TOOL.parents[2]
        self.assertTrue((root / "gym" / "docs" / "discriminate.md").is_file())
        self.assertTrue((root / "mydocs" / "working" / "gym_discriminate.md").is_file())


class CatchableExceptionCatalogTests(unittest.TestCase):
    def test_catchable_includes_os_and_json(self):
        mod = load()
        self.assertIn(OSError, mod.CATCHABLE_EXCEPTIONS)
        self.assertIn(ValueError, mod.CATCHABLE_EXCEPTIONS)
        self.assertIn(json.JSONDecodeError, mod.CATCHABLE_EXCEPTIONS)
        for fatal in mod.FATAL_EXCEPTIONS:
            self.assertNotIn(fatal, mod.CATCHABLE_EXCEPTIONS)

    def test_score_task_safe_folds_oserror(self):
        mod = load()
        got = mod.score_task_safe({}, "p", "bin", score_fn=lambda *_a, **_k: (_ for _ in ()).throw(OSError("disk")))
        self.assertFalse(got["pass"])
        self.assertIn("OSError", got["error"])


class OptionalReportKeysTests(unittest.TestCase):
    def test_optional_keys_are_named(self):
        mod = load()
        for key in (
            "results", "loadErrors", "scoreErrors", "buildErrors",
            "skipped", "toolFailed", "toolErrors", "controlKinds",
        ):
            self.assertIn(key, mod.OPTIONAL_REPORT_KEYS)

    def test_aggregate_extras_override(self):
        mod = load()
        report = mod.aggregate_rows([], 0, {"toolFailed": True, "toolErrors": ["x"]})
        self.assertTrue(report["toolFailed"])
        self.assertEqual(report["toolErrors"], ["x"])
        self.assertTrue(report["ok"])


class ExtraHelperContractTests(unittest.TestCase):
    def test_expected_control_count(self):
        mod = load()
        self.assertEqual(mod.expected_control_count(_artifact_task()), 2)
        self.assertEqual(mod.expected_control_count(_answer_task()), 1)
        self.assertEqual(mod.expected_control_count(_pair_task()), 1)
        self.assertEqual(mod.expected_control_count({}), 1)

    def test_row_is_false_pass(self):
        mod = load()
        self.assertTrue(mod.row_is_false_pass({"discriminates": False}))
        self.assertFalse(mod.row_is_false_pass({"discriminates": True}))
        self.assertTrue(mod.row_is_false_pass({}))
        self.assertFalse(mod.row_is_false_pass(None))

    def test_split_false_pass_control_label(self):
        mod = load()
        self.assertEqual(
            mod.split_false_pass_control_label("serialization/SR05 (garbage)"),
            ("serialization", "SR05", "garbage"),
        )
        self.assertIsNone(mod.split_false_pass_control_label("p/T"))
        self.assertIsNone(mod.split_false_pass_control_label("p/T (truncate)"))
        self.assertIsNone(mod.split_false_pass_control_label(""))

    def test_format_json_report_roundtrip(self):
        mod = load()
        report = mod.empty_report()
        text = mod.format_json_report(report)
        self.assertTrue(text.endswith("\n"))
        parsed = json.loads(text)
        self.assertEqual(parsed["kind"], "gymDiscrimination")
        self.assertEqual(mod.validate_report(parsed), [])

    def test_control_spec_returns_copy(self):
        mod = load()
        spec = mod.control_spec("garbage")
        spec["id"] = "mutated"
        self.assertEqual(mod.control_spec("garbage")["id"], "garbage")

    def test_submit_mapping_non_dict(self):
        mod = load()
        self.assertEqual(mod.submit_mapping(None), {})
        self.assertEqual(mod.submit_mapping({"submit": []}), {})
        self.assertEqual(mod.submit_mapping({"submit": {"kind": "artifact"}})["kind"], "artifact")

    def test_write_artifact_file_actions(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            src = os.path.join(mod.REPO_ROOT, SAMPLE)
            copied = os.path.join(d, "c.bin")
            garb = os.path.join(d, "g.bin")
            missing_dst = os.path.join(d, "m.bin")
            self.assertEqual(mod.write_artifact_file(copied, src, "input-copy"), "copied")
            self.assertEqual(mod.write_artifact_file(garb, src, "garbage"), "garbage")
            self.assertEqual(mod.write_artifact_file(missing_dst, os.path.join(d, "no"), "input-copy"), "skipped")
            self.assertEqual(mod.write_artifact_file(copied, src, "wrong-answer"), "rejected")
            self.assertEqual(_read(garb, binary=True), mod.GARBAGE_BYTES)

    def test_score_task_safe_folds_type_and_json(self):
        mod = load()
        typed = mod.score_task_safe({}, "p", "b", score_fn=lambda *_a, **_k: (_ for _ in ()).throw(TypeError("t")))
        self.assertIn("TypeError", typed["error"])
        jerr = mod.score_task_safe(
            {}, "p", "b",
            score_fn=lambda *_a, **_k: (_ for _ in ()).throw(json.JSONDecodeError("e", "{", 0)),
        )
        self.assertIn("JSONDecodeError", jerr["error"])

    def test_validate_report_bad_false_pass_label(self):
        mod = load()
        report = {
            "kind": "gymDiscrimination", "schemaVersion": "1.0", "ok": False,
            "taskCount": 1, "controlCount": 0, "discriminating": 0,
            "falsePass": ["nopath"], "falsePassControls": [],
        }
        issues = mod.validate_report(report)
        self.assertTrue(any("falsePass 라벨" in i for i in issues))

    def test_human_report_without_control_extra(self):
        mod = load()
        lines = mod.human_lines({"ok": False, "falsePass": ["p/T"], "falsePassControls": []})
        self.assertTrue(any("p/T" in line for line in lines))
        self.assertFalse(any(line.startswith("대조별") for line in lines))

    def test_default_neg_root_joins_submissions(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            path = mod.default_neg_root(d)
            self.assertEqual(path, os.path.join(d, "submissions", "_negative_control"))

    def test_load_json_reads_utf8(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "t.json")
            _write(path, '{"k":"한글"}')
            self.assertEqual(mod.load_json(path), {"k": "한글"})

    def test_build_negative_return_keys(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(_artifact_task(), d)
        for key in ("taskId", "mode", "answerKeys", "answerPath", "files", "errors", "dir"):
            self.assertIn(key, built)
        self.assertEqual(built["mode"], "input-copy")
        self.assertEqual(built["taskId"], "T")

    def test_artifact_files_not_list_does_not_raise(self):
        mod = load()
        task = {"id": "Q", "input": SAMPLE, "submit": {"kind": "artifact", "files": "out.svg"},
                "checks": []}
        with tempfile.TemporaryDirectory() as d:
            built = mod.build_negative(task, d)
        self.assertEqual(built["files"], [])

    def test_two_answer_tasks_false_pass_keep_order(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            _temp_gym(d, _answer_task(tid="B"), pack="p")
            _temp_gym(d, _answer_task(tid="A"), pack="p")
            report = mod.discriminate("bin", os.path.join(d, "gym"), os.path.join(d, "neg"))
        self.assertEqual(report["falsePass"], ["p/A", "p/B"])
        self.assertEqual(report["taskCount"], 2)

    def test_windows_rel_normalizes_to_posix(self):
        mod = load()
        self.assertEqual(mod.normalize_rel("nested\\out.svg"), "nested/out.svg")
        self.assertTrue(mod.is_safe_rel("nested\\out.svg"))

    def test_emit_json_uses_format_json_report(self):
        mod = load()
        report = mod.empty_report()
        buf = io.StringIO()
        mod.emit_report(report, True, stream=buf)
        self.assertEqual(buf.getvalue(), mod.format_json_report(report))

    def test_garbage_min_size_constant(self):
        mod = load()
        self.assertEqual(mod.GARBAGE_MIN_SIZE, 1024)
        self.assertGreater(len(mod.GARBAGE_MARKER) * mod.GARBAGE_REPEAT, 1024)

    def test_report_keys_do_not_overlap_optional(self):
        mod = load()
        self.assertEqual(set(mod.REPORT_KEYS) & set(mod.OPTIONAL_REPORT_KEYS), set())

    def test_fatal_tuple_has_four(self):
        mod = load()
        self.assertEqual(len(mod.FATAL_EXCEPTIONS), 4)

    def test_join_sub_nested_dirs_under_root(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            path = mod.join_sub(d, "a/b/c.svg")
            self.assertTrue(os.path.commonpath([d, path]) == os.path.abspath(d))

    def test_answer_task_control_count_in_live_report(self):
        mod = load()
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            report = mod.discriminate("bin", _temp_gym(d, _answer_task()), os.path.join(d, "neg"))
        self.assertEqual(report["controlCount"], mod.expected_control_count(_answer_task()))

    def test_split_label_roundtrip(self):
        mod = load()
        label = mod.false_pass_control_label("core-cli", "T01", "wrong-answer")
        self.assertEqual(mod.split_false_pass_control_label(label),
                         ("core-cli", "T01", "wrong-answer"))

    def test_docs_mention_three_control_ids(self):
        root = TOOL.parents[2]
        text = (root / "gym" / "docs" / "discriminate.md").read_text(encoding="utf-8")
        for token in ("wrong-answer", "input-copy", "garbage", "WRONG_SENTINEL", "GARBAGE_BYTES"):
            self.assertIn(token, text)

    def test_working_doc_points_at_issue(self):
        root = TOOL.parents[2]
        text = (root / "mydocs" / "working" / "gym_discriminate.md").read_text(encoding="utf-8")
        self.assertIn("#5255", text)
        self.assertIn("feat/gym-discriminate-hardening", text)
        self.assertIn("audit.py", text)


if __name__ == "__main__":
    unittest.main()

