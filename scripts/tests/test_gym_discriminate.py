"""[discriminate] gym 판별력 감사 계약 — 음성 대조 구성 + false-pass 색출.

핵심: 각 과제는 '일 안 한 제출'(음성 대조)을 거부해야 판별력이 있다. 음성 대조는
오답 answer.json + 입력을 산출로 무편집 복사로 만든다. 음성이 통과하면 약한 오라클
(false-pass, SWE-Bench 59.4% 결함과 같은 계열)로 잡는다. 채점은 목킹해 바이너리 없이 시험.
"""

from __future__ import annotations

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "discriminate.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_discriminate", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


SAMPLE = "samples/2010-01-06.hwp"  # REPO_ROOT 기준 실재 입력


def _artifact_task():
    return {"id": "T", "input": SAMPLE, "submit": {"kind": "artifact", "files": ["out.svg"]},
            "checks": [{"op": "answer_eq", "answer": "pages",
                        "cmd": ["info", "{input}", "--json"], "path": "pageCount"}]}


def _temp_gym(root, task):
    td = os.path.join(root, "gym", "packs", "p1", "tasks")
    os.makedirs(td)
    json.dump(task, open(os.path.join(td, "T.json"), "w", encoding="utf-8"), ensure_ascii=False)
    return os.path.join(root, "gym")


class DiscriminateTests(unittest.TestCase):
    def test_build_negative_wrong_answer_and_noop_copy(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            mod.build_negative(_artifact_task(), d)
            sub = os.path.join(d, "T")
            ans = json.load(open(os.path.join(sub, "answer.json"), encoding="utf-8"))
            self.assertEqual(ans["pages"], mod.WRONG_SENTINEL)         # 오답 대조
            real = open(os.path.join(mod.REPO_ROOT, SAMPLE), "rb").read()
            self.assertEqual(open(os.path.join(sub, "out.svg"), "rb").read(), real)  # 무편집 복사

    def test_flags_false_pass_when_negative_passes(self):
        mod = load()
        mod.runner.score_task = lambda task, pack_dir, bin_path: {"pass": True}  # 음성이 통과=약한 오라클
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _artifact_task())
            r = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertFalse(r["ok"])
            self.assertIn("p1/T", r["falsePass"])

    def test_clean_when_negative_rejected(self):
        mod = load()
        mod.runner.score_task = lambda task, pack_dir, bin_path: {"pass": False}  # 음성 거부=판별력 있음
        with tempfile.TemporaryDirectory() as d:
            gym = _temp_gym(d, _artifact_task())
            r = mod.discriminate("bin", gym, os.path.join(d, "neg"))
            self.assertTrue(r["ok"])
            self.assertEqual(r["falsePass"], [])
            self.assertEqual(r["discriminating"], 1)


if __name__ == "__main__":
    unittest.main()
