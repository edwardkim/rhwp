#!/usr/bin/env python3
"""rhwp_doctor.py 의 순수 로직 가드 테스트 — 바이너리 불요.

.mcp.json 방출기와 리포트 집계(종료 코드), 레시피 지도 실존 검증, 샘플 선택을
스텁 경로로 검증한다. 예외 경로(바이너리 없음·불량 샘플·네트워크 없음)와
호스트별 MCP 스니펫, 첫 5분 레시피 실존, 매직 분류를 같은 게이트에 둔다.
rhwp 바이너리 없이도 돌므로 CI 의 바이너리 불요 게이트에 맞는다.

실행:
    python -m unittest tools/agent_onboarding/test_rhwp_doctor.py
"""

import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

# CWD 와 무관하게 대상 모듈을 import 한다(CI 는 저장소 루트에서 파일 경로로 호출).
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rhwp_doctor as doc  # noqa: E402


class TestMcpSnippet(unittest.TestCase):
    def test_path_case_uses_bare_command(self):
        snip = doc.build_mcp_snippet("rhwp")
        self.assertEqual(snip["mcpServers"]["rhwp"]["command"], "rhwp")
        self.assertEqual(snip["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_absolute_path_case(self):
        abspath = r"C:\repo\target\release\rhwp.exe"
        snip = doc.build_mcp_snippet(abspath)
        self.assertEqual(snip["mcpServers"]["rhwp"]["command"], abspath)
        self.assertEqual(snip["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_snippet_is_json_roundtrippable(self):
        snip = doc.build_mcp_snippet("rhwp")
        again = json.loads(json.dumps(snip, ensure_ascii=False))
        self.assertEqual(again["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_args_are_copied_not_aliased(self):
        shared = ["mcp-serve"]
        snip = doc.build_mcp_snippet("rhwp", shared)
        shared.append("--boom")
        self.assertEqual(snip["mcpServers"]["rhwp"]["args"], ["mcp-serve"])


class TestMcpHostShapes(unittest.TestCase):
    def test_unknown_host_is_none(self):
        self.assertIsNone(doc.build_mcp_snippet_for_host("not-a-host", "rhwp"))

    def test_claude_code_is_shape_a(self):
        pack = doc.build_mcp_snippet_for_host("claude-code", "rhwp")
        self.assertEqual(pack["shape"], "A")
        self.assertEqual(pack["file"], ".mcp.json")
        self.assertEqual(pack["snippet"]["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_cursor_is_shape_a(self):
        pack = doc.build_mcp_snippet_for_host("cursor", r"C:\bin\rhwp.exe")
        self.assertEqual(pack["shape"], "A")
        self.assertEqual(pack["snippet"]["mcpServers"]["rhwp"]["command"], r"C:\bin\rhwp.exe")

    def test_vscode_is_shape_b(self):
        pack = doc.build_mcp_snippet_for_host("vscode", "rhwp")
        self.assertEqual(pack["shape"], "B")
        server = pack["snippet"]["servers"]["rhwp"]
        self.assertEqual(server["type"], "stdio")
        self.assertEqual(server["command"], "rhwp")
        self.assertEqual(server["args"], ["mcp-serve"])

    def test_zed_uses_context_servers(self):
        pack = doc.build_mcp_snippet_for_host("zed", "/opt/rhwp")
        self.assertEqual(pack["shape"], "zed")
        cmd = pack["snippet"]["context_servers"]["rhwp"]["command"]
        self.assertEqual(cmd["path"], "/opt/rhwp")
        self.assertEqual(cmd["args"], ["mcp-serve"])

    def test_goose_uses_cmd_key(self):
        pack = doc.build_mcp_snippet_for_host("goose", "rhwp")
        self.assertEqual(pack["snippet"]["rhwp"]["type"], "stdio")
        self.assertEqual(pack["snippet"]["rhwp"]["cmd"], "rhwp")

    def test_continue_is_list(self):
        pack = doc.build_mcp_snippet_for_host("continue", "rhwp")
        self.assertIsInstance(pack["snippet"]["mcpServers"], list)
        self.assertEqual(pack["snippet"]["mcpServers"][0]["name"], "rhwp")

    def test_host_args_are_copied(self):
        shared = ["mcp-serve"]
        pack = doc.build_mcp_snippet_for_host("claude-code", "rhwp", shared)
        shared.append("--nope")
        self.assertEqual(pack["snippet"]["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_every_catalog_host_builds(self):
        for host in doc.MCP_HOSTS:
            pack = doc.build_mcp_snippet_for_host(host["id"], "rhwp")
            self.assertIsNotNone(pack, host["id"])
            self.assertEqual(pack["host"], host["id"])
            self.assertIn(pack["shape"], {"A", "B", "zed", "goose", "continue"})
            raw = json.dumps(pack["snippet"], ensure_ascii=False)
            self.assertIn("rhwp", raw)
            self.assertNotIn("mcp-serve-v2", raw)

    def test_list_mcp_hosts_is_a_copy(self):
        rows = doc.list_mcp_hosts()
        rows[0]["id"] = "mutated"
        self.assertNotEqual(doc.MCP_HOSTS[0]["id"], "mutated")

    def test_no_host_invents_a_port(self):
        for host in doc.MCP_HOSTS:
            pack = doc.build_mcp_snippet_for_host(host["id"], "rhwp")
            blob = json.dumps(pack["snippet"])
            self.assertNotIn("localhost:", blob)
            self.assertNotIn("PORT", blob)
            self.assertNotIn("Authorization", blob)


class TestAggregate(unittest.TestCase):
    def _chk(self, status, critical=True):
        return {"id": "x", "status": status, "critical": critical}

    def test_all_pass_is_zero(self):
        ok, code = doc.aggregate([self._chk(doc.PASS), self._chk(doc.PASS)], binary_found=True)
        self.assertTrue(ok)
        self.assertEqual(code, 0)

    def test_critical_fail_is_one(self):
        ok, code = doc.aggregate([self._chk(doc.PASS), self._chk(doc.FAIL)], binary_found=True)
        self.assertFalse(ok)
        self.assertEqual(code, 1)

    def test_critical_skip_is_not_ok(self):
        ok, code = doc.aggregate([self._chk(doc.SKIP)], binary_found=True)
        self.assertFalse(ok)
        self.assertEqual(code, 1)

    def test_binary_missing_is_three(self):
        # 바이너리가 없으면 검사 목록이 비어 있어도 종료 코드 3(빌드 필요).
        ok, code = doc.aggregate([], binary_found=False)
        self.assertFalse(ok)
        self.assertEqual(code, 3)

    def test_noncritical_fail_does_not_sink_health(self):
        ok, code = doc.aggregate([self._chk(doc.PASS), self._chk(doc.FAIL, critical=False)], binary_found=True)
        self.assertTrue(ok)
        self.assertEqual(code, 0)

    def test_noncritical_skip_does_not_sink_health(self):
        ok, code = doc.aggregate(
            [self._chk(doc.PASS), self._chk(doc.SKIP, critical=False)], binary_found=True
        )
        self.assertTrue(ok)
        self.assertEqual(code, 0)

    def test_binary_missing_wins_over_passing_checks(self):
        ok, code = doc.aggregate([self._chk(doc.PASS, critical=False)], binary_found=False)
        self.assertFalse(ok)
        self.assertEqual(code, 3)

    def test_network_skip_does_not_force_exit_one(self):
        checks = [
            {"id": "version", "status": doc.PASS, "critical": True},
            {"id": "selftest-info", "status": doc.PASS, "critical": True},
            {"id": "selftest-export-text", "status": doc.PASS, "critical": True},
            {"id": "network", "status": doc.SKIP, "critical": False},
        ]
        ok, code = doc.aggregate(checks, binary_found=True)
        self.assertTrue(ok)
        self.assertEqual(code, 0)


class TestRecipeMap(unittest.TestCase):
    def test_missing_repo_marks_everything_absent(self):
        with tempfile.TemporaryDirectory() as d:
            rows = doc.resolve_recipe_map(Path(d))
            self.assertEqual(len(rows), len(doc.RECIPES))
            for r in rows:
                self.assertFalse(r["skillExists"])
                self.assertFalse(r["recipeExists"])

    def test_detects_existing_skill_and_recipe(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            # 첫 레시피의 스킬 SKILL.md 를 만들어 실존 검출을 확인.
            skill = doc.RECIPES[0]["skill"]
            (root / ".claude" / "skills" / skill).mkdir(parents=True)
            (root / ".claude" / "skills" / skill / "SKILL.md").write_text("x", encoding="utf-8")
            # recipe 가 있는 항목 하나를 골라 파일 생성.
            with_recipe = next(r for r in doc.RECIPES if r["recipe"])
            rp = root / with_recipe["recipe"]
            rp.parent.mkdir(parents=True, exist_ok=True)
            rp.write_text("x", encoding="utf-8")

            rows = doc.resolve_recipe_map(root)
            by_skill = {r["skill"]: r for r in rows}
            self.assertTrue(by_skill[skill]["skillExists"])
            self.assertTrue(next(r for r in rows if r["recipe"] == with_recipe["recipe"])["recipeExists"])

    def test_recipe_none_is_never_marked_existing(self):
        # recipe 가 None 인 항목은 recipeExists 가 항상 False 여야 한다(빈 인용 방지).
        rows = doc.resolve_recipe_map(doc.default_repo_root())
        for r in rows:
            if r["recipe"] is None:
                self.assertFalse(r["recipeExists"])

    def test_recipes_do_not_point_at_gym(self):
        for r in doc.RECIPES:
            blob = json.dumps(r, ensure_ascii=False)
            self.assertNotIn("gym/", blob)
            self.assertNotIn("gym\\", blob)

    def test_first_five_min_do_not_invent_edit_subcommands(self):
        invented = ("edit invent", "edit magic", "fill-fields --new-engine")
        for step in doc.FIRST_5_MIN:
            self.assertTrue(step["readOnly"])
            for cmd in step["commands"]:
                for bad in invented:
                    self.assertNotIn(bad, cmd)

    def test_first_five_min_resolution_missing_repo(self):
        with tempfile.TemporaryDirectory() as d:
            rows = doc.resolve_first_5_min(Path(d))
            self.assertEqual(len(rows), len(doc.FIRST_5_MIN))
            for r in rows:
                self.assertFalse(r["skillExists"])
                self.assertFalse(r["referenceExists"])

    def test_first_five_min_resolution_detects_reference(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            step = doc.FIRST_5_MIN[0]
            ref = root / step["reference"]
            ref.parent.mkdir(parents=True, exist_ok=True)
            ref.write_text("# triage\n", encoding="utf-8")
            rows = doc.resolve_first_5_min(root)
            hit = next(r for r in rows if r["id"] == step["id"])
            self.assertTrue(hit["referenceExists"])
            self.assertFalse(hit["skillExists"])


class TestOnboardingReferences(unittest.TestCase):
    def test_catalog_ids_are_unique(self):
        ids = [i["id"] for i in doc.ONBOARDING_REFERENCES]
        self.assertEqual(len(ids), len(set(ids)))

    def test_missing_repo_marks_absent(self):
        with tempfile.TemporaryDirectory() as d:
            rows = doc.resolve_onboarding_references(Path(d))
            self.assertTrue(rows)
            self.assertTrue(all(not r["exists"] for r in rows))

    def test_detects_created_reference(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            item = doc.ONBOARDING_REFERENCES[0]
            p = root / item["path"]
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text("x", encoding="utf-8")
            rows = {r["id"]: r for r in doc.resolve_onboarding_references(root)}
            self.assertTrue(rows[item["id"]]["exists"])

    def test_required_reference_ids_present(self):
        ids = {i["id"] for i in doc.ONBOARDING_REFERENCES}
        for needed in (
            "skill",
            "first-5-min",
            "mcp-json-paste",
            "binary-discovery",
            "sample-selftest",
            "exception-missing-binary",
            "exception-bad-sample",
            "exception-no-network",
            "exception-transcripts",
            "first-5-min-envelopes",
            "binary-discovery-matrix",
            "first-5-min-receipt",
            "doctor-report-schema",
            "onboarding-catalog",
            "host-paste-examples",
            "working-doc",
        ):
            self.assertIn(needed, ids)


class TestPickSample(unittest.TestCase):
    def test_none_when_absent(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertIsNone(doc.pick_sample(Path(d), None))

    def test_override_wins_when_present(self):
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / "my.hwp"
            f.write_text("x", encoding="utf-8")
            self.assertEqual(doc.pick_sample(Path(d), str(f)), f)

    def test_override_absent_returns_none(self):
        with tempfile.TemporaryDirectory() as d:
            self.assertIsNone(doc.pick_sample(Path(d), str(Path(d) / "nope.hwp")))

    def test_finds_candidate_in_tree(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            rel = doc.SAMPLE_CANDIDATES[0]
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text("x", encoding="utf-8")
            self.assertEqual(doc.pick_sample(root, None), p)


class TestSampleClassification(unittest.TestCase):
    def test_none_is_missing(self):
        cls = doc.classify_sample(None)
        self.assertFalse(cls["ok"])
        self.assertEqual(cls["kind"], doc.KIND_MISSING)

    def test_absent_file_is_missing(self):
        with tempfile.TemporaryDirectory() as d:
            cls = doc.classify_sample(Path(d) / "nope.hwp")
            self.assertFalse(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_MISSING)

    def test_empty_file(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "empty.hwp"
            p.write_bytes(b"")
            cls = doc.classify_sample(p)
            self.assertFalse(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_EMPTY)

    def test_too_small_text(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "tiny.hwp"
            p.write_bytes(b"hello")
            cls = doc.classify_sample(p)
            self.assertFalse(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_TOO_SMALL)

    def test_not_document_text(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "fake.hwp"
            p.write_bytes(b"this is not a hangul document at all. " * 8)
            cls = doc.classify_sample(p)
            self.assertFalse(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_NOT_DOCUMENT)

    def test_ole_magic_is_hwp5(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "ole.hwp"
            p.write_bytes(doc.OLE_MAGIC + b"\x00" * 80)
            cls = doc.classify_sample(p)
            self.assertTrue(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_HWP5)

    def test_zip_magic_is_hwpx(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "pack.hwpx"
            p.write_bytes(b"PK\x03\x04" + b"\x00" * 80)
            cls = doc.classify_sample(p)
            self.assertTrue(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_HWPX)

    def test_hwp3_magic(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "old.hwp"
            p.write_bytes(doc.HWP3_MAGIC + b"\x00" * 80)
            cls = doc.classify_sample(p)
            self.assertTrue(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_HWP3)

    def test_avoid_prefix(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            p = root / "samples" / "broken" / "x.hwp"
            p.parent.mkdir(parents=True)
            p.write_bytes(doc.OLE_MAGIC + b"\x00" * 80)
            cls = doc.classify_sample(p, root)
            self.assertFalse(cls["ok"])
            self.assertEqual(cls["kind"], doc.KIND_AVOID)

    def test_gym_prefix_is_avoided(self):
        self.assertTrue(doc.should_avoid_sample(Path("gym/packs/foo.hwp")))

    def test_output_prefix_is_avoided(self):
        self.assertTrue(doc.should_avoid_sample(Path("output/tmp.hwp")))

    def test_basic_sample_is_not_avoided(self):
        self.assertFalse(doc.should_avoid_sample(Path("samples/basic/english.hwp")))

    def test_classify_magic_empty(self):
        self.assertEqual(doc.classify_magic(b""), doc.KIND_EMPTY)

    def test_fixture_bad_samples_if_present(self):
        fx = doc.fixture_dir() / "samples"
        if not fx.is_dir():
            self.skipTest("fixtures/samples 없음")
        mapping = {
            "empty.hwp": doc.KIND_EMPTY,
            "not_hwp.txt": {doc.KIND_TOO_SMALL, doc.KIND_NOT_DOCUMENT},
            "tiny.hwp": doc.KIND_TOO_SMALL,
            "text_named_hwp.hwp": doc.KIND_NOT_DOCUMENT,
            "truncated_ole.hwp": {doc.KIND_TOO_SMALL, doc.KIND_HWP5},
        }
        for name, expected in mapping.items():
            p = fx / name
            if not p.is_file():
                continue
            kind = doc.classify_sample(p)["kind"]
            if isinstance(expected, set):
                self.assertIn(kind, expected, name)
            else:
                self.assertEqual(kind, expected, name)


class TestJsonHelpers(unittest.TestCase):
    def test_empty_stdout(self):
        obj, err = doc.parse_json_object("   ")
        self.assertIsNone(obj)
        self.assertIn("비었", err)

    def test_array_rejected(self):
        obj, err = doc.parse_json_object("[1,2]")
        self.assertIsNone(obj)
        self.assertIn("object", err)

    def test_object_ok(self):
        obj, err = doc.parse_json_object('{"format":"hwp5","pageCount":1}')
        self.assertIsNone(err)
        self.assertEqual(obj["format"], "hwp5")

    def test_invalid_json(self):
        obj, err = doc.parse_json_object("{")
        self.assertIsNone(obj)
        self.assertIn("JSON", err)

    def test_missing_keys(self):
        self.assertEqual(doc.missing_keys({"format": "hwp5"}, doc.INFO_REQUIRED_KEYS), ["pageCount"])
        self.assertEqual(doc.missing_keys({"format": "hwp5", "pageCount": 1}, doc.INFO_REQUIRED_KEYS), [])


class TestSelftestFailureClass(unittest.TestCase):
    def test_timeout(self):
        self.assertEqual(doc.classify_selftest_failure(1, "", True), doc.EXC_SELFTEST_TIMEOUT)

    def test_runtime_is_bad_sample(self):
        self.assertEqual(doc.classify_selftest_failure(1, "파싱 실패"), doc.EXC_BAD_SAMPLE)

    def test_usage_is_parse(self):
        self.assertEqual(doc.classify_selftest_failure(2, "사용법"), doc.EXC_SELFTEST_PARSE)

    def test_json_decode_hint(self):
        self.assertEqual(
            doc.classify_selftest_failure(0, "json decode boom", False),
            doc.EXC_SELFTEST_PARSE,
        )


class TestExceptionPlaybook(unittest.TestCase):
    def test_three_required_kinds_have_steps(self):
        for kind in (doc.EXC_MISSING_BINARY, doc.EXC_BAD_SAMPLE, doc.EXC_NO_NETWORK):
            book = doc.exception_playbook(kind)
            self.assertTrue(book["nextSteps"])
            blob = "\n".join(book["nextSteps"])
            self.assertIn(".md", blob)

    def test_missing_binary_mentions_build(self):
        book = doc.exception_playbook(doc.EXC_MISSING_BINARY)
        blob = " ".join(book["nextSteps"])
        self.assertIn("cargo build --release --bin rhwp", blob)
        self.assertNotIn("cargo install gym", blob)

    def test_no_network_is_not_failure_language_only(self):
        book = doc.exception_playbook(doc.EXC_NO_NETWORK)
        blob = " ".join(book["nextSteps"])
        self.assertIn("오프라인", blob)

    def test_make_exception_shape(self):
        exc = doc.make_exception(doc.EXC_BAD_SAMPLE, "tiny", "x.hwp")
        self.assertEqual(exc["kind"], doc.EXC_BAD_SAMPLE)
        self.assertEqual(exc["path"], "x.hwp")
        self.assertTrue(exc["nextSteps"])

    def test_collect_exceptions_dedupes(self):
        checks = [
            {"status": doc.FAIL, "exception": doc.EXC_BAD_SAMPLE, "detail": "a"},
            {"status": doc.FAIL, "exception": doc.EXC_BAD_SAMPLE, "detail": "b"},
        ]
        extra = [doc.make_exception(doc.EXC_BAD_SAMPLE, "first", None)]
        out = doc.collect_exceptions(checks, extra)
        kinds = [e["kind"] for e in out]
        self.assertEqual(kinds.count(doc.EXC_BAD_SAMPLE), 1)

    def test_collect_ignores_pass_exception_tag(self):
        checks = [{"status": doc.PASS, "exception": doc.EXC_NO_NETWORK, "detail": "x"}]
        out = doc.collect_exceptions(checks, [])
        self.assertEqual(out, [])


class TestSkippedSelftests(unittest.TestCase):
    def test_missing_is_skip(self):
        rows = doc.skipped_selftests("없음")
        self.assertEqual(len(rows), 2)
        self.assertTrue(all(r["status"] == doc.SKIP for r in rows))
        self.assertTrue(all(r["critical"] for r in rows))

    def test_bad_sample_is_fail(self):
        rows = doc.skipped_selftests("가짜", doc.EXC_BAD_SAMPLE)
        self.assertTrue(all(r["status"] == doc.FAIL for r in rows))
        self.assertTrue(all(r["exception"] == doc.EXC_BAD_SAMPLE for r in rows))


class TestNetwork(unittest.TestCase):
    def test_skipped_network_shape(self):
        net = doc.skipped_network("--offline")
        self.assertFalse(net["probed"])
        self.assertTrue(net["offline"])
        self.assertIsNone(net["reachable"])

    def test_check_network_offline_is_noncritical_skip(self):
        chk = doc.check_network(doc.skipped_network("--offline"))
        self.assertEqual(chk["status"], doc.SKIP)
        self.assertFalse(chk["critical"])
        self.assertEqual(chk["exception"], doc.EXC_NO_NETWORK)

    def test_check_network_reachable_is_pass(self):
        net = {
            "probed": True,
            "reachable": True,
            "offline": False,
            "targets": [{"host": "1.1.1.1", "port": 443, "ok": True, "error": None}],
        }
        chk = doc.check_network(net)
        self.assertEqual(chk["status"], doc.PASS)
        self.assertFalse(chk["critical"])

    def test_check_network_unreachable_is_skip(self):
        net = {"probed": True, "reachable": False, "offline": True, "targets": []}
        chk = doc.check_network(net)
        self.assertEqual(chk["status"], doc.SKIP)
        self.assertEqual(chk["exception"], doc.EXC_NO_NETWORK)

    def test_probe_network_fake_target_offline(self):
        net = doc.probe_network(timeout=0.05, probes=(("127.0.0.1", 1),))
        self.assertTrue(net["probed"])
        self.assertTrue(net["offline"])
        self.assertFalse(net["targets"][0]["ok"])


class TestBinaryDiscovery(unittest.TestCase):
    def test_plan_starts_with_override_when_given(self):
        with tempfile.TemporaryDirectory() as d:
            plan = doc.binary_search_plan(Path(d), "/tmp/rhwp")
            self.assertEqual(plan[0]["source"], "--rhwp")

    def test_plan_includes_release_and_debug(self):
        with tempfile.TemporaryDirectory() as d:
            sources = [r["source"] for r in doc.binary_search_plan(Path(d), None)]
            self.assertIn("PATH", sources)
            self.assertIn("target/release", sources)
            self.assertIn("target/debug", sources)

    def test_plan_includes_env_when_set(self):
        with tempfile.TemporaryDirectory() as d:
            env = {"RHWP_BIN": str(Path(d) / "rhwp")}
            plan = doc.binary_search_plan(Path(d), None, env)
            self.assertTrue(any(r["source"] == "RHWP_BIN" for r in plan))

    def test_discover_marks_missing(self):
        with tempfile.TemporaryDirectory() as d:
            rows = doc.discover_binary_candidates(Path(d), None, env={})
            self.assertTrue(any(r["source"] == "target/release" and not r["exists"] for r in rows))

    def test_discover_hits_created_release(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            exe = root / "target" / "release" / doc._exe_name()
            exe.parent.mkdir(parents=True)
            exe.write_bytes(b"x")
            rows = doc.discover_binary_candidates(root, None, env={})
            hit = next(r for r in rows if r["source"] == "target/release")
            self.assertTrue(hit["exists"])

    def test_find_binary_override_missing(self):
        with tempfile.TemporaryDirectory() as d:
            path, source, on_path = doc.find_binary(Path(d), str(Path(d) / "nope"))
            self.assertIsNone(path)
            self.assertEqual(source, "--rhwp(미발견)")
            self.assertFalse(on_path)

    def test_find_binary_override_hit(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "rhwp.bin"
            p.write_bytes(b"x")
            path, source, on_path = doc.find_binary(Path(d), str(p))
            self.assertEqual(path, p)
            self.assertEqual(source, "--rhwp")
            self.assertFalse(on_path)

    def test_find_binary_release_when_no_path(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            exe = root / "target" / "release" / doc._exe_name()
            exe.parent.mkdir(parents=True)
            exe.write_bytes(b"x")
            with mock.patch.object(doc.shutil, "which", return_value=None):
                path, source, on_path = doc.find_binary(root, None)
            self.assertEqual(path, exe)
            self.assertEqual(source, "target/release")
            self.assertFalse(on_path)

    def test_find_binary_debug_fallback(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            exe = root / "target" / "debug" / doc._exe_name()
            exe.parent.mkdir(parents=True)
            exe.write_bytes(b"x")
            with mock.patch.object(doc.shutil, "which", return_value=None):
                path, source, on_path = doc.find_binary(root, None)
            self.assertEqual(path, exe)
            self.assertEqual(source, "target/debug")
            self.assertFalse(on_path)

    def test_find_binary_env_after_path_and_release_miss(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            p = root / "custom-rhwp"
            p.write_bytes(b"x")
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.dict(os.environ, {"RHWP_BIN": str(p)}, clear=False):
                    path, source, on_path = doc.find_binary(root, None)
            self.assertEqual(path, p)
            self.assertEqual(source, "RHWP_BIN")
            self.assertFalse(on_path)

    def test_choose_mcp_command(self):
        self.assertEqual(doc.choose_mcp_command(None, False), "rhwp")
        self.assertEqual(doc.choose_mcp_command(Path("/x/rhwp"), True), "rhwp")
        self.assertEqual(doc.choose_mcp_command(Path("/x/rhwp"), False), str(Path("/x/rhwp")))


class TestMkAndRender(unittest.TestCase):
    def test_mk_includes_exception_steps(self):
        row = doc._mk("x", "t", doc.FAIL, "cmd", "d", True, exception=doc.EXC_MISSING_BINARY)
        self.assertEqual(row["exception"], doc.EXC_MISSING_BINARY)
        self.assertTrue(row["nextSteps"])

    def test_render_human_missing_binary(self):
        report = {
            "ok": False,
            "exitCode": 3,
            "repoRoot": "/tmp/repo",
            "buildCommand": doc.BUILD_COMMAND,
            "binary": {"found": False, "path": None, "source": "(미발견)", "onPath": False},
            "binaryInventory": [{"source": "PATH", "exists": False, "path": "rhwp", "resolved": None}],
            "checks": [],
            "mcpJson": doc.build_mcp_snippet("rhwp"),
            "recipes": [],
            "first5Min": [],
            "exceptions": [doc.make_exception(doc.EXC_MISSING_BINARY, "없음")],
            "network": {"probed": False, "reason": "--offline"},
            "sampleClassification": None,
        }
        buf = io.StringIO()
        doc.render_human(report, buf)
        text = buf.getvalue()
        self.assertIn("미발견", text)
        self.assertIn("missing_binary", text)
        self.assertIn(doc.BUILD_COMMAND, text)

    def test_render_human_bad_sample(self):
        report = {
            "ok": False,
            "exitCode": 1,
            "repoRoot": "/tmp/repo",
            "buildCommand": doc.BUILD_COMMAND,
            "binary": {"found": True, "path": "/bin/rhwp", "source": "PATH", "onPath": True},
            "binaryInventory": [],
            "checks": [
                {
                    "id": "selftest-info",
                    "title": "자가검증: info",
                    "status": doc.FAIL,
                    "command": "rhwp info x",
                    "detail": "OLE 없음",
                    "critical": True,
                    "exception": doc.EXC_BAD_SAMPLE,
                }
            ],
            "mcpJson": doc.build_mcp_snippet("rhwp"),
            "recipes": [],
            "first5Min": [
                {
                    "id": "triage",
                    "title": "트리아지",
                    "reference": "x.md",
                    "referenceExists": True,
                    "commands": ["rhwp info a --json"],
                }
            ],
            "exceptions": [doc.make_exception(doc.EXC_BAD_SAMPLE, "OLE 없음", "x.hwp")],
            "network": {"probed": True, "reachable": False},
            "sampleClassification": {
                "ok": False,
                "kind": doc.KIND_NOT_DOCUMENT,
                "reason": "시그니처 없음",
                "sizeBytes": 80,
            },
        }
        buf = io.StringIO()
        doc.render_human(report, buf)
        text = buf.getvalue()
        self.assertIn("시그니처 없음", text)
        self.assertIn("bad_sample", text)
        self.assertIn("트리아지", text)


class TestMainCli(unittest.TestCase):
    def test_list_hosts_exit_zero(self):
        buf = io.StringIO()
        err = io.StringIO()
        with mock.patch.object(sys, "stdout", buf), mock.patch.object(sys, "stderr", err):
            code = doc.main(["--list-hosts"])
        self.assertEqual(code, 0)
        payload = json.loads(buf.getvalue())
        ids = {h["id"] for h in payload["hosts"]}
        self.assertIn("claude-code", ids)
        self.assertIn("vscode", ids)

    def test_list_recipes_exit_zero(self):
        with tempfile.TemporaryDirectory() as d:
            buf = io.StringIO()
            with mock.patch.object(sys, "stdout", buf):
                code = doc.main(["--list-recipes", "--repo-root", d])
            self.assertEqual(code, 0)
            payload = json.loads(buf.getvalue())
            self.assertIn("recipes", payload)
            self.assertIn("first5Min", payload)
            self.assertEqual(len(payload["recipes"]), len(doc.RECIPES))

    def test_unknown_host_is_usage(self):
        err = io.StringIO()
        with mock.patch.object(sys, "stderr", err):
            code = doc.main(["--host", "not-real"])
        self.assertEqual(code, 2)
        self.assertIn("알 수 없는 --host", err.getvalue())

    def test_missing_binary_json_exit_three(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            out = io.StringIO()
            err = io.StringIO()
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", err):
                    code = doc.main(["--json", "--offline", "--repo-root", str(root)])
            self.assertEqual(code, 3)
            report = json.loads(out.getvalue())
            self.assertFalse(report["ok"])
            self.assertEqual(report["exitCode"], 3)
            self.assertFalse(report["binary"]["found"])
            kinds = {e["kind"] for e in report["exceptions"]}
            self.assertIn(doc.EXC_MISSING_BINARY, kinds)
            self.assertIn(doc.EXC_NO_NETWORK, kinds)
            # stdout 은 JSON 하나. 사람용 안내는 stderr.
            self.assertIn("미발견", err.getvalue())

    def test_bad_sample_json_exit_one(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            exe = root / "target" / "release" / doc._exe_name()
            exe.parent.mkdir(parents=True)
            exe.write_bytes(b"x")
            bad = root / "fake.hwp"
            bad.write_bytes(b"this is not a hangul document. " * 8)
            out = io.StringIO()
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.object(doc, "check_version", return_value=doc._mk(
                    "version", "바이너리 버전", doc.PASS, "rhwp --version", "rhwp 0.0-test", True, version="rhwp 0.0-test"
                )):
                    with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", io.StringIO()):
                        code = doc.main(
                            [
                                "--json",
                                "--offline",
                                "--repo-root",
                                str(root),
                                "--sample",
                                str(bad),
                                "--skip-extra",
                            ]
                        )
            self.assertEqual(code, 1)
            report = json.loads(out.getvalue())
            self.assertFalse(report["ok"])
            self.assertEqual(report["sampleClassification"]["kind"], doc.KIND_NOT_DOCUMENT)
            kinds = {e["kind"] for e in report["exceptions"]}
            self.assertIn(doc.EXC_BAD_SAMPLE, kinds)
            info = next(c for c in report["checks"] if c["id"] == "selftest-info")
            self.assertEqual(info["status"], doc.FAIL)

    def test_write_refuses_existing_without_force(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            target = root / ".mcp.json"
            target.write_text("{}\n", encoding="utf-8")
            out = io.StringIO()
            err = io.StringIO()
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", err):
                    code = doc.main(
                        ["--json", "--offline", "--repo-root", str(root), "--write", str(target)]
                    )
            self.assertEqual(code, 2)
            self.assertEqual(target.read_text(encoding="utf-8"), "{}\n")
            self.assertIn("이미 있어", err.getvalue())

    def test_write_creates_file(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            target = root / "out" / ".mcp.json"
            out = io.StringIO()
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", io.StringIO()):
                    code = doc.main(
                        ["--json", "--offline", "--repo-root", str(root), "--write", str(target)]
                    )
            self.assertEqual(code, 3)  # 바이너리 없음
            self.assertTrue(target.is_file())
            payload = json.loads(target.read_text(encoding="utf-8"))
            self.assertEqual(payload["mcpServers"]["rhwp"]["args"], ["mcp-serve"])

    def test_skip_selftest_does_not_require_sample(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            exe = root / "target" / "release" / doc._exe_name()
            exe.parent.mkdir(parents=True)
            exe.write_bytes(b"x")
            out = io.StringIO()
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.object(doc, "check_version", return_value=doc._mk(
                    "version", "바이너리 버전", doc.PASS, "rhwp --version", "rhwp 0.0-test", True, version="rhwp 0.0-test"
                )):
                    with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", io.StringIO()):
                        code = doc.main(
                            ["--json", "--offline", "--repo-root", str(root), "--skip-selftest"]
                        )
            self.assertEqual(code, 0)
            report = json.loads(out.getvalue())
            self.assertTrue(report["ok"])
            ids = {c["id"] for c in report["checks"]}
            self.assertNotIn("selftest-info", ids)

    def test_schema_version_is_additive(self):
        with tempfile.TemporaryDirectory() as d:
            out = io.StringIO()
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", io.StringIO()):
                    doc.main(["--json", "--offline", "--repo-root", d])
            report = json.loads(out.getvalue())
            self.assertEqual(report["schemaVersion"], doc.SCHEMA_VERSION)
            for key in (
                "binary",
                "checks",
                "mcpJson",
                "recipes",
                "exceptions",
                "network",
                "first5Min",
                "references",
                "binaryInventory",
            ):
                self.assertIn(key, report)


class TestNoNewCliAndNoGym(unittest.TestCase):
    def test_doctor_source_has_no_gym_runner(self):
        src = Path(doc.__file__).read_text(encoding="utf-8")
        self.assertNotIn("gym/score.py", src)
        self.assertNotIn("gym.certify", src)

    def test_doctor_does_not_define_new_rhwp_subcommand(self):
        src = Path(doc.__file__).read_text(encoding="utf-8")
        self.assertNotIn("cmd_onboard", src)
        self.assertNotIn('Some("doctor")', src)

    def test_first_5_min_commands_are_existing_surface(self):
        allowed_prefixes = (
            "rhwp info ",
            "rhwp explain ",
            "rhwp digest ",
            "rhwp export-tables ",
            "rhwp table-to-csv ",
            "rhwp fields ",
            "rhwp inspect ",
            "rhwp mcp-serve",
            "rhwp capabilities ",
            "rhwp replay ",
        )
        for step in doc.FIRST_5_MIN:
            for cmd in step["commands"]:
                self.assertTrue(
                    any(cmd.startswith(p) for p in allowed_prefixes),
                    f"unexpected command in {step['id']}: {cmd}",
                )


class TestEnvelopeContracts(unittest.TestCase):
    def test_info_keys(self):
        self.assertEqual(doc.INFO_REQUIRED_KEYS, ("format", "pageCount"))

    def test_export_text_keys(self):
        self.assertEqual(doc.EXPORT_TEXT_REQUIRED_KEYS, ("pages",))

    def test_explain_keys_include_summary(self):
        self.assertIn("summary", doc.EXPLAIN_REQUIRED_KEYS)
        self.assertIn("pageCount", doc.EXPLAIN_REQUIRED_KEYS)

    def test_injection_keys(self):
        self.assertIn("clean", doc.INJECTION_REQUIRED_KEYS)
        self.assertIn("signalCount", doc.INJECTION_REQUIRED_KEYS)

    def test_fixture_envelope_files_if_present(self):
        fx = doc.fixture_dir() / "envelopes"
        if not fx.is_dir():
            self.skipTest("fixtures/envelopes 없음")
        mapping = {
            "info_required_keys.json": list(doc.INFO_REQUIRED_KEYS),
            "export_text_required_keys.json": list(doc.EXPORT_TEXT_REQUIRED_KEYS),
            "explain_required_keys.json": list(doc.EXPLAIN_REQUIRED_KEYS),
            "digest_required_keys.json": list(doc.DIGEST_REQUIRED_KEYS),
            "injection_required_keys.json": list(doc.INJECTION_REQUIRED_KEYS),
        }
        for name, expected in mapping.items():
            p = fx / name
            if not p.is_file():
                continue
            payload = json.loads(p.read_text(encoding="utf-8"))
            self.assertEqual(payload["required"], expected, name)


class TestDefaultRepoRoot(unittest.TestCase):
    def test_points_at_worktree_root(self):
        root = doc.default_repo_root()
        self.assertTrue((root / "tools" / "agent_onboarding" / "rhwp_doctor.py").is_file())


class TestLooksLikeHelpers(unittest.TestCase):
    def test_pe(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "a.exe"
            p.write_bytes(b"MZ\x90\x00")
            self.assertTrue(doc.looks_like_pe(p))
            p.write_bytes(b"XX")
            self.assertFalse(doc.looks_like_pe(p))

    def test_elf(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "a"
            p.write_bytes(b"\x7fELF\x02")
            self.assertTrue(doc.looks_like_elf(p))

    def test_macho(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "a"
            p.write_bytes(b"\xcf\xfa\xed\xfe" + b"\x00" * 4)
            self.assertTrue(doc.looks_like_macho(p))


class TestParser(unittest.TestCase):
    def test_known_flags(self):
        ap = doc.build_parser()
        args = ap.parse_args(["--json", "--offline", "--skip-selftest", "--skip-extra", "--host", "cursor"])
        self.assertTrue(args.json)
        self.assertTrue(args.offline)
        self.assertTrue(args.skip_selftest)
        self.assertTrue(args.skip_extra)
        self.assertEqual(args.host, "cursor")


class TestReferenceFilesExistInWorktree(unittest.TestCase):
    """워크트리에 온보딩 참고 문서가 실제로 있는지를 가드한다."""

    def test_every_catalog_path_exists_here(self):
        root = doc.default_repo_root()
        missing = []
        for item in doc.ONBOARDING_REFERENCES:
            if not (root / item["path"]).is_file():
                missing.append(item["path"])
        self.assertEqual(missing, [])

    def test_first_5_min_reference_files_exist(self):
        root = doc.default_repo_root()
        missing = [s["reference"] for s in doc.FIRST_5_MIN if not (root / s["reference"]).is_file()]
        self.assertEqual(missing, [])

    def test_skill_mentions_doctor_and_not_gym_score(self):
        text = (doc.default_repo_root() / ".claude/skills/rhwp-onboarding/SKILL.md").read_text(encoding="utf-8")
        self.assertIn("rhwp_doctor.py", text)
        self.assertIn("missing_binary", text)
        self.assertIn("bad_sample", text)
        self.assertIn("no_network", text)
        self.assertNotIn("gym/score.py", text)
        self.assertNotIn("gym certify", text)

    def test_working_doc_declares_issue_and_non_scope(self):
        text = (doc.default_repo_root() / "mydocs/working/agent_onboarding.md").read_text(encoding="utf-8")
        self.assertIn("#5292", text)
        self.assertIn("gym/", text)
        self.assertIn("새 rhwp CLI", text)
        self.assertIn("rhwp_doctor.py", text)

    def test_exception_docs_name_their_kind(self):
        root = doc.default_repo_root() / ".claude/skills/rhwp-onboarding/references"
        mapping = {
            "exception-missing-binary.md": "missing_binary",
            "exception-bad-sample.md": "bad_sample",
            "exception-no-network.md": "no_network",
        }
        for name, kind in mapping.items():
            text = (root / name).read_text(encoding="utf-8")
            self.assertIn(kind, text, name)
            self.assertIn("```bash", text, name)

    def test_mcp_paste_has_no_listen_port(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/mcp-json-paste.md"
        ).read_text(encoding="utf-8")
        self.assertIn("stdio", text)
        self.assertIn("claude-code", text)
        self.assertIn("vscode", text)
        self.assertNotIn("127.0.0.1:", text)
        self.assertNotIn("--port", text)

    def test_form_read_does_not_invent_fill_flags(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/first-5-min-form-read.md"
        ).read_text(encoding="utf-8")
        self.assertIn("rhwp fields", text)
        self.assertIn("읽기 전용", text)
        self.assertNotIn("edit fill-fields --auto-guess", text)
        self.assertNotIn("new-fill-engine", text)

    def test_triage_uses_existing_read_commands(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/first-5-min-triage.md"
        ).read_text(encoding="utf-8")
        for cmd in ("rhwp info", "rhwp explain", "rhwp digest"):
            self.assertIn(cmd, text)

    def test_tables_points_at_export_tables(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/first-5-min-tables.md"
        ).read_text(encoding="utf-8")
        self.assertIn("export-tables", text)
        self.assertIn("table-to-csv", text)
        self.assertIn("samples/hwp_table_test.hwp", text)

    def test_security_is_read_only_inspect(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/first-5-min-security.md"
        ).read_text(encoding="utf-8")
        self.assertIn("inspect hidden-text", text)
        self.assertIn("inspect injection", text)
        self.assertIn("inspect unicode", text)
        self.assertIn("clean", text)

    def test_sample_selftest_lists_candidates(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/sample-selftest.md"
        ).read_text(encoding="utf-8")
        for rel in doc.SAMPLE_CANDIDATES:
            self.assertIn(rel, text)

    def test_binary_discovery_lists_search_order(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/binary-discovery.md"
        ).read_text(encoding="utf-8")
        self.assertIn("RHWP_BIN", text)
        self.assertIn("target/release", text)
        self.assertIn("target/debug", text)
        self.assertIn(doc.BUILD_COMMAND, text)

    def test_report_schema_documents_1_1(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/doctor-report-schema.md"
        ).read_text(encoding="utf-8")
        self.assertIn("schemaVersion", text)
        self.assertIn("1.1", text)
        self.assertIn("exceptions", text)
        self.assertIn("binaryInventory", text)
        self.assertIn("first5Min", text)

    def test_envelopes_do_not_invent_keys(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/first-5-min-envelopes.md"
        ).read_text(encoding="utf-8")
        self.assertIn("format", text)
        self.assertIn("pageCount", text)
        self.assertIn("untrustedContent", text)
        self.assertNotIn("madeUpScore", text)
        self.assertNotIn("gymPackId", text)

    def test_catalog_says_not_gym(self):
        text = (
            doc.default_repo_root()
            / ".claude/skills/rhwp-onboarding/references/onboarding-catalog.md"
        ).read_text(encoding="utf-8")
        self.assertIn("never start gym here", text)
        self.assertIn("--offline", text)


class TestFixtureContracts(unittest.TestCase):
    def test_mcp_fixtures_have_no_ports(self):
        folder = doc.fixture_dir() / "mcp"
        if not folder.is_dir():
            self.skipTest("fixtures/mcp 없음")
        for p in folder.glob("*.json"):
            blob = p.read_text(encoding="utf-8")
            self.assertNotIn("port", blob.lower(), p.name)
            self.assertNotIn("Authorization", blob, p.name)
            json.loads(blob)

    def test_recipe_index_is_read_only(self):
        p = doc.fixture_dir() / "recipes" / "first_5_min_index.json"
        if not p.is_file():
            self.skipTest("recipe index 없음")
        payload = json.loads(p.read_text(encoding="utf-8"))
        self.assertTrue(payload["notGym"])
        self.assertTrue(payload["noNewCli"])
        for step in payload["steps"]:
            self.assertTrue(step["readOnly"], step["id"])

    def test_report_shapes_cover_three_exceptions(self):
        folder = doc.fixture_dir() / "reports"
        if not folder.is_dir():
            self.skipTest("fixtures/reports 없음")
        missing = json.loads((folder / "missing_binary.shape.json").read_text(encoding="utf-8"))
        bad = json.loads((folder / "bad_sample.shape.json").read_text(encoding="utf-8"))
        net = json.loads((folder / "no_network.shape.json").read_text(encoding="utf-8"))
        self.assertEqual(missing["exitCode"], 3)
        self.assertIn("missing_binary", missing["exceptions.kind"])
        self.assertEqual(bad["exitCode"], 1)
        self.assertIn("bad_sample", bad["exceptions.kind"])
        self.assertTrue(net["network.offline"])

    def test_readme_warns_fixtures_are_failures(self):
        p = doc.fixture_dir() / "README.md"
        if not p.is_file():
            self.skipTest("fixtures README 없음")
        text = p.read_text(encoding="utf-8")
        self.assertIn("실패", text)
        self.assertIn("samples/basic/english.hwp", text)


class TestFirst5MinIntegrity(unittest.TestCase):
    def test_ids_unique(self):
        ids = [s["id"] for s in doc.FIRST_5_MIN]
        self.assertEqual(len(ids), len(set(ids)))

    def test_all_readonly(self):
        self.assertTrue(all(s["readOnly"] for s in doc.FIRST_5_MIN))

    def test_minutes_sum_to_five(self):
        self.assertEqual(sum(s["minutes"] for s in doc.FIRST_5_MIN), 5)

    def test_attach_step_has_mcp(self):
        step = next(s for s in doc.FIRST_5_MIN if s["id"] == "attach")
        blob = " ".join(step["commands"])
        self.assertIn("mcp-serve", blob)
        self.assertIn("capabilities --mcp", blob)

    def test_form_step_only_fields(self):
        step = next(s for s in doc.FIRST_5_MIN if s["id"] == "form-read")
        self.assertEqual(len(step["commands"]), 1)
        self.assertTrue(step["commands"][0].startswith("rhwp fields"))

    def test_security_step_three_axes(self):
        step = next(s for s in doc.FIRST_5_MIN if s["id"] == "security")
        blob = "\n".join(step["commands"])
        self.assertIn("hidden-text", blob)
        self.assertIn("injection", blob)
        self.assertIn("unicode", blob)


class TestJsonStdoutIsolation(unittest.TestCase):
    def test_json_stdout_is_one_object(self):
        with tempfile.TemporaryDirectory() as d:
            out = io.StringIO()
            err = io.StringIO()
            with mock.patch.object(doc.shutil, "which", return_value=None):
                with mock.patch.object(sys, "stdout", out), mock.patch.object(sys, "stderr", err):
                    code = doc.main(["--json", "--offline", "--repo-root", d])
            self.assertEqual(code, 3)
            payload = json.loads(out.getvalue())
            self.assertIsInstance(payload, dict)
            self.assertEqual(payload["tool"], doc.TOOL_NAME)
            self.assertIn("rhwp doctor", err.getvalue())
            # stdout 에 사람용 머리글이 섞이면 안 된다.
            self.assertFalse(out.getvalue().lstrip().startswith("rhwp doctor"))


if __name__ == "__main__":
    unittest.main()
