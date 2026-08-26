#!/usr/bin/env python3
"""adapter_diff 하네스 단위 시험 — 가짜 트리, 실문서 불필요.

실행 (저장소 루트):
    python -m unittest tools.adapter_diff.test_harness
    python -m unittest tools/adapter_diff/test_harness.py
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import harness as ad  # noqa: E402


def _write(path: Path, text: str) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


SVG_SRC = """
impl RenderBackend for SvgBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            clipping: false,
            ..BackendCapabilities::vector("svg")
        }
    }
}
"""

BACKENDS_SRC = """
impl RenderBackend for NullBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            ..BackendCapabilities::none("null")
        }
    }
}
impl RenderBackend for TraceBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            ..BackendCapabilities::none("trace")
        }
    }
}
"""

PNG_SRC = """
impl RenderBackend for PngBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            ..BackendCapabilities::raster("png")
        }
    }
}
"""

SKIA_SRC = """
impl RenderBackend for SkiaBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            ..BackendCapabilities::raster("skia")
        }
    }
}
"""

MOD_CORE = """
pub mod backends;
pub mod svg_adapter;
pub use backends::{NullBackend, TraceBackend};
pub use svg_adapter::SvgBackend;
"""

MOD_WITH_OPTIONAL = MOD_CORE + """
pub mod png_adapter;
pub mod skia_adapter;
pub use png_adapter::PngBackend;
pub use skia_adapter::SkiaBackend;
"""

SCENE = {
    "schemaVersion": 1,
    "kind": "adapterDiffScene",
    "page": {"width": 400, "height": 300},
    "marker": "M06-4",
    "ops": [{"type": "rectangle", "x": 20, "y": 20, "w": 40, "h": 24}],
    "expectedFamilies": {
        "svg": "vector",
        "null": "none",
        "trace": "none",
        "png": "raster",
        "skia": "raster",
    },
}


def _scene(root: Path) -> Path:
    path = root / "tools" / "adapter_diff" / "fixtures" / "ci-scene.json"
    _write(path, json.dumps(SCENE))
    return path


def _core_tree(root: Path, mod: str = MOD_CORE) -> None:
    _write(root / "src" / "render_backend" / "svg_adapter.rs", SVG_SRC)
    _write(root / "src" / "render_backend" / "backends.rs", BACKENDS_SRC)
    _write(root / "src" / "render_backend" / "mod.rs", mod)


class DiscoverTests(unittest.TestCase):
    def test_missing_optional_adapters_are_skipped_honestly(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _core_tree(root)
            scene = SCENE
            rows = {item.name: item for item in ad.discover(root, scene)}
            self.assertEqual(rows["svg"].status, "present")
            self.assertEqual(rows["svg"].verdict, "FAMILY_OK")
            self.assertEqual(rows["null"].status, "present")
            self.assertEqual(rows["trace"].status, "present")
            self.assertEqual(rows["png"].status, "skipped_missing")
            self.assertEqual(rows["png"].verdict, "SKIPPED_MISSING")
            self.assertEqual(rows["skia"].status, "skipped_missing")
            self.assertIn("정직한 skip", rows["png"].note)

    def test_present_optional_adapters_are_compared(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _core_tree(root, MOD_WITH_OPTIONAL)
            _write(root / "src" / "render_backend" / "png_adapter.rs", PNG_SRC)
            _write(root / "src" / "render_backend" / "skia_adapter.rs", SKIA_SRC)
            rows = {item.name: item for item in ad.discover(root, SCENE)}
            self.assertEqual(rows["png"].status, "present")
            self.assertEqual(rows["png"].family, "raster")
            self.assertEqual(rows["png"].verdict, "FAMILY_OK")
            self.assertEqual(rows["skia"].status, "present")
            self.assertEqual(rows["skia"].family, "raster")

    def test_unexported_file_is_not_pretended_present(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _core_tree(root)
            _write(root / "src" / "render_backend" / "png_adapter.rs", PNG_SRC)
            rows = {item.name: item for item in ad.discover(root, SCENE)}
            self.assertEqual(rows["png"].status, "skipped_unexported")
            self.assertEqual(rows["png"].verdict, "SKIPPED_UNEXPORTED")

    def test_family_mismatch_is_data_not_skip(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _core_tree(root)
            bad = SVG_SRC.replace('vector("svg")', 'raster("svg")')
            _write(root / "src" / "render_backend" / "svg_adapter.rs", bad)
            rows = {item.name: item for item in ad.discover(root, SCENE)}
            self.assertEqual(rows["svg"].verdict, "FAMILY_MISMATCH")
            self.assertEqual(rows["svg"].status, "present")


class PairTests(unittest.TestCase):
    def test_cross_family_does_not_compare_output_hash(self) -> None:
        adapters = [
            ad.AdapterStatus("svg", "svg.rs", "SvgBackend", True, "present", "vector"),
            ad.AdapterStatus("png", "png.rs", "PngBackend", False, "present", "raster"),
        ]
        rows = ad.compare_pairs(adapters, SCENE)
        hash_rows = [row for row in rows if row.axis == "output_hash"]
        self.assertTrue(hash_rows)
        self.assertTrue(all(row.verdict.startswith("SKIP") for row in hash_rows))

    def test_same_family_hash_is_left_to_rust(self) -> None:
        adapters = [
            ad.AdapterStatus("png", "png.rs", "PngBackend", False, "present", "raster"),
            ad.AdapterStatus("skia", "skia.rs", "SkiaBackend", False, "present", "raster"),
        ]
        rows = ad.compare_pairs(adapters, SCENE)
        hash_rows = [row for row in rows if row.axis == "output_hash"]
        self.assertEqual(hash_rows[0].verdict, "SKIP_LIVE")


class RunTests(unittest.TestCase):
    def test_ci_scene_on_devel_like_tree(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _core_tree(root)
            scene = _scene(root)
            report = ad.run(root, scene, strict=True)
            self.assertEqual(report.summary["present"], 3)
            self.assertEqual(report.summary["skipped_missing"], 2)
            self.assertEqual(report.summary["family_mismatch"], 0)
            self.assertEqual(ad.exit_code(report), 0)
            payload = report.to_json()
            self.assertEqual(payload["kind"], "adapterDiffReport")
            self.assertEqual(payload["summary"]["skipped_missing"], 2)
            names = {item["name"]: item["status"] for item in payload["adapters"]}
            self.assertEqual(names["png"], "skipped_missing")
            self.assertEqual(names["skia"], "skipped_missing")

    def test_missing_required_adapter_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(root / "src" / "render_backend" / "backends.rs", BACKENDS_SRC)
            _write(root / "src" / "render_backend" / "mod.rs", MOD_CORE)
            scene = _scene(root)
            report = ad.run(root, scene, strict=False)
            self.assertEqual(ad.exit_code(report), 1)
            svg = next(item for item in report.adapters if item.name == "svg")
            self.assertEqual(svg.status, "skipped_missing")

    def test_strict_family_mismatch_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _core_tree(root)
            bad = SVG_SRC.replace('vector("svg")', 'none("svg")')
            _write(root / "src" / "render_backend" / "svg_adapter.rs", bad)
            scene = _scene(root)
            report = ad.run(root, scene, strict=True)
            self.assertEqual(ad.exit_code(report), 1)
            self.assertEqual(ad.exit_code(ad.run(root, scene, strict=False)), 0)

    def test_bad_scene_is_usage_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "bad.json"
            _write(path, '{"kind":"nope"}')
            with self.assertRaises(ad.AdapterDiffError):
                ad.load_scene(path)


class RepoFixtureTests(unittest.TestCase):
    def test_committed_ci_scene_is_cheap(self) -> None:
        scene = ad.load_scene(HERE / "fixtures" / "ci-scene.json")
        self.assertEqual(scene["page"]["width"], 400)
        self.assertEqual(scene["page"]["height"], 300)
        self.assertLessEqual(len(scene["ops"]), 4)
        self.assertNotIn("samples/", json.dumps(scene.get("ops")))


if __name__ == "__main__":
    unittest.main()
