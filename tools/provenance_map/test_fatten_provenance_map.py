#!/usr/bin/env python3
"""M-prov fatten 계약 테스트 — 픽스처가 MAP 과 금지 자리 규칙을 지키는지.

라이브 rhwp 바이너리를 요구하지 않는다. rust MAP 파서와 생성된 JSON 을 대조한다.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from catalog import EXTRAS, FAMILIES, SLOTS, extra_for  # noqa: E402
from fatten_provenance_map import (  # noqa: E402
    INJECTION_CANARY,
    build_envelope_sample,
    build_field_fixture,
    generate,
)
from parse_map import parse_map, unique_by_first  # noqa: E402


class ParseMapTests(unittest.TestCase):
    def test_map_has_json_contract_commands(self) -> None:
        raw = parse_map()
        names = [e.command for e in unique_by_first(raw)]
        for must in (
            "info",
            "export-text",
            "export-structure",
            "search",
            "fields",
            "export-provenance-map",
            "inspect",
            "edit",
            "run",
        ):
            self.assertIn(must, names)

    def test_every_declared_path_has_origin(self) -> None:
        for entry in parse_map():
            for item in entry.untrusted:
                self.assertTrue(item.path, entry.command)
                self.assertTrue(item.origin.strip(), f"{entry.command} {item.path}")

    def test_duplicate_charts_first_wins(self) -> None:
        raw = parse_map()
        charts = [e for e in raw if e.command == "charts"]
        self.assertGreaterEqual(len(charts), 1)
        first = unique_by_first(raw)
        self.assertEqual(sum(1 for e in first if e.command == "charts"), 1)


class CatalogCoverageTests(unittest.TestCase):
    def test_extras_cover_unique_map(self) -> None:
        names = {e.command for e in unique_by_first(parse_map())}
        self.assertEqual(names, set(EXTRAS))

    def test_families_known(self) -> None:
        for extra in EXTRAS.values():
            self.assertIn(extra.family, FAMILIES)
            self.assertIn(extra.risk, {"none", "low", "medium", "high", "critical"})

    def test_mode_present_subset_of_map(self) -> None:
        by_cmd = {e.command: e for e in unique_by_first(parse_map())}
        for name, extra in EXTRAS.items():
            declared = {item.path for item in by_cmd[name].untrusted}
            for mode in extra.modes:
                unknown = set(mode.present) - declared
                self.assertFalse(unknown, f"{name}/{mode.mode}: {unknown}")

    def test_slots_have_unique_ids(self) -> None:
        ids = [s.slot for s in SLOTS]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertGreaterEqual(len(SLOTS), 20)


class FixtureContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.entries = unique_by_first(parse_map())

    def test_field_fixture_matches_map(self) -> None:
        for entry in self.entries:
            fixture = build_field_fixture(entry)
            self.assertEqual(fixture["command"], entry.command)
            self.assertEqual(fixture["untrusted"], [i.path for i in entry.untrusted])
            self.assertEqual(fixture["note"], entry.note)
            for item in entry.untrusted:
                self.assertEqual(fixture["origins"][item.path], item.origin)

    def test_envelope_subset_and_mark(self) -> None:
        for entry in self.entries:
            extra = extra_for(entry.command)
            for mode in extra.modes:
                sample = build_envelope_sample(entry, mode.mode)
                self.assertTrue(sample["subsetOk"], sample["id"])
                env = sample["envelope"]
                self.assertIn("untrustedContent", env)
                self.assertIn("untrustedFields", env)
                self.assertEqual(env["untrustedContent"], bool(mode.present))
                self.assertEqual(env["untrustedFields"], list(mode.present))
                if mode.present:
                    blob = json.dumps(env, ensure_ascii=False)
                    # at least one present path should have a document token
                    self.assertTrue(
                        any(entry.command in blob for _ in mode.present),
                        sample["id"],
                    )

    def test_canary_only_on_document_text_modes(self) -> None:
        texty = {
            "pages[].text",
            "text",
            "excerpt",
            "armoredText",
            "matches[].text",
            "matches[].context",
            "injectionSignals[].excerpt",
            "structure.roots[].heading",
            "summary",
        }
        for entry in self.entries:
            extra = extra_for(entry.command)
            for mode in extra.modes:
                sample = build_envelope_sample(entry, mode.mode)
                blob = json.dumps(sample["envelope"], ensure_ascii=False)
                if set(mode.present) & texty:
                    self.assertIn(INJECTION_CANARY, blob, sample["id"])
                else:
                    self.assertNotIn(INJECTION_CANARY, blob, sample["id"])

    def test_empty_untrusted_commands_are_false(self) -> None:
        for entry in self.entries:
            if entry.untrusted:
                continue
            extra = extra_for(entry.command)
            for mode in extra.modes:
                self.assertEqual(mode.present, (), f"{entry.command} empty MAP but mode has D")
                sample = build_envelope_sample(entry, mode.mode)
                self.assertFalse(sample["envelope"]["untrustedContent"])


class GeneratedTreeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        from parse_map import repo_root

        cls.counts = generate(repo_root())
        cls.fx = HERE / "fixtures"

    def test_file_counts(self) -> None:
        entries = unique_by_first(parse_map())
        self.assertEqual(self.counts["commands"], len(entries))
        self.assertEqual(len(list((self.fx / "untrusted_fields").glob("*.json"))), len(entries))
        expected_env = sum(len(extra_for(e.command).modes) for e in entries)
        self.assertEqual(len(list((self.fx / "envelopes").glob("*.json"))), expected_env)
        self.assertEqual(len(list((self.fx / "forbidden_slots").glob("*.json"))), len(SLOTS))
        self.assertGreater(self.counts["cross_files"], 200)
        self.assertFalse((self.fx / "field_slots").exists())
        from parse_map import repo_root

        fields = repo_root() / "tools" / "provenance_map" / "tables" / "untrusted_fields.tsv"
        self.assertTrue(fields.is_file())
        self.assertGreaterEqual(len(fields.read_text(encoding="utf-8").splitlines()), 60)

    def test_on_disk_envelope_mark(self) -> None:
        for path in (self.fx / "envelopes").glob("*.json"):
            data = json.loads(path.read_text(encoding="utf-8"))
            env = data["envelope"]
            self.assertIsInstance(env["untrustedFields"], list)
            self.assertEqual(env["untrustedContent"], bool(env["untrustedFields"]))
            declared = set(data["declaredUntrusted"])
            self.assertTrue(set(env["untrustedFields"]) <= declared)

    def test_working_docs_exist(self) -> None:
        from parse_map import repo_root

        working = repo_root() / "mydocs" / "working" / "m-prov-fatten"
        for name in (
            "WORKING.md",
            "02_forbidden_slots.md",
            "03_mode_presence.md",
            "04_injection_boundary.md",
            "05_consumer_checklist.md",
            "06_command_families.md",
        ):
            self.assertTrue((working / name).is_file(), name)
        self.assertFalse(list(working.glob("family_*.md")))

    def test_no_gym_and_no_new_cli(self) -> None:
        text = (HERE / "WORKING.md").read_text(encoding="utf-8")
        self.assertIn("새 CLI", text)
        self.assertIn("gym", text.lower())
        self.assertIn("export-provenance-map", text)


if __name__ == "__main__":
    unittest.main()
