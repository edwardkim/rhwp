"""[competitive_bench] 명시 예외 경로 — 깨진 입력을 숫자로 위장하지 않는다.

기존 test_gym_competitive_bench.py 의 집계·평결·재렌더 계약은 그대로 둔다.
이 파일은 BenchError 가족, UTF-8/JSON 읽기, 스코어카드·에이전트 식별,
정직성 가드만 고정한다. 바이너리·외부 도구 불요. CLI 플래그를 늘리지 않는다.
"""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "competitive_bench.py"


def load():
    spec = importlib.util.spec_from_file_location("competitive_bench_exc", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ErrorCatalogTests(unittest.TestCase):
    def test_catalog_matches_constants(self):
        m = load()
        catalog = m.error_catalog()
        codes = [row["code"] for row in catalog]
        self.assertEqual(tuple(codes), m.ERROR_CODES)
        self.assertEqual(set(codes), {
            "missing-file", "bad-json", "encoding",
            "empty-scorecard", "unknown-agent", "payload-shape",
        })
        for row in catalog:
            self.assertEqual(row["exit"], m.DEFAULT_ERROR_EXIT)
            self.assertTrue(row["when"])

    def test_error_kind_and_reserved_set(self):
        m = load()
        self.assertEqual(m.ERROR_KIND, "gymCompetitiveBenchError")
        self.assertEqual(m.SCORECARD_KIND, "gymScorecard")
        self.assertEqual(m.SCORECARD_SCHEMA_VERSIONS, ("1.0", "2.0"))
        self.assertEqual(m.SCORECARD_TOTAL_KEYS, ("score", "max", "packsScored"))
        for name in ("rhwp", "pyhwp", "soffice", "hwplib", "all", "none", "baseline"):
            self.assertIn(name, m.RESERVED_AGENT_IDS)


class BenchErrorShapeTests(unittest.TestCase):
    def test_to_dict_omits_empty_path_and_details(self):
        m = load()
        err = m.BenchError("bad-json", "깨졌다")
        rec = err.to_dict()
        self.assertEqual(rec["ok"], False)
        self.assertEqual(rec["kind"], "gymCompetitiveBenchError")
        self.assertEqual(rec["code"], "bad-json")
        self.assertEqual(rec["exitCode"], 2)
        self.assertNotIn("path", rec)
        self.assertNotIn("details", rec)

    def test_to_dict_includes_posix_path_and_details(self):
        m = load()
        err = m.BenchError(
            m.ERR_MISSING_FILE, "없다",
            path=r"samples\a.hwp", details={"errno": 2},
        )
        rec = err.to_dict()
        self.assertEqual(rec["path"], "samples/a.hwp")
        self.assertEqual(rec["details"], {"errno": 2})

    def test_format_and_exit_code(self):
        m = load()
        err = m.MissingFileError("x.json")
        self.assertTrue(m.format_bench_error(err).startswith("오류[missing-file]:"))
        self.assertIn("x.json", m.format_bench_error(err))
        self.assertEqual(m.error_exit_code(err), 2)
        self.assertEqual(m.error_exit_code(RuntimeError("x")), 1)
        self.assertEqual(m.error_exit_code("not-an-error"), 1)

    def test_subclass_codes(self):
        m = load()
        self.assertEqual(m.MissingFileError("p").code, "missing-file")
        self.assertEqual(m.BadJsonError("p").code, "bad-json")
        self.assertEqual(m.EncodingError("p").code, "encoding")
        self.assertEqual(m.EmptyScorecardError("p").code, "empty-scorecard")
        self.assertEqual(m.UnknownAgentError("ghost").code, "unknown-agent")
        self.assertEqual(m.UnknownAgentError("ghost").details["agent"], "ghost")
        shape = m.PayloadShapeError(["tasks 누락"], path="in.json")
        self.assertEqual(shape.code, "payload-shape")
        self.assertEqual(shape.details["issues"], ["tasks 누락"])
        self.assertIn("tasks 누락", shape.message)


class Utf8DecodeTests(unittest.TestCase):
    def test_none_and_non_bytes(self):
        m = load()
        with self.assertRaises(m.EncodingError) as ctx:
            m.utf8_decode(None, path="a.json")
        self.assertEqual(ctx.exception.code, "encoding")
        with self.assertRaises(m.EncodingError):
            m.utf8_decode(12, path="a.json")

    def test_str_passthrough_and_bom_strip(self):
        m = load()
        self.assertEqual(m.utf8_decode("한글"), "한글")
        raw = b"\xef\xbb\xbf{\"ok\": true}"
        self.assertEqual(m.utf8_decode(raw), '{"ok": true}')

    def test_invalid_utf8_is_explicit(self):
        m = load()
        with self.assertRaises(m.EncodingError) as ctx:
            m.utf8_decode(b"\xff\xfe", path="bad.txt")
        self.assertEqual(ctx.exception.code, "encoding")
        self.assertIn("offset", ctx.exception.message)
        self.assertIn("start", ctx.exception.details)


class ReadBytesTests(unittest.TestCase):
    def test_empty_path_and_missing(self):
        m = load()
        with self.assertRaises(m.MissingFileError):
            m.read_bytes("")
        with self.assertRaises(m.MissingFileError):
            m.read_bytes(None)
        with self.assertRaises(m.MissingFileError) as ctx:
            m.read_bytes("definitely-missing-xyz.json")
        self.assertEqual(ctx.exception.code, "missing-file")

    def test_directory_is_not_a_file(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(m.MissingFileError) as ctx:
                m.read_bytes(tmp)
        self.assertIn("디렉터리", ctx.exception.message)

    def test_empty_file_is_allowed(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "empty.json"
            path.write_bytes(b"")
            self.assertEqual(m.read_bytes(path), b"")
            self.assertEqual(m.read_text_utf8(path), "")


class ParseJsonTextTests(unittest.TestCase):
    def test_none_empty_and_non_string(self):
        m = load()
        with self.assertRaises(m.BadJsonError):
            m.parse_json_text(None)
        with self.assertRaises(m.BadJsonError):
            m.parse_json_text("   ")
        with self.assertRaises(m.BadJsonError):
            m.parse_json_text(b"{}")

    def test_truncated_and_trailing_comma(self):
        m = load()
        with self.assertRaises(m.BadJsonError) as ctx:
            m.parse_json_text("{", path="x.json")
        self.assertEqual(ctx.exception.code, "bad-json")
        self.assertIn("lineno", ctx.exception.details)
        with self.assertRaises(m.BadJsonError):
            m.parse_json_text('{"a": 1,}')

    def test_object_and_array_and_require_object(self):
        m = load()
        self.assertEqual(m.parse_json_text('{"a": 1}'), {"a": 1})
        self.assertEqual(m.parse_json_text("[1, 2]"), [1, 2])
        self.assertEqual(m.require_json_object({"k": 1}), {"k": 1})
        with self.assertRaises(m.BadJsonError) as ctx:
            m.require_json_object([1, 2], path="arr.json")
        self.assertIn("객체가 아니다", ctx.exception.message)
        self.assertEqual(ctx.exception.details["jsonType"], "list")


class LoadJsonObjectTests(unittest.TestCase):
    def test_array_top_level_is_bad_json(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "arr.json"
            path.write_text("[1, 2]", encoding="utf-8")
            with self.assertRaises(m.BadJsonError):
                m.load_json_object(path)

    def test_valid_object(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "ok.json"
            path.write_text('{"tasks": []}\n', encoding="utf-8")
            self.assertEqual(m.load_json_object(path), {"tasks": []})

    def test_latin1_bytes_are_encoding_error(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "latin1.json"
            path.write_bytes("{\"x\": \"café\"}".encode("latin-1"))
            with self.assertRaises(m.EncodingError):
                m.load_json_object(path)


class LoadReportFromPathTests(unittest.TestCase):
    def test_missing_and_wrong_kind(self):
        m = load()
        with self.assertRaises(m.MissingFileError):
            m.load_report_from_path("no-such-report.json")
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "wrong.json"
            path.write_text(json.dumps({"kind": "other", "tasks": []}), encoding="utf-8")
            with self.assertRaises(m.PayloadShapeError) as ctx:
                m.load_report_from_path(path)
        self.assertEqual(ctx.exception.code, "payload-shape")

    def test_valid_legacy_without_kind(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "legacy.json"
            path.write_text(json.dumps({"tasks": []}), encoding="utf-8")
            payload = m.load_report_from_path(path)
        self.assertEqual(payload["tasks"], [])

    def test_string_loader_still_returns_tuple(self):
        m = load()
        payload, issues = m.load_report_payload(json.dumps({"tasks": []}))
        self.assertEqual(issues, [])
        self.assertEqual(payload["kind"], "gymCompetitiveBench")
        none, bad = m.load_report_payload("{")
        self.assertIsNone(none)
        self.assertTrue(bad)


class AgentIdTests(unittest.TestCase):
    def test_missing_and_non_string(self):
        m = load()
        self.assertEqual(m.agent_id_issues(None), ["agent 가 없다"])
        self.assertTrue(any("문자열이 아니다" in i for i in m.agent_id_issues(3)))
        self.assertEqual(m.agent_id_issues("   "), ["agent 가 비었다"])

    def test_reserved_and_path_chars(self):
        m = load()
        reserved = m.agent_id_issues("rhwp")
        self.assertTrue(any("예약어" in i for i in reserved))
        pathish = m.agent_id_issues("a/b")
        self.assertTrue(any("경로" in i for i in pathish))
        spaced = m.agent_id_issues("bad agent")
        self.assertTrue(any("공백" in i for i in spaced))

    def test_regex_rejects_leading_digit_and_too_long(self):
        m = load()
        self.assertTrue(any("A-Za-z" in i for i in m.agent_id_issues("1agent")))
        self.assertTrue(any("A-Za-z" in i for i in m.agent_id_issues("a" * 65)))

    def test_normalize_raises_unknown(self):
        m = load()
        with self.assertRaises(m.UnknownAgentError):
            m.normalize_agent_id("")
        with self.assertRaises(m.UnknownAgentError):
            m.normalize_agent_id("soffice")
        self.assertEqual(m.normalize_agent_id("claude-fable-5"), "claude-fable-5")
        self.assertEqual(m.normalize_agent_id("Atlas.1_probe"), "Atlas.1_probe")

    def test_require_known_agent(self):
        m = load()
        with self.assertRaises(m.UnknownAgentError) as empty:
            m.require_known_agent("ok-agent", [])
        self.assertIn("비어", empty.exception.message)
        with self.assertRaises(m.UnknownAgentError) as miss:
            m.require_known_agent("ghost", ["claude-fable-5"])
        self.assertIn("알 수 없는 에이전트", miss.exception.message)
        self.assertEqual(miss.exception.details["known"], ["claude-fable-5"])
        self.assertEqual(
            m.require_known_agent("claude-fable-5", ["claude-fable-5", "novice"]),
            "claude-fable-5",
        )


class DiscoverKnownAgentsTests(unittest.TestCase):
    def test_empty_and_non_dir(self):
        m = load()
        self.assertEqual(m.discover_known_agents(None, ""), [])
        with tempfile.TemporaryDirectory() as tmp:
            missing = str(Path(tmp) / "nope")
            self.assertEqual(m.discover_known_agents(missing), [])

    def test_folder_name_epoch_and_scorecard_agent(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "claude-fable-5-0000").mkdir()
            card_dir = root / "probe-diagnostician-0002"
            card_dir.mkdir()
            (card_dir / "scorecard.json").write_text(
                json.dumps({
                    "kind": "gymScorecard", "schemaVersion": "2.0",
                    "agent": "probe-diagnostician",
                    "total": {"score": 1, "max": 1, "packsScored": 1},
                    "packs": [{"id": "core-cli", "status": "scored", "taskCount": 1}],
                }),
                encoding="utf-8",
            )
            (root / "readme.txt").write_text("ignore", encoding="utf-8")
            broken = root / "broken-0003"
            broken.mkdir()
            (broken / "scorecard.json").write_text("{", encoding="utf-8")
            found = m.discover_known_agents(str(root))
        self.assertIn("claude-fable-5-0000", found)
        self.assertIn("claude-fable-5", found)
        self.assertIn("probe-diagnostician-0002", found)
        self.assertIn("probe-diagnostician", found)
        self.assertIn("broken-0003", found)
        self.assertIn("broken", found)


class ScorecardIssueTests(unittest.TestCase):
    def test_kind_and_version(self):
        m = load()
        self.assertEqual(m.scorecard_kind_issues("x"), ["스코어카드가 객체가 아니다"])
        self.assertTrue(any("kind" in i for i in m.scorecard_kind_issues({"kind": "nope"})))
        self.assertTrue(any("schemaVersion" in i for i in m.scorecard_kind_issues({
            "kind": "gymScorecard", "schemaVersion": "9.9",
        })))
        self.assertEqual(m.scorecard_kind_issues({
            "kind": "gymScorecard", "schemaVersion": "2.0",
        }), [])
        self.assertEqual(m.scorecard_kind_issues({}), [])

    def test_emptiness_matrix(self):
        m = load()
        self.assertEqual(m.scorecard_emptiness_issues({}), ["스코어카드 객체가 비었다"])
        self.assertIn("packs 와 total 이 모두 없다", m.scorecard_emptiness_issues({"agent": "x"}))
        self.assertIn("packs 가 배열이 아니다", m.scorecard_emptiness_issues({"packs": {}}))
        self.assertIn("packs 가 빈 배열이다", m.scorecard_emptiness_issues({"packs": []}))
        self.assertIn("total 이 객체가 아니다", m.scorecard_emptiness_issues({"total": 1}))
        missing = m.scorecard_emptiness_issues({"total": {"score": 1}})
        self.assertTrue(any("total 키 누락" in i for i in missing))
        self.assertIn("packsScored 가 0 이다", m.scorecard_emptiness_issues({
            "total": {"score": 0, "max": 0, "packsScored": 0},
        }))
        self.assertIn("packsScored 가 null 이다", m.scorecard_emptiness_issues({
            "total": {"score": 0, "max": 0, "packsScored": None},
        }))
        self.assertIn("전부 unavailable", m.scorecard_emptiness_issues({
            "packs": [{"id": "a", "status": "unavailable", "taskCount": 3}],
        })[0] if False else " ".join(m.scorecard_emptiness_issues({
            "packs": [{"id": "a", "status": "unavailable", "taskCount": 3}],
        })))
        self.assertIn("taskCount 가 0", " ".join(m.scorecard_emptiness_issues({
            "packs": [{"id": "a", "status": "scored", "taskCount": 0}],
        })))

    def test_load_scorecard_empty_and_agent_mismatch(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            empty = Path(tmp) / "empty.json"
            empty.write_text(json.dumps({
                "kind": "gymScorecard", "schemaVersion": "2.0",
            }), encoding="utf-8")
            with self.assertRaises(m.EmptyScorecardError):
                m.load_scorecard(empty)
            card = Path(tmp) / "card.json"
            card.write_text(json.dumps({
                "kind": "gymScorecard", "schemaVersion": "2.0",
                "agent": "claude-fable-5",
                "total": {"score": 10, "max": 10, "packsScored": 2},
                "packs": [{"id": "core-cli", "status": "scored", "taskCount": 4}],
            }), encoding="utf-8")
            with self.assertRaises(m.UnknownAgentError):
                m.load_scorecard(card, expected_agent="other-agent")
            with self.assertRaises(m.UnknownAgentError):
                m.load_scorecard(card, expected_agent="claude-fable-5",
                                 known_agents=["novice-starter"])
            loaded = m.load_scorecard(
                card, expected_agent="claude-fable-5",
                known_agents=["claude-fable-5"],
            )
        self.assertEqual(loaded["agent"], "claude-fable-5")

    def test_load_scorecard_missing_agent_field(self):
        m = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "no-agent.json"
            path.write_text(json.dumps({
                "kind": "gymScorecard", "schemaVersion": "1.0",
                "total": {"score": 1, "max": 1, "packsScored": 1},
                "packs": [{"id": "x", "status": "scored", "taskCount": 1}],
            }), encoding="utf-8")
            with self.assertRaises(m.UnknownAgentError) as ctx:
                m.load_scorecard(path, expected_agent="want-agent")
        self.assertIn("agent 필드가 없어", ctx.exception.message)

    def test_scorecard_summary_and_attach(self):
        m = load()
        card = {
            "kind": "gymScorecard",
            "schemaVersion": "2.0",
            "agent": "claude-fable-5",
            "total": {"score": 194, "max": 194, "packsScored": 10},
            "packs": [
                {"id": "core-cli", "status": "scored", "taskCount": 14},
                {"id": "security", "status": "scored", "taskCount": 9},
                "bad",
            ],
        }
        summary = m.scorecard_summary(card)
        self.assertEqual(summary["score"], 194)
        self.assertEqual(summary["packCount"], 3)
        self.assertEqual(summary["packIds"], ["core-cli", "security"])
        payload = {"tasks": [], "verdict": ["그대로"]}
        attached = m.attach_scorecard(payload, card)
        self.assertEqual(attached["verdict"], ["그대로"])
        self.assertEqual(attached["scorecard"]["agent"], "claude-fable-5")
        self.assertNotIn("scorecard", payload)


class HonestyGuardTests(unittest.TestCase):
    def test_run_record_issues(self):
        m = load()
        self.assertEqual(m.run_record_issues("x"), ["run 이 객체가 아니다"])
        issues = m.run_record_issues({"ok": True, "ms": "fast", "chars": True})
        self.assertTrue(any("file 이 없다" in i for i in issues))
        self.assertTrue(any("ms 가 숫자가 아니다" in i for i in issues))
        self.assertTrue(any("chars 가 숫자가 아니다" in i for i in issues))
        self.assertEqual(
            m.run_record_issues({"file": "a.hwp", "ok": False, "ms": None, "chars": None}),
            [],
        )

    def test_task_result_unavailable_with_numbers(self):
        m = load()
        self.assertEqual(m.task_result_issues(1), ["result 가 객체가 아니다"])
        dirty = m.unavailable_result("hwplib", "CLI 아님")
        dirty["summary"] = {"ok": 1, "attempted": 1}
        issues = m.task_result_issues(dirty)
        self.assertTrue(any("숫자를 실었다" in i for i in issues))
        avail_bad = {"tool": "rhwp", "available": True}
        self.assertTrue(any("summary 가 없다" in i for i in m.task_result_issues(avail_bad)))

    def test_task_result_bad_runs(self):
        m = load()
        issues = m.task_result_issues({
            "tool": "rhwp", "available": True,
            "summary": {"attempted": 1, "ok": 1},
            "runs": {"file": "a.hwp"},
        })
        self.assertTrue(any("runs 가 배열이 아니다" in i for i in issues))
        issues = m.task_result_issues({
            "tool": "rhwp", "available": True,
            "summary": {"attempted": 1, "ok": 1},
            "runs": [{"ok": True, "ms": 1}],
        })
        self.assertTrue(any("file 이 없다" in i for i in issues))

    def test_payload_honesty_and_require(self):
        m = load()
        self.assertEqual(m.payload_honesty_issues("x"), ["payload 가 객체가 아니다"])
        empty = m.payload_honesty_issues({"tasks": []})
        self.assertTrue(any("빈 배열" in i for i in empty))
        broken = {
            "kind": "gymCompetitiveBench",
            "tasks": [
                "nope",
                {"results": []},
                {"task": "export-text", "results": {"tool": "rhwp"}},
                {"task": "info", "results": [
                    m.unavailable_result("pyhwp", "n/a") | {"runs": [1]},
                ]},
            ],
        }
        text = " ".join(m.payload_honesty_issues(broken))
        self.assertIn("task 가 객체가 아니다", text)
        self.assertIn("task 이름이 없다", text)
        self.assertIn("results 가 배열이 아니다", text)
        self.assertIn("숫자를 실었다", text)
        with self.assertRaises(m.PayloadShapeError):
            m.require_honest_payload({"tasks": []})
        honest = {
            "kind": "gymCompetitiveBench",
            "schemaVersion": "1.0",
            "tasks": [{
                "task": "export-text",
                "results": [
                    m.available_result("rhwp", [
                        {"file": "a.hwp", "ext": ".hwp", "ok": True, "ms": 1, "chars": 2},
                    ]),
                    m.unavailable_result("hwplib", "CLI 아님"),
                ],
            }],
        }
        self.assertIs(m.require_honest_payload(honest), honest)


class ClassifyFailureTests(unittest.TestCase):
    def test_timeout_missing_permission_encoding_json_runtime(self):
        m = load()
        self.assertEqual(m.classify_cli_failure("Timeout>60s"), "timeout")
        self.assertEqual(m.classify_cli_failure("cannot find the file"), "missing_input")
        self.assertEqual(m.classify_cli_failure("파일이 없는 파일입니다"), "missing_input")
        self.assertEqual(m.classify_cli_failure("Permission denied"), "permission")
        self.assertEqual(m.classify_cli_failure("액세스가 거부되었습니다"), "permission")
        self.assertEqual(m.classify_cli_failure("utf-8 codec can't decode"), m.ERR_ENCODING)
        self.assertEqual(m.classify_cli_failure("not valid JSON"), m.ERR_BAD_JSON)
        self.assertEqual(m.classify_cli_failure("segfault"), "runtime")
        self.assertEqual(m.classify_cli_failure(""), "runtime")
        self.assertEqual(m.classify_cli_failure(None), "runtime")


class DefaultRootsTests(unittest.TestCase):
    def test_default_known_agent_roots_point_at_gym(self):
        m = load()
        roots = m.default_known_agent_roots()
        self.assertEqual(len(roots), 3)
        self.assertTrue(any(r.endswith("baselines") or r.replace("\\", "/").endswith("baselines")
                            for r in roots))
        self.assertTrue(any("scorecards" in r.replace("\\", "/") for r in roots))
        self.assertTrue(any(r.replace("\\", "/").endswith("submissions") for r in roots))


if __name__ == "__main__":
    unittest.main()
