"""[trajectory] gym 트라젝토리 필요성 감사 계약 — 무의미한 마지막 스텝(연극) 색출.

핵심: 다단계 과제는 마지막 스텝을 빼면(부분 트라젝토리) 채점에 실패해야 한다.
부분 트라젝토리가 통과 = 마지막 스텝이 load-bearing 아님 = 트라젝토리 연극.
조립·채점은 목킹해 바이너리 없이 로직만 시험한다.

예외 경로(침묵 금지): 기준풀이 부재, 빈 steps, 수집 전용 tail, 바이너리 부재.
마지막 스텝 load-bearing 판정 자체는 바꾸지 않는다.
"""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "trajectory.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_trajectory", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _write(path, payload):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        if isinstance(payload, str):
            fh.write(payload)
        else:
            json.dump(payload, fh, ensure_ascii=False)


def _task_body(task_id="T"):
    return {
        "id": task_id,
        "tier": 1,
        "title": "t",
        "input": "samples/x.hwp",
        "submit": {"kind": "artifact", "files": ["o"]},
        "checks": [],
    }


def _temp_gym(root, steps):
    """packs/p1 에 T 과제 + steps 개 스텝짜리 reference 를 심는다."""
    tasks = os.path.join(root, "gym", "packs", "p1", "tasks")
    refs = os.path.join(root, "gym", "packs", "p1", "reference")
    os.makedirs(tasks)
    os.makedirs(refs)
    task = _task_body("T")
    with open(os.path.join(tasks, "T.json"), "w", encoding="utf-8") as fh:
        json.dump(task, fh)
    ref = {"id": "T", "steps": [{"run": ["a"]} for _ in range(steps)]}
    with open(os.path.join(refs, "T.json"), "w", encoding="utf-8") as fh:
        json.dump(ref, fh)
    return os.path.join(root, "gym")


def _plant(gym_root, pack, task_id, steps=None, reference=None, task=None, write_ref=True):
    """한 과제를 gym_root/packs/<pack> 에 심는다."""
    tasks = os.path.join(gym_root, "packs", pack, "tasks")
    refs = os.path.join(gym_root, "packs", pack, "reference")
    os.makedirs(tasks, exist_ok=True)
    os.makedirs(refs, exist_ok=True)
    body = task if task is not None else _task_body(task_id)
    _write(os.path.join(tasks, f"{task_id}.json"), body)
    if write_ref:
        if reference is not None:
            _write(os.path.join(refs, f"{task_id}.json"), reference)
        else:
            step_list = steps if steps is not None else [{"run": ["a"]}, {"run": ["b"]}]
            _write(os.path.join(refs, f"{task_id}.json"), {"id": task_id, "steps": step_list})
    return gym_root


def _kinds(rows):
    return [row.get("kind") for row in rows]


def _by_task(rows, task_id):
    return [row for row in rows if row.get("task") == task_id]


class TrajectoryTests(unittest.TestCase):
    def test_flags_theater_when_truncated_passes(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None                 # 조립 no-op
        mod.runner.score_task = lambda task, sub_root, bin_path: {"pass": True}  # 부분이 통과=연극
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=2)
            r = mod.audit("bin", gym, os.path.join(d, "w"))
            self.assertFalse(r["ok"])
            self.assertIn("p1/T (마지막 실제 스텝 run을 빼도 통과 — 2→1)", r["theater"])

    def test_load_bearing_when_truncated_fails(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda task, sub_root, bin_path: {"pass": False}  # 부분이 실패=필수
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=3)
            r = mod.audit("bin", gym, os.path.join(d, "w"))
            self.assertTrue(r["ok"])
            self.assertEqual(r["theater"], [])
            self.assertEqual(r["loadBearing"], 1)

    def test_build_error_means_load_bearing(self):
        mod = load()
        def boom(*a, **k):
            raise RuntimeError("부분 트라젝토리가 유효 제출 못 만듦")
        mod.baseline.build_task = boom
        mod.runner.score_task = lambda *a, **k: {"pass": True}  # 조립이 터지면 채점까지 못 감
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=2)
            r = mod.audit("bin", gym, os.path.join(d, "w"))
            self.assertTrue(r["ok"])            # 필수로 취급
            self.assertEqual(r["theater"], [])

    def test_single_step_tasks_are_ignored(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=1)          # 단일 스텝 = 트라젝토리 아님
            r = mod.audit("bin", gym, os.path.join(d, "w"))
            self.assertEqual(r["taskCount"], 0)  # 감사 대상 아님
            self.assertTrue(r["ok"])

    def test_removes_last_meaningful_step_but_keeps_answer_collection(self):
        mod = load()
        captured = []

        def build(_bin, _pack, _task, reference, _root):
            captured.append(reference["steps"])

        mod.baseline.build_task = build
        mod.runner.score_task = lambda *_args: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=2)
            reference = Path(gym) / "packs" / "p1" / "reference" / "T.json"
            reference.write_text(json.dumps({"id": "T", "steps": [
                {"run": ["agent-action"]}, {"answer": {"value": {"const": 1}}}
            ]}), encoding="utf-8")
            r = mod.audit("bin", gym, os.path.join(d, "w"))

        self.assertTrue(r["ok"])
        self.assertEqual(captured, [[{"answer": {"value": {"const": 1}}}]])
        self.assertEqual(r["loadBearing"], 1)
        self.assertEqual(r["taskCount"], 1)


class CollectionStepTests(unittest.TestCase):
    def test_answer_is_collection(self):
        mod = load()
        self.assertTrue(mod.is_collection_step({"answer": {"value": 1}}))

    def test_keyring_from_is_collection(self):
        mod = load()
        self.assertTrue(mod.is_collection_step({"keyring_from": "T13"}))

    def test_run_is_meaningful(self):
        mod = load()
        self.assertFalse(mod.is_collection_step({"run": ["info"]}))
        self.assertTrue(mod.has_meaningful_key({"run": ["info"]}))

    def test_mixed_keys_with_answer_count_as_collection(self):
        mod = load()
        # 수집 키가 하나라도 있으면 수집. 마지막 의미 스텝을 고를 때 tail 로 남긴다.
        self.assertTrue(mod.is_collection_step({"answer": {}, "run": ["x"]}))

    def test_non_mapping_is_not_collection(self):
        mod = load()
        self.assertFalse(mod.is_collection_step(None))
        self.assertFalse(mod.is_collection_step("run"))
        self.assertFalse(mod.is_collection_step(["run"]))
        self.assertFalse(mod.has_meaningful_key(None))

    def test_empty_mapping_is_meaningful(self):
        mod = load()
        # 키가 없으면 수집이 아니다. last_meaningful 이 이 칸을 고른다.
        self.assertFalse(mod.is_collection_step({}))
        self.assertTrue(mod.has_meaningful_key({}))

    def test_step_kind_label_sorts_keys(self):
        mod = load()
        self.assertEqual(mod.step_kind_label({"run": 1}), "run")
        self.assertEqual(mod.step_kind_label({"b": 1, "a": 2}), "a/b")
        self.assertEqual(mod.step_kind_label({}), "empty")
        self.assertEqual(mod.step_kind_label("nope"), "empty")

    def test_step_keys_are_strings(self):
        mod = load()
        self.assertEqual(mod.step_keys({1: "x", "a": 2}), ["1", "a"])


class LastMeaningfulStepTests(unittest.TestCase):
    def test_last_run_when_no_collection_tail(self):
        mod = load()
        steps = [{"run": ["a"]}, {"run": ["b"]}]
        self.assertEqual(mod.last_meaningful_step_index(steps), 1)

    def test_skips_trailing_answer(self):
        mod = load()
        steps = [{"run": ["a"]}, {"run": ["b"]}, {"answer": {}}]
        self.assertEqual(mod.last_meaningful_step_index(steps), 1)

    def test_skips_trailing_keyring_and_answer(self):
        mod = load()
        steps = [{"run": ["a"]}, {"keyring_from": "x"}, {"answer": {}}]
        self.assertEqual(mod.last_meaningful_step_index(steps), 0)

    def test_all_collection_returns_none(self):
        mod = load()
        steps = [{"answer": {}}, {"keyring_from": "x"}]
        self.assertIsNone(mod.last_meaningful_step_index(steps))

    def test_empty_list_returns_none(self):
        mod = load()
        self.assertIsNone(mod.last_meaningful_step_index([]))

    def test_non_list_returns_none(self):
        mod = load()
        self.assertIsNone(mod.last_meaningful_step_index(None))
        self.assertIsNone(mod.last_meaningful_step_index({"run": 1}))

    def test_non_mapping_entries_are_skipped(self):
        mod = load()
        steps = [{"run": ["a"]}, "broken", None]
        self.assertEqual(mod.last_meaningful_step_index(steps), 0)

    def test_middle_collection_does_not_hide_later_run(self):
        mod = load()
        steps = [{"run": ["a"]}, {"answer": {}}, {"run": ["b"]}]
        self.assertEqual(mod.last_meaningful_step_index(steps), 2)


class TruncateTests(unittest.TestCase):
    def test_drops_only_removed_index(self):
        mod = load()
        steps = [{"run": ["a"]}, {"run": ["b"]}, {"answer": {}}]
        self.assertEqual(mod.truncate_steps(steps, 1), [{"run": ["a"]}, {"answer": {}}])

    def test_does_not_mutate_original(self):
        mod = load()
        steps = [{"run": ["a"]}, {"run": ["b"]}]
        out = mod.truncate_steps(steps, 1)
        self.assertEqual(steps, [{"run": ["a"]}, {"run": ["b"]}])
        self.assertEqual(out, [{"run": ["a"]}])

    def test_out_of_range_returns_copy(self):
        mod = load()
        steps = [{"run": ["a"]}]
        self.assertEqual(mod.truncate_steps(steps, 3), [{"run": ["a"]}])
        self.assertEqual(mod.truncate_steps(steps, -1), [{"run": ["a"]}])

    def test_non_list_is_empty(self):
        mod = load()
        self.assertEqual(mod.truncate_steps(None, 0), [])

    def test_truncate_reference_copies_other_keys(self):
        mod = load()
        ref = {"id": "T", "note": "keep", "steps": [{"run": ["a"]}, {"run": ["b"]}]}
        out = mod.truncate_reference(ref, 1)
        self.assertEqual(out["id"], "T")
        self.assertEqual(out["note"], "keep")
        self.assertEqual(out["steps"], [{"run": ["a"]}])
        self.assertEqual(ref["steps"], [{"run": ["a"]}, {"run": ["b"]}])

    def test_truncate_reference_non_dict(self):
        mod = load()
        self.assertEqual(mod.truncate_reference(None, 0), {"steps": []})


CLASSIFY_CASES = (
    (None, "empty-steps"),
    ([], "empty-steps"),
    ("nope", "malformed-reference"),
    ({"run": ["a"]}, "malformed-reference"),
    ([{"run": ["a"]}], "single-step"),
    ([{"answer": {}}], "single-step"),
    ([{"keyring_from": "x"}], "single-step"),
    ([{"run": ["a"]}, {"run": ["b"]}], "multi"),
    ([{"run": ["a"]}, {"answer": {}}], "multi"),
    ([{"run": ["a"]}, {"keyring_from": "x"}], "multi"),
    ([{"answer": {}}, {"keyring_from": "x"}], "collection-only-tail"),
    ([{"answer": {}}, {"answer": {}}, {"keyring_from": "x"}], "collection-only-tail"),
    ([{}, {"answer": {}}], "multi"),
    ([{"run": ["a"]}, {"run": ["b"]}, {"run": ["c"]}], "multi"),
)


class ClassifyStepsTests(unittest.TestCase):
    def test_catalog_table(self):
        mod = load()
        for steps, expected in CLASSIFY_CASES:
            with self.subTest(steps=steps, expected=expected):
                self.assertEqual(mod.classify_steps(steps), expected)

    def test_classify_reference_reads_steps(self):
        mod = load()
        self.assertEqual(mod.classify_reference({"steps": [{"run": ["a"]}, {"run": ["b"]}]}), "multi")
        self.assertEqual(mod.classify_reference({"id": "T"}), "empty-steps")
        self.assertEqual(mod.classify_reference({"steps": []}), "empty-steps")
        self.assertEqual(mod.classify_reference({"steps": None}), "empty-steps")
        self.assertEqual(mod.classify_reference(None), "malformed-reference")
        self.assertEqual(mod.classify_reference("x"), "malformed-reference")
        self.assertEqual(mod.classify_reference({"steps": {"run": 1}}), "malformed-reference")
        self.assertEqual(mod.classify_reference({"steps": 3}), "malformed-reference")
        self.assertEqual(mod.classify_reference({"steps": "run"}), "malformed-reference")

    def test_audit_candidate_only_multi(self):
        mod = load()
        self.assertTrue(mod.is_audit_candidate("multi"))
        for label in ("single-step", "empty-steps", "collection-only-tail", "missing-reference"):
            self.assertFalse(mod.is_audit_candidate(label))

    def test_skip_only_single(self):
        mod = load()
        self.assertTrue(mod.is_skip_label("single-step"))
        self.assertFalse(mod.is_skip_label("multi"))
        self.assertFalse(mod.is_skip_label("empty-steps"))

    def test_exception_labels(self):
        mod = load()
        for label in ("missing-reference", "empty-steps", "collection-only-tail", "missing-bin"):
            self.assertTrue(mod.is_exception_label(label), label)
        self.assertFalse(mod.is_exception_label("single-step"))
        self.assertFalse(mod.is_exception_label("multi"))


class ExceptionKindTests(unittest.TestCase):
    def test_file_not_found_is_missing_bin(self):
        mod = load()
        self.assertEqual(mod.exception_kind(FileNotFoundError("rhwp")), "missing-bin")
        self.assertTrue(mod.is_missing_bin_exception(FileNotFoundError("rhwp")))

    def test_runtime_error_is_not_missing_bin(self):
        mod = load()
        self.assertFalse(mod.is_missing_bin_exception(RuntimeError("boom")))
        self.assertEqual(mod.verdict_from_build_error(RuntimeError("boom")), "load-bearing")

    def test_file_not_found_verdict_is_missing_bin(self):
        mod = load()
        self.assertEqual(mod.verdict_from_build_error(FileNotFoundError("x")), "missing-bin")

    def test_permission_and_os(self):
        mod = load()
        self.assertEqual(mod.exception_kind(PermissionError("x")), "permission")
        self.assertEqual(mod.exception_kind(TimeoutError("x")), "timeout")
        self.assertEqual(mod.exception_kind(UnicodeDecodeError("utf-8", b"\xff", 0, 1, "bad")), "decode-error")
        self.assertEqual(mod.exception_kind(TypeError("x")), "type-error")
        self.assertEqual(mod.exception_kind(ValueError("x")), "value-error")
        self.assertEqual(mod.exception_kind(KeyError("x")), "value-error")

    def test_json_decode_depends_on_context(self):
        mod = load()
        exc = json.JSONDecodeError("msg", "doc", 0)
        self.assertEqual(mod.exception_kind(exc, context="load"), "malformed-json")
        self.assertEqual(mod.exception_kind(exc, context="audit"), "value-error")

    def test_none_is_unexpected(self):
        mod = load()
        self.assertEqual(mod.exception_kind(None), "unexpected")

    def test_catalog_contains_required_paths(self):
        mod = load()
        for kind in ("missing-reference", "empty-steps", "collection-only-tail", "missing-bin"):
            self.assertIn(kind, mod.EXCEPTION_KINDS)
            self.assertTrue(mod.is_known_exception_kind(kind))

    def test_unknown_kind_is_folded(self):
        mod = load()
        row = mod.exception_row("not-a-kind", pack="p", task="T")
        self.assertEqual(row["kind"], "unexpected")

    def test_exception_row_truncates_head(self):
        mod = load()
        row = mod.exception_row("empty-steps", pack="p", task="T", head="가" * 400)
        self.assertEqual(len(row["head"]), mod.ERROR_HEAD_LIMIT)
        self.assertEqual(row["pack"], "p")
        self.assertEqual(row["task"], "T")

    def test_fatal_exceptions(self):
        mod = load()
        self.assertTrue(mod.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(mod.is_fatal_exception(SystemExit(1)))
        self.assertTrue(mod.is_fatal_exception(MemoryError()))
        self.assertTrue(mod.is_fatal_exception(GeneratorExit()))
        self.assertFalse(mod.is_fatal_exception(RuntimeError("x")))
        self.assertEqual(mod.verdict_from_build_error(KeyboardInterrupt()), "fatal")


class VerdictTests(unittest.TestCase):
    def test_score_pass_is_theater(self):
        mod = load()
        self.assertFalse(mod.verdict_from_score({"pass": True}))

    def test_score_fail_is_load_bearing(self):
        mod = load()
        self.assertTrue(mod.verdict_from_score({"pass": False}))

    def test_missing_pass_is_load_bearing(self):
        mod = load()
        self.assertTrue(mod.verdict_from_score({}))
        self.assertTrue(mod.verdict_from_score(None))
        self.assertTrue(mod.verdict_from_score("pass"))

    def test_theater_line_keeps_legacy_wording(self):
        mod = load()
        line = mod.make_theater_line("p1", "T", "run", 2)
        self.assertEqual(line, "p1/T (마지막 실제 스텝 run을 빼도 통과 — 2→1)")

    def test_format_exception_line(self):
        mod = load()
        row = mod.exception_row("missing-reference", pack="p1", task="T", head="짝 기준풀이가 없다")
        text = mod.format_exception_line(row)
        self.assertIn("missing-reference", text)
        self.assertIn("p1/T", text)
        self.assertEqual(mod.format_exception_line(None), "예외: (형식 오류)")
        self.assertEqual(mod.format_exception_line({"kind": "empty-steps"}), "empty-steps")


class ReportContractTests(unittest.TestCase):
    def test_empty_report_is_valid(self):
        mod = load()
        report = mod.empty_report()
        self.assertEqual(mod.validate_report(report), [])
        self.assertTrue(report["ok"])
        self.assertTrue(report["trusted"])
        self.assertEqual(report["exit"], 0)
        self.assertEqual(report["kind"], "gymTrajectoryNecessity")
        self.assertEqual(report["schemaVersion"], "1.0")

    def test_ok_is_theater_only(self):
        mod = load()
        self.assertTrue(mod.report_ok([]))
        self.assertFalse(mod.report_ok(["x"]))
        self.assertTrue(mod.report_ok([], missing_bin=True))

    def test_exit_fails_on_missing_bin_even_when_ok(self):
        mod = load()
        self.assertEqual(mod.report_exit([], missing_bin=True), 1)
        self.assertEqual(mod.report_exit(["t"]), 1)
        self.assertEqual(mod.report_exit([]), 0)

    def test_trusted_false_when_exceptions(self):
        mod = load()
        self.assertFalse(mod.report_trusted([{"kind": "empty-steps"}]))
        self.assertFalse(mod.report_trusted([], missing_bin=True))
        self.assertTrue(mod.report_trusted([]))

    def test_attach_counts_recomputes(self):
        mod = load()
        report = mod.empty_report()
        report["results"] = [
            mod.make_result_row("p", "A", True, 2, "run"),
            mod.make_result_row("p", "B", False, 2, "run"),
        ]
        report["theater"] = [mod.make_theater_line("p", "B", "run", 2)]
        report["exceptions"] = [mod.exception_row("empty-steps", pack="p", task="C")]
        report["skipped"] = [mod.make_skip_row("p", "D", "single-step", 1)]
        mod.attach_report_counts(report)
        self.assertEqual(report["taskCount"], 2)
        self.assertEqual(report["loadBearing"], 1)
        self.assertEqual(report["exceptionCount"], 1)
        self.assertEqual(report["skipCount"], 1)
        self.assertFalse(report["ok"])
        self.assertFalse(report["trusted"])
        self.assertEqual(report["exit"], 1)
        self.assertEqual(mod.validate_report(report), [])

    def test_validate_catches_ok_lie(self):
        mod = load()
        report = mod.empty_report()
        report["theater"] = ["x"]
        report["ok"] = True
        issues = mod.validate_report(report)
        self.assertTrue(any("ok" in item for item in issues))

    def test_validate_catches_missing_bin_exit(self):
        mod = load()
        report = mod.empty_report()
        report["missingBin"] = True
        report["exit"] = 0
        issues = mod.validate_report(report)
        self.assertTrue(any("missing-bin" in item for item in issues))

    def test_validate_non_dict(self):
        mod = load()
        self.assertEqual(mod.validate_report(None), ["report 가 dict 가 아니다"])

    def test_required_keys_present(self):
        mod = load()
        for key in mod.REPORT_KEYS:
            self.assertIn(key, mod.empty_report())


class TruncateHeadTests(unittest.TestCase):
    def test_none_and_non_string(self):
        mod = load()
        self.assertEqual(mod.truncate_head(None), "")
        self.assertEqual(mod.truncate_head(12), "12")

    def test_limit_zero(self):
        mod = load()
        self.assertEqual(mod.truncate_head("abc", 0), "")

    def test_bad_limit_falls_back(self):
        mod = load()
        text = "x" * 200
        self.assertEqual(len(mod.truncate_head(text, "nope")), mod.HEAD_LIMIT)


class ScanDiscoveryTests(unittest.TestCase):
    def test_missing_reference_is_exception_not_skip(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", write_ref=False)
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(r["taskCount"], 0)
        self.assertEqual(_kinds(r["exceptions"]), ["missing-reference"])
        self.assertEqual(r["exceptions"][0]["task"], "T")
        self.assertTrue(r["ok"])
        self.assertFalse(r["trusted"])
        self.assertEqual(r["exit"], 0)

    def test_empty_steps_is_exception(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", steps=[])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(_kinds(r["exceptions"]), ["empty-steps"])
        self.assertEqual(r["taskCount"], 0)
        self.assertEqual(r["skipCount"], 0)
        self.assertTrue(r["ok"])

    def test_missing_steps_key_is_empty_steps(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", reference={"id": "T"})
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(_kinds(r["exceptions"]), ["empty-steps"])

    def test_collection_only_tail_is_exception(self):
        mod = load()
        called = {"n": 0}

        def build(*_a, **_k):
            called["n"] += 1

        mod.baseline.build_task = build
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", steps=[{"answer": {}}, {"keyring_from": "x"}])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(_kinds(r["exceptions"]), ["collection-only-tail"])
        self.assertEqual(called["n"], 0)
        self.assertEqual(r["taskCount"], 0)
        self.assertEqual(r["theater"], [])

    def test_single_answer_task_is_skip_not_collection_only(self):
        # T01 형태. 단스텝 answer 는 예외가 아니다.
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T01", steps=[{"answer": {"pages": 1}}])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(r["exceptions"], [])
        self.assertEqual(r["skipCount"], 1)
        self.assertEqual(r["skipped"][0]["reason"], "single-step")
        self.assertEqual(r["taskCount"], 0)

    def test_malformed_task_json(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            tasks = os.path.join(gym, "packs", "p1", "tasks")
            refs = os.path.join(gym, "packs", "p1", "reference")
            os.makedirs(tasks)
            os.makedirs(refs)
            _write(os.path.join(tasks, "T.json"), "{not-json")
            _write(os.path.join(refs, "T.json"), {"id": "T", "steps": [{"run": ["a"]}, {"run": ["b"]}]})
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertIn(r["exceptions"][0]["kind"], ("malformed-json", "malformed-task"))
        self.assertEqual(r["taskCount"], 0)

    def test_malformed_reference_json(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", write_ref=False)
            _write(os.path.join(gym, "packs", "p1", "reference", "T.json"), "{nope")
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(r["exceptions"][0]["kind"], "malformed-json")

    def test_steps_object_is_malformed_reference(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", reference={"id": "T", "steps": {"run": ["a"]}})
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(_kinds(r["exceptions"]), ["malformed-reference"])

    def test_scan_is_deterministic_across_packs(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "b-pack", "B", write_ref=False)
            _plant(gym, "a-pack", "A", steps=[])
            recs, err = mod.scan_gym(gym)
        self.assertEqual(err, [])
        self.assertEqual([(r["pack"], r["name"]) for r in recs], [("a-pack", "A.json"), ("b-pack", "B.json")])

    def test_pack_without_tasks_dir_is_ignored(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            os.makedirs(os.path.join(gym, "packs", "empty"))
            recs, err = mod.scan_gym(gym)
        self.assertEqual(recs, [])
        self.assertEqual(err, [])

    def test_non_json_files_are_ignored(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", steps=[{"run": ["a"]}])
            _write(os.path.join(gym, "packs", "p1", "tasks", "notes.txt"), "x")
            recs, _ = mod.scan_gym(gym)
        self.assertEqual(len(recs), 1)
        self.assertEqual(recs[0]["name"], "T.json")


class MissingBinTests(unittest.TestCase):
    def test_build_file_not_found_is_not_load_bearing(self):
        mod = load()

        def boom(*_a, **_k):
            raise FileNotFoundError("rhwp")

        mod.baseline.build_task = boom
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=2)
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(r["loadBearing"], 0)
        self.assertEqual(r["taskCount"], 0)
        self.assertEqual(_kinds(r["exceptions"]), ["missing-bin"])
        self.assertTrue(r["ok"])
        self.assertTrue(r["missingBin"])
        self.assertFalse(r["trusted"])
        self.assertEqual(r["exit"], 1)

    def test_score_file_not_found_is_not_load_bearing(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None

        def boom(*_a, **_k):
            raise FileNotFoundError("rhwp")

        mod.runner.score_task = boom
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=2)
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(_kinds(r["exceptions"]), ["missing-bin"])
        self.assertEqual(r["loadBearing"], 0)
        self.assertEqual(r["exit"], 1)

    def test_missing_bin_does_not_mark_later_tasks_load_bearing(self):
        mod = load()
        calls = {"n": 0}

        def boom(*_a, **_k):
            calls["n"] += 1
            raise FileNotFoundError("rhwp")

        mod.baseline.build_task = boom
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "A", steps=[{"run": ["a"]}, {"run": ["b"]}])
            _plant(gym, "p1", "B", steps=[{"run": ["a"]}, {"run": ["b"]}])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(calls["n"], 1)
        self.assertEqual(r["taskCount"], 0)
        self.assertEqual(r["loadBearing"], 0)
        self.assertTrue(r["missingBin"])
        self.assertEqual(sum(1 for row in r["exceptions"] if row["kind"] == "missing-bin"), 1)

    def test_dummy_bin_name_is_not_prechecked(self):
        # 시험이 넘기는 "bin" 은 파일이 없어도 조립 목킹 경로로 간다.
        mod = load()
        self.assertTrue(mod.bin_looks_present("bin"))
        self.assertTrue(mod.bin_looks_present("rhwp"))
        self.assertFalse(mod.bin_looks_present(""))
        self.assertFalse(mod.bin_looks_present(None))

    def test_path_like_missing_file_is_absent(self):
        mod = load()
        self.assertFalse(mod.bin_looks_present(os.path.join("no", "such", "rhwp")))
        self.assertFalse(mod.bin_looks_present("C:/definitely/missing/rhwp.exe"))


class MixedGymTests(unittest.TestCase):
    def test_mixed_exceptions_and_one_theater(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "OK", steps=[{"run": ["a"]}, {"run": ["b"]}])
            _plant(gym, "p1", "MISS", write_ref=False)
            _plant(gym, "p1", "EMPTY", steps=[])
            _plant(gym, "p1", "TAIL", steps=[{"answer": {}}, {"keyring_from": "x"}])
            _plant(gym, "p1", "ONE", steps=[{"run": ["only"]}])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        kinds = sorted(_kinds(r["exceptions"]))
        self.assertEqual(kinds, ["collection-only-tail", "empty-steps", "missing-reference"])
        self.assertEqual(r["taskCount"], 1)
        self.assertFalse(r["ok"])
        self.assertEqual(r["skipCount"], 1)
        self.assertEqual(len(r["theater"]), 1)
        self.assertIn("p1/OK", r["theater"][0])

    def test_keeps_answer_tail_in_mixed_pack(self):
        mod = load()
        captured = []

        def build(_bin, _pack, _task, reference, _root):
            captured.append(( _task["id"], reference["steps"]))

        mod.baseline.build_task = build
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", steps=[{"run": ["act"]}, {"answer": {"n": 1}}])
            _plant(gym, "p1", "U", steps=[{"run": ["act"]}, {"run": ["act2"]}, {"keyring_from": "T"}])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(r["loadBearing"], 2)
        self.assertEqual(r["exceptions"], [])
        ids = [item[0] for item in captured]
        self.assertEqual(ids, ["T", "U"])
        self.assertEqual(captured[0][1], [{"answer": {"n": 1}}])
        self.assertEqual(captured[1][1], [{"run": ["act"]}, {"keyring_from": "T"}])

    def test_multi_step_tasks_still_yields_collection_only(self):
        # 예전 생성기는 길이 ≥2 이면 배출한다. 분류는 audit 가 한다.
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "TAIL", steps=[{"answer": {}}, {"keyring_from": "x"}])
            pairs = list(mod.multi_step_tasks(gym))
        self.assertEqual(len(pairs), 1)
        self.assertEqual(pairs[0][0], "p1")
        self.assertEqual(pairs[0][1]["id"], "TAIL")

    def test_multi_step_tasks_skips_missing_and_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "MISS", write_ref=False)
            _plant(gym, "p1", "EMPTY", steps=[])
            _plant(gym, "p1", "ONE", steps=[{"run": ["a"]}])
            pairs = list(mod.multi_step_tasks(gym))
        self.assertEqual(pairs, [])


class RenderTextTests(unittest.TestCase):
    def test_ok_message(self):
        mod = load()
        report = mod.empty_report()
        report["taskCount"] = 3
        text = mod.render_text_report(report)
        self.assertIn("3 다단계 과제 전부", text)
        self.assertIn("연극 0", text)

    def test_theater_message(self):
        mod = load()
        report = mod.empty_report()
        report["ok"] = False
        report["theater"] = ["p1/T (마지막 실제 스텝 run을 빼도 통과 — 2→1)"]
        text = mod.render_text_report(report)
        self.assertIn("연극(무의미한 마지막 스텝) 1건", text)
        self.assertIn("p1/T", text)

    def test_lists_exceptions(self):
        mod = load()
        report = mod.empty_report()
        report["exceptions"] = [mod.exception_row("empty-steps", pack="p1", task="T", head="비었다")]
        report["exceptionCount"] = 1
        text = mod.render_text_report(report)
        self.assertIn("예외 경로 1건", text)
        self.assertIn("empty-steps", text)

    def test_tool_failed_without_theater(self):
        mod = load()
        report = mod.empty_report()
        report["missingBin"] = True
        report["toolFailed"] = True
        report["trusted"] = False
        report["exceptionCount"] = 1
        text = mod.render_text_report(report)
        self.assertIn("도구 실패", text)

    def test_non_dict_report(self):
        mod = load()
        self.assertIn("보고 봉투가 아니다", mod.render_text_report(None))


class AuditOneTests(unittest.TestCase):
    def test_collection_only_returns_exception(self):
        mod = load()
        out = mod.audit_one("bin", "p", {"id": "T"}, {"steps": [{"answer": {}}, {"answer": {}}]}, "w")
        self.assertIsNone(out["result"])
        self.assertEqual(out["exception"]["kind"], "collection-only-tail")

    def test_malformed_steps_returns_exception(self):
        mod = load()
        out = mod.audit_one("bin", "p", {"id": "T"}, {"steps": {"run": 1}}, "w")
        self.assertEqual(out["exception"]["kind"], "malformed-reference")

    def test_fatal_is_propagated_from_audit(self):
        mod = load()

        def boom(*_a, **_k):
            raise KeyboardInterrupt()

        mod.baseline.build_task = boom
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=2)
            with self.assertRaises(KeyboardInterrupt):
                mod.audit("bin", gym, os.path.join(d, "w"))


class SafeIoTests(unittest.TestCase):
    def test_safe_load_json_ok(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "a.json")
            _write(path, {"id": "T"})
            obj, err = mod.safe_load_json(path)
        self.assertEqual(obj, {"id": "T"})
        self.assertIsNone(err)

    def test_safe_load_json_bad(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "a.json")
            _write(path, "{")
            obj, err = mod.safe_load_json(path)
        self.assertIsNone(obj)
        self.assertIsInstance(err, json.JSONDecodeError)

    def test_safe_load_json_missing(self):
        mod = load()
        obj, err = mod.safe_load_json(os.path.join("no", "such.json"))
        self.assertIsNone(obj)
        self.assertIsNotNone(err)

    def test_safe_listdir_missing(self):
        mod = load()
        names, err = mod.safe_listdir(os.path.join("no", "such", "dir"))
        self.assertIsNone(names)
        self.assertIsNotNone(err)

    def test_safe_isdir_isfile_false_on_error(self):
        mod = load()
        self.assertFalse(mod.safe_isdir(os.path.join("no", "dir")))
        self.assertFalse(mod.safe_isfile(os.path.join("no", "file")))

    def test_task_id_fallback(self):
        mod = load()
        self.assertEqual(mod.task_id_of({"id": "T01"}, "x.json"), "T01")
        self.assertEqual(mod.task_id_of({}, "T01.json"), "T01")
        self.assertEqual(mod.task_id_of(None, "T01.json"), "T01")
        self.assertEqual(mod.task_id_of({"id": ""}, "Z.json"), "Z")


class MainCliTests(unittest.TestCase):
    def test_json_missing_bin_path_exits_one(self):
        mod = load()
        missing = os.path.join("definitely", "missing", "rhwp-not-here")
        with mock.patch.object(mod.sys, "argv", ["trajectory.py", "--bin", missing, "--json"]):
            with mock.patch.object(mod.sys.stdout, "write") as write:
                code = mod.main()
        self.assertEqual(code, 1)
        payload = json.loads(write.call_args[0][0])
        self.assertTrue(payload["missingBin"])
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["exit"], 1)
        self.assertIn("missing-bin", _kinds(payload["exceptions"]))

    def test_text_ok_path_uses_render(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, steps=2)
            captured = {}

            def fake_audit(*_a, **_k):
                captured["ran"] = True
                report = mod.empty_report()
                report["taskCount"] = 1
                report["loadBearing"] = 1
                return report

            with mock.patch.object(mod, "GYM_ROOT", gym):
                with mock.patch.object(mod, "audit", fake_audit):
                    with mock.patch.object(mod, "resolve_bin_safe", return_value=("bin", None)):
                        with mock.patch.object(mod.sys, "argv", ["trajectory.py", "--bin", "bin"]):
                            with mock.patch.object(mod.sys.stdout, "write") as write:
                                code = mod.main()
        self.assertEqual(code, 0)
        self.assertTrue(captured.get("ran"))
        self.assertIn("연극 0", write.call_args[0][0])


class GeneratedClassifyTableTests(unittest.TestCase):
    """분류 표를 한 칸씩 다시 고정한다. 단스텝 answer ≠ collection-only."""

    CASES = (
        ("empty_none", None, "empty-steps"),
        ("empty_list", [], "empty-steps"),
        ("single_run", [{"run": [1]}], "single-step"),
        ("single_answer", [{"answer": {}}], "single-step"),
        ("single_keyring", [{"keyring_from": "A"}], "single-step"),
        ("two_run", [{"run": [1]}, {"run": [2]}], "multi"),
        ("run_answer", [{"run": [1]}, {"answer": {}}], "multi"),
        ("run_keyring", [{"run": [1]}, {"keyring_from": "A"}], "multi"),
        ("two_answer", [{"answer": {}}, {"answer": {}}], "collection-only-tail"),
        ("answer_keyring", [{"answer": {}}, {"keyring_from": "A"}], "collection-only-tail"),
        ("three_collection", [{"answer": {}}, {"answer": {}}, {"keyring_from": "A"}], "collection-only-tail"),
        ("three_multi", [{"run": [1]}, {"run": [2]}, {"answer": {}}], "multi"),
        ("empty_map_then_answer", [{}, {"answer": {}}], "multi"),
        ("not_list", {"steps": 1}, "malformed-reference"),
    )

    def test_each_row(self):
        mod = load()
        for name, steps, expected in self.CASES:
            with self.subTest(name=name):
                self.assertEqual(mod.classify_steps(steps), expected)


class GeneratedExceptionCatalogTests(unittest.TestCase):
    def test_required_kinds_are_stable(self):
        mod = load()
        required = (
            "missing-reference",
            "empty-steps",
            "collection-only-tail",
            "missing-bin",
        )
        self.assertEqual(mod.EXCEPTION_KINDS[:4], required)

    def test_step_labels_are_stable(self):
        mod = load()
        self.assertEqual(
            mod.STEP_LABELS,
            ("multi", "single-step", "empty-steps", "collection-only-tail", "malformed-reference"),
        )

    def test_collection_keys_are_stable(self):
        mod = load()
        self.assertEqual(set(mod.COLLECTION_STEP_KEYS), {"answer", "keyring_from"})

    def test_report_kind_stable(self):
        mod = load()
        self.assertEqual(mod.REPORT_KIND, "gymTrajectoryNecessity")
        self.assertEqual(mod.SCHEMA_VERSION, "1.0")
        self.assertEqual(mod.EXIT_OK, 0)
        self.assertEqual(mod.EXIT_FAILED, 1)


class GeneratedHonestyMatrixTests(unittest.TestCase):
    """ok / exit / trusted 정직 행렬. 연극과 도구 실패를 섞지 않는다."""

    def _matrix(self):
        return (
            # theater, missing_bin, exceptions, expected_ok, expected_exit, expected_trusted
            (False, False, False, True, 0, True),
            (True, False, False, False, 1, True),
            (False, True, False, True, 1, False),
            (True, True, False, False, 1, False),
            (False, False, True, True, 0, False),
            (True, False, True, False, 1, False),
            (False, True, True, True, 1, False),
        )

    def test_matrix(self):
        mod = load()
        for theater, missing_bin, has_exc, ok, exit_code, trusted in self._matrix():
            with self.subTest(theater=theater, missing_bin=missing_bin, has_exc=has_exc):
                theater_list = ["x"] if theater else []
                exceptions = [{"kind": "empty-steps"}] if has_exc else []
                self.assertEqual(mod.report_ok(theater_list, missing_bin=missing_bin), ok)
                self.assertEqual(
                    mod.report_exit(theater_list, missing_bin=missing_bin, tool_failed=missing_bin),
                    exit_code,
                )
                self.assertEqual(
                    mod.report_trusted(exceptions, tool_failed=missing_bin, missing_bin=missing_bin),
                    trusted,
                )


class LoadBearingLogicKeptTests(unittest.TestCase):
    """마지막 스텝 판정 다섯 칸을 예외 보강 뒤에도 다시 고정한다."""

    def test_pass_true_is_still_theater(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            r = mod.audit("bin", _temp_gym(d, 2), os.path.join(d, "w"))
        self.assertFalse(r["ok"])
        self.assertEqual(r["loadBearing"], 0)

    def test_pass_false_is_still_load_bearing(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: None
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            r = mod.audit("bin", _temp_gym(d, 2), os.path.join(d, "w"))
        self.assertTrue(r["ok"])
        self.assertEqual(r["loadBearing"], 1)

    def test_runtime_error_is_still_load_bearing(self):
        mod = load()
        mod.baseline.build_task = lambda *a, **k: (_ for _ in ()).throw(RuntimeError("x"))
        mod.runner.score_task = lambda *a, **k: {"pass": True}
        with tempfile.TemporaryDirectory() as d:
            r = mod.audit("bin", _temp_gym(d, 2), os.path.join(d, "w"))
        self.assertTrue(r["ok"])
        self.assertEqual(r["loadBearing"], 1)
        self.assertEqual(r["exceptions"], [])

    def test_three_steps_removes_only_last_meaningful(self):
        mod = load()
        captured = []
        mod.baseline.build_task = lambda _b, _p, _t, ref, _r: captured.append(ref["steps"])
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", steps=[
                {"run": ["one"]},
                {"run": ["two"]},
                {"run": ["three"]},
                {"answer": {}},
            ])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(r["loadBearing"], 1)
        self.assertEqual(captured, [[{"run": ["one"]}, {"run": ["two"]}, {"answer": {}}]])
        self.assertEqual(r["results"][0]["removedStep"], "run")
        self.assertEqual(r["results"][0]["steps"], 4)


class ScanTaskPairUnitTests(unittest.TestCase):
    def test_missing_ref_uses_task_id(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            task_path = os.path.join(d, "T.json")
            _write(task_path, _task_body("HELLO"))
            rec = mod.scan_task_pair("p", "T.json", task_path, os.path.join(d, "no.json"))
        self.assertEqual(rec["label"], "missing-reference")
        self.assertEqual(rec["exception"]["task"], "HELLO")

    def test_task_not_object(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            task_path = os.path.join(d, "T.json")
            _write(task_path, [1, 2, 3])
            rec = mod.scan_task_pair("p", "T.json", task_path, os.path.join(d, "r.json"))
        self.assertEqual(rec["label"], "malformed-task")

    def test_reference_not_object(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            task_path = os.path.join(d, "T.json")
            ref_path = os.path.join(d, "R.json")
            _write(task_path, _task_body("T"))
            _write(ref_path, [1])
            rec = mod.scan_task_pair("p", "T.json", task_path, ref_path)
        self.assertEqual(rec["label"], "malformed-reference")


class ResolveBinSafeTests(unittest.TestCase):
    def test_plain_name_does_not_invent_missing_bin(self):
        mod = load()
        path, err = mod.resolve_bin_safe("bin")
        self.assertIsNone(err)
        self.assertTrue(path)

    def test_missing_path_returns_error(self):
        mod = load()
        path, err = mod.resolve_bin_safe(os.path.join("no", "rhwp"))
        self.assertIsInstance(err, FileNotFoundError)
        self.assertTrue(path)


class StepsOfReferenceTests(unittest.TestCase):
    def test_none_reference(self):
        mod = load()
        self.assertIsNone(mod.steps_of_reference(None))
        self.assertIsNone(mod.normalize_steps(None))
        self.assertEqual(mod.normalize_steps([]), [])
        self.assertIsNone(mod.normalize_steps({"a": 1}))

    def test_task_label(self):
        mod = load()
        self.assertEqual(mod.task_label("p1", "T"), "p1/T")


class RealGymScanSmokeTests(unittest.TestCase):
    """devel 운동장을 읽기만 한다. pack 을 고치지 않는다.

    이 가지는 예외 경로를 열 뿐 기존 기준풀이를 연극으로 바꾸지 않는다.
    단스텝 answer(T01 등)는 skip 이고, 길이 ≥2 수집 전용 tail 은 없어야
    한다. 빈 steps 도 없어야 한다 — 있으면 원 도입 이후 정합이 깨진 것이다.
    """

    @classmethod
    def setUpClass(cls):
        cls.mod = load()
        gym_root = cls.mod.GYM_ROOT
        cls.records, cls.tool_errors = cls.mod.scan_gym(gym_root)
        cls.by_label = {}
        for rec in cls.records:
            cls.by_label.setdefault(rec.get("label"), []).append(rec)

    def test_scan_sees_real_packs(self):
        self.assertGreater(len(self.records), 10)
        self.assertEqual(self.tool_errors, [])

    def test_no_empty_steps_in_devel_refs(self):
        self.assertEqual(self.by_label.get("empty-steps", []), [])

    def test_no_collection_only_multi_in_devel_refs(self):
        self.assertEqual(self.by_label.get("collection-only-tail", []), [])

    def test_no_malformed_in_devel_refs(self):
        self.assertEqual(self.by_label.get("malformed-reference", []), [])
        self.assertEqual(self.by_label.get("malformed-json", []), [])
        self.assertEqual(self.by_label.get("malformed-task", []), [])

    def test_single_step_exists_and_is_not_exception(self):
        singles = self.by_label.get("single-step", [])
        self.assertGreater(len(singles), 0)
        # T01 은 단스텝 answer. 예외로 승격되면 안 된다.
        ids = {self.mod.task_id_of(r.get("task"), r.get("name") or "") for r in singles}
        self.assertIn("T01", ids)

    def test_multi_candidates_have_meaningful_index(self):
        multis = self.by_label.get("multi", [])
        self.assertGreater(len(multis), 0)
        for rec in multis:
            steps = rec["reference"]["steps"]
            idx = self.mod.last_meaningful_step_index(steps)
            self.assertIsNotNone(idx, rec.get("name"))
            self.assertGreaterEqual(len(steps), 2)

    def test_multi_step_generator_matches_length_filter(self):
        pairs = list(self.mod.multi_step_tasks(self.mod.GYM_ROOT))
        long_refs = [
            rec for rec in self.records
            if isinstance(rec.get("reference"), dict)
            and isinstance(rec["reference"].get("steps"), list)
            and len(rec["reference"]["steps"]) >= 2
        ]
        self.assertEqual(len(pairs), len(long_refs))


class KeyringTailKeptTests(unittest.TestCase):
    def test_trailing_keyring_is_kept_like_answer(self):
        mod = load()
        captured = []
        mod.baseline.build_task = lambda _b, _p, _t, ref, _r: captured.append(ref["steps"])
        mod.runner.score_task = lambda *a, **k: {"pass": False}
        with tempfile.TemporaryDirectory() as d:
            gym = os.path.join(d, "gym")
            _plant(gym, "p1", "T", steps=[
                {"run": ["wrap"]},
                {"keyring_from": "T13"},
            ])
            r = mod.audit("bin", gym, os.path.join(d, "w"))
        self.assertEqual(r["exceptions"], [])
        self.assertEqual(captured, [[{"keyring_from": "T13"}]])
        self.assertEqual(r["results"][0]["removedStep"], "run")

    def test_two_collection_keys_on_same_step_stay_tail(self):
        mod = load()
        steps = [{"run": ["a"]}, {"answer": 1, "keyring_from": "x"}]
        self.assertEqual(mod.last_meaningful_step_index(steps), 0)
        self.assertEqual(mod.step_kind_label(steps[1]), "answer/keyring_from")
        self.assertEqual(mod.truncate_steps(steps, 0), [{"answer": 1, "keyring_from": "x"}])


class ReportValidateEdgeTests(unittest.TestCase):
    def test_missing_required_key(self):
        mod = load()
        report = mod.empty_report()
        del report["kind"]
        issues = mod.validate_report(report)
        self.assertTrue(any("kind" in item for item in issues))

    def test_wrong_kind(self):
        mod = load()
        report = mod.empty_report()
        report["kind"] = "gymAudit"
        issues = mod.validate_report(report)
        self.assertTrue(any("kind" in item for item in issues))

    def test_wrong_schema(self):
        mod = load()
        report = mod.empty_report()
        report["schemaVersion"] = "9.9"
        issues = mod.validate_report(report)
        self.assertTrue(any("schemaVersion" in item for item in issues))

    def test_theater_not_list(self):
        mod = load()
        report = mod.empty_report()
        report["theater"] = "x"
        issues = mod.validate_report(report)
        self.assertTrue(any("theater" in item for item in issues))

    def test_task_count_mismatch(self):
        mod = load()
        report = mod.empty_report()
        report["results"] = [mod.make_result_row("p", "T", True, 2, "run")]
        report["taskCount"] = 0
        report["loadBearing"] = 1
        issues = mod.validate_report(report)
        self.assertTrue(any("taskCount" in item for item in issues))

    def test_unknown_exception_kind(self):
        mod = load()
        report = mod.empty_report()
        report["exceptions"] = [{"kind": "not-real"}]
        report["exceptionCount"] = 1
        report["trusted"] = False
        issues = mod.validate_report(report)
        self.assertTrue(any("카탈로그" in item for item in issues))

    def test_trusted_lie(self):
        mod = load()
        report = mod.empty_report()
        report["exceptions"] = [mod.exception_row("empty-steps", pack="p", task="T")]
        report["exceptionCount"] = 1
        report["trusted"] = True
        issues = mod.validate_report(report)
        self.assertTrue(any("trusted" in item for item in issues))


class ExceptionFromExcTests(unittest.TestCase):
    def test_includes_error_type(self):
        mod = load()
        row = mod.exception_from_exc(FileNotFoundError("rhwp"), context="audit", pack="p", task="T")
        self.assertEqual(row["kind"], "missing-bin")
        self.assertEqual(row["error"], "FileNotFoundError")
        self.assertEqual(row["pack"], "p")
        self.assertEqual(row["task"], "T")

    def test_none_exc(self):
        mod = load()
        row = mod.exception_from_exc(None)
        self.assertEqual(row["kind"], "unexpected")
        self.assertEqual(row["error"], "NoneType")


class IterPackIdsTests(unittest.TestCase):
    def test_missing_packs_dir(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            ids, err = mod.iter_pack_ids(d)
        self.assertEqual(ids, [])
        self.assertIsNone(err)

    def test_file_is_not_a_pack(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            os.makedirs(os.path.join(d, "packs"))
            _write(os.path.join(d, "packs", "readme.txt"), "x")
            os.makedirs(os.path.join(d, "packs", "real"))
            ids, err = mod.iter_pack_ids(d)
        self.assertEqual(ids, ["real"])
        self.assertIsNone(err)


class FormatExceptionLineMoreTests(unittest.TestCase):
    def test_path_only(self):
        mod = load()
        text = mod.format_exception_line({"kind": "os-error", "path": "packs"})
        self.assertEqual(text, "os-error: packs")

    def test_head_only(self):
        mod = load()
        text = mod.format_exception_line({"kind": "empty-steps", "head": "비었다"})
        self.assertEqual(text, "empty-steps: 비었다")

    def test_empty_kind_falls_back(self):
        mod = load()
        text = mod.format_exception_line({"kind": "", "pack": "p", "task": "T"})
        self.assertTrue(text.startswith("unexpected") or "p/T" in text)


class AttachMissingBinFromRowsTests(unittest.TestCase):
    def test_flag_from_exception_rows(self):
        mod = load()
        report = mod.empty_report()
        report["exceptions"] = [mod.exception_row("missing-bin", pack="p", task="T")]
        mod.attach_report_counts(report)
        self.assertTrue(report["missingBin"])
        self.assertTrue(report["toolFailed"])
        self.assertEqual(report["exit"], 1)
        self.assertTrue(report["ok"])


class ScorePassVariantsTests(unittest.TestCase):
    def test_pass_zero_is_load_bearing(self):
        mod = load()
        self.assertTrue(mod.verdict_from_score({"pass": 0}))

    def test_pass_empty_string_is_load_bearing(self):
        mod = load()
        self.assertTrue(mod.verdict_from_score({"pass": ""}))

    def test_pass_one_is_theater(self):
        # 파이썬 진릿값. 1 은 True 와 같다. 채점 봉투는 bool 이지만
        # 예전 계약처럼 진릿값만 본다.
        mod = load()
        self.assertFalse(mod.verdict_from_score({"pass": 1}))


if __name__ == "__main__":
    unittest.main()
