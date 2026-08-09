#!/usr/bin/env python3
"""schema_validator.py 회귀 테스트 (#4044 리뷰 반영).

실행:
    python tools/schema-validator/test_schema_validator.py
    python -m unittest discover -s tools/schema-validator -p "test_*.py"

고정 대상:
1. 저장소 실물 샘플(tools/rhwp-ingest/schema/sample_*.json)이 오류 0건·경고 0건으로
   통과한다 — 이전 구현은 oneOf 대안 전부 일치라는 허위 경고를 내고 valid=false 였다.
2. 고의로 깨뜨린 사본은 valid=false 이고 CLI 가 비 0 으로 종료한다.
3. oneOf 는 정확히 1개 일치 = 통과, 0개/2개 일치 = 실패(ERROR).
4. 중첩 검증 실패가 상위 반환 bool 로 전파된다.
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOL_DIR = Path(__file__).resolve().parent
REPO_ROOT = TOOL_DIR.parent.parent
SCHEMA_PATH = REPO_ROOT / "tools" / "rhwp-ingest" / "schema" / "ingest_schema_v1.json"
SAMPLE_MINIMAL = REPO_ROOT / "tools" / "rhwp-ingest" / "schema" / "sample_minimal.json"
SAMPLE_STRUCTURED = (
    REPO_ROOT / "tools" / "rhwp-ingest" / "schema" / "sample_structured.json"
)
VALIDATOR_SCRIPT = TOOL_DIR / "schema_validator.py"

sys.path.insert(0, str(TOOL_DIR))

from schema_validator import (  # noqa: E402
    EXIT_INVALID,
    EXIT_OK,
    EXIT_USAGE,
    ErrorLevel,
    IngestSchemaValidator,
)


def error_codes(errors, level=ErrorLevel.ERROR):
    return {e.code for e in errors if e.level is level}


class RepoSampleRegressionTest(unittest.TestCase):
    """리뷰 3번 회귀 — 저장소 실물 샘플은 허위 경고 없이 통과해야 한다."""

    @classmethod
    def setUpClass(cls):
        cls.validator = IngestSchemaValidator(SCHEMA_PATH)

    def test_sample_minimal_valid_no_errors_no_warnings(self):
        errors = self.validator.validate_file(SAMPLE_MINIMAL)
        self.assertEqual(
            [], errors, f"샘플이 오류/경고 없이 통과해야 합니다: {[str(e) for e in errors]}"
        )

    def test_sample_structured_valid_no_errors_no_warnings(self):
        # boxed 블록·공유 지문·머리말/꼬리말까지 포함한 샘플도 동일하게 깨끗해야 한다.
        errors = self.validator.validate_file(SAMPLE_STRUCTURED)
        self.assertEqual(
            [], errors, f"샘플이 오류/경고 없이 통과해야 합니다: {[str(e) for e in errors]}"
        )

    def test_broken_copy_is_invalid(self):
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        data["version"] = "2"  # const "1" 위반
        del data["questions"][0]["number"]  # required 위반
        data["questions"][1]["stem"] = 42  # type 위반 (string 이어야 함)
        data["questions"][2]["number"] = 0  # minimum 1 위반

        errors = self.validator.validate(data)
        codes = error_codes(errors)
        self.assertLessEqual(
            {"CONST_MISMATCH", "MISSING_REQUIRED_FIELD", "TYPE_MISMATCH", "BELOW_MINIMUM"},
            codes,
            f"기대한 오류 코드가 빠졌습니다: {codes}",
        )


class NestedPropagationTest(unittest.TestCase):
    """리뷰 3번 원인 수정 — 중첩 검증 실패는 상위 bool 로 전파된다."""

    @classmethod
    def setUpClass(cls):
        cls.validator = IngestSchemaValidator(SCHEMA_PATH)

    def test_deep_failure_propagates_to_top_level_bool(self):
        # questions[0].choices[0] 에서 required 'text' 를 제거 —
        # 루트 → questions 배열 → Question 객체 → choices 배열 → Choice 객체까지
        # 4단계 중첩을 관통해 False 가 올라와야 한다.
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        del data["questions"][0]["choices"][0]["text"]

        errors = []
        ok = self.validator._validate(data, self.validator.schema, "$", errors)
        self.assertFalse(ok, "중첩 오류가 최상위 반환값으로 전파되어야 합니다")
        self.assertIn("MISSING_REQUIRED_FIELD", error_codes(errors))
        self.assertTrue(
            any(e.path == "questions[0].choices[0]" for e in errors),
            f"오류 경로가 중첩 위치를 가리켜야 합니다: {[e.path for e in errors]}",
        )

    def test_valid_document_returns_true(self):
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        errors = []
        ok = self.validator._validate(data, self.validator.schema, "$", errors)
        self.assertTrue(ok)
        self.assertEqual([], errors)


class OneOfSemanticsTest(unittest.TestCase):
    """리뷰 3번 — oneOf 는 정확히 1개 일치 = 통과, 0개/2개 일치 = 실패."""

    @classmethod
    def setUpClass(cls):
        cls.validator = IngestSchemaValidator(SCHEMA_PATH)
        # 합성 미니 스키마: 대안 a(문자열 a 필수) / 대안 b(정수 b 필수)
        cls.one_of_schema = {
            "oneOf": [
                {
                    "type": "object",
                    "required": ["a"],
                    "properties": {"a": {"type": "string"}},
                },
                {
                    "type": "object",
                    "required": ["b"],
                    "properties": {"b": {"type": "integer"}},
                },
            ]
        }

    def _run(self, value):
        errors = []
        ok = self.validator._validate(value, self.one_of_schema, "$", errors)
        return ok, errors

    def test_exactly_one_match_passes(self):
        ok, errors = self._run({"a": "x"})
        self.assertTrue(ok, f"1개 일치는 통과해야 합니다: {[str(e) for e in errors]}")
        self.assertEqual(set(), error_codes(errors))

    def test_zero_match_fails(self):
        ok, errors = self._run({"c": 1})
        self.assertFalse(ok)
        self.assertIn("ONEOF_FAILED", error_codes(errors))

    def test_two_matches_fail_as_ambiguous(self):
        ok, errors = self._run({"a": "x", "b": 1})
        self.assertFalse(ok, "2개 대안 일치는 draft-07 oneOf 위반으로 실패해야 합니다")
        self.assertIn("ONEOF_AMBIGUOUS", error_codes(errors))

    def test_real_schema_stem_block_matches_exactly_one(self):
        # 실물 StemBlock oneOf: text 블록은 정확히 text 대안 하나에만 일치.
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        errors = self.validator.validate(data)
        self.assertEqual(
            set(), {e.code for e in errors}, "샘플의 StemBlock 판정은 무경고여야 합니다"
        )

    def test_real_schema_unknown_block_type_is_oneof_failure(self):
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        data["questions"][0]["stem_blocks"][0] = {"type": "video", "text": "x"}
        errors = self.validator.validate(data)
        self.assertIn("ONEOF_FAILED", error_codes(errors))
        self.assertTrue(
            any(e.path == "questions[0].stem_blocks[0]" for e in errors),
            f"경로: {[e.path for e in errors]}",
        )

    def test_selected_oneof_branch_warnings_are_preserved(self):
        # text 대안에 없는 bold와 image 전용 ref는 JSON Schema 기본값상 허용되지만
        # Rust StemBlock 파서는 조용한 내용 유실을 막기 위해 거부한다. oneOf가 text
        # 대안 하나와 일치해도 이 경고는 최종 결과에 남아야 한다.
        for extra in ({"bold": True}, {"ref": "img/q1.png"}):
            with self.subTest(extra=extra):
                data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
                data["questions"][0]["stem_blocks"][0].update(extra)
                errors = self.validator.validate(data)

                self.assertEqual(set(), error_codes(errors))
                warnings = [e for e in errors if e.level is ErrorLevel.WARNING]
                self.assertEqual(1, len(warnings), [str(e) for e in errors])
                self.assertEqual("UNKNOWN_FIELD", warnings[0].code)
                self.assertEqual(
                    f"questions[0].stem_blocks[0].{next(iter(extra))}",
                    warnings[0].path,
                )


class UnknownFieldWarningTest(unittest.TestCase):
    """미지 필드는 스키마상 허용이지만 Rust 측이 거부하므로 경고로 조기 신고한다."""

    @classmethod
    def setUpClass(cls):
        cls.validator = IngestSchemaValidator(SCHEMA_PATH)

    def test_typo_field_warns_but_stays_valid(self):
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        data["defaul_font"] = "바탕"  # 오탈자 — build-from-ingest 는 거부한다
        errors = self.validator.validate(data)
        self.assertEqual(set(), error_codes(errors, ErrorLevel.ERROR))
        self.assertIn("UNKNOWN_FIELD", error_codes(errors, ErrorLevel.WARNING))


class CliExitCodeTest(unittest.TestCase):
    """CLI 종료 코드·JSON 출력 계약: 0 = 유효, 1 = 위반, 2 = 사용법·환경 오류."""

    def _run_cli(self, *args):
        return subprocess.run(
            [sys.executable, str(VALIDATOR_SCRIPT), *args],
            capture_output=True,
            encoding="utf-8",
            errors="replace",
        )

    def test_valid_sample_exits_zero_and_reports_valid_true(self):
        proc = self._run_cli(str(SAMPLE_MINIMAL), "--json")
        self.assertEqual(EXIT_OK, proc.returncode, proc.stderr)
        result = json.loads(proc.stdout)
        self.assertTrue(result["valid"])
        self.assertEqual(0, result["error_count"])
        self.assertEqual(0, result["warning_count"])
        self.assertEqual([], result["errors"])

    def test_broken_copy_exits_nonzero_and_reports_valid_false(self):
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        data["version"] = "2"
        del data["questions"][0]["number"]
        with tempfile.TemporaryDirectory() as tmp:
            broken = Path(tmp) / "broken.json"
            broken.write_text(
                json.dumps(data, ensure_ascii=False), encoding="utf-8"
            )
            proc = self._run_cli(str(broken), "--json")
        self.assertEqual(EXIT_INVALID, proc.returncode)
        result = json.loads(proc.stdout)
        self.assertFalse(result["valid"])
        self.assertGreater(result["error_count"], 0)

    def test_selected_oneof_branch_warning_stays_valid_and_exits_zero(self):
        data = json.loads(SAMPLE_MINIMAL.read_text(encoding="utf-8"))
        data["questions"][0]["stem_blocks"][0]["bold"] = True
        with tempfile.TemporaryDirectory() as tmp:
            candidate = Path(tmp) / "oneof-warning.json"
            candidate.write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")
            proc = self._run_cli(str(candidate), "--json")

        self.assertEqual(EXIT_OK, proc.returncode, proc.stderr)
        result = json.loads(proc.stdout)
        self.assertTrue(result["valid"])
        self.assertEqual(0, result["error_count"])
        self.assertEqual(1, result["warning_count"])
        self.assertEqual("WARNING", result["errors"][0]["level"])
        self.assertEqual("UNKNOWN_FIELD", result["errors"][0]["code"])
        self.assertEqual("questions[0].stem_blocks[0].bold", result["errors"][0]["path"])

    def test_syntax_error_reports_line_column(self):
        with tempfile.TemporaryDirectory() as tmp:
            bad = Path(tmp) / "syntax.json"
            bad.write_text('{"version": "1",\n  "questions": [}', encoding="utf-8")
            proc = self._run_cli(str(bad), "--json")
        self.assertEqual(EXIT_INVALID, proc.returncode)
        result = json.loads(proc.stdout)
        self.assertFalse(result["valid"])
        self.assertEqual("INVALID_JSON", result["errors"][0]["code"])
        self.assertIn("Line 2", result["errors"][0]["position"])

    def test_missing_schema_is_usage_error(self):
        proc = self._run_cli(str(SAMPLE_MINIMAL), "--schema", "no_such_schema.json")
        self.assertEqual(EXIT_USAGE, proc.returncode)


if __name__ == "__main__":
    unittest.main(verbosity=2)
