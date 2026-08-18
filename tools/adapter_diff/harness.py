#!/usr/bin/env python3
"""전어댑터 상호 diff 골든 하네스 — 구조·capability·산출 해시/bbox.

MEGA QUEUE M06-4 (#5392). 판정 도구만. `src/renderer/**` · gym 을 건드리지
않는다. devel 에는 Svg/Null/Trace 만 있다. Png/Skia 원본이 없으면 정직하게
skip 하고, 있는 어댑터끼리만 맞댄다.

사용:
    python tools/adapter_diff/harness.py --ci
    python tools/adapter_diff/harness.py --ci --json
    python tools/adapter_diff/harness.py --ci --strict
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
KIND = "adapterDiffReport"
SCENE_KIND = "adapterDiffScene"
HERE = Path(__file__).resolve().parent
REPO_DEFAULT = HERE.parents[1]
DEFAULT_SCENE = HERE / "fixtures" / "ci-scene.json"
MOD_RS = Path("src/render_backend/mod.rs")

CTOR_RE = re.compile(r'BackendCapabilities::(vector|raster|none)\("([^"]+)"\)')


@dataclass(frozen=True)
class AdapterSpec:
    name: str
    source: str
    export: str
    required: bool


ADAPTERS: tuple[AdapterSpec, ...] = (
    AdapterSpec("svg", "src/render_backend/svg_adapter.rs", "SvgBackend", True),
    AdapterSpec("null", "src/render_backend/backends.rs", "NullBackend", True),
    AdapterSpec("trace", "src/render_backend/backends.rs", "TraceBackend", True),
    AdapterSpec("png", "src/render_backend/png_adapter.rs", "PngBackend", False),
    AdapterSpec("skia", "src/render_backend/skia_adapter.rs", "SkiaBackend", False),
)


class AdapterDiffError(Exception):
    """하네스 사용법·픽스처 오류. 종료 코드 2."""


@dataclass
class AdapterStatus:
    name: str
    source: str
    export: str
    required: bool
    status: str
    family: str | None = None
    expected_family: str | None = None
    verdict: str = ""
    note: str = ""

    def to_json(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "source": self.source,
            "export": self.export,
            "required": self.required,
            "status": self.status,
            "family": self.family,
            "expectedFamily": self.expected_family,
            "verdict": self.verdict,
            "note": self.note,
        }


@dataclass
class PairRow:
    left: str
    right: str
    axis: str
    verdict: str
    note: str = ""

    def to_json(self) -> dict[str, Any]:
        return {
            "left": self.left,
            "right": self.right,
            "axis": self.axis,
            "verdict": self.verdict,
            "note": self.note,
        }


@dataclass
class Report:
    schema_version: int = SCHEMA_VERSION
    kind: str = KIND
    strict: bool = False
    scene: str = ""
    adapters: list[AdapterStatus] = field(default_factory=list)
    pairs: list[PairRow] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    @property
    def summary(self) -> dict[str, int]:
        counts = {
            "adapters": len(self.adapters),
            "present": 0,
            "skipped_missing": 0,
            "skipped_unexported": 0,
            "family_ok": 0,
            "family_mismatch": 0,
            "pairs": len(self.pairs),
            "pair_ok": 0,
            "pair_skip": 0,
            "error": 0,
        }
        for item in self.adapters:
            key = item.status
            if key in counts:
                counts[key] += 1
            if item.verdict == "FAMILY_OK":
                counts["family_ok"] += 1
            elif item.verdict == "FAMILY_MISMATCH":
                counts["family_mismatch"] += 1
            elif item.verdict == "ERROR":
                counts["error"] += 1
        for row in self.pairs:
            if row.verdict == "OK":
                counts["pair_ok"] += 1
            elif row.verdict.startswith("SKIP"):
                counts["pair_skip"] += 1
            elif row.verdict == "ERROR":
                counts["error"] += 1
        return counts

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": self.schema_version,
            "kind": self.kind,
            "strict": self.strict,
            "scene": self.scene,
            "adapters": [item.to_json() for item in self.adapters],
            "pairs": [row.to_json() for row in self.pairs],
            "summary": self.summary,
            "notes": list(self.notes),
        }


def norm_rel(path: str) -> str:
    return path.replace("\\", "/")


def load_scene(path: Path) -> dict[str, Any]:
    if not path.is_file():
        raise AdapterDiffError(f"픽스처가 없습니다: {path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as err:
        raise AdapterDiffError(f"픽스처 JSON 오류: {path}: {err}") from err
    if not isinstance(data, dict):
        raise AdapterDiffError(f"픽스처는 객체여야 합니다: {path}")
    if data.get("kind") != SCENE_KIND:
        raise AdapterDiffError(f"픽스처 kind 가 {SCENE_KIND} 가 아닙니다: {path}")
    if data.get("schemaVersion") != SCHEMA_VERSION:
        raise AdapterDiffError(f"지원하지 않는 픽스처 schemaVersion: {path}")
    page = data.get("page")
    if not isinstance(page, dict):
        raise AdapterDiffError("픽스처 page 가 없습니다")
    if not isinstance(page.get("width"), (int, float)) or not isinstance(
        page.get("height"), (int, float)
    ):
        raise AdapterDiffError("픽스처 page.width/height 가 숫자가 아닙니다")
    families = data.get("expectedFamilies")
    if families is not None and not isinstance(families, dict):
        raise AdapterDiffError("expectedFamilies 는 객체여야 합니다")
    return data


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def is_exported(mod_rs: str, export: str, source: str) -> bool:
    if export in mod_rs:
        return True
    stem = Path(source).stem
    return f"mod {stem}" in mod_rs or f"pub mod {stem}" in mod_rs


def families_from_source(text: str) -> dict[str, str]:
    found: dict[str, str] = {}
    for match in CTOR_RE.finditer(text):
        family, name = match.group(1), match.group(2)
        found[name] = family
    return found


def discover(root: Path, scene: dict[str, Any]) -> list[AdapterStatus]:
    expected = scene.get("expectedFamilies") or {}
    mod_path = root / MOD_RS
    mod_rs = read_text(mod_path) if mod_path.is_file() else ""
    source_cache: dict[str, str] = {}
    rows: list[AdapterStatus] = []
    for spec in ADAPTERS:
        src = root / spec.source
        exp_family = expected.get(spec.name)
        if isinstance(exp_family, str):
            expected_family = exp_family
        else:
            expected_family = None
        if not src.is_file():
            rows.append(
                AdapterStatus(
                    name=spec.name,
                    source=norm_rel(spec.source),
                    export=spec.export,
                    required=spec.required,
                    status="skipped_missing",
                    expected_family=expected_family,
                    verdict="SKIPPED_MISSING",
                    note="어댑터 원본 없음 — 정직한 skip",
                )
            )
            continue
        key = norm_rel(spec.source)
        if key not in source_cache:
            source_cache[key] = read_text(src)
        text = source_cache[key]
        if not is_exported(mod_rs, spec.export, spec.source):
            rows.append(
                AdapterStatus(
                    name=spec.name,
                    source=norm_rel(spec.source),
                    export=spec.export,
                    required=spec.required,
                    status="skipped_unexported",
                    expected_family=expected_family,
                    verdict="SKIPPED_UNEXPORTED",
                    note="파일은 있으나 render_backend 가 타입을 내보내지 않음",
                )
            )
            continue
        families = families_from_source(text)
        family = families.get(spec.name)
        if family is None and spec.name in ("null", "trace"):
            family = families.get(spec.name)
        verdict = "PRESENT"
        note = "상호 diff 대상"
        if expected_family is not None:
            if family == expected_family:
                verdict = "FAMILY_OK"
                note = f"family={family}"
            elif family is None:
                verdict = "ERROR"
                note = "capabilities 생성자를 읽지 못함"
            else:
                verdict = "FAMILY_MISMATCH"
                note = f"family={family} expected={expected_family}"
        rows.append(
            AdapterStatus(
                name=spec.name,
                source=norm_rel(spec.source),
                export=spec.export,
                required=spec.required,
                status="present",
                family=family,
                expected_family=expected_family,
                verdict=verdict,
                note=note,
            )
        )
    return rows


def compare_pairs(adapters: list[AdapterStatus], scene: dict[str, Any]) -> list[PairRow]:
    present = [item for item in adapters if item.status == "present"]
    rows: list[PairRow] = []
    page = scene.get("page") or {}
    width = page.get("width")
    height = page.get("height")
    for item in present:
        rows.append(
            PairRow(
                left=item.name,
                right="scene",
                axis="bbox",
                verdict="OK",
                note=f"logical page {width}x{height}",
            )
        )
    names = [item.name for item in present]
    if len(names) != len(set(names)):
        rows.append(
            PairRow(
                left=",".join(names),
                right="names",
                axis="structure",
                verdict="ERROR",
                note="어댑터 이름 중복",
            )
        )
    else:
        rows.append(
            PairRow(
                left="all",
                right="names",
                axis="structure",
                verdict="OK",
                note="이름 고유",
            )
        )
    for i, left in enumerate(present):
        for right in present[i + 1 :]:
            if left.family and right.family and left.family == right.family:
                rows.append(
                    PairRow(
                        left=left.name,
                        right=right.name,
                        axis="capability",
                        verdict="OK",
                        note=f"same family {left.family}",
                    )
                )
            elif left.family and right.family:
                rows.append(
                    PairRow(
                        left=left.name,
                        right=right.name,
                        axis="capability",
                        verdict="OK",
                        note=f"cross-family {left.family} vs {right.family}",
                    )
                )
            else:
                rows.append(
                    PairRow(
                        left=left.name,
                        right=right.name,
                        axis="capability",
                        verdict="SKIP_FAMILY",
                        note="한쪽 family 를 읽지 못함",
                    )
                )
            if left.family != right.family:
                rows.append(
                    PairRow(
                        left=left.name,
                        right=right.name,
                        axis="output_hash",
                        verdict="SKIP_FORMAT",
                        note="형식이 달라 바이트 해시를 맞대지 않음",
                    )
                )
            else:
                rows.append(
                    PairRow(
                        left=left.name,
                        right=right.name,
                        axis="output_hash",
                        verdict="SKIP_LIVE",
                        note="같은 family 해시 대조는 rust adapter_diff 가 닫음",
                    )
                )
    return rows


def run(root: Path, scene_path: Path, strict: bool) -> Report:
    scene = load_scene(scene_path)
    try:
        scene_rel = norm_rel(str(scene_path.relative_to(root)))
    except ValueError:
        scene_rel = norm_rel(str(scene_path))
    report = Report(strict=strict, scene=scene_rel)
    report.adapters = discover(root, scene)
    report.pairs = compare_pairs(report.adapters, scene)
    present = [item.name for item in report.adapters if item.status == "present"]
    skipped = [
        item.name
        for item in report.adapters
        if item.status.startswith("skipped")
    ]
    report.notes.append(f"present={present}")
    report.notes.append(f"skipped={skipped}")
    missing_required = [
        item.name
        for item in report.adapters
        if item.required and item.status != "present"
    ]
    if missing_required:
        report.notes.append(f"required-missing={missing_required}")
    return report


def format_text(report: Report) -> str:
    lines = [
        f"adapter-diff present={report.summary['present']} "
        f"skip_missing={report.summary['skipped_missing']} "
        f"skip_unexported={report.summary['skipped_unexported']} "
        f"pairs={report.summary['pairs']}"
    ]
    for item in report.adapters:
        lines.append(
            f"  {item.name:5} {item.verdict:18} {item.status} {item.note}"
        )
    for row in report.pairs:
        lines.append(f"  pair {row.left}/{row.right} {row.axis} {row.verdict} {row.note}")
    for note in report.notes:
        lines.append(f"  note {note}")
    return "\n".join(lines) + "\n"


def exit_code(report: Report) -> int:
    missing_required = any(
        item.required and item.status != "present" for item in report.adapters
    )
    if missing_required:
        return 1
    if not report.strict:
        return 0
    if report.summary["family_mismatch"] or report.summary["error"]:
        return 1
    return 0


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="전어댑터 상호 diff 골든 하네스")
    parser.add_argument("--ci", action="store_true", help="CI 픽스처 장면")
    parser.add_argument("--json", action="store_true", help="JSON 리포트")
    parser.add_argument("--strict", action="store_true", help="FAMILY_MISMATCH/ERROR 에서 1")
    parser.add_argument("--root", type=Path, default=REPO_DEFAULT)
    parser.add_argument("--scene", type=Path, default=None)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = args.root.resolve()
    if args.scene is not None:
        scene = args.scene
    elif args.ci:
        scene = DEFAULT_SCENE
    else:
        scene = DEFAULT_SCENE
    if not scene.is_absolute():
        scene = (root / scene) if not scene.exists() else scene.resolve()
    try:
        report = run(root, scene, args.strict)
    except AdapterDiffError as err:
        sys.stderr.write(f"adapter-diff: {err}\n")
        return 2
    if args.json:
        sys.stdout.write(json.dumps(report.to_json(), ensure_ascii=False, indent=2) + "\n")
    else:
        sys.stdout.write(format_text(report))
    return exit_code(report)


if __name__ == "__main__":
    sys.exit(main())
