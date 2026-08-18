"""[#4653] pack 구조 계약 — manifest·과제·프로파일·기준 풀이의 상시 검증.

pack 이 늘어나는 만큼 "선언만 있고 돌지 않는 과제" 의 위험이 커진다. 이 가드는
저장소에 들어온 pack 이 스스로 지켜야 할 것들을 매 CI 마다 확인한다.
"""

from __future__ import annotations

import importlib.util
import json
import os
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACKS = GYM / "packs"
PROFILES = GYM / "profiles"


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def load_core():
    sys.path.insert(0, str(REPO_ROOT))
    from gym.core import checks, runner, schema  # noqa: WPS433
    return checks, runner, schema


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def pack_ids():
    return sorted(p.name for p in PACKS.iterdir() if (p / "pack.json").is_file())


class PackManifestTests(unittest.TestCase):
    def test_every_pack_manifest_is_valid(self):
        _checks, _runner, schema = load_core()
        errors = []
        for pid in pack_ids():
            schema.validate_pack(read_json(PACKS / pid / "pack.json"), str(PACKS / pid), errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_every_pack_declares_runner_identity(self):
        """점수에는 신원이 붙는다 — 어느 바이너리 기준인지 없는 pack 은 등재 불가."""
        for pid in pack_ids():
            runner_decl = read_json(PACKS / pid / "pack.json")["runner"]
            self.assertEqual(len(runner_decl["capabilitiesSha256"]), 64, pid)
            self.assertEqual(len(runner_decl["rhwpCommit"]), 40, pid)
            self.assertTrue(runner_decl["rhwpVersion"], pid)


class TaskContractTests(unittest.TestCase):
    def test_every_task_is_valid_and_uses_pinpoint_checks(self):
        _checks, _runner, schema = load_core()
        errors = []
        for pid in pack_ids():
            manifest = read_json(PACKS / pid / "pack.json")
            for path in sorted((PACKS / pid / "tasks").glob("*.json")):
                # known_commands=None: 바이너리 없이도 스키마·연산자 계약은 검사한다.
                schema.validate_task(read_json(path), manifest, None, errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_task_ids_are_unique_across_packs(self):
        seen = {}
        for pid in pack_ids():
            for path in sorted((PACKS / pid / "tasks").glob("*.json")):
                tid = read_json(path)["id"]
                self.assertNotIn(tid, seen, f"{pid}/{tid} 가 {seen.get(tid)} 와 중복")
                seen[tid] = pid
        self.assertGreaterEqual(len(seen), 40, "과제가 줄었다면 이관 사고를 의심하라")

    def test_every_new_task_ships_a_reference_solution(self):
        """기준 풀이 없는 과제는 '풀 수 있음' 이 실측되지 않은 과제다.

        [#4689] core-cli 도 이제 예외가 아니다 — 14과제 전건에 reference/ 를 채워
        저장소 단독으로 scorecard 가 재현된다. 12 pack 전체가 같은 완결성 기준을 지킨다.
        """
        missing = []
        for pid in pack_ids():
            for path in sorted((PACKS / pid / "tasks").glob("*.json")):
                tid = read_json(path)["id"]
                if not (PACKS / pid / "reference" / f"{tid}.json").is_file():
                    missing.append(f"{pid}/{tid}")
        self.assertEqual(missing, [], f"기준 풀이 없음: {missing}")

    def test_every_check_names_itself(self):
        for pid in pack_ids():
            for path in sorted((PACKS / pid / "tasks").glob("*.json")):
                task = read_json(path)
                for check in task["checks"]:
                    self.assertTrue(check.get("name"), f"{pid}/{task['id']}: 이름 없는 검사")


class TierRangeTests(unittest.TestCase):
    """[#4664] 난도 티어 1~5 — 입문(부모님)부터 보스까지. 놀이공원의 키 제한."""

    def _pack(self, axis="자동화"):
        return {"id": "p", "axis": axis}

    def test_tier_five_is_valid_and_six_is_not(self):
        _checks, _runner, schema = load_core()
        base = {"id": "X", "title": "t", "input": "samples/table-001.hwp",
                "instructions": "i", "submit": {"kind": "artifact"},
                "checks": [{"name": "c", "op": "file_exists", "file": "x"}]}
        for tier in (1, 5):
            errors = []
            schema.validate_task({**base, "tier": tier}, self._pack(), None, errors)
            self.assertEqual(errors, [], f"tier {tier} 는 유효해야 한다: {errors}")
        for tier in (0, 6):
            errors = []
            schema.validate_task({**base, "tier": tier}, self._pack(), None, errors)
            self.assertTrue(any("tier" in e for e in errors), f"tier {tier} 는 거부돼야 한다")

    def test_park_has_both_a_kiddie_ride_and_a_boss(self):
        """테마파크는 양쪽 극단을 모두 가진다 — 부모님용 tier1 과 보스 tier5."""
        tiers = set()
        for pid in pack_ids():
            for path in sorted((PACKS / pid / "tasks").glob("*.json")):
                tiers.add(read_json(path)["tier"])
        self.assertIn(1, tiers, "입문(tier 1) 놀이기구가 없다 — 부모님이 탈 것이 없다")
        self.assertIn(5, tiers, "보스(tier 5) 어트랙션이 없다 — 고난도 챌린지가 없다")


class BaselineResolveTests(unittest.TestCase):
    """[#4664] 기준 풀이의 자리표 치환 — 한 문자열의 여러 {sub:} 를 모두 바꾼다.

    다세대 계획서는 input·output 을 모두 {sub:} 로 가리킨다. 첫 하나만 바꾸면
    나머지가 리터럴로 남아 엉뚱한 이름의 파일이 생기고 다음 세대가 입력을 잃는다.
    """

    def test_multiple_sub_placeholders_all_resolve(self):
        build_baseline = load_module(
            "gym_build_baseline", REPO_ROOT / "gym" / "tools" / "build_baseline.py")
        import tempfile
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"input": "{sub:o1.hwp}", "output": "{sub:o2.hwp}"}'
            out = build_baseline.resolve(token, {"input": "in.hwp"}, sub_dir)
            self.assertNotIn("{sub:", out, "치환되지 않은 자리표가 남았다")
            self.assertIn("o1.hwp", out)
            self.assertIn("o2.hwp", out)

    def test_built_submission_is_scored_in_its_pack_directory(self):
        """생성 성공만으로 통과 처리하지 않고 같은 pack 경로를 실제 채점한다."""
        from unittest import mock

        build_baseline = load_module(
            "gym_build_baseline_score", REPO_ROOT / "gym" / "tools" / "build_baseline.py")
        task = {"id": "T01"}
        with mock.patch.object(build_baseline.runner, "score_task", return_value={"pass": True}) as score:
            failure = build_baseline.verify_built_task("/tmp/rhwp", "pack-a", task, "/tmp/sub")

        self.assertIsNone(failure)
        # 기대 경로는 구현과 같은 os.path.join 으로 — 리터럴 "/" 는 Windows 에서
        # 백슬래시 결합과 어긋나 크로스플랫폼으로 깨진다(#4689).
        score.assert_called_once_with(task, os.path.join("/tmp/sub", "pack-a"), "/tmp/rhwp")

    def test_failed_built_submission_reports_the_task(self):
        from unittest import mock

        build_baseline = load_module(
            "gym_build_baseline_failure", REPO_ROOT / "gym" / "tools" / "build_baseline.py")
        task = {"id": "T02"}
        with mock.patch.object(build_baseline.runner, "score_task",
                               return_value={"pass": False, "error": "제출 폴더 없음"}):
            failure = build_baseline.verify_built_task("/tmp/rhwp", "pack-b", task, "/tmp/sub")

        self.assertEqual(failure, "pack-b/T02: 제출 폴더 없음")

    def test_three_sub_placeholders_all_resolve(self):
        """다세대 계획서는 세 개 이상의 {sub:} 를 한 문자열에 넣는다(#5273)."""
        import tempfile

        build_baseline = load_module(
            "gym_build_baseline_triple", REPO_ROOT / "gym" / "tools" / "build_baseline.py")
        with tempfile.TemporaryDirectory() as sub_dir:
            token = '{"a": "{sub:a.hwp}", "b": "{sub:b.hwp}", "c": "{sub:c.hwp}"}'
            out = build_baseline.resolve(token, {"input": "in.hwp"}, sub_dir)
            self.assertNotIn("{sub:", out)
            self.assertIn("a.hwp", out)
            self.assertIn("b.hwp", out)
            self.assertIn("c.hwp", out)

    def test_missing_artifact_is_reported_before_score(self):
        """submit.files 가 선언한 산출이 없으면 채점 전에 부재를 보고한다."""
        import tempfile
        from unittest import mock

        build_baseline = load_module(
            "gym_build_baseline_missing", REPO_ROOT / "gym" / "tools" / "build_baseline.py")
        task = {"id": "T02", "submit": {"kind": "artifact", "files": ["edited.hwp"]}}
        with tempfile.TemporaryDirectory() as root:
            os.makedirs(os.path.join(root, "pack-b", "T02"))
            with mock.patch.object(build_baseline.runner, "score_task") as score:
                inspected = build_baseline.inspect_built_task(
                    "/tmp/rhwp", "pack-b", task, root)
            score.assert_not_called()
        self.assertEqual(inspected["kind"], "missing-artifact")
        self.assertEqual(inspected["missing"], ["edited.hwp"])
        self.assertEqual(inspected["message"], "pack-b/T02: 부재 산출: edited.hwp")

    def test_failed_score_lists_check_names(self):
        """채점 실패는 과제 ID 와 검사 이름을 남긴다. 침묵하지 않는다."""
        from unittest import mock

        build_baseline = load_module(
            "gym_build_baseline_checks", REPO_ROOT / "gym" / "tools" / "build_baseline.py")
        task = {"id": "T09"}
        result = {
            "pass": False,
            "checks": [
                {"name": "1단계 반영", "ok": False, "error": "없음"},
                {"name": "2단계 반영", "ok": True},
            ],
        }
        with mock.patch.object(build_baseline.runner, "score_task", return_value=result):
            failure = build_baseline.verify_built_task("/tmp/rhwp", "core-cli", task, "/tmp/sub")
        self.assertEqual(failure, "core-cli/T09: 1단계 반영: 없음")


class ProfileTests(unittest.TestCase):
    def test_profiles_reference_existing_packs(self):
        _checks, _runner, schema = load_core()
        errors = []
        ids = set(pack_ids())
        for path in sorted(PROFILES.glob("*.json")):
            schema.validate_profile(read_json(path), ids, errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_maintainer_profile_covers_every_pack(self):
        """전 표면 프로파일이 pack 추가를 따라가지 못하면 조용히 구멍이 생긴다."""
        maintainer = read_json(PROFILES / "maintainer.json")
        self.assertEqual(sorted(maintainer["packs"]), pack_ids())


class UnavailableReportingTests(unittest.TestCase):
    """부재는 실패가 아니다 — 요구 명령이 없는 pack 은 0점이 아니라 unavailable."""

    def test_missing_capability_reports_unavailable_not_zero(self):
        _checks, runner, _schema = load_core()
        entry = runner.score_pack("core-cli", str(GYM / "submissions" / "none"),
                                  "rhwp", available={"info"})
        self.assertEqual(entry["status"], "unavailable")
        self.assertIsNone(entry["score"])
        self.assertTrue(entry["missingCommands"])

    def test_present_capability_is_scored(self):
        _checks, runner, _schema = load_core()
        manifest = read_json(PACKS / "core-cli" / "pack.json")
        entry = runner.score_pack("core-cli", str(GYM / "submissions" / "none"),
                                  "rhwp", available=set(manifest["requires"]["commands"]))
        self.assertEqual(entry["status"], "scored")


class ExtractionPackTests(unittest.TestCase):
    """extraction pack — 읽기 추출. 새 pack·T07 복제 없음."""

    PACK = PACKS / "extraction"
    EXPECTED = [f"EX{i:02d}" for i in range(1, 29)]

    def _tasks(self):
        return [read_json(p) for p in sorted((self.PACK / "tasks").glob("EX*.json"))]

    def test_task_ids_are_ex01_to_ex28(self):
        ids = [t["id"] for t in self._tasks()]
        self.assertEqual(ids, self.EXPECTED)

    def test_every_task_has_matching_reference(self):
        for tid in self.EXPECTED:
            ref = read_json(self.PACK / "reference" / f"{tid}.json")
            self.assertEqual(ref["id"], tid)
            self.assertTrue(ref.get("steps"))

    def test_all_are_live_oracle_answers(self):
        for task in self._tasks():
            self.assertEqual(task["submit"]["kind"], "answer", task["id"])
            self.assertTrue(task["checks"], task["id"])
            for check in task["checks"]:
                self.assertIn(check["op"], {"answer_eq", "len_answer_eq"}, task["id"])
                self.assertTrue(check.get("cmd"), task["id"])
                self.assertTrue(check.get("path"), task["id"])

    def test_commands_stay_in_pack_requires(self):
        allowed = set(read_json(self.PACK / "pack.json")["requires"]["commands"])
        seen = set()
        for task in self._tasks():
            for check in task["checks"]:
                seen.add(check["cmd"][0])
        self.assertTrue(seen <= allowed, f"pack 밖 명령: {seen - allowed}")
        self.assertEqual(seen, allowed)

    def test_does_not_clone_t07_fill_fields(self):
        for task in self._tasks():
            blob = json.dumps(task, ensure_ascii=False)
            self.assertNotIn("fill-fields", blob, task["id"])
            self.assertNotIn("T07", blob, task["id"])

    def test_kind_filters_are_distinct(self):
        """날짜·금액·수량·전종은 서로 다른 과제다."""
        kinds = {}
        for task in self._tasks():
            cmd = task["checks"][0]["cmd"]
            if cmd[0] != "extract-data":
                continue
            kind = "all"
            if "--kind" in cmd:
                kind = cmd[cmd.index("--kind") + 1]
            kinds.setdefault(kind, []).append(task["id"])
        for flag in ("date", "amount", "number", "all"):
            self.assertTrue(kinds.get(flag), f"--kind {flag} 과제가 없다")

    def test_export_text_page_and_length_contracts(self):
        page_tasks = [t for t in self._tasks() if t["checks"][0]["path"] == "pageCount"]
        len_tasks = [t for t in self._tasks() if t["checks"][0]["op"] == "len_answer_eq"]
        self.assertGreaterEqual(len(page_tasks), 3)
        self.assertGreaterEqual(len(len_tasks), 2)
        p0 = next(t for t in self._tasks() if t["id"] == "EX04")
        self.assertIn("-p", p0["checks"][0]["cmd"])
        self.assertIn("0", p0["checks"][0]["cmd"])

    def test_chart_samples_are_not_all_the_same(self):
        charts = [
            t["input"] for t in self._tasks()
            if t["checks"][0]["cmd"][0] == "chart-to-csv"
        ]
        self.assertGreaterEqual(len(set(charts)), 3)

    def test_inputs_exist_in_repo(self):
        for task in self._tasks():
            self.assertTrue(
                (REPO_ROOT / task["input"]).is_file(),
                f"{task['id']} 입력 없음: {task['input']}",
            )

    def test_reference_answer_mirrors_check_cmd(self):
        for task in self._tasks():
            ref = read_json(self.PACK / "reference" / f"{task['id']}.json")
            check = task["checks"][0]
            spec = ref["steps"][0]["answer"][check["answer"]]
            self.assertEqual(spec["cmd"], check["cmd"], task["id"])
            self.assertEqual(spec["path"], check["path"], task["id"])
            if check["op"] == "len_answer_eq":
                self.assertTrue(spec.get("len"), task["id"])


class TableCsvPackTests(unittest.TestCase):
    """table-csv pack — 표 CSV 왕복. 전역 훑기 금지."""

    PACK = PACKS / "table-csv"
    EXPECTED = [f"TC{i:02d}" for i in range(1, 26)]

    def _tasks(self):
        return [read_json(p) for p in sorted((self.PACK / "tasks").glob("TC*.json"))]

    def test_task_ids_are_tc01_to_tc25(self):
        self.assertEqual([t["id"] for t in self._tasks()], self.EXPECTED)

    def test_every_task_has_matching_reference(self):
        for tid in self.EXPECTED:
            ref = read_json(self.PACK / "reference" / f"{tid}.json")
            self.assertEqual(ref["id"], tid)

    def test_editing_tasks_use_pinpoint_ops(self):
        _checks, _runner, schema = load_core()
        banned = _checks.GLOBAL_SCAN_OPS
        for task in self._tasks():
            for check in task["checks"]:
                self.assertNotIn(check["op"], banned, f"{task['id']} {check['op']}")

    def test_extract_tasks_pin_known_grid(self):
        """basic-table-01 격자는 1..12, (1,2)=7. 다른 표본은 머리 칸만 찍는다."""
        extract = [t for t in self._tasks() if t["submit"]["kind"] == "artifact"
                   and t["submit"]["files"] == ["table.csv"]]
        self.assertGreaterEqual(len(extract), 3)
        basic = [t for t in extract if "basic-table-01" in t["input"]]
        self.assertTrue(basic)
        for task in basic:
            cells = [
                (c["row"], c["col"], c["value"])
                for c in task["checks"] if c["op"] == "csv_cell_eq"
            ]
            self.assertTrue(cells, task["id"])
            for _r, _c, value in cells:
                self.assertIn(int(value), range(1, 13), task["id"])

    def test_rewrite_tasks_keep_an_untouched_cell(self):
        rewrites = [
            t for t in self._tasks()
            if t["submit"]["kind"] == "artifact" and "out.hwpx" in t["submit"]["files"]
        ]
        self.assertGreaterEqual(len(rewrites), 3)
        for task in rewrites:
            cells = [c for c in task["checks"] if c["op"] == "cell_text_eq"]
            self.assertGreaterEqual(len(cells), 2, task["id"])
            self.assertTrue(
                any("그대로" in c["name"] for c in cells),
                f"{task['id']} 가 유지 칸을 안 본다",
            )

    def test_rewrite_assets_exist_and_are_referenced(self):
        for task in self._tasks():
            if task["submit"]["kind"] != "artifact":
                continue
            if "out.hwpx" not in task["submit"].get("files", []) and "out.hwp" not in task["submit"].get("files", []):
                continue
            ref = read_json(self.PACK / "reference" / f"{task['id']}.json")
            run = ref["steps"][0]["run"]
            self.assertEqual(run[0], "csv-to-table", task["id"])
            csv_path = run[run.index("--csv") + 1]
            self.assertTrue((REPO_ROOT / csv_path).is_file(), csv_path)

    def test_bom_task_uses_utf8_bom(self):
        bom_ids = []
        for tid in self.EXPECTED:
            task = read_json(self.PACK / "tasks" / f"{tid}.json")
            if any(c["op"] == "utf8_bom" for c in task["checks"]):
                bom_ids.append(tid)
                ref = read_json(self.PACK / "reference" / f"{tid}.json")
                self.assertIn("--bom", ref["steps"][0]["run"], tid)
        self.assertIn("TC05", bom_ids)
        self.assertIn("TC06", bom_ids)

    def test_json_envelope_tasks_are_answers(self):
        for tid in ("TC07", "TC08", "TC10", "TC11", "TC12", "TC13", "TC15", "TC17",
                    "TC18", "TC21", "TC22", "TC25"):
            task = read_json(self.PACK / "tasks" / f"{tid}.json")
            self.assertEqual(task["submit"]["kind"], "answer", tid)
            self.assertEqual(task["checks"][0]["op"], "answer_eq", tid)

    def test_no_deep_contains_anywhere(self):
        for task in self._tasks():
            for check in task["checks"]:
                self.assertNotEqual(check["op"], "deep_contains", task["id"])

    def test_does_not_clone_t07(self):
        for task in self._tasks():
            self.assertNotIn("fill-fields", json.dumps(task))

    def test_runner_identity_untouched(self):
        runner = read_json(self.PACK / "pack.json")["runner"]
        self.assertEqual(len(runner["rhwpCommit"]), 40)
        self.assertEqual(len(runner["capabilitiesSha256"]), 64)


class BatchOpsPackTests(unittest.TestCase):
    """batch-ops pack — batch fill 메일머지. fill-fields(T07) 복제 아님."""

    PACK = PACKS / "batch-ops"
    EXPECTED = [f"BO{i:02d}" for i in range(1, 21)]

    def _tasks(self):
        return [read_json(p) for p in sorted((self.PACK / "tasks").glob("BO*.json"))]

    def test_task_ids_are_bo01_to_bo20(self):
        self.assertEqual([t["id"] for t in self._tasks()], self.EXPECTED)

    def test_every_task_has_matching_reference(self):
        for tid in self.EXPECTED:
            ref = read_json(self.PACK / "reference" / f"{tid}.json")
            self.assertEqual(ref["id"], tid)
            step = ref["steps"][0]
            if "run" in step:
                self.assertEqual(step["run"][:2], ["batch", "fill"], tid)
            else:
                self.assertIn("answer", step, tid)

    def test_all_are_artifact_mail_merges(self):
        artifacts = []
        answers = []
        for task in self._tasks():
            if task["submit"]["kind"] == "answer":
                answers.append(task["id"])
                continue
            artifacts.append(task["id"])
            # BO13 은 1행 최소 대량 — 1부만 제출한다.
            min_files = 1 if task["id"] == "BO13" else 2
            self.assertGreaterEqual(len(task["submit"]["files"]), min_files, task["id"])
            ops = [c["op"] for c in task["checks"]]
            self.assertIn("file_exists", ops, task["id"])
            self.assertIn("differs_from_input", ops, task["id"])
            self.assertIn("value_ge", ops, task["id"])
        self.assertGreaterEqual(len(artifacts), 8)
        self.assertGreaterEqual(len(answers), 2)

    def test_references_never_call_fill_fields(self):
        for tid in self.EXPECTED:
            ref = read_json(self.PACK / "reference" / f"{tid}.json")
            blob = json.dumps(ref, ensure_ascii=False)
            self.assertNotIn("fill-fields", blob, tid)
            self.assertNotIn("edit", blob.split(), tid)

    def test_name_field_tasks_use_field_value_filenames(self):
        named = []
        for tid in self.EXPECTED:
            ref = read_json(self.PACK / "reference" / f"{tid}.json")
            run = ref["steps"][0].get("run") or []
            if "--name-field" in run:
                named.append(tid)
                task = read_json(self.PACK / "tasks" / f"{tid}.json")
                for fname in task["submit"]["files"]:
                    self.assertNotRegex(fname, r"000\d\.", tid)
        self.assertTrue({"BO02", "BO06", "BO12", "BO15", "BO16", "BO17"}.issubset(named), named)

    def test_sequential_tasks_use_padded_names(self):
        sequential = []
        for tid in self.EXPECTED:
            ref = read_json(self.PACK / "reference" / f"{tid}.json")
            run = ref["steps"][0].get("run") or []
            if not run or "--name-field" in run:
                continue
            sequential.append(tid)
            task = read_json(self.PACK / "tasks" / f"{tid}.json")
            self.assertTrue(
                any("0001." in f for f in task["submit"]["files"]),
                tid,
            )
        self.assertGreaterEqual(len(sequential), 6)

    def test_hwp5_and_hwpx_forms_are_both_covered(self):
        forms = {t["input"] for t in self._tasks()}
        self.assertTrue(any(p.endswith(".hwpx") for p in forms))
        self.assertTrue(any(p.endswith(".hwp") and "/hwpx/" not in p.replace("\\", "/") for p in forms))

    def test_csv_and_jsonl_data_are_both_covered(self):
        kinds = set()
        for tid in self.EXPECTED:
            ref = read_json(self.PACK / "reference" / f"{tid}.json")
            run = ref["steps"][0].get("run") or []
            if "--data" not in run:
                continue
            data = run[run.index("--data") + 1]
            kinds.add(Path(data).suffix)
            self.assertTrue((REPO_ROOT / data).is_file(), data)
        self.assertEqual(kinds, {".csv", ".jsonl"})

    def test_merge_tokens_are_unique_across_new_tasks(self):
        tokens = []
        for task in self._tasks():
            if task["id"] in {"BO01"}:
                continue
            for check in task["checks"]:
                if check["op"] == "value_ge":
                    tokens.append(check["cmd"][-1])
        self.assertEqual(len(tokens), len(set(tokens)), tokens)

    def test_data_files_declare_mymsg01(self):
        for path in sorted((self.PACK / "assets").iterdir()):
            text = path.read_text(encoding="utf-8")
            self.assertIn("myMsg01", text, path.name)

    def test_does_not_introduce_new_pack_or_t07(self):
        self.assertTrue((self.PACK / "pack.json").is_file())
        self.assertEqual(read_json(self.PACK / "pack.json")["id"], "batch-ops")
        for task in self._tasks():
            self.assertNotEqual(task["id"], "T07")


class CoverageDocsTests(unittest.TestCase):
    """이번 PR 이 남긴 커버리지 문서 계약."""

    def test_canonical_coverage_doc_exists(self):
        path = REPO_ROOT / "gym" / "docs" / "coverage.md"
        self.assertTrue(path.is_file())
        text = path.read_text(encoding="utf-8")
        self.assertIn("pack×명령", text)
        self.assertIn("unusedOperators", text)
        self.assertIn("gymCoverage", text)

    def test_working_notes_exist(self):
        path = REPO_ROOT / "mydocs" / "working" / "gym_coverage.md"
        self.assertTrue(path.is_file())
        text = path.read_text(encoding="utf-8")
        self.assertIn("EX03", text)
        self.assertIn("TC02", text)
        self.assertIn("BO02", text)
        self.assertNotIn("T07 복제", text)
        fat = REPO_ROOT / "mydocs" / "working" / "gym_coverage_and_extract.md"
        self.assertTrue(fat.is_file())
        fat_text = fat.read_text(encoding="utf-8")
        self.assertIn("EX05", fat_text)
        self.assertIn("TC04", fat_text)
        self.assertIn("BO04", fat_text)
        self.assertGreater(len(fat_text.splitlines()), 200)


class ThinPackExpansionGateTests(unittest.TestCase):
    """얇은 pack 세 곳이 과제+기준풀이 짝을 유지한다."""

    def test_no_new_pack_directory(self):
        known = {
            "automation", "batch-ops", "casual-rides", "core-cli",
            "corpus-diagnostics", "expert-challenges", "extraction",
            "layout-rendering", "objects-media", "render-tree", "security",
            "self-description", "serialization", "showcase", "studio-e2e",
            "table-csv", "table-editing", "text-editing",
        }
        self.assertTrue(set(pack_ids()) <= known | set(pack_ids()))
        extra = set(pack_ids()) - known
        self.assertEqual(extra, set(), f"새 pack 금지: {extra}")

    def test_core_cli_t07_still_owns_fill_fields(self):
        t07 = read_json(PACKS / "core-cli" / "tasks" / "T07.json")
        blob = json.dumps(t07)
        self.assertIn("fill-fields", blob)
        for pid in ("extraction", "table-csv", "batch-ops"):
            for path in (PACKS / pid / "tasks").glob("*.json"):
                self.assertNotIn("fill-fields", path.read_text(encoding="utf-8"), path)

    def test_audit_tool_accepts_expanded_packs(self):
        spec = importlib.util.spec_from_file_location(
            "gym_audit_expand", GYM / "tools" / "audit.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], report)
        self.assertEqual(report["issueCount"], 0)


if __name__ == "__main__":
    unittest.main()
