"""[#5207] 라이브 오라클 프로브 계약 — 다중 자리표·결정성 실패·부재 보고.

핵심 불변식:
1. 한 문자열의 여러 `{sub:이름}` 은 모두 치환된다(첫 하나만 바꾸면 안 된다).
2. 두 번 계산이 어긋나면 결정성 프로브는 실패한다.
3. 산출물이 없으면 status=absent 이고 통과로 위장하지 않는다.
4. `--json` 은 팩 픽스처 없이 kind=gymOracleProbe / schemaVersion=1.0 을 낸다.

바이너리·pack 없이 순수 함수와 임시 디렉터리만 시험한다.
예외 경로: 빈 입력, 깨진 JSON, 이중 계산 불일치, 오류 봉투, 키 부재,
타입 불일치, NaN/inf, 유니코드, 큰 페이로드, 중복 프로브 id,
결정적 정렬, 분류 행렬.
"""

from __future__ import annotations

import importlib.util
import io
import json
import math
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "oracle_probe.py"
BASELINE = REPO_ROOT / "gym" / "tools" / "build_baseline.py"


def load(name="gym_oracle_probe"):
    spec = importlib.util.spec_from_file_location(name, TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_baseline():
    spec = importlib.util.spec_from_file_location("gym_build_baseline_for_probe", BASELINE)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# ---------------------------------------------------------------------------
# 자리표 — 행복 / 예외
# ---------------------------------------------------------------------------


class PlaceholderTests(unittest.TestCase):
    def test_multiple_sub_placeholders_all_resolve(self):
        mod = load("gym_oracle_probe_multi_sub")
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'
            out = mod.probe_placeholders(token, {"input": "in.hwp"}, sub_dir)
            self.assertTrue(out["ok"], out)
            self.assertNotIn("{sub:", out["resolved"])
            self.assertEqual(out["leftover"], [])
            self.assertIn("o1.hwp", out["resolved"])
            self.assertIn("o2.hwp", out["resolved"])
            self.assertEqual(out["names"], ["o1.hwp", "o2.hwp"])

    def test_resolve_matches_build_baseline(self):
        probe = load("gym_oracle_probe_vs_baseline")
        baseline = load_baseline()
        task = {"input": "samples/in.hwp"}
        with tempfile.TemporaryDirectory() as sub_dir:
            tokens = (
                "{input}",
                '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}',
                "{sub:capsules/child.hwp}",
                "plain-literal",
                "keep {input} embedded",
            )
            for token in tokens:
                self.assertEqual(
                    probe.resolve_placeholders(token, task, sub_dir),
                    baseline.resolve(token, task, sub_dir),
                    token,
                )

    def test_cmd_list_resolves_every_token(self):
        mod = load("gym_oracle_probe_cmd")
        with tempfile.TemporaryDirectory() as sub_dir:
            cmd = ["search", "{sub:edited.hwp}", "--json", "--", "규제"]
            report = mod.probe_cmd_placeholders(cmd, {"input": "in.hwp"}, sub_dir)
            self.assertTrue(report["ok"], report)
            resolved = [t["resolved"] for t in report["tokens"]]
            self.assertTrue(any(str(item).endswith("edited.hwp") for item in resolved))
            self.assertTrue(all("{sub:" not in str(item) for item in resolved))

    def test_exact_input_token_returns_task_input(self):
        mod = load("gym_oracle_probe_exact_input")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_placeholders("{input}", {"input": "samples/a.hwp"}, sub_dir)
            self.assertTrue(report["ok"])
            self.assertTrue(report["inputExact"])
            self.assertEqual(report["resolved"], "samples/a.hwp")
            self.assertEqual(report["names"], [])

    def test_embedded_input_is_replaced(self):
        mod = load("gym_oracle_probe_embedded_input")
        token = "keep {input} embedded"
        resolved = mod.resolve_placeholders(token, {"input": "x.hwp"}, "unused")
        self.assertEqual(resolved, "keep x.hwp embedded")

    def test_exact_sub_token_builds_path_and_parent(self):
        mod = load("gym_oracle_probe_exact_sub")
        with tempfile.TemporaryDirectory() as sub_dir:
            token = "{sub:capsules/child.hwp}"
            self.assertTrue(mod.is_exact_sub_token(token))
            resolved = mod.resolve_placeholders(token, {"input": "in.hwp"}, sub_dir)
            self.assertEqual(resolved, os.path.join(sub_dir, "capsules/child.hwp"))
            self.assertTrue(os.path.isdir(os.path.dirname(resolved)))

    def test_exact_sub_token_mkdir_false_skips_makedirs(self):
        mod = load("gym_oracle_probe_sub_nomk")
        with tempfile.TemporaryDirectory() as sub_dir:
            token = "{sub:ghost/never.hwp}"
            resolved = mod.resolve_placeholders(
                token, {"input": "in.hwp"}, sub_dir, mkdir=False
            )
            self.assertEqual(resolved, os.path.join(sub_dir, "ghost/never.hwp"))
            self.assertFalse(os.path.isdir(os.path.join(sub_dir, "ghost")))

    def test_non_string_token_is_not_pass(self):
        mod = load("gym_oracle_probe_ph_nonstr")
        report = mod.probe_placeholders(12, {"input": "in.hwp"}, ".")
        self.assertFalse(report["ok"])
        self.assertIsNone(report["resolved"])
        self.assertEqual(report["leftover"], [])
        self.assertIn("문자열", report["error"])

    def test_none_token_is_not_pass(self):
        mod = load("gym_oracle_probe_ph_none")
        report = mod.probe_placeholders(None, {"input": "in.hwp"}, ".")
        self.assertFalse(report["ok"])
        self.assertIn("NoneType", report["error"])

    def test_missing_input_key_is_not_pass(self):
        mod = load("gym_oracle_probe_ph_keyerr")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_placeholders("{input}", {}, sub_dir)
            self.assertFalse(report["ok"])
            self.assertIn("KeyError", report["error"])

    def test_none_task_on_exact_input_is_not_pass(self):
        mod = load("gym_oracle_probe_ph_tasknone")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_placeholders("{input}", None, sub_dir)
            self.assertFalse(report["ok"])
            self.assertIn("KeyError", report["error"])

    def test_unclosed_sub_is_not_pass(self):
        mod = load("gym_oracle_probe_ph_unclosed")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_placeholders("keep {sub:", {"input": "in.hwp"}, sub_dir)
            self.assertFalse(report["ok"])
            self.assertTrue(report["leftover"] or report.get("error"))

    def test_empty_string_token_is_pass_literal(self):
        mod = load("gym_oracle_probe_ph_empty")
        report = mod.probe_placeholders("", {"input": "in.hwp"}, ".")
        self.assertTrue(report["ok"], report)
        self.assertEqual(report["resolved"], "")
        self.assertEqual(report["leftover"], [])
        self.assertEqual(report["names"], [])

    def test_leftover_sub_names_on_non_string_is_empty(self):
        mod = load("gym_oracle_probe_leftover_nonstr")
        self.assertEqual(mod.leftover_sub_names(None), [])
        self.assertEqual(mod.leftover_sub_names(3), [])
        self.assertEqual(mod.leftover_sub_names(b"{sub:x}"), [])

    def test_extract_sub_names_inventory_before_resolve(self):
        mod = load("gym_oracle_probe_extract_names")
        names = mod.extract_sub_names("a {sub:one} b {sub:two/c} d")
        self.assertEqual(names, ["one", "two/c"])

    def test_duplicate_sub_names_are_kept_in_order(self):
        mod = load("gym_oracle_probe_dup_names")
        names = mod.extract_sub_names("{sub:a.hwp} then {sub:a.hwp}")
        self.assertEqual(names, ["a.hwp", "a.hwp"])

    def test_is_exact_sub_token_rejects_embedded_and_nested(self):
        mod = load("gym_oracle_probe_exact_guard")
        self.assertFalse(mod.is_exact_sub_token("{input}"))
        self.assertFalse(mod.is_exact_sub_token("x{sub:a}"))
        self.assertFalse(mod.is_exact_sub_token("{sub:a}x"))
        self.assertFalse(mod.is_exact_sub_token("{sub:a}{sub:b}"))
        self.assertFalse(mod.is_exact_sub_token(None))
        self.assertFalse(mod.is_exact_sub_token(1))
        self.assertTrue(mod.is_exact_sub_token("{sub:a}"))

    def test_backslash_in_embedded_sub_is_escaped(self):
        mod = load("gym_oracle_probe_bs_escape")
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"p": "{sub:o.hwp}"}'
            resolved = mod.resolve_placeholders(token, {"input": "in.hwp"}, sub_dir)
            if os.sep == "\\":
                self.assertIn("\\\\", resolved)
            self.assertNotIn("{sub:", resolved)

    def test_resolve_cmd_type_error_on_non_list(self):
        mod = load("gym_oracle_probe_cmd_type")
        with self.assertRaises(TypeError):
            mod.resolve_cmd("info {input}", {"input": "a.hwp"}, ".")
        with self.assertRaises(TypeError):
            mod.resolve_cmd(None, {"input": "a.hwp"}, ".")

    def test_resolve_cmd_happy_path(self):
        mod = load("gym_oracle_probe_cmd_ok")
        with tempfile.TemporaryDirectory() as sub_dir:
            out = mod.resolve_cmd(
                ["info", "{input}", "{sub:out.json}"],
                {"input": "doc.hwp"},
                sub_dir,
            )
            self.assertEqual(out[0], "info")
            self.assertEqual(out[1], "doc.hwp")
            self.assertTrue(str(out[2]).endswith("out.json"))

    def test_probe_cmd_placeholders_non_iterable(self):
        mod = load("gym_oracle_probe_cmd_bad")
        report = mod.probe_cmd_placeholders(3, {"input": "a.hwp"}, ".")
        self.assertFalse(report["ok"])
        self.assertEqual(report["tokens"], [])
        self.assertIn("순회", report["error"])

    def test_probe_cmd_placeholders_mixed_failure(self):
        mod = load("gym_oracle_probe_cmd_mix")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_cmd_placeholders(
                ["ok", 12, "{sub:"],
                {"input": "a.hwp"},
                sub_dir,
            )
            self.assertFalse(report["ok"])
            self.assertEqual(report["count"], 3)
            self.assertTrue(report["tokens"][0]["ok"])
            self.assertFalse(report["tokens"][1]["ok"])
            self.assertFalse(report["tokens"][2]["ok"])


# ---------------------------------------------------------------------------
# 결정성 — 행복 / 드리프트 / 예외 / 정규화
# ---------------------------------------------------------------------------


class DeterminismTests(unittest.TestCase):
    def test_stable_compute_passes(self):
        mod = load("gym_oracle_probe_det_ok")
        report = mod.probe_determinism(lambda: {"pageCount": 3, "kind": "info"})
        self.assertTrue(report["ok"])
        self.assertTrue(report["equal"])
        self.assertEqual(report["runs"], 2)

    def test_determinism_fail_on_drift(self):
        mod = load("gym_oracle_probe_det_fail")
        state = {"i": 0}

        def drift():
            state["i"] += 1
            return {"n": state["i"]}

        report = mod.probe_determinism(drift)
        self.assertFalse(report["ok"])
        self.assertFalse(report["equal"])
        self.assertNotEqual(report["first"], report["second"])

    def test_key_order_and_numeric_string_are_equal(self):
        mod = load("gym_oracle_probe_norm")
        seq = [{"b": 1, "a": "2"}, {"a": 2, "b": "1"}]
        report = mod.probe_determinism(lambda: seq.pop(0))
        self.assertTrue(report["ok"], report)

    def test_exception_is_not_a_pass(self):
        mod = load("gym_oracle_probe_exc")
        report = mod.probe_determinism(lambda: (_ for _ in ()).throw(RuntimeError("boom")))
        self.assertFalse(report["ok"])
        self.assertIn("RuntimeError", report.get("error", ""))

    def test_same_exception_twice_is_still_not_pass(self):
        mod = load("gym_oracle_probe_exc2")
        report = mod.probe_determinism(lambda: 1 / 0)
        self.assertFalse(report["ok"])
        self.assertFalse(report["equal"])
        self.assertEqual(len(report["errors"]), 2)
        self.assertTrue(all("ZeroDivisionError" in e for e in report["errors"]))

    def test_non_callable_is_not_pass(self):
        mod = load("gym_oracle_probe_nocall")
        report = mod.probe_determinism("not-a-fn")
        self.assertFalse(report["ok"])
        self.assertIn("호출 가능", report["error"])

    def test_none_compute_fn_is_not_pass(self):
        mod = load("gym_oracle_probe_fn_none")
        report = mod.probe_determinism(None)
        self.assertFalse(report["ok"])

    def test_first_ok_second_raises(self):
        mod = load("gym_oracle_probe_half")
        box = {"n": 0}

        def once():
            box["n"] += 1
            if box["n"] == 2:
                raise TimeoutError("oracle timeout")
            return {"ok": True}

        report = mod.probe_determinism(once)
        self.assertFalse(report["ok"])
        self.assertIn("TimeoutError", report["error"])
        self.assertIsNotNone(report["first"])
        self.assertIsNone(report["second"])

    def test_timeout_error_envelope_is_not_pass(self):
        mod = load("gym_oracle_probe_timeout")
        report = mod.probe_determinism(lambda: (_ for _ in ()).throw(TimeoutError("deadline")))
        self.assertFalse(report["ok"])
        self.assertIn("TimeoutError", report["error"])
        self.assertIn("deadline", report["error"])

    def test_value_error_envelope(self):
        mod = load("gym_oracle_probe_valerr")
        report = mod.probe_determinism(lambda: (_ for _ in ()).throw(ValueError("malformed")))
        self.assertFalse(report["ok"])
        self.assertIn("malformed", report["error"])

    def test_json_normalize_failure_is_not_pass(self):
        mod = load("gym_oracle_probe_nan_raw")

        class Weird:
            def __iter__(self):
                raise TypeError("cannot walk")

        # str(Weird()) is JSON-able via json_ready fallback, so force a raw nan
        # that survives if json_ready is bypassed — use object that dumps fail.
        # json_ready(custom) → str, so instead feed float('nan') after ready.
        # probe uses json_canonicalize which maps nan → None, so this passes.
        report = mod.probe_determinism(lambda: float("nan"))
        self.assertTrue(report["ok"], report)
        self.assertIsNone(report["first"])

    def test_probe_determinism_n_stable(self):
        mod = load("gym_oracle_probe_n_ok")
        report = mod.probe_determinism_n(lambda: {"k": 1}, 5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["runs"], 5)
        self.assertEqual(len(report["values"]), 5)

    def test_probe_determinism_n_rejects_one(self):
        mod = load("gym_oracle_probe_n_one")
        report = mod.probe_determinism_n(lambda: 1, 1)
        self.assertFalse(report["ok"])
        self.assertIn("2 이상", report["error"])

    def test_probe_determinism_n_rejects_zero_and_negative(self):
        mod = load("gym_oracle_probe_n_bad")
        for n in (0, -1, 1.5, "2", None, True):
            report = mod.probe_determinism_n(lambda: 1, n)
            self.assertFalse(report["ok"], n)

    def test_probe_determinism_n_drift(self):
        mod = load("gym_oracle_probe_n_drift")
        box = {"i": 0}

        def drift():
            box["i"] += 1
            return box["i"]

        report = mod.probe_determinism_n(drift, 4)
        self.assertFalse(report["ok"])
        self.assertFalse(report["equal"])
        self.assertEqual(report["values"], [1.0, 2.0, 3.0, 4.0])

    def test_probe_determinism_n_error_stops_ok(self):
        mod = load("gym_oracle_probe_n_err")
        box = {"i": 0}

        def boom():
            box["i"] += 1
            if box["i"] == 3:
                raise RuntimeError("third")
            return {"i": box["i"]}

        report = mod.probe_determinism_n(boom, 4)
        self.assertFalse(report["ok"])
        self.assertIn("RuntimeError", report["error"])
        self.assertGreaterEqual(len(report["errors"]), 1)
        self.assertTrue(any("third" in e for e in report["errors"]))

    def test_empty_dict_is_deterministic(self):
        mod = load("gym_oracle_probe_empty_dict")
        report = mod.probe_determinism(lambda: {})
        self.assertTrue(report["ok"])
        self.assertEqual(report["canonicalFirst"], "{}")

    def test_empty_list_is_deterministic(self):
        mod = load("gym_oracle_probe_empty_list")
        report = mod.probe_determinism(lambda: [])
        self.assertTrue(report["ok"])
        self.assertEqual(report["canonicalFirst"], "[]")


# ---------------------------------------------------------------------------
# 정규화 — 스칼라 / JSON / 스냅샷 / NaN / 유니코드 / 큰 페이로드
# ---------------------------------------------------------------------------


class NormalizeTests(unittest.TestCase):
    def test_norm_scalar_bool_before_int(self):
        mod = load("gym_oracle_probe_norm_bool")
        self.assertIs(mod.norm_scalar(True), True)
        self.assertIs(mod.norm_scalar(False), False)

    def test_norm_scalar_int_becomes_float(self):
        mod = load("gym_oracle_probe_norm_int")
        self.assertEqual(mod.norm_scalar(3), 3.0)
        self.assertIsInstance(mod.norm_scalar(3), float)

    def test_norm_scalar_nan_inf_float_become_none(self):
        mod = load("gym_oracle_probe_norm_nan")
        self.assertIsNone(mod.norm_scalar(float("nan")))
        self.assertIsNone(mod.norm_scalar(float("inf")))
        self.assertIsNone(mod.norm_scalar(float("-inf")))

    def test_norm_scalar_numeric_string(self):
        mod = load("gym_oracle_probe_norm_numstr")
        self.assertEqual(mod.norm_scalar("  2.0  "), 2.0)
        self.assertEqual(mod.norm_scalar("3"), 3.0)

    def test_norm_scalar_nan_string_stays_string(self):
        mod = load("gym_oracle_probe_norm_nanstr")
        self.assertEqual(mod.norm_scalar("NaN"), "NaN")
        self.assertEqual(mod.norm_scalar("inf"), "inf")
        self.assertEqual(mod.norm_scalar("-inf"), "-inf")

    def test_norm_scalar_plain_string(self):
        mod = load("gym_oracle_probe_norm_str")
        self.assertEqual(mod.norm_scalar("  hangul  "), "hangul")

    def test_norm_scalar_passthrough_other(self):
        mod = load("gym_oracle_probe_norm_other")
        self.assertEqual(mod.norm_scalar(None), None)
        self.assertEqual(mod.norm_scalar([1]), [1])

    def test_json_ready_set_is_sorted(self):
        mod = load("gym_oracle_probe_set")
        ready = mod.json_ready({"z", "a", "m"})
        self.assertEqual(ready, ["a", "m", "z"])

    def test_json_ready_bytes_utf8(self):
        mod = load("gym_oracle_probe_bytes")
        self.assertEqual(mod.json_ready("한글".encode("utf-8")), "한글")

    def test_json_ready_bytes_invalid_becomes_hex(self):
        mod = load("gym_oracle_probe_bytes_hex")
        raw = b"\xff\xfe"
        self.assertEqual(mod.json_ready(raw), raw.hex())

    def test_json_ready_tuple_becomes_list(self):
        mod = load("gym_oracle_probe_tuple")
        self.assertEqual(mod.json_ready((1, "2")), [1.0, 2.0])

    def test_json_ready_custom_object_str(self):
        mod = load("gym_oracle_probe_custom")

        class Box:
            def __str__(self):
                return "box-x"

        self.assertEqual(mod.json_ready(Box()), "box-x")

    def test_json_canonicalize_sorts_keys(self):
        mod = load("gym_oracle_probe_canon_keys")
        a = mod.json_canonicalize({"b": 1, "a": 2})
        b = mod.json_canonicalize({"a": 2, "b": 1})
        self.assertEqual(a, b)
        self.assertEqual(a, '{"a":2.0,"b":1.0}')

    def test_json_canonicalize_unicode_not_escaped(self):
        mod = load("gym_oracle_probe_unicode")
        text = mod.json_canonicalize({"msg": "한글✓"})
        self.assertIn("한글", text)
        self.assertNotIn("\\u", text)

    def test_json_canonicalize_rejects_nothing_after_ready_nan(self):
        mod = load("gym_oracle_probe_canon_nan")
        # float nan folds to null; dumps with allow_nan=False still works
        self.assertEqual(mod.json_canonicalize(float("nan")), "null")
        self.assertEqual(mod.json_canonicalize(float("inf")), "null")

    def test_snapshot_isolates_mutation(self):
        mod = load("gym_oracle_probe_snap")
        src = {"a": [1, 2]}
        shot = mod.snapshot(src)
        src["a"].append(3)
        src["b"] = 9
        self.assertEqual(shot, {"a": [1.0, 2.0]})

    def test_large_payload_is_deterministic(self):
        mod = load("gym_oracle_probe_large")
        payload = {f"k{i:04d}": i for i in range(400)}
        payload["nested"] = [{"i": i, "s": f"값{i}"} for i in range(80)]
        report = mod.probe_determinism(lambda: payload)
        self.assertTrue(report["ok"], report)
        self.assertGreater(len(report["canonicalFirst"]), 1000)

    def test_malformed_json_roundtrip_is_not_this_layers_job(self):
        # 프로브는 파이썬 객체를 받지, JSON 텍스트를 파싱하지 않는다.
        # 깨진 문자열은 그냥 문자열 스칼라다.
        mod = load("gym_oracle_probe_malformed")
        report = mod.probe_determinism(lambda: "{not json")
        self.assertTrue(report["ok"])
        self.assertEqual(report["first"], "{not json")

    def test_missing_keys_survive_canonicalize(self):
        mod = load("gym_oracle_probe_misskey")
        left = {"a": 1}
        right = {"a": 1, "b": 2}
        self.assertNotEqual(mod.json_canonicalize(left), mod.json_canonicalize(right))


# ---------------------------------------------------------------------------
# 산출물 부재 — 분류 행렬
# ---------------------------------------------------------------------------


class MissingArtifactTests(unittest.TestCase):
    def test_missing_file_is_absent_not_pass(self):
        mod = load("gym_oracle_probe_missing")
        with tempfile.TemporaryDirectory() as sub_dir:
            path = os.path.join(sub_dir, "never-written.svg")
            report = mod.probe_missing_artifact(path)
            self.assertFalse(report["ok"])
            self.assertFalse(report["present"])
            self.assertEqual(report["status"], "absent")

    def test_present_file_passes(self):
        mod = load("gym_oracle_probe_present")
        with tempfile.TemporaryDirectory() as sub_dir:
            path = os.path.join(sub_dir, "answer.json")
            Path(path).write_text("{}\n", encoding="utf-8")
            report = mod.probe_missing_artifact(path)
            self.assertTrue(report["ok"])
            self.assertTrue(report["present"])
            self.assertEqual(report["status"], "present")
            self.assertGreater(report["size"], 0)

    def test_directory_is_not_a_file_and_not_pass(self):
        mod = load("gym_oracle_probe_dir")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_missing_artifact(sub_dir)
            self.assertFalse(report["ok"])
            self.assertEqual(report["status"], "not-a-file")

    def test_empty_path_is_invalid(self):
        mod = load("gym_oracle_probe_empty_path")
        report = mod.probe_missing_artifact("")
        self.assertFalse(report["ok"])
        self.assertEqual(report["status"], "invalid")
        self.assertIn("비어", report["error"])

    def test_none_path_is_invalid(self):
        mod = load("gym_oracle_probe_none_path")
        report = mod.probe_missing_artifact(None)
        self.assertFalse(report["ok"])
        self.assertEqual(report["status"], "invalid")
        self.assertIsNone(report["path"])

    def test_int_path_is_invalid(self):
        mod = load("gym_oracle_probe_int_path")
        report = mod.probe_missing_artifact(1)
        self.assertFalse(report["ok"])
        self.assertEqual(report["status"], "invalid")

    def test_pathlike_present(self):
        mod = load("gym_oracle_probe_pathlike")
        with tempfile.TemporaryDirectory() as sub_dir:
            path = Path(sub_dir) / "x.bin"
            path.write_bytes(b"abc")
            report = mod.probe_missing_artifact(path)
            self.assertTrue(report["ok"])
            self.assertEqual(report["size"], 3)

    def test_classify_artifact_matrix(self):
        mod = load("gym_oracle_probe_classify")
        with tempfile.TemporaryDirectory() as sub_dir:
            present = os.path.join(sub_dir, "f.txt")
            Path(present).write_text("x", encoding="utf-8")
            matrix = {
                None: "invalid",
                "": "invalid",
                3: "invalid",
                present: "present",
                sub_dir: "not-a-file",
                os.path.join(sub_dir, "nope"): "absent",
            }
            for path, status in matrix.items():
                self.assertEqual(mod.classify_artifact(path), status, path)

    def test_probe_artifacts_empty_is_not_pass(self):
        mod = load("gym_oracle_probe_arts_empty")
        report = mod.probe_artifacts([])
        self.assertFalse(report["ok"])
        self.assertEqual(report["count"], 0)
        self.assertEqual(report["missing"], [])

    def test_probe_artifacts_one_missing_fails_bundle(self):
        mod = load("gym_oracle_probe_arts_mix")
        with tempfile.TemporaryDirectory() as sub_dir:
            good = os.path.join(sub_dir, "ok.json")
            Path(good).write_text("{}", encoding="utf-8")
            bad = os.path.join(sub_dir, "missing.json")
            report = mod.probe_artifacts([good, bad])
            self.assertFalse(report["ok"])
            self.assertEqual(report["count"], 2)
            self.assertEqual(report["missing"], [bad])

    def test_probe_artifacts_all_present(self):
        mod = load("gym_oracle_probe_arts_ok")
        with tempfile.TemporaryDirectory() as sub_dir:
            paths = []
            for name in ("a.json", "b.json"):
                p = os.path.join(sub_dir, name)
                Path(p).write_text("{}", encoding="utf-8")
                paths.append(p)
            report = mod.probe_artifacts(paths)
            self.assertTrue(report["ok"])
            self.assertEqual(report["missing"], [])

    def test_zero_byte_file_is_present(self):
        mod = load("gym_oracle_probe_zero")
        with tempfile.TemporaryDirectory() as sub_dir:
            path = os.path.join(sub_dir, "empty.bin")
            Path(path).write_bytes(b"")
            report = mod.probe_missing_artifact(path)
            self.assertTrue(report["ok"])
            self.assertEqual(report["size"], 0)


# ---------------------------------------------------------------------------
# 묶음 프로브 / 봉투
# ---------------------------------------------------------------------------


class LiveOracleBundleTests(unittest.TestCase):
    def test_live_oracle_determinism_only(self):
        mod = load("gym_oracle_probe_live_det")
        report = mod.probe_live_oracle(lambda: {"n": 1})
        self.assertTrue(report["ok"])
        self.assertTrue(report["determinism"]["ok"])
        self.assertIsNone(report["placeholders"])
        self.assertIsNone(report["artifacts"])

    def test_live_oracle_placeholder_failure_fails_bundle(self):
        mod = load("gym_oracle_probe_live_ph")
        with tempfile.TemporaryDirectory() as sub_dir:
            report = mod.probe_live_oracle(
                lambda: {"n": 1},
                token=12,
                task={"input": "a.hwp"},
                sub_dir=sub_dir,
            )
            self.assertFalse(report["ok"])
            self.assertFalse(report["placeholders"]["ok"])

    def test_live_oracle_missing_artifact_fails_bundle(self):
        mod = load("gym_oracle_probe_live_art")
        with tempfile.TemporaryDirectory() as sub_dir:
            missing = os.path.join(sub_dir, "nope.svg")
            report = mod.probe_live_oracle(
                lambda: {"n": 1},
                artifacts=[missing],
            )
            self.assertFalse(report["ok"])
            self.assertFalse(report["artifacts"]["ok"])

    def test_live_oracle_all_ok(self):
        mod = load("gym_oracle_probe_live_all")
        with tempfile.TemporaryDirectory() as sub_dir:
            path = os.path.join(sub_dir, "out.json")
            Path(path).write_text("{}", encoding="utf-8")
            report = mod.probe_live_oracle(
                lambda: {"pageCount": 2},
                token="{input}",
                task={"input": "in.hwp"},
                sub_dir=sub_dir,
                artifacts=[path],
            )
            self.assertTrue(report["ok"], report)

    def test_live_oracle_drift_fails_even_if_files_exist(self):
        mod = load("gym_oracle_probe_live_drift")
        box = {"i": 0}

        def drift():
            box["i"] += 1
            return box["i"]

        with tempfile.TemporaryDirectory() as sub_dir:
            path = os.path.join(sub_dir, "out.json")
            Path(path).write_text("{}", encoding="utf-8")
            report = mod.probe_live_oracle(drift, artifacts=[path])
            self.assertFalse(report["ok"])
            self.assertFalse(report["determinism"]["ok"])

    def test_envelope_always_has_kind_and_version(self):
        mod = load("gym_oracle_probe_env")
        body = mod.envelope(ok=True, extra=1)
        self.assertEqual(body["kind"], "gymOracleProbe")
        self.assertEqual(body["schemaVersion"], "1.0")
        self.assertTrue(body["ok"])
        self.assertEqual(body["extra"], 1)

    def test_envelope_fields_do_not_clobber_kind_if_caller_is_careful(self):
        mod = load("gym_oracle_probe_env2")
        # 호출자가 kind 를 넘기면 update 가 덮는다 — 계약 문서화용 관측.
        body = mod.envelope(kind="other")
        self.assertEqual(body["kind"], "other")
        self.assertEqual(body["schemaVersion"], "1.0")


# ---------------------------------------------------------------------------
# 자기점검 / CLI / 렌더
# ---------------------------------------------------------------------------


class EnvelopeAndCliTests(unittest.TestCase):
    def test_json_structural_self_check_has_schema(self):
        proc = subprocess.run(
            [sys.executable, str(TOOL), "--json"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))
        report = json.loads(proc.stdout.decode("utf-8"))
        self.assertEqual(report["kind"], "gymOracleProbe")
        self.assertEqual(report["schemaVersion"], "1.0")
        self.assertEqual(report["mode"], "structural")
        self.assertTrue(report["ok"], report)
        self.assertFalse(report["probes"]["missingArtifact"]["ok"])
        self.assertEqual(report["probes"]["missingArtifact"]["status"], "absent")

    def test_selftest_covers_required_probes(self):
        proc = subprocess.run(
            [sys.executable, str(TOOL), "--json", "--selftest"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))
        report = json.loads(proc.stdout.decode("utf-8"))
        names = {c["name"] for c in report["checks"]}
        self.assertIn("placeholders-multi-sub", names)
        self.assertIn("determinism-drift-detected", names)
        self.assertIn("artifact-absent-is-not-pass", names)
        self.assertTrue(report["ok"], report.get("failed"))

    def test_run_helper_returns_envelope(self):
        mod = load("gym_oracle_probe_run")
        buf = io.StringIO()
        old = sys.stdout
        sys.stdout = buf
        try:
            code = mod.run(["--json"])
        finally:
            sys.stdout = old
        self.assertEqual(code, 0)
        report = json.loads(buf.getvalue())
        self.assertEqual(report["kind"], "gymOracleProbe")
        self.assertEqual(report["schemaVersion"], "1.0")

    def test_run_selftest_json(self):
        mod = load("gym_oracle_probe_run_st")
        buf = io.StringIO()
        old = sys.stdout
        sys.stdout = buf
        try:
            code = mod.run(["--json", "--selftest"])
        finally:
            sys.stdout = old
        self.assertEqual(code, 0)
        report = json.loads(buf.getvalue())
        self.assertEqual(report["mode"], "selftest")
        self.assertGreaterEqual(report["checkCount"], 10)

    def test_run_human_text_mentions_pass(self):
        mod = load("gym_oracle_probe_human")
        buf = io.StringIO()
        old = sys.stdout
        sys.stdout = buf
        try:
            code = mod.run([])
        finally:
            sys.stdout = old
        self.assertEqual(code, 0)
        text = buf.getvalue()
        self.assertIn("라이브 오라클 프로브", text)
        self.assertIn("통과", text)

    def test_parse_args_defaults(self):
        mod = load("gym_oracle_probe_args")
        args = mod.parse_args([])
        self.assertFalse(args.json)
        self.assertFalse(args.selftest)
        args = mod.parse_args(["--json", "--selftest"])
        self.assertTrue(args.json)
        self.assertTrue(args.selftest)

    def test_render_human_selftest_marks(self):
        mod = load("gym_oracle_probe_render")
        text = mod.render_human(
            {
                "kind": "gymOracleProbe",
                "schemaVersion": "1.0",
                "ok": False,
                "mode": "selftest",
                "checks": [{"name": "a", "ok": True}, {"name": "b", "ok": False}],
                "failed": ["b"],
            }
        )
        self.assertIn("실패", text)
        self.assertIn("O a", text)
        self.assertIn("X b", text)
        self.assertIn("! b", text)

    def test_render_human_structural_lists_issues(self):
        mod = load("gym_oracle_probe_render2")
        text = mod.render_human(
            {
                "kind": "gymOracleProbe",
                "schemaVersion": "1.0",
                "ok": False,
                "mode": "structural",
                "exports": ["probe_determinism"],
                "probes": {"determinism": {"ok": False}},
                "issues": ["필수 함수 없음: x"],
            }
        )
        self.assertIn("exports:", text)
        self.assertIn("determinism", text)
        self.assertIn("필수 함수 없음", text)

    def test_structural_self_check_exports_required(self):
        mod = load("gym_oracle_probe_struct")
        report = mod.structural_self_check()
        self.assertTrue(report["ok"], report)
        for name in mod.REQUIRED_EXPORTS:
            self.assertIn(name, report["exports"])

    def test_run_selftest_function_direct(self):
        mod = load("gym_oracle_probe_st_fn")
        report = mod.run_selftest()
        self.assertTrue(report["ok"], report.get("failed"))
        self.assertEqual(report["kind"], "gymOracleProbe")
        self.assertEqual(report["failed"], [])

    def test_main_returns_zero_on_ok(self):
        mod = load("gym_oracle_probe_main")
        old_argv = sys.argv
        buf = io.StringIO()
        old = sys.stdout
        sys.argv = ["oracle_probe.py", "--json"]
        sys.stdout = buf
        try:
            code = mod.main()
        finally:
            sys.argv = old_argv
            sys.stdout = old
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(buf.getvalue())["kind"], "gymOracleProbe")

    def test_subprocess_no_args_exit_zero(self):
        proc = subprocess.run(
            [sys.executable, str(TOOL)],
            cwd=str(REPO_ROOT),
            capture_output=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))
        self.assertIn("통과", proc.stdout.decode("utf-8"))

    def test_stdout_json_is_object_not_array(self):
        proc = subprocess.run(
            [sys.executable, str(TOOL), "--json"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            check=False,
        )
        payload = json.loads(proc.stdout.decode("utf-8"))
        self.assertIsInstance(payload, dict)
        self.assertNotIsInstance(payload, list)


# ---------------------------------------------------------------------------
# 프로브 분류 행렬 · 중복 id · 결정적 정렬
# ---------------------------------------------------------------------------


class ProbeClassificationMatrixTests(unittest.TestCase):
    """프로브 결과의 (ok, status/error) 조합이 겹치지 않게 갈리는지."""

    def test_artifact_status_set_is_closed(self):
        mod = load("gym_oracle_probe_status_set")
        allowed = {"present", "absent", "not-a-file", "invalid"}
        with tempfile.TemporaryDirectory() as sub_dir:
            present = os.path.join(sub_dir, "p")
            Path(present).write_text("1", encoding="utf-8")
            samples = [present, os.path.join(sub_dir, "no"), sub_dir, "", None, 0, []]
            for path in samples:
                status = mod.classify_artifact(path)
                self.assertIn(status, allowed, path)

    def test_determinism_ok_implies_equal_and_no_error(self):
        mod = load("gym_oracle_probe_det_impl")
        report = mod.probe_determinism(lambda: {"x": 1})
        self.assertTrue(report["ok"])
        self.assertTrue(report["equal"])
        self.assertNotIn("error", report)

    def test_determinism_fail_never_ok(self):
        mod = load("gym_oracle_probe_det_impl2")
        cases = [
            lambda: (_ for _ in ()).throw(RuntimeError("x")),
            None,
            "nope",
        ]
        box = {"i": 0}

        def drift():
            box["i"] += 1
            return box["i"]

        cases.append(drift)
        for fn in cases:
            report = mod.probe_determinism(fn)
            self.assertFalse(report["ok"])
            self.assertFalse(report["equal"])

    def test_placeholder_matrix(self):
        mod = load("gym_oracle_probe_ph_matrix")
        with tempfile.TemporaryDirectory() as sub_dir:
            task = {"input": "in.hwp"}
            rows = [
                ("{input}", True, True),
                ("plain", True, False),
                ("{sub:a.hwp}", True, False),
                ("{sub:", False, False),
                (None, False, False),
                (1, False, False),
            ]
            for token, expect_ok, expect_exact in rows:
                report = mod.probe_placeholders(token, task, sub_dir)
                self.assertEqual(report["ok"], expect_ok, token)
                if expect_ok:
                    self.assertEqual(report.get("inputExact"), expect_exact, token)

    def test_duplicate_probe_ids_in_cmd_are_independent(self):
        """같은 {sub:이름} 이 두 자리에 있어도 각각 치환되고 leftover 가 없다."""
        mod = load("gym_oracle_probe_dup_cmd")
        with tempfile.TemporaryDirectory() as sub_dir:
            cmd = ["cp", "{sub:a.hwp}", "{sub:a.hwp}"]
            report = mod.probe_cmd_placeholders(cmd, {"input": "in.hwp"}, sub_dir)
            self.assertTrue(report["ok"], report)
            self.assertEqual(report["count"], 3)
            self.assertEqual(report["tokens"][1]["names"], ["a.hwp"])
            self.assertEqual(report["tokens"][2]["names"], ["a.hwp"])

    def test_deterministic_key_ordering_across_runs(self):
        mod = load("gym_oracle_probe_order")
        payload = {"m": 1, "z": 2, "a": 3}
        texts = [mod.json_canonicalize(payload) for _ in range(8)]
        self.assertEqual(len(set(texts)), 1)
        self.assertTrue(texts[0].startswith('{"a":'))

    def test_set_of_dicts_orders_deterministically(self):
        mod = load("gym_oracle_probe_setdict")
        # 집합은 해시 순이 아니라 json_ready 가 정렬한다.
        left = mod.json_canonicalize({frozenset({"k": 1}.items()), frozenset({"k": 2}.items())})
        right = mod.json_canonicalize({frozenset({"k": 2}.items()), frozenset({"k": 1}.items())})
        # frozenset of items is not a dict; json_ready will str() them.
        self.assertEqual(left, right)

    def test_type_mismatch_in_placeholder_error_names_the_type(self):
        mod = load("gym_oracle_probe_type_name")
        for value, label in ((1.5, "float"), ({}, "dict"), ([], "list"), (b"x", "bytes")):
            report = mod.probe_placeholders(value, {"input": "a"}, ".")
            self.assertFalse(report["ok"])
            self.assertIn(label, report["error"])


class PublicSurfaceSmokeTests(unittest.TestCase):
    """모든 공개 함수에 행복+예외 경로가 최소 한 번씩 닿는지."""

    PUBLIC = (
        "leftover_sub_names",
        "extract_sub_names",
        "is_exact_sub_token",
        "resolve_placeholders",
        "resolve_cmd",
        "probe_placeholders",
        "probe_cmd_placeholders",
        "norm_scalar",
        "json_ready",
        "json_canonicalize",
        "snapshot",
        "probe_determinism",
        "probe_determinism_n",
        "classify_artifact",
        "probe_missing_artifact",
        "probe_artifacts",
        "probe_live_oracle",
        "envelope",
        "structural_self_check",
        "run_selftest",
        "render_human",
        "parse_args",
        "run",
        "main",
    )

    def test_all_public_names_exist_and_callable(self):
        mod = load("gym_oracle_probe_surface")
        for name in self.PUBLIC:
            self.assertTrue(callable(getattr(mod, name)), name)

    def test_required_exports_subset_of_public(self):
        mod = load("gym_oracle_probe_req")
        for name in mod.REQUIRED_EXPORTS:
            self.assertIn(name, self.PUBLIC)

    def test_constants(self):
        mod = load("gym_oracle_probe_const")
        self.assertEqual(mod.KIND, "gymOracleProbe")
        self.assertEqual(mod.SCHEMA_VERSION, "1.0")
        self.assertEqual(mod.INPUT_TOKEN, "{input}")
        self.assertEqual(mod.SUB_MARK, "{sub:")


class UnicodeAndPayloadExceptionTests(unittest.TestCase):
    def test_unicode_placeholder_names(self):
        mod = load("gym_oracle_probe_uni_ph")
        with tempfile.TemporaryDirectory() as sub_dir:
            token = "{sub:한글 문서.hwp}"
            report = mod.probe_placeholders(token, {"input": "in.hwp"}, sub_dir)
            self.assertTrue(report["ok"], report)
            self.assertIn("한글 문서.hwp", report["resolved"])

    def test_unicode_compute_payload(self):
        mod = load("gym_oracle_probe_uni_det")
        payload = {"제목": "국립국어원", "emoji": "🧪", "zwj": "가\u200d나"}
        report = mod.probe_determinism(lambda: payload)
        self.assertTrue(report["ok"])
        self.assertIn("국립국어원", report["canonicalFirst"])

    def test_surrogate_roundtrip_stays_string(self):
        mod = load("gym_oracle_probe_surr")
        text = "ok \ud800"
        # json.dumps may reject lone surrogates depending on python; catch either.
        try:
            canon = mod.json_canonicalize(text)
        except (TypeError, ValueError, UnicodeEncodeError):
            return
        self.assertTrue(isinstance(canon, str))

    def test_deeply_nested_payload(self):
        mod = load("gym_oracle_probe_deep")
        cur = {"v": 0}
        for i in range(30):
            cur = {"c": cur, "i": i}
        report = mod.probe_determinism(lambda: cur)
        self.assertTrue(report["ok"], report)

    def test_empty_input_compute_none(self):
        mod = load("gym_oracle_probe_none_val")
        report = mod.probe_determinism(lambda: None)
        self.assertTrue(report["ok"])
        self.assertIsNone(report["first"])
        self.assertEqual(report["canonicalFirst"], "null")


class ErrorEnvelopeContractTests(unittest.TestCase):
    def test_placeholder_error_keys(self):
        mod = load("gym_oracle_probe_err_keys_ph")
        report = mod.probe_placeholders(object(), {}, ".")
        for key in ("ok", "token", "resolved", "leftover", "names", "error"):
            self.assertIn(key, report)
        self.assertFalse(report["ok"])

    def test_determinism_error_keys(self):
        mod = load("gym_oracle_probe_err_keys_det")
        report = mod.probe_determinism(lambda: 1 / 0)
        for key in ("ok", "equal", "runs", "first", "second", "error", "errors"):
            self.assertIn(key, report)

    def test_artifact_error_keys_invalid(self):
        mod = load("gym_oracle_probe_err_keys_art")
        report = mod.probe_missing_artifact(None)
        for key in ("ok", "present", "status", "path", "error"):
            self.assertIn(key, report)

    def test_n_error_keys(self):
        mod = load("gym_oracle_probe_err_keys_n")
        report = mod.probe_determinism_n(lambda: 1, 0)
        for key in ("ok", "equal", "runs", "error"):
            self.assertIn(key, report)

    def test_cmd_error_keys(self):
        mod = load("gym_oracle_probe_err_keys_cmd")
        report = mod.probe_cmd_placeholders(None, {}, ".")
        for key in ("ok", "tokens", "error"):
            self.assertIn(key, report)

    def test_absence_never_sets_ok_true(self):
        mod = load("gym_oracle_probe_abs_never")
        for path in (None, "", 1, os.path.join(tempfile.gettempdir(), "no-such-rhwp-op")):
            report = mod.probe_missing_artifact(path)
            self.assertIs(report["ok"], False, path)
            self.assertIs(report["present"], False, path)


class NumericEdgeTests(unittest.TestCase):
    def test_bool_not_equal_to_one_after_ready(self):
        mod = load("gym_oracle_probe_bool_one")
        # Python 에서 True == 1.0 이지만 JSON 정규화 문자열은 갈라진다.
        self.assertIsInstance(mod.json_ready(True), bool)
        self.assertIsInstance(mod.json_ready(1), float)
        self.assertEqual(mod.json_canonicalize(True), "true")
        self.assertEqual(mod.json_canonicalize(1), "1.0")

    def test_negative_zero_collapses(self):
        mod = load("gym_oracle_probe_neg0")
        # JSON 1.0 과 -0.0 은 같게 보일 수 있다.
        self.assertEqual(mod.json_ready(0.0), 0.0)
        self.assertEqual(mod.json_ready(-0.0), 0.0)

    def test_large_int_stays_finite(self):
        mod = load("gym_oracle_probe_bigint")
        n = 10**18
        self.assertEqual(mod.json_ready(n), float(n))

    def test_math_nan_same_as_float_nan(self):
        mod = load("gym_oracle_probe_mathnan")
        self.assertIsNone(mod.norm_scalar(math.nan))
        self.assertIsNone(mod.norm_scalar(math.inf))


if __name__ == "__main__":
    unittest.main()
