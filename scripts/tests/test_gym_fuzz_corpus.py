"""[fuzz_corpus] gym 코퍼스 퍼징 발견 엔진 계약 — 결정적 변형·분류·근본원인 클러스터링.

퍼징(subprocess)은 목킹해 바이너리 없이 로직만 시험한다.

확대 계약:
- 같은 입력은 같은 라벨·바이트를 낸다(무작위 없음).
- 원본과 바이트가 같은 무의미 변형은 버린다.
- 없는 바이너리·빈 코퍼스·읽기 실패에서도 엔진이 죽지 않는다.
- JSON 봉투 키가 REPORT_KEYS 와 같고 ok 는 패닉·행 부재 및 toolFailed 와 일치한다.
- 패닉은 file:line 으로 클러스터하고, 행은 명령으로 클러스터한다.
"""

from __future__ import annotations

import errno
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

TOOL = Path(__file__).resolve().parents[2] / "gym" / "tools" / "fuzz_corpus.py"

REPORT_KEYS = (
    "kind",
    "schemaVersion",
    "ok",
    "samplesTested",
    "totalSamples",
    "commands",
    "mutantsPerSample",
    "runsChecked",
    "distinctPanicSites",
    "panicClusters",
    "hangClusters",
    "unreadables",
    "probeErrors",
    "toolErrors",
    "emptyCorpus",
    "missingBin",
    "toolFailed",
    "inputShapes",
    "exit",
)

LEGACY_ALWAYS_LABELS = (
    "trunc5",
    "trunc25",
    "trunc50",
    "trunc75",
    "trunc95",
    "flip10",
    "flip30",
    "flip50",
    "flip70",
    "flip90",
    "biglen10",
    "biglen40",
    "biglen70",
)

EXPANDED_ALWAYS_LABELS = (
    "trunc1",
    "trunc10",
    "trunc99",
    "flip0",
    "flip25",
    "flip75",
    "flip99",
    "chop-last",
    "cut-first",
    "zero-header",
    "header-smash",
    "rotate-header",
    "increment-header",
    "nibble-swap-head",
    "ole-trunc-tail",
    "ole-magic-poison",
    "ole-sector-shift-poison",
    "ff-run",
    "aa-run",
    "nul-mid",
    "00-run",
    "55-run",
    "utf16-nul-sprinkle",
    "utf16-bom-inject",
    "utf8-overlong",
    "ascii-ctrl-sprinkle",
    "path-sep-sprinkle",
    "zip-magic-inject",
    "length-zero30",
    "length-one60",
    "i32-min20",
    "u16-max12",
    "reverse-prefix",
    "swap-ends",
    "high-bit-stripe",
    "low-bit-stripe",
    "xor-stride7",
    "interleave-zero-head",
    "duplicate-prefix",
    "tail-over-head",
    "invert-tail-64",
    "complement-mid-32",
    "bit-rotate-head",
    "decrement-tail",
    "slide-window-left",
    "slide-window-right",
    "repeat-mid-block",
    "odd-length-chop",
    "splice-nul-mid",
    "crlf-inject",
    "pad-eof",
    "widen-gap",
    "shrink-gap",
    "hwp3-sig-inject",
)

FAMILY_IDS = (
    "empty",
    "truncate",
    "flip",
    "length",
    "header",
    "ole",
    "run",
    "unicode",
    "zip",
    "permute",
    "stripe",
    "splice",
    "hwp3",
)

NORMAL_2K = bytes(range(256)) * 8  # 2KB, 짝수, ZIP/HWP3 서명 없음
EXCEPTION_KINDS = (
    "missing-bin",
    "empty-corpus",
    "unreadable",
    "permission",
    "timeout",
    "os-error",
    "type-error",
    "value-error",
    "decode-error",
    "invalid-timeout",
    "invalid-workers",
    "probe-error",
    "unexpected",
)


def load():
    spec = importlib.util.spec_from_file_location("gym_fuzz_corpus", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def labels_of(pairs):
    return [label for label, _ in pairs]


def as_map(pairs):
    return {label: payload for label, payload in pairs}


def _fuzz(mod, samples, commands, **kwargs):
    work = kwargs.pop("work_dir", None)
    if work is None:
        raise AssertionError("work_dir 필요")
    return mod.fuzz(
        kwargs.pop("bin_path", "bin"),
        samples,
        commands,
        kwargs.pop("limit", 0),
        kwargs.pop("workers", 2),
        kwargs.pop("timeout", 5),
        work,
    )


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


class ExpandedMutantContractTests(unittest.TestCase):
    def test_catalog_covers_declared_families(self):
        mod = load()
        catalog = mod.mutant_catalog()
        self.assertGreaterEqual(len(catalog), 40)
        ids = [row["id"] for row in catalog]
        self.assertEqual(ids, list(mod.catalog_ids()))
        self.assertEqual(len(ids), len(set(ids)))
        families = {row["family"] for row in catalog}
        for fam in FAMILY_IDS:
            self.assertIn(fam, families)
        self.assertEqual(set(mod.catalog_families()), families)
        for row in catalog:
            self.assertIn("id", row)
            self.assertIn("family", row)
            self.assertIn("when", row)
            self.assertIn("why", row)
            self.assertTrue(row["why"])

    def test_mutant_family_mapping(self):
        mod = load()
        cases = {
            "empty-to-nul": "empty",
            "trunc25": "truncate",
            "trunc5": "truncate",
            "chop-last": "truncate",
            "cut-first": "truncate",
            "odd-length-chop": "truncate",
            "shrink-gap": "truncate",
            "flip0": "flip",
            "flip90": "flip",
            "biglen10": "length",
            "length-zero30": "length",
            "length-one60": "length",
            "i32-min20": "length",
            "u16-max12": "length",
            "zero-header": "header",
            "header-smash": "header",
            "rotate-header": "header",
            "increment-header": "header",
            "nibble-swap-head": "header",
            "ole-trunc-tail": "ole",
            "ole-magic-poison": "ole",
            "ole-sector-shift-poison": "ole",
            "ole-mini-fat-poison": "ole",
            "ff-run": "run",
            "aa-run": "run",
            "nul-mid": "run",
            "00-run": "run",
            "55-run": "run",
            "utf16-nul-sprinkle": "unicode",
            "utf16-bom-inject": "unicode",
            "utf8-overlong": "unicode",
            "ascii-ctrl-sprinkle": "unicode",
            "path-sep-sprinkle": "unicode",
            "zip-local-header-flip": "zip",
            "zip-magic-inject": "zip",
            "zip-cd-magic-flip": "zip",
            "zip-eocd-flip": "zip",
            "reverse-prefix": "permute",
            "swap-ends": "permute",
            "slide-window-left": "permute",
            "slide-window-right": "permute",
            "repeat-mid-block": "permute",
            "high-bit-stripe": "stripe",
            "low-bit-stripe": "stripe",
            "xor-stride7": "stripe",
            "interleave-zero-head": "stripe",
            "duplicate-prefix": "stripe",
            "tail-over-head": "stripe",
            "invert-tail-64": "stripe",
            "complement-mid-32": "stripe",
            "bit-rotate-head": "stripe",
            "decrement-tail": "stripe",
            "splice-nul-mid": "splice",
            "crlf-inject": "splice",
            "pad-eof": "splice",
            "widen-gap": "splice",
            "even-length-pad": "splice",
            "hwp3-sig-flip": "hwp3",
            "hwp3-sig-inject": "hwp3",
            "": "other",
            "unknown-label": "other",
        }
        for label, family in cases.items():
            self.assertEqual(mod.mutant_family(label), family, label)
        self.assertEqual(mod.mutant_family(None), "other")

    def test_legacy_labels_survive_on_normal_payload(self):
        mod = load()
        labels = labels_of(mod.deterministic_mutants(NORMAL_2K))
        for name in LEGACY_ALWAYS_LABELS:
            self.assertIn(name, labels, name)
        for name in EXPANDED_ALWAYS_LABELS:
            self.assertIn(name, labels, name)
        self.assertNotIn("zip-local-header-flip", labels)
        self.assertNotIn("empty-to-nul", labels)
        self.assertIn("zip-magic-inject", labels)
        self.assertIn("hwp3-sig-inject", labels)

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

    def test_coerce_bytes_accepts_bytes_like_only(self):
        mod = load()
        self.assertEqual(mod.coerce_bytes(b"ab"), b"ab")
        self.assertEqual(mod.coerce_bytes(bytearray(b"ab")), b"ab")
        self.assertEqual(mod.coerce_bytes(memoryview(b"ab")), b"ab")
        for bad in ("ab", 12, None, ["x"], {"a": 1}):
            with self.assertRaises(TypeError):
                mod.coerce_bytes(bad)
            with self.assertRaises(TypeError):
                mod.deterministic_mutants(bad)
            with self.assertRaises(TypeError):
                mod.classify_input_shape(bad)

    def test_classify_input_shape_buckets(self):
        mod = load()
        self.assertEqual(mod.classify_input_shape(b""), "empty")
        self.assertEqual(mod.classify_input_shape(b"\x00"), "tiny")
        self.assertEqual(mod.classify_input_shape(b"\x00" * 64), "tiny")
        self.assertEqual(mod.classify_input_shape(b"\x00" * 65), "normal")
        self.assertEqual(mod.classify_input_shape(NORMAL_2K), "normal")
        self.assertEqual(mod.classify_input_shape(b"\x00" * mod.HUGE_MIN), "huge")

    def test_header_smash_uses_fixed_pattern(self):
        mod = load()
        payload = as_map(mod.deterministic_mutants(NORMAL_2K))["header-smash"]
        self.assertTrue(payload.startswith(mod.HEADER_SMASH_PAT))
        self.assertEqual(payload[64:], NORMAL_2K[64:])

    def test_zero_header_clears_first_512(self):
        mod = load()
        payload = as_map(mod.deterministic_mutants(NORMAL_2K))["zero-header"]
        self.assertEqual(payload[:512], b"\x00" * 512)
        self.assertEqual(payload[512:], NORMAL_2K[512:])

    def test_ole_magic_poison_xors_signature(self):
        mod = load()
        payload = as_map(mod.deterministic_mutants(NORMAL_2K))["ole-magic-poison"]
        expected = bytes(x ^ 0xFF for x in mod.OLE_MAGIC)
        self.assertEqual(payload[:8], expected)
        self.assertEqual(payload[8:], NORMAL_2K[8:])

    def test_ole_sector_and_minifat_offsets(self):
        mod = load()
        mapped = as_map(mod.deterministic_mutants(NORMAL_2K))
        self.assertEqual(mapped["ole-sector-shift-poison"][30:32], b"\xff\xff")
        self.assertEqual(mapped["ole-mini-fat-poison"][60:64], b"\xff\xff\xff\xff")
        tiny = as_map(mod.deterministic_mutants(b"\x00" * 20))
        self.assertNotIn("ole-sector-shift-poison", tiny)
        self.assertNotIn("ole-mini-fat-poison", tiny)

    def test_length_bombs_are_little_endian_constants(self):
        mod = load()
        mapped = as_map(mod.deterministic_mutants(NORMAL_2K))
        n = len(NORMAL_2K)
        bomb10 = min(n - 4, n * 10 // 100)
        self.assertEqual(mapped["biglen10"][bomb10 : bomb10 + 4], b"\xff\xff\xff\x7f")
        zero30 = min(n - 4, n * 30 // 100)
        self.assertEqual(mapped["length-zero30"][zero30 : zero30 + 4], b"\x00\x00\x00\x00")
        one60 = min(n - 4, n * 60 // 100)
        self.assertEqual(mapped["length-one60"][one60 : one60 + 4], b"\x01\x00\x00\x00")
        i32 = min(n - 4, n * 20 // 100)
        self.assertEqual(mapped["i32-min20"][i32 : i32 + 4], b"\x00\x00\x00\x80")
        self.assertEqual(mapped["u16-max12"][12:14], b"\xff\xff")

    def test_run_families_overwrite_expected_windows(self):
        mod = load()
        mapped = as_map(mod.deterministic_mutants(NORMAL_2K))
        n = len(NORMAL_2K)
        start = n // 3
        run = min(128, n - start)
        self.assertEqual(mapped["ff-run"][start : start + run], b"\xff" * run)
        aa_n = max(1, n // 4)
        self.assertEqual(mapped["aa-run"][:aa_n], b"\xaa" * aa_n)
        mid = n // 2
        nul_n = min(64, n - mid)
        self.assertEqual(mapped["nul-mid"][mid : mid + nul_n], b"\x00" * nul_n)
        two_third = (n * 2) // 3
        zero_n = min(64, n - two_third)
        self.assertEqual(mapped["00-run"][two_third : two_third + zero_n], b"\x00" * zero_n)
        five_n = max(1, n // 5)
        self.assertEqual(mapped["55-run"][:five_n], b"\x55" * five_n)

    def test_unicode_and_control_injections(self):
        mod = load()
        mapped = as_map(mod.deterministic_mutants(NORMAL_2K))
        self.assertEqual(mapped["utf16-bom-inject"][:2], b"\xff\xfe")
        over_at = len(NORMAL_2K) // 5
        self.assertEqual(mapped["utf8-overlong"][over_at : over_at + 2], b"\xc0\x80")
        n = len(NORMAL_2K)
        ctrl = mapped["ascii-ctrl-sprinkle"]
        for pct in (15, 35, 55, 75):
            self.assertEqual(ctrl[n * pct // 100], 0x01)
        seps = mapped["path-sep-sprinkle"]
        self.assertEqual(seps[n * 18 // 100], 0x2F)
        self.assertEqual(seps[n * 42 // 100], 0x5C)

    def test_permute_and_stripe_contracts(self):
        mod = load()
        mapped = as_map(mod.deterministic_mutants(NORMAL_2K))
        self.assertEqual(mapped["reverse-prefix"][:16], bytes(reversed(NORMAL_2K[:16])))
        self.assertEqual(mapped["swap-ends"][:8], NORMAL_2K[-8:])
        self.assertEqual(mapped["swap-ends"][-8:], NORMAL_2K[:8])
        self.assertEqual(mapped["slide-window-left"][:32], NORMAL_2K[1:32] + NORMAL_2K[:1])
        self.assertEqual(mapped["slide-window-right"][:32], NORMAL_2K[31:32] + NORMAL_2K[:31])
        self.assertEqual(mapped["duplicate-prefix"][8:16], NORMAL_2K[:8])
        self.assertEqual(mapped["tail-over-head"][:16], NORMAL_2K[-16:])
        self.assertEqual(mapped["high-bit-stripe"][0] & 0x80, 0x80)
        self.assertEqual(mapped["low-bit-stripe"][0], NORMAL_2K[0] ^ 0x01)
        self.assertEqual(mapped["xor-stride7"][0], NORMAL_2K[0] ^ 0xA5)
        self.assertEqual(mapped["interleave-zero-head"][1], 0)
        self.assertEqual(mapped["increment-header"][0], (NORMAL_2K[0] + 1) & 0xFF)
        self.assertEqual(mapped["decrement-tail"][-1], (NORMAL_2K[-1] - 1) & 0xFF)
        self.assertEqual(mapped["bit-rotate-head"][0], ((NORMAL_2K[0] << 1) | (NORMAL_2K[0] >> 7)) & 0xFF)
        self.assertEqual(mapped["nibble-swap-head"][0], ((NORMAL_2K[0] & 0x0F) << 4) | ((NORMAL_2K[0] & 0xF0) >> 4))
        self.assertEqual(mapped["invert-tail-64"][-1], NORMAL_2K[-1] ^ 0xFF)
        self.assertEqual(mapped["complement-mid-32"][len(NORMAL_2K) // 2], NORMAL_2K[len(NORMAL_2K) // 2] ^ 0xFF)
        self.assertEqual(mapped["rotate-header"][:8], NORMAL_2K[1:8] + NORMAL_2K[:1])

    def test_splice_grows_and_shrink_shortens(self):
        mod = load()
        mapped = as_map(mod.deterministic_mutants(NORMAL_2K))
        n = len(NORMAL_2K)
        self.assertEqual(len(mapped["splice-nul-mid"]), n + 16)
        self.assertEqual(mapped["splice-nul-mid"][n // 2 : n // 2 + 16], b"\x00" * 16)
        self.assertEqual(len(mapped["crlf-inject"]), n + 2)
        self.assertEqual(mapped["crlf-inject"][n // 2 : n // 2 + 2], b"\r\n")
        self.assertEqual(mapped["pad-eof"][-1:], b"\x1a")
        self.assertEqual(len(mapped["widen-gap"]), n + 4)
        self.assertEqual(len(mapped["shrink-gap"]), n - 4)
        self.assertEqual(len(mapped["chop-last"]), n - 1)
        self.assertEqual(len(mapped["cut-first"]), n - 1)
        self.assertEqual(mapped["cut-first"], NORMAL_2K[1:])
        self.assertEqual(len(mapped["odd-length-chop"]) % 2, 1)
        self.assertNotIn("even-length-pad", mapped)

    def test_odd_payload_gets_even_pad_not_odd_chop(self):
        mod = load()
        data = b"\x11" * 65
        labels = labels_of(mod.deterministic_mutants(data))
        self.assertIn("even-length-pad", labels)
        self.assertNotIn("odd-length-chop", labels)
        mapped = as_map(mod.deterministic_mutants(data))
        self.assertEqual(mapped["even-length-pad"], data + b"\x00")

    def test_zip_conditional_families(self):
        mod = load()
        local_only = b"xxxx" + b"PK\x03\x04" + b"yyyy" * 20
        local_map = as_map(mod.deterministic_mutants(local_only))
        self.assertIn("zip-local-header-flip", local_map)
        self.assertNotIn("zip-magic-inject", local_map)
        self.assertNotIn("zip-cd-magic-flip", local_map)
        self.assertNotIn("zip-eocd-flip", local_map)
        idx = local_only.find(b"PK\x03\x04")
        self.assertEqual(
            local_map["zip-local-header-flip"][idx : idx + 4],
            bytes(x ^ 0xFF for x in b"PK\x03\x04"),
        )

        full = b"PK\x03\x04" + b"body" * 8 + b"PK\x01\x02" + b"cd" * 4 + b"PK\x05\x06" + b"end" * 4
        full_map = as_map(mod.deterministic_mutants(full))
        self.assertIn("zip-local-header-flip", full_map)
        self.assertIn("zip-cd-magic-flip", full_map)
        self.assertIn("zip-eocd-flip", full_map)
        self.assertNotIn("zip-magic-inject", full_map)
        cd = full.find(b"PK\x01\x02")
        eocd = full.find(b"PK\x05\x06")
        self.assertEqual(full_map["zip-cd-magic-flip"][cd : cd + 4], bytes(x ^ 0xFF for x in b"PK\x01\x02"))
        self.assertEqual(full_map["zip-eocd-flip"][eocd : eocd + 4], bytes(x ^ 0xFF for x in b"PK\x05\x06"))

    def test_hwp3_conditional_families(self):
        mod = load()
        inject = as_map(mod.deterministic_mutants(NORMAL_2K))
        self.assertIn("hwp3-sig-inject", inject)
        self.assertTrue(inject["hwp3-sig-inject"].startswith(mod.HWP3_SIG))
        signed = mod.HWP3_SIG + b"\x00" * 80
        flipped = as_map(mod.deterministic_mutants(signed))
        self.assertIn("hwp3-sig-flip", flipped)
        self.assertNotIn("hwp3-sig-inject", flipped)
        expected = bytes(x ^ 0xFF for x in mod.HWP3_SIG[:4]) + mod.HWP3_SIG[4:]
        self.assertEqual(flipped["hwp3-sig-flip"][: len(mod.HWP3_SIG)], expected)

    def test_huge_input_skips_growing_splices(self):
        mod = load()
        huge = bytes([0x5A]) * mod.HUGE_MIN
        labels = labels_of(mod.deterministic_mutants(huge))
        for name in ("splice-nul-mid", "crlf-inject", "pad-eof", "widen-gap", "even-length-pad"):
            self.assertNotIn(name, labels, name)
        for name in ("zero-header", "header-smash", "trunc50", "flip50", "shrink-gap"):
            self.assertIn(name, labels, name)
        self.assertEqual(mod.classify_input_shape(huge), "huge")

    def test_tiny_ole_trunc_tail_plants_magic_prefix(self):
        mod = load()
        data = b"ABCD"
        payload = as_map(mod.deterministic_mutants(data))["ole-trunc-tail"]
        self.assertTrue(payload.endswith(mod.OLE_MAGIC[: min(4, len(data))]))

    def test_utf16_sprinkle_uses_even_offsets(self):
        mod = load()
        payload = as_map(mod.deterministic_mutants(NORMAL_2K))["utf16-nul-sprinkle"]
        n = len(NORMAL_2K)
        for pct in (20, 40, 60, 80):
            pos = min(n - 2, n * pct // 100) & ~1
            self.assertEqual(payload[pos : pos + 2], b"\x00\x00")
            self.assertEqual(pos % 2, 0)

    def test_legacy_trunc_and_flip_byte_contracts(self):
        mod = load()
        mapped = as_map(mod.deterministic_mutants(NORMAL_2K))
        n = len(NORMAL_2K)
        self.assertEqual(mapped["trunc25"], NORMAL_2K[: max(1, n * 25 // 100)])
        self.assertEqual(mapped["trunc50"], NORMAL_2K[: max(1, n * 50 // 100)])
        pos = min(n - 1, n * 30 // 100)
        expected = bytearray(NORMAL_2K)
        expected[pos] ^= 0xFF
        self.assertEqual(mapped["flip30"], bytes(expected))

    def test_module_constants_match_test_contract(self):
        mod = load()
        self.assertEqual(mod.LEGACY_ALWAYS_LABELS, LEGACY_ALWAYS_LABELS)
        self.assertEqual(mod.EXPANDED_ALWAYS_LABELS, EXPANDED_ALWAYS_LABELS)
        self.assertEqual(mod.REPORT_KIND, "gymFuzzCorpus")
        self.assertEqual(mod.SCHEMA_VERSION, "1.0")
        self.assertEqual(mod.TINY_MAX, 64)
        self.assertEqual(mod.HUGE_MIN, 1_048_576)
        self.assertEqual(mod.OLE_MAGIC, b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")
        self.assertEqual(mod.ZIP_LOCAL, b"PK\x03\x04")
        self.assertEqual(mod.SAMPLE_EXTS, (".hwp", ".hwpx", ".hml"))
        self.assertEqual(mod.EXIT_OK, 0)
        self.assertEqual(mod.EXIT_DOS, 1)
        self.assertEqual(mod.EXIT_TOOL_FAILED, 2)


class ClassifyAndSelectTests(unittest.TestCase):
    def test_classify_accepts_none_and_non_str_err(self):
        mod = load()
        self.assertEqual(mod.classify(0, None), (None, None))
        self.assertEqual(mod.classify(0, 12345), (None, None))
        self.assertEqual(mod.classify(101, None)[0], "panic")
        self.assertEqual(mod.classify(101, 0)[0], "panic")

    def test_classify_panic_markers_without_location(self):
        mod = load()
        for marker in (
            "core dumped",
            "fatal runtime error",
            "SIGSEGV",
            "access violation",
            "Segmentation Fault",
        ):
            # 위치 정규식이 없으면 어보트 코드/표식 버킷. 깨끗한 1 은 패닉이 아니다.
            kind, bucket = mod.classify(0, marker)
            self.assertIsNone(kind, marker)
            self.assertIsNone(bucket, marker)
        self.assertEqual(mod.classify(101, "fatal runtime error")[0], "panic")
        self.assertTrue(mod.is_panic_code(101, ""))
        self.assertFalse(mod.is_panic_code(1, "오류"))
        self.assertFalse(mod.is_panic_code(0, "정상"))

    def test_classify_timeout_strings_and_os_errnos(self):
        mod = load()
        self.assertTrue(mod.classify_timeout(True))
        self.assertFalse(mod.classify_timeout(False))
        self.assertFalse(mod.classify_timeout(None))
        self.assertTrue(mod.classify_timeout(subprocess.TimeoutExpired("rhwp", 1)))
        self.assertTrue(mod.classify_timeout(TimeoutError("late")))
        self.assertTrue(mod.classify_timeout("Deadline Exceeded"))
        self.assertTrue(mod.classify_timeout("process timed out"))
        self.assertTrue(mod.classify_timeout("TIME-OUT"))
        self.assertFalse(mod.classify_timeout("ok"))
        self.assertFalse(mod.classify_timeout(RuntimeError("other")))
        timed = OSError(getattr(errno, "ETIMEDOUT", 110), "late")
        timed.errno = getattr(errno, "ETIMEDOUT", 110)
        self.assertTrue(mod.classify_timeout(timed))
        other = OSError(errno.ENOENT, "missing")
        other.errno = errno.ENOENT
        self.assertFalse(mod.classify_timeout(other))
        self.assertFalse(mod.classify_timeout(ValueError("timeout in name only")))

    def test_classify_probe_outcome_matrix(self):
        mod = load()
        self.assertEqual(mod.classify_probe_outcome("hang", "info"), "hang")
        self.assertEqual(mod.classify_probe_outcome("panic", "src/x.rs:1"), "panic")
        self.assertEqual(mod.classify_probe_outcome("error", "missing-bin"), "error")
        self.assertEqual(mod.classify_probe_outcome(None, None), "clean")
        self.assertEqual(mod.classify_probe_outcome("ok", None), "clean")
        self.assertEqual(mod.classify_probe_outcome("clean", None), "clean")
        self.assertEqual(mod.classify_probe_outcome("weird", "permission"), "error")

    def test_select_samples_includes_hwpx_and_hml(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "a.hwp").write_bytes(b"x")
            Path(d, "b.hwpx").write_bytes(b"x")
            Path(d, "c.hml").write_bytes(b"x")
            Path(d, "d.HWP").write_bytes(b"x")
            Path(d, "note.md").write_bytes(b"x")
            picked, total = mod.select_samples(d, 0)
            self.assertEqual(total, 4)
            self.assertEqual(picked, ["a.hwp", "b.hwpx", "c.hml", "d.HWP"])

    def test_select_samples_oserror_and_bad_limit(self):
        mod = load()
        missing = os.path.join(tempfile.gettempdir(), "gym-fuzz-missing-dir-does-not-exist")
        picked, total = mod.select_samples(missing, 8)
        self.assertEqual(picked, [])
        self.assertEqual(total, 0)
        with tempfile.TemporaryDirectory() as d:
            Path(d, "a.hwp").write_bytes(b"x")
            Path(d, "b.hwp").write_bytes(b"x")
            all_picked, count = mod.select_samples(d, 0)
            self.assertEqual(all_picked, ["a.hwp", "b.hwp"])
            self.assertEqual(count, 2)
            empty2, count2 = mod.select_samples(d, "bad")
            self.assertEqual(empty2, ["a.hwp", "b.hwp"])
            self.assertEqual(count2, 2)
            all2, _ = mod.select_samples(d, 99)
            self.assertEqual(all2, ["a.hwp", "b.hwp"])

    def test_is_sample_name_and_parse_commands(self):
        mod = load()
        self.assertTrue(mod.is_sample_name("x.hwp"))
        self.assertTrue(mod.is_sample_name("X.HWPX"))
        self.assertTrue(mod.is_sample_name("y.Hml"))
        self.assertFalse(mod.is_sample_name("note.txt"))
        self.assertFalse(mod.is_sample_name(""))
        self.assertFalse(mod.is_sample_name(None))
        self.assertEqual(mod.parse_commands(None), list(mod.DEFAULT_COMMANDS))
        self.assertEqual(mod.parse_commands("info, export-text, info"), ["info", "export-text"])
        self.assertEqual(mod.parse_commands(["a", " a ", "", "b"]), ["a", "b"])
        self.assertEqual(mod.parse_commands(" , , "), list(mod.DEFAULT_COMMANDS))

    def test_normalize_limit_timeout_workers(self):
        mod = load()
        self.assertEqual(mod.normalize_limit(8), 8)
        self.assertEqual(mod.normalize_limit("3"), 3)
        self.assertEqual(mod.normalize_limit(-4), 0)
        self.assertEqual(mod.normalize_limit(None), 0)
        self.assertEqual(mod.normalize_limit("nope"), 0)
        self.assertEqual(mod.normalize_timeout(8), 8)
        self.assertEqual(mod.normalize_timeout(0), 0)
        self.assertEqual(mod.normalize_timeout(-1), 0)
        self.assertEqual(mod.normalize_timeout("nope"), 0)
        self.assertEqual(mod.normalize_workers(8), 8)
        self.assertEqual(mod.normalize_workers(0), 1)
        self.assertEqual(mod.normalize_workers(-2), 1)
        self.assertEqual(mod.normalize_workers("nope"), 1)
        self.assertEqual(mod.normalize_workers(None), 1)

    def test_stride_selection_is_stable_across_limits(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            for i in range(20):
                Path(d, f"{i:02d}.hwp").write_bytes(b"x")
            five, total = mod.select_samples(d, 5)
            ten, _ = mod.select_samples(d, 10)
            self.assertEqual(total, 20)
            self.assertEqual(len(five), 5)
            self.assertEqual(len(ten), 10)
            self.assertEqual(mod.select_samples(d, 5)[0], five)


class ExceptionPathTests(unittest.TestCase):
    def test_exception_kind_catalog_and_context(self):
        mod = load()
        self.assertEqual(tuple(mod.EXCEPTION_KINDS), EXCEPTION_KINDS)
        self.assertEqual(mod.exception_kind(None), "unexpected")
        self.assertEqual(mod.exception_kind(FileNotFoundError("x"), "probe"), "missing-bin")
        self.assertEqual(mod.exception_kind(FileNotFoundError("x"), "find-bin"), "missing-bin")
        self.assertEqual(mod.exception_kind(FileNotFoundError("x"), "read"), "unreadable")
        self.assertEqual(mod.exception_kind(FileNotFoundError("x"), "select"), "empty-corpus")
        self.assertEqual(mod.exception_kind(PermissionError("x"), "probe"), "permission")
        self.assertEqual(mod.exception_kind(PermissionError("x"), "read"), "unreadable")
        self.assertEqual(mod.exception_kind(subprocess.TimeoutExpired("rhwp", 1)), "timeout")
        self.assertEqual(mod.exception_kind(TimeoutError()), "timeout")
        self.assertEqual(mod.exception_kind(UnicodeDecodeError("utf-8", b"x", 0, 1, "bad")), "decode-error")
        self.assertEqual(mod.exception_kind(TypeError("t")), "type-error")
        self.assertEqual(mod.exception_kind(ValueError("v")), "value-error")
        self.assertEqual(mod.exception_kind(OSError(errno.EIO, "io")), "os-error")
        self.assertEqual(mod.exception_kind(RuntimeError("x")), "unexpected")
        self.assertTrue(mod.is_fatal_exception(KeyboardInterrupt()))
        self.assertTrue(mod.is_fatal_exception(SystemExit(1)))
        self.assertTrue(mod.is_fatal_exception(MemoryError()))
        self.assertFalse(mod.is_fatal_exception(RuntimeError("x")))

    def test_read_sample_missing_and_success(self):
        mod = load()
        data, err = mod.read_sample(os.path.join(tempfile.gettempdir(), "no-such-fuzz.hwp"))
        self.assertIsNone(data)
        self.assertIsInstance(err, str)
        self.assertIn("unreadable", err)
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "ok.hwp")
            Path(path).write_bytes(b"abc")
            payload, err2 = mod.read_sample(path)
            self.assertEqual(payload, b"abc")
            self.assertIsNone(err2)

    def test_write_mutant_typeerror_and_oserror(self):
        mod = load()
        self.assertTrue(mod.write_mutant("ignored", "not-bytes").startswith("type-error"))
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "mut.hwp")
            self.assertIsNone(mod.write_mutant(path, b"xyz"))
            self.assertEqual(Path(path).read_bytes(), b"xyz")
            self.assertIsNone(mod.write_mutant(path, bytearray(b"zz")))
            missing_parent = os.path.join(d, "no-dir", "mut.hwp")
            err = mod.write_mutant(missing_parent, b"x")
            self.assertIsInstance(err, str)
            self.assertTrue(err)

    def test_probe_invalid_timeout_and_missing_bin(self):
        mod = load()
        self.assertEqual(mod.probe("bin", "info", "x.hwp", 0), ("error", "invalid-timeout"))
        self.assertEqual(mod.probe("bin", "info", "x.hwp", -3), ("error", "invalid-timeout"))
        self.assertEqual(mod.probe("", "info", "x.hwp", 5), ("error", "missing-bin"))
        self.assertEqual(mod.probe(None, "info", "x.hwp", 5), ("error", "missing-bin"))
        self.assertEqual(mod.probe("bin", "", "x.hwp", 5), ("error", "value-error"))

    def test_probe_oserror_is_not_panic(self):
        mod = load()
        kind, bucket = mod.probe(
            os.path.join(tempfile.gettempdir(), "no-such-rhwp-bin-xyz"),
            "info",
            os.path.join(tempfile.gettempdir(), "no-such.hwp"),
            2,
        )
        self.assertEqual(kind, "error")
        self.assertEqual(bucket, "missing-bin")
        self.assertEqual(mod.classify_probe_outcome(kind, bucket), "error")

    def test_probe_timeout_expired_is_hang(self):
        mod = load()

        def boom(*_a, **_k):
            raise subprocess.TimeoutExpired("rhwp", 1)

        with mock.patch("subprocess.run", side_effect=boom):
            kind, bucket = mod.probe("rhwp", "info", "x.hwp", 1)
        self.assertEqual(kind, "hang")
        self.assertEqual(bucket, "info")

    def test_probe_unexpected_exception_is_error(self):
        mod = load()

        def boom(*_a, **_k):
            raise RuntimeError("broken pipe")

        with mock.patch("subprocess.run", side_effect=boom):
            kind, bucket = mod.probe("rhwp", "info", "x.hwp", 1)
        self.assertEqual(kind, "error")
        self.assertIn(bucket, ("unexpected", "probe-error", "os-error"))

    def test_probe_permission_is_error(self):
        mod = load()

        def boom(*_a, **_k):
            raise PermissionError("denied")

        with mock.patch("subprocess.run", side_effect=boom):
            kind, bucket = mod.probe("rhwp", "info", "x.hwp", 1)
        self.assertEqual((kind, bucket), ("error", "permission"))

    def test_probe_success_path_classifies_panic_from_output(self):
        mod = load()

        class Fake:
            returncode = 0
            stdout = b""
            stderr = b"thread 'main' panicked at src/x.rs:10:1"

        with mock.patch("subprocess.run", return_value=Fake()):
            kind, bucket = mod.probe("rhwp", "info", "x.hwp", 1)
        self.assertEqual(kind, "panic")
        self.assertEqual(bucket, "src/x.rs:10")

    def test_probe_convert_appends_output_path(self):
        mod = load()
        seen = {}

        class Fake:
            returncode = 0
            stdout = b"ok"
            stderr = b""

        def fake_run(args, **_k):
            seen["args"] = list(args)
            return Fake()

        with mock.patch("subprocess.run", side_effect=fake_run):
            kind, bucket = mod.probe("rhwp", "convert", "mut.hwp", 1)
        self.assertEqual((kind, bucket), (None, None))
        self.assertEqual(seen["args"][-1], "mut.hwp.out.hwpx")

    def test_find_bin_safe_missing_and_empty(self):
        mod = load()
        path, err = mod.find_bin_safe(os.path.join(tempfile.gettempdir(), "no-such-rhwp-xyz"))
        self.assertIsNone(path)
        self.assertIsInstance(err, str)
        self.assertIn("missing-bin", err)
        with mock.patch.object(mod.runner, "find_bin", return_value=""):
            path2, err2 = mod.find_bin_safe("x")
        self.assertIsNone(path2)
        self.assertIn("empty-path", err2)

    def test_find_bin_safe_swallows_lookup_exception(self):
        mod = load()
        with mock.patch.object(mod.runner, "find_bin", side_effect=FileNotFoundError("gone")):
            path, err = mod.find_bin_safe("x")
        self.assertIsNone(path)
        self.assertIn("missing-bin", err)

    def test_fuzz_missing_bin_path_is_tool_failure(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            work = os.path.join(d, "w")
            os.mkdir(work)
            r = mod.fuzz("", d, ["info"], 0, 1, 5, work)
        self.assertFalse(r["ok"])
        self.assertTrue(r["missingBin"])
        self.assertTrue(r["toolFailed"])
        self.assertEqual(r["exit"], mod.EXIT_TOOL_FAILED)
        self.assertTrue(r["toolErrors"])
        self.assertEqual(mod.validate_report(r), [])

    def test_fuzz_empty_corpus_is_not_dos(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "note.txt").write_text("x", encoding="utf-8")
            work = os.path.join(d, "w")
            os.mkdir(work)
            r = mod.fuzz("bin", d, ["info"], 0, 1, 5, work)
        self.assertTrue(r["ok"])
        self.assertTrue(r["emptyCorpus"])
        self.assertFalse(r["toolFailed"])
        self.assertEqual(r["samplesTested"], 0)
        self.assertEqual(r["totalSamples"], 0)
        self.assertEqual(r["panicClusters"], [])
        self.assertEqual(r["exit"], mod.EXIT_OK)
        self.assertEqual(mod.validate_report(r), [])

    def test_fuzz_missing_samples_dir_does_not_raise(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            work = os.path.join(d, "w")
            os.mkdir(work)
            missing = os.path.join(d, "no-samples")
            r = mod.fuzz("bin", missing, ["info"], 8, 1, 5, work)
        self.assertIsInstance(r, dict)
        self.assertEqual(r["samplesTested"], 0)
        self.assertTrue(r["toolErrors"] or r["emptyCorpus"])
        self.assertEqual(r["panicClusters"], [])
        self.assertEqual(r["hangClusters"], [])

    def test_fuzz_records_unreadable_samples(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "ok.hwp").write_bytes(NORMAL_2K)
            Path(d, "bad.hwp").write_bytes(NORMAL_2K)
            real_read = mod.read_sample

            def wrapped(path):
                if path.endswith("bad.hwp"):
                    return None, "unreadable: PermissionError: denied"
                return real_read(path)

            mod.read_sample = wrapped
            mod.probe = lambda *_a, **_k: (None, None)
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 0, 1, 5, work)
        self.assertTrue(report["ok"])
        self.assertEqual(report["samplesTested"], 2)
        self.assertEqual(len(report["unreadables"]), 1)
        self.assertIn("bad.hwp", report["unreadables"][0])
        self.assertGreater(report["runsChecked"], 0)
        self.assertEqual(report["inputShapes"]["normal"], 1)

    def test_fuzz_records_write_errors(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.write_mutant = lambda *_a, **_k: "os-error: OSError: disk full"
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 1, 1, 5, work)
        self.assertTrue(report["ok"])
        self.assertEqual(report["runsChecked"], 0)
        self.assertGreater(len(report["probeErrors"]), 0)
        self.assertTrue(any("disk full" in item for item in report["probeErrors"]))

    def test_fuzz_records_probe_error_heads(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = lambda *_a, **_k: ("error", "missing-bin")
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 1, 1, 5, work)
        self.assertFalse(report["ok"])
        self.assertTrue(report["missingBin"])
        self.assertTrue(report["toolFailed"])
        self.assertEqual(report["panicClusters"], [])
        self.assertEqual(report["hangClusters"], [])
        self.assertGreater(len(report["probeErrors"]), 0)
        self.assertEqual(report["exit"], mod.EXIT_TOOL_FAILED)

    def test_fuzz_probe_raising_is_caught(self):
        mod = load()

        def boom(*_a, **_k):
            raise RuntimeError("probe exploded")

        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = boom
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 1, 1, 5, work)
        self.assertTrue(report["ok"])
        self.assertTrue(any("probe exploded" in item or "unexpected" in item for item in report["probeErrors"]))

    def test_fuzz_mutant_typeerror_is_unreadable(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.deterministic_mutants = lambda *_a, **_k: (_ for _ in ()).throw(TypeError("bad"))
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 1, 1, 5, work)
        self.assertTrue(report["ok"])
        self.assertTrue(any("type-error" in item or "TypeError" in item for item in report["unreadables"]))
        self.assertEqual(report["runsChecked"], 0)

    def test_empty_report_validates(self):
        mod = load()
        report = mod.empty_report(["info"])
        self.assertEqual(mod.validate_report(report), [])
        self.assertTrue(report["ok"])
        self.assertEqual(set(report), set(REPORT_KEYS))
        self.assertEqual(set(report), set(mod.REPORT_KEYS))
        self.assertEqual(report["inputShapes"], {"empty": 0, "tiny": 0, "normal": 0, "huge": 0})
        self.assertEqual(report["commands"], ["info"])

    def test_validate_report_detects_schema_breaks(self):
        mod = load()
        self.assertEqual(mod.validate_report("nope"), ["report-not-dict"])
        report = mod.empty_report()
        report.pop("hangClusters")
        issues = mod.validate_report(report)
        self.assertTrue(any(item.startswith("missing:") for item in issues))
        report = mod.empty_report()
        report["extraKey"] = 1
        issues = mod.validate_report(report)
        self.assertTrue(any(item.startswith("extra:") for item in issues))
        report = mod.empty_report()
        report["kind"] = "other"
        report["schemaVersion"] = "9"
        report["ok"] = "yes"
        report["samplesTested"] = -1
        report["panicClusters"] = [1]
        report["inputShapes"] = {"empty": 0}
        issues = mod.validate_report(report)
        self.assertIn("kind", issues)
        self.assertIn("schemaVersion", issues)
        self.assertIn("ok-type", issues)
        self.assertIn("samplesTested-negative", issues)
        self.assertIn("panicClusters-item-type", issues)
        self.assertTrue(any(item.startswith("inputShapes-missing-") for item in issues))
        report = mod.empty_report()
        report["ok"] = False
        self.assertIn("ok-mismatch", mod.validate_report(report))
        report = mod.empty_report()
        report["emptyCorpus"] = True
        report["totalSamples"] = 3
        self.assertIn("emptyCorpus-mismatch", mod.validate_report(report))

    def test_format_human_report_ok_fail_and_exceptions(self):
        mod = load()
        ok = mod.empty_report(["info"])
        text = mod.format_human_report(ok)
        self.assertIn("DoS 0", text)
        fail = mod.empty_report(["info"])
        fail["ok"] = False
        fail["distinctPanicSites"] = 1
        fail["panicClusters"] = [{"location": "src/x.rs:1", "count": 2, "example": "a:trunc5:info"}]
        fail["hangClusters"] = [{"command": "export-text", "count": 1, "samples": ["a.hwp"], "example": "a:flip10:export-text"}]
        fail["exit"] = 1
        text = mod.format_human_report(fail)
        self.assertIn("PANIC src/x.rs:1", text)
        self.assertIn("HANG  export-text", text)
        empty = mod.empty_report()
        empty["emptyCorpus"] = True
        self.assertIn("빈 코퍼스", mod.format_human_report(empty))
        missing = mod.empty_report()
        missing["missingBin"] = True
        missing["toolFailed"] = True
        missing["ok"] = False
        missing["toolErrors"] = ["missing-bin: not-found"]
        missing["exit"] = 2
        self.assertIn("도구 실패", mod.format_human_report(missing))

    def test_resolve_exit_matrix(self):
        mod = load()
        report = mod.empty_report()
        self.assertEqual(mod.resolve_exit(report), 0)
        report["panicClusters"] = [{"location": "x", "count": 1, "example": "e"}]
        self.assertEqual(mod.resolve_exit(report), 1)
        report = mod.empty_report()
        report["missingBin"] = True
        report["toolFailed"] = True
        self.assertEqual(mod.resolve_exit(report), 2)

    def test_input_shape_counts_in_fuzz(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "empty.hwp").write_bytes(b"")
            Path(d, "tiny.hwp").write_bytes(b"\x00" * 8)
            Path(d, "normal.hwp").write_bytes(NORMAL_2K)
            mod.probe = lambda *_a, **_k: (None, None)
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 0, 1, 5, work)
        self.assertTrue(report["ok"])
        self.assertEqual(report["inputShapes"]["empty"], 1)
        self.assertEqual(report["inputShapes"]["tiny"], 1)
        self.assertEqual(report["inputShapes"]["normal"], 1)
        self.assertEqual(report["inputShapes"]["huge"], 0)
        self.assertGreater(report["runsChecked"], 3)

    def test_empty_sample_still_probes_single_mutant(self):
        mod = load()
        seen = []

        def probe(_bin, cmd, path, _timeout):
            seen.append(Path(path).read_bytes())
            return None, None

        with tempfile.TemporaryDirectory() as d:
            Path(d, "empty.hwp").write_bytes(b"")
            mod.probe = probe
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 1, 1, 5, work)
        self.assertEqual(seen, [b"\x00"])
        self.assertEqual(report["runsChecked"], 1)
        self.assertEqual(report["inputShapes"]["empty"], 1)

    def test_mixed_panic_and_hang_keep_ok_false(self):
        mod = load()

        def probe(_bin, cmd, _path, _timeout):
            if cmd == "a":
                return "panic", "src/x.rs:10"
            if cmd == "b":
                return "hang", "b"
            return None, None

        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = probe
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["a", "b"], 1, 1, 5, work)
        self.assertFalse(report["ok"])
        self.assertEqual(report["distinctPanicSites"], 1)
        self.assertEqual(len(report["hangClusters"]), 1)
        self.assertEqual(report["exit"], mod.EXIT_DOS)
        self.assertEqual(mod.validate_report(report), [])

    def test_panic_clusters_sort_by_count_then_location(self):
        mod = load()
        calls = {"n": 0}

        def probe(_bin, cmd, _path, _timeout):
            calls["n"] += 1
            if cmd == "a":
                return "panic", "src/z.rs:1"
            return "panic", "src/a.rs:1"

        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = probe
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["a", "b"], 1, 1, 5, work)
        locs = [c["location"] for c in report["panicClusters"]]
        self.assertEqual(sorted(locs), ["src/a.rs:1", "src/z.rs:1"])
        # 같은 count 이면 location 사전순.
        if report["panicClusters"][0]["count"] == report["panicClusters"][1]["count"]:
            self.assertEqual(locs, ["src/a.rs:1", "src/z.rs:1"])

    def test_workers_one_matches_workers_many_clusters(self):
        mod = load()

        def probe(_bin, cmd, _path, _timeout):
            if cmd == "info":
                return "panic", "src/x.rs:9"
            return None, None

        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            work = os.path.join(d, "w")
            os.mkdir(work)
            mod.probe = probe
            one = mod.fuzz("bin", d, ["info", "export-text"], 1, 1, 5, work)
            many = mod.fuzz("bin", d, ["info", "export-text"], 1, 4, 5, work)
        self.assertEqual(one["distinctPanicSites"], many["distinctPanicSites"])
        self.assertEqual(one["panicClusters"][0]["location"], many["panicClusters"][0]["location"])
        self.assertEqual(one["ok"], many["ok"])

    def test_probe_head_truncates(self):
        mod = load()
        self.assertEqual(mod.probe_head(None), "")
        self.assertEqual(mod.probe_head(1234), "1234")
        self.assertEqual(mod.probe_head("abcdef", 3), "abc")
        self.assertEqual(mod.probe_head("abcdef", 0), "")


class MainCliTests(unittest.TestCase):
    def test_main_missing_bin_exits_two_and_json(self):
        mod = load()
        buf = io.StringIO()
        with mock.patch.object(sys, "stdout", buf):
            code = mod.main([
                "--bin",
                os.path.join(tempfile.gettempdir(), "no-such-rhwp-bin-abc"),
                "--json",
            ])
        self.assertEqual(code, 2)
        report = json.loads(buf.getvalue())
        self.assertTrue(report["missingBin"])
        self.assertTrue(report["toolFailed"])
        self.assertFalse(report["ok"])
        self.assertEqual(report["kind"], "gymFuzzCorpus")

    def test_main_missing_bin_human_goes_to_stderr(self):
        mod = load()
        err = io.StringIO()
        out = io.StringIO()
        with mock.patch.object(sys, "stderr", err), mock.patch.object(sys, "stdout", out):
            code = mod.main(["--bin", os.path.join(tempfile.gettempdir(), "no-such-rhwp-bin-abc")])
        self.assertEqual(code, 2)
        self.assertIn("도구 실패", err.getvalue())
        self.assertEqual(out.getvalue(), "")

    def test_main_invokes_fuzz_when_bin_exists(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            fake = os.path.join(d, "rhwp.exe" if os.name == "nt" else "rhwp")
            Path(fake).write_bytes(b"x")
            captured = {}

            def fake_fuzz(*args, **kwargs):
                captured["args"] = args
                report = mod.empty_report(["info"])
                report["ok"] = True
                report["exit"] = 0
                return report

            buf = io.StringIO()
            with mock.patch.object(mod, "fuzz", side_effect=fake_fuzz), mock.patch.object(sys, "stdout", buf):
                code = mod.main(["--bin", fake, "--commands", "info", "--json", "--limit", "1"])
            self.assertEqual(code, 0)
            report = json.loads(buf.getvalue())
            self.assertTrue(report["ok"])
            self.assertEqual(captured["args"][2], ["info"])


class GeneratedCatalogTableTests(unittest.TestCase):
    """카탈로그 행마다 why/when 이 비지 않았는지 표로 고정한다."""

    def test_every_catalog_row_has_nonempty_why(self):
        mod = load()
        for row in mod.mutant_catalog():
            self.assertTrue(row["id"], row)
            self.assertTrue(row["family"], row)
            self.assertTrue(row["when"], row)
            self.assertGreaterEqual(len(row["why"]), 8, row["id"])

    def test_family_ids_match_catalog_families_constant(self):
        mod = load()
        self.assertEqual(tuple(mod.FAMILY_IDS), FAMILY_IDS)
        self.assertEqual(set(mod.FAMILY_IDS), set(mod.catalog_families()))

    def test_exception_kinds_have_no_panic_or_hang(self):
        mod = load()
        self.assertNotIn("panic", mod.EXCEPTION_KINDS)
        self.assertNotIn("hang", mod.EXCEPTION_KINDS)
        self.assertIn("missing-bin", mod.EXCEPTION_KINDS)
        self.assertIn("empty-corpus", mod.EXCEPTION_KINDS)
        self.assertIn("unreadable", mod.EXCEPTION_KINDS)


class HonestyTests(unittest.TestCase):
    def test_ok_is_false_when_only_tool_failed(self):
        mod = load()
        report = mod.empty_report(["info"])
        report["toolFailed"] = True
        report["missingBin"] = True
        report["ok"] = False
        report["exit"] = 2
        report["toolErrors"] = ["missing-bin"]
        self.assertEqual(mod.validate_report(report), [])
        self.assertFalse(report["ok"])
        self.assertEqual(report["panicClusters"], [])

    def test_empty_corpus_does_not_claim_dos(self):
        mod = load()
        report = mod.empty_report(["info"])
        report["emptyCorpus"] = True
        self.assertTrue(report["ok"])
        self.assertEqual(report["exit"], 0)
        self.assertEqual(mod.validate_report(report), [])

    def test_probe_error_is_not_hang_cluster(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = lambda *_a, **_k: ("error", "permission")
            work = os.path.join(d, "w")
            os.mkdir(work)
            report = mod.fuzz("bin", d, ["info"], 1, 1, 5, work)
        self.assertEqual(report["hangClusters"], [])
        self.assertEqual(report["panicClusters"], [])
        self.assertTrue(report["probeErrors"])
        self.assertTrue(report["ok"])

    def test_fatal_exception_is_not_swallowed_by_exception_kind(self):
        mod = load()
        self.assertTrue(mod.is_fatal_exception(GeneratorExit()))
        # exception_kind 는 치명 여부를 바꾸지 않는다 — 호출자가 raise 한다.
        self.assertEqual(mod.exception_kind(GeneratorExit()), "unexpected")


if __name__ == "__main__":
    unittest.main()
