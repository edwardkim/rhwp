"""[fuzz_corpus] gym 코퍼스 퍼징 발견 엔진 계약 — 결정적 변형·분류·근본원인 클러스터링.

퍼징(subprocess)은 목킹해 바이너리 없이 로직만 시험한다.
"""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "fuzz_corpus.py"


def load():
    spec = importlib.util.spec_from_file_location("gym_fuzz_corpus", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FuzzCorpusTests(unittest.TestCase):
    def test_mutants_deterministic_and_nontrivial(self):
        mod = load()
        data = bytes(range(256)) * 16
        a = mod.deterministic_mutants(data)
        b = mod.deterministic_mutants(data)
        self.assertEqual(a, b)  # 결정적
        self.assertGreaterEqual(len(a), 10)
        for label, mut in a:
            self.assertNotEqual(mut, data, f"{label} 이 원본과 같다")

    def test_classify_distinguishes_panic_from_clean(self):
        mod = load()
        self.assertEqual(mod.classify(101, "thread 'main' panicked at src/x.rs:42:9")[0], "panic")
        self.assertEqual(mod.classify(101, "panicked at src/x.rs:42:9")[1], "src/x.rs:42")
        self.assertEqual(mod.classify(134, "stack overflow"), ("panic", "stack-overflow"))
        self.assertEqual(mod.classify(101, "")[0], "panic")           # 어보트 코드
        self.assertEqual(mod.classify(-1073741819, "")[0], "panic")   # AV(음수)
        self.assertEqual(mod.classify(1, "오류: 유효하지 않은 파일"), (None, None))  # 깨끗한 실패
        self.assertEqual(mod.classify(0, "정상"), (None, None))

    def test_select_samples_deterministic_bounded(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            for i in range(40):
                (root / f"s{i:03d}.hwp").write_bytes(b"x")
            (root / "note.txt").write_bytes(b"x")
            picked, total = mod.select_samples(d, 8)
            self.assertEqual(total, 40)                 # .txt 제외
            self.assertLessEqual(len(picked), 8)
            self.assertEqual(picked, mod.select_samples(d, 8)[0])  # 결정적

    def test_fuzz_clusters_panics_by_location(self):
        mod = load()
        # probe 를 목킹: cmd 별로 서로 다른 결과. 두 위치 패닉 + 한 행.
        def fake_probe(bin_path, cmd, path, timeout):
            if cmd == "a":
                return ("panic", "src/x.rs:10")
            if cmd == "b":
                return ("panic", "src/x.rs:10")  # 같은 위치(다른 명령) → 한 클러스터
            if cmd == "c":
                return ("hang", "c")
            return (None, None)
        mod.probe = fake_probe
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            samples = root / "samples"
            samples.mkdir()
            (samples / "one.hwp").write_bytes(bytes(range(256)) * 16)
            work = root / "w"
            work.mkdir()
            r = mod.fuzz("bin", str(samples), ["a", "b", "c"], limit=0, workers=2, timeout=5, work_dir=str(work))
        self.assertFalse(r["ok"])
        self.assertEqual(r["distinctPanicSites"], 1)                 # x.rs:10 한 곳으로 묶임
        self.assertEqual(r["panicClusters"][0]["location"], "src/x.rs:10")
        self.assertEqual(len(r["hangClusters"]), 1)
        self.assertEqual(r["hangClusters"][0]["command"], "c")

    def test_fuzz_clean_when_no_dos(self):
        mod = load()
        mod.probe = lambda *a, **k: (None, None)
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            samples = root / "samples"
            samples.mkdir()
            (samples / "one.hwp").write_bytes(bytes(range(256)) * 16)
            work = root / "w"
            work.mkdir()
            r = mod.fuzz("bin", str(samples), ["info"], limit=0, workers=2, timeout=5, work_dir=str(work))
        self.assertTrue(r["ok"])
        self.assertEqual(r["panicClusters"], [])
        self.assertEqual(r["hangClusters"], [])


if __name__ == "__main__":
    unittest.main()
