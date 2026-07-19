#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import shutil
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SAMPLES_DIR = ROOT / "samples"
STUDIO_ROOT = ROOT / "rhwp-studio"
DEFAULT_MANIFEST = ROOT / "scripts" / "renderer_baseline_manifest.json"
DEFAULT_OUTPUT = ROOT / "output" / "renderer-baseline" / "latest"
NPM_CMD = "npm.cmd" if sys.platform == "win32" else "npm"
ALLOWED_PROFILES = ("screen", "print", "high-quality", "fast-preview")
ALLOWED_CANVASKIT_SURFACES = ("auto", "webgpu", "webgl", "software")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Capture a fixed multi-backend renderer baseline for transition hardening."
    )
    parser.add_argument(
        "--manifest",
        default=str(DEFAULT_MANIFEST),
        help="baseline manifest JSON path",
    )
    parser.add_argument(
        "--output",
        default=str(DEFAULT_OUTPUT),
        help="output directory for captured artifacts",
    )
    parser.add_argument(
        "--filter",
        default="",
        help="case-insensitive literal filter applied to sample id/file/category",
    )
    parser.add_argument(
        "--scope",
        choices=("representative", "full"),
        default="representative",
        help="capture the bounded representative tier or the complete corpus",
    )
    parser.add_argument(
        "--browser-mode",
        choices=("host", "headless"),
        default="headless",
        help="browser capture mode for rhwp-studio baseline screenshots",
    )
    parser.add_argument(
        "--profiles",
        default="screen,fast-preview",
        help="comma-separated layered render profiles to capture "
        f"({', '.join(ALLOWED_PROFILES)})",
    )
    parser.add_argument(
        "--canvaskit-surface",
        default=os.environ.get("RHWP_CANVASKIT_SURFACE", "auto"),
        help="CanvasKit surface preference for browser captures "
        f"({', '.join(ALLOWED_CANVASKIT_SURFACES)}; aliases: gpu=webgpu, sw/cpu=software)",
    )
    parser.add_argument(
        "--skip-native",
        action="store_true",
        help="skip legacy svg / layer svg / native skia captures",
    )
    parser.add_argument(
        "--skip-browser",
        action="store_true",
        help="skip canvas2d / canvaskit browser captures",
    )
    parser.add_argument(
        "--include-pdf",
        action="store_true",
        help="also capture print-profile PDF artifacts in the native baseline matrix",
    )
    parser.add_argument(
        "--readiness-only",
        action="store_true",
        help="capture only manifest entries selected for the CanvasKit readiness gate",
    )
    return parser.parse_args()


def parse_profiles(raw: str) -> list[str]:
    profiles = [item.strip().lower() for item in raw.split(",") if item.strip()]
    if not profiles:
        raise SystemExit("at least one layered render profile must be specified")

    invalid = [profile for profile in profiles if profile not in ALLOWED_PROFILES]
    if invalid:
        raise SystemExit(
            "unsupported layered render profile(s): "
            + ", ".join(invalid)
            + f" (allowed: {', '.join(ALLOWED_PROFILES)})"
        )

    seen: set[str] = set()
    ordered: list[str] = []
    for profile in profiles:
        if profile in seen:
            continue
        seen.add(profile)
        ordered.append(profile)
    return ordered


def load_manifest(
    manifest_path: Path,
    filter_pattern: str,
    scope: str,
    readiness_only: bool = False,
) -> dict:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schemaVersion") != 1:
        raise SystemExit("baseline manifest schemaVersion must be 1")
    samples = manifest.get("samples", [])
    if not isinstance(samples, list) or not samples:
        raise SystemExit("baseline manifest must contain a non-empty samples array")

    filter_text = filter_pattern.strip().lower()
    selected = []
    seen_sample_ids: set[str] = set()
    for sample in samples:
        file_name = str(sample.get("file") or "").strip()
        if (
            not file_name
            or "\0" in file_name
            or "\\" in file_name
            or "?" in file_name
            or "#" in file_name
        ):
            raise SystemExit(f"invalid baseline sample file: {sample['file']}")
        if Path(file_name).is_absolute() or urllib.parse.urlsplit(file_name).scheme:
            raise SystemExit(
                f"baseline sample file must be relative to samples/: {sample['file']}"
            )
        if urllib.parse.unquote(file_name) != file_name:
            raise SystemExit(
                f"baseline sample file must not use percent-encoding: {sample['file']}"
            )
        parts = file_name.split("/")
        if any(not part or part in (".", "..") for part in parts):
            raise SystemExit(f"baseline sample file escapes samples/: {sample['file']}")
        sample_path = (SAMPLES_DIR / file_name).resolve()
        try:
            sample_path.relative_to(SAMPLES_DIR.resolve())
        except ValueError:
            raise SystemExit(f"baseline sample file escapes samples/: {sample['file']}") from None
        if not sample_path.is_file():
            raise SystemExit(f"baseline sample file not found: {sample_path}")

        sample_id = str(sample.get("id") or Path(file_name).stem).strip()
        if not sample_id or any(
            not (char.isascii() and (char.isalnum() or char in "._-"))
            for char in sample_id
        ):
            raise SystemExit(f"invalid baseline sample id: {sample.get('id')}")
        if sample_id in seen_sample_ids:
            raise SystemExit(f"duplicate baseline sample id: {sample_id}")
        seen_sample_ids.add(sample_id)
        category = sample.get("category", "uncategorized")
        baseline_tier = sample.get("baselineTier")
        if baseline_tier not in ("representative", "extended"):
            raise SystemExit(f"invalid baselineTier for baseline sample: {sample_id}")
        page = sample.get("page", 0)
        if type(page) is not int or page < 0:
            raise SystemExit(
                f"baseline sample page must be a non-negative integer: {sample_id} page={page}"
            )
        diagnostic_axes = sample.get("diagnosticAxes")
        if (
            not isinstance(diagnostic_axes, list)
            or not diagnostic_axes
            or len(diagnostic_axes) > 16
            or any(
                not isinstance(axis, str)
                or not axis
                or len(axis) > 64
                or any(
                    not (char.isascii() and (char.isalnum() or char in "._-"))
                    for char in axis
                )
                for axis in diagnostic_axes
            )
            or len(set(diagnostic_axes)) != len(diagnostic_axes)
        ):
            raise SystemExit(f"invalid diagnosticAxes for baseline sample: {sample_id}")
        document_digest = f"sha256:{hashlib.sha256(sample_path.read_bytes()).hexdigest()}"
        manifest_digest = sample.get("documentDigest")
        if manifest_digest is not None and manifest_digest != document_digest:
            raise SystemExit(f"baseline sample documentDigest mismatch: {sample_id}")
        if readiness_only and sample.get("canvaskitReadinessGate") is not True:
            continue
        if scope == "representative" and baseline_tier != "representative":
            continue
        minimum_ink_pixels = (sample.get("browserParityThresholds") or {}).get(
            "minimumInkPixels"
        )
        if readiness_only and (
            type(minimum_ink_pixels) is not int or minimum_ink_pixels <= 0
        ):
            raise SystemExit(
                f"readiness sample {sample_id} requires a positive minimumInkPixels threshold"
            )
        if readiness_only:
            performance_budget = sample.get("canvaskitPerformanceBudget")
            required_budget_keys = {
                "maxColdDocumentLoadAndInitialRenderMs",
                "maxWarmReplayMs",
                "maxWarmRendererDurationMs",
                "maxImageCachePixels",
            }
            if not isinstance(performance_budget, dict) or set(performance_budget) != required_budget_keys:
                raise SystemExit(
                    f"readiness sample {sample_id} requires a complete CanvasKit performance budget"
                )
            if any(
                not isinstance(value, (int, float))
                or isinstance(value, bool)
                or not math.isfinite(value)
                or value <= 0
                for value in performance_budget.values()
            ):
                raise SystemExit(
                    f"readiness sample {sample_id} CanvasKit performance budgets must be positive"
                )
            readiness_expectations = sample.get("canvaskitReadinessExpectations")
            if readiness_expectations is not None:
                allowed_expectation_keys = {
                    "glyphOutlinePayloadKinds",
                    "minWarmImageCacheHits",
                }
                if (
                    not isinstance(readiness_expectations, dict)
                    or not set(readiness_expectations).issubset(allowed_expectation_keys)
                ):
                    raise SystemExit(
                        f"readiness sample {sample_id} has invalid CanvasKit expectations"
                    )
                payload_kinds = readiness_expectations.get("glyphOutlinePayloadKinds", [])
                if (
                    not isinstance(payload_kinds, list)
                    or any(kind not in {"bitmapGlyph", "svgGlyph"} for kind in payload_kinds)
                ):
                    raise SystemExit(
                        f"readiness sample {sample_id} has invalid glyph payload expectations"
                    )
                min_cache_hits = readiness_expectations.get("minWarmImageCacheHits", 0)
                if (
                    not isinstance(min_cache_hits, int)
                    or isinstance(min_cache_hits, bool)
                    or min_cache_hits < 0
                ):
                    raise SystemExit(
                        f"readiness sample {sample_id} has invalid warm image cache expectations"
                    )
        if filter_text and not (
            filter_text in str(sample_id).lower()
            or filter_text in str(file_name).lower()
            or filter_text in str(category).lower()
        ):
            continue
        selected_sample = dict(sample)
        selected_sample.update(
            {
                "id": sample_id,
                "file": file_name,
                "category": category,
                "baselineTier": baseline_tier,
                "page": page,
                "diagnosticAxes": diagnostic_axes,
                "documentDigest": document_digest,
                "notes": sample.get("notes", ""),
            }
        )
        selected.append(selected_sample)

    if not selected:
        raise SystemExit("sample filter removed every manifest entry")

    manifest["samples"] = selected
    return manifest


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def command_env(extra: dict[str, str] | None = None) -> dict[str, str]:
    env = os.environ.copy()
    if extra:
        env.update(extra)
    return env


def log_command(cmd: list[str], cwd: Path) -> None:
    printable = " ".join(cmd)
    print(f"$ (cd {cwd} && {printable})", flush=True)


def run_command(cmd: list[str], cwd: Path, extra_env: dict[str, str] | None = None) -> None:
    log_command(cmd, cwd)
    subprocess.run(cmd, cwd=cwd, env=command_env(extra_env), check=True)


def collect_files(output_dir: Path, suffix: str) -> list[str]:
    collected: list[str] = []
    for path in output_dir.glob(f"*{suffix}"):
        try:
            collected.append(str(path.relative_to(ROOT)))
        except ValueError:
            collected.append(str(path))
    return sorted(collected)


def capture_native_sample(
    sample: dict, output_root: Path, profiles: list[str], include_pdf: bool
) -> list[dict]:
    sample_path = (SAMPLES_DIR / sample["file"]).resolve()
    try:
        sample_path.relative_to(SAMPLES_DIR.resolve())
    except ValueError:
        raise SystemExit(f"sample file escapes samples/: {sample['file']}") from None
    if not sample_path.exists():
        raise SystemExit(f"sample file not found: {sample_path}")

    outputs: list[dict] = []
    target_page = str(sample["page"])

    legacy_dir = output_root / sample["id"] / "legacy-svg"
    if legacy_dir.exists():
        shutil.rmtree(legacy_dir)
    ensure_dir(legacy_dir)
    run_command(
        [
            "cargo",
            "run",
            "--bin",
            "rhwp",
            "--",
            "export-svg",
            str(sample_path),
            "--page",
            target_page,
            "--output",
            str(legacy_dir),
        ],
        ROOT,
    )
    legacy_files = collect_files(legacy_dir, ".svg")
    if len(legacy_files) != 1:
        raise SystemExit(
            "legacy SVG baseline export must create exactly one artifact: "
            + ", ".join(legacy_files)
        )
    legacy_path = Path(legacy_files[0])
    if not legacy_path.is_absolute():
        legacy_path = ROOT / legacy_path
    if not legacy_path.is_file() or legacy_path.stat().st_size == 0:
        raise SystemExit(
            f"legacy SVG baseline export did not create a non-empty artifact: {legacy_path}"
        )
    outputs.append(
        {
            "backend": "legacy-svg",
            "profile": None,
            "comparisonIdentity": {
                "schemaVersion": 1,
                "sampleId": sample["id"],
                "documentDigest": sample["documentDigest"],
                "page": sample["page"],
                "profile": None,
                "backend": "legacy-svg",
                "surface": None,
            },
            "artifact": {
                "sha256": hashlib.sha256(legacy_path.read_bytes()).hexdigest(),
                "sizeBytes": legacy_path.stat().st_size,
            },
            "files": legacy_files,
        }
    )

    if include_pdf:
        pdf_dir = output_root / sample["id"] / "pdf-print"
        if pdf_dir.exists():
            shutil.rmtree(pdf_dir)
        ensure_dir(pdf_dir)
        pdf_path = pdf_dir / f"{sample['id']}-page-{target_page}.pdf"
        run_command(
            [
                "cargo",
                "run",
                "--bin",
                "rhwp",
                "--",
                "export-pdf",
                str(sample_path),
                "--page",
                target_page,
                "--profile",
                "print",
                "--output",
                str(pdf_path),
            ],
            ROOT,
        )
        if not pdf_path.is_file() or pdf_path.stat().st_size == 0:
            raise SystemExit(
                f"PDF baseline export did not create a non-empty artifact: {pdf_path}"
            )
        outputs.append(
            {
                "backend": "pdf",
                "profile": "print",
                "comparisonIdentity": {
                    "schemaVersion": 1,
                    "sampleId": sample["id"],
                    "documentDigest": sample["documentDigest"],
                    "page": sample["page"],
                    "profile": "print",
                    "backend": "pdf",
                    "surface": "vector",
                },
                "artifact": {
                    "sha256": hashlib.sha256(pdf_path.read_bytes()).hexdigest(),
                    "sizeBytes": pdf_path.stat().st_size,
                },
                "files": collect_files(pdf_dir, ".pdf"),
            }
        )

    for profile in profiles:
        layer_dir = output_root / sample["id"] / f"layer-svg-{profile}"
        if layer_dir.exists():
            shutil.rmtree(layer_dir)
        ensure_dir(layer_dir)
        run_command(
            [
                "cargo",
                "run",
                "--bin",
                "rhwp",
                "--",
                "export-svg",
                str(sample_path),
                "--page",
                target_page,
                "--profile",
                profile,
                "--output",
                str(layer_dir),
            ],
            ROOT,
        )
        layer_files = collect_files(layer_dir, ".svg")
        if len(layer_files) != 1:
            raise SystemExit(
                f"layer SVG ({profile}) baseline export must create exactly one artifact: "
                + ", ".join(layer_files)
            )
        layer_path = Path(layer_files[0])
        if not layer_path.is_absolute():
            layer_path = ROOT / layer_path
        if not layer_path.is_file() or layer_path.stat().st_size == 0:
            raise SystemExit(
                f"layer SVG ({profile}) baseline export did not create a non-empty artifact: "
                f"{layer_path}"
            )
        outputs.append(
            {
                "backend": "layer-svg",
                "profile": profile,
                "comparisonIdentity": {
                    "schemaVersion": 1,
                    "sampleId": sample["id"],
                    "documentDigest": sample["documentDigest"],
                    "page": sample["page"],
                    "profile": profile,
                    "backend": "layer-svg",
                    "surface": None,
                },
                "artifact": {
                    "sha256": hashlib.sha256(layer_path.read_bytes()).hexdigest(),
                    "sizeBytes": layer_path.stat().st_size,
                },
                "files": layer_files,
            }
        )

    for profile in profiles:
        skia_dir = output_root / sample["id"] / f"native-skia-{profile}"
        if skia_dir.exists():
            shutil.rmtree(skia_dir)
        ensure_dir(skia_dir)
        run_command(
            [
                "cargo",
                "run",
                "--features",
                "native-skia",
                "--bin",
                "rhwp",
                "--",
                "export-png",
                str(sample_path),
                "--page",
                target_page,
                "--profile",
                profile,
                "--output",
                str(skia_dir),
            ],
            ROOT,
        )
        skia_files = collect_files(skia_dir, ".png")
        if len(skia_files) != 1:
            raise SystemExit(
                f"native Skia ({profile}) baseline export must create exactly one artifact: "
                + ", ".join(skia_files)
            )
        skia_path = Path(skia_files[0])
        if not skia_path.is_absolute():
            skia_path = ROOT / skia_path
        if not skia_path.is_file() or skia_path.stat().st_size == 0:
            raise SystemExit(
                f"native Skia ({profile}) baseline export did not create a non-empty artifact: "
                f"{skia_path}"
            )
        outputs.append(
            {
                "backend": "native-skia",
                "profile": profile,
                "comparisonIdentity": {
                    "schemaVersion": 1,
                    "sampleId": sample["id"],
                    "documentDigest": sample["documentDigest"],
                    "page": sample["page"],
                    "profile": profile,
                    "backend": "native-skia",
                    "surface": "raster",
                },
                "artifact": {
                    "sha256": hashlib.sha256(skia_path.read_bytes()).hexdigest(),
                    "sizeBytes": skia_path.stat().st_size,
                },
                "files": skia_files,
            }
        )

    return outputs


def find_available_port(start_port: int = 7700, attempts: int = 20) -> int:
    for port in range(start_port, start_port + attempts):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            try:
                sock.bind(("127.0.0.1", port))
            except OSError:
                continue
            return port
    raise SystemExit(f"failed to find an available port starting at {start_port}")


def wait_for_server(url: str, timeout_sec: float = 30.0) -> None:
    deadline = time.time() + timeout_sec
    last_error: Exception | None = None
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(url) as response:
                if response.status < 500:
                    return
        except (urllib.error.URLError, TimeoutError) as error:
            last_error = error
        time.sleep(0.5)
    raise SystemExit(f"timed out waiting for {url}: {last_error}")


def stop_process(child: subprocess.Popen[bytes]) -> None:
    if child.poll() is not None:
        return
    child.terminate()
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait(timeout=5)


def capture_browser_baseline(
    manifest_path: Path,
    output_root: Path,
    browser_mode: str,
    filter_pattern: str,
    scope: str,
    profiles: list[str],
    canvaskit_surface: str,
    readiness_only: bool,
) -> tuple[Path, bool]:
    port = find_available_port()
    vite_url = f"http://127.0.0.1:{port}"
    report_path = output_root / "browser-baseline-report.json"
    report_path.unlink(missing_ok=True)
    dev_server = subprocess.Popen(
        [
            NPM_CMD,
            "run",
            "dev",
            "--",
            "--host",
            "0.0.0.0",
            "--port",
            str(port),
            "--strictPort",
        ],
        cwd=STUDIO_ROOT,
        env=command_env({"BROWSER": "none"}),
    )
    try:
        wait_for_server(vite_url)
        cmd = [
            "node",
            "e2e/renderer-baseline.mjs",
            f"--mode={browser_mode}",
            f"--manifest={manifest_path}",
            f"--output={output_root}",
            f"--profiles={','.join(profiles)}",
            f"--scope={scope}",
            f"--canvaskit-surface={canvaskit_surface}",
        ]
        if filter_pattern:
            cmd.append(f"--filter={filter_pattern}")
        if readiness_only:
            cmd.append("--readiness-only")
        log_command(cmd, STUDIO_ROOT)
        completed = subprocess.run(
            cmd,
            cwd=STUDIO_ROOT,
            env=command_env(
                {
                    "VITE_URL": vite_url,
                    "RHWP_CANVASKIT_SURFACE": canvaskit_surface,
                }
            ),
            check=False,
        )
    finally:
        stop_process(dev_server)
    if completed.returncode == 0:
        if not report_path.exists():
            raise RuntimeError(f"browser baseline did not write report: {report_path}")
        return report_path, True
    if report_path.exists():
        return report_path, False
    raise subprocess.CalledProcessError(completed.returncode, cmd)


def repo_relative(path_value: str | Path) -> str:
    path = Path(path_value)
    if not path.is_absolute():
        path = (ROOT / path).resolve()
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def format_ms(value: object) -> str:
    if not isinstance(value, (int, float)):
        return "-"
    return f"{value:.1f}"


def format_count(value: object) -> str:
    if not isinstance(value, int):
        return "-"
    return str(value)


def run_native_canvaskit_parity_report(
    native_results: list[dict],
    browser_report: Path | None,
    output_root: Path,
    profiles: list[str],
) -> Path | None:
    if not native_results or not browser_report or not browser_report.exists():
        return None

    native_results_path = output_root / "native-results.json"
    native_results_path.write_text(
        json.dumps(native_results, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )
    parity_report_path = output_root / "native-canvaskit-parity-report.json"
    run_command(
        [
            "node",
            "e2e/renderer-baseline-native-diff.mjs",
            f"--native={native_results_path}",
            f"--browser={browser_report}",
            f"--output={parity_report_path}",
            f"--profiles={','.join(profiles)}",
        ],
        STUDIO_ROOT,
    )
    return parity_report_path


def write_reports(
    manifest: dict,
    output_root: Path,
    native_results: list[dict],
    browser_report: Path | None,
    profiles: list[str],
    parity_report: Path | None,
    canvaskit_surface: str,
) -> None:
    browser_data = None
    if browser_report and browser_report.exists():
        browser_data = json.loads(browser_report.read_text(encoding="utf-8"))
    parity_data = None
    if parity_report and parity_report.exists():
        parity_data = json.loads(parity_report.read_text(encoding="utf-8"))
    browser_replay_diagnostics = (
        browser_data.get("canvaskitReplayDiagnostics") if browser_data else None
    ) or {}

    browser_performance_summary: list[dict] = []
    browser_surface_diagnostics_summary: list[dict] = []
    if browser_data and browser_data.get("results"):
        summary_by_key: dict[tuple[str, str], dict] = {}
        surface_summary_by_key: dict[tuple[str, str, str, str], dict] = {}
        for item in browser_data["results"]:
            backend = item.get("backend", "")
            profile = item.get("profile", "")
            key = (backend, profile)
            summary = summary_by_key.setdefault(
                key,
                {
                    "backend": backend,
                    "profile": profile,
                    "sampleCount": 0,
                    "appLoadMsTotal": 0.0,
                    "documentLoadAndInitialRenderMsTotal": 0.0,
                    "selectedPageRenderMsTotal": 0.0,
                    "screenshotMsTotal": 0.0,
                    "totalMsTotal": 0.0,
                    "effectPixelsTotal": 0,
                    "effectCacheHitsTotal": 0,
                    "effectCacheMissesTotal": 0,
                    "effectFailuresTotal": 0,
                },
            )
            summary["sampleCount"] += 1
            timings = item.get("timings") or {}
            for field in (
                "appLoadMs",
                "documentLoadAndInitialRenderMs",
                "selectedPageRenderMs",
                "screenshotMs",
                "totalMs",
            ):
                value = timings.get(field)
                if isinstance(value, (int, float)):
                    summary[f"{field}Total"] += float(value)

            image_effects = (item.get("diagnostics") or {}).get("imageEffects") or {}
            if backend == "canvas2d":
                effect_diagnostics = image_effects.get("canvas2d") or {}
            else:
                effect_diagnostics = image_effects.get("canvaskit") or {}
            if isinstance(effect_diagnostics.get("preprocessedPixels"), int):
                summary["effectPixelsTotal"] += effect_diagnostics["preprocessedPixels"]
            if isinstance(effect_diagnostics.get("cacheHits"), int):
                summary["effectCacheHitsTotal"] += effect_diagnostics["cacheHits"]
            if isinstance(effect_diagnostics.get("cacheMisses"), int):
                summary["effectCacheMissesTotal"] += effect_diagnostics["cacheMisses"]
            if isinstance(effect_diagnostics.get("preprocessFailures"), int):
                summary["effectFailuresTotal"] += effect_diagnostics["preprocessFailures"]

            if str(backend).startswith("canvaskit"):
                diagnostics = item.get("diagnostics") or {}
                surface_diagnostics = diagnostics.get("surfaceDiagnostics") or {}
                render_diagnostics = diagnostics.get("canvaskitRender") or {}
                preference = (
                    surface_diagnostics.get("preference")
                    or render_diagnostics.get("surfacePreference")
                    or "-"
                )
                surface_backend = (
                    surface_diagnostics.get("backend")
                    or render_diagnostics.get("surfaceBackend")
                    or "-"
                )
                surface_key = (
                    backend,
                    profile,
                    str(preference),
                    str(surface_backend),
                )
                surface_summary = surface_summary_by_key.setdefault(
                    surface_key,
                    {
                        "backend": backend,
                        "profile": profile,
                        "preference": preference,
                        "surfaceBackend": surface_backend,
                        "sampleCount": 0,
                        "usedGpuSurfaceCount": 0,
                        "createdSurfacesTotal": 0,
                        "reusedSurfacesTotal": 0,
                        "webgpuAttemptsTotal": 0,
                        "webgpuFailuresTotal": 0,
                        "webglAttemptsTotal": 0,
                        "webglFailuresTotal": 0,
                        "softwareAttemptsTotal": 0,
                        "softwareFailuresTotal": 0,
                        "softwareFallbacksTotal": 0,
                        "webgpuFailureExamples": [],
                        "webglFailureExamples": [],
                        "softwareFailureExamples": [],
                        "lastFailureExamples": [],
                    },
                )
                surface_summary["sampleCount"] += 1
                if surface_diagnostics.get("usedGpuSurface") is True:
                    surface_summary["usedGpuSurfaceCount"] += 1
                for source_field, target_field in (
                    ("createdSurfaces", "createdSurfacesTotal"),
                    ("reusedSurfaces", "reusedSurfacesTotal"),
                    ("webgpuAttempts", "webgpuAttemptsTotal"),
                    ("webgpuFailures", "webgpuFailuresTotal"),
                    ("webglAttempts", "webglAttemptsTotal"),
                    ("webglFailures", "webglFailuresTotal"),
                    ("softwareAttempts", "softwareAttemptsTotal"),
                    ("softwareFailures", "softwareFailuresTotal"),
                    ("softwareFallbacks", "softwareFallbacksTotal"),
                ):
                    value = surface_diagnostics.get(source_field)
                    if isinstance(value, int):
                        surface_summary[target_field] += value
                for source_field, target_field in (
                    ("webgpuLastFailure", "webgpuFailureExamples"),
                    ("webglLastFailure", "webglFailureExamples"),
                    ("softwareLastFailure", "softwareFailureExamples"),
                ):
                    failure = surface_diagnostics.get(source_field)
                    if (
                        isinstance(failure, str)
                        and failure
                        and failure not in surface_summary[target_field]
                        and len(surface_summary[target_field]) < 3
                    ):
                        surface_summary[target_field].append(failure)
                last_failure = surface_diagnostics.get("lastFailure")
                if not last_failure:
                    last_failure = render_diagnostics.get("surfaceFallbackReason")
                    if last_failure and surface_backend == "software":
                        surface_summary["softwareFallbacksTotal"] += 1
                if (
                    isinstance(last_failure, str)
                    and last_failure
                    and last_failure not in surface_summary["lastFailureExamples"]
                    and len(surface_summary["lastFailureExamples"]) < 3
                ):
                    surface_summary["lastFailureExamples"].append(last_failure)

        for summary in summary_by_key.values():
            sample_count = max(1, summary["sampleCount"])
            browser_performance_summary.append(
                {
                    "backend": summary["backend"],
                    "profile": summary["profile"],
                    "sampleCount": summary["sampleCount"],
                    "averageAppLoadMs": summary["appLoadMsTotal"] / sample_count,
                    "averageDocumentLoadAndInitialRenderMs": summary[
                        "documentLoadAndInitialRenderMsTotal"
                    ]
                    / sample_count,
                    "averageSelectedPageRenderMs": summary[
                        "selectedPageRenderMsTotal"
                    ]
                    / sample_count,
                    "averageScreenshotMs": summary["screenshotMsTotal"] / sample_count,
                    "averageTotalMs": summary["totalMsTotal"] / sample_count,
                    "effectPixelsTotal": summary["effectPixelsTotal"],
                    "effectCacheHitsTotal": summary["effectCacheHitsTotal"],
                    "effectCacheMissesTotal": summary["effectCacheMissesTotal"],
                    "effectFailuresTotal": summary["effectFailuresTotal"],
                }
            )
        browser_performance_summary.sort(
            key=lambda item: (item["profile"], item["backend"])
        )
        browser_surface_diagnostics_summary = sorted(
            surface_summary_by_key.values(),
            key=lambda item: (
                item["profile"],
                item["backend"],
                item["preference"],
                item["surfaceBackend"],
            ),
        )

    effective_canvaskit_surface = canvaskit_surface
    if browser_data and isinstance(browser_data.get("canvaskitSurface"), str):
        effective_canvaskit_surface = browser_data["canvaskitSurface"]

    performance_summary = {
        "browser": browser_performance_summary,
        "canvaskitSurface": effective_canvaskit_surface,
        "canvaskitSurfaceDiagnostics": browser_surface_diagnostics_summary,
        "canvaskitReplayDiagnostics": browser_replay_diagnostics,
    }
    (output_root / "performance-summary.json").write_text(
        json.dumps(performance_summary, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )

    report_json = {
        "manifest": manifest,
        "canvaskitSurface": effective_canvaskit_surface,
        "native": native_results,
        "browser": browser_data,
        "performance": performance_summary,
        "nativeCanvasKitParity": parity_data,
    }
    (output_root / "baseline-report.json").write_text(
        json.dumps(report_json, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )

    browser_metadata = []
    if browser_data and browser_data.get("browserVersion"):
        browser_metadata.append(f"- browser: `{browser_data['browserVersion']}`")
    if browser_data and browser_data.get("chromiumBuildId"):
        browser_metadata.append(
            f"- Chromium build ID: `{browser_data['chromiumBuildId']}`"
        )
    if browser_data and browser_data.get("captureError"):
        browser_metadata.append(f"- browser capture error: `{browser_data['captureError']}`")

    lines = [
        f"# Renderer Baseline: {manifest.get('label', 'unnamed')}",
        "",
        manifest.get("description", ""),
        "",
        f"- manifest: `{repo_relative(manifest['_path']) if manifest.get('_path') else 'n/a'}`",
        f"- samples: {len(manifest['samples'])}",
        f"- layered profiles: {', '.join(profiles)}",
        f"- CanvasKit surface: `{effective_canvaskit_surface}`",
        *browser_metadata,
        "",
        "## Sample Matrix",
        "",
        "| Sample | Category | Page | Diagnostic Axes | Document Digest | Native Outputs | Browser Outputs |",
        "| --- | --- | ---: | --- | --- | --- | --- |",
    ]

    browser_by_sample: dict[str, list[str]] = {}
    if browser_data:
        for item in browser_data.get("results", []):
            browser_by_sample.setdefault(item["sampleId"], []).append(item["path"])

    for sample in manifest["samples"]:
        native_entries = next((entry for entry in native_results if entry["sampleId"] == sample["id"]), None)
        native_paths: list[str] = []
        if native_entries:
            for backend in native_entries["backends"]:
                prefix = backend["backend"]
                if backend.get("profile"):
                    prefix = f"{prefix}@{backend['profile']}"
                native_paths.extend(f"{prefix}: {path}" for path in backend["files"])
        browser_paths = browser_by_sample.get(sample["id"], [])
        native_text = "<br>".join(f"`{path}`" for path in native_paths) or "-"
        browser_text = "<br>".join(f"`{repo_relative(path)}`" for path in browser_paths) or "-"
        lines.append(
            "| "
            + " | ".join(
                [
                    sample["id"],
                    sample["category"],
                    str(sample["page"]),
                    ", ".join(sample["diagnosticAxes"]),
                    f"`{sample['documentDigest']}`",
                    native_text,
                    browser_text,
                ]
            )
            + " |"
        )

    if browser_data and browser_data.get("results"):
        lines.extend(
            [
                "",
                "## Browser Performance",
                "",
                "| Sample | Backend | Profile | Page | App Load ms | Document Load + Initial Render ms | Selected Page Render ms | Warm Replay ms | Warm Renderer ms | Screenshot ms | Total ms | Effect Pixels | Effect Cache Hits | Effect Cache Misses | Effect Failures |",
                "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in browser_data["results"]:
            timings = item.get("timings") or {}
            image_effects = (item.get("diagnostics") or {}).get("imageEffects") or {}
            backend = item.get("backend", "")
            if backend == "canvas2d":
                effect_diagnostics = image_effects.get("canvas2d") or {}
            else:
                effect_diagnostics = image_effects.get("canvaskit") or {}

            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("sampleId", "-"),
                        backend or "-",
                        item.get("profile", "-"),
                        format_count(item.get("page")),
                        format_ms(timings.get("appLoadMs")),
                        format_ms(timings.get("documentLoadAndInitialRenderMs")),
                        format_ms(timings.get("selectedPageRenderMs")),
                        format_ms(timings.get("warmReplayMs")),
                        format_ms(timings.get("warmRendererDurationMs")),
                        format_ms(timings.get("screenshotMs")),
                        format_ms(timings.get("totalMs")),
                        format_count(effect_diagnostics.get("preprocessedPixels")),
                        format_count(effect_diagnostics.get("cacheHits")),
                        format_count(effect_diagnostics.get("cacheMisses")),
                        format_count(effect_diagnostics.get("preprocessFailures")),
                    ]
                )
                + " |"
            )

        lines.extend(
            [
                "",
                "## Browser Performance Summary",
                "",
                "| Backend | Profile | Samples | Avg App Load ms | Avg Document Load + Initial Render ms | Avg Selected Page Render ms | Avg Screenshot ms | Avg Total ms | Effect Pixels | Effect Cache Hits | Effect Cache Misses | Effect Failures |",
                "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in browser_performance_summary:
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("backend", "-"),
                        item.get("profile", "-"),
                        format_count(item.get("sampleCount")),
                        format_ms(item.get("averageAppLoadMs")),
                        format_ms(item.get("averageDocumentLoadAndInitialRenderMs")),
                        format_ms(item.get("averageSelectedPageRenderMs")),
                        format_ms(item.get("averageScreenshotMs")),
                        format_ms(item.get("averageTotalMs")),
                        format_count(item.get("effectPixelsTotal")),
                        format_count(item.get("effectCacheHitsTotal")),
                        format_count(item.get("effectCacheMissesTotal")),
                        format_count(item.get("effectFailuresTotal")),
                    ]
                )
                + " |"
            )
        if browser_surface_diagnostics_summary:
            lines.extend(
                [
                    "",
                    "## CanvasKit Surface Diagnostics Summary",
                    "",
                    "| Backend | Profile | Preference | Surface Backend | Samples | Confirmed GPU Samples | Created | Reused | WebGPU Attempts | WebGPU Failures | WebGPU Failures Seen | WebGL Attempts | WebGL Failures | WebGL Failures Seen | Software Attempts | Software Failures | Software Fallbacks | Last Failures Seen |",
                    "| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | --- | ---: | ---: | ---: | --- |",
                ]
            )
            for item in browser_surface_diagnostics_summary:
                lines.append(
                    "| "
                    + " | ".join(
                        [
                            item.get("backend", "-"),
                            item.get("profile", "-"),
                            item.get("preference", "-"),
                            item.get("surfaceBackend", "-"),
                            format_count(item.get("sampleCount")),
                            format_count(item.get("usedGpuSurfaceCount")),
                            format_count(item.get("createdSurfacesTotal")),
                            format_count(item.get("reusedSurfacesTotal")),
                            format_count(item.get("webgpuAttemptsTotal")),
                            format_count(item.get("webgpuFailuresTotal")),
                            "<br>".join(item.get("webgpuFailureExamples") or []) or "-",
                            format_count(item.get("webglAttemptsTotal")),
                            format_count(item.get("webglFailuresTotal")),
                            "<br>".join(item.get("webglFailureExamples") or []) or "-",
                            format_count(item.get("softwareAttemptsTotal")),
                            format_count(item.get("softwareFailuresTotal")),
                            format_count(item.get("softwareFallbacksTotal")),
                            "<br>".join(item.get("lastFailureExamples") or []) or "-",
                        ]
                    )
                    + " |"
                )

    replay_summary_rows = browser_replay_diagnostics.get("summaryByBackendProfile") or []
    if replay_summary_rows:
        lines.extend(
            [
                "",
                "## CanvasKit Replay Diagnostics",
                "",
                f"- mode: `{browser_replay_diagnostics.get('mode', '-')}`",
                f"- hard-gate violations: {browser_replay_diagnostics.get('hardGateViolationCount', 0)}",
                "",
                "| Backend | Profile | Captures | Items | Direct | Direct Required | Text Fallback | Unsupported | Compat Overlay | Hidden Overlay Violations | Runtime Errors | Unexpected Runtime Ops |",
                "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in replay_summary_rows:
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("backend", "-"),
                        item.get("profile", "-"),
                        format_count(item.get("captureCount")),
                        format_count(item.get("totalItems")),
                        format_count(item.get("directItems")),
                        format_count(item.get("directRequiredItems")),
                        format_count(item.get("textFallbackItems")),
                        format_count(item.get("unsupportedItems")),
                        format_count(item.get("compatOverlayItems")),
                        format_count(item.get("hiddenOverlayViolations")),
                        format_count(item.get("runtimeRenderErrors")),
                        format_count(item.get("runtimeUnexpectedUnsupportedOps")),
                    ]
                )
                + " |"
            )

        lines.extend(
            [
                "",
                "### Replay Diagnostic Inventory",
                "",
                "| Backend | Profile | Plan Statuses | Plan Reasons | Plan Features | Expected Runtime Ops | Unexpected Runtime Ops |",
                "| --- | --- | --- | --- | --- | --- | --- |",
            ]
        )
        for item in replay_summary_rows:
            inventory_columns = []
            for field in (
                "planStatusCounts",
                "planReasonCounts",
                "planFeatureCounts",
                "expectedUnsupportedOpCounts",
                "unexpectedUnsupportedOpCounts",
            ):
                counts = item.get(field) or {}
                inventory_columns.append(
                    "<br>".join(
                        f"`{reason}`: {count}" for reason, count in sorted(counts.items())
                    )
                    or "-"
                )
            lines.append(
                "| "
                + " | ".join(
                    [item.get("backend", "-"), item.get("profile", "-"), *inventory_columns]
                )
                + " |"
            )

    browser_canvaskit_readiness = (
        browser_data.get("canvaskitReadinessGate") if browser_data else None
    )
    if browser_canvaskit_readiness:
        summary = browser_canvaskit_readiness.get("summary") or {}
        criteria = browser_canvaskit_readiness.get("criteria") or {}
        lines.extend(
            [
                "",
                "## CanvasKit Readiness Gate",
                "",
                f"- mode: `{browser_canvaskit_readiness.get('mode', 'selectedCorpus')}`",
                f"- profile: `{criteria.get('profile', '-')}`",
                f"- target backend: `{criteria.get('targetBackend', '-')}`",
                f"- surface: `{criteria.get('canvaskitSurface', '-')}`",
                f"- total: {summary.get('total', 0)}",
                f"- evaluated: {summary.get('evaluated', 0)}",
                f"- passed: {summary.get('passed', 0)}",
                f"- failed: {summary.get('failed', 0)}",
                f"- missing: {summary.get('missing', 0)}",
                "",
                "| Sample | Category | Backend | Profile | Active Backend | Canvas Owned | Surface Backend | Surface Fallback | Passed | Blockers | Expected Gaps | Unexpected Gaps | Diff Ratio | Expected Ink | Actual Ink | Min Ink | Ink Floor | Cold ms | Warm Replay ms | Warm Renderer ms | Image Cache Pixels |",
                "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in browser_canvaskit_readiness.get("checks", []):
            selected_diff_ratio = item.get("selectedDiffRatio")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("sampleId", "-"),
                        item.get("category", "-"),
                        item.get("targetBackend", "-"),
                        item.get("profile", "-"),
                        item.get("activeBackend") or "-",
                        "yes" if item.get("canvasOwnershipTracked") else "no",
                        item.get("surfaceBackend") or "-",
                        item.get("surfaceFallbackReason") or "-",
                        "yes" if item.get("passed") else "no",
                        "<br>".join(item.get("blockers") or []) or "-",
                        "<br>".join(item.get("expectedUnsupportedOps") or []) or "-",
                        "<br>".join(item.get("unexpectedUnsupportedOps") or []) or "-",
                        f"{selected_diff_ratio:.6f}"
                        if isinstance(selected_diff_ratio, (int, float))
                        else "-",
                        format_count(item.get("expectedInkPixels")),
                        format_count(item.get("actualInkPixels")),
                        format_count(item.get("minimumInkPixels")),
                        "yes" if item.get("minimumInkBudgetPassed") else "no",
                        format_ms(item.get("coldDocumentLoadAndInitialRenderMs")),
                        format_ms((item.get("warmReplay") or {}).get("replayMs")),
                        format_ms((item.get("warmReplay") or {}).get("rendererDurationMs")),
                        format_count((item.get("warmReplay") or {}).get("imageCachePixels")),
                    ]
                )
                + " |"
            )

    browser_backend_parity = (
        browser_data.get("browserBackendParity") if browser_data else None
    )
    if browser_backend_parity:
        summary = browser_backend_parity.get("summary") or {}
        thresholds = browser_backend_parity.get("thresholds") or {}
        lines.extend(
            [
                "",
                "## Browser Canvas2D vs CanvasKit Fuzzy Parity",
                "",
                f"- mode: `{browser_backend_parity.get('mode', 'reportOnly')}`",
                f"- default ignore channel delta: {thresholds.get('ignoreChannelDelta', '-')}",
                f"- default max diff ratio: {thresholds.get('maxDiffRatio', '-')}",
                f"- compared: {summary.get('compared', 0)}",
                f"- passed: {summary.get('passed', 0)}",
                f"- failed: {summary.get('failed', 0)}",
                f"- missing: {summary.get('missing', 0)}",
                f"- errors: {summary.get('errors', 0)}",
                f"- identity mismatches: {summary.get('identityMismatches', 0)}",
                "",
                "### Target Backend Summary",
                "",
                "| Target Backend | Total | Compared | Passed | Failed | Missing | Errors | Identity Mismatches | Worst Selected Diff Ratio | Worst Raw Diff Ratio | Worst Channel Delta |",
                "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in browser_backend_parity.get("summaryByTargetBackend", []):
            worst_ratio = item.get("worstSelectedDiffRatio")
            worst_raw_ratio = item.get("worstTolerantDiffRatio")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("targetBackend", "-"),
                        format_count(item.get("total")),
                        format_count(item.get("compared")),
                        format_count(item.get("passed")),
                        format_count(item.get("failed")),
                        format_count(item.get("missing")),
                        format_count(item.get("errors")),
                        format_count(item.get("identityMismatches")),
                        f"{worst_ratio:.6f}" if isinstance(worst_ratio, (int, float)) else "-",
                        f"{worst_raw_ratio:.6f}" if isinstance(worst_raw_ratio, (int, float)) else "-",
                        format_count(item.get("worstMaxChannelDelta")),
                    ]
                )
                + " |"
            )
        lines.extend(
            [
                "",
                "### Profile Summary",
                "",
                "| Profile | Total | Compared | Passed | Failed | Missing | Errors | Identity Mismatches | Worst Selected Diff Ratio | Worst Raw Diff Ratio | Worst Channel Delta |",
                "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in browser_backend_parity.get("summaryByProfile", []):
            worst_ratio = item.get("worstSelectedDiffRatio")
            worst_raw_ratio = item.get("worstTolerantDiffRatio")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("profile", "-"),
                        format_count(item.get("total")),
                        format_count(item.get("compared")),
                        format_count(item.get("passed")),
                        format_count(item.get("failed")),
                        format_count(item.get("missing")),
                        format_count(item.get("errors")),
                        format_count(item.get("identityMismatches")),
                        f"{worst_ratio:.6f}" if isinstance(worst_ratio, (int, float)) else "-",
                        f"{worst_raw_ratio:.6f}" if isinstance(worst_raw_ratio, (int, float)) else "-",
                        format_count(item.get("worstMaxChannelDelta")),
                    ]
                )
                + " |"
            )
        category_summary = browser_backend_parity.get("summaryByCategory") or []
        if category_summary:
            lines.extend(
                [
                    "",
                    "### Category Summary",
                    "",
                    "| Category | Total | Compared | Passed | Failed | Missing | Errors | Identity Mismatches | Worst Selected Diff Ratio | Worst Raw Diff Ratio | Worst Channel Delta |",
                    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
                ]
            )
            for item in category_summary:
                worst_ratio = item.get("worstSelectedDiffRatio")
                worst_raw_ratio = item.get("worstTolerantDiffRatio")
                lines.append(
                    "| "
                    + " | ".join(
                        [
                            item.get("category") or "-",
                            format_count(item.get("total")),
                            format_count(item.get("compared")),
                            format_count(item.get("passed")),
                            format_count(item.get("failed")),
                            format_count(item.get("missing")),
                            format_count(item.get("errors")),
                            format_count(item.get("identityMismatches")),
                            f"{worst_ratio:.6f}" if isinstance(worst_ratio, (int, float)) else "-",
                            f"{worst_raw_ratio:.6f}" if isinstance(worst_raw_ratio, (int, float)) else "-",
                            format_count(item.get("worstMaxChannelDelta")),
                        ]
                    )
                    + " |"
                )
        diagnostic_axis_summary = (
            browser_backend_parity.get("summaryByDiagnosticAxis") or []
        )
        if diagnostic_axis_summary:
            lines.extend(
                [
                    "",
                    "### Diagnostic Axis Summary",
                    "",
                    "| Axis | Total | Compared | Passed | Failed | Missing | Errors | Identity Mismatches | Worst Selected Diff Ratio |",
                    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
                ]
            )
            for item in diagnostic_axis_summary:
                worst_ratio = item.get("worstSelectedDiffRatio")
                lines.append(
                    "| "
                    + " | ".join(
                        [
                            item.get("diagnosticAxis") or "-",
                            format_count(item.get("total")),
                            format_count(item.get("compared")),
                            format_count(item.get("passed")),
                            format_count(item.get("failed")),
                            format_count(item.get("missing")),
                            format_count(item.get("errors")),
                            format_count(item.get("identityMismatches")),
                            f"{worst_ratio:.6f}"
                            if isinstance(worst_ratio, (int, float))
                            else "-",
                        ]
                    )
                    + " |"
                )
        lines.extend(
            [
                "",
                "### Worst Comparisons",
                "",
                "| Sample | Category | Profile | Target Backend | Surface | Passed | Diff Pixels | Selected Diff Ratio | Raw Diff Ratio | Max Channel Delta | Mean Abs Channel Delta |",
                "| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in browser_backend_parity.get("worstComparisons", []):
            diff_ratio = item.get("selectedDiffRatio")
            tolerant_ratio = item.get("tolerantDiffRatio")
            mean_abs = item.get("meanAbsChannelDelta")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("sampleId", "-"),
                        item.get("category") or "-",
                        item.get("profile", "-"),
                        item.get("targetBackend", "-"),
                        item.get("canvaskitSurface") or "-",
                        "yes" if item.get("passed") else "no",
                        format_count(item.get("selectedDiffPixels")),
                        f"{diff_ratio:.6f}" if isinstance(diff_ratio, (int, float)) else "-",
                        f"{tolerant_ratio:.6f}" if isinstance(tolerant_ratio, (int, float)) else "-",
                        format_count(item.get("maxChannelDelta")),
                        f"{mean_abs:.3f}" if isinstance(mean_abs, (int, float)) else "-",
                    ]
                )
                + " |"
            )
        lines.extend(
            [
                "",
                "### Comparisons",
                "",
                "| Sample | Category | Profile | Target Backend | Surface | Status | Passed | Diff Pixels | Selected Diff Ratio | Raw Diff Ratio | Max Diff Ratio | Ink Mask Max Ratio | Non-Ink Max Pixels | Solid Ink Max Ratio | Max Channel Delta |",
                "| --- | --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in browser_backend_parity.get("comparisons", []):
            diff = item.get("diff") or {}
            item_thresholds = item.get("thresholds") or thresholds
            passed = "-"
            if "passed" in diff:
                passed = "yes" if diff.get("passed") else "no"
            diff_pixels = diff.get("selectedDiffPixels")
            diff_ratio = diff.get("selectedDiffRatio")
            tolerant_ratio = diff.get("tolerantDiffRatio")
            max_channel_delta = diff.get("maxChannelDelta")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("sampleId", "-"),
                        item.get("category") or "-",
                        item.get("profile", "-"),
                        item.get("targetBackend", "-"),
                        item.get("canvaskitSurface") or "-",
                        item.get("status", "-"),
                        passed,
                        format_count(diff_pixels),
                        f"{diff_ratio:.6f}" if isinstance(diff_ratio, (int, float)) else "-",
                        f"{tolerant_ratio:.6f}" if isinstance(tolerant_ratio, (int, float)) else "-",
                        (
                            f"{item_thresholds.get('maxDiffRatio'):.6f}"
                            if isinstance(item_thresholds.get("maxDiffRatio"), (int, float))
                            else "-"
                        ),
                        (
                            f"{item_thresholds.get('inkMaskMaxDiffRatio'):.6f}"
                            if isinstance(item_thresholds.get("inkMaskMaxDiffRatio"), (int, float))
                            else "-"
                        ),
                        format_count(item_thresholds.get("nonInkMaxDiffPixels")),
                        (
                            f"{item_thresholds.get('solidInkMaxDiffRatio'):.6f}"
                            if isinstance(item_thresholds.get("solidInkMaxDiffRatio"), (int, float))
                            else "-"
                        ),
                        format_count(max_channel_delta),
                    ]
                )
                + " |"
            )

    if parity_data:
        summary = parity_data.get("summary") or {}
        lines.extend(
            [
                "",
                "## Native Skia vs CanvasKit Fuzzy Parity",
                "",
                f"- mode: `{parity_data.get('mode', 'reportOnly')}`",
                f"- CanvasKit surface: `{parity_data.get('canvaskitSurface') or effective_canvaskit_surface}`",
                f"- compared: {summary.get('compared', 0)}",
                f"- passed: {summary.get('passed', 0)}",
                f"- failed: {summary.get('failed', 0)}",
                f"- missing: {summary.get('missing', 0)}",
                f"- errors: {summary.get('errors', 0)}",
                f"- identity mismatches: {summary.get('identityMismatches', 0)}",
                "",
                "### Profile Summary",
                "",
                "| Profile | Total | Compared | Passed | Failed | Missing | Errors | Worst Diff Ratio | Worst Channel Delta |",
                "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in parity_data.get("summaryByProfile", []):
            worst_ratio = item.get("worstSelectedDiffRatio")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("profile", "-"),
                        format_count(item.get("total")),
                        format_count(item.get("compared")),
                        format_count(item.get("passed")),
                        format_count(item.get("failed")),
                        format_count(item.get("missing")),
                        format_count(item.get("errors")),
                        f"{worst_ratio:.6f}" if isinstance(worst_ratio, (int, float)) else "-",
                        format_count(item.get("worstMaxChannelDelta")),
                    ]
                )
                + " |"
            )
        category_summary = parity_data.get("summaryByCategory") or []
        if category_summary:
            lines.extend(
                [
                    "",
                    "### Category Summary",
                    "",
                    "| Category | Total | Compared | Passed | Failed | Missing | Errors | Worst Diff Ratio | Worst Channel Delta |",
                    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
                ]
            )
            for item in category_summary:
                worst_ratio = item.get("worstSelectedDiffRatio")
                lines.append(
                    "| "
                    + " | ".join(
                        [
                            item.get("category") or "-",
                            format_count(item.get("total")),
                            format_count(item.get("compared")),
                            format_count(item.get("passed")),
                            format_count(item.get("failed")),
                            format_count(item.get("missing")),
                            format_count(item.get("errors")),
                            f"{worst_ratio:.6f}" if isinstance(worst_ratio, (int, float)) else "-",
                            format_count(item.get("worstMaxChannelDelta")),
                        ]
                    )
                    + " |"
                )
        diagnostic_axis_summary = parity_data.get("summaryByDiagnosticAxis") or []
        if diagnostic_axis_summary:
            lines.extend(
                [
                    "",
                    "### Diagnostic Axis Summary",
                    "",
                    "| Axis | Total | Compared | Passed | Failed | Missing | Errors | Identity Mismatches | Worst Diff Ratio |",
                    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
                ]
            )
            for item in diagnostic_axis_summary:
                worst_ratio = item.get("worstSelectedDiffRatio")
                lines.append(
                    "| "
                    + " | ".join(
                        [
                            item.get("diagnosticAxis") or "-",
                            format_count(item.get("total")),
                            format_count(item.get("compared")),
                            format_count(item.get("passed")),
                            format_count(item.get("failed")),
                            format_count(item.get("missing")),
                            format_count(item.get("errors")),
                            format_count(item.get("identityMismatches")),
                            f"{worst_ratio:.6f}"
                            if isinstance(worst_ratio, (int, float))
                            else "-",
                        ]
                    )
                    + " |"
                )
        surface_summary = parity_data.get("summaryByCanvasKitSurface") or []
        if surface_summary:
            lines.extend(
                [
                    "",
                    "### CanvasKit Surface Summary",
                    "",
                    "| Surface | Total | Compared | Passed | Failed | Missing | Errors | Worst Diff Ratio | Worst Channel Delta |",
                    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
                ]
            )
            for item in surface_summary:
                worst_ratio = item.get("worstSelectedDiffRatio")
                lines.append(
                    "| "
                    + " | ".join(
                        [
                            item.get("canvaskitSurface") or "-",
                            format_count(item.get("total")),
                            format_count(item.get("compared")),
                            format_count(item.get("passed")),
                            format_count(item.get("failed")),
                            format_count(item.get("missing")),
                            format_count(item.get("errors")),
                            f"{worst_ratio:.6f}" if isinstance(worst_ratio, (int, float)) else "-",
                            format_count(item.get("worstMaxChannelDelta")),
                        ]
                    )
                    + " |"
                )

        lines.extend(
            [
                "",
                "### Worst Comparisons",
                "",
                "| Sample | Category | Profile | Surface | Passed | Diff Pixels | Diff Ratio | Max Channel Delta | Mean Abs Channel Delta |",
                "| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |",
            ]
        )
        for item in parity_data.get("worstComparisons", []):
            diff_ratio = item.get("selectedDiffRatio")
            mean_abs = item.get("meanAbsChannelDelta")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("sampleId", "-"),
                        item.get("category") or "-",
                        item.get("profile", "-"),
                        item.get("canvaskitSurface") or "-",
                        "yes" if item.get("passed") else "no",
                        format_count(item.get("selectedDiffPixels")),
                        f"{diff_ratio:.6f}" if isinstance(diff_ratio, (int, float)) else "-",
                        format_count(item.get("maxChannelDelta")),
                        f"{mean_abs:.3f}" if isinstance(mean_abs, (int, float)) else "-",
                    ]
                )
                + " |"
            )

        lines.extend(
            [
                "",
                "### Comparisons",
                "",
                "| Sample | Category | Profile | Surface | Status | Passed | Diff Pixels | Diff Ratio | Max Channel Delta |",
                "| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: |",
            ]
        )
        for item in parity_data.get("comparisons", []):
            diff = item.get("diff") or {}
            passed = "-"
            if "passed" in diff:
                passed = "yes" if diff.get("passed") else "no"
            diff_pixels = diff.get("selectedDiffPixels")
            diff_ratio = diff.get("selectedDiffRatio")
            max_channel_delta = diff.get("maxChannelDelta")
            lines.append(
                "| "
                + " | ".join(
                    [
                        item.get("sampleId", "-"),
                        item.get("category") or "-",
                        item.get("profile", "-"),
                        item.get("canvaskitSurface") or "-",
                        item.get("status", "-"),
                        passed,
                        format_count(diff_pixels),
                        f"{diff_ratio:.6f}" if isinstance(diff_ratio, (int, float)) else "-",
                        format_count(max_channel_delta),
                    ]
                )
                + " |"
            )

    (output_root / "baseline-report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    args = parse_args()
    manifest_path = Path(args.manifest).resolve()
    output_root = Path(args.output).resolve()
    profiles = parse_profiles(args.profiles)
    canvaskit_surface = str(args.canvaskit_surface).strip().lower()
    if canvaskit_surface in ("sw", "cpu"):
        canvaskit_surface = "software"
    elif canvaskit_surface == "gpu":
        canvaskit_surface = "webgpu"
    if canvaskit_surface not in ALLOWED_CANVASKIT_SURFACES:
        raise SystemExit(
            "unsupported CanvasKit surface: "
            + canvaskit_surface
            + f" (allowed: {', '.join(ALLOWED_CANVASKIT_SURFACES)}; aliases: gpu, sw, cpu)"
        )
    if args.readiness_only and profiles != ["screen"]:
        raise SystemExit("--readiness-only requires --profiles=screen")
    if args.readiness_only and canvaskit_surface != "auto":
        raise SystemExit("--readiness-only requires --canvaskit-surface=auto")
    if args.readiness_only and args.skip_browser:
        raise SystemExit("--readiness-only cannot be combined with --skip-browser")
    if args.readiness_only and args.filter.strip():
        raise SystemExit("--readiness-only cannot be combined with --filter")
    ensure_dir(output_root)

    manifest = load_manifest(manifest_path, args.filter, args.scope, args.readiness_only)
    manifest["_path"] = str(manifest_path)
    shutil.copy2(manifest_path, output_root / manifest_path.name)

    filtered_manifest_path = output_root / "baseline-manifest.filtered.json"
    filtered_manifest_path.write_text(
        json.dumps(
            {
                key: value
                for key, value in manifest.items()
                if key != "_path"
            },
            indent=2,
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )

    native_results: list[dict] = []
    if not args.skip_native and not args.readiness_only:
        for sample in manifest["samples"]:
            print(f"\n[native] {sample['id']} ({sample['category']})", flush=True)
            backends = capture_native_sample(
                sample, output_root, profiles, args.include_pdf
            )
            native_results.append(
                {
                    "sampleId": sample["id"],
                    "category": sample["category"],
                    "page": sample["page"],
                    "documentDigest": sample["documentDigest"],
                    "diagnosticAxes": sample["diagnosticAxes"],
                    "backends": backends,
                }
            )

    browser_report: Path | None = None
    browser_capture_passed = True
    if not args.skip_browser:
        print("\n[browser] capturing canvas2d/canvaskit baseline", flush=True)
        browser_report, browser_capture_passed = capture_browser_baseline(
            filtered_manifest_path,
            output_root / "browser",
            args.browser_mode,
            args.filter,
            args.scope,
            profiles,
            canvaskit_surface,
            args.readiness_only,
        )

    parity_report = run_native_canvaskit_parity_report(
        native_results, browser_report, output_root, profiles
    )
    write_reports(
        manifest,
        output_root,
        native_results,
        browser_report,
        profiles,
        parity_report,
        canvaskit_surface,
    )
    print(f"\n[baseline] complete: {output_root}", flush=True)
    if not browser_capture_passed:
        failure = "CanvasKit readiness gate" if args.readiness_only else "renderer baseline contract"
        raise SystemExit(f"{failure} failed; see {output_root / 'baseline-report.md'}")


if __name__ == "__main__":
    main()
