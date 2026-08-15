"""[trajectory] gym 트라젝토리 필요성 감사 계약 — 무의미한 마지막 스텝(연극) 색출.

핵심: 다단계 과제는 마지막 스텝을 빼면(부분 트라젝토리) 채점에 실패해야 한다.
부분 트라젝토리가 통과 = 마지막 스텝이 load-bearing 아님 = 트라젝토리 연극.
조립·채점은 목킹해 바이너리 없이 로직만 시험한다.
"""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "trajectory.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_trajectory", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _temp_gym(root, steps):
    """packs/p1 에 T 과제 + steps 개 스텝짜리 reference 를 심는다."""
    tasks = os.path.join(root, "gym", "packs", "p1", "tasks")
    refs = os.path.join(root, "gym", "packs", "p1", "reference")
    os.makedirs(tasks)
    os.makedirs(refs)
    task = {"id": "T", "tier": 1, "title": "t", "input": "samples/x.hwp",
            "submit": {"kind": "artifact", "files": ["o"]}, "checks": []}
    with open(os.path.join(tasks, "T.json"), "w", encoding="utf-8") as fh:
        json.dump(task, fh)
    ref = {"id": "T", "steps": [{"run": ["a"]} for _ in range(steps)]}
    with open(os.path.join(refs, "T.json"), "w", encoding="utf-8") as fh:
        json.dump(ref, fh)
    return os.path.join(root, "gym")


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


if __name__ == "__main__":
    unittest.main()
