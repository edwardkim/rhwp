#!/usr/bin/env python3
"""hwp_test_data_generator.py 회귀 테스트 (#4044 리뷰 반영).

실행 (rhwp 바이너리 필요 — 없으면 파이프라인 테스트는 skip):
    RHWP_BIN=target/debug/rhwp python tools/test-data-gen/test_hwp_test_data_generator.py
    python -m unittest discover -s tools/test-data-gen -p "test_*.py"

고정 대상:
1. 성공 케이스: 전 템플릿 산출물이 `rhwp info --json` 을 통과한다
   (생성기 내장 검증과 별도로 테스트가 독립 재실행해 이중 확인).
2. 실패 케이스: 스키마 위반 ingest(raw_ingest)는 명확한 오류와 함께 비 0 종료.
3. 설정 오류(미지 키·범위 밖 값·없는 바이너리·없는 템플릿)는 종료 코드 2.
4. 같은 (템플릿, 시드) 는 항상 같은 ingest 를 만든다(결정성).
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOL_DIR = Path(__file__).resolve().parent
GENERATOR_SCRIPT = TOOL_DIR / "hwp_test_data_generator.py"
CONFIG_PATH = TOOL_DIR / "config_templates.json"

sys.path.insert(0, str(TOOL_DIR))

from hwp_test_data_generator import (  # noqa: E402
    EXIT_FAIL,
    EXIT_OK,
    EXIT_USAGE,
    ConfigError,
    build_ingest,
    load_templates,
    validate_template_name,
    validate_template,
)

RHWP_BIN = os.environ.get("RHWP_BIN") or shutil.which("rhwp")
NEEDS_RHWP = unittest.skipUnless(
    RHWP_BIN and Path(RHWP_BIN).exists(),
    "rhwp 바이너리 없음 — RHWP_BIN 환경변수를 설정하거나 PATH 에 rhwp 를 두세요",
)


def run_cli(*args, env_extra=None):
    env = dict(os.environ)
    if env_extra:
        env.update(env_extra)
    return subprocess.run(
        [sys.executable, str(GENERATOR_SCRIPT), *args],
        capture_output=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )


class BuildIngestTest(unittest.TestCase):
    """ingest 생성 로직 — rhwp 바이너리 불필요."""

    def test_same_seed_is_deterministic(self):
        spec = {"questions": 4, "choices_per_question": 5, "boxed_every": 2}
        self.assertEqual(build_ingest(spec, 42), build_ingest(spec, 42))

    def test_knobs_map_to_ingest_features(self):
        spec = {
            "questions": 6,
            "choices_per_question": 4,
            "stem_paragraphs": 2,
            "boxed_every": 3,
            "media_every": 2,
            "passages": 2,
            "questions_per_passage": 2,
            "header_text": "국어 영역",
            "form_label": "홀수형",
        }
        doc = build_ingest(spec, 7)

        self.assertEqual("1", doc["version"])
        self.assertEqual("국어 영역", doc["header_text"])
        self.assertEqual("홀수형", doc["form_label"])
        self.assertEqual(6, len(doc["questions"]))
        self.assertEqual(2, len(doc["passages"]))

        for q in doc["questions"]:
            self.assertEqual(4, len(q["choices"]))
            self.assertGreaterEqual(len(q["stem_blocks"]), 2)

        boxed = [
            q["number"]
            for q in doc["questions"]
            if any(b["type"] == "boxed" for b in q["stem_blocks"])
        ]
        self.assertEqual([3, 6], boxed)

        with_media = [q["number"] for q in doc["questions"] if q.get("media")]
        self.assertEqual([2, 4, 6], with_media)
        for q in doc["questions"]:
            if q.get("media"):
                refs = [b["ref"] for b in q["stem_blocks"] if b["type"] == "image"]
                self.assertEqual([m["id"] for m in q["media"]], refs)

        # 공유 지문: 1~2번은 첫 지문, 3~4번은 둘째 지문을 참조한다.
        self.assertEqual(doc["passages"][0]["id"], doc["questions"][0]["passage_ref"])
        self.assertEqual(doc["passages"][0]["id"], doc["questions"][1]["passage_ref"])
        self.assertEqual(doc["passages"][1]["id"], doc["questions"][2]["passage_ref"])
        self.assertNotIn("passage_ref", doc["questions"][4])

    def test_bundled_config_is_valid(self):
        templates = load_templates(CONFIG_PATH)
        self.assertIn("minimal", templates)
        for name, spec in templates.items():
            build_ingest(spec, 42)  # 예외 없이 생성 가능해야 한다

    def test_unknown_template_key_is_config_error(self):
        with self.assertRaises(ConfigError):
            validate_template("bad", {"questions": 1, "num_tables": 3})

    def test_out_of_range_choices_is_config_error(self):
        with self.assertRaises(ConfigError):
            validate_template("bad", {"questions": 1, "choices_per_question": 6})

    def test_passage_overflow_is_config_error(self):
        with self.assertRaises(ConfigError):
            validate_template(
                "bad", {"questions": 2, "passages": 2, "questions_per_passage": 2}
            )

    def test_template_name_must_be_a_single_safe_filename(self):
        for name in ("", ".", "..", "../escaped", "/tmp/escaped", r"dir\\escaped", "nul\x00name"):
            with self.subTest(name=name):
                with self.assertRaises(ConfigError):
                    validate_template_name(name)


@NEEDS_RHWP
class PipelineRegressionTest(unittest.TestCase):
    """실측 파이프라인 — build-from-ingest 생성물 전수 rhwp info --json 통과."""

    def test_all_templates_generate_hwpx_that_pass_rhwp_info(self):
        with tempfile.TemporaryDirectory() as tmp:
            proc = run_cli(
                "--output-dir", tmp, "--rhwp-bin", RHWP_BIN, "--json", "--seed", "42"
            )
            self.assertEqual(EXIT_OK, proc.returncode, proc.stderr)
            summary = json.loads(proc.stdout)

            templates = load_templates(CONFIG_PATH)
            self.assertEqual(len(templates), summary["count"])

            for doc in summary["documents"]:
                self.assertTrue(doc["verified"], doc)
                self.assertGreater(doc["bytes"], 0, doc)
                self.assertGreaterEqual(doc["pageCount"], 1, doc)

                # 생성기 내장 검증을 신뢰하지 않고 독립적으로 재검증한다.
                info = subprocess.run(
                    [RHWP_BIN, "info", doc["output"], "--json"],
                    capture_output=True,
                    encoding="utf-8",
                    errors="replace",
                )
                self.assertEqual(0, info.returncode, f"{doc['name']}: {info.stderr}")
                info_json = json.loads(info.stdout)
                self.assertEqual("hwpx", info_json["format"], doc["name"])
                self.assertEqual(doc["pageCount"], info_json["pageCount"], doc["name"])

            # large 템플릿은 다중 페이지 흐름을 실제로 만든다.
            large = next(d for d in summary["documents"] if d["name"] == "large")
            self.assertGreater(large["pageCount"], 1)

    def test_schema_violating_raw_ingest_fails_with_nonzero_exit(self):
        # Rust 측 deny_unknown_fields 가 거부하는 입력(boxed 블록에 text)을
        # raw_ingest 로 주입 — 생성기는 rhwp 오류를 표면화하고 exit 1 이어야 한다.
        bad_config = {
            "templates": {
                "broken": {
                    "description": "스키마 위반 fixture",
                    "raw_ingest": {
                        "version": "1",
                        "questions": [
                            {
                                "number": 1,
                                "stem": "잘못된 문서",
                                "stem_blocks": [{"type": "boxed", "text": "잘못"}],
                                "choices": [{"label": "①", "text": "가"}],
                            }
                        ],
                    },
                }
            }
        }
        with tempfile.TemporaryDirectory() as tmp:
            config_path = Path(tmp) / "bad_config.json"
            config_path.write_text(
                json.dumps(bad_config, ensure_ascii=False), encoding="utf-8"
            )
            proc = run_cli(
                "--output-dir",
                str(Path(tmp) / "out"),
                "--config",
                str(config_path),
                "--rhwp-bin",
                RHWP_BIN,
            )
        self.assertEqual(EXIT_FAIL, proc.returncode, proc.stdout)
        self.assertIn("build-from-ingest 실패", proc.stderr)
        self.assertIn("boxed", proc.stderr)  # rhwp 의 원인 메시지가 표면화된다

    def test_missing_required_field_fails_with_nonzero_exit(self):
        # questions[].choices 자체가 없는 입력 — serde required 위반.
        bad_config = {
            "templates": {
                "no-choices": {
                    "description": "필수 필드 누락 fixture",
                    "raw_ingest": {
                        "version": "1",
                        "questions": [{"number": 1, "stem": "선택지 없음"}],
                    },
                }
            }
        }
        with tempfile.TemporaryDirectory() as tmp:
            config_path = Path(tmp) / "bad_config.json"
            config_path.write_text(
                json.dumps(bad_config, ensure_ascii=False), encoding="utf-8"
            )
            proc = run_cli(
                "--output-dir",
                str(Path(tmp) / "out"),
                "--config",
                str(config_path),
                "--rhwp-bin",
                RHWP_BIN,
            )
        self.assertEqual(EXIT_FAIL, proc.returncode, proc.stdout)
        self.assertIn("build-from-ingest 실패", proc.stderr)


class CliUsageErrorTest(unittest.TestCase):
    """설정·환경 오류는 종료 코드 2 — rhwp 실행 전에 걸러진다."""

    def test_list_works_without_binary(self):
        proc = run_cli("--list", env_extra={"RHWP_BIN": ""})
        self.assertEqual(EXIT_OK, proc.returncode, proc.stderr)
        self.assertIn("minimal", proc.stdout)

    def test_missing_rhwp_bin_is_usage_error(self):
        with tempfile.TemporaryDirectory() as tmp:
            proc = run_cli(
                "--output-dir", tmp, "--rhwp-bin", str(Path(tmp) / "no_rhwp.exe")
            )
        self.assertEqual(EXIT_USAGE, proc.returncode)
        self.assertIn("rhwp 바이너리", proc.stderr)

    def test_non_executable_rhwp_bin_is_usage_error_without_traceback(self):
        with tempfile.TemporaryDirectory() as tmp:
            bin_path = Path(tmp) / "not-executable-rhwp"
            bin_path.write_text("", encoding="utf-8")
            bin_path.chmod(0o644)
            proc = run_cli("--output-dir", tmp, "--rhwp-bin", str(bin_path))
        self.assertEqual(EXIT_USAGE, proc.returncode)
        self.assertIn("실행할 수 없습니다", proc.stderr)
        self.assertNotIn("Traceback", proc.stderr)

    def test_path_like_template_name_is_usage_error_before_output_write(self):
        bad_config = {
            "templates": {
                "../escaped": {
                    "description": "출력 경로 이탈 시도",
                    "questions": 1,
                    "choices_per_question": 2,
                }
            }
        }
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            config_path = root / "config.json"
            config_path.write_text(
                json.dumps(bad_config, ensure_ascii=False), encoding="utf-8"
            )
            proc = run_cli(
                "--output-dir", str(root / "out"), "--config", str(config_path),
                env_extra={"RHWP_BIN": ""},
            )
            self.assertFalse((root / "escaped.hwpx").exists())
        self.assertEqual(EXIT_USAGE, proc.returncode)
        self.assertIn("단일 파일명", proc.stderr)

    def test_unknown_template_name_is_usage_error(self):
        with tempfile.TemporaryDirectory() as tmp:
            proc = run_cli(
                "--output-dir", tmp, "--template", "no_such_template",
                env_extra={"RHWP_BIN": ""},
            )
        self.assertEqual(EXIT_USAGE, proc.returncode)
        self.assertIn("알 수 없는 템플릿", proc.stderr)

    def test_invalid_config_is_usage_error(self):
        bad_config = {"templates": {"bad": {"questions": 1, "num_tables": 2}}}
        with tempfile.TemporaryDirectory() as tmp:
            config_path = Path(tmp) / "config.json"
            config_path.write_text(json.dumps(bad_config), encoding="utf-8")
            proc = run_cli(
                "--output-dir", tmp, "--config", str(config_path),
                env_extra={"RHWP_BIN": ""},
            )
        self.assertEqual(EXIT_USAGE, proc.returncode)
        self.assertIn("지원하지 않는 키", proc.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
