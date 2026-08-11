"""[#4653] pack 구조 계약 — manifest·과제·프로파일·기준 풀이의 상시 검증.

pack 이 늘어나는 만큼 "선언만 있고 돌지 않는 과제" 의 위험이 커진다. 이 가드는
저장소에 들어온 pack 이 스스로 지켜야 할 것들을 매 CI 마다 확인한다.
"""

from __future__ import annotations

import importlib.util
import json
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

        core-cli 는 1부 유산이라 예외다(제출물이 이미 baselines 에 있다).
        """
        missing = []
        for pid in pack_ids():
            if pid == "core-cli":
                continue
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


if __name__ == "__main__":
    unittest.main()
