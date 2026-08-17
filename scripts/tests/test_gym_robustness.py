"""[robustness] gym 손상-강건성 감사 계약 — 결정적 손상 + 패닉/행 색출.

핵심: rhwp 는 손상 입력에 절대 패닉·행 하면 안 된다. 감사기는 코퍼스를 결정적으로
손상시켜 파싱하고, 패닉(코드 101/음수/'panicked')·행(timeout) 이 있으면 실패로 잡는다.
파싱은 목킹해 바이너리 없이 로직만 시험한다.
"""

from __future__ import annotations

import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "robustness.py"

REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "samplesTested",
    "totalSamples",
    "mutantsChecked",
    "gracefullyDegraded",
    "panics",
    "hangs",
)

ALWAYS_LABELS = (
    "truncate@25%",
    "truncate@50%",
    "truncate@75%",
    "truncate@95%",
    "flip@10%",
    "flip@50%",
    "flip@90%",
    "zero-header",
    "header-smash",
    "ole-trunc-tail",
    "ff-run",
    "utf16-nul-sprinkle",
)


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
        self.assertGreaterEqual(len(a), 12)
        for label, mut in a:
            self.assertNotEqual(mut, data, f"{label} 이 원본과 같다(무의미 변형)")

    def test_expanded_mutant_families_are_present(self):
        mod = load()
        data = bytes(range(256)) * 8
        labels = [l for l, _ in mod.deterministic_mutants(data)]
        for name in ALWAYS_LABELS:
            self.assertIn(name, labels)
        self.assertNotIn("zip-local-header-flip", labels)

        zip_data = b"PK\x03\x04" + data
        zip_labels = [l for l, _ in mod.deterministic_mutants(zip_data)]
        self.assertIn("zip-local-header-flip", zip_labels)
        flipped = dict(mod.deterministic_mutants(zip_data))["zip-local-header-flip"]
        self.assertEqual(flipped[:4], bytes(x ^ 0xFF for x in b"PK\x03\x04"))
        self.assertNotEqual(flipped, zip_data)

    def test_empty_input_has_a_deterministic_mutant(self):
        mod = load()
        self.assertEqual(mod.deterministic_mutants(b""), [("empty-to-nul", b"\0")])

    def test_empty_and_tiny_inputs_still_work(self):
        mod = load()
        shapes = (
            b"",
            b"\x00",
            b"\xff",
            b"AB",
            b"\x00" * 64,
            b"\xff" * 128,
            b"PK\x03\x04",
            b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1" + b"\x00" * 80,
        )
        for data in shapes:
            a = mod.deterministic_mutants(data)
            b = mod.deterministic_mutants(data)
            self.assertEqual(a, b, f"결정성 깨짐: {data[:16]!r}")
            self.assertGreater(len(a), 0)
            labels = []
            for label, mut in a:
                self.assertNotEqual(mut, data, f"{label} 이 원본과 같다")
                labels.append(label)
            self.assertEqual(labels, list(dict.fromkeys(labels)))

    def test_is_panic_distinguishes_crash_from_clean_failure(self):
        mod = load()
        self.assertTrue(mod.is_panic(101, ""))                       # 어보트
        self.assertTrue(mod.is_panic(0, "thread 'main' panicked"))   # 패닉 메시지
        self.assertTrue(mod.is_panic(-1073741819, ""))               # AV(음수)
        self.assertTrue(mod.is_panic(0xC0000005, ""))                # Windows AV(NTSTATUS)
        self.assertFalse(mod.is_panic(1, "오류: 유효하지 않은 파일")) # 깨끗한 실패
        self.assertFalse(mod.is_panic(255, "명시적 CLI 오류"))        # 일반 오류 코드는 패닉 아님
        self.assertFalse(mod.is_panic(0, "정상"))                    # 정상

    def test_classify_panic_and_timeout_helpers(self):
        mod = load()
        self.assertTrue(mod.classify_panic(101, ""))
        self.assertTrue(mod.classify_panic(0, "thread 'main' panicked"))
        self.assertTrue(mod.classify_panic(-1073741819, ""))
        self.assertTrue(mod.classify_panic(0xC0000005, ""))
        self.assertFalse(mod.classify_panic(1, "오류: 유효하지 않은 파일"))
        self.assertFalse(mod.classify_panic(255, "명시적 CLI 오류"))
        self.assertFalse(mod.classify_panic(0, "정상"))
        self.assertFalse(mod.classify_panic(None, ""))
        self.assertTrue(mod.classify_timeout(True))
        self.assertFalse(mod.classify_timeout(False))
        self.assertFalse(mod.classify_timeout(None))
        self.assertTrue(mod.classify_timeout(subprocess.TimeoutExpired("rhwp", 1)))
        self.assertFalse(mod.classify_timeout(RuntimeError("other")))

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

    def test_json_report_shape(self):
        mod = load()
        r = self._audit_with_probe(mod, (1, False, False, "오류"))
        self.assertEqual(set(r), set(REPORT_KEYS))
        self.assertEqual(r["kind"], "gymRobustness")
        self.assertEqual(r["schemaVersion"], "1.0")
        self.assertIsInstance(r["ok"], bool)
        self.assertIsInstance(r["samplesTested"], int)
        self.assertIsInstance(r["totalSamples"], int)
        self.assertIsInstance(r["mutantsChecked"], int)
        self.assertIsInstance(r["gracefullyDegraded"], int)
        self.assertIsInstance(r["panics"], list)
        self.assertIsInstance(r["hangs"], list)
        self.assertEqual(r["samplesTested"], 1)
        self.assertEqual(r["totalSamples"], 1)
        self.assertGreaterEqual(r["mutantsChecked"], 12)
        self.assertTrue(r["ok"])


if __name__ == "__main__":
    unittest.main()
