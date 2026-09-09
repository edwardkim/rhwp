"""[#5273] gym 기준 풀이 조립기 계약 — 자리표·부재 산출·실패 보고.

바이너리 없이 순수 함수와 목킹된 채점만 시험한다. 조립(`build_task`)의
run 경로는 목킹하고, 자리표 치환·경로 안전·산출 목록·채점 접기는
실제 구현을 탄다.

기존 `BaselineResolveTests`(test_gym_packs.py) 의 세 계약은 유지한다:

- 한 문자열의 여러 `{sub:}` 를 모두 바꾼다.
- 생성 성공만으로 통과 처리하지 않고 같은 pack 경로를 채점한다.
- 실패한 채점은 `pack/task: 이유` 한 줄로 남긴다.

이 파일이 더하는 것:

- `{sub:}` 세 개 이상·`{input}` 혼합·닫히지 않은 자리표.
- 부모/절대/드라이브/UNC/홈 제출 경로 거부.
- `submit.files` 부재 산출을 채점 전에 보고.
- 채점 봉투가 비-dict·키 없음·검사 실패일 때 통과로 접지 않음.
- CLI 플래그는 `--agent` / `--pack` / `--bin` 만.
"""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "build_baseline.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_build_baseline_hardening", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _task(task_id="T01", submit_files=None, input_path="samples/x.hwp"):
    body = {
        "id": task_id,
        "tier": 1,
        "title": "t",
        "input": input_path,
        "submit": {"kind": "artifact", "files": submit_files or []},
        "checks": [],
    }
    if submit_files is None:
        body["submit"] = {"kind": "answer"}
    return body


def _write(path, payload):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        if isinstance(payload, str):
            fh.write(payload)
        else:
            json.dump(payload, fh, ensure_ascii=False)


class TokenClassifyTests(unittest.TestCase):
    def test_catalog_has_eight_kinds(self):
        mod = load()
        self.assertEqual(len(mod.TOKEN_KINDS), 8)
        self.assertIn("exact-sub", mod.TOKEN_KINDS)
        self.assertIn("embedded-sub", mod.TOKEN_KINDS)
        self.assertIn("mixed", mod.TOKEN_KINDS)
        self.assertIn("unclosed-sub", mod.TOKEN_KINDS)

    def test_exact_input(self):
        mod = load()
        self.assertEqual(mod.classify_token("{input}"), "exact-input")
        self.assertTrue(mod.is_exact_input("{input}"))

    def test_exact_sub(self):
        mod = load()
        self.assertEqual(mod.classify_token("{sub:edited.hwp}"), "exact-sub")
        self.assertTrue(mod.is_exact_sub("{sub:edited.hwp}"))

    def test_embedded_two_subs(self):
        mod = load()
        token = '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'
        self.assertEqual(mod.classify_token(token), "embedded-sub")

    def test_embedded_three_subs(self):
        mod = load()
        token = "{sub:a} then {sub:b} and {sub:c}"
        self.assertEqual(mod.classify_token(token), "embedded-sub")
        self.assertEqual(mod.count_sub_placeholders(token), 3)

    def test_mixed_input_and_sub(self):
        mod = load()
        token = '{"in": "{input}", "out": "{sub:o.hwp}"}'
        self.assertEqual(mod.classify_token(token), "mixed")

    def test_embedded_input_only(self):
        mod = load()
        self.assertEqual(mod.classify_token("src={input}"), "embedded-input")

    def test_literal(self):
        mod = load()
        self.assertEqual(mod.classify_token("--json"), "literal")
        self.assertEqual(mod.classify_token("samples/x.hwp"), "literal")

    def test_unclosed_sub(self):
        mod = load()
        self.assertEqual(mod.classify_token("{sub:o.hwp"), "unclosed-sub")
        self.assertTrue(mod.has_unclosed_sub("{sub:o.hwp"))
        self.assertFalse(mod.has_unclosed_sub("{sub:o.hwp}"))

    def test_not_str(self):
        mod = load()
        self.assertEqual(mod.classify_token(None), "not-str")
        self.assertEqual(mod.classify_token(3), "not-str")

    def test_placeholder_builders(self):
        mod = load()
        self.assertEqual(mod.placeholder_input(), "{input}")
        self.assertEqual(mod.placeholder_sub("o.hwp"), "{sub:o.hwp}")


class SubExtractTests(unittest.TestCase):
    def test_extract_keeps_order_and_duplicates(self):
        mod = load()
        token = "{sub:a.hwp}-{sub:b.hwp}-{sub:a.hwp}"
        self.assertEqual(mod.extract_sub_names(token), ["a.hwp", "b.hwp", "a.hwp"])
        self.assertEqual(mod.unique_sub_names(token), ["a.hwp", "b.hwp"])

    def test_extract_empty(self):
        mod = load()
        self.assertEqual(mod.extract_sub_names("nope"), [])
        self.assertEqual(mod.extract_sub_names(None), [])

    def test_extract_placeholders_mixed(self):
        mod = load()
        found = mod.extract_placeholders("{input} -> {sub:o.hwp}")
        kinds = [row["kind"] for row in found]
        self.assertEqual(kinds, ["input", "sub"])
        self.assertEqual(found[1]["name"], "o.hwp")

    def test_extract_placeholders_unclosed(self):
        mod = load()
        found = mod.extract_placeholders("x {sub:oops")
        self.assertEqual(found[-1]["kind"], "unclosed-sub")

    def test_counts(self):
        mod = load()
        token = '{"a": "{input}", "b": "{sub:1}", "c": "{sub:2}"}'
        self.assertEqual(mod.count_input_placeholders(token), 1)
        self.assertEqual(mod.count_sub_placeholders(token), 2)

    def test_remaining_placeholders(self):
        mod = load()
        self.assertEqual(mod.remaining_placeholders("{sub:x}"), ["{sub:"])
        self.assertEqual(mod.remaining_placeholders("{input}"), ["{input}"])
        self.assertEqual(mod.remaining_placeholders("done"), [])
        self.assertTrue(mod.has_unresolved_placeholder("{sub:x}"))
        self.assertFalse(mod.has_unresolved_placeholder("done"))


class PathSafetyTests(unittest.TestCase):
    def test_safe_nested_rel(self):
        mod = load()
        self.assertEqual(mod.normalize_rel("capsules/work.json"), "capsules/work.json")
        self.assertIsNone(mod.unsafe_rel_reason("capsules/work.json"))
        self.assertTrue(mod.is_safe_sub_name("capsules/work.json"))

    def test_dots_and_slashes_collapse(self):
        mod = load()
        self.assertEqual(mod.normalize_rel("./edited.hwp"), "edited.hwp")
        self.assertEqual(mod.normalize_rel("a//b"), "a/b")

    def test_parent_rejected(self):
        mod = load()
        self.assertEqual(mod.unsafe_rel_reason("../escape.hwp"), "parent")
        self.assertIsNone(mod.normalize_rel("../escape.hwp"))
        self.assertFalse(mod.is_safe_rel("../escape.hwp"))

    def test_absolute_rejected(self):
        mod = load()
        self.assertEqual(mod.unsafe_rel_reason("/tmp/x"), "absolute")

    def test_drive_rejected(self):
        mod = load()
        self.assertEqual(mod.unsafe_rel_reason("C:/tmp/x"), "drive")

    def test_unc_rejected(self):
        mod = load()
        self.assertEqual(mod.unsafe_rel_reason("//server/share"), "unc")

    def test_home_rejected(self):
        mod = load()
        self.assertEqual(mod.unsafe_rel_reason("~/secret"), "home")

    def test_empty_and_not_str(self):
        mod = load()
        self.assertEqual(mod.unsafe_rel_reason(""), "empty")
        self.assertEqual(mod.unsafe_rel_reason("   "), "empty")
        self.assertEqual(mod.unsafe_rel_reason(None), "not-str")
        self.assertIn("empty", mod.UNSAFE_REL_REASONS)
        self.assertIn("parent", mod.UNSAFE_REL_REASONS)

    def test_join_sub_path_mkdir(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = mod.join_sub_path(tmp, "capsules/work.json", mkdir=True)
            self.assertTrue(os.path.isdir(os.path.dirname(path)))
            self.assertTrue(path.endswith(os.path.join("capsules", "work.json"))
                            or path.replace("\\", "/").endswith("capsules/work.json"))

    def test_join_rejects_parent(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(RuntimeError) as ctx:
                mod.join_sub_path(tmp, "../escape.hwp")
            self.assertIn("parent", str(ctx.exception))

    def test_require_safe_sub_name(self):
        mod = load()
        self.assertEqual(mod.require_safe_sub_name("o.hwp"), "o.hwp")
        with self.assertRaises(RuntimeError):
            mod.require_safe_sub_name("..")

    def test_escape_json_path_doubles_backslashes(self):
        mod = load()
        self.assertEqual(mod.escape_json_path("a\\b"), "a\\\\b")
        self.assertEqual(mod.escape_json_path("a/b"), "a/b")


class ResolveTests(unittest.TestCase):
    def test_multiple_sub_placeholders_all_resolve(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'
            out = mod.resolve(token, {"input": "in.hwp"}, sub_dir)
            self.assertNotIn("{sub:", out, "치환되지 않은 자리표가 남았다")
            self.assertIn("o1.hwp", out)
            self.assertIn("o2.hwp", out)

    def test_three_sub_placeholders_in_plan_json(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"a": "{sub:a.hwp}", "b": "{sub:b.hwp}", "c": "{sub:c.hwp}"}'
            out = mod.resolve(token, {"input": "in.hwp"}, sub_dir)
            self.assertEqual(mod.count_sub_placeholders(out), 0)
            self.assertIn("a.hwp", out)
            self.assertIn("b.hwp", out)
            self.assertIn("c.hwp", out)

    def test_exact_input_keeps_original_separators(self):
        mod = load()
        task = {"input": "samples\\nested\\x.hwp"}
        self.assertEqual(mod.resolve("{input}", task, "/tmp"), "samples\\nested\\x.hwp")

    def test_exact_sub_makes_parent_and_does_not_escape(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            out = mod.resolve("{sub:capsules/work.json}", {"input": "in.hwp"}, sub_dir)
            self.assertTrue(os.path.isdir(os.path.join(sub_dir, "capsules")))
            self.assertTrue(out.endswith("work.json"))
            self.assertNotIn("{sub:", out)

    def test_mixed_resolves_input_and_all_subs(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"in": "{input}", "o1": "{sub:o1.hwp}", "o2": "{sub:o2.hwp}"}'
            out = mod.resolve(token, {"input": "samples/x.hwp"}, sub_dir)
            self.assertNotIn("{input}", out)
            self.assertNotIn("{sub:", out)
            self.assertIn("samples/x.hwp", out)
            self.assertIn("o1.hwp", out)
            self.assertIn("o2.hwp", out)

    def test_embedded_input_only(self):
        mod = load()
        out = mod.resolve("src={input}", {"input": "samples\\a.hwp"}, "/tmp")
        self.assertEqual(out, "src=samples/a.hwp")

    def test_literal_unchanged(self):
        mod = load()
        self.assertEqual(mod.resolve("--json", {"input": "in.hwp"}, "/tmp"), "--json")

    def test_reference_file_placeholder_is_rejected(self):
        mod = load()
        with self.assertRaises(RuntimeError) as ctx:
            mod.resolve("{file:out.hwp}", {"input": "in.hwp"}, "/tmp")
        self.assertIn("{sub:}", str(ctx.exception))

    def test_unclosed_raises_runtime_error(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            with self.assertRaises(RuntimeError) as ctx:
                mod.resolve("{sub:o.hwp", {"input": "in.hwp"}, sub_dir)
            self.assertIn("닫히지 않은", str(ctx.exception))

    def test_unsafe_embedded_sub_raises(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            with self.assertRaises(RuntimeError) as ctx:
                mod.resolve('{"out": "{sub:../x.hwp}"}', {"input": "in.hwp"}, sub_dir)
            self.assertIn("parent", str(ctx.exception))

    def test_resolve_args_maps_each(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            args = ["edit", "{input}", "-o", "{sub:o.hwp}"]
            out = mod.resolve_args(args, {"input": "in.hwp"}, sub_dir)
            self.assertEqual(out[1], "in.hwp")
            self.assertTrue(out[3].endswith("o.hwp"))

    def test_resolve_args_rejects_non_list(self):
        mod = load()
        with self.assertRaises(RuntimeError):
            mod.resolve_args("edit", {"input": "in.hwp"}, "/tmp")

    def test_non_str_token_passthrough(self):
        mod = load()
        self.assertIsNone(mod.resolve(None, {"input": "in.hwp"}, "/tmp"))
        self.assertEqual(mod.resolve(1, {"input": "in.hwp"}, "/tmp"), 1)


class WriteJsonBodyTests(unittest.TestCase):
    def test_replaces_input_and_sub_in_body(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            body = {"planVersion": "1.0", "input": "{input}", "output": "{sub:o.hwp}"}
            out = mod.resolve_write_json_body(body, {"input": "samples\\a.hwp"}, sub_dir)
            self.assertEqual(out["input"], "samples/a.hwp")
            self.assertNotIn("{sub:", out["output"])
            self.assertTrue(out["output"].endswith("o.hwp") or "o.hwp" in out["output"])

    def test_body_without_placeholders_roundtrips(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            body = {"planVersion": "1.0", "steps": [{"action": "set_cell"}]}
            out = mod.resolve_write_json_body(body, {"input": "in.hwp"}, sub_dir)
            self.assertEqual(out["planVersion"], "1.0")
            self.assertEqual(out["steps"][0]["action"], "set_cell")


class StepClassifyTests(unittest.TestCase):
    def test_known_kinds_tuple(self):
        mod = load()
        self.assertEqual(mod.STEP_KINDS, ("run", "copy", "write_json", "keyring_from", "answer"))
        self.assertEqual(set(mod.STEP_KINDS), set(mod.KNOWN_STEP_KINDS))

    def test_step_kind_prefers_declared_order(self):
        mod = load()
        self.assertEqual(mod.step_kind({"run": ["a"]}), "run")
        self.assertEqual(mod.step_kind({"copy": {}}), "copy")
        self.assertEqual(mod.step_kind({"write_json": {}}), "write_json")
        self.assertEqual(mod.step_kind({"keyring_from": {}}), "keyring_from")
        self.assertEqual(mod.step_kind({"answer": {}}), "answer")
        self.assertIsNone(mod.step_kind({"allowExits": [0]}))
        self.assertIsNone(mod.step_kind("run"))

    def test_classify_step_unknown_and_not_mapping(self):
        mod = load()
        self.assertEqual(mod.classify_step({"nope": 1}), "unknown")
        self.assertEqual(mod.classify_step(None), "not-mapping")

    def test_classify_reference(self):
        mod = load()
        self.assertEqual(mod.classify_reference({"steps": [{"run": ["a"]}]}), "ok")
        self.assertEqual(mod.classify_reference({"steps": []}), "empty-steps")
        self.assertEqual(mod.classify_reference({"steps": None}), "malformed-reference")
        self.assertEqual(mod.classify_reference("x"), "malformed-reference")
        self.assertEqual(mod.classify_reference({"steps": [{"nope": 1}]}), "unknown-step")

    def test_validate_step_and_reference(self):
        mod = load()
        self.assertEqual(mod.validate_step({"run": ["edit"]}), [])
        self.assertTrue(mod.validate_step({"run": "edit"}))
        self.assertTrue(mod.validate_step({"copy": {}}))
        self.assertTrue(any("from" in e for e in mod.validate_step({"copy": {}})))
        errors = mod.validate_reference({"steps": [{"copy": {"from": "a"}}]})
        self.assertTrue(any("to" in e for e in errors))
        self.assertEqual(mod.validate_reference("nope"), ["기준 풀이가 객체가 아니다"])
        self.assertEqual(mod.validate_reference({"steps": []}), ["steps 가 비었다"])

    def test_collect_sub_names_across_steps(self):
        mod = load()
        reference = {
            "id": "T",
            "steps": [
                {"run": ["edit", "{input}", "-o", "{sub:a.hwp}"]},
                {"write_json": {"to": "{sub:plan.json}", "body": {"out": "{sub:b.hwp}"}}},
                {"keyring_from": {"key": "{sub:key.json}", "out": "{sub:keyring.json}", "keyId": "k"}},
                {"answer": {"n": {"cmd": ["search", "{sub:a.hwp}"]}}},
            ],
        }
        names = mod.collect_sub_names(reference)
        self.assertEqual(names, ["a.hwp", "plan.json", "b.hwp", "key.json", "keyring.json"])


class ArtifactTests(unittest.TestCase):
    def test_submit_files_keeps_order_and_drops_unsafe(self):
        mod = load()
        task = _task("T", submit_files=["o.hwp", "../x", "o.hwp", "capsules/a.json"])
        self.assertEqual(mod.submit_files(task), ["o.hwp", "capsules/a.json"])

    def test_submit_files_empty_when_answer_task(self):
        mod = load()
        self.assertEqual(mod.submit_files(_task("T")), [])
        self.assertEqual(mod.submit_kind(_task("T")), "answer")

    def test_declared_artifacts_uses_submit_files_only(self):
        mod = load()
        task = _task("T", submit_files=["edited.hwp"])
        reference = {"steps": [{"run": ["-o", "{sub:tmp.hwp}"]}]}
        self.assertEqual(mod.declared_artifacts(task, reference), ["edited.hwp"])
        self.assertEqual(mod.declared_artifacts(_task("T"), reference), [])

    def test_missing_artifacts_reports_absent_file(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            open(os.path.join(tmp, "kept.hwp"), "w", encoding="utf-8").write("x")
            missing = mod.missing_artifacts(tmp, ["kept.hwp", "gone.hwp", "capsules/a.json"])
            self.assertEqual(missing, ["gone.hwp", "capsules/a.json"])

    def test_missing_artifacts_empty_when_all_present(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "capsules"))
            open(os.path.join(tmp, "edited.hwp"), "w", encoding="utf-8").write("x")
            open(os.path.join(tmp, "capsules", "work.json"), "w", encoding="utf-8").write("{}")
            self.assertEqual(mod.missing_artifacts(tmp, ["edited.hwp", "capsules/work.json"]), [])

    def test_artifact_status(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            open(os.path.join(tmp, "a"), "w", encoding="utf-8").write("1")
            status = mod.artifact_status(tmp, ["a", "b"])
            self.assertFalse(status["ok"])
            self.assertEqual(status["present"], ["a"])
            self.assertEqual(status["missing"], ["b"])

    def test_list_submission_files_sorted(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "n"))
            open(os.path.join(tmp, "z.txt"), "w", encoding="utf-8").write("z")
            open(os.path.join(tmp, "n", "a.txt"), "w", encoding="utf-8").write("a")
            self.assertEqual(mod.list_submission_files(tmp), ["n/a.txt", "z.txt"])
            self.assertEqual(mod.list_submission_files(os.path.join(tmp, "nope")), [])

    def test_format_missing_artifact(self):
        mod = load()
        msg = mod.format_missing_artifact("pack-b", "T02", ["edited.hwp", "plan.json"])
        self.assertEqual(msg, "pack-b/T02: 부재 산출: edited.hwp, plan.json")

    def test_missing_artifact_message_none_when_no_files_declared(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            self.assertIsNone(mod.missing_artifact_message("p", _task("T"), tmp))

    def test_missing_artifact_message_when_declared_absent(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            task = _task("T02", submit_files=["edited.hwp"])
            msg = mod.missing_artifact_message("pack-b", task, tmp)
            self.assertEqual(msg, "pack-b/T02: 부재 산출: edited.hwp")

    def test_missing_artifact_message_none_when_present(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            open(os.path.join(tmp, "edited.hwp"), "w", encoding="utf-8").write("x")
            task = _task("T02", submit_files=["edited.hwp"])
            self.assertIsNone(mod.missing_artifact_message("pack-b", task, tmp))


class ScoreFoldTests(unittest.TestCase):
    def test_pass_true_is_ok(self):
        mod = load()
        self.assertTrue(mod.score_is_pass({"pass": True}))
        folded = mod.fold_score_result({"pass": True})
        self.assertTrue(folded["ok"])
        self.assertEqual(folded["kind"], "ok")

    def test_error_field_is_failed_score(self):
        mod = load()
        folded = mod.fold_score_result({"pass": False, "error": "제출 폴더 없음"})
        self.assertFalse(folded["ok"])
        self.assertEqual(folded["kind"], "failed-score")
        self.assertEqual(folded["reason"], "제출 폴더 없음")

    def test_failed_checks_join_names(self):
        mod = load()
        result = {
            "pass": False,
            "checks": [
                {"name": "1단계 반영", "ok": False, "error": "없음"},
                {"name": "2단계 반영", "ok": True},
                {"op": "file_exists", "ok": False},
            ],
        }
        folded = mod.fold_score_result(result)
        self.assertEqual(folded["reason"], "1단계 반영: 없음; file_exists: 판정 불일치")
        self.assertEqual(len(folded["failedChecks"]), 2)

    def test_missing_pass_key_is_failure(self):
        mod = load()
        self.assertFalse(mod.score_is_pass({}))
        folded = mod.fold_score_result({})
        self.assertFalse(folded["ok"])
        self.assertEqual(folded["reason"], "채점 실패")

    def test_none_and_non_dict(self):
        mod = load()
        self.assertFalse(mod.score_is_pass(None))
        self.assertEqual(mod.fold_score_result(None)["reason"], "채점 결과가 dict 가 아니다")
        self.assertEqual(mod.normalize_score("x")["error"], "채점 결과가 dict 가 아니다")

    def test_failed_check_lines_skips_non_dict(self):
        mod = load()
        lines = mod.failed_check_lines([None, "x", {"ok": False, "name": "c"}])
        self.assertEqual(lines, ["c: 판정 불일치"])

    def test_score_failure_message_preserves_legacy_error_line(self):
        mod = load()
        msg = mod.score_failure_message("pack-b", "T02", {"pass": False, "error": "제출 폴더 없음"})
        self.assertEqual(msg, "pack-b/T02: 제출 폴더 없음")

    def test_score_failure_message_none_on_pass(self):
        mod = load()
        self.assertIsNone(mod.score_failure_message("p", "T", {"pass": True}))

    def test_format_task_failure(self):
        mod = load()
        self.assertEqual(mod.format_task_failure("core-cli", "T09", "채점 실패"),
                         "core-cli/T09: 채점 실패")


class VerifyBuiltTaskTests(unittest.TestCase):
    def test_scores_in_pack_directory(self):
        mod = load()
        task = {"id": "T01"}
        with mock.patch.object(mod.runner, "score_task", return_value={"pass": True}) as score:
            failure = mod.verify_built_task("/tmp/rhwp", "pack-a", task, "/tmp/sub")
        self.assertIsNone(failure)
        score.assert_called_once_with(task, os.path.join("/tmp/sub", "pack-a"), "/tmp/rhwp")

    def test_failed_built_submission_reports_the_task(self):
        mod = load()
        task = {"id": "T02"}
        with mock.patch.object(mod.runner, "score_task",
                               return_value={"pass": False, "error": "제출 폴더 없음"}):
            failure = mod.verify_built_task("/tmp/rhwp", "pack-b", task, "/tmp/sub")
        self.assertEqual(failure, "pack-b/T02: 제출 폴더 없음")

    def test_failed_checks_are_joined(self):
        mod = load()
        task = {"id": "T09"}
        result = {
            "pass": False,
            "checks": [
                {"name": "1단계 반영", "ok": False, "error": "0"},
                {"name": "2단계 반영", "ok": False, "error": "없음"},
            ],
        }
        with mock.patch.object(mod.runner, "score_task", return_value=result):
            failure = mod.verify_built_task("/tmp/rhwp", "core-cli", task, "/tmp/sub")
        self.assertEqual(failure, "core-cli/T09: 1단계 반영: 0; 2단계 반영: 없음")

    def test_non_dict_score_is_not_a_pass(self):
        mod = load()
        with mock.patch.object(mod.runner, "score_task", return_value=None):
            failure = mod.verify_built_task("/tmp/rhwp", "p", {"id": "T"}, "/tmp/sub")
        self.assertEqual(failure, "p/T: 채점 결과가 dict 가 아니다")


class InspectBuiltTaskTests(unittest.TestCase):
    def test_missing_artifact_short_circuits_score(self):
        mod = load()
        task = _task("T02", submit_files=["edited.hwp"])
        with tempfile.TemporaryDirectory() as root:
            sub_dir = os.path.join(root, "pack-b", "T02")
            os.makedirs(sub_dir)
            with mock.patch.object(mod.runner, "score_task") as score:
                inspected = mod.inspect_built_task("/tmp/rhwp", "pack-b", task, root)
            score.assert_not_called()
            self.assertEqual(inspected["kind"], "missing-artifact")
            self.assertEqual(inspected["missing"], ["edited.hwp"])
            self.assertIn("부재 산출", inspected["message"])

    def test_present_artifact_then_failed_score(self):
        mod = load()
        task = _task("T02", submit_files=["edited.hwp"])
        with tempfile.TemporaryDirectory() as root:
            sub_dir = os.path.join(root, "pack-b", "T02")
            os.makedirs(sub_dir)
            open(os.path.join(sub_dir, "edited.hwp"), "w", encoding="utf-8").write("x")
            with mock.patch.object(mod.runner, "score_task",
                                   return_value={"pass": False, "error": "판정 불일치"}):
                inspected = mod.inspect_built_task("/tmp/rhwp", "pack-b", task, root)
        self.assertEqual(inspected["kind"], "failed-score")
        self.assertEqual(inspected["message"], "pack-b/T02: 판정 불일치")

    def test_ok_when_artifact_present_and_score_passes(self):
        mod = load()
        task = _task("T02", submit_files=["edited.hwp"])
        with tempfile.TemporaryDirectory() as root:
            sub_dir = os.path.join(root, "pack-b", "T02")
            os.makedirs(sub_dir)
            open(os.path.join(sub_dir, "edited.hwp"), "w", encoding="utf-8").write("x")
            with mock.patch.object(mod.runner, "score_task", return_value={"pass": True}):
                inspected = mod.inspect_built_task("/tmp/rhwp", "pack-b", task, root)
        self.assertTrue(inspected["ok"])
        self.assertEqual(inspected["kind"], "ok")


class BuildTaskPureStepsTests(unittest.TestCase):
    def test_write_json_and_copy_without_bin(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            src = os.path.join(tmp, "src.txt")
            open(src, "w", encoding="utf-8").write("hello")
            # copy.from 가 절대 경로면 ROOT 를 앞에 붙이지 않는다.
            task = _task("T", submit_files=["out.txt", "plan.json"], input_path=src)
            reference = {
                "id": "T",
                "steps": [
                    {"copy": {"from": src, "to": "{sub:out.txt}"}},
                    {"write_json": {"to": "{sub:plan.json}",
                                    "body": {"input": "{input}", "note": "ok"}}},
                ],
            }
            sub_root = os.path.join(tmp, "sub")
            built = mod.build_task("/bin/false", "p", task, reference, sub_root)
            self.assertTrue(os.path.isfile(os.path.join(built, "out.txt")))
            plan = json.loads(Path(built, "plan.json").read_text(encoding="utf-8"))
            self.assertIn("src.txt", plan["input"].replace("\\", "/"))
            self.assertEqual(plan["note"], "ok")

    def test_unknown_step_raises(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(RuntimeError) as ctx:
                mod.build_task("/bin/false", "p", _task("T"),
                               {"steps": [{"nope": 1}]}, tmp)
            self.assertIn("알 수 없는", str(ctx.exception))

    def test_missing_steps_raises(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(RuntimeError) as ctx:
                mod.build_task("/bin/false", "p", _task("T"), {"id": "T"}, tmp)
            self.assertIn("steps", str(ctx.exception))

    def test_answer_const_writes_answer_json(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            reference = {"steps": [{"answer": {"n": {"const": 3}}}]}
            built = mod.build_task("/bin/false", "p", _task("T"), reference, tmp)
            answer = json.loads(Path(built, "answer.json").read_text(encoding="utf-8"))
            self.assertEqual(answer, {"n": 3})

    def test_keyring_from_reads_public_key(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            task = _task("T", submit_files=["keyring.json"])
            # 키 파일을 제출 폴더에 미리 둘 수 없으므로 copy 로 심는다.
            key_src = os.path.join(tmp, "key.json")
            _write(key_src, {"publicKey": "PUB"})
            reference = {
                "steps": [
                    {"copy": {"from": key_src, "to": "{sub:key.json}"}},
                    {"keyring_from": {"key": "{sub:key.json}", "out": "{sub:keyring.json}",
                                      "keyId": "gym/t"}},
                ],
            }
            built = mod.build_task("/bin/false", "p", task, reference, tmp)
            keyring = json.loads(Path(built, "keyring.json").read_text(encoding="utf-8"))
            self.assertEqual(keyring["keys"][0]["publicKey"], "PUB")
            self.assertEqual(keyring["keys"][0]["keyId"], "gym/t")


class ProcessOneTaskTests(unittest.TestCase):
    def test_counts_missing_artifact(self):
        mod = load()
        counts = mod.empty_counts()
        task = _task("T02", submit_files=["edited.hwp"])
        reference = {"steps": [{"answer": {"n": {"const": 1}}}]}
        with tempfile.TemporaryDirectory() as tmp:
            inspected = mod.process_one_task("/bin/false", "pack-b", task, reference, tmp, counts)
        self.assertEqual(inspected["kind"], "missing-artifact")
        self.assertEqual(counts["failed"], 1)
        self.assertEqual(counts["missingArtifact"], 1)
        self.assertEqual(counts["built"], 0)

    def test_counts_failed_score(self):
        mod = load()
        counts = mod.empty_counts()
        task = _task("T02", submit_files=["edited.hwp"])
        with tempfile.TemporaryDirectory() as tmp:
            src = os.path.join(tmp, "blank")
            open(src, "w", encoding="utf-8").write("x")
            reference = {"steps": [{"copy": {"from": src, "to": "{sub:edited.hwp}"}}]}
            with mock.patch.object(mod.runner, "score_task",
                                   return_value={"pass": False, "error": "판정 불일치"}):
                inspected = mod.process_one_task("/bin/false", "pack-b", task, reference, tmp, counts)
        self.assertEqual(inspected["kind"], "failed-score")
        self.assertEqual(counts["failedScore"], 1)
        self.assertEqual(counts["failed"], 1)

    def test_counts_build_error_on_unsafe_sub(self):
        mod = load()
        counts = mod.empty_counts()
        task = _task("T")
        reference = {"steps": [{"run": ["edit", "-o", "{sub:../x.hwp}"]}]}
        with tempfile.TemporaryDirectory() as tmp:
            inspected = mod.process_one_task("/bin/false", "p", task, reference, tmp, counts)
        self.assertEqual(inspected["kind"], "build-error")
        self.assertEqual(counts["buildError"], 1)
        self.assertEqual(counts["failed"], 1)

    def test_counts_built_on_success(self):
        mod = load()
        counts = mod.empty_counts()
        task = _task("T")
        reference = {"steps": [{"answer": {"n": {"const": 0}}}]}
        with tempfile.TemporaryDirectory() as tmp:
            with mock.patch.object(mod.runner, "score_task", return_value={"pass": True}):
                inspected = mod.process_one_task("/bin/false", "p", task, reference, tmp, counts)
        self.assertTrue(inspected["ok"])
        self.assertEqual(counts["built"], 1)
        self.assertEqual(counts["failed"], 0)


class SummaryTests(unittest.TestCase):
    def test_empty_counts_keys(self):
        mod = load()
        counts = mod.empty_counts()
        for key in mod.COUNT_KEYS:
            self.assertIn(key, counts)
            self.assertEqual(counts[key], 0)

    def test_format_summary_legacy_line(self):
        mod = load()
        counts = mod.empty_counts()
        counts["built"] = 12
        counts["failed"] = 3
        counts["skipped"] = 1
        self.assertEqual(mod.format_summary(counts),
                         "기준 풀이 왕복: 성공 12 · 실패 3 · 기준 풀이 없음 1")

    def test_format_summary_detail(self):
        mod = load()
        counts = {"missingArtifact": 2, "failedScore": 1, "buildError": 0}
        self.assertEqual(mod.format_summary_detail(counts),
                         "  내역: 부재 산출 2 · 채점 실패 1 · 조립 오류 0")

    def test_summary_exit(self):
        mod = load()
        self.assertEqual(mod.summary_exit({"failed": 0}), 0)
        self.assertEqual(mod.summary_exit({"failed": 1}), 1)
        self.assertEqual(mod.summary_exit(None), 1)

    def test_bump_count(self):
        mod = load()
        counts = mod.empty_counts()
        mod.bump_count(counts, "failed", 2)
        self.assertEqual(counts["failed"], 2)
        created = mod.bump_count(None, "built")
        self.assertEqual(created["built"], 1)

    def test_release_verification_report_requires_every_task_to_pass(self):
        mod = load()
        counts = mod.empty_counts()
        counts["built"] = 2
        report = mod.verification_report(
            "/tmp/rhwp",
            "agent",
            ["p"],
            counts,
            [
                {"pack": "p", "task": "T1", "ok": True},
                {"pack": "p", "task": "T2", "ok": True},
            ],
        )
        self.assertTrue(report["ok"])
        self.assertEqual(report["exit"], 0)
        self.assertEqual(report["taskCount"], 2)

        counts["skipped"] = 1
        report = mod.verification_report(
            "/tmp/rhwp", "agent", ["p"], counts, report["results"]
        )
        self.assertFalse(report["ok"])
        self.assertEqual(report["exit"], 1)


class CliContractTests(unittest.TestCase):
    def test_structured_release_audit_flag_is_explicit(self):
        mod = load()
        self.assertEqual(mod.cli_flag_names(), ("--agent", "--pack", "--bin", "--json"))
        self.assertEqual(mod.CLI_FLAGS, ("--agent", "--pack", "--bin", "--json"))

    def test_parse_args_defaults(self):
        mod = load()
        a = mod.parse_args([])
        self.assertEqual(a.agent, "claude-fable-5")
        self.assertIsNone(a.pack)
        self.assertIsNone(a.bin)
        self.assertFalse(a.json)

    def test_parse_args_pack_appends(self):
        mod = load()
        a = mod.parse_args(["--pack", "core-cli", "--pack", "text-editing", "--bin", "rhwp"])
        self.assertEqual(a.pack, ["core-cli", "text-editing"])
        self.assertEqual(a.bin, "rhwp")

    def test_failure_kinds_catalog(self):
        mod = load()
        self.assertIn("missing-artifact", mod.FAILURE_KINDS)
        self.assertIn("failed-score", mod.FAILURE_KINDS)
        self.assertIn("unclosed-placeholder", mod.FAILURE_KINDS)

    def test_catchable_includes_value_error(self):
        mod = load()
        self.assertIn(ValueError, mod.CATCHABLE_EXCEPTIONS)
        self.assertFalse(mod.is_fatal_exception(RuntimeError("x")))
        self.assertTrue(mod.is_fatal_exception(KeyboardInterrupt()))


class CopySourceTests(unittest.TestCase):
    def test_relative_joins_root(self):
        mod = load()
        src = mod.resolve_copy_source("samples/x.hwp", {"input": "samples/x.hwp"}, "/tmp")
        self.assertEqual(src, os.path.join(mod.ROOT, "samples/x.hwp"))

    def test_input_placeholder_joins_root(self):
        mod = load()
        src = mod.resolve_copy_source("{input}", {"input": "samples/x.hwp"}, "/tmp")
        self.assertEqual(src, os.path.join(mod.ROOT, "samples/x.hwp"))


class AllowExitsTests(unittest.TestCase):
    def test_default_zero(self):
        mod = load()
        self.assertEqual(mod.allow_exits_of({"run": ["a"]}), [0])
        self.assertEqual(mod.allow_exits_of(None), [0])

    def test_declared_list(self):
        mod = load()
        self.assertEqual(mod.allow_exits_of({"run": ["a"], "allowExits": [0, 3]}), [0, 3])


class WalkStringsTests(unittest.TestCase):
    def test_nested(self):
        mod = load()
        values = list(mod.walk_strings({"a": ["{sub:x}", {"b": "{input}"}]}))
        self.assertEqual(values, ["{sub:x}", "{input}"])


class ReferencePathTests(unittest.TestCase):
    def test_reference_path_uses_packs_dir(self):
        mod = load()
        path = mod.reference_path("core-cli", "T09")
        self.assertTrue(path.endswith(os.path.join("core-cli", "reference", "T09.json"))
                        or path.replace("\\", "/").endswith("core-cli/reference/T09.json"))

    def test_submission_dir(self):
        mod = load()
        self.assertEqual(
            mod.submission_dir("/tmp/sub", "core-cli", {"id": "T09"}),
            os.path.join("/tmp/sub", "core-cli", "T09"),
        )


class TaskHelpersTests(unittest.TestCase):
    def test_task_id_and_input(self):
        mod = load()
        self.assertEqual(mod.task_id_of({"id": "T09"}), "T09")
        self.assertEqual(mod.task_id_of({}), "?")
        self.assertEqual(mod.task_id_of(None), "?")
        self.assertEqual(mod.task_input({"input": "a.hwp"}), "a.hwp")
        with self.assertRaises(RuntimeError):
            mod.task_input({})
        with self.assertRaises(RuntimeError):
            mod.task_input(None)


class WriteHelpersTests(unittest.TestCase):
    def test_write_answer_file_skips_empty(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            self.assertIsNone(mod.write_answer_file(tmp, {}))
            self.assertFalse(os.path.exists(os.path.join(tmp, "answer.json")))
            path = mod.write_answer_file(tmp, {"n": 1})
            self.assertTrue(os.path.isfile(path))


class MultiPlaceholderPlanTests(unittest.TestCase):
    """다세대 계획서 JSON 이 한 토큰에 넣는 자리표 조합."""

    def test_four_subs_and_input_leave_no_placeholder(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            token = (
                '{"in":"{input}","g1":"{sub:g1.hwp}",'
                '"g2":"{sub:g2.hwp}","g3":"{sub:g3.hwp}","g4":"{sub:g4.hwp}"}'
            )
            out = mod.resolve(token, {"input": "samples/a.hwp"}, sub_dir)
            self.assertFalse(mod.has_unresolved_placeholder(out))
            for name in ("g1.hwp", "g2.hwp", "g3.hwp", "g4.hwp"):
                self.assertIn(name, out)
            self.assertIn("samples/a.hwp", out)

    def test_nested_capsule_and_output_together(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"capsule":"{sub:capsules/work.json}","out":"{sub:o.hwp}"}'
            out = mod.resolve(token, {"input": "in.hwp"}, sub_dir)
            self.assertFalse(mod.has_unresolved_placeholder(out))
            self.assertTrue(os.path.isdir(os.path.join(sub_dir, "capsules")))
            self.assertIn("work.json", out)
            self.assertIn("o.hwp", out)

    def test_same_sub_twice_resolves_both(self):
        mod = load()
        with tempfile.TemporaryDirectory() as sub_dir:
            token = "{sub:same.hwp} and again {sub:same.hwp}"
            out = mod.resolve(token, {"input": "in.hwp"}, sub_dir)
            self.assertEqual(mod.count_sub_placeholders(out), 0)
            self.assertEqual(out.count("same.hwp"), 2)


class MissingArtifactEdgeTests(unittest.TestCase):
    def test_directory_instead_of_file_is_missing(self):
        mod = load()
        with tempfile.TemporaryDirectory() as tmp:
            os.makedirs(os.path.join(tmp, "edited.hwp"))
            missing = mod.missing_artifacts(tmp, ["edited.hwp"])
            self.assertEqual(missing, ["edited.hwp"])

    def test_duplicate_expected_reported_once_in_submit_files(self):
        mod = load()
        task = _task("T", submit_files=["a.hwp", "a.hwp", "b.hwp"])
        self.assertEqual(mod.submit_files(task), ["a.hwp", "b.hwp"])

    def test_inspect_lists_only_absent(self):
        mod = load()
        task = _task("T", submit_files=["kept.hwp", "gone.hwp"])
        with tempfile.TemporaryDirectory() as root:
            sub = os.path.join(root, "p", "T")
            os.makedirs(sub)
            with open(os.path.join(sub, "kept.hwp"), "w", encoding="utf-8") as fh:
                fh.write("k")
            inspected = mod.inspect_built_task("/bin/false", "p", task, root)
        self.assertEqual(inspected["kind"], "missing-artifact")
        self.assertEqual(inspected["missing"], ["gone.hwp"])

    def test_answer_task_does_not_require_files(self):
        mod = load()
        task = _task("T")
        with tempfile.TemporaryDirectory() as root:
            os.makedirs(os.path.join(root, "p", "T"))
            with mock.patch.object(mod.runner, "score_task", return_value={"pass": True}):
                inspected = mod.inspect_built_task("/bin/false", "p", task, root)
        self.assertTrue(inspected["ok"])


class FailedScoreEdgeTests(unittest.TestCase):
    def test_empty_checks_without_error(self):
        mod = load()
        folded = mod.fold_score_result({"pass": False, "checks": []})
        self.assertEqual(folded["reason"], "채점 실패")
        self.assertEqual(folded["kind"], "failed-score")

    def test_check_without_name_uses_op(self):
        mod = load()
        folded = mod.fold_score_result({
            "pass": False,
            "checks": [{"op": "file_exists", "ok": False, "error": "없음"}],
        })
        self.assertEqual(folded["reason"], "file_exists: 없음")

    def test_check_without_name_or_op(self):
        mod = load()
        folded = mod.fold_score_result({"pass": False, "checks": [{"ok": False}]})
        self.assertEqual(folded["reason"], "검사: 판정 불일치")

    def test_pass_false_with_all_checks_ok_still_fails(self):
        """봉투의 pass 가 거짓이면 검사 칸이 모두 ok 여도 통과가 아니다."""
        mod = load()
        folded = mod.fold_score_result({
            "pass": False,
            "checks": [{"name": "c", "ok": True}],
        })
        self.assertFalse(folded["ok"])
        self.assertEqual(folded["reason"], "채점 실패")

    def test_truthy_non_bool_pass_is_pass(self):
        mod = load()
        self.assertTrue(mod.score_is_pass({"pass": 1}))
        self.assertFalse(mod.score_is_pass({"pass": 0}))


class ProcessPackSkipTests(unittest.TestCase):
    def test_missing_reference_dir_prints_skip(self):
        mod = load()
        counts = mod.empty_counts()
        with mock.patch.object(mod, "PACKS_DIR", "/no/such/packs"):
            with mock.patch("builtins.print") as printer:
                mod.process_pack("/bin/false", "ghost", "/tmp", counts)
        printer.assert_any_call("[ghost] 기준 풀이 없음 — 건너뜀")
        self.assertEqual(counts["skipped"], 0)
        self.assertEqual(counts["failed"], 0)

    def test_missing_reference_file_bumps_skipped(self):
        mod = load()
        counts = mod.empty_counts()
        with tempfile.TemporaryDirectory() as tmp:
            packs = os.path.join(tmp, "packs")
            ref = os.path.join(packs, "p", "reference")
            os.makedirs(ref)
            with mock.patch.object(mod, "PACKS_DIR", packs):
                with mock.patch.object(mod.runner, "load_pack",
                                       return_value=({}, [{"id": "T01"}])):
                    mod.process_pack("/bin/false", "p", tmp, counts)
        self.assertEqual(counts["skipped"], 1)

    def test_broken_reference_json_is_build_error(self):
        mod = load()
        counts = mod.empty_counts()
        with tempfile.TemporaryDirectory() as tmp:
            packs = os.path.join(tmp, "packs")
            ref = os.path.join(packs, "p", "reference")
            os.makedirs(ref)
            with open(os.path.join(ref, "T01.json"), "w", encoding="utf-8") as fh:
                fh.write("{not-json")
            with mock.patch.object(mod, "PACKS_DIR", packs):
                with mock.patch.object(mod.runner, "load_pack",
                                       return_value=({}, [{"id": "T01"}])):
                    with mock.patch("builtins.print"):
                        mod.process_pack("/bin/false", "p", tmp, counts)
        self.assertEqual(counts["buildError"], 1)
        self.assertEqual(counts["failed"], 1)


class NormalizeScoreTests(unittest.TestCase):
    def test_keeps_error_id_checks(self):
        mod = load()
        out = mod.normalize_score({
            "pass": False, "error": "e", "id": "T", "checks": [1],
        })
        self.assertEqual(out["error"], "e")
        self.assertEqual(out["id"], "T")
        self.assertEqual(out["checks"], [1])
        self.assertFalse(out["pass"])

    def test_main_returns_one_when_failed(self):
        mod = load()
        args = mock.Mock(agent="a", pack=["p"], bin=None, json=False)
        with mock.patch.object(mod, "parse_args", return_value=args):
            with mock.patch.object(mod.runner, "find_bin", return_value="/bin/false"):
                with mock.patch.object(mod, "process_pack",
                                       side_effect=lambda *a: a[-1].update(failed=1, built=0)):
                    with mock.patch("builtins.print"):
                        code = mod.main([])
        self.assertEqual(code, 1)

    def test_main_returns_zero_when_clean(self):
        mod = load()
        args = mock.Mock(agent="a", pack=["p"], bin=None, json=False)
        with mock.patch.object(mod, "parse_args", return_value=args):
            with mock.patch.object(mod.runner, "find_bin", return_value="/bin/false"):
                with mock.patch.object(mod, "process_pack"):
                    with mock.patch("builtins.print"):
                        code = mod.main([])
        self.assertEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
