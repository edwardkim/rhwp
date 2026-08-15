"""[robustness] gym 손상-강건성 감사 계약 — 결정적 손상 + 패닉/행 색출.

핵심: rhwp 는 손상 입력에 절대 패닉·행 하면 안 된다. 감사기는 코퍼스를 결정적으로
손상시켜 파싱하고, 패닉(코드 101/음수/'panicked')·행(timeout) 이 있으면 실패로 잡는다.
파싱은 목킹해 바이너리 없이 로직만 시험한다.
"""

from __future__ import annotations

import importlib.util
import os
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "robustness.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_robustness", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class RobustnessTests(unittest.TestCase):
    def test_mutants_deterministic_and_nontrivial(self):
        mod = load()
        data = bytes(range(256)) * 8  # 2KB
        a = mod.deterministic_mutants(data)
        b = mod.deterministic_mutants(data)
        self.assertEqual([l for l, _ in a], [l for l, _ in b])   # 결정적(라벨 동일)
        self.assertEqual([m for _, m in a], [m for _, m in b])   # 결정적(바이트 동일)
        self.assertGreaterEqual(len(a), 8)
        for label, mut in a:
            self.assertNotEqual(mut, data, f"{label} 이 원본과 같다(무의미 변형)")

    def test_empty_input_has_a_deterministic_mutant(self):
        mod = load()
        self.assertEqual(mod.deterministic_mutants(b""), [("empty-to-nul", b"\0")])

    def test_is_panic_distinguishes_crash_from_clean_failure(self):
        mod = load()
        self.assertTrue(mod.is_panic(101, ""))                       # 어보트
        self.assertTrue(mod.is_panic(0, "thread 'main' panicked"))   # 패닉 메시지
        self.assertTrue(mod.is_panic(-1073741819, ""))               # AV(음수)
        self.assertTrue(mod.is_panic(0xC0000005, ""))                # Windows AV(NTSTATUS)
        self.assertFalse(mod.is_panic(1, "오류: 유효하지 않은 파일")) # 깨끗한 실패
        self.assertFalse(mod.is_panic(255, "명시적 CLI 오류"))        # 일반 오류 코드는 패닉 아님
        self.assertFalse(mod.is_panic(0, "정상"))                    # 정상

    def test_select_samples_deterministic_and_bounded(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            for i in range(50):
                Path(d, f"s{i:03d}.hwp").write_bytes(b"x")
            Path(d, "not-a-sample.txt").write_bytes(b"x")
            picked1, total = mod.select_samples(d, 10)
            picked2, _ = mod.select_samples(d, 10)
            self.assertEqual(total, 50)              # .txt 제외
            self.assertLessEqual(len(picked1), 10)
            self.assertEqual(picked1, picked2)       # 결정적
            self.assertTrue(all(f.endswith(".hwp") for f in picked1))

    def _audit_with_probe(self, mod, probe_result):
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(bytes(range(256)) * 8)
            mod.probe = lambda bin_path, path, timeout: probe_result
            return mod.audit("bin", d, limit=1, timeout=5)

    def test_flags_panic(self):
        mod = load()
        r = self._audit_with_probe(mod, (101, True, False, "panicked"))
        self.assertFalse(r["ok"])
        self.assertTrue(r["panics"])

    def test_flags_hang(self):
        mod = load()
        r = self._audit_with_probe(mod, (None, False, True, "timeout"))
        self.assertFalse(r["ok"])
        self.assertTrue(r["hangs"])

    def test_clean_when_graceful(self):
        mod = load()
        r = self._audit_with_probe(mod, (1, False, False, "오류"))
        self.assertTrue(r["ok"])
        self.assertEqual(r["panics"], [])
        self.assertEqual(r["hangs"], [])
        self.assertGreater(r["gracefullyDegraded"], 0)


if __name__ == "__main__":
    unittest.main()
