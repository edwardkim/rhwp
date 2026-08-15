"""[audit] gym 정합 감사 계약 — 전-저장소 정합 불변식.

CI 강제: 실제 저장소의 전 pack 이 "그 방식"(해결 가능·고유·정합)을 지킨다. 비정합
pack(짝 기준풀이 없음·과제 ID 전역 충돌)이 들어오면 이 테스트가 red 로 막는다.
바이너리 없이 순수 파일 검사만 시험한다.
"""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "gym" / "tools" / "audit.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_audit", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _write_pack(root, pid, task_id, with_ref=True):
    pd = os.path.join(root, "packs", pid)
    os.makedirs(os.path.join(pd, "tasks"))
    os.makedirs(os.path.join(pd, "reference"))
    json.dump(
        {"schemaVersion": "1.0", "kind": "gymPack", "id": pid, "title": "t", "axis": "조회 (x)",
         "requires": {"commands": ["info"]},
         "runner": {"rhwpVersion": "0.8.4", "rhwpCommit": "a" * 40, "capabilitiesSha256": "b" * 64}},
        open(os.path.join(pd, "pack.json"), "w", encoding="utf-8"), ensure_ascii=False)
    json.dump(
        {"id": task_id, "tier": 2, "title": "t", "input": "samples/x.hwp", "instructions": "i",
         "submit": {"kind": "answer"},
         "checks": [{"op": "answer_eq", "answer": "p", "cmd": ["info", "{input}", "--json"],
                     "path": "pageCount"}]},
        open(os.path.join(pd, "tasks", f"{task_id}.json"), "w", encoding="utf-8"), ensure_ascii=False)
    if with_ref:
        json.dump(
            {"id": task_id, "steps": [{"answer": {"p": {"cmd": ["info", "{input}", "--json"],
                                                        "path": "pageCount"}}}]},
            open(os.path.join(pd, "reference", f"{task_id}.json"), "w", encoding="utf-8"),
            ensure_ascii=False)


class AuditTests(unittest.TestCase):
    def test_real_repo_all_packs_conform(self):
        report = load().audit(str(REPO_ROOT / "gym"))
        self.assertTrue(
            report["ok"],
            f"gym 정합 위반: {report['packs']} · 충돌 {report['taskIdCollisions']}")
        self.assertGreaterEqual(report["packCount"], 10)

    def test_missing_reference_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=False)
            r = load().audit(d)
            self.assertFalse(r["ok"])
            self.assertTrue(any("기준풀이" in i for p in r["packs"] for i in p["issues"]))

    def test_orphan_reference_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "X01", with_ref=True)
            os.remove(os.path.join(d, "packs", "p1", "tasks", "X01.json"))
            r = load().audit(d)
            self.assertFalse(r["ok"])
            self.assertTrue(any("고아" in i for p in r["packs"] for i in p["issues"]))

    def test_task_id_collision_across_packs_is_flagged(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "DUP", with_ref=True)
            _write_pack(d, "p2", "DUP", with_ref=True)
            r = load().audit(d)
            self.assertFalse(r["ok"])
            self.assertIn("DUP", r["taskIdCollisions"])

    def test_clean_fixture_passes(self):
        with tempfile.TemporaryDirectory() as d:
            _write_pack(d, "p1", "A01", with_ref=True)
            _write_pack(d, "p2", "B01", with_ref=True)
            self.assertTrue(load().audit(d)["ok"])


if __name__ == "__main__":
    unittest.main()
