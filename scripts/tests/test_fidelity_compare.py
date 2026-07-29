from __future__ import annotations

import contextlib
import importlib.util
import io
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

MODULE_PATH = (
    Path(__file__).resolve().parents[2]
    / "tools"
    / "fidelity_compare"
    / "fidelity_compare.py"
)
SPEC = importlib.util.spec_from_file_location("fidelity_compare", MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"fidelity_compare 모듈을 불러올 수 없습니다: {MODULE_PATH}")
FIDELITY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = FIDELITY
SPEC.loader.exec_module(FIDELITY)


class ExecutableDiscoveryTests(unittest.TestCase):
    def test_find_rhwp_uses_platform_specific_release_test_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory)
            binary = repo / "target" / "release-test" / "rhwp"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"binary")

            resolved = FIDELITY.find_rhwp(repo=repo, env={}, os_name="posix")

            self.assertEqual(resolved, str(binary))

    def test_find_rhwp_accepts_path_discovered_override(self) -> None:
        with mock.patch.object(
            FIDELITY.shutil, "which", return_value="/opt/rhwp/bin/rhwp"
        ):
            resolved = FIDELITY.find_rhwp(env={"RHWP_BIN": "rhwp-custom"})

        self.assertEqual(resolved, "/opt/rhwp/bin/rhwp")

    def test_find_chrome_uses_linux_path_lookup(self) -> None:
        def which(name: str) -> str | None:
            return "/usr/bin/chromium" if name == "chromium" else None

        with mock.patch.object(FIDELITY.shutil, "which", side_effect=which):
            resolved = FIDELITY.find_chrome(env={}, os_name="posix", platform="linux")

        self.assertEqual(resolved, "/usr/bin/chromium")

    def test_find_chrome_uses_windows_program_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            chrome = (
                Path(directory) / "Google" / "Chrome" / "Application" / "chrome.exe"
            )
            chrome.parent.mkdir(parents=True)
            chrome.write_bytes(b"binary")
            with mock.patch.object(FIDELITY.shutil, "which", return_value=None):
                resolved = FIDELITY.find_chrome(
                    env={"PROGRAMFILES": directory}, os_name="nt", platform="win32"
                )

        self.assertEqual(resolved, str(chrome))


class ChromeCaptureTests(unittest.TestCase):
    def test_capture_retries_once_and_surfaces_first_stderr(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.html"
            output = root / "capture.png"
            source.write_text("<html></html>", encoding="utf-8")
            calls = 0

            def fake_run(
                *_args: object, **_kwargs: object
            ) -> subprocess.CompletedProcess[str]:
                nonlocal calls
                calls += 1
                if calls == 1:
                    return subprocess.CompletedProcess(
                        [], 1, stdout="", stderr="first failure"
                    )
                output.write_bytes(b"png")
                return subprocess.CompletedProcess([], 0, stdout="", stderr="")

            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                succeeded = FIDELITY.capture_with_chrome(
                    "chrome", source, output, 800, 600, run=fake_run
                )

        self.assertTrue(succeeded)
        self.assertEqual(calls, 2)
        self.assertIn("first failure", stderr.getvalue())
        self.assertIn("1/2", stderr.getvalue())

    def test_capture_returns_false_after_two_failures(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.svg"
            output = root / "capture.png"
            source.write_text("<svg></svg>", encoding="utf-8")

            def fake_run(
                *_args: object, **_kwargs: object
            ) -> subprocess.CompletedProcess[str]:
                return subprocess.CompletedProcess(
                    [], 2, stdout="", stderr="still failing"
                )

            with contextlib.redirect_stderr(io.StringIO()):
                succeeded = FIDELITY.capture_with_chrome(
                    "chrome", source, output, 800, 600, run=fake_run
                )

        self.assertFalse(succeeded)
        self.assertFalse(output.exists())


class TextLayerComparisonTests(unittest.TestCase):
    def test_multiset_comparison_ignores_order_whitespace_and_unicode_form(
        self,
    ) -> None:
        reference = "A e\u0301 ·\n"
        rendered = "éA×"

        missing, extra = FIDELITY.compare_text_layers(reference, rendered)

        self.assertEqual(missing, {"·": 1})
        self.assertEqual(extra, {"×": 1})

    def test_svg_text_reads_only_text_elements(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            svg = Path(directory) / "page.svg"
            svg.write_text(
                '<svg xmlns="http://www.w3.org/2000/svg"><style>ignored</style>'
                '<text>A<tspan>가</tspan></text><path d="M0 0"/></svg>',
                encoding="utf-8",
            )

            extracted = FIDELITY.svg_text(svg)

        self.assertEqual(extracted, "A가")


class RegistryAndArgumentsTests(unittest.TestCase):
    def test_recognized_reference_patterns_use_pdf_directory_and_version_suffix(
        self,
    ) -> None:
        for key in ("plan", "manual", "korexam", "math", "eng"):
            with self.subTest(key=key):
                fixture = FIDELITY.REG[key]
                self.assertTrue(fixture.reference_pattern.startswith("pdf/"))
                self.assertRegex(fixture.reference_pattern, r"-20(?:22|24)\.pdf$")
                self.assertIn("기준 PDF", fixture.reference_grade)

    def test_legacy_sample_reference_is_explicitly_downgraded(self) -> None:
        fixture = FIDELITY.REG["bunjang"]

        self.assertTrue(fixture.reference_pattern.startswith("samples/"))
        self.assertIn("참고 PDF", fixture.reference_grade)
        self.assertIn("별도 확인 필요", fixture.reference_grade)

    def test_out_dir_is_parsed_as_exact_path(self) -> None:
        args = FIDELITY.parse_args(
            ["plan", "0", "9", "--out-dir", "/tmp/fidelity-plan"]
        )

        self.assertEqual(args.out_dir, Path("/tmp/fidelity-plan"))


if __name__ == "__main__":
    unittest.main()
