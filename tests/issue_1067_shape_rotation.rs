//! Issue #1067: HWPX 도형 IR 화 + HWP/HWPX 회전 부호 정정 회귀 가드.
//!
//! 정정 영역 3 가지:
//!
//! 1. **HWPX `<hc:pt>` 점 파싱** (`src/parser/hwpx/section.rs::parse_shape_object`):
//!    polygon 의 가변 꼭짓점 (rect 의 pt0/pt1/pt2/pt3 와 별개) — 누락 시 PolygonShape::points
//!    빈 Vec → HWPX 의 도형 path 가 빈 상태로 렌더링되어 도형 미표시.
//!
//! 2. **flip + 회전 동시 적용 시 회전 부호 반전** (`src/renderer/svg.rs::open_shape_transform`,
//!    `src/renderer/web_canvas.rs::open_shape_transform`): 한컴 정답지 시각 표준 정합.
//!    누락 시 첫 도형이 180도 반시계 방향으로 잘못 회전 (작업지시자 보고).
//!
//! 3. **U+FFFC OBJECT REPLACEMENT CHARACTER skip** (`src/renderer/svg.rs::draw_text`,
//!    `src/renderer/web_canvas.rs::draw_text`): inline 컨트롤 (treat_as_char) placeholder 가
//!    paragraph text 의 U+FFFC 로 표현됨 — 시각적으로 invisible 해야 함.
//!
//! 작업지시자 시각 판정 통과 (Stage 1+2+3): "정답지 이미지 정합".

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::parse_document;
use std::fs;
use std::path::Path;

fn read_sample(rel: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e))
}

fn control_kind(ctrl: &Control) -> &'static str {
    match ctrl {
        Control::SectionDef(_) => "SectionDef",
        Control::ColumnDef(_) => "ColumnDef",
        Control::Shape(_) => "Shape",
        _ => "Other",
    }
}

// 최종 SVG를 판독한다. 변환이 path에 적용되거나 group으로 남는 직렬화 차이를
// rotate 문자열 존재로 검사하지 않는다. 이 수용 원본의 출력은 닫힌 직선 다각형이다.
fn final_polygon_points(svg: &str) -> Vec<Vec<(f64, f64)>> {
    let document = roxmltree::Document::parse(svg).expect("최종 SVG 파싱");
    document
        .descendants()
        .filter(|node| node.has_tag_name("path"))
        .map(|node| {
            let path = node.attribute("d").expect("최종 다각형 path");
            let mut points = Vec::new();
            let mut current = (0.0, 0.0);
            let mut closed = false;
            for segment in svgtypes::PathParser::from(path) {
                use svgtypes::PathSegment::*;
                current = match segment.expect("SVG path 명령") {
                    MoveTo { abs, x, y } | LineTo { abs, x, y } => {
                        if abs {
                            (x, y)
                        } else {
                            (current.0 + x, current.1 + y)
                        }
                    }
                    HorizontalLineTo { abs, x } => (if abs { x } else { current.0 + x }, current.1),
                    VerticalLineTo { abs, y } => (current.0, if abs { y } else { current.1 + y }),
                    ClosePath { .. } => {
                        closed = true;
                        continue;
                    }
                    _ => panic!("이 원본의 최종 윤곽은 직선 삼각형이다"),
                };
                points.push(current);
            }
            assert!(closed, "삼각형은 닫힌 path다");
            if points.first() == points.last() {
                points.pop();
            }
            // path/부모 group의 변환까지 적용한 최종 꼭짓점을 검사한다.
            // SVG가 같은 도형을 다른 transform 직렬화 방식으로 표현해도 된다.
            for ancestor in node.ancestors() {
                if let Some(value) = ancestor.attribute("transform") {
                    let transform: svgtypes::Transform = value.parse().expect("SVG 변환");
                    for (x, y) in &mut points {
                        (*x, *y) = (
                            transform.a * *x + transform.c * *y + transform.e,
                            transform.b * *x + transform.d * *y + transform.f,
                        );
                    }
                }
            }
            points
        })
        .collect()
}

fn polygon_bounds(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    assert!(!points.is_empty());
    (
        points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min),
        points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min),
        points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max),
        points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
    )
}

/// HWPX polygon 의 `<hc:pt>` 점들이 IR PolygonShape::points 에 4 개로 매핑.
#[test]
fn issue_1067_hwpx_polygon_points_mapped() {
    let bytes = read_sample("samples/hwpx/shape-001.hwpx");
    let doc = parse_document(&bytes).expect("parse hwpx");
    let mut polygon_count = 0;
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Shape(shape) = ctrl {
                    if let ShapeObject::Polygon(poly) = shape.as_ref() {
                        polygon_count += 1;
                        assert_eq!(
                            poly.points.len(),
                            4,
                            "HWPX polygon points: 4 점 매핑 필요 (parse_shape_object b\"pt\")"
                        );
                    }
                }
            }
        }
    }
    assert!(polygon_count >= 2, "shape-001.hwpx 는 최소 2 polygon 보유");
}

/// HWP 정답지의 polygon points 와 HWPX 의 points 완전 동일 (정답지 정합).
#[test]
fn issue_1067_hwpx_polygon_points_match_oracle() {
    let hwp_bytes = read_sample("samples/shape-001.hwp");
    let hwpx_bytes = read_sample("samples/hwpx/shape-001.hwpx");
    let hwp_doc = parse_document(&hwp_bytes).expect("parse hwp");
    let hwpx_doc = parse_document(&hwpx_bytes).expect("parse hwpx");

    let collect_polygon_points = |doc: &rhwp::model::document::Document| -> Vec<Vec<(i32, i32)>> {
        let mut out = Vec::new();
        for section in &doc.sections {
            for para in &section.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Shape(shape) = ctrl {
                        if let ShapeObject::Polygon(poly) = shape.as_ref() {
                            out.push(poly.points.iter().map(|p| (p.x, p.y)).collect());
                        }
                    }
                }
            }
        }
        out
    };

    let hwp_polys = collect_polygon_points(&hwp_doc);
    let hwpx_polys = collect_polygon_points(&hwpx_doc);
    assert_eq!(hwp_polys, hwpx_polys, "HWP/HWPX polygon points 정합");
}

/// HWPX polygon 의 flip + rotation 정보가 IR 에 정확 보존.
#[test]
fn issue_1067_hwpx_polygon_flip_rotation_preserved() {
    let bytes = read_sample("samples/hwpx/shape-001.hwpx");
    let doc = parse_document(&bytes).expect("parse hwpx");
    let polygons: Vec<&rhwp::model::shape::PolygonShape> = doc
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| {
            if let Control::Shape(shape) = c {
                if let ShapeObject::Polygon(poly) = shape.as_ref() {
                    return Some(poly);
                }
            }
            None
        })
        .collect();
    assert!(polygons.len() >= 2, "polygon 개수 ≥ 2");
    // 도형 1: horz_flip=true, rotation=270
    assert!(
        polygons[0].drawing.shape_attr.horz_flip,
        "도형 1: horz_flip"
    );
    assert_eq!(
        polygons[0].drawing.shape_attr.rotation_angle, 270,
        "도형 1: rotation_angle = 270"
    );
    // 도형 2: horz_flip=false, rotation=90
    assert!(
        !polygons[1].drawing.shape_attr.horz_flip,
        "도형 2: horz_flip 없음"
    );
    assert_eq!(
        polygons[1].drawing.shape_attr.rotation_angle, 90,
        "도형 2: rotation_angle = 90"
    );
}

/// 독립 PDF의 두 삼각형 방향·같은 밴드·좌우 순서를 최종 윤곽으로 검사한다.
#[test]
fn issue_1067_svg_rotation_sign_negated_with_flip() {
    let bytes = read_sample("samples/shape-001.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("원본 파싱");
    let svg = doc.render_page_svg_native(0).expect("최종 SVG");
    let polygons = final_polygon_points(&svg);
    assert_eq!(polygons.len(), 2, "원본 두 도형이 한 번씩 표시된다");
    let bounds: Vec<_> = polygons.iter().map(|p| polygon_bounds(p)).collect();
    for (index, points) in polygons.iter().enumerate() {
        assert_eq!(points.len(), 3, "닫힌 삼각형의 세 꼭짓점");
        let (left, top, right, bottom) = bounds[index];
        assert!(left < right && top < bottom, "도형이 퇴화하지 않는다");
        let epsilon = (right - left).min(bottom - top) * 1e-9;
        let apex: Vec<_> = points
            .iter()
            .filter(|p| (p.1 - top).abs() <= epsilon)
            .collect();
        assert_eq!(apex.len(), 1, "위 꼭짓점 하나");
        let apex_x = if index == 0 { right } else { left };
        assert!(
            (apex[0].0 - apex_x).abs() <= epsilon,
            "한컴 PDF: 첫 도형은 오른쪽 위, 둘째는 왼쪽 위 꼭짓점"
        );
        for corner_x in [left, right] {
            assert!(
                points
                    .iter()
                    .any(|p| (p.0 - corner_x).abs() <= epsilon && (p.1 - bottom).abs() <= epsilon),
                "밑변은 두 아래 꼭짓점을 잇는다"
            );
        }
    }
    assert!(bounds[0].2 < bounds[1].0, "원본 순서와 좌우 분리 보존");
    let epsilon = (bounds[0].2 - bounds[0].0).min(bounds[0].3 - bounds[0].1) * 1e-9;
    assert!(
        (bounds[0].1 - bounds[1].1).abs() <= epsilon,
        "두 도형이 같은 위 밴드에 있다"
    );
    assert!(
        (bounds[0].3 - bounds[1].3).abs() <= epsilon,
        "두 도형의 밑변이 같은 밴드에 있다"
    );
}

/// SVG export 결과에 U+FFFC OBJECT REPLACEMENT CHARACTER 가 emit 되지 않음.
#[test]
fn issue_1067_svg_no_object_replacement_character() {
    let bytes = read_sample("samples/shape-001.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let svg = doc.render_page_svg_native(0).expect("svg");
    assert!(
        !svg.contains('\u{FFFC}'),
        "SVG 에 U+FFFC (OBJ placeholder) 표시되지 않음 (inline 컨트롤 placeholder 는 invisible)"
    );
}

/// Issue #1071: HWPX secPr 도 HWP 와 같은 문단 control stream 슬롯을 가져야 한다.
///
/// secPr 를 `Section.section_def` 로만 보존하고 `Control::SectionDef` 를 만들지 않으면,
/// HWPX paragraph text 의 inline marker 수와 `para.controls` 순서가 어긋난다. 이 경우
/// treat-as-character 도형의 문단 내 가로 위치가 HWP 정답지보다 왼쪽으로 밀린다.
#[test]
fn issue_1071_hwpx_secpr_materialized_in_control_stream() {
    let hwp_bytes = read_sample("samples/shape-001.hwp");
    let hwpx_bytes = read_sample("samples/hwpx/shape-001.hwpx");
    let hwp_doc = parse_document(&hwp_bytes).expect("parse hwp");
    let hwpx_doc = parse_document(&hwpx_bytes).expect("parse hwpx");

    let hwp_para = &hwp_doc.sections[0].paragraphs[0];
    let hwpx_para = &hwpx_doc.sections[0].paragraphs[0];

    let hwp_kinds: Vec<_> = hwp_para.controls.iter().map(control_kind).collect();
    let hwpx_kinds: Vec<_> = hwpx_para.controls.iter().map(control_kind).collect();
    assert_eq!(
        hwpx_kinds, hwp_kinds,
        "HWPX secPr/colPr/shape control stream 은 HWP 정답지와 같은 순서여야 함"
    );
    assert_eq!(
        hwpx_para.char_offsets, hwp_para.char_offsets,
        "HWPX char_offsets 도 HWP 정답지와 같아야 TAC control 위치가 안정됨"
    );
    assert_eq!(
        hwpx_para.control_text_positions(),
        hwp_para.control_text_positions(),
        "HWPX control_text_positions 는 HWP 정답지와 같아야 함"
    );
}

/// Issue #1071: 같은 원문의 두 포맷은 최종 도형 윤곽과 배치 관계가 같아야 한다.
#[test]
fn issue_1071_hwpx_tac_shape_svg_centers_match_hwp() {
    let hwp_bytes = read_sample("samples/shape-001.hwp");
    let hwpx_bytes = read_sample("samples/hwpx/shape-001.hwpx");
    let hwp_doc = rhwp::wasm_api::HwpDocument::from_bytes(&hwp_bytes).expect("HWP 파싱");
    let hwpx_doc = rhwp::wasm_api::HwpDocument::from_bytes(&hwpx_bytes).expect("HWPX 파싱");
    let hwp = final_polygon_points(&hwp_doc.render_page_svg_native(0).expect("HWP SVG"));
    let hwpx = final_polygon_points(&hwpx_doc.render_page_svg_native(0).expect("HWPX SVG"));
    assert_eq!(hwp.len(), 2, "원본 도형 수");
    assert_eq!(hwpx.len(), hwp.len(), "포맷 간 도형 소유 수 보존");
    for (hwp_points, hwpx_points) in hwp.iter().zip(&hwpx) {
        assert_eq!(hwp_points.len(), hwpx_points.len(), "꼭짓점 소유와 순서");
        let (left, top, right, bottom) = polygon_bounds(hwp_points);
        let tolerance = (right - left).min(bottom - top) * 1e-6;
        for ((hwp_x, hwp_y), (hwpx_x, hwpx_y)) in hwp_points.iter().zip(hwpx_points) {
            assert!(
                (hwp_x - hwpx_x).abs() <= tolerance && (hwp_y - hwpx_y).abs() <= tolerance,
                "두 포맷의 같은 도형 꼭짓점이 대응한다"
            );
        }
    }
}
