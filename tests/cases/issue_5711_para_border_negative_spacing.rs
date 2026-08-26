//! Issue #5711: 줄간격이 음수인 문단에서 문단 아래 테두리가 줄 상자 아래가 아니라 글자 위에
//! 그려진다.
//!
//! 테두리 범위의 아래 경계로 줄 전진값 `y` 를 썼다. 줄 배치는 `y += line_height + line_spacing`
//! 으로 나아가는데, 저장 줄간격이 음수면 그 값이 마지막 줄 상자 아래보다 위에 놓인다.
//! 재현 문서(3143955 제목 `안과ㆍ정신건강의학과 …`)는 줄 상자 147.6~171.6px 에 줄간격
//! −720(−9.6px) 이라, 이중선이 기준선(168.0) 위인 159.7 / 163.7 에 그려졌다.
//!
//! 계약: 문단 테두리의 아래 변은 마지막 줄 상자 아래 경계 아래에 있다.

use rhwp::document_core::DocumentCore;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::style::{BorderFill, BorderLine, BorderLineType, ParaShape};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 줄 높이(HWPUNIT) — 24.0px.
const LINE_H: i32 = 1800;
/// 음수 줄간격(HWPUNIT) — −9.6px.
const NEG_SPACING: i32 = -720;

fn document_with_bordered_paragraph(line_spacing: i32) -> Document {
    let mut doc = Document::default();

    let mut shape = ParaShape::default();
    shape.border_fill_id = 1;
    doc.doc_info.para_shapes = vec![shape];

    // 아래 변만 있는 문단 테두리.
    let mut fill = BorderFill::default();
    fill.borders[3] = BorderLine {
        line_type: BorderLineType::Solid,
        width: 1,
        color: 0,
    };
    doc.doc_info.border_fills = vec![fill];

    let mut para = Paragraph {
        text: "안과ㆍ정신건강의학과 질환자 취득 제한 자격ㆍ면허".to_string(),
        ..Default::default()
    };
    para.para_shape_id = 0;
    para.line_segs = vec![LineSeg {
        text_start: 0,
        vertical_pos: 0,
        line_height: LINE_H,
        text_height: LINE_H,
        baseline_distance: LINE_H * 85 / 100,
        line_spacing,
        segment_width: 48188,
        ..Default::default()
    }];

    let mut section = Section::default();
    section.paragraphs.push(para);
    doc.sections.push(section);
    doc
}

/// (테두리 선들의 최대 y, 글자 줄 상자의 아래 끝).
fn border_bottom_vs_line_box(doc: &Document) -> (f64, f64) {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("재로드 실패");
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");

    let mut border_max = f64::MIN;
    let mut line_bottom = f64::MIN;
    fn walk(node: &serde_json::Value, border_max: &mut f64, line_bottom: &mut f64) {
        if let Some(obj) = node.as_object() {
            let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let bbox = obj
                .get("bbox")
                .and_then(|b| Some((b.get("y")?.as_f64()?, b.get("h")?.as_f64()?)));
            if let Some((y, h)) = bbox {
                match ty {
                    "Line" | "Rect" => *border_max = border_max.max(y + h),
                    "TextLine" => *line_bottom = line_bottom.max(y + h),
                    _ => {}
                }
            }
            for (_, v) in obj {
                walk(v, border_max, line_bottom);
            }
        } else if let Some(arr) = node.as_array() {
            for v in arr {
                walk(v, border_max, line_bottom);
            }
        }
    }
    walk(&json, &mut border_max, &mut line_bottom);
    (border_max, line_bottom)
}

#[test]
fn issue_5711_border_stays_below_the_line_box() {
    let (border_bottom, line_bottom) =
        border_bottom_vs_line_box(&document_with_bordered_paragraph(NEG_SPACING));
    assert!(
        line_bottom > f64::MIN,
        "글자 줄 상자를 찾지 못했다 — 전제 불성립"
    );
    assert!(
        border_bottom > f64::MIN,
        "문단 테두리 노드를 찾지 못했다 — 전제 불성립"
    );
    assert!(
        border_bottom >= line_bottom - 1.0,
        "음수 줄간격 문단에서 테두리가 줄 상자 안({border_bottom:.1} < {line_bottom:.1})으로 \
         올라와 글자를 가로지른다 (#5711)"
    );
}

#[test]
fn issue_5711_positive_spacing_is_unchanged() {
    // 양수 줄간격이면 전진값이 줄 상자 아래보다 아래라 종전과 같아야 한다.
    let (border_bottom, line_bottom) =
        border_bottom_vs_line_box(&document_with_bordered_paragraph(600));
    assert!(
        border_bottom >= line_bottom - 1.0,
        "양수 줄간격 문단에서 테두리가 줄 상자 위로 올라갔다 ({border_bottom:.1} < {line_bottom:.1})"
    );
}
