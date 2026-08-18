#!/usr/bin/env python3
"""M-sec inspect 픽스처 계약 시험 (#5476).

생성기 산출이 기존 3축 kind·예외 규약을 지키는지 본다.
새 탐지 규칙을 주장하지 않는다. 라이브 rhwp 를 부르지 않는다.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "tests" / "fixtures" / "inspect_msec"
WORKING = ROOT / "mydocs" / "working"

ALLOWED_HT = {
    "same_as_background",
    "near_invisible",
    "zero_size",
    "off_page",
}
ALLOWED_INJ = {
    "role_impersonation",
    "instruction_override",
    "tool_directive",
    "authority_claim",
    "exfiltration_hint",
    "delimiter_break",
}
ALLOWED_UNI = {"zero_width", "bidi_override", "tag_char", "confusable"}
ALLOWED_AXIS = {"hidden-text", "injection", "unicode", "inspect"}


def load(rel: str):
    return json.loads((OUT / rel).read_text(encoding="utf-8"))


class TestCatalog(unittest.TestCase):
    def setUp(self) -> None:
        self.cat = load("catalog.json")

    def test_catalog_declares_three_axes_only(self) -> None:
        self.assertEqual(self.cat["axes"], ["hidden-text", "injection", "unicode"])
        self.assertFalse(self.cat["inventedRule"])
        self.assertGreaterEqual(self.cat["counts"]["total"], 200)
        self.assertGreaterEqual(self.cat["counts"]["exceptions"], 20)

    def test_kinds_are_devel_only(self) -> None:
        self.assertEqual(set(self.cat["kinds"]["hidden-text"]), ALLOWED_HT)
        self.assertEqual(set(self.cat["kinds"]["injection"]), ALLOWED_INJ)
        self.assertEqual(set(self.cat["kinds"]["unicode"]), ALLOWED_UNI)

    def test_every_catalog_path_exists(self) -> None:
        text = (OUT / "matrices" / "catalog.tsv").read_text(encoding="utf-8")
        rows = [ln.split("\t") for ln in text.splitlines()[1:] if ln.strip()]
        self.assertGreaterEqual(len(rows), 200)
        for row in rows:
            cid, axis, _family, _pol, rel = row[:5]
            path = OUT / rel
            self.assertTrue(path.is_file(), rel)
            rec = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(rec["id"], cid)
            self.assertEqual(rec["issue"], 5476)
            self.assertFalse(rec["inventedRule"])
            self.assertIn(rec["axis"], ALLOWED_AXIS)
            self.assertEqual(rec["axis"], axis)


class TestEnvelopes(unittest.TestCase):
    def setUp(self) -> None:
        self.cat = load("catalog.json")
        self.recs = []
        for path in sorted((OUT / "envelopes").rglob("*.json")):
            self.recs.append(json.loads(path.read_text(encoding="utf-8")))

    def test_success_envelopes_have_required_keys(self) -> None:
        required = self.cat["requiredEnvelopeKeys"]
        for rec in self.recs:
            env = rec.get("envelope")
            if env is None:
                continue
            axis = rec["axis"]
            if axis not in required:
                continue
            for key in required[axis]:
                self.assertIn(key, env, f"{rec['id']} missing {key}")
            self.assertEqual(env["schemaVersion"], "1.0")

    def test_hidden_text_kinds_not_invented(self) -> None:
        for rec in self.recs:
            env = rec.get("envelope") or {}
            for hit in env.get("hiddenText") or []:
                self.assertIn(hit["kind"], ALLOWED_HT, rec["id"])
                self.assertLessEqual(len(hit["excerpt"]), 201, rec["id"])

    def test_injection_kinds_not_invented(self) -> None:
        for rec in self.recs:
            env = rec.get("envelope") or {}
            for sig in env.get("injectionSignals") or []:
                self.assertIn(sig["kind"], ALLOWED_INJ, rec["id"])
                self.assertIn(sig["confidence"], {"low", "medium", "high"})
            if (
                rec["axis"] == "injection"
                and env.get("clean") is False
                and rec["polarity"] == "positive"
            ):
                self.assertEqual(rec["consume"].get("matchedIs"), "DATA", rec["id"])

    def test_unicode_kinds_not_invented(self) -> None:
        for rec in self.recs:
            env = rec.get("envelope") or {}
            for hit in env.get("findings") or []:
                self.assertIn(hit["kind"], ALLOWED_UNI, rec["id"])
                self.assertIn("rendered", hit)
                self.assertIn("raw", hit)
            counts = env.get("kindCounts")
            if counts:
                self.assertEqual(set(counts), ALLOWED_UNI)

    def test_clean_keeps_empty_arrays(self) -> None:
        for rec in self.recs:
            env = rec.get("envelope")
            if not env or env.get("clean") is not True:
                continue
            if rec["axis"] == "hidden-text":
                self.assertEqual(env["hiddenText"], [])
                self.assertEqual(env["hiddenCharCount"], 0)
            if rec["axis"] == "injection":
                self.assertEqual(env["injectionSignals"], [])
                self.assertEqual(env["signalCount"], 0)
                self.assertIsNone(env["highestConfidence"])
            if rec["axis"] == "unicode":
                self.assertEqual(env["findings"], [])
                self.assertEqual(env["findingCount"], 0)

    def test_detection_is_exit_zero(self) -> None:
        for rec in self.recs:
            if rec["polarity"] == "exception":
                continue
            self.assertEqual(rec["cli"]["exitCode"], 0, rec["id"])
            self.assertTrue(rec["cli"]["detectionIsNotFailure"], rec["id"])

    def test_exceptions_keep_stdout_empty(self) -> None:
        found = 0
        for rec in self.recs:
            if rec["polarity"] != "exception":
                continue
            found += 1
            self.assertIsNone(rec.get("envelope"), rec["id"])
            self.assertEqual(rec["cli"]["stdoutBytes"], 0, rec["id"])
            self.assertIn(rec["cli"]["exitCode"], (1, 2), rec["id"])
            self.assertTrue(rec["cli"].get("stderrContains"), rec["id"])
        self.assertGreaterEqual(found, 20)

    def test_offpage_flag_pair(self) -> None:
        off = None
        on = None
        for rec in self.recs:
            if rec["id"] == "ht-offpage-flag-excluded":
                off = rec
            if rec["id"] == "ht-offpage-flag-included":
                on = rec
        self.assertIsNotNone(off)
        self.assertIsNotNone(on)
        self.assertFalse(off["envelope"]["includeOffPage"])
        self.assertTrue(off["envelope"]["clean"])
        self.assertTrue(on["envelope"]["includeOffPage"])
        self.assertFalse(on["envelope"]["clean"])
        self.assertEqual(on["envelope"]["hiddenText"][0]["kind"], "off_page")

    def test_min_confidence_matrix(self) -> None:
        by = {r["id"]: r for r in self.recs}
        low = by["inj-min-confidence-low"]["envelope"]
        med = by["inj-min-confidence-medium"]["envelope"]
        high = by["inj-min-confidence-high"]["envelope"]
        self.assertEqual(low["signalCount"], 6)
        self.assertEqual(med["signalCount"], 5)
        self.assertEqual(high["signalCount"], 3)
        self.assertNotIn(
            "delimiter_break", [s["kind"] for s in med["injectionSignals"]]
        )
        self.assertEqual(
            {s["kind"] for s in high["injectionSignals"]},
            {"role_impersonation", "instruction_override", "tool_directive"},
        )

    def test_scan_scopes_field_gate(self) -> None:
        by = {r["id"]: r for r in self.recs}
        off = by["inj-scopes-include-fields-off"]["envelope"]
        on = by["inj-scopes-include-fields-on"]["envelope"]
        self.assertNotIn("fieldName", off["scanScopes"])
        self.assertIn("fieldName", on["scanScopes"])
        self.assertIn("body", off["scanScopes"])
        self.assertTrue(on["includeFields"])

    def test_excerpt_limit(self) -> None:
        rec = next(r for r in self.recs if r["id"] == "ht-excerpt-limit-200")
        hit = rec["envelope"]["hiddenText"][0]
        self.assertEqual(hit["charCount"], 5000)
        self.assertTrue(hit["excerpt"].endswith("…"))
        self.assertEqual(len(hit["excerpt"]), 201)

    def test_bidi_rendered_raw(self) -> None:
        rec = next(r for r in self.recs if r["id"] == "uni-bidi-rendered-vs-raw-exe-doc")
        hit = rec["envelope"]["findings"][0]
        self.assertEqual(hit["rendered"], "exe.doc")
        self.assertIn("U+202E", hit["raw"])
        self.assertNotEqual(hit["rendered"], hit["raw"])

    def test_pair_ids_resolve(self) -> None:
        ids = {r["id"] for r in self.recs}
        for rec in self.recs:
            if rec["pair"]:
                self.assertIn(rec["pair"], ids, rec["id"])


class TestMatrices(unittest.TestCase):
    def test_override_cartesian_is_existing_tokens_only(self) -> None:
        text = (OUT / "matrices" / "injection_override_ko.tsv").read_text(encoding="utf-8")
        lines = [ln for ln in text.splitlines() if ln.strip()]
        self.assertGreater(len(lines), 10 * 7 * 12)  # header + 10×7×13
        self.assertIn("무시하", text)
        self.assertIn("이전", text)
        self.assertNotIn("please jailbreak", text)

    def test_tag_range_is_e0000_e007f(self) -> None:
        text = (OUT / "matrices" / "unicode_tag_range.tsv").read_text(encoding="utf-8")
        rows = text.splitlines()[1:]
        self.assertEqual(len(rows), 128)
        self.assertTrue(rows[0].startswith("U+E0000"))
        self.assertTrue(rows[-1].startswith("U+E007F"))

    def test_confusable_table_has_devel_letters(self) -> None:
        text = (OUT / "matrices" / "unicode_confusable.tsv").read_text(encoding="utf-8")
        self.assertIn("а\tU+0430\ta", text)
        self.assertIn("Т\tU+0422\tT", text)
        self.assertIn("α\tU+03B1\ta", text)

    def test_zero_width_excludes_soft_hyphen(self) -> None:
        text = (OUT / "matrices" / "unicode_zero_width.tsv").read_text(encoding="utf-8")
        self.assertIn("U+200B", text)
        self.assertIn("U+00AD", text)
        for line in text.splitlines():
            if line.startswith("U+00AD"):
                self.assertIn("False", line)


class TestWorkingDocs(unittest.TestCase):
    def test_overview_exists(self) -> None:
        text = (WORKING / "m_sec_inspect_fatten.md").read_text(encoding="utf-8")
        self.assertIn("closes #5476", text)
        self.assertIn("feat/m-sec-inspect-fatten", text)
        self.assertIn("cargo fmt --all -- --check", text)

    def test_axis_docs_exist(self) -> None:
        for name in (
            "hidden_text_envelopes.md",
            "injection_envelopes.md",
            "unicode_envelopes.md",
            "inspect_envelopes.md",
            "resweep_gate.md",
        ):
            path = WORKING / "inspect_msec" / name
            self.assertTrue(path.is_file(), name)
            text = path.read_text(encoding="utf-8")
            self.assertGreater(len(text.splitlines()), 10, name)
            self.assertIn("#5476", text)


if __name__ == "__main__":
    if not (OUT / "catalog.json").is_file():
        print("catalog.json 없음 — gen_msec_fixtures.py 를 먼저 실행하세요", file=sys.stderr)
        sys.exit(2)
    unittest.main(verbosity=2)
