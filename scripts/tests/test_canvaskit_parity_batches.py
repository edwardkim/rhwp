from __future__ import annotations

import importlib.util
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "canvaskit_parity_batches.py"
SPEC = importlib.util.spec_from_file_location("canvaskit_parity_batches", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"canvaskit_parity_batches 모듈을 불러올 수 없습니다: {MODULE_PATH}")
BATCHES = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = BATCHES
SPEC.loader.exec_module(BATCHES)


class BatchMapTests(unittest.TestCase):
    def test_supported_batches_match_parity_plan(self) -> None:
        self.assertEqual(
            BATCHES.BATCH_NAMES,
            {
                2: "Paint Family Parity",
                3: "Strict Text Variant Replay",
            },
        )

    def test_batch_two_covers_paint_family_harnesses(self) -> None:
        ids = [entry["id"] for entry in BATCHES.batch_jobs()[2]]
        self.assertIn("rust-canvaskit-policy", ids)
        self.assertIn("studio-renderer-contract", ids)
        self.assertIn("renderer-baseline-readiness", ids)

    def test_batch_three_covers_strict_text_variant_harnesses(self) -> None:
        ids = [entry["id"] for entry in BATCHES.batch_jobs()[3]]
        self.assertIn("rust-text-variants", ids)
        self.assertIn("studio-text-variant-selection", ids)
        self.assertIn("studio-renderer-contract", ids)

    def test_driver_never_points_at_manifest_as_output(self) -> None:
        for entries in BATCHES.batch_jobs().values():
            for entry in entries:
                joined = " ".join(entry["command"])
                self.assertNotIn("--write-manifest", joined)
                self.assertFalse(
                    any(
                        part.endswith("renderer_baseline_manifest.json")
                        and prev in {"-o", "--output"}
                        for prev, part in zip(entry["command"], entry["command"][1:])
                    )
                )

    def test_list_mode_prints_plan_and_no_manifest_rewrite(self) -> None:
        buffer = io.StringIO()
        with redirect_stdout(buffer):
            exit_code = BATCHES.main(["--list", "--batches", "2,3"])
        text = buffer.getvalue()
        self.assertEqual(exit_code, 0)
        self.assertIn("batch 2: Paint Family Parity", text)
        self.assertIn("batch 3: Strict Text Variant Replay", text)
        self.assertIn("manifest updates: never from this driver", text)

    def test_parse_batches_rejects_unknown(self) -> None:
        with self.assertRaises(SystemExit):
            BATCHES.parse_batches("1")


if __name__ == "__main__":
    unittest.main()
