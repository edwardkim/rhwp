//! Issue #2164: 표 셀에서 Enter로 만든 문단의 vpos가 앞 문단과 겹치는 회귀.
//!
//! 셀 문단 분할 뒤 `LINE_SEG.vertical_pos` 축을 다시 연결하지 않으면 새 문단이
//! 앞 문단과 같은 셀 상단에 배치된다. 실제 제보 원본에서 모델 vpos와 캐럿 y가
//! 모두 문단 순서대로 증가하는지 검증한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

const SAMPLE: &str = "samples/issue2164/의견제출서(양식).hwp";

fn load_sample() -> DocumentCore {
    let bytes = std::fs::read(SAMPLE).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"))
}

fn find_target_cell(core: &DocumentCore) -> (usize, usize, usize) {
    for (parent_para_idx, para) in core.document().sections[0].paragraphs.iter().enumerate() {
        for (control_idx, control) in para.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            let Some(heading) = table.cells.iter().find(|cell| {
                cell.paragraphs
                    .iter()
                    .any(|para| para.text.contains("의견제출 요지"))
            }) else {
                continue;
            };
            let target_row = heading.row + heading.row_span;
            let target_idx = table
                .cells
                .iter()
                .position(|cell| cell.row == target_row && cell.col == heading.col)
                .expect("의견제출 요지 다음 입력 셀");
            return (parent_para_idx, control_idx, target_idx);
        }
    }
    panic!("의견제출 요지 표를 찾지 못함");
}

fn cell_paragraphs(
    core: &DocumentCore,
    parent: usize,
    control: usize,
    cell: usize,
) -> &[rhwp::model::paragraph::Paragraph] {
    match &core.document().sections[0].paragraphs[parent].controls[control] {
        Control::Table(table) => &table.cells[cell].paragraphs,
        other => panic!("대상 컨트롤이 표가 아님: {other:?}"),
    }
}

fn cursor_y(
    core: &DocumentCore,
    parent: usize,
    control: usize,
    cell: usize,
    cell_para: usize,
    offset: usize,
) -> f64 {
    let json = core
        .get_cursor_rect_in_cell_native(0, parent, control, cell, cell_para, offset)
        .unwrap_or_else(|e| panic!("cell paragraph {cell_para} cursor rect: {e}"));
    serde_json::from_str::<Value>(&json).expect("cursor rect JSON")["y"]
        .as_f64()
        .expect("cursor rect y")
}

fn first_vpos(paragraphs: &[rhwp::model::paragraph::Paragraph], index: usize) -> i32 {
    paragraphs[index]
        .line_segs
        .first()
        .unwrap_or_else(|| panic!("cell paragraph {index} LINE_SEG"))
        .vertical_pos
}

#[test]
fn enter_in_table_cell_keeps_following_paragraphs_below_previous_paragraph() {
    let mut core = load_sample();
    let (parent, control, cell) = find_target_cell(&core);
    assert_eq!(cell_paragraphs(&core, parent, control, cell).len(), 2);

    let text = "1212121212121212121";
    core.insert_text_in_cell_native(0, parent, control, cell, 0, 0, text)
        .expect("셀 텍스트 입력");
    core.split_paragraph_in_cell_native(0, parent, control, cell, 0, text.chars().count(), None)
        .expect("셀 문단 분할");

    let paragraphs = cell_paragraphs(&core, parent, control, cell);
    assert_eq!(paragraphs.len(), 3, "Enter로 셀 문단이 하나 늘어야 함");
    let vpos = [
        first_vpos(paragraphs, 0),
        first_vpos(paragraphs, 1),
        first_vpos(paragraphs, 2),
    ];
    assert!(
        vpos[0] < vpos[1] && vpos[1] < vpos[2],
        "Enter 뒤 셀 문단 vpos가 순서대로 증가해야 함: {vpos:?}"
    );

    let y = [
        cursor_y(&core, parent, control, cell, 0, text.chars().count()),
        cursor_y(&core, parent, control, cell, 1, 0),
        cursor_y(&core, parent, control, cell, 2, 0),
    ];
    assert!(
        y[0] < y[1] && y[1] < y[2],
        "Enter 뒤 캐럿 y가 문단 순서대로 증가해야 함: {y:?}"
    );

    let saved = core.export_hwp_native().expect("편집 HWP 저장");
    let reopened = DocumentCore::from_bytes(&saved).expect("편집 HWP 재로드");
    let (saved_parent, saved_control, saved_cell) = find_target_cell(&reopened);
    let saved_paragraphs = cell_paragraphs(&reopened, saved_parent, saved_control, saved_cell);
    assert_eq!(saved_paragraphs.len(), 3, "저장 후 셀 문단 수 보존");
    let saved_vpos = [
        first_vpos(saved_paragraphs, 0),
        first_vpos(saved_paragraphs, 1),
        first_vpos(saved_paragraphs, 2),
    ];
    assert!(
        saved_vpos[0] < saved_vpos[1] && saved_vpos[1] < saved_vpos[2],
        "저장 후에도 셀 문단 vpos가 순서대로 증가해야 함: {saved_vpos:?}"
    );
    let saved_y = [
        cursor_y(
            &reopened,
            saved_parent,
            saved_control,
            saved_cell,
            0,
            text.chars().count(),
        ),
        cursor_y(&reopened, saved_parent, saved_control, saved_cell, 1, 0),
        cursor_y(&reopened, saved_parent, saved_control, saved_cell, 2, 0),
    ];
    assert!(
        saved_y[0] < saved_y[1] && saved_y[1] < saved_y[2],
        "저장 후 캐럿 y가 문단 순서대로 증가해야 함: {saved_y:?}"
    );
}

#[test]
fn backspace_merge_then_enter_reuses_the_same_cell_paragraph_flow() {
    let mut core = load_sample();
    let (parent, control, cell) = find_target_cell(&core);
    let text = "1212121212121212121";

    core.insert_text_in_cell_native(0, parent, control, cell, 0, 0, text)
        .expect("셀 텍스트 입력");
    core.split_paragraph_in_cell_native(0, parent, control, cell, 0, text.chars().count(), None)
        .expect("첫 Enter");
    core.merge_paragraph_in_cell_native(0, parent, control, cell, 1)
        .expect("Backspace 문단 병합");
    assert_eq!(cell_paragraphs(&core, parent, control, cell).len(), 2);

    core.split_paragraph_in_cell_native(0, parent, control, cell, 0, text.chars().count(), None)
        .expect("두 번째 Enter");
    let y = [
        cursor_y(&core, parent, control, cell, 0, text.chars().count()),
        cursor_y(&core, parent, control, cell, 1, 0),
        cursor_y(&core, parent, control, cell, 2, 0),
    ];
    assert!(
        y[0] < y[1] && y[1] < y[2],
        "Backspace 뒤 다시 Enter해도 같은 문단 흐름이어야 함: {y:?}"
    );
}

#[test]
fn repeated_enter_in_table_cell_advances_to_the_new_third_paragraph() {
    let mut core = load_sample();
    let (parent, control, cell) = find_target_cell(&core);
    let first_text = "1111";
    let second_text = "2222";

    core.insert_text_in_cell_native(0, parent, control, cell, 0, 0, first_text)
        .expect("첫 셀 텍스트 입력");
    core.split_paragraph_in_cell_native(
        0,
        parent,
        control,
        cell,
        0,
        first_text.chars().count(),
        None,
    )
    .expect("첫 Enter");
    core.insert_text_in_cell_native(0, parent, control, cell, 1, 0, second_text)
        .expect("두 번째 셀 텍스트 입력");
    core.split_paragraph_in_cell_native(
        0,
        parent,
        control,
        cell,
        1,
        second_text.chars().count(),
        None,
    )
    .expect("두 번째 Enter");

    let paragraphs = cell_paragraphs(&core, parent, control, cell);
    assert_eq!(
        paragraphs.len(),
        4,
        "연속 Enter로 셀 문단이 두 개 늘어야 함"
    );
    let vpos: Vec<_> = (0..paragraphs.len())
        .map(|index| first_vpos(paragraphs, index))
        .collect();
    assert!(
        vpos.windows(2).all(|pair| pair[0] < pair[1]),
        "두 번째 Enter 뒤에도 셀 문단 vpos가 순서대로 증가해야 함: {vpos:?}"
    );

    let y = [
        cursor_y(&core, parent, control, cell, 0, first_text.chars().count()),
        cursor_y(&core, parent, control, cell, 1, second_text.chars().count()),
        cursor_y(&core, parent, control, cell, 2, 0),
        cursor_y(&core, parent, control, cell, 3, 0),
    ];
    assert!(
        y.windows(2).all(|pair| pair[0] < pair[1]),
        "두 번째 Enter 뒤 캐럿 y가 새 세 번째 문단까지 증가해야 함: {y:?}"
    );
}

// ── Issue #6882: 편집 세션에서 자란 표의 재조판 ──────────────────────────────
//
// 셀 Enter 로 표가 자라도 (1) 분할/병합 명령이 host 문단 측정 캐시를 무효화하지
// 않아 표가 저장 형상에 얼어붙고, (2) 저장 시점 형상 전용 보정(TAC 비례 축소,
// 선언 높이 fit-down, 저장 vpos 사다리)이 편집 후의 낡은 좌표로 작동해 후행
// 문단이 커진 표와 겹치거나 쪽 밖으로 밀렸다. 한글 오라클(합성 픽스처의 원본
// 실측): 표가 행 단위로 자라다 쪽 잔여를 넘으면 후행 문단 → RowBreak 표 분할 →
// 표 통째 이월 순으로 쪽이 늘어난다(Enter 8회 3쪽, 20회 4쪽). 병합으로 전량
// 되돌리면 로드 시점 배분으로 원복된다(행 하한 = baseline_row_heights).

const GROWTH_SAMPLE: &str = "samples/issue6882/synth_cell_enter_table_growth.hwp";
const GROWTH_TABLE_PARA: usize = 1;
const GROWTH_TABLE_CTRL: usize = 0;
const GROWTH_CELL: usize = 31;

fn load_growth_sample() -> DocumentCore {
    let bytes =
        std::fs::read(GROWTH_SAMPLE).unwrap_or_else(|e| panic!("read {GROWTH_SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {GROWTH_SAMPLE}: {e}"))
}

/// 픽스처 표의 성장 셀 꼬리에 Enter(문단 분할)를 `count`회 넣는다.
/// 반환값은 첫 분할 지점 문단 인덱스(병합 원복의 기준점).
fn enter_in_growth_cell(core: &mut DocumentCore, count: usize) -> usize {
    let (last_cp, last_len) = {
        let Control::Table(table) =
            &core.document().sections[0].paragraphs[GROWTH_TABLE_PARA].controls[GROWTH_TABLE_CTRL]
        else {
            panic!("픽스처 표가 없음");
        };
        assert!(table.common.treat_as_char, "픽스처 표는 TAC 자리 표시");
        let paragraphs = &table.cells[GROWTH_CELL].paragraphs;
        let last = paragraphs.len() - 1;
        (last, paragraphs[last].char_offsets.len())
    };
    for i in 0..count {
        let (cell_para, offset) = if i == 0 {
            (last_cp, last_len)
        } else {
            (last_cp + i, 0)
        };
        core.split_paragraph_in_cell_native(
            0,
            GROWTH_TABLE_PARA,
            GROWTH_TABLE_CTRL,
            GROWTH_CELL,
            cell_para,
            offset,
            None,
        )
        .unwrap_or_else(|e| panic!("split {i}: {e:?}"));
    }
    last_cp
}

#[test]
fn cell_enter_growth_reflows_following_content_to_new_pages() {
    let mut core = load_growth_sample();
    assert_eq!(core.page_count(), 2, "픽스처 열람 쪽수");

    enter_in_growth_cell(&mut core, 8);
    assert_eq!(
        core.page_count(),
        3,
        "Enter 8회: 자란 표가 후행 문단·RowBreak 표를 다음 쪽으로 밀어야 함"
    );

    // 렌더 트리가 쪽마다 실제로 만들어지고, 표 노드가 쪽 밖으로 벗어나지 않는다.
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render p{}: {e:?}", page + 1));
        let page_h = tree.root.bbox.height;
        fn max_table_bottom(node: &rhwp::renderer::render_tree::RenderNode, out: &mut f64) {
            if let rhwp::renderer::render_tree::RenderNodeType::Table { .. } = node.node_type {
                *out = out.max(node.bbox.y + node.bbox.height);
            }
            for child in &node.children {
                max_table_bottom(child, out);
            }
        }
        let mut bottom = 0.0;
        max_table_bottom(&tree.root, &mut bottom);
        assert!(
            bottom <= page_h + 0.5,
            "p{}: 표 하단({bottom:.1})이 쪽 높이({page_h:.1})를 넘음",
            page + 1
        );
    }
}

#[test]
fn cell_enter_growth_whole_table_carries_over_and_merge_restores() {
    let mut core = load_growth_sample();
    let anchor = enter_in_growth_cell(&mut core, 20);
    assert_eq!(
        core.page_count(),
        4,
        "Enter 20회: 표 통째 이월 + 후행 콘텐츠 연쇄 이월로 4쪽"
    );

    for i in (0..20).rev() {
        core.merge_paragraph_in_cell_native(
            0,
            GROWTH_TABLE_PARA,
            GROWTH_TABLE_CTRL,
            GROWTH_CELL,
            anchor + i + 1,
        )
        .unwrap_or_else(|e| panic!("merge {i}: {e:?}"));
    }
    assert_eq!(
        core.page_count(),
        2,
        "병합 원복: 행 하한이 로드 시점 배분(baseline)이라 쪽수·형상이 되돌아와야 함"
    );
}
