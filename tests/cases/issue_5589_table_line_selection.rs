//! Issue #5589: 표가 놓인 줄을 문단의 **마지막** 저장 줄로 가정해, 표 뒤에 글자가 이어지는
//! 문단에서 표를 글줄 자리까지 끌어내린다.
//!
//! 재현 문서(코퍼스 00398 `잔류원인조사결과`)의 host 문단은 칸 안에서 두 줄로 저장돼 있다 —
//! 첫 줄이 표 밴드(`vertsize=11608` = 154.8px), 둘째 줄이 표 뒤에 이어지는 글줄(16.0px).
//! 종전에는 마지막 줄의 `vertpos` 를 표 줄로 읽어 표를 158.8px 아래에 그렸고, 표가 예약한
//! 자리에는 빈 밴드만 남고 표는 그 아래 소제목과 겹쳤다.
//!
//! 계약: 표 뒤에 글줄이 이어진다고 해서 표 자리가 달라지면 안 된다. 표 밴드를 담은 저장 줄을
//! 골라 그 줄에 표를 놓는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::style::{CharShape, ParaShape};
use rhwp::model::table::{Cell, Table};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 중첩 표 크기(HWPUNIT) — 96dpi 에서 616.2 × 151.0px.
const NESTED_W: u32 = 46212;
const NESTED_H: u32 = 11326;
/// 표 위·아래 바깥 여백(HWPUNIT). 밴드 높이 = NESTED_H + 141 + 141 = 11608.
const OUTER_MARGIN: i16 = 141;
/// 표 밴드 줄과 그 뒤 글줄의 저장 좌표(코퍼스 00398 실측).
const BAND_VPOS: i32 = 26392;
const BAND_LH: i32 = 11608;
const TEXT_VPOS: i32 = 38300;
const TEXT_LH: i32 = 1200;

fn nested_table() -> Table {
    let mut inner = Cell {
        width: NESTED_W,
        height: NESTED_H,
        ..Default::default()
    };
    inner.paragraphs.push(Paragraph {
        text: "제품명".to_string(),
        ..Default::default()
    });

    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![inner],
        outer_margin_left: OUTER_MARGIN,
        outer_margin_right: OUTER_MARGIN,
        outer_margin_top: OUTER_MARGIN,
        outer_margin_bottom: OUTER_MARGIN,
        ..Default::default()
    };
    table.common.width = NESTED_W;
    table.common.height = NESTED_H;
    table.common.treat_as_char = true;
    table
}

/// 표 host 문단이 `line_segs` 를 `segs` 로 가진 문서.
fn document_with_host_line_segs(segs: Vec<LineSeg>, table_after_text: bool) -> Document {
    // has_preceding_text 를 세우는 앞 문단 — 실제 문서와 같은 배치다.
    let lead = Paragraph {
        text: "나. 사료 급여 내역".to_string(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![line_seg(0, TEXT_LH, 0)],
        ..Default::default()
    };

    let mut host = Paragraph {
        text: "볏짚".to_string(),
        char_count: 11,
        char_offsets: if table_after_text {
            vec![0, 1]
        } else {
            vec![8, 9]
        },
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        ..Default::default()
    };
    host.controls.push(Control::Table(Box::new(nested_table())));
    host.line_segs = segs;

    let outer_cell = Cell {
        width: 47000,
        height: 40000,
        paragraphs: vec![lead, host],
        ..Default::default()
    };
    let mut outer = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![outer_cell],
        ..Default::default()
    };
    outer.common.width = 47000;
    outer.common.height = 40000;

    let mut anchor = Paragraph::default();
    anchor.controls.push(Control::Table(Box::new(outer)));

    let mut section = Section::default();
    section.paragraphs.push(anchor);

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.doc_info.char_shapes = vec![CharShape::default()];
    doc.sections.push(section);
    doc
}

fn line_seg(vertical_pos: i32, line_height: i32, text_start: u32) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos,
        line_height,
        text_height: line_height,
        baseline_distance: line_height * 85 / 100,
        line_spacing: 0,
        segment_width: 47000,
        ..Default::default()
    }
}

/// 가장 안쪽(=중첩) 표 노드의 y. 바깥 표보다 폭이 좁은 것으로 고른다.
fn nested_table_y(doc: &Document) -> f64 {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("재로드 실패");
    let tree = core
        .build_page_render_tree(0)
        .expect("render tree 생성 실패");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON 파싱");

    let mut found: Vec<(f64, f64)> = Vec::new();
    fn walk(node: &serde_json::Value, out: &mut Vec<(f64, f64)>) {
        if let Some(obj) = node.as_object() {
            if obj.get("type").and_then(|v| v.as_str()) == Some("Table") {
                if let Some(b) = obj.get("bbox") {
                    if let (Some(y), Some(w)) = (
                        b.get("y").and_then(|v| v.as_f64()),
                        b.get("w").and_then(|v| v.as_f64()),
                    ) {
                        out.push((w, y));
                    }
                }
            }
            for (_, v) in obj {
                walk(v, out);
            }
        } else if let Some(arr) = node.as_array() {
            for v in arr {
                walk(v, out);
            }
        }
    }
    walk(&json, &mut found);
    assert!(found.len() >= 2, "중첩 표를 찾지 못했다: {found:?}");
    // 가장 좁은 표 = 중첩 표
    found
        .into_iter()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, y)| y)
        .expect("중첩 표 y")
}

// 저장 줄 소속을 검사하려면 글자모양과 실제 제어 슬롯 축도 함께 유효해야 한다.
// 이 합성 계약은 원 기관 문서의 시각 검증을 대신하지 않는다.
#[test]
fn stored_table_slot_keeps_its_origin_when_a_trailing_text_line_is_present() {
    let only_band = nested_table_y(&document_with_host_line_segs(
        vec![line_seg(BAND_VPOS, BAND_LH, 0)],
        false,
    ));
    let with_text = nested_table_y(&document_with_host_line_segs(
        vec![
            line_seg(BAND_VPOS, BAND_LH, 0),
            line_seg(TEXT_VPOS, TEXT_LH, 8),
        ],
        false,
    ));
    assert!(
        (only_band - with_text).abs() <= 0.2,
        "뒤 글줄이 같은 슬롯의 표 원점을 이동시키지 않음: {only_band}, {with_text}"
    );
}

#[test]
fn stored_text_prefix_places_the_table_in_its_own_following_band() {
    let text_first = nested_table_y(&document_with_host_line_segs(
        vec![
            line_seg(BAND_VPOS, TEXT_LH, 0),
            line_seg(TEXT_VPOS, BAND_LH, 2),
        ],
        true,
    ));
    let band_first = nested_table_y(&document_with_host_line_segs(
        vec![
            line_seg(BAND_VPOS, BAND_LH, 0),
            line_seg(TEXT_VPOS, TEXT_LH, 8),
        ],
        false,
    ));
    let stored_offset = f64::from(TEXT_VPOS - BAND_VPOS) / 7200.0 * 96.0;
    assert!(text_first > band_first, "앞 글줄 뒤 표 밴드의 순서를 보존");
    assert!(
        (text_first - band_first - stored_offset).abs() <= 0.2,
        "원본 저장 줄 간격과 실제 표 원점 차이의 관계: {text_first}, {band_first}"
    );
}
