from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from support import PKG, REPO
from oracle_vs_self.contracts import ALL_CONTRACTS, PAGE_SMOKE_CONTRACT


class ContractSnapshotTests(unittest.TestCase):
    def test_snapshots_exist(self) -> None:
        names = [
            "oracle_public.page_smoke.fields.json",
            "oracle_public.resolver.fields.json",
            "oracle_public.multiver.fields.json",
            "fidelity_compare.fields.json",
            "visual_sweep.fields.json",
        ]
        for name in names:
            path = PKG / "fixtures" / "contracts" / name
            blob = json.loads(path.read_text(encoding="utf-8"))
            self.assertFalse(blob["rewritten"], name)
            self.assertEqual(blob["consumed_as"], "data", name)
            self.assertTrue(blob["producer"].endswith((".py",)) or blob["producer"].startswith("rhwp"), name)

    def test_page_smoke_fields_live_in_producer(self) -> None:
        src = (REPO / "tools" / "oracle_public" / "page_smoke.py").read_text(encoding="utf-8")
        for field in PAGE_SMOKE_CONTRACT["row_fields"]:
            self.assertIn(f'"{field}"' if field[0].islower() else field, src)
        self.assertIn("pageSmokeReport", src)
        self.assertIn("MATCH", src)
        self.assertIn("MISMATCH", src)
        self.assertIn("ERROR", src)

    def test_resolver_years_live_in_producer(self) -> None:
        src = (REPO / "tools" / "oracle_public" / "oracle_resolver.py").read_text(encoding="utf-8")
        self.assertIn('HANCOM_YEARS = ("2018", "2020", "2022", "2024")', src)
        self.assertIn("DEFAULT_ORACLE_ROOTS", src)
        schema = json.loads(
            (REPO / "tools" / "oracle_public" / "schema" / "oracle_pair_manifest.schema.json").read_text(
                encoding="utf-8"
            )
        )
        years = schema["definitions"]["Pair"]["properties"]["hancomVersion"]["enum"]
        self.assertEqual(years, ["2018", "2020", "2022", "2024"])

    def test_multiver_years_include_2010(self) -> None:
        src = (REPO / "tools" / "oracle_public" / "multiver_index.py").read_text(encoding="utf-8")
        self.assertIn("2010", src)
        self.assertIn("page_count_disagree", src)
        self.assertIn("pypdf_page_count", src)

    def test_fidelity_page_count_ledger_columns_live(self) -> None:
        src = (REPO / "tools" / "fidelity_compare" / "fidelity_compare.py").read_text(encoding="utf-8")
        self.assertIn("page-count-ledger.tsv", src)
        for column in ("measure", "pages", "delta_from_reference", "scope", "note"):
            self.assertIn(column, src)
        self.assertIn("reference_pdf", src)
        self.assertIn("rhwp_svg", src)
        self.assertIn("rhwp_render_tree", src)
        self.assertIn("page-count difference is a candidate", src)

    def test_visual_sweep_schema_version_live(self) -> None:
        src = (REPO / "scripts" / "visual_sweep.py").read_text(encoding="utf-8")
        self.assertIn("VISUAL_SWEEP_RUN_SCHEMA_VERSION = 1", src)
        self.assertIn("schema_version", src)
        self.assertIn("pixel_diff_threshold", src)
        self.assertIn('hwp: Path', src)
        self.assertIn("pdf: Path", src)

    def test_this_package_does_not_import_producers(self) -> None:
        root = PKG
        import_re = re.compile(
            r"^\s*(?:import|from)\s+(?:fidelity_compare|oracle_public|visual_sweep)\b",
            re.MULTILINE,
        )
        for path in root.rglob("*.py"):
            if "tests" in path.parts:
                continue
            text = path.read_text(encoding="utf-8")
            self.assertIsNone(import_re.search(text), f"{path} imports a producer module")
            self.assertNotIn("importlib.import_module", text, f"{path} uses importlib")

    def test_all_contracts_registered(self) -> None:
        self.assertEqual(
            set(ALL_CONTRACTS),
            {
                "oracle_resolver",
                "page_smoke",
                "multiver_index",
                "fidelity_compare",
                "visual_sweep",
                "render_diff_self",
            },
        )
        self.assertTrue(ALL_CONTRACTS["fidelity_compare"]["cannot_run_without_reference_pdf"])
        self.assertTrue(ALL_CONTRACTS["visual_sweep"]["cannot_run_without_reference_pdf"])
        self.assertFalse(ALL_CONTRACTS["render_diff_self"]["is_independent_oracle"])


if __name__ == "__main__":
    unittest.main()
