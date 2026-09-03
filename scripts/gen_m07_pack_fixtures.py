#!/usr/bin/env python3
"""Generate M07-pack CanvasKit fallback matrices, envelopes, and working docs.

Each row is a unique style combination with an expected policy verdict.
Unknown decoration/tab codes stay Direct (local no-op). Invalid geometry and
the 4096 visual-item bound stay fail-closed. visualItemLimitExceeded is not
relaxed: runtime MAX_TEXT_SPECIAL_VISUAL_ITEMS and Rust
MAX_POSITIONED_CONTROL_MARKS_PER_RUN are both 4096.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DIR = ROOT / "tests" / "fixtures" / "m07_pack"
WORKING = ROOT / "mydocs" / "working" / "m07_pack_canvaskit_fallbacks.md"
PUBLIC = ROOT / "mydocs" / "tech" / "canvaskit-m07-pack-fallback-matrix.md"

ARROW_STYLES = [
    "none",
    "arrow",
    "concaveArrow",
    "openDiamond",
    "openCircle",
    "openSquare",
    "diamond",
    "circle",
    "square",
]
LINE_TYPES = [
    "single",
    "double",
    "thinThickDouble",
    "thickThinDouble",
    "thinThickThinTriple",
]
DASHES = ["solid", "dash", "dot", "dashDot", "dashDotDot"]
PATTERN_TYPES = list(range(1, 7))
SHADOW_TYPES = list(range(1, 9))
DECORATION_KINDS = ["underline", "strikethrough", "emphasisDot"]
DECORATION_SHAPES = list(range(0, 16))
EMPHASIS_DOTS = list(range(0, 8))
TAB_FILLS = list(range(0, 16))
ARROW_SIZES = list(range(0, 9))
OP_TYPES = ["line", "rectangle", "ellipse", "path"]

PATTERN_MEANING = {
    1: "horizontal-hatch",
    2: "vertical-hatch",
    3: "backslash-hatch",
    4: "slash-hatch",
    5: "cross-hatch",
    6: "lattice-hatch",
}
ARROW_MEANING = {
    "none": "no-head",
    "arrow": "filled-triangle",
    "concaveArrow": "filled-concave-triangle",
    "openDiamond": "stroked-diamond",
    "openCircle": "stroked-ellipse",
    "openSquare": "stroked-rect",
    "diamond": "filled-diamond",
    "circle": "filled-ellipse",
    "square": "filled-rect",
}
LINE_TYPE_MEANING = {
    "single": "one-stroke",
    "double": "equal-double",
    "thinThickDouble": "thin-then-thick",
    "thickThinDouble": "thick-then-thin",
    "thinThickThinTriple": "thin-thick-thin",
}


def row(
    case_id: str,
    family: str,
    op_type: str,
    status: str,
    detail: str | None,
    **fields: object,
) -> dict[str, object]:
    payload = {
        "caseId": case_id,
        "family": family,
        "opType": op_type,
        "status": status,
        "detail": detail,
        "pinsDocumentToCanvas2d": status != "direct",
        "schemaVersion": 1,
    }
    payload.update(fields)
    return payload


def build_matrix() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []

    for start in ARROW_STYLES:
        for end in ARROW_STYLES:
            for size in ARROW_SIZES:
                for op in ("line", "path"):
                    rows.append(
                        row(
                            f"lineArrow/{op}/{start}/{end}/size{size}",
                            "lineArrow",
                            op,
                            "direct",
                            None,
                            startArrow=start,
                            endArrow=end,
                            arrowSize=size,
                            arrowMeaningStart=ARROW_MEANING[start],
                            arrowMeaningEnd=ARROW_MEANING[end],
                            notes=(
                                "CanvasKit draws each serialized ArrowStyle with the "
                                f"HWP size nibble {size} (width={size // 3}, length={size % 3})."
                            ),
                        )
                    )

    for line_type in LINE_TYPES:
        for dash in DASHES:
            for op in ("line", "path"):
                rows.append(
                    row(
                        f"compoundLine/{op}/{line_type}/{dash}",
                        "compoundLine",
                        op,
                        "direct",
                        None,
                        lineType=line_type,
                        dash=dash,
                        lineTypeMeaning=LINE_TYPE_MEANING[line_type],
                        notes=(
                            "Offset ratios match SVG/Canvas2D draw_multi_line: "
                            "double=(0.30,-0.35)/(0.30,0.35)."
                        ),
                    )
                )

    for shadow_type in SHADOW_TYPES:
        for offset in (-4.0, -1.5, 0.0, 1.5, 4.0, 8.0):
            for alpha in (0, 40, 80, 128, 200, 255):
                for family, op in (
                    ("shapeShadow", "rectangle"),
                    ("shapeShadow", "ellipse"),
                    ("shapeShadow", "path"),
                    ("lineShadow", "line"),
                    ("lineShadow", "path"),
                ):
                    rows.append(
                        row(
                            f"{family}/{op}/type{shadow_type}/ox{offset:g}/a{alpha}",
                            family,
                            op,
                            "direct",
                            None,
                            shadowType=shadow_type,
                            offsetX=offset,
                            offsetY=offset * 0.5,
                            alpha=alpha,
                            opacity=round(1 - alpha / 255, 6),
                            notes=(
                                "HWP shadow alpha 0 is opaque. CanvasKit opacity is "
                                "1 - alpha/255."
                            ),
                        )
                    )

    colors = [
        ("#000000", "#ffffff"),
        ("#112233", "#eeddcc"),
        ("#cc0000", "#fff8f0"),
        ("#0066aa", "#f0f7ff"),
        ("#228822", "#f4fff4"),
        ("#663399", "#f8f0ff"),
        ("#886600", "#fff8e0"),
        ("#333333", "#eeeeee"),
    ]
    for pattern_type in PATTERN_TYPES:
        for fg, bg in colors:
            for op in ("rectangle", "ellipse", "path"):
                rows.append(
                    row(
                        f"patternFill/{op}/type{pattern_type}/{fg[1:]}-{bg[1:]}",
                        "patternFill",
                        op,
                        "direct",
                        None,
                        patternType=pattern_type,
                        patternMeaning=PATTERN_MEANING[pattern_type],
                        patternColor=fg,
                        backgroundColor=bg,
                        notes="Hatch tile is 6px, matching the SVG pattern def.",
                    )
                )

    for kind in DECORATION_KINDS:
        for shape in DECORATION_SHAPES:
            for emphasis in EMPHASIS_DOTS:
                rows.append(
                    row(
                        f"textDecoration/{kind}/shape{shape}/dot{emphasis}",
                        "unsupportedTextDecoration",
                        kind,
                        "direct",
                        None,
                        shape=shape,
                        emphasisDot=emphasis,
                        treatedAsSolid=shape > 12,
                        treatedAsNoDot=emphasis > 6,
                        notes=(
                            "Shapes 0-12 and emphasis 0-6 have explicit CanvasKit "
                            "geometry. Higher codes stay Direct and draw the solid/"
                            "no-dot conservative branch instead of pinning Canvas2D."
                        ),
                    )
                )

    geometries = [
        ("ok", 1.0, 12.0, True),
        ("ok-wide", 0.0, 80.0, True),
        ("ok-touch", 4.0, 4.5, True),
        ("inverted", 20.0, 4.0, False),
        ("nan-start", float("nan"), 12.0, False),
        ("nan-end", 1.0, float("nan"), False),
        ("equal", 8.0, 8.0, False),
        ("neg-span", 15.0, 14.9, False),
    ]
    for fill in TAB_FILLS:
        for geo_name, start, end, valid in geometries:
            status = "direct" if valid else "directRequired"
            detail = None if valid else "invalidTabLeader"
            if geo_name.startswith("nan"):
                detail = "invalidGeometry" if not valid else None
                # NaN geometry is caught by text_visual_geometry only when
                # run metrics are NaN; tab-leader start/end NaN is invalidTabLeader.
                if not valid:
                    status = "directRequired"
                    detail = "invalidTabLeader"
            rows.append(
                row(
                    f"tabLeader/fill{fill}/{geo_name}",
                    "invalidTabLeader",
                    "tabLeader",
                    status,
                    detail,
                    fillType=fill,
                    startX=None if start != start else start,
                    endX=None if end != end else end,
                    geometry=geo_name,
                    unknownFillSkipped=fill > 11,
                    notes=(
                        "fillType 0-11 keep their CanvasKit stroke recipes. "
                        "fillType>11 is a local skip. Inverted or non-finite "
                        "ranges stay fail-closed."
                    ),
                )
            )

    fonts = [
        "Batang",
        "Dotum",
        "Gulim",
        "Malgun Gothic",
        "NanumMyeongjo",
        "Noto Serif KR",
        "Pretendard",
        "Source Han Serif K",
    ]
    for number in range(1, 41):
        for font in fonts:
            rows.append(
                row(
                    f"footnoteMarker/n{number}/{font.replace(' ', '_')}",
                    "footnoteMarker",
                    "footnoteMarker",
                    "direct",
                    "footnoteMarker",
                    number=number,
                    text=f"{number})",
                    fontFamily=font,
                    fontSize=7 if number < 10 else 8,
                    notes="Runtime replays footnoteMarker as a textRun with baseline fontSize??7.",
                )
            )

    for count, expected in (
        (1, "direct"),
        (16, "direct"),
        (256, "direct"),
        (4096, "direct"),
        (4097, "directRequired"),
        (8192, "directRequired"),
    ):
        for op in ("textControlMark", "tabLeader", "textDecoration", "charOverlap"):
            rows.append(
                row(
                    f"visualItemLimit/{op}/n{count}",
                    "visualItemLimitExceeded",
                    op,
                    expected,
                    "visualItemLimitExceeded" if expected != "direct" else None,
                    itemCount=count,
                    sharedBound=4096,
                    relaxed=False,
                    notes=(
                        "Runtime MAX_TEXT_SPECIAL_VISUAL_ITEMS and Rust "
                        "MAX_POSITIONED_CONTROL_MARKS_PER_RUN are both 4096. "
                        "Raising one side would desync preflight from replay."
                    ),
                )
            )

    # Cross-family lines: every compound type with every arrow and a shadow.
    for line_type in LINE_TYPES:
        for arrow in ARROW_STYLES:
            for shadow_type in SHADOW_TYPES:
                rows.append(
                    row(
                        f"cross/line/{line_type}/{arrow}/shadow{shadow_type}",
                        "crossVector",
                        "line",
                        "direct",
                        None,
                        lineType=line_type,
                        endArrow=arrow,
                        shadowType=shadow_type,
                        notes="Compound + arrow + shadow must stay one Direct line op.",
                    )
                )

    return rows


def build_envelopes(rows: list[dict[str, object]]) -> list[dict[str, object]]:
    envelopes: list[dict[str, object]] = []
    for item in rows:
        status = item["status"]
        summary = {
            "totalItems": 1,
            "directItems": 1 if status == "direct" else 0,
            "directRequiredItems": 1 if status == "directRequired" else 0,
            "compatOverlayItems": 0,
            "textFallbackItems": 0,
            "unsupportedItems": 0,
            "hiddenOverlayViolations": 1 if status == "directRequired" else 0,
        }
        envelopes.append(
            {
                "schemaVersion": 1,
                "issue": 5448,
                "claim": "M07-pack",
                "caseId": item["caseId"],
                "family": item["family"],
                "mode": "default",
                "hiddenCanvas2dOverlayAllowed": False,
                "directReplayRequired": True,
                "summary": summary,
                "items": [
                    {
                        "path": "root/leaf/0",
                        "opType": item["opType"],
                        "feature": (
                            "textSpecialVisual"
                            if item["family"]
                            in {
                                "unsupportedTextDecoration",
                                "invalidTabLeader",
                                "footnoteMarker",
                                "visualItemLimitExceeded",
                            }
                            else "vectorShape"
                        ),
                        "status": status,
                        "reason": (
                            "directReplaySupported"
                            if status == "direct"
                            else "hiddenOverlayForbidden"
                        ),
                        "compatOverlayAllowed": False,
                        "detail": item["detail"],
                    }
                ],
                "input": {
                    key: value
                    for key, value in item.items()
                    if key
                    not in {
                        "schemaVersion",
                        "pinsDocumentToCanvas2d",
                    }
                },
            }
        )
    return envelopes


def write_jsonl(path: Path, rows: list[dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        for item in rows:
            handle.write(json.dumps(item, ensure_ascii=False, sort_keys=True))
            handle.write("\n")


def family_counts(rows: list[dict[str, object]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for item in rows:
        family = str(item["family"])
        counts[family] = counts.get(family, 0) + 1
    return counts


def write_working_doc(rows: list[dict[str, object]]) -> None:
    counts = family_counts(rows)
    direct = sum(1 for item in rows if item["status"] == "direct")
    blocked = len(rows) - direct
    WORKING.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# M07-pack CanvasKit 잔여 폴백 소거",
        "",
        "이슈: `#5448`. 소유 파일: `src/renderer/canvaskit_policy.rs`.",
        "devel 기준 M07-1..5(boxed-PUA, 조판부호, 세로, 회전, gradientFill)는",
        "이 작업에서 다시 열지 않는다. 이 문서는 **남은 폴백 가족**만 다룬다.",
        "",
        "## 판정",
        "",
        "| 가족 | 정책 | 런타임 | 문서 고정 |",
        "|---|---|---|---|",
        "| lineArrow | Direct | `drawArrowHead` 9종 | 아니오 |",
        "| compoundLine | Direct | SVG와 동일 오프셋 비율 | 아니오 |",
        "| shapeShadow / lineShadow | Direct | offset + `1-alpha/255` | 아니오 |",
        "| patternFill | Direct | 배경 + 6px 해치 1..6 | 아니오 |",
        "| unsupportedTextDecoration | Direct (형태 코드) | 0-12 명시, 그 외 실선 | 아니오 |",
        "| invalidTabLeader | 기하만 fail-closed | 0-11 명시, 그 외 skip | 역전/비유한만 예 |",
        "| footnoteMarker | Direct (`detail=footnoteMarker`) | `renderTextRun` baseline `fontSize??7` | 아니오 |",
        "| visualItemLimitExceeded | 4096 유지 | 동일 상수 | 예 (한도 초과만) |",
        "",
        "## visualItemLimitExceeded 를 완화하지 않은 이유",
        "",
        "- Studio `MAX_TEXT_SPECIAL_VISUAL_ITEMS = 4096`.",
        "- Rust `MAX_POSITIONED_CONTROL_MARKS_PER_RUN = 4096`.",
        "- paint JSON 도 같은 한도로 `positionsComplete`/`leadersComplete` 를 자른다.",
        "- 한쪽만 올리면 preflight 가 Direct 인데 런타임이 `visualItemLimitExceeded` 를",
        "  찍거나, 반대로 런타임은 그리는데 문서 전체가 Canvas2D 로 고정된다.",
        "- 한도를 올리려면 브라우저 작업량 측정이 필요하다. 이 PR 은 28→14 기본",
        "  렌더러 전환을 측정하지 않았고 이슈도 열지 않는다.",
        "",
        "## 실측 픽스처 규모",
        "",
        f"- 행렬 행: **{len(rows)}**",
        f"- Direct: **{direct}**",
        f"- fail-closed: **{blocked}**",
        "",
        "| 가족 | 행 |",
        "|---|---:|",
    ]
    for family, count in sorted(counts.items()):
        lines.append(f"| `{family}` | {count} |")
    lines.extend(
        [
            "",
            "## 화살표 크기 계약",
            "",
            "HWP `arrow_size` 0..8 = `{작은,중간,큰} × {작은,중간,큰}` (너비 × 길이).",
            "",
            "| size | width level | length level | width× | length× |",
            "|---:|---:|---:|---:|---:|",
        ]
    )
    for size in ARROW_SIZES:
        lines.append(
            f"| {size} | {size // 3} | {size % 3} | "
            f"{1.5 if size // 3 == 0 else 2.5 if size // 3 == 1 else 3.5} | "
            f"{1.0 if size % 3 == 0 else 1.5 if size % 3 == 1 else 2.0} |"
        )
    lines.extend(
        [
            "",
            "## 복합선 오프셋 계약",
            "",
            "| lineType | (width_ratio, offset_ratio) |",
            "|---|---|",
            "| single | `(1, 0)` |",
            "| double | `(0.30,-0.35)`, `(0.30,0.35)` |",
            "| thickThinDouble | `(0.4,-0.30)`, `(0.2,0.40)` |",
            "| thinThickDouble | `(0.2,-0.40)`, `(0.4,0.30)` |",
            "| thinThickThinTriple | `(0.15,-0.425)`, `(0.30,0)`, `(0.15,0.425)` |",
            "",
            "## 패턴 타일 계약",
            "",
            "| patternType | 의미 | 스트로크 |",
            "|---:|---|---|",
        ]
    )
    for pattern_type, meaning in PATTERN_MEANING.items():
        lines.append(f"| {pattern_type} | {meaning} | 6px tile |")
    lines.extend(
        [
            "",
            "## 탭 리더 fillType",
            "",
            "| fillType | CanvasKit |",
            "|---:|---|",
            "| 0 | skip |",
            "| 1 | solid 0.5 |",
            "| 2 | dash [3,3] |",
            "| 3 | round cap dotted |",
            "| 4 | dash-dot |",
            "| 5 | dash-dot-dot |",
            "| 6 | long dash |",
            "| 7 | dense dotted |",
            "| 8 | double hairline |",
            "| 9 | thick-bottom double |",
            "| 10 | thick-top double |",
            "| 11 | triple |",
            "| 12+ | skip, Direct |",
            "",
            "## 장식 shape",
            "",
            "| shape | 의미 |",
            "|---:|---|",
            "| 0 | solid |",
            "| 1 | dash [3,3] |",
            "| 2 | dot [1,2] |",
            "| 3 | dash-dot |",
            "| 4 | dash-dot-dot |",
            "| 5 | long dash |",
            "| 6 | dense dotted |",
            "| 7 | double |",
            "| 8 | thick-bottom double |",
            "| 9 | thick-top double |",
            "| 10 | triple |",
            "| 11 | wave |",
            "| 12 | double wave |",
            "| 13+ | solid, Direct |",
            "",
            "## 강조점",
            "",
            "| emphasisDot | 의미 |",
            "|---:|---|",
            "| 0 | none |",
            "| 1 | filled circle |",
            "| 2 | stroked circle |",
            "| 3 | caret |",
            "| 4 | zigzag |",
            "| 5 | small filled |",
            "| 6 | double stacked |",
            "| 7+ | skip, Direct |",
            "",
            "## 행 표본 (가족별 앞 12개)",
            "",
        ]
    )
    by_family: dict[str, list[dict[str, object]]] = {}
    for item in rows:
        by_family.setdefault(str(item["family"]), []).append(item)
    for family, items in sorted(by_family.items()):
        lines.append(f"### {family}")
        lines.append("")
        lines.append("| caseId | op | status | detail |")
        lines.append("|---|---|---|---|")
        for item in items[:12]:
            lines.append(
                f"| `{item['caseId']}` | `{item['opType']}` | `{item['status']}` | "
                f"`{item['detail']}` |"
            )
        if len(items) > 12:
            lines.append(f"| … | {len(items) - 12} more | | |")
        lines.append("")
    lines.extend(
        [
            "## 검증 명령",
            "",
            "```",
            "cargo test --lib -- vector_style_arrows_shadows_patterns_and_compound_lines_are_direct m07_pack",
            "node rhwp-studio/e2e/renderer-contract.test.mjs",
            "cargo fmt --all -- --check",
            "node scripts/rust-unit-test-tiers.mjs --check",
            "```",
            "",
            "## 하지 않은 것",
            "",
            "- CanvasKit 기본 렌더러 선언 (28→14 미측정).",
            "- gradientFill / imageFill / rotatedText / verticalText 재작업.",
            "- gym, M08, planet serializer.",
            "- visualItemLimitExceeded 한도 상향.",
            "",
        ]
    )
    # Unique case catalogue — one line per case keeps the doc a real index.
    lines.append("## 전체 caseId 색인")
    lines.append("")
    for item in rows:
        lines.append(
            f"- `{item['caseId']}` · {item['family']} · {item['opType']} · "
            f"{item['status']} · detail={item['detail']}"
        )
    lines.append("")
    WORKING.write_text("\n".join(lines), encoding="utf-8", newline="\n")


def write_public_doc(rows: list[dict[str, object]]) -> None:
    counts = family_counts(rows)
    lines = [
        "---",
        "kind: reference",
        "status: active",
        "canonical: mydocs/tech/canvaskit-parity-implementation.md",
        "last_verified: 2026-09-03",
        "---",
        "",
        "# CanvasKit M07-pack fallback matrix",
        "",
        "This is the public contract for issue #5448. Policy lives in",
        "`src/renderer/canvaskit_policy.rs`. Runtime replay lives in",
        "`rhwp-studio/src/view/canvaskit-renderer.ts`. The machine-readable",
        "matrix is `tests/fixtures/m07_pack/reason-matrix.jsonl` and the",
        "envelope transcripts are `tests/fixtures/m07_pack/envelopes.jsonl`.",
        "",
        "## Direct promotions",
        "",
        "| reason | was | now | proof |",
        "|---|---|---|---|",
        "| `lineArrow` | overlay | Direct | `drawArrowHead` |",
        "| `compoundLine` | overlay | Direct | `drawCompoundLine` |",
        "| `shapeShadow` | overlay | Direct | `resolvedShadow` + translate |",
        "| `lineShadow` | overlay | Direct | same, then compound stroke |",
        "| `patternFill` | overlay | Direct | `drawPatternFill` |",
        "| `unsupportedTextDecoration` | overlay on shape>12 | Direct local no-op | default solid |",
        "| `invalidTabLeader` fill>11 | overlay | Direct skip | switch default |",
        "| `footnoteMarker` | already Direct | Direct | `renderTextRun` |",
        "| `visualItemLimitExceeded` | overlay | unchanged | 4096 shared bound |",
        "",
        f"Matrix rows: {len(rows)}.",
        "",
        "## Family counts",
        "",
        "| family | rows |",
        "|---|---:|",
    ]
    for family, count in sorted(counts.items()):
        lines.append(f"| `{family}` | {count} |")
    lines.extend(
        [
            "",
            "## Fail-closed remainder",
            "",
            "- `invalidGeometry` for non-finite bbox/baseline/fontSize/ratio.",
            "- `invalidTabLeader` for inverted or non-finite leader ranges.",
            "- `visualItemLimitExceeded` above 4096 projected display items.",
            "- `verticalText` / `rotatedText` on special-visual ops.",
            "- `lineTransform` / `shapeTransform` / `gradientFill` / `imageFill`.",
            "- `scriptTextRequiresShaping` for complex scripts without cluster authority.",
            "",
            "## Envelope fields",
            "",
            "Each envelope row is a one-item `CanvasKitReplayPlan` transcript:",
            "`mode`, `hiddenCanvas2dOverlayAllowed`, `directReplayRequired`,",
            "`summary`, and `items[0].{opType,status,reason,detail}`.",
            "The Rust loader rebuilds the same paint op and compares the live plan.",
            "",
        ]
    )
    PUBLIC.write_text("\n".join(lines), encoding="utf-8", newline="\n")


def main() -> None:
    rows = build_matrix()
    envelopes = build_envelopes(rows)
    FIXTURE_DIR.mkdir(parents=True, exist_ok=True)
    write_jsonl(FIXTURE_DIR / "reason-matrix.jsonl", rows)
    write_jsonl(FIXTURE_DIR / "envelopes.jsonl", envelopes)
    write_working_doc(rows)
    write_public_doc(rows)
    print(f"rows={len(rows)} envelopes={len(envelopes)}")
    print(f"wrote {FIXTURE_DIR}")
    print(f"wrote {WORKING}")
    print(f"wrote {PUBLIC}")


if __name__ == "__main__":
    main()
