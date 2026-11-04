"""다른 한컴 세대의 산출물을 시각 증거로 섞지 않는 도구 계약."""

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.tests.test_visual_sweep import SWEEP


class GenerationTests(unittest.TestCase):
    def test_native_svg_and_tree_use_the_same_generation(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            for name in ("svg", "render_tree"):
                (base / name).mkdir()

            def exporter(command, **kwargs):
                output = Path(command[command.index("-o") + 1])
                name = "source_001.svg" if command[1] == "export-svg" else "page_001.json"
                (output / name).write_text("{}", encoding="utf-8")
                envelope = {"pageCount": 1, "renderedCount": 1, "pages": [{"page": 0}]}
                return subprocess.CompletedProcess(command, 0, json.dumps(envelope), "")

            with patch.object(SWEEP, "run", side_effect=exporter) as run:
                SWEEP.export_native_target(
                    base, base / "source.hwp", "rhwp", base, None,
                    ["--font-style"], ["--compat", "2024"],
                )
            self.assertEqual([call.args[0][1] for call in run.call_args_list],
                             ["export-svg", "export-render-tree"])
            for call in run.call_args_list:
                command = call.args[0]
                self.assertEqual(command[command.index("--compat") + 1], "2024")

    def test_wasm_and_font_policy_share_generation_and_reject_wrong_manifest(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)

            def exporter(command, **kwargs):
                if command[0] == "node":
                    output = Path(command[command.index("--out") + 1])
                    for name in ("raw_svg", "render_tree"):
                        (output / name).mkdir(parents=True)
                    (output / "raw_svg/wasm_001.svg").write_text("<svg/>")
                    (output / "render_tree/render_tree_001.json").write_text("{}")
                    generation = command[command.index("--compat") + 1]
                    (output / "manifest.json").write_text(json.dumps({
                        "pageCount": 1, "layoutGeneration": generation,
                    }))
                else:
                    output = Path(command[command.index("-o") + 1])
                    (output / "policy_001.svg").write_text("<svg/>")
                return subprocess.CompletedProcess(command, 0, "", "")

            with patch.object(SWEEP, "run", side_effect=exporter) as run:
                SWEEP.export_wasm_target(
                    base, base / "source.hwp", base / "pkg", "rhwp", base / "out",
                    compat="2024",
                )
            for call in run.call_args_list:
                command = call.args[0]
                self.assertEqual(command[command.index("--compat") + 1], "2024")

            def wrong_generation(command, **kwargs):
                result = exporter(command, **kwargs)
                if command[0] == "node":
                    output = Path(command[command.index("--out") + 1])
                    (output / "manifest.json").write_text(
                        '{"pageCount":1,"layoutGeneration":"2022"}',
                    )
                return result

            with patch.object(SWEEP, "run", side_effect=wrong_generation):
                with self.assertRaisesRegex(SystemExit, "조판 세대"):
                    SWEEP.export_wasm_target(
                        base, base / "source.hwp", base / "pkg", "rhwp", base / "out",
                        compat="2024",
                    )
            self.assertFalse((base / "out/wasm-export-complete.json").exists())

    def test_resume_cannot_reuse_another_generation(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            target = SWEEP.Target("case", Path("source.hwp"), Path("oracle.pdf"))
            SWEEP.run_manifest_for_target(
                base, target, {"layout_generation": "2022"}, 96, 32, resume=False,
            )
            with self.assertRaisesRegex(SystemExit, "provenance"):
                SWEEP.run_manifest_for_target(
                    base, target, {"layout_generation": "2024"}, 96, 32, resume=True,
                )

    def test_resume_rejects_changed_font_registration_and_observation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "scripts").mkdir()
            package = root / "pkg"
            package.mkdir()
            for name in ("rhwp.js", "rhwp_bg.wasm"):
                (package / name).write_bytes(b"package")
            (root / "scripts/export-wasm-png-for-sweep.mjs").write_text("exporter")
            helper = root / "scripts/visual-sweep-font-registration.mjs"
            helper.write_text("original registration")
            receipt = root / "observation.json"
            receipt.write_text('{"source":"original program"}')

            def provenance():
                return {"wasm": SWEEP.wasm_package_provenance(root, package, "png"),
                        "observed_font_receipt": {"path": str(receipt),
                                                  "sha256": SWEEP.sha256_file(receipt)}}

            base = root / "output"
            base.mkdir()
            target = SWEEP.Target("case", Path("source.hwp"), Path("oracle.pdf"))
            SWEEP.run_manifest_for_target(base, target, provenance(), 96, 32, resume=False)
            SWEEP.run_manifest_for_target(base, target, provenance(), 96, 32, resume=True)
            helper.write_text("changed registration")
            with self.assertRaisesRegex(SystemExit, "provenance"):
                SWEEP.run_manifest_for_target(base, target, provenance(), 96, 32, resume=True)
            helper.write_text("original registration")
            receipt.write_text('{"source":"different program"}')
            with self.assertRaisesRegex(SystemExit, "provenance"):
                SWEEP.run_manifest_for_target(base, target, provenance(), 96, 32, resume=True)


if __name__ == "__main__":
    unittest.main()
