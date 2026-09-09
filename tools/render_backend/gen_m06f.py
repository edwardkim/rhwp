#!/usr/bin/env python3
"""M06-f render_backend 계약 픽스처·통합시험·문서 생성기.

devel `paint_op_kind` / replay plane / 광고 정직성 표를 파이썬으로 재현해
장면 JSON·기대 추적·통합 시험·카탈로그 문서를 만든다.
`src/renderer/**` 와 serializer 는 건드리지 않는다.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / "tests" / "fixtures" / "render_backend"
CASES = ROOT / "tests" / "cases"
DOCS_MANUAL = ROOT / "mydocs" / "manual"
DOCS_TECH = ROOT / "mydocs" / "tech"
DOCS_WORKING = ROOT / "mydocs" / "working" / "archives"

KINDS = [
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
]
MATERIALIZABLE = [k for k in KINDS if k not in ("glyphRun", "glyphOutline")]
PLANE = {
    "pageBackground": "background",
}
for k in KINDS:
    PLANE.setdefault(k, "flow")
FEATURE = {
    "textRun": "vectorText",
    "glyphRun": "vectorText",
    "glyphOutline": "vectorText",
    "charOverlap": "vectorText",
    "textDecoration": "vectorText",
    "footnoteMarker": "vectorText",
    "image": "images",
}
SUMMARY = {
    "pageBackground": "페이지 배경. 재생은 항상 첫 plane.",
    "textRun": "선택·검색 가능한 텍스트 런.",
    "glyphRun": "셰이핑된 글리프 런.",
    "glyphOutline": "글리프 외곽선. 일반 Path 가 아니다.",
    "charOverlap": "글자겹침 명시 visual op.",
    "textControlMark": "문단 끝·줄바꿈·필드 마커.",
    "tabLeader": "탭 리더 geometry.",
    "textDecoration": "밑줄·취소선·강조점.",
    "footnoteMarker": "각주·미주 위첨자 마커.",
    "line": "직선.",
    "rectangle": "사각형.",
    "ellipse": "타원.",
    "path": "임의 패스.",
    "image": "래스터 이미지.",
    "equation": "수식 SVG 조각.",
    "formObject": "양식 컨트롤.",
    "placeholder": "자리표시.",
    "rawSvg": "미리 렌더된 SVG 조각.",
}


def op(kind, x, y, w, h, text=None, gradient=False, image=False):
    return {
        "kind": kind,
        "x": float(x),
        "y": float(y),
        "w": float(w),
        "h": float(h),
        "text": text,
        "gradient": bool(gradient),
        "image": bool(image),
    }


def replay_kinds(ops):
    planes = {"background": [], "behindText": [], "flow": [], "inFrontOfText": []}
    for item in ops:
        planes[PLANE[item["kind"]]].append(item["kind"])
    out = []
    for name in ("background", "behindText", "flow", "inFrontOfText"):
        out.extend(planes[name])
    return out


def expected_trace(width, height, ops):
    kinds = replay_kinds(ops)
    # bbox follows original op order within a plane, matching tree order.
    by_plane = {"background": [], "behindText": [], "flow": [], "inFrontOfText": []}
    for item in ops:
        by_plane[PLANE[item["kind"]]].append(item)
    lines = [f"begin_page {width:.2f}x{height:.2f}"]
    count = 0
    for name in ("background", "behindText", "flow", "inFrontOfText"):
        for item in by_plane[name]:
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
    assert [item["kind"] for name in ("background", "behindText", "flow", "inFrontOfText") for item in by_plane[name]] == kinds
    return lines


def scene(sid, width, height, ops, contract):
    return {
        "schema": 1,
        "id": sid,
        "width": float(width),
        "height": float(height),
        "contract": contract,
        "ops": ops,
        "expectedKinds": replay_kinds(ops),
        "expectedTrace": expected_trace(width, height, ops),
    }


def build_scenes():
    scenes = []
    scenes.append(scene("s000-empty", 400, 300, [], "빈 페이지도 begin/end 경계를 남긴다"))
    scenes.append(
        scene(
            "s001-background",
            400,
            300,
            [op("pageBackground", 0, 0, 400, 300)],
            "배경만 있으면 Background plane 한 줄",
        )
    )
    scenes.append(
        scene(
            "s002-rect",
            400,
            300,
            [op("rectangle", 20, 20, 10, 10)],
            "사각형 하나",
        )
    )
    scenes.append(
        scene(
            "s003-line",
            400,
            300,
            [op("line", 0, 0, 50, 0)],
            "수평선 하나",
        )
    )
    scenes.append(
        scene(
            "s004-reorder",
            400,
            300,
            [
                op("rectangle", 20, 20, 10, 10),
                op("line", 0, 0, 50, 0),
                op("pageBackground", 0, 0, 400, 300),
            ],
            "트리 순서가 뒤바뀌어도 배경이 먼저 재생된다",
        )
    )
    scenes.append(
        scene(
            "s005-text",
            400,
            300,
            [op("textRun", 10, 20, 120, 16, text="M06-F-CAP")],
            "벡터 텍스트 정직성 문자열",
        )
    )
    scenes.append(
        scene(
            "s006-gradient-rect",
            400,
            300,
            [op("rectangle", 0, 0, 80, 40, gradient=True)],
            "그라디언트 사각형",
        )
    )
    scenes.append(
        scene(
            "s007-image",
            400,
            300,
            [op("image", 0, 0, 8, 8, image=True)],
            "1x1 PNG 이미지",
        )
    )

    for i, kind in enumerate(MATERIALIZABLE):
        extra = {}
        x, y, w, h = 12.0 + i, 24.0, 40.0, 18.0
        if kind == "pageBackground":
            x, y, w, h = 0, 0, 400, 300
        if kind == "textRun":
            extra["text"] = "M06-F-CAP"
        if kind == "image":
            extra["image"] = True
        scenes.append(
            scene(
                f"s1{i:02d}-{kind}",
                400,
                300,
                [op(kind, x, y, w, h, **extra)],
                f"{kind} 단독 장면",
            )
        )

    # size ladder — distinct page sizes keep the trace header unique
    sizes = [
        (1, 1),
        (10, 10),
        (40, 30),
        (96, 96),
        (200, 150),
        (400, 300),
        (595, 842),
        (800, 600),
        (1024, 768),
        (1280, 720),
    ]
    for i, (w, h) in enumerate(sizes):
        scenes.append(
            scene(
                f"s2{i:02d}-size-{w}x{h}",
                w,
                h,
                [op("rectangle", 0, 0, min(10, w), min(10, h))],
                f"페이지 치수 {w}x{h} 가 begin_page 에 그대로 찍힌다",
            )
        )

    # grid of rectangles — each cell is a distinct bbox
    grid_ops = []
    for row in range(4):
        for col in range(5):
            grid_ops.append(op("rectangle", 8 + col * 70, 8 + row * 60, 50, 40))
    scenes.append(scene("s300-rect-grid", 400, 300, grid_ops, "20칸 사각형 격자"))

    # mixed plane reorder: background last, text first
    scenes.append(
        scene(
            "s301-text-then-bg",
            400,
            300,
            [
                op("textRun", 16, 32, 200, 18, text="앞"),
                op("pageBackground", 0, 0, 400, 300),
            ],
            "텍스트가 트리 앞에 있어도 배경이 먼저",
        )
    )

    # all materializable kinds on one page, background last
    mixed = []
    for i, kind in enumerate(MATERIALIZABLE):
        extra = {}
        x, y, w, h = 10 + (i % 8) * 45, 20 + (i // 8) * 80, 36, 24
        if kind == "pageBackground":
            x, y, w, h = 0, 0, 400, 300
        if kind == "textRun":
            extra["text"] = f"k{i}"
        if kind == "image":
            extra["image"] = True
        mixed.append(op(kind, x, y, w, h, **extra))
    mixed.append(mixed.pop(0))  # move background to end if it was first
    # ensure background is last
    mixed = [item for item in mixed if item["kind"] != "pageBackground"] + [
        op("pageBackground", 0, 0, 400, 300)
    ]
    scenes.append(scene("s302-all-materializable", 400, 300, mixed, "만들 수 있는 kind 전부 + 배경 재정렬"))

    # empty pages of distinct sizes
    for i, (w, h) in enumerate(((50, 50), (100, 200), (300, 100), (777, 333))):
        scenes.append(
            scene(f"s4{i:02d}-empty-{w}x{h}", w, h, [], f"빈 {w}x{h} 페이지")
        )

    # stacked lines
    line_ops = [op("line", 0, 10 + i * 12, 380, 0) for i in range(12)]
    scenes.append(scene("s500-line-stack", 400, 300, line_ops, "12개 수평선"))

    # text ladder
    text_ops = [
        op("textRun", 10, 16 + i * 20, 180, 16, text=f"줄{i:02d}") for i in range(10)
    ]
    scenes.append(scene("s501-text-ladder", 400, 300, text_ops, "텍스트 10줄"))

    # decoration trio
    scenes.append(
        scene(
            "s502-decorations",
            400,
            300,
            [
                op("textRun", 10, 20, 100, 16, text="본문"),
                op("textDecoration", 10, 34, 100, 2, text="밑줄"),
                op("tabLeader", 120, 28, 80, 2),
                op("textControlMark", 210, 20, 12, 16, text="¶"),
            ],
            "장식·탭·제어 표식",
        )
    )

    # form + placeholder + rawSvg
    scenes.append(
        scene(
            "s503-chrome",
            400,
            300,
            [
                op("formObject", 20, 20, 80, 24),
                op("placeholder", 120, 20, 80, 60),
                op("rawSvg", 220, 20, 80, 60),
                op("equation", 20, 100, 60, 40),
                op("footnoteMarker", 100, 100, 16, 16),
            ],
            "양식·자리표시·수식·각주",
        )
    )

    # ellipse/path/image cluster
    scenes.append(
        scene(
            "s504-shapes",
            400,
            300,
            [
                op("ellipse", 20, 20, 60, 40),
                op("path", 100, 20, 60, 40),
                op("image", 180, 20, 32, 32, image=True),
                op("rectangle", 230, 20, 60, 40, gradient=True),
            ],
            "도형 가족",
        )
    )

    # off-origin bboxes (still on page)
    scenes.append(
        scene(
            "s505-offset",
            400,
            300,
            [
                op("rectangle", 350, 250, 40, 40),
                op("line", 0, 299, 400, 0),
                op("textRun", 2, 2, 40, 12, text="모서리"),
            ],
            "페이지 가장자리 bbox",
        )
    )

    # zero-height line (legal)
    scenes.append(
        scene(
            "s506-zero-height-line",
            400,
            300,
            [op("line", 10, 50, 200, 0)],
            "높이 0 선분은 유효하다",
        )
    )

    # many tiny rects
    tiny = [op("rectangle", (i % 20) * 18, (i // 20) * 18, 12, 12) for i in range(60)]
    scenes.append(scene("s507-tiny-60", 400, 300, tiny, "60개 작은 사각형"))

    # A4-ish with header/body/footer analog
    scenes.append(
        scene(
            "s508-a4-zones",
            595,
            842,
            [
                op("pageBackground", 0, 0, 595, 842),
                op("rectangle", 48, 36, 499, 28),
                op("textRun", 56, 42, 200, 16, text="머리글"),
                op("textRun", 56, 80, 400, 16, text="본문 첫 줄"),
                op("textRun", 56, 800, 120, 14, text="바닥글"),
            ],
            "A4 근사 머리/본문/바닥",
        )
    )

    # landscape
    scenes.append(
        scene(
            "s509-landscape",
            842,
            595,
            [
                op("pageBackground", 0, 0, 842, 595),
                op("rectangle", 20, 20, 800, 40),
                op("textRun", 28, 28, 240, 16, text="가로"),
            ],
            "가로 페이지",
        )
    )

    # char overlap pair
    scenes.append(
        scene(
            "s510-overlap-pair",
            400,
            300,
            [
                op("textRun", 40, 40, 80, 16, text="한"),
                op("charOverlap", 48, 40, 80, 16, text="겹"),
            ],
            "글자겹침 쌍",
        )
    )

    # capability probe scenes used by honesty tests
    scenes.append(
        scene(
            "s600-honesty-text",
            400,
            300,
            [op("textRun", 10, 20, 160, 16, text="M06-F-CAP")],
            "정직성 텍스트 프로브",
        )
    )
    scenes.append(
        scene(
            "s601-honesty-gradient",
            400,
            300,
            [op("rectangle", 0, 0, 80, 40, gradient=True)],
            "정직성 그라디언트 프로브",
        )
    )
    scenes.append(
        scene(
            "s602-honesty-image",
            400,
            300,
            [op("image", 0, 0, 8, 8, image=True)],
            "정직성 이미지 프로브",
        )
    )

    # position sweep for a single rectangle
    for i, x in enumerate((0, 1, 7, 13, 50, 99, 150, 200, 250, 300, 350, 389)):
        scenes.append(
            scene(
                f"s7{i:02d}-rect-x-{x}",
                400,
                300,
                [op("rectangle", x, 40, 10, 10)],
                f"사각형 x={x}",
            )
        )
    for i, y in enumerate((0, 1, 7, 13, 50, 99, 150, 200, 250, 289)):
        scenes.append(
            scene(
                f"s8{i:02d}-rect-y-{y}",
                400,
                300,
                [op("rectangle", 40, y, 10, 10)],
                f"사각형 y={y}",
            )
        )

    # kind pairs (first, second) for order stability inside flow
    pairs = [
        ("rectangle", "line"),
        ("line", "ellipse"),
        ("ellipse", "path"),
        ("path", "textRun"),
        ("textRun", "image"),
        ("image", "equation"),
        ("equation", "formObject"),
        ("formObject", "placeholder"),
        ("placeholder", "rawSvg"),
        ("rawSvg", "footnoteMarker"),
        ("footnoteMarker", "tabLeader"),
        ("tabLeader", "textDecoration"),
        ("textDecoration", "charOverlap"),
        ("charOverlap", "textControlMark"),
        ("textControlMark", "rectangle"),
    ]
    for i, (a, b) in enumerate(pairs):
        extra_a = {"text": "A"} if a in ("textRun", "charOverlap", "textControlMark", "textDecoration") else {}
        extra_b = {"text": "B"} if b in ("textRun", "charOverlap", "textControlMark", "textDecoration") else {}
        if a == "image":
            extra_a["image"] = True
        if b == "image":
            extra_b["image"] = True
        scenes.append(
            scene(
                f"s9{i:02d}-pair-{a}-{b}",
                400,
                300,
                [op(a, 20, 20, 30, 16, **extra_a), op(b, 80, 20, 30, 16, **extra_b)],
                f"flow 안 {a} 다음 {b} 순서 유지",
            )
        )

    # kind × page-size matrix — each cell has a distinct begin_page header
    matrix_sizes = ((80, 60), (160, 120), (240, 180), (320, 240), (480, 360), (640, 480))
    for kind in MATERIALIZABLE:
        for w, h in matrix_sizes:
            extra = {}
            x, y, bw, bh = 4.0, 6.0, min(24.0, w), min(12.0, h)
            if kind == "pageBackground":
                x, y, bw, bh = 0.0, 0.0, float(w), float(h)
            if kind == "textRun":
                extra["text"] = f"{kind}-{w}x{h}"
            if kind == "image":
                extra["image"] = True
            scenes.append(
                scene(
                    f"m-{kind}-{w}x{h}",
                    w,
                    h,
                    [op(kind, x, y, bw, bh, **extra)],
                    f"{kind} 를 {w}x{h} 페이지에 올리면 begin_page 헤더가 {w:.2f}x{h:.2f}",
                )
            )

    # three-op flow clusters with unique bbox
    clusters = [
        ("rectangle", "ellipse", "path"),
        ("line", "textRun", "textDecoration"),
        ("image", "placeholder", "rawSvg"),
        ("formObject", "equation", "footnoteMarker"),
        ("charOverlap", "tabLeader", "textControlMark"),
        ("rectangle", "textRun", "image"),
        ("ellipse", "path", "line"),
        ("placeholder", "formObject", "rawSvg"),
    ]
    for i, (a, b, c) in enumerate(clusters):
        def extras(kind, tag):
            extra = {}
            if kind in ("textRun", "charOverlap", "textControlMark", "textDecoration"):
                extra["text"] = tag
            if kind == "image":
                extra["image"] = True
            return extra

        scenes.append(
            scene(
                f"c{i:02d}-{a}-{b}-{c}",
                400,
                300,
                [
                    op(a, 10, 10, 40, 20, **extras(a, "A")),
                    op(b, 70, 10, 40, 20, **extras(b, "B")),
                    op(c, 130, 10, 40, 20, **extras(c, "C")),
                ],
                f"flow 클러스터 {a}+{b}+{c} 순서 유지",
            )
        )

    # unique ids, then sort so manifest matches glob 이름 순
    ids = [s["id"] for s in scenes]
    if len(ids) != len(set(ids)):
        raise SystemExit("중복 장면 id")
    scenes.sort(key=lambda spec: spec["id"])
    return scenes


def write_fixtures(scenes):
    scene_dir = FIXTURE / "scenes"
    scene_dir.mkdir(parents=True, exist_ok=True)
    for old in scene_dir.glob("*.json"):
        old.unlink()
    for spec in scenes:
        path = scene_dir / f"{spec['id']}.json"
        path.write_text(json.dumps(spec, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    manifest = {
        "schema": 1,
        "sceneCount": len(scenes),
        "ids": [s["id"] for s in scenes],
        "materializableKinds": MATERIALIZABLE,
        "catalogKinds": KINDS,
        "notes": "M06-f 합성 장면. 실제 HWP 가 아니다. TraceBackend 기대 로그를 담는다.",
    }
    (FIXTURE / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    readme = """---
kind: reference
status: active
canonical: mydocs/tech/render_backend_fixture_catalog.md
last_verified: 2026-08-18
---

# render_backend 장면 픽스처

`gen_m06f.py` 가 만든 합성 장면이다. 각 JSON 은 한 페이지의 leaf op 와
TraceBackend 기대 로그를 담는다. 실제 HWP 샘플이 아니다.

- `manifest.json` — 장면 수와 id 목록
- `scenes/*.json` — 장면 한 장
"""
    (FIXTURE / "README.md").write_text(readme, encoding="utf-8")


HEADER = """//! M06-f render_backend 계약 통합 시험.
//!
//! source-side `#[cfg(test)]` 는 늘리지 않는다. 이 파일은 `tests/cases/` 에만 둔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::paint::PageLayerTree;
use rhwp::render_backend::{
    all_families_share_trace, builtin_scenes, catalog_invariants_hold, compare_shots,
    expected_honesty_table, fixture_root, honesty_table_holds, load_manifest,
    load_scene_fixtures, materializable_kinds, observe_svg, page_size_cases_hold,
    paint_op_kind, replay_page, shot_from_tree, spec_for_kind, standard_lifecycle_scripts,
    svg_backend_reject_second, BackendFamily, BackendFeature, FixtureScene, HonestyRow,
    LifecycleExpect, LifecycleStep, NullBackend, PageSize, PairVerdict, PngBackend,
    RenderBackend, RenderBackendError, SceneOp, SceneSpec, SkiaBackend, SvgBackend,
    TraceBackend, ALL_FEATURES, HONESTY_TEXT, PAINT_OP_KIND_COUNT, PAINT_OP_KIND_SPECS,
    PAGE_SIZE_CASES, PNG_SIGNATURE,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn replay_trace(tree: &PageLayerTree) -> String {
    let mut backend = TraceBackend::new();
    replay_page(&mut backend, tree).expect("replay");
    backend.finish().expect("finish")
}

fn rect_scene() -> SceneSpec {
    SceneSpec::empty("tmp-rect", 400.0, 300.0)
        .push(SceneOp::new("rectangle", 20.0, 20.0, 10.0, 10.0))
}
"""


def rust_ident(sid: str) -> str:
    out = []
    for ch in sid:
        if ch.isalnum():
            out.append(ch.lower())
        else:
            out.append("_")
    ident = "".join(out)
    while "__" in ident:
        ident = ident.replace("__", "_")
    return ident.strip("_")


def write_case_catalog(scenes):
    lines = [
        "//! M06-f 카탈로그·장면 빌더 계약.",
        "#![cfg(not(target_arch = \"wasm32\"))]",
        "",
        "use rhwp::render_backend::{",
        "    builtin_scenes, catalog_invariants_hold, materializable_kinds, materialize_scene_op,",
        "    paint_op_kind, spec_for_kind, SceneOp, HONESTY_TEXT, PAINT_OP_KIND_COUNT,",
        "    PAINT_OP_KIND_SPECS,",
        "};",
        "",
        "#[test]",
        "fn catalog_invariants_and_count() {",
        "    catalog_invariants_hold().unwrap();",
        f"    assert_eq!(PAINT_OP_KIND_SPECS.len(), {len(KINDS)});",
        "    assert_eq!(PAINT_OP_KIND_COUNT, PAINT_OP_KIND_SPECS.len());",
        "    let names: Vec<_> = PAINT_OP_KIND_SPECS.iter().map(|s| s.kind).collect();",
        "    let mut sorted = names.clone();",
        "    sorted.sort();",
        "    sorted.dedup();",
        "    assert_eq!(sorted.len(), names.len());",
        "}",
        "",
        "#[test]",
        "fn every_catalog_kind_has_spec() {",
        "    for spec in PAINT_OP_KIND_SPECS {",
        "        let found = spec_for_kind(spec.kind).unwrap();",
        "        assert_eq!(found.kind, spec.kind);",
        "        assert!(found.appears_in_trace);",
        "        assert!(!found.summary_ko.is_empty());",
        "        assert_eq!(found.plane_name(), spec.default_plane.as_str());",
        "    }",
        "    assert!(spec_for_kind(\"not-a-kind\").is_none());",
        "}",
        "",
        "#[test]",
        "fn materializable_kinds_roundtrip_paint_op_kind() {",
        "    for kind in materializable_kinds() {",
        "        let op = if *kind == \"pageBackground\" {",
        "            SceneOp::new(*kind, 0.0, 0.0, 400.0, 300.0)",
        "        } else if *kind == \"textRun\" {",
        "            SceneOp::new(*kind, 10.0, 20.0, 80.0, 16.0).with_text(HONESTY_TEXT)",
        "        } else if *kind == \"image\" {",
        "            SceneOp::new(*kind, 0.0, 0.0, 8.0, 8.0).with_image()",
        "        } else {",
        "            SceneOp::new(*kind, 12.0, 24.0, 40.0, 18.0)",
        "        };",
        "        let paint = materialize_scene_op(&op);",
        "        assert_eq!(paint_op_kind(&paint), *kind, \"{kind}\");",
        "        let b = paint.bounds();",
        "        assert_eq!(b.x, op.bounds.x);",
        "        assert_eq!(b.y, op.bounds.y);",
        "        assert_eq!(b.width, op.bounds.width);",
        "        assert_eq!(b.height, op.bounds.height);",
        "    }",
        "}",
        "",
        "#[test]",
        "fn builtin_scenes_ids_are_unique_and_catalogued() {",
        "    let scenes = builtin_scenes();",
        "    let mut ids = std::collections::BTreeSet::new();",
        "    for scene in &scenes {",
        "        assert!(ids.insert(scene.id.clone()), \"중복 {}\", scene.id);",
        "        assert!(!scene.contract.is_empty());",
        "        assert!(scene.width > 0.0 && scene.height > 0.0);",
        "        for op in &scene.ops {",
        "            assert!(spec_for_kind(&op.kind).is_some(), \"{}\", op.kind);",
        "        }",
        "    }",
        "    assert!(scenes.len() >= 20);",
        "}",
        "",
    ]
    for spec in PAINT_OP_KIND_SPECS_ROWS():
        ident = rust_ident(f"catalog_row_{spec[0]}")
        lines += [
            "#[test]",
            f"fn {ident}() {{",
            f"    let spec = spec_for_kind(\"{spec[0]}\").unwrap();",
            f"    assert_eq!(spec.kind, \"{spec[0]}\");",
            f"    assert_eq!(spec.plane_name(), \"{spec[1]}\");",
            f"    assert_eq!(spec.feature_name(), \"{spec[2]}\");",
            f"    assert_eq!(spec.survives_flatten, {str(spec[3]).lower()});",
            f"    assert!(spec.appears_in_trace);",
            "}",
            "",
        ]
    (CASES / "render_backend_m06f_catalog.rs").write_text("\n".join(lines), encoding="utf-8")


def PAINT_OP_KIND_SPECS_ROWS():
    rows = []
    for kind in KINDS:
        feat = FEATURE.get(kind, "none")
        rows.append((kind, PLANE[kind], feat, True))
    return rows


def write_case_lifecycle():
    lines = [
        "//! M06-f 생명주기·치수·오류 Display 계약.",
        "#![cfg(not(target_arch = \"wasm32\"))]",
        "",
        "use rhwp::render_backend::{",
        "    error_display_holds, page_size_cases_hold, run_lifecycle, standard_lifecycle_scripts,",
        "    LifecycleExpect, LifecycleStep, NullBackend, PageSize, PngBackend, RenderBackend,",
        "    RenderBackendError, SceneOp, SkiaBackend, SvgBackend, TraceBackend, PAGE_SIZE_CASES,",
        "};",
        "",
        "fn draw_rect(backend: &mut impl RenderBackend<Error = RenderBackendError>) -> Result<(), RenderBackendError> {",
        "    backend.draw(&rhwp::render_backend::materialize_scene_op(&SceneOp::new(",
        "        \"rectangle\", 0.0, 0.0, 10.0, 10.0,",
        "    )))",
        "}",
        "",
        "#[test]",
        "fn page_size_table_matches_is_valid() {",
        "    page_size_cases_hold().unwrap();",
        "    assert!(PAGE_SIZE_CASES.len() >= 10);",
        "    let valid = PAGE_SIZE_CASES.iter().filter(|c| c.valid).count();",
        "    let invalid = PAGE_SIZE_CASES.iter().filter(|c| !c.valid).count();",
        "    assert!(valid >= 3 && invalid >= 5);",
        "}",
        "",
        "#[test]",
        "fn error_display_contains_tokens() {",
        "    let errors = [",
        "        RenderBackendError::NoOpenPage { call: \"draw\" },",
        "        RenderBackendError::NoOpenPage { call: \"end_page\" },",
        "        RenderBackendError::PageAlreadyOpen,",
        "        RenderBackendError::UnclosedPage { pages_completed: 0 },",
        "        RenderBackendError::InvalidPageSize { width: 0.0, height: 1.0 },",
        "        RenderBackendError::UnsupportedOp { backend: \"svg\", op: \"clip\" },",
        "        RenderBackendError::MultiplePagesUnsupported { backend: \"svg\" },",
        "        RenderBackendError::Backend(\"boom\".into()),",
        "    ];",
        "    for err in &errors {",
        "        rhwp::render_backend::error_display_holds(err).unwrap();",
        "    }",
        "}",
        "",
        "fn run_on_null(script_id: &str) {",
        "    let script = standard_lifecycle_scripts()",
        "        .iter()",
        "        .find(|s| s.id == script_id)",
        "        .unwrap();",
        "    let mut backend = NullBackend::new();",
        "    run_lifecycle(&mut backend, script, draw_rect).unwrap();",
        "    if matches!(script.rows.last().map(|r| &r.step), Some(LifecycleStep::Finish)) {",
        "        match script.rows.last().unwrap().expect {",
        "            LifecycleExpect::Ok => {",
        "                backend.finish().unwrap();",
        "            }",
        "            LifecycleExpect::Err(_) => {",
        "                assert!(backend.finish().is_err());",
        "            }",
        "        }",
        "    }",
        "}",
        "",
    ]
    scripts = [
        "draw-without-begin",
        "end-without-begin",
        "double-begin",
        "finish-while-open",
        "empty-page",
        "one-draw",
        "invalid-then-valid",
    ]
    for sid in scripts:
        ident = rust_ident(f"lifecycle_{sid}")
        lines += [
            "#[test]",
            f"fn {ident}() {{",
            f"    run_on_null(\"{sid}\");",
            "}",
            "",
        ]
    # same scripts on svg/trace/png/skia for the ones that don't finish-consume awkwardly
    for backend, ctor in (
        ("trace", "TraceBackend::new()"),
        ("svg", "SvgBackend::new()"),
        ("png", "PngBackend::new()"),
        ("skia", "SkiaBackend::new()"),
    ):
        lines += [
            "#[test]",
            f"fn {backend}_draw_without_begin_is_no_open_page() {{",
            f"    let mut backend = {ctor};",
            "    let err = draw_rect(&mut backend).unwrap_err();",
            "    assert_eq!(err, RenderBackendError::NoOpenPage { call: \"draw\" });",
            "}",
            "",
            "#[test]",
            f"fn {backend}_end_without_begin_is_no_open_page() {{",
            f"    let mut backend = {ctor};",
            "    let err = backend.end_page().unwrap_err();",
            "    assert_eq!(err, RenderBackendError::NoOpenPage { call: \"end_page\" });",
            "}",
            "",
            "#[test]",
            f"fn {backend}_invalid_size_rejected() {{",
            f"    let mut backend = {ctor};",
            "    let err = backend.begin_page(PageSize::new(0.0, 10.0)).unwrap_err();",
            "    assert_eq!(",
            "        err,",
            "        RenderBackendError::InvalidPageSize { width: 0.0, height: 10.0 }",
            "    );",
            "}",
            "",
        ]
    (CASES / "render_backend_m06f_lifecycle.rs").write_text("\n".join(lines), encoding="utf-8")


def write_case_fixtures(scenes):
    # one file that loads all fixtures + per-scene tests split into chunks to keep compile times ok
    chunks = [scenes[i:i + 40] for i in range(0, len(scenes), 40)]
    for idx, chunk in enumerate(chunks):
        lines = [
            f"//! M06-f 장면 픽스처 재생 {idx + 1}/{len(chunks)}.",
            "#![cfg(not(target_arch = \"wasm32\"))]",
            "",
            "use std::path::PathBuf;",
            "",
            "use rhwp::render_backend::{",
            "    load_scene_fixtures, parse_fixture_json, replay_page, FixtureScene, TraceBackend,",
            "    RenderBackend,",
            "};",
            "",
            "fn manifest_dir() -> PathBuf {",
            "    PathBuf::from(env!(\"CARGO_MANIFEST_DIR\"))",
            "}",
            "",
            "fn load_named(id: &str) -> FixtureScene {",
            "    let all = load_scene_fixtures(&manifest_dir()).expect(\"fixtures\");",
            "    all.into_iter()",
            "        .map(|(_, f)| f)",
            "        .find(|f| f.scene.id == id)",
            "        .unwrap_or_else(|| panic!(\"missing fixture {id}\"))",
            "}",
            "",
            "fn assert_fixture(id: &str) {",
            "    let fixture = load_named(id);",
            "    assert_eq!(fixture.schema, FixtureScene::SCHEMA);",
            "    let tree = fixture.scene.to_layer_tree();",
            "    let mut backend = TraceBackend::new();",
            "    replay_page(&mut backend, &tree).unwrap();",
            "    let trace = backend.finish().unwrap();",
            "    let kinds = fixture.scene.expected_replay_kinds();",
            "    let got_kinds: Vec<&str> = kinds.iter().copied().collect();",
            "    let expect_kinds: Vec<&str> = fixture.expected_kinds.iter().map(String::as_str).collect();",
            "    assert_eq!(got_kinds, expect_kinds, \"{id} kinds\");",
            "    if let Some(lines) = &fixture.expected_trace {",
            "        let got: Vec<&str> = trace.lines().collect();",
            "        assert_eq!(got, lines.iter().map(String::as_str).collect::<Vec<_>>(), \"{id} trace\");",
            "    }",
            "}",
            "",
        ]
        for spec in chunk:
            ident = rust_ident(f"fixture_{spec['id']}")
            lines += [
                "#[test]",
                f"fn {ident}() {{",
                f"    assert_fixture(\"{spec['id']}\");",
                "}",
                "",
            ]
        (CASES / f"render_backend_m06f_fixtures_{idx + 1:02d}.rs").write_text(
            "\n".join(lines), encoding="utf-8"
        )

    # loader / manifest test
    lines = [
        "//! M06-f 픽스처 매니페스트·파서 계약.",
        "#![cfg(not(target_arch = \"wasm32\"))]",
        "",
        "use std::path::PathBuf;",
        "",
        "use rhwp::render_backend::{",
        "    fixture_root, load_manifest, load_scene_fixtures, parse_fixture_json, FixtureScene,",
        "};",
        "",
        "fn manifest_dir() -> PathBuf {",
        "    PathBuf::from(env!(\"CARGO_MANIFEST_DIR\"))",
        "}",
        "",
        "#[test]",
        "fn manifest_ids_match_files() {",
        "    let manifest = load_manifest(&manifest_dir()).unwrap();",
        f"    assert!(manifest.scene_count >= {len(scenes)});",
        "    let files = load_scene_fixtures(&manifest_dir()).unwrap();",
        "    assert_eq!(files.len(), manifest.scene_count);",
        "    let file_ids: Vec<String> = files.iter().map(|(_, f)| f.scene.id.clone()).collect();",
        "    assert_eq!(file_ids, manifest.ids);",
        "    assert!(fixture_root(&manifest_dir()).join(\"scenes\").is_dir());",
        "}",
        "",
        "#[test]",
        "fn parse_roundtrip_first_scene() {",
        "    let files = load_scene_fixtures(&manifest_dir()).unwrap();",
        "    let (_, first) = &files[0];",
        "    let json = first.to_json_value();",
        "    let parsed = parse_fixture_json(&json).unwrap();",
        "    assert_eq!(parsed.scene.id, first.scene.id);",
        "    assert_eq!(parsed.scene.ops.len(), first.scene.ops.len());",
        "    assert_eq!(parsed.schema, FixtureScene::SCHEMA);",
        "}",
        "",
        "#[test]",
        "fn parse_rejects_bad_schema() {",
        "    let err = parse_fixture_json(",
        "        r#\"{\"schema\":99,\"id\":\"x\",\"width\":1.0,\"height\":1.0,\"contract\":\"c\",\"ops\":[],\"expectedKinds\":[],\"expectedTrace\":null}\"#,",
        "    )",
        "    .unwrap_err();",
        "    assert!(err.contains(\"schema\"), \"{err}\");",
        "}",
        "",
    ]
    (CASES / "render_backend_m06f_fixture_loader.rs").write_text("\n".join(lines), encoding="utf-8")


def write_case_honesty():
    lines = [
        "//! M06-f 광고 vs 실지원 정직성.",
        "#![cfg(not(target_arch = \"wasm32\"))]",
        "",
        "use rhwp::render_backend::{",
        "    expected_honesty_table, honesty_table_holds, observe_svg, replay_page, BackendFeature,",
        "    NullBackend, PageSize, PngBackend, RenderBackend, RenderBackendError,",
        "    SceneOp, SceneSpec, SkiaBackend, SvgBackend, TraceBackend, ALL_FEATURES, HONESTY_TEXT,",
        "    PNG_SIGNATURE,",
        "};",
        "",
        "#[test]",
        "fn honesty_table_matches_live_capabilities() {",
        "    honesty_table_holds().unwrap();",
        "    let table = expected_honesty_table();",
        "    assert_eq!(table.len(), 5);",
        "    let names: Vec<_> = table.iter().map(|r| r.name).collect();",
        "    assert_eq!(names, vec![\"svg\", \"null\", \"trace\", \"png\", \"skia\"]);",
        "}",
        "",
        "#[test]",
        "fn all_features_have_stable_names() {",
        "    let names: Vec<_> = ALL_FEATURES.iter().map(|f| f.as_str()).collect();",
        "    assert_eq!(",
        "        names,",
        "        vec![",
        "            \"vectorText\",",
        "            \"embeddedFonts\",",
        "            \"gradients\",",
        "            \"clipping\",",
        "            \"images\",",
        "            \"multiPage\",",
        "            \"deterministic\",",
        "        ]",
        "    );",
        "}",
        "",
    ]
    for name in ("svg", "null", "trace", "png", "skia"):
        lines += [
            "#[test]",
            f"fn honesty_row_{name}_is_consistent() {{",
            "    let row = expected_honesty_table()",
            "        .into_iter()",
            f"        .find(|r| r.name == \"{name}\")",
            "        .unwrap();",
            "    assert!(row.is_consistent());",
            "    assert!(!row.note.is_empty());",
            "}",
            "",
        ]
    lines += [
        "fn assert_multi_page<B: RenderBackend<Error = RenderBackendError>>(mut backend: B) {",
        "    let caps = backend.capabilities();",
        "    let name = caps.name;",
        "    backend.begin_page(PageSize::new(40.0, 30.0)).unwrap();",
        "    backend.end_page().unwrap();",
        "    let second = backend.begin_page(PageSize::new(40.0, 30.0));",
        "    if caps.supports(BackendFeature::MultiPage) {",
        "        second.unwrap();",
        "        backend.end_page().unwrap();",
        "        backend.finish().unwrap();",
        "    } else {",
        "        assert_eq!(",
        "            second.unwrap_err(),",
        "            RenderBackendError::MultiplePagesUnsupported { backend: name }",
        "        );",
        "    }",
        "}",
        "",
        "#[test]",
        "fn svg_multi_page_matches_advertisement() {",
        "    assert_multi_page(SvgBackend::new());",
        "}",
        "",
        "#[test]",
        "fn null_multi_page_matches_advertisement() {",
        "    assert_multi_page(NullBackend::new());",
        "}",
        "",
        "#[test]",
        "fn trace_multi_page_matches_advertisement() {",
        "    assert_multi_page(TraceBackend::new());",
        "}",
        "",
        "#[test]",
        "fn png_multi_page_matches_advertisement() {",
        "    assert_multi_page(PngBackend::new());",
        "}",
        "",
        "#[test]",
        "fn skia_multi_page_matches_advertisement() {",
        "    assert_multi_page(SkiaBackend::new());",
        "}",
        "",
        "#[test]",
        "fn svg_text_observation_matches_vector_text_flag() {",
        "    let scene = SceneSpec::empty(\"h-text\", 400.0, 300.0)",
        "        .push(SceneOp::new(\"textRun\", 10.0, 20.0, 160.0, 16.0).with_text(HONESTY_TEXT));",
        "    let mut backend = SvgBackend::new();",
        "    replay_page(&mut backend, &scene.to_layer_tree()).unwrap();",
        "    let svg = backend.finish().unwrap();",
        "    let obs = observe_svg(&svg, HONESTY_TEXT);",
        "    let caps = SvgBackend::new().capabilities();",
        "    assert_eq!(obs.vector_text, caps.supports(BackendFeature::VectorText));",
        "    assert_eq!(obs.embedded_fonts, caps.supports(BackendFeature::EmbeddedFonts));",
        "}",
        "",
        "#[test]",
        "fn svg_gradient_observation_matches_flag() {",
        "    let scene = SceneSpec::empty(\"h-grad\", 400.0, 300.0)",
        "        .push(SceneOp::new(\"rectangle\", 0.0, 0.0, 80.0, 40.0).with_gradient());",
        "    let mut backend = SvgBackend::new();",
        "    replay_page(&mut backend, &scene.to_layer_tree()).unwrap();",
        "    let svg = backend.finish().unwrap();",
        "    let obs = observe_svg(&svg, HONESTY_TEXT);",
        "    assert_eq!(",
        "        obs.gradients,",
        "        SvgBackend::new().capabilities().supports(BackendFeature::Gradients)",
        "    );",
        "}",
        "",
        "#[test]",
        "fn svg_image_observation_matches_flag() {",
        "    let scene = SceneSpec::empty(\"h-img\", 400.0, 300.0)",
        "        .push(SceneOp::new(\"image\", 0.0, 0.0, 8.0, 8.0).with_image());",
        "    let mut backend = SvgBackend::new();",
        "    replay_page(&mut backend, &scene.to_layer_tree()).unwrap();",
        "    let svg = backend.finish().unwrap();",
        "    let obs = observe_svg(&svg, HONESTY_TEXT);",
        "    assert_eq!(",
        "        obs.images,",
        "        SvgBackend::new().capabilities().supports(BackendFeature::Images)",
        "    );",
        "}",
        "",
        "#[test]",
        "fn png_signature_constant_is_real() {",
        "    assert_eq!(PNG_SIGNATURE, &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);",
        "    if PngBackend::raster_available() {",
        "        let scene = SceneSpec::empty(\"png\", 40.0, 30.0)",
        "            .push(SceneOp::new(\"rectangle\", 0.0, 0.0, 10.0, 10.0));",
        "        let mut backend = PngBackend::new();",
        "        replay_page(&mut backend, &scene.to_layer_tree()).unwrap();",
        "        let bytes = backend.finish().unwrap();",
        "        assert!(bytes.starts_with(PNG_SIGNATURE));",
        "    }",
        "}",
        "",
        "#[test]",
        "fn instrument_backends_have_no_visual_features() {",
        "    for row in expected_honesty_table() {",
        "        if row.name == \"null\" || row.name == \"trace\" {",
        "            assert!(!row.vector_text);",
        "            assert!(!row.gradients);",
        "            assert!(!row.images);",
        "            assert!(!row.clipping);",
        "            assert!(!row.embedded_fonts);",
        "            assert!(row.multi_page);",
        "            assert!(row.deterministic);",
        "        }",
        "    }",
        "}",
        "",
    ]
    (CASES / "render_backend_m06f_honesty.rs").write_text("\n".join(lines), encoding="utf-8")


def write_case_diff():
    lines = [
        "//! M06-f 전어댑터 상호 diff — 같은 입력 추적 공유.",
        "#![cfg(not(target_arch = \"wasm32\"))]",
        "",
        "use rhwp::render_backend::{",
        "    all_families_share_trace, compare_shots, kind_set, shot_from_tree, svg_is_deterministic,",
        "    BackendFamily, PairVerdict, SceneOp, SceneSpec,",
        "};",
        "",
        "fn sample() -> SceneSpec {",
        "    SceneSpec::empty(\"diff-sample\", 400.0, 300.0)",
        "        .push(SceneOp::new(\"rectangle\", 20.0, 20.0, 10.0, 10.0))",
        "        .push(SceneOp::new(\"line\", 0.0, 0.0, 50.0, 0.0))",
        "        .push(SceneOp::new(\"pageBackground\", 0.0, 0.0, 400.0, 300.0))",
        "}",
        "",
        "#[test]",
        "fn families_share_the_same_trace() {",
        "    let tree = sample().to_layer_tree();",
        "    let trace = all_families_share_trace(&tree).unwrap();",
        "    assert!(trace.starts_with(\"begin_page 400.00x300.00\"), \"{trace}\");",
        "    assert!(trace.contains(\"pageBackground\"));",
        "    assert!(trace.contains(\"rectangle\"));",
        "    assert!(trace.contains(\"line\"));",
        "}",
        "",
        "#[test]",
        "fn same_family_shots_match() {",
        "    let tree = sample().to_layer_tree();",
        "    let a = shot_from_tree(BackendFamily::Svg, &tree).unwrap();",
        "    let b = shot_from_tree(BackendFamily::Svg, &tree).unwrap();",
        "    assert_eq!(compare_shots(&a, &b), PairVerdict::Match);",
        "}",
        "",
        "#[test]",
        "fn different_output_family_is_skipped() {",
        "    let tree = sample().to_layer_tree();",
        "    let svg = shot_from_tree(BackendFamily::Svg, &tree).unwrap();",
        "    let png = shot_from_tree(BackendFamily::Png, &tree).unwrap();",
        "    assert_eq!(compare_shots(&svg, &png), PairVerdict::SkippedDifferentFamily);",
        "}",
        "",
        "#[test]",
        "fn svg_output_is_deterministic() {",
        "    svg_is_deterministic(&sample().to_layer_tree()).unwrap();",
        "}",
        "",
        "#[test]",
        "fn kind_set_counts_tree_not_replay_order() {",
        "    let tree = sample().to_layer_tree();",
        "    let set = kind_set(&tree);",
        "    assert_eq!(set.get(\"rectangle\").copied(), Some(1));",
        "    assert_eq!(set.get(\"line\").copied(), Some(1));",
        "    assert_eq!(set.get(\"pageBackground\").copied(), Some(1));",
        "}",
        "",
    ]
    for fam in ("Null", "Trace", "Svg", "Png", "Skia"):
        lines += [
            "#[test]",
            f"fn shot_{fam.lower()}_has_three_ops() {{",
            "    let tree = sample().to_layer_tree();",
            f"    let shot = shot_from_tree(BackendFamily::{fam}, &tree).unwrap();",
            "    assert_eq!(shot.op_count, 3);",
            f"    assert_eq!(shot.family, BackendFamily::{fam});",
            "    assert_eq!(shot.caps.name, shot.family.as_str());",
            "}",
            "",
        ]
    (CASES / "render_backend_m06f_diff.rs").write_text("\n".join(lines), encoding="utf-8")


def write_docs(scenes):
    kinds_table = "\n".join(
        f"| `{k}` | `{PLANE[k]}` | `{FEATURE.get(k, 'none')}` | {SUMMARY[k]} |"
        for k in KINDS
    )
    catalog = f"""---
kind: guide
status: active
canonical: mydocs/manual/render_backend_contract_catalog.md
last_verified: 2026-08-18
---

# RenderBackend 계약 카탈로그 (M06-f)

`src/render_backend/` 가 지키는 **출력 백엔드 최소 계약** 을 종류·능력·생명주기·
정직성·픽스처로 펼친 작성 가이드다. 어댑터 본체는 M06-1/2, 광고 정직성은 M06-3,
상호 diff 하네스는 M06-4, 네 번째 어댑터 작성 가이드는 M06-5 가 맡는다.
이 문서는 그 위에 **시험 가능한 표** 를 얹는다.

`src/renderer/**` 는 고치지 않는다. 직렬화기(`src/serializer/**`) 도 고치지 않는다.

## 1. 생명주기

호출 순서는 다음 정규식과 같아야 한다.

```
( begin_page  draw*  end_page )*  finish
```

어기면 백엔드는 오류를 내야 하고, 조용히 넘어가면 안 된다. 판정은
`PageState` 한 곳이 맡는다.

| 위반 | 오류 |
| --- | --- |
| `begin_page` 없이 `draw` | `NoOpenPage {{ call: "draw" }}` |
| `begin_page` 없이 `end_page` | `NoOpenPage {{ call: "end_page" }}` |
| 열린 페이지에 `begin_page` | `PageAlreadyOpen` |
| 열린 페이지에 `finish` | `UnclosedPage` |
| 폭/높이가 양수 유한값이 아님 | `InvalidPageSize` |
| `multi_page: false` 인데 두 번째 페이지 | `MultiplePagesUnsupported` |

`finish(self)` 는 산출물 소유권을 넘긴다. trait object 는 `finish_boxed` 를 쓴다.

## 2. 좌표·단위

- 단위는 **px**. HWPUNIT 환산은 이 계층 앞에서 끝난다.
- 원점은 페이지 왼쪽 위, y 는 아래로 증가.
- 좌표는 페이지 절대 좌표. `PaintOp` 는 평탄화된 leaf 다.
- 형식 고유 단위(pt, device px) 환산은 백엔드 안에서만 한다.

## 3. PaintOp 종류 표

문자열은 LayerTree JSON `"type"` 과 글자 그대로 같다.

| kind | 기본 plane | 필요 capability | 설명 |
| --- | --- | --- | --- |
{kinds_table}

`glyphRun` / `glyphOutline` 는 셰이핑 입력이 필요해 합성 장면 빌더가 만들지 않는다.
카탈로그 행과 `paint_op_kind` match 는 존재한다.

## 4. 능력 정직성

`BackendCapabilities` 필드는 **최종 산출물이 그 성질을 보존하는가** 이다.

| 백엔드 | raster | vectorText | fonts | gradients | clip | images | multiPage | deterministic |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| svg | no | yes | no | yes | no | yes | no | yes |
| null | no | no | no | no | no | no | yes | yes |
| trace | no | no | no | no | no | no | yes | yes |
| png | yes | no | no | live | no | live | no | no |
| skia | yes | no | no | live | no | live | no | no |

`live` 는 `native-skia` 네이티브 빌드에서만 켜진다. 꺼져 있으면 `finish` 는 빈 산출물이다.

래스터 전용인데 `vector_text: true` 이면 `is_consistent()` 가 거짓이다.

## 5. 재생 순서

`replay_page` 는 `PaintReplayPlane::ORDERED` (배경 → 글 뒤 → 본문 → 글 앞) 를
바깥 루프로 돈다. 트리에 배경을 마지막에 넣어도 추적 로그의 첫 op 는
`pageBackground` 다. 픽스처 `s004-reorder` 가 이 불변식을 닫는다.

## 6. 픽스처

합성 장면은 `tests/fixtures/render_backend/scenes/*.json` 이다. 생성기는
`tools/render_backend/gen_m06f.py`. 각 장면은 id·치수·op·기대 kind 순서·
기대 TraceBackend 로그를 담는다. 장면 수는 {len(scenes)} 이다.

통합 시험은 `tests/cases/render_backend_m06f_*.rs` 다. source-side
`#[cfg(test)]` 모듈은 늘리지 않는다.

## 7. 새 어댑터 체크리스트

1. `src/renderer/**` 를 고치지 않고 기존 공개 API 만 호출한다.
2. `PageState` 로 생명주기를 판정한다.
3. `BackendCapabilities` 광고 = 실지원. 얇게 평탄화하면 `clipping` 을 켜지 않는다.
4. 선택 피처가 꺼져도 타입은 컴파일되고 생명주기는 지키며, 광고가 빈 산출물을 숨기지 않는다.
5. 정직성 대조는 `honesty_table_holds` / M06-3 단위 시험에 접는다.
6. 상호 비교는 같은 `OutputFamily` 끼리만 바이트를 맞댄다.
7. 카탈로그에 없는 kind 를 새로 만들면 `PAINT_OP_KIND_SPECS` 와 `paint_op_kind` 를 같이 고친다.

## 8. 하지 않는 것

- gym / `scripts/visual_sweep.py` 수정
- serializer 수정
- `src/renderer/canvaskit_policy.rs` · `src/renderer/pdf.rs` 수정
- source-side `#[test]` 증가
"""
    (DOCS_MANUAL / "render_backend_contract_catalog.md").write_text(catalog, encoding="utf-8")

    fixture_rows = "\n".join(
        f"| `{s['id']}` | {s['width']:.0f}×{s['height']:.0f} | {len(s['ops'])} | {s['contract']} |"
        for s in scenes
    )
    tech = f"""---
kind: reference
status: active
canonical: mydocs/tech/render_backend_fixture_catalog.md
last_verified: 2026-08-18
---

# render_backend 픽스처 카탈로그 (M06-f)

합성 장면 {len(scenes)} 장의 목록이다. 각 장은 `tests/fixtures/render_backend/scenes/<id>.json`
이고, TraceBackend 기대 로그를 포함한다. 실제 HWP 가 아니다.

생성기: `tools/render_backend/gen_m06f.py`.
스키마: `FixtureScene::SCHEMA == 1`.

## 장면 목록

| id | 치수(px) | op 수 | 계약 |
| --- | --- | --- | --- |
{fixture_rows}

## JSON 필드

| 필드 | 의미 |
| --- | --- |
| `schema` | 지금 `1` |
| `id` | 안정 식별자 |
| `width` / `height` | 페이지 치수 px |
| `contract` | 이 장이 닫는 불변식 한 줄 |
| `ops[].kind` | 카탈로그 kind |
| `ops[].x,y,w,h` | bbox px |
| `ops[].text` | textRun 계열 문자열 |
| `ops[].gradient` | 그라디언트 채우기 |
| `ops[].image` | TINY_PNG 적재 |
| `expectedKinds` | plane 재정렬 후 kind 순서 |
| `expectedTrace` | TraceBackend `finish` 줄 |

## 기대 추적 형식

```
begin_page 400.00x300.00
  pageBackground bbox=0.00,0.00,400.00,300.00
  rectangle bbox=20.00,20.00,10.00,10.00
end_page ops=2
```

좌표는 항상 소수 2자리다. `f64` 기본 출력의 자릿수 흔들림을 없앤다.

## 상호 diff

같은 장면을 다섯 가족(null/trace/svg/png/skia)으로 재생해도 **추적 로그는 같다**.
다른 `OutputFamily` 끼리 PNG 바이트와 SVG 문자열을 맞대지 않는다.
없는 래스터는 skip 이 아니라, 타입은 있고 `finish` 가 빈 산출물이다.

## 관련

- 계약 표: [RenderBackend 계약 카탈로그](../manual/render_backend_contract_catalog.md)
- 설계 배경: [출력 백엔드 공통 계약](render_backend.md)
"""
    (DOCS_TECH / "render_backend_fixture_catalog.md").write_text(tech, encoding="utf-8")

    working = f"""---
kind: snapshot
status: active
canonical: mydocs/working/m06f_render_backend_fatten.md
last_verified: 2026-08-18
---

# M06-f render_backend 계약·픽스처 고도화

이슈 #5462. `src/renderer/**` 미수정. serializer 미수정. gym 미수정.

## 무엇을

devel 의 `RenderBackend`(Svg/Png/Skia + Null/Trace) 위에 계약 카탈로그·장면
빌더·정직성 표·픽스처 로더·상호 diff 요약을 얹고, 통합 시험 {len(scenes)} 장면을
닫았다.

## 왜

M06-1~3 이 어댑터와 광고 정직성을 넣었지만, kind 전수·치수 사다리·plane
재정렬·형식 가족 skip 을 한 표로 재현하는 픽스처가 없었다. source-side
`#[test]` 는 총량 동결이라 `tests/cases/` 와 JSON 픽스처로 고도화한다.

## 어떻게

- `src/render_backend/catalog.rs` — 18 kind 표
- `scenes.rs` — 합성 장면 빌더
- `contract.rs` — 생명주기 스크립트
- `honesty.rs` — 광고 vs 실지원
- `fixture.rs` — JSON 스키마·최소 파서
- `diff.rs` — 가족 비교
- `tests/fixtures/render_backend/scenes/` — {len(scenes)} JSON
- `tests/cases/render_backend_m06f_*.rs` — 통합 시험

## 검증

- `cargo fmt --all -- --check`
- `node scripts/rust-test-suite-manifest.mjs --check`
- `node scripts/rust-unit-test-tiers.mjs --check`
- `cargo test --lib render_backend::`
- `cargo test --test render_backend_m06f_catalog` 등 케이스
"""
    DOCS_WORKING.mkdir(parents=True, exist_ok=True)
    (DOCS_WORKING / "m06f_render_backend_fatten.md").write_text(working, encoding="utf-8")


def main():
    scenes = build_scenes()
    write_fixtures(scenes)
    write_case_catalog(scenes)
    write_case_lifecycle()
    write_case_fixtures(scenes)
    write_case_honesty()
    write_case_diff()
    write_docs(scenes)
    print(f"scenes={len(scenes)}")
    print(f"fixture_dir={FIXTURE}")


if __name__ == "__main__":
    main()
