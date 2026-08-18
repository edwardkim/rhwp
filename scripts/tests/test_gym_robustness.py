"""[robustness] gym 손상-강건성 감사 계약 — 결정적 손상 + 패닉/행 색출.

핵심: rhwp 는 손상 입력에 절대 패닉·행 하면 안 된다. 감사기는 코퍼스를 결정적으로
손상시켜 파싱하고, 패닉(코드 101/음수/'panicked')·행(timeout) 이 있으면 실패로 잡는다.
파싱은 목킹해 바이너리 없이 로직만 시험한다.

확대 계약:
- 같은 입력은 같은 라벨·바이트를 낸다(무작위 없음).
- 원본과 바이트가 같은 무의미 변형은 버린다.
- 빈/극소/거대/비-바이트/읽기실패/쓰기실패/프로브 예외에서도 감사기가 죽지 않는다.
- JSON 봉투 키가 REPORT_KEYS 와 같고 ok 는 패닉·행 부재와 일치한다.
"""

from __future__ import annotations

import errno
import importlib.util
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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
    "unreadables",
    "probeErrors",
    "inputShapes",
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

EXPANDED_ALWAYS_LABELS = (
    "truncate@10%",
    "truncate@99%",
    "flip@0%",
    "flip@25%",
    "flip@75%",
    "flip@99%",
    "chop-last",
    "cut-first",
    "aa-run",
    "nul-mid",
    "00-run",
    "55-run",
    "ole-magic-poison",
    "ole-sector-shift-poison",
    "zip-magic-inject",
    "length-bomb@10%",
    "length-bomb@40%",
    "length-bomb@70%",
    "length-zero@30%",
    "length-one@60%",
    "i32-min@20%",
    "u16-max@12",
    "reverse-prefix",
    "swap-ends",
    "high-bit-stripe",
    "low-bit-stripe",
    "xor-stride7",
    "rotate-header",
    "nibble-swap-head",
    "increment-header",
    "decrement-tail",
    "interleave-zero-head",
    "duplicate-prefix",
    "tail-over-head",
    "invert-tail-64",
    "complement-mid-32",
    "bit-rotate-head",
    "utf16-bom-inject",
    "ascii-ctrl-sprinkle",
    "utf8-overlong",
    "path-sep-sprinkle",
    "slide-window-left",
    "slide-window-right",
    "repeat-mid-block",
    "odd-length-chop",
    "splice-nul-mid",
    "crlf-inject",
    "pad-eof",
    "widen-gap",
    "shrink-gap",
)

FAMILY_IDS = (
    "empty",
    "truncate",
    "flip",
    "header",
    "ole",
    "run",
    "unicode",
    "zip",
    "length",
    "permute",
    "stripe",
    "splice",
    "hwp3",
)

NORMAL_2K = bytes(range(256)) * 8  # 2KB, 짝수, ZIP/HWP3 서명 없음


def load():
    spec = importlib.util.spec_from_file_location("gym_robustness", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def labels_of(pairs):
    return [label for label, _ in pairs]


def as_map(pairs):
    return {label: payload for label, payload in pairs}


class RobustnessTests(unittest.TestCase):
    def test_mutants_deterministic_and_nontrivial(self):
        mod = load()
        data = NORMAL_2K
        a = mod.deterministic_mutants(data)
        b = mod.deterministic_mutants(data)
        self.assertEqual([l for l, _ in a], [l for l, _ in b])  # 결정적(라벨 동일)
        self.assertEqual([m for _, m in a], [m for _, m in b])  # 결정적(바이트 동일)
        self.assertGreaterEqual(len(a), 40)
        for label, mut in a:
            self.assertNotEqual(mut, data, f"{label} 이 원본과 같다(무의미 변형)")

    def test_expanded_mutant_families_are_present(self):
        mod = load()
        data = NORMAL_2K
        labels = labels_of(mod.deterministic_mutants(data))
        for name in ALWAYS_LABELS:
            self.assertIn(name, labels)
        self.assertNotIn("zip-local-header-flip", labels)

        zip_data = b"PK\x03\x04" + data
        zip_labels = labels_of(mod.deterministic_mutants(zip_data))
        self.assertIn("zip-local-header-flip", zip_labels)
        flipped = as_map(mod.deterministic_mutants(zip_data))["zip-local-header-flip"]
        self.assertEqual(flipped[:4], bytes(x ^ 0xFF for x in b"PK\x03\x04"))
        self.assertNotEqual(flipped, zip_data)

    def test_expanded_always_labels_on_normal_payload(self):
        mod = load()
        labels = labels_of(mod.deterministic_mutants(NORMAL_2K))
        for name in EXPANDED_ALWAYS_LABELS:
            self.assertIn(name, labels, name)
        self.assertNotIn("zip-local-header-flip", labels)
        self.assertNotIn("zip-cd-magic-flip", labels)
        self.assertNotIn("zip-eocd-flip", labels)
        self.assertIn("zip-magic-inject", labels)
        self.assertIn("hwp3-sig-inject", labels)
        self.assertNotIn("hwp3-sig-flip", labels)
        self.assertNotIn("empty-to-nul", labels)

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

    def test_is_panic_distinguishes_crash_from_clean_failure(self):
        mod = load()
        self.assertTrue(mod.is_panic(101, ""))  # 어보트
        self.assertTrue(mod.is_panic(0, "thread 'main' panicked"))  # 패닉 메시지
        self.assertTrue(mod.is_panic(-1073741819, ""))  # AV(음수)
        self.assertTrue(mod.is_panic(0xC0000005, ""))  # Windows AV(NTSTATUS)
        self.assertFalse(mod.is_panic(1, "오류: 유효하지 않은 파일"))  # 깨끗한 실패
        self.assertFalse(mod.is_panic(255, "명시적 CLI 오류"))  # 일반 오류 코드는 패닉 아님
        self.assertFalse(mod.is_panic(0, "정상"))  # 정상

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
            self.assertEqual(total, 50)  # .txt 제외
            self.assertLessEqual(len(picked1), 10)
            self.assertEqual(picked1, picked2)  # 결정적
            self.assertTrue(all(f.endswith(".hwp") for f in picked1))

    def _audit_with_probe(self, mod, probe_result):
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
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
        self.assertEqual(set(r), set(mod.REPORT_KEYS))
        self.assertEqual(r["kind"], "gymRobustness")
        self.assertEqual(r["schemaVersion"], "1.0")
        self.assertIsInstance(r["ok"], bool)
        self.assertIsInstance(r["samplesTested"], int)
        self.assertIsInstance(r["totalSamples"], int)
        self.assertIsInstance(r["mutantsChecked"], int)
        self.assertIsInstance(r["gracefullyDegraded"], int)
        self.assertIsInstance(r["panics"], list)
        self.assertIsInstance(r["hangs"], list)
        self.assertIsInstance(r["unreadables"], list)
        self.assertIsInstance(r["probeErrors"], list)
        self.assertIsInstance(r["inputShapes"], dict)
        self.assertEqual(r["samplesTested"], 1)
        self.assertEqual(r["totalSamples"], 1)
        self.assertGreaterEqual(r["mutantsChecked"], 40)
        self.assertTrue(r["ok"])
        self.assertEqual(mod.validate_report(r), [])


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
            "truncate@25%": "truncate",
            "chop-last": "truncate",
            "cut-first": "truncate",
            "odd-length-chop": "truncate",
            "shrink-gap": "truncate",
            "flip@0%": "flip",
            "flip@90%": "flip",
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
            "length-bomb@10%": "length",
            "length-zero@30%": "length",
            "length-one@60%": "length",
            "i32-min@20%": "length",
            "u16-max@12": "length",
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
        self.assertEqual(mapped["length-bomb@10%"][bomb10 : bomb10 + 4], b"\xff\xff\xff\x7f")
        zero30 = min(n - 4, n * 30 // 100)
        self.assertEqual(mapped["length-zero@30%"][zero30 : zero30 + 4], b"\x00\x00\x00\x00")
        one60 = min(n - 4, n * 60 // 100)
        self.assertEqual(mapped["length-one@60%"][one60 : one60 + 4], b"\x01\x00\x00\x00")
        i32 = min(n - 4, n * 20 // 100)
        self.assertEqual(mapped["i32-min@20%"][i32 : i32 + 4], b"\x00\x00\x00\x80")
        self.assertEqual(mapped["u16-max@12"][12:14], b"\xff\xff")

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
        # 실제 1MiB 를 여러 번 복사하면 시험이 느려지므로 최소 거대 크기만 쓴다.
        huge = bytes([0x5A]) * mod.HUGE_MIN
        labels = labels_of(mod.deterministic_mutants(huge))
        for name in (
            "splice-nul-mid",
            "crlf-inject",
            "pad-eof",
            "widen-gap",
            "even-length-pad",
        ):
            self.assertNotIn(name, labels, name)
        for name in ("zero-header", "header-smash", "truncate@50%", "flip@50%", "shrink-gap"):
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


class ExceptionPathTests(unittest.TestCase):
    def test_normalize_limit_and_timeout(self):
        mod = load()
        self.assertEqual(mod.normalize_limit(8), 8)
        self.assertEqual(mod.normalize_limit("3"), 3)
        self.assertEqual(mod.normalize_limit(-4), 0)
        self.assertEqual(mod.normalize_limit(None), 0)
        self.assertEqual(mod.normalize_limit("nope"), 0)
        self.assertEqual(mod.normalize_limit(4.9), 4)
        self.assertEqual(mod.normalize_timeout(8), 8)
        self.assertEqual(mod.normalize_timeout("3"), 3)
        self.assertEqual(mod.normalize_timeout(0), 0)
        self.assertEqual(mod.normalize_timeout(-1), 0)
        self.assertEqual(mod.normalize_timeout(None), 0)
        self.assertEqual(mod.normalize_timeout("nope"), 0)

    def test_select_samples_oserror_and_bad_limit(self):
        mod = load()
        missing = os.path.join(tempfile.gettempdir(), "gym-robust-missing-dir-does-not-exist")
        picked, total = mod.select_samples(missing, 8)
        self.assertEqual(picked, [])
        self.assertEqual(total, 0)
        with tempfile.TemporaryDirectory() as d:
            Path(d, "a.hwp").write_bytes(b"x")
            Path(d, "b.hwp").write_bytes(b"x")
            empty, count = mod.select_samples(d, 0)
            self.assertEqual(empty, [])
            self.assertEqual(count, 2)
            empty2, count2 = mod.select_samples(d, "bad")
            self.assertEqual(empty2, [])
            self.assertEqual(count2, 2)
            all_picked, _ = mod.select_samples(d, 99)
            self.assertEqual(all_picked, ["a.hwp", "b.hwp"])

    def test_read_sample_missing_and_success(self):
        mod = load()
        data, err = mod.read_sample(os.path.join(tempfile.gettempdir(), "no-such-robust.hwp"))
        self.assertIsNone(data)
        self.assertIsInstance(err, str)
        self.assertIn("Error", err)
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "ok.hwp")
            Path(path).write_bytes(b"abc")
            payload, err2 = mod.read_sample(path)
            self.assertEqual(payload, b"abc")
            self.assertIsNone(err2)

    def test_write_mutant_typeerror_and_oserror(self):
        mod = load()
        self.assertTrue(mod.write_mutant("ignored", "not-bytes").startswith("TypeError"))
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "mut.hwp")
            self.assertIsNone(mod.write_mutant(path, b"xyz"))
            self.assertEqual(Path(path).read_bytes(), b"xyz")
            self.assertIsNone(mod.write_mutant(path, bytearray(b"zz")))
            missing_parent = os.path.join(d, "no-dir", "mut.hwp")
            err = mod.write_mutant(missing_parent, b"x")
            self.assertIsInstance(err, str)
            self.assertTrue(err)

    def test_probe_head_and_posix_signal_helper(self):
        mod = load()
        self.assertEqual(mod.probe_head(None), "")
        self.assertEqual(mod.probe_head(1234), "1234")
        self.assertEqual(mod.probe_head("abcdef", 3), "abc")
        self.assertEqual(mod.probe_head("abcdef", 0), "")
        self.assertTrue(mod._posix_signal_timeout(-9))
        self.assertTrue(mod._posix_signal_timeout(-24))
        self.assertFalse(mod._posix_signal_timeout(9))
        self.assertFalse(mod._posix_signal_timeout(None))

    def test_classify_timeout_strings_and_os_errnos(self):
        mod = load()
        self.assertTrue(mod.classify_timeout("Deadline Exceeded"))
        self.assertTrue(mod.classify_timeout("process timed out"))
        self.assertTrue(mod.classify_timeout("TIME-OUT"))
        self.assertFalse(mod.classify_timeout("ok"))
        self.assertTrue(mod.classify_timeout(TimeoutError("late")))
        timed = OSError(getattr(errno, "ETIMEDOUT", 110), "late")
        timed.errno = getattr(errno, "ETIMEDOUT", 110)
        self.assertTrue(mod.classify_timeout(timed))
        other = OSError(errno.ENOENT, "missing")
        other.errno = errno.ENOENT
        self.assertFalse(mod.classify_timeout(other))
        self.assertFalse(mod.classify_timeout(ValueError("timeout in name only")))

    def test_classify_panic_extra_markers_and_bad_code(self):
        mod = load()
        for marker in (
            "stack overflow",
            "core dumped",
            "fatal runtime error",
            "SIGSEGV",
            "SIGABRT",
            "access violation",
            "Segmentation Fault",
            "illegal instruction",
            "Abort trap",
        ):
            self.assertTrue(mod.classify_panic(0, marker), marker)
        self.assertFalse(mod.classify_panic("not-int", "clean"))
        self.assertFalse(mod.classify_panic(1, None))
        self.assertTrue(mod.classify_panic(101, None))
        self.assertTrue(mod.classify_panic(101, 12345))
        self.assertFalse(mod.classify_panic(2, b"bytes-err"))

    def test_classify_probe_outcome_matrix(self):
        mod = load()
        self.assertEqual(mod.classify_probe_outcome(0, False, True, ""), "hang")
        self.assertEqual(mod.classify_probe_outcome(101, False, False, ""), "panic")
        self.assertEqual(mod.classify_probe_outcome(0, True, False, "ok"), "panic")
        self.assertEqual(mod.classify_probe_outcome(0, False, False, "oserror FileNotFound"), "error")
        self.assertEqual(mod.classify_probe_outcome(None, False, False, "probe-error boom"), "error")
        self.assertEqual(mod.classify_probe_outcome(None, False, False, "unreadable x"), "error")
        self.assertEqual(mod.classify_probe_outcome(2, False, False, "bad file"), "graceful")
        self.assertEqual(mod.classify_probe_outcome(0, False, False, ""), "ok")
        self.assertEqual(mod.classify_probe_outcome(None, False, False, "other"), "error")

    def test_probe_invalid_timeout_and_missing_bin(self):
        mod = load()
        self.assertEqual(mod.probe("bin", "x.hwp", 0), (None, False, False, "probe-error invalid-timeout"))
        self.assertEqual(mod.probe("bin", "x.hwp", -3), (None, False, False, "probe-error invalid-timeout"))
        self.assertEqual(mod.probe("", "x.hwp", 5), (None, False, False, "probe-error missing-bin"))
        self.assertEqual(mod.probe(None, "x.hwp", 5), (None, False, False, "probe-error missing-bin"))

    def test_probe_oserror_is_not_panic(self):
        mod = load()
        code, panicked, timed_out, head = mod.probe(
            os.path.join(tempfile.gettempdir(), "no-such-rhwp-bin-xyz"),
            os.path.join(tempfile.gettempdir(), "no-such.hwp"),
            2,
        )
        self.assertIsNone(code)
        self.assertFalse(panicked)
        self.assertFalse(timed_out)
        self.assertTrue(head.startswith("oserror "))
        self.assertEqual(mod.classify_probe_outcome(code, panicked, timed_out, head), "error")

    def test_probe_timeout_expired_is_hang(self):
        mod = load()

        def boom(*_a, **_k):
            raise subprocess.TimeoutExpired("rhwp", 1)

        with mock.patch("subprocess.run", side_effect=boom):
            code, panicked, timed_out, head = mod.probe("rhwp", "x.hwp", 1)
        self.assertIsNone(code)
        self.assertFalse(panicked)
        self.assertTrue(timed_out)
        self.assertTrue(head.startswith("timeout "))
        self.assertEqual(mod.classify_probe_outcome(code, panicked, timed_out, head), "hang")

    def test_probe_unexpected_exception_is_error(self):
        mod = load()

        def boom(*_a, **_k):
            raise RuntimeError("broken pipe")

        with mock.patch("subprocess.run", side_effect=boom):
            code, panicked, timed_out, head = mod.probe("rhwp", "x.hwp", 1)
        self.assertIsNone(code)
        self.assertFalse(panicked)
        self.assertFalse(timed_out)
        self.assertTrue(head.startswith("probe-error RuntimeError"))
        self.assertEqual(mod.classify_probe_outcome(code, panicked, timed_out, head), "error")

    def test_probe_success_path_classifies_panic_from_output(self):
        mod = load()

        class Fake:
            returncode = 0
            stdout = b""
            stderr = b"thread 'main' panicked at src/x.rs"

        with mock.patch("subprocess.run", return_value=Fake()):
            code, panicked, timed_out, head = mod.probe("rhwp", "x.hwp", 1)
        self.assertEqual(code, 0)
        self.assertTrue(panicked)
        self.assertFalse(timed_out)
        self.assertIn("panicked", head)

    def test_audit_records_unreadable_samples(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "ok.hwp").write_bytes(NORMAL_2K)
            Path(d, "bad.hwp").write_bytes(NORMAL_2K)
            real_read = mod.read_sample

            def wrapped(path):
                if path.endswith("bad.hwp"):
                    return None, "PermissionError: denied"
                return real_read(path)

            mod.read_sample = wrapped
            mod.probe = lambda *_a, **_k: (1, False, False, "오류")
            report = mod.audit("bin", d, limit=2, timeout=5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["samplesTested"], 2)
        self.assertEqual(len(report["unreadables"]), 1)
        self.assertIn("bad.hwp", report["unreadables"][0])
        self.assertGreater(report["mutantsChecked"], 0)
        self.assertEqual(report["inputShapes"]["normal"], 1)

    def test_audit_records_write_errors(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.write_mutant = lambda *_a, **_k: "OSError: disk full"
            report = mod.audit("bin", d, limit=1, timeout=5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["mutantsChecked"], 0)
        self.assertGreater(len(report["probeErrors"]), 0)
        self.assertTrue(any("disk full" in item for item in report["probeErrors"]))

    def test_audit_records_probe_error_heads(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = lambda *_a, **_k: (None, False, False, "oserror FileNotFoundError: gone")
            report = mod.audit("bin", d, limit=1, timeout=5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["panics"], [])
        self.assertEqual(report["hangs"], [])
        self.assertGreater(len(report["probeErrors"]), 0)
        self.assertGreater(report["mutantsChecked"], 0)

    def test_audit_probe_raising_is_caught(self):
        mod = load()

        def boom(*_a, **_k):
            raise RuntimeError("probe exploded")

        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = boom
            report = mod.audit("bin", d, limit=1, timeout=5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["mutantsChecked"], 0)
        self.assertTrue(any("probe exploded" in item for item in report["probeErrors"]))

    def test_audit_mutant_typeerror_is_unreadable(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.deterministic_mutants = lambda *_a, **_k: (_ for _ in ()).throw(TypeError("bad"))
            report = mod.audit("bin", d, limit=1, timeout=5)
        self.assertTrue(report["ok"])
        self.assertTrue(any("TypeError" in item for item in report["unreadables"]))
        self.assertEqual(report["mutantsChecked"], 0)

    def test_empty_report_validates(self):
        mod = load()
        report = mod.empty_report()
        self.assertEqual(mod.validate_report(report), [])
        self.assertTrue(report["ok"])
        self.assertEqual(set(report), set(REPORT_KEYS))
        self.assertEqual(report["inputShapes"], {"empty": 0, "tiny": 0, "normal": 0, "huge": 0})

    def test_validate_report_detects_schema_breaks(self):
        mod = load()
        self.assertEqual(mod.validate_report("nope"), ["report-not-dict"])
        report = mod.empty_report()
        report.pop("hangs")
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
        report["panics"] = [1]
        report["inputShapes"] = {"empty": 0}
        issues = mod.validate_report(report)
        self.assertIn("kind", issues)
        self.assertIn("schemaVersion", issues)
        self.assertIn("ok-type", issues)
        self.assertIn("samplesTested-negative", issues)
        self.assertIn("panics-item-type", issues)
        self.assertTrue(any(item.startswith("inputShapes-missing-") for item in issues))
        report = mod.empty_report()
        report["ok"] = False
        self.assertIn("ok-mismatch", mod.validate_report(report))

    def test_format_human_report_ok_and_fail(self):
        mod = load()
        ok = mod.empty_report()
        text = mod.format_human_report(ok)
        self.assertIn("패닉 0", text)
        self.assertIn("행 0", text)
        fail = mod.empty_report()
        fail["ok"] = False
        fail["panics"] = ["s.hwp:ff-run (code 101): panicked"]
        fail["hangs"] = ["s.hwp:truncate@50%"]
        text = mod.format_human_report(fail)
        self.assertIn("패닉 1", text)
        self.assertIn("행 1", text)
        self.assertIn("s.hwp:ff-run", text)
        self.assertIn("s.hwp:truncate@50%", text)

    def test_ok_success_is_not_graceful(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = lambda *_a, **_k: (0, False, False, "")
            report = mod.audit("bin", d, limit=1, timeout=5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["gracefullyDegraded"], 0)
        self.assertGreater(report["mutantsChecked"], 0)
        self.assertEqual(report["inputShapes"]["normal"], 1)

    def test_mixed_panic_and_hang_keep_ok_false(self):
        mod = load()
        calls = {"n": 0}

        def probe(*_a, **_k):
            calls["n"] += 1
            if calls["n"] == 1:
                return 101, True, False, "panicked"
            if calls["n"] == 2:
                return None, False, True, "timeout"
            return 1, False, False, "오류"

        with tempfile.TemporaryDirectory() as d:
            Path(d, "s.hwp").write_bytes(NORMAL_2K)
            mod.probe = probe
            report = mod.audit("bin", d, limit=1, timeout=5)
        self.assertFalse(report["ok"])
        self.assertEqual(len(report["panics"]), 1)
        self.assertEqual(len(report["hangs"]), 1)
        self.assertGreater(report["gracefullyDegraded"], 0)
        self.assertEqual(mod.validate_report(report), [])


class ShapeAndSelectEdgeTests(unittest.TestCase):
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
            self.assertEqual(five, [five[0], five[1], five[2], five[3], five[4]])
            self.assertEqual(mod.select_samples(d, 5)[0], five)

    def test_input_shape_counts_in_audit(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "empty.hwp").write_bytes(b"")
            Path(d, "tiny.hwp").write_bytes(b"\x00" * 8)
            Path(d, "normal.hwp").write_bytes(NORMAL_2K)
            mod.probe = lambda *_a, **_k: (1, False, False, "오류")
            report = mod.audit("bin", d, limit=3, timeout=5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["inputShapes"]["empty"], 1)
        self.assertEqual(report["inputShapes"]["tiny"], 1)
        self.assertEqual(report["inputShapes"]["normal"], 1)
        self.assertEqual(report["inputShapes"]["huge"], 0)
        self.assertGreater(report["mutantsChecked"], 3)

    def test_empty_sample_still_probes_single_mutant(self):
        mod = load()
        seen = []

        def probe(_bin, path, _timeout):
            seen.append(Path(path).read_bytes())
            return 1, False, False, "오류"

        with tempfile.TemporaryDirectory() as d:
            Path(d, "empty.hwp").write_bytes(b"")
            mod.probe = probe
            report = mod.audit("bin", d, limit=1, timeout=5)
        self.assertEqual(seen, [b"\x00"])
        self.assertEqual(report["mutantsChecked"], 1)
        self.assertEqual(report["inputShapes"]["empty"], 1)
        self.assertEqual(report["gracefullyDegraded"], 1)

    def test_no_hwp_files_yields_empty_clean_report(self):
        mod = load()
        with tempfile.TemporaryDirectory() as d:
            Path(d, "note.txt").write_text("x", encoding="utf-8")
            report = mod.audit("bin", d, limit=8, timeout=5)
        self.assertTrue(report["ok"])
        self.assertEqual(report["samplesTested"], 0)
        self.assertEqual(report["totalSamples"], 0)
        self.assertEqual(report["mutantsChecked"], 0)
        self.assertEqual(mod.validate_report(report), [])

    def test_module_constants_match_test_contract(self):
        mod = load()
        self.assertEqual(mod.ALWAYS_LABELS, ALWAYS_LABELS)
        self.assertEqual(mod.EXPANDED_ALWAYS_LABELS, EXPANDED_ALWAYS_LABELS)
        self.assertEqual(mod.REPORT_KIND, "gymRobustness")
        self.assertEqual(mod.SCHEMA_VERSION, "1.0")
        self.assertEqual(mod.TINY_MAX, 64)
        self.assertEqual(mod.HUGE_MIN, 1_048_576)
        self.assertEqual(mod.OLE_MAGIC, b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1")
        self.assertEqual(mod.ZIP_LOCAL, b"PK\x03\x04")
        self.assertEqual(mod.ZIP_CD, b"PK\x01\x02")
        self.assertEqual(mod.ZIP_EOCD, b"PK\x05\x06")


if __name__ == "__main__":
    unittest.main()
