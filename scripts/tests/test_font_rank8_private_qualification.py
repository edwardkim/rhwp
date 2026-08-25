import unittest
from collections import Counter
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from font_rank8_private_qualification import (  # noqa: E402
    Rank8PrivateQualificationError,
    apply_metric_transform_precise,
    classify_document,
    context_from_ancestors,
    infer_font_size_hwpunit,
    line_disposition,
    resolve_font_size_hwpunit,
)


def record(width_source: str, base: int) -> dict:
    return {
        "layoutMetric": {
            "baseAdvanceHwpunit": base,
            "widthSource": width_source,
        }
    }


class Rank8PrivateQualificationTest(unittest.TestCase):
    def test_font_size_inference_reconciles_all_heuristics(self) -> None:
        records = [
            record("heuristicFullwidth", 1000),
            record("heuristicHalfwidth", 500),
            record("heuristicNarrow", 300),
        ]
        self.assertEqual(infer_font_size_hwpunit(records), 1000)

    def test_font_size_inference_fails_closed_on_mixed_runs(self) -> None:
        with self.assertRaises(Rank8PrivateQualificationError):
            infer_font_size_hwpunit(
                [record("heuristicFullwidth", 1000), record("heuristicHalfwidth", 600)]
            )

    def test_font_size_resolution_replays_integer_quantization(self) -> None:
        value = record("heuristicFullwidth", 1001)
        value["layoutMetric"].update(
            {
                "finalAdvanceHwpunit": 887,
                "transforms": [
                    {"kind": "ratio", "input": "13.346", "output": "0.9"},
                    {"kind": "letterSpacing", "input": "-0.2"},
                ],
            }
        )
        resolved = resolve_font_size_hwpunit([value])
        self.assertLessEqual(abs(resolved - 1001), 8)

    def test_precise_replay_preserves_fractional_base_until_final_quantization(self) -> None:
        value = record("heuristicNarrow", 389)
        value["layoutMetric"].update(
            {
                "finalAdvanceHwpunit": 359,
                "transforms": [
                    {"kind": "ratio", "input": "5.199999999999999", "output": "0.95"},
                    {"kind": "letterSpacing", "input": "-0.52"},
                ],
            }
        )
        self.assertEqual(
            apply_metric_transform_precise(value, 5.2, font_size_hwpunit=1300), 359
        )

    def test_context_uses_nearest_specialized_owner(self) -> None:
        self.assertEqual(context_from_ancestors(["Page", "Body", "Table", "Cell"]), "table-cell")
        self.assertEqual(context_from_ancestors(["Page", "Body", "TextBox"]), "text-box")
        self.assertEqual(context_from_ancestors(["Page", "Header"]), "header")
        self.assertEqual(context_from_ancestors(["Page", "Body"]), "body")

    def test_line_disposition_distinguishes_improvement_and_regression(self) -> None:
        self.assertEqual(line_disposition(1.0, 0.0), "overflow-removed")
        self.assertEqual(line_disposition(0.0, 1.0), "overflow-introduced")
        self.assertEqual(line_disposition(2.0, 1.0), "overflow-reduced")
        self.assertEqual(line_disposition(1.0, 2.0), "overflow-increased")

    def test_document_classification_is_fail_closed_for_any_regression(self) -> None:
        self.assertEqual(
            classify_document(Counter({"overflow-reduced": 2})), "improved"
        )
        self.assertEqual(
            classify_document(
                Counter({"overflow-reduced": 2, "overflow-introduced": 1})
            ),
            "worsened",
        )
        self.assertEqual(classify_document(Counter({"slack-increased": 2})), "unchanged")


if __name__ == "__main__":
    unittest.main()
