//! [#6923] 본문을 감싼 1칸 표가 한/글이 적어 둔 쪽 프레임을 지나쳐 용지 밖까지 채운다.
//!
//! ## 무엇이 문제였나
//!
//! `148738070` 보도자료는 본문 전체(87문단)를 1칸 `RowBreak` 표로 감싼 서식이라 쪽 나눔이
//! **칸 안에서** 일어난다. 한/글이 저장한 사다리는 4쪽 끝을 **빈 문단의 vpos 되감김**으로
//! 적어 두었다.
//!
//! ```text
//!   p[70] vpos=66790 lh=1400 → 끝 68190HU (= 909px, 예산의 98%)
//!   p[71] vpos=0                      ← 한/글이 여기서 쪽을 넘긴다
//!   p[72] vpos=2088  (4 기대효과 및 향후계획 제목 상자)
//! ```
//!
//! 그런데 빈 문단의 되감김은 `#1488` 계약대로 하드 브레이크로 올리지 않는다(기계 문서의
//! 촘촘한 오버레이 리셋이 쪽을 양산했다). 그래서 조각 컷이 그 경계를 지나쳐 다음 쪽 몫인
//! 제목 상자와 5줄을 4쪽에 실었고, 마지막 줄이 **용지 밖**(1134.9px, 용지 1122.5)까지 갔다.
//!
//! ## 수정
//!
//! HWPX 쪽에 이미 있던 판별(`#5880`: 되감김까지 쌓인 높이가 예산의 ≥70%면 저장 쪽 프레임)을
//! HWP5 저장 조판에도 준다. 신호는 `stored_frame_break_before` 가 아니라 **되감김 기하**
//! (`page_frame_reset_before`)다 — 전자는 겹치는 줄 상자(이 문서 p46→p47)도 참이라 쪽
//! 경계가 아닌 자리에서 끊겨 쪽수가 7→8로 늘었다(실측).
//!
//! ## 정답지
//!
//! `tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf`
//! (한/글 2020 11.0.0.9136, 7쪽). 4쪽은 법조문 표(`1. 거짓·과장의표시·광고`)에서 끝나고
//! **5쪽이 `4 기대효과 및 향후계획` 제목으로 시작**한다.
//!
//! ## 중첩 표의 저장 줄 소속 (같은 이슈의 둘째 축)
//!
//! 한/글이 저장한 사다리는 표를 소유한 줄을 따로 적는다(p69: ls[0] 49113HU 글줄 ·
//! ls[1] 51229HU 표 밴드). 종전 렌더는 문단 첫 줄 좌표에 표를 앉혀 앞 글줄 위로
//! 28.2px 올라왔다 — 4쪽 `□ 적용법조` 줄(735.0..753.7)과 법조문 표(740.0)가 겹쳤다
//! (글자 겹침 13건). 정본은 그 둘을 34.8px 띄운다(줄 765.9px · 표 800.7px).
//! 저장 델타(33.3px)를 더해 앉히면 겹침이 사라진다.
//!
//! ## 메인터너 보정: 저장 프레임과 소유 줄의 실제 원점
//!
//! 이어받는 컷의 source unit을 프레임 원점으로 쓴다. 첫 빈 문단도 공간을 점유하므로
//! 첫 가시 제목으로 원점을 대체하지 않는다. 이것으로 앞 본문의 압축과 뒤 저장 vpos
//! 스냅 사이에 생겼던 116px 빈 띠를 없앤다. 제목 표의 선언 높이 자체는 2414HU다.
//! 중첩 표의 가로 원점은 자기 줄 들여쓰기와 자기 앞 텍스트만 소비한다.
//! 독립 PDF의 세로 좌표와 wrapper 대비 가로 좌표를 아래 테스트로 확인한다.
//! 글꼴 모양과 바깥 wrapper의 잔여 외곽선 차이를 픽셀 일치로 주장하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_document, scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("픽스처")).expect("문서 로드")
}

/// 본문(Body) 상자.
fn body_box(root: &RenderNode) -> (f64, f64) {
    fn find<'a>(node: &'a RenderNode, out: &mut Option<&'a RenderNode>) {
        if out.is_some() {
            return;
        }
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            *out = Some(node);
            return;
        }
        for child in &node.children {
            find(child, out);
        }
    }
    let mut body = None;
    find(root, &mut body);
    let body = body.expect("Body");
    (body.bbox.y, body.bbox.y + body.bbox.height)
}

fn has_visible_text(node: &RenderNode) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.display_or_text().trim().is_empty())
        || node.children.iter().any(has_visible_text)
}

/// 감싼 칸 안의 직계 내용(줄·중첩 표)을 위에서 아래로.
fn wrapper_cell_items(root: &RenderNode) -> Vec<(&'static str, f64, f64)> {
    fn find_cell<'a>(node: &'a RenderNode, out: &mut Option<&'a RenderNode>) {
        if out.is_some() {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            *out = Some(node);
            return;
        }
        for child in &node.children {
            find_cell(child, out);
        }
    }
    let mut cell = None;
    find_cell(root, &mut cell);
    let Some(cell) = cell else {
        return Vec::new();
    };
    cell.children
        .iter()
        .filter_map(|child| match child.node_type {
            RenderNodeType::TextLine(_) if has_visible_text(child) => {
                Some(("TextLine", child.bbox.y, child.bbox.y + child.bbox.height))
            }
            RenderNodeType::Table(_) => {
                Some(("Table", child.bbox.y, child.bbox.y + child.bbox.height))
            }
            _ => None,
        })
        .collect()
}

/// 쪽수가 정본과 같다.
#[test]
fn page_count_matches_the_hancom_oracle() {
    assert_eq!(core().page_count(), 7, "한/글 2020 정본은 7쪽이다");
}

/// 4쪽 내용이 본문 상자 안에서 끝난다 — 용지 밖으로 나가지 않는다.
#[test]
fn page4_content_stays_inside_the_body() {
    let core = core();
    let tree = core.build_page_render_tree(3).expect("4쪽 render tree");
    let (_body_top, body_bottom) = body_box(&tree.root);
    let items = wrapper_cell_items(&tree.root);
    assert!(!items.is_empty(), "감싼 칸의 내용을 찾지 못했다");

    let lowest = items
        .iter()
        .map(|(_, _, bottom)| *bottom)
        .fold(f64::MIN, f64::max);
    assert!(
        lowest <= body_bottom + 0.5,
        "4쪽 내용이 본문 바닥을 넘는다: 최하단 {lowest:.1}px, 본문 바닥 {body_bottom:.1}px \
         (수정 전 1134.9px — 용지 1122.5px 밖)"
    );
}

/// 5쪽은 저장 쪽 프레임이 가리키는 `4 기대효과 및 향후계획` 제목 상자로 시작한다.
#[test]
fn page5_starts_at_the_stored_page_frame() {
    let core = core();
    let tree = core.build_page_render_tree(4).expect("5쪽 render tree");
    let items = wrapper_cell_items(&tree.root);
    let first = items.first().expect("5쪽 첫 내용");
    assert_eq!(
        first.0, "Table",
        "정본 5쪽은 제목 상자(1×3 표)로 시작한다 — 수정 전에는 그 상자가 4쪽에 있었다. got {items:?}"
    );

    fn first_text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(run.display_or_text());
        }
        for child in &node.children {
            first_text(child, out);
        }
    }
    let mut text = String::new();
    first_text(&tree.root, &mut text);
    assert!(
        text.contains("기대효과"),
        "5쪽 첫머리가 '4 기대효과 및 향후계획' 이어야 한다: {:?}",
        text.chars().take(40).collect::<String>()
    );
}

/// 4쪽에서 글자 겹침이 없다 — 중첩 표가 자기 저장 줄에 앉는다.
#[test]
fn page4_has_no_text_overlap() {
    let core = core();
    let tree = core.build_page_render_tree(3).expect("4쪽 render tree");
    let anomalies = scan_page(3, &tree.root, core.page_count(), &AnomalyOptions::default());
    assert!(
        anomalies.text_overlap.is_empty(),
        "글자 겹침 {}건 (수정 전 13건: `□ 적용법조` 줄 위로 법조문 표가 올라왔다)",
        anomalies.text_overlap.len()
    );
}

/// 문서 전체에 쪽 밖 요소가 없다.
#[test]
fn no_off_canvas_in_the_document() {
    let core = core();
    let anomalies = scan_document(&core, &AnomalyOptions::default()).expect("layout-anomaly");
    let off: Vec<String> = anomalies
        .pages
        .iter()
        .flat_map(|page| {
            page.off_canvas
                .iter()
                .map(move |a| format!("{}쪽 {} {:.1}px", page.page + 1, a.path, a.max_over()))
        })
        .collect();
    assert!(
        off.is_empty(),
        "쪽 밖 요소가 남아 있다 (수정 전 4쪽 표 12.4px): {off:?}"
    );
}

#[test]
fn continuation_heading_boxes_follow_the_pdf_stored_frame() {
    fn text(node: &RenderNode) -> String {
        let mut out = match &node.node_type {
            RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
            _ => String::new(),
        };
        for child in &node.children {
            out.push_str(&text(child));
        }
        out
    }
    fn heading(node: &RenderNode, needle: &str, found: &mut Vec<f64>) {
        if matches!(node.node_type, RenderNodeType::Table(_))
            && node.bbox.height < 50.0
            && text(node).contains(needle)
        {
            found.push(node.bbox.y);
        }
        for child in &node.children {
            heading(child, needle, found);
        }
    }
    let core = core();
    // Independent Hancom PDF vector top strokes (96dpi). The PDF media box
    // is 841pt high; the original HWP page is 84188 HU = 841.88pt. Normalize
    // the PDF coordinate system, rather than increasing the stroke tolerance.
    let pdf_to_source =
        f64::from(core.document().sections[0].section_def.page_def.height) / 100.0 / 841.0;
    for (page, needle, pdf_top) in [(3, "조치내용", 532.38), (4, "기대효과", 129.78)] {
        let expected = pdf_top * pdf_to_source;
        let tree = core.build_page_render_tree(page).unwrap();
        let mut tops = Vec::new();
        heading(&tree.root, needle, &mut tops);
        assert_eq!(tops.len(), 1, "heading {needle}: {tops:?}");
        assert!(
            (tops[0] - expected).abs() < 0.5,
            "page {} heading {needle}: top={}, PDF={expected}",
            page + 1,
            tops[0]
        );
    }
}

#[test]
fn page5_keeps_the_empty_leading_source_slot() {
    fn empty_line(node: &RenderNode) -> bool {
        (matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(71))
            && !has_visible_text(node)
            && (node.bbox.height - 1400.0 * 96.0 / 7200.0).abs() < 0.05
            && node.bbox.y > 98.0
            && node.bbox.y < 102.0)
            || node.children.iter().any(empty_line)
    }
    assert!(
        empty_line(&core().build_page_render_tree(4).unwrap().root),
        "the stored blank paragraph must retain its line box before the heading"
    );
}

/// Independent PDF border centers, relative to the wrapper's left border.
/// Prefix text belongs only to the object's saved line; following text and
/// earlier lines must not shift the object. Hanging indent belongs to line 2.
#[test]
fn continuation_tables_follow_their_owner_line_horizontal_origin() {
    fn table_x(node: &RenderNode, para: usize) -> Option<f64> {
        if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(para)) {
            return Some(node.bbox.x);
        }
        node.children.iter().find_map(|child| table_x(child, para))
    }
    let core = core();
    let scale = f64::from(core.document().sections[0].section_def.page_def.width) / 100.0 / 595.0;
    for (page, para, pdf_relative) in [
        (3, 63, 11.11),
        (3, 66, 19.03),
        (3, 69, 22.55),
        (4, 72, 11.11),
    ] {
        let tree = core.build_page_render_tree(page).unwrap();
        let actual = table_x(&tree.root, para).unwrap() - table_x(&tree.root, 5).unwrap();
        let expected = pdf_relative * scale;
        assert!(
            (actual - expected).abs() < 0.5,
            "page {} para {para}: relative x={actual}, PDF={expected}",
            page + 1
        );
    }
}

/// 1쪽에서 본문을 감싼 표(1행×1열, 머리 표보다 큰 상자)와 그 칸의 직계 내용.
///
/// 1쪽은 머리 표(5행×4열)가 먼저 나오므로 `wrapper_cell_items` 의 "첫 칸" 규칙을 쓸 수
/// 없다. 감싼 표는 본문 절반을 넘는 높이로 구분한다.
fn page1_wrapper(root: &RenderNode) -> (&RenderNode, Vec<(&'static str, f64, f64)>) {
    fn find(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::Table(_))
            && node.bbox.height > 400.0
            && node
                .children
                .iter()
                .any(|child| matches!(child.node_type, RenderNodeType::TableCell(_)))
        {
            return Some(node);
        }
        node.children.iter().find_map(find)
    }
    let table = find(root).expect("1쪽 감싼 표");
    let cell = table
        .children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::TableCell(_)))
        .expect("감싼 칸");
    let items = cell
        .children
        .iter()
        .filter_map(|child| match child.node_type {
            RenderNodeType::TextLine(_) if has_visible_text(child) => {
                Some(("TextLine", child.bbox.y, child.bbox.y + child.bbox.height))
            }
            RenderNodeType::Table(_) => {
                Some(("Table", child.bbox.y, child.bbox.y + child.bbox.height))
            }
            _ => None,
        })
        .collect();
    (table, items)
}

/// [#6923 잔여 축 A] 1쪽 중첩 표가 저장 사다리가 가리키는 자리에 앉는다.
///
/// 감싼 칸의 `p[2]`·`p[3]` 은 **글자가 없는 문단**이지만 저장 사다리는 그 점유를 적어
/// 두었다 — `p[2] vpos=9800 lh=1400 ls=−560`(전진 840HU = 11.2px) ·
/// `p[3] vpos=10640 lh=1300 ls=−948`(전진 352HU = 4.7px). 이 문서의 줄 상자는 음수
/// `line_spacing` 으로 계통적으로 겹치므로(같은 파일 `#5585` 주석), "다음 줄이 이 줄
/// 바닥 아래에서 시작" 을 요구하던 종전 보존 판별은 두 줄을 모두 거부했고 1칸 RowBreak
/// 칸의 빈 줄 접기가 둘을 0 높이로 만들었다. 그래서 뒤따르는 중첩 표가 15.9px 위로
/// 올라가 앞 문단과의 간격이 정본의 절반이 됐다.
///
/// 기대값은 구현이 아니라 정본에서 온다 —
/// `tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf`
/// 1쪽을 96dpi 로 래스터해 가로 괘선 행을 읽으면 이 표의 위·아래 괘선이 **488 · 932**
/// 다(같은 방법으로 읽은 위쪽 괘선 102·177·202·246·281·284·338 은 수정 전후 모두 일치).
/// 수정 전 값은 472.8 · 917.8 로 15.2px · 14.2px 어긋났다.
#[test]
fn page1_nested_table_sits_on_its_stored_ladder_position() {
    let core = core();
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let (_, items) = page1_wrapper(&tree.root);
    let (_, top, bottom) = items
        .iter()
        .find(|(kind, _, _)| *kind == "Table")
        .copied()
        .expect("1쪽 감싼 칸 안의 중첩 표");
    // 괘선 행 판독(±1px)과 테두리 굵기를 감안한 허용치.
    assert!(
        (top - 488.0).abs() <= 2.0,
        "중첩 표 윗변 {top:.1}px — 한/글 2020 정본 괘선 488px (수정 전 472.8px)"
    );
    assert!(
        (bottom - 932.0).abs() <= 2.0,
        "중첩 표 아랫변 {bottom:.1}px — 한/글 2020 정본 괘선 932px (수정 전 917.8px)"
    );
}

/// [#6923 잔여 축 A] 1쪽 감싼 표의 상자가 **자기 조각이 소유한 마지막 줄**을 담는다.
///
/// 측정(`cell_units`)만 저장 전진을 받고 배치가 받지 않으면 조각 상자는 늘어나는데
/// 내용은 제자리라 아래 테두리가 마지막 줄 **위로** 지나간다. 수정 전에는 반대 방향으로
/// 같은 결함이 있었다 — 상자 아래 987.9px, 칸 내용 바닥 997.8px 로 각주 마지막 줄을
/// 테두리가 가로질렀다. 두 경로가 같은 결과를 소비하는지 이 불변식으로 고정한다.
///
/// 정본의 상자 아래는 1022px 로 여기보다 더 아래다(쪽 상자 고정 축, #7095). 이 검사는
/// 그 축을 주장하지 않고 "상자가 자기 내용을 담는다" 만 고정한다.
#[test]
fn page1_wrapper_frame_contains_its_own_last_line() {
    let core = core();
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");

    let (table, items) = page1_wrapper(&tree.root);
    let frame_bottom = table.bbox.y + table.bbox.height;
    let last_line_bottom = items
        .iter()
        .map(|(_, _, bottom)| *bottom)
        .fold(f64::MIN, f64::max);
    assert!(
        frame_bottom + 0.5 >= last_line_bottom,
        "감싼 표 아랫변 {frame_bottom:.1}px 가 자기 마지막 줄 바닥 {last_line_bottom:.1}px \
         위에 있다 (수정 전 987.9 < 997.8 — 테두리가 각주 마지막 줄을 가로질렀다)"
    );
}
