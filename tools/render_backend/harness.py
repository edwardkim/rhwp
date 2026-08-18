#!/usr/bin/env python3
"""M06-f 픽스처 정적 검사 — JSON 스키마·id 유일·기대 추적 형식.

Rust 통합 시험이 재생을 닫기 전에, 픽스처 디렉터리만으로 할 수 있는
불변식을 파이썬에서 먼저 본다. `src/renderer/**` 를 호출하지 않는다.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tests" / "fixtures" / "render_backend"

CATALOG = {
    "pageBackground",
    "textRun",
    "glyphRun",
    "glyphOutline",
    "charOverlap",
    "textControlMark",
    "tabLeader",
    "textDecoration",
    "footnoteMarker",
    "line",
    "rectangle",
    "ellipse",
    "path",
    "image",
    "equation",
    "formObject",
    "placeholder",
    "rawSvg",
}
PLANE = {"pageBackground": "background"}
for kind in CATALOG:
    PLANE.setdefault(kind, "flow")


def replay_kinds(ops):
    buckets = {name: [] for name in ("background", "behindText", "flow", "inFrontOfText")}
    for item in ops:
        buckets[PLANE[item["kind"]]].append(item["kind"])
    out = []
    for name in ("background", "behindText", "flow", "inFrontOfText"):
        out.extend(buckets[name])
    return out


def expected_trace(width, height, ops):
    buckets = {name: [] for name in ("background", "behindText", "flow", "inFrontOfText")}
    for item in ops:
        buckets[PLANE[item["kind"]]].append(item)
    lines = [f"begin_page {width:.2f}x{height:.2f}"]
    count = 0
    for name in ("background", "behindText", "flow", "inFrontOfText"):
        for item in buckets[name]:
            lines.append(
                "  {kind} bbox={x:.2f},{y:.2f},{w:.2f},{h:.2f}".format(
                    kind=item["kind"],
                    x=item["x"],
                    y=item["y"],
                    w=item["w"],
                    h=item["h"],
                )
            )
            count += 1
    lines.append(f"end_page ops={count}")
    return lines


def load_manifest():
    return json.loads((FIXTURE / "manifest.json").read_text(encoding="utf-8"))


def load_scenes():
    scenes = []
    for path in sorted((FIXTURE / "scenes").glob("*.json")):
        scenes.append((path, json.loads(path.read_text(encoding="utf-8"))))
    return scenes


def validate():
    errors = []
    manifest = load_manifest()
    scenes = load_scenes()
    if manifest["schema"] != 1:
        errors.append(f"manifest schema {manifest['schema']}")
    if manifest["sceneCount"] != len(scenes):
        errors.append(f"sceneCount {manifest['sceneCount']} != files {len(scenes)}")
    ids = [spec["id"] for _, spec in scenes]
    if ids != manifest["ids"]:
        errors.append("manifest ids 와 파일 id 순서가 다르다")
    if len(ids) != len(set(ids)):
        errors.append("중복 id")
    for path, spec in scenes:
        if spec.get("schema") != 1:
            errors.append(f"{path.name}: schema")
        if spec["id"] != path.stem:
            errors.append(f"{path.name}: 파일명 != id {spec['id']}")
        if spec["width"] <= 0 or spec["height"] <= 0:
            errors.append(f"{spec['id']}: 치수")
        if not spec.get("contract"):
            errors.append(f"{spec['id']}: 빈 contract")
        for item in spec["ops"]:
            if item["kind"] not in CATALOG:
                errors.append(f"{spec['id']}: 미등록 kind {item['kind']}")
            if item["kind"] in ("glyphRun", "glyphOutline"):
                errors.append(f"{spec['id']}: 빌더가 못 만드는 kind")
        kinds = replay_kinds(spec["ops"])
        if kinds != spec["expectedKinds"]:
            errors.append(f"{spec['id']}: expectedKinds {spec['expectedKinds']} != {kinds}")
        trace = expected_trace(spec["width"], spec["height"], spec["ops"])
        if spec.get("expectedTrace") != trace:
            errors.append(f"{spec['id']}: expectedTrace 불일치")
    return errors


def main():
    errors = validate()
    if errors:
        for err in errors:
            print(err)
        raise SystemExit(1)
    print("ok", load_manifest()["sceneCount"], "scenes")


if __name__ == "__main__":
    main()
