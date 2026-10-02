//! [#7379] 분할 표·각주 경로에서 통과 중인 기존 대조군을 보존한다.
//!
//! 원본 정책연구 문서의 시각 비교가 90% 미만이므로 실제 실패한
//! 78개 함수는 사용자 지시에 따라 #7445에서 후속 검토한다.
//! 원문과 독립 PDF는 유지하며 제거 목록·실패·시각 증거는
//! mydocs/pr/assets/issue7445/policy_report_blocking_scope_validation.json에 있다.
//! 아래 독립 기준 설명은 후속 검토 근거이며 현재 일치 완료 주장이 아니다.
//!
//! 독립 기준은 원본 정책연구 HWPX와 한컴2024 PDF 215쪽이다.
//! 66쪽에는 머리행+본문4행(0..4), 67쪽에는 본문2행(5..6)이 있다.
//! 원본 common.height=11645HU도 첫5행의 저장 높이 합과 같다.
//! 각주77은 저장 vpos [0,1172,0]의 앞 두 줄을66쪽에 두고,
//! Part 482... 및 출처 꼬리는67쪽에 번호 반복 없이 이어야 한다.
//!
//! 원 기여자 변경은 전체 각주 사전 예약을 통째/분할 경로 모두에서 제거해
//! 표 존재 검사를 개선했지만 3+3행과 각주77 누락이 남았다. 메인터너는
//! 통째 예약을 유지하고 유효 저장 각주 경계를 분할 대기열에 연결한다.
//! 행·각주 소유 개선과 전체 페이지/시각 gate 통과는 별도다. 남은 차이와
//! 실제 전후 실행은 mydocs/pr/archives/pr_7382_review.md에 기록한다.
//!
//! 각주 없는 표 대조군은 수정 전에도 통과하며 결함 검출 증거가 아니다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx";
/// 문제의 표를 든 host 문단.
const HOST_PARA: usize = 728;

fn core() -> DocumentCore {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("정식 원본");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 해당 쪽에서 `HOST_PARA` 가 소유한 최상위 표의 `(y, height)` 를 모은다.
fn host_tables(root: &RenderNode) -> Vec<(f64, f64)> {
    fn walk(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(HOST_PARA) {
                out.push((node.bbox.y, node.bbox.height));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn host_table(root: &RenderNode) -> Option<&RenderNode> {
    table_for_para(root, HOST_PARA)
}

fn table_for_para(root: &RenderNode, para: usize) -> Option<&RenderNode> {
    if matches!(&root.node_type, RenderNodeType::Table(t) if t.para_index == Some(para)) {
        return Some(root);
    }
    root.children
        .iter()
        .find_map(|child| table_for_para(child, para))
}

fn visible_rows(table: &RenderNode) -> BTreeSet<u16> {
    table
        .children
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TableCell(cell) => Some(cell.row),
            _ => None,
        })
        .collect()
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn notes(root: &RenderNode) -> Option<&RenderNode> {
    if matches!(root.node_type, RenderNodeType::FootnoteArea) {
        return Some(root);
    }
    root.children.iter().find_map(notes)
}

fn line_top(root: &RenderNode, needle: &str) -> Option<f64> {
    if matches!(root.node_type, RenderNodeType::TextLine(_)) && text(root).contains(needle) {
        return Some(root.bbox.y);
    }
    root.children
        .iter()
        .find_map(|child| line_top(child, needle))
}

/// IR 여백 변형은 캡션 종료 예산의 알고리즘 반례이며 한컴 출력의 대용이 아니다.
/// 모든 행을 한 번씩 보존하고 캡션과 끝 바깥여백도 각주 lane 밖에 수용해야 한다.
#[test]
fn terminal_caption_margin_budget_preserves_rows_and_footer_space() {
    for (margin, gap, reduced_body_hu) in [
        (283, 850, 0),
        (30_000, 850, 0),
        (30_000, 31_000, 0),
        (32_700, 32_700, 1_500),
    ] {
        assert_terminal_caption_budget(margin, gap, reduced_body_hu, false);
    }
}

#[test]
fn terminal_caption_withholds_the_last_rowspan_unit_together() {
    assert_terminal_caption_budget(32_700, 32_700, 0, true);
}

/// 현재 조각의 예산만 부족하며 마지막 행 하나와 캡션은 새 쪽에 들어간다.
/// 이월하면서 그 행과 뒤 본문을 보존해야 한다.
#[test]
fn terminal_caption_last_single_row_defers_without_consuming_it() {
    assert_single_row_caption_deferral(rhwp::model::shape::CaptionDirection::Bottom, 0);
}

#[test]
fn opening_caption_last_single_row_reserves_its_full_object_frame() {
    assert_single_row_caption_deferral(rhwp::model::shape::CaptionDirection::Top, 0);
}

#[test]
fn paragraph_caption_frame_preserves_positive_vertical_offset() {
    assert_single_row_caption_deferral(rhwp::model::shape::CaptionDirection::Bottom, 2_250);
    assert_single_row_caption_deferral(rhwp::model::shape::CaptionDirection::Top, 2_250);
}

fn assert_single_row_caption_deferral(
    direction: rhwp::model::shape::CaptionDirection,
    vertical_offset: u32,
) {
    use rhwp::model::control::Control;
    let mut core = core();
    let mut doc = core.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[HOST_PARA].controls[0] else {
        panic!("원표");
    };
    table.cells.retain(|cell| cell.row == 6);
    for cell in &mut table.cells {
        cell.row = 0;
        for paragraph in &mut cell.paragraphs {
            paragraph
                .controls
                .retain(|control| !matches!(control, Control::Footnote(_)));
        }
    }
    table.row_count = 1;
    table.common.vertical_offset = vertical_offset;
    table.common.height = table
        .cells
        .iter()
        .map(|cell| cell.height)
        .max()
        .expect("마지막 행 높이");
    table.outer_margin_bottom = 32_700;
    let caption = table.caption.as_mut().expect("캡션");
    caption.spacing = 32_700;
    caption.direction = direction;
    let outer_top = f64::from(table.outer_margin_top) / 75.0;
    let following_para = doc.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .skip(HOST_PARA + 1)
        .find(|(_, para)| para.text.contains("42 CFR Part 482"))
        .map(|(index, _)| index)
        .expect("뒤 본문 원본");
    core.set_document(doc);
    let dump = core.dump_page_items_json(None);
    let owners: Vec<_> = dump
        .as_array()
        .expect("쪽")
        .iter()
        .filter(|page| {
            page["columns"].as_array().expect("단").iter().any(|col| {
                col["items"]
                    .as_array()
                    .expect("항목")
                    .iter()
                    .any(|item| item["paraIndex"].as_u64() == Some(HOST_PARA as u64))
            })
        })
        .collect();
    assert_eq!(
        owners.len(),
        1,
        "한 행을 누락/중복/빈 조각 없이 한 쪽에 보존"
    );
    let page = owners[0]["pageIndex"].as_u64().expect("쪽 번호") as u32;
    let tree = core.build_page_render_tree(page).expect("실제 소유 쪽");
    let table = host_table(&tree.root).expect("마지막 행");
    assert_eq!(visible_rows(table), BTreeSet::from([0]));
    let body_top = owners[0]["bodyArea"]["y"].as_f64().expect("본문 상단");
    let opening_caption_height = if matches!(direction, rhwp::model::shape::CaptionDirection::Top) {
        (1_000.0 + 32_700.0) / 75.0
    } else {
        0.0
    };
    let expected_table_top =
        body_top + outer_top + opening_caption_height + f64::from(vertical_offset) / 75.0;
    assert!(
        (table.bbox.y - expected_table_top).abs() <= 1.5,
        "중간 쪽에 과수용하지 않고 새 쪽에서 시작: table={}, body={body_top}",
        table.bbox.y
    );
    let caption_y = line_top(&tree.root, "표 23.").expect("최종 캡션");
    let body_bottom = body_top + owners[0]["bodyArea"]["height"].as_f64().expect("본문 높이");
    let occupied_end = if matches!(direction, rhwp::model::shape::CaptionDirection::Top) {
        assert!(
            caption_y >= body_top - 0.5 && caption_y < table.bbox.y,
            "위 캡션은 본문 안에서 표보다 앞에 배치: {caption_y}/{}",
            table.bbox.y
        );
        table.bbox.y + table.bbox.height + 32_700.0 / 75.0
    } else {
        caption_y + (1_000.0 + 32_700.0) / 75.0
    };
    assert!(
        occupied_end <= body_bottom + 1.0,
        "캡션/표/바깥 여백 전체 수용: {occupied_end}/{body_bottom}"
    );
    let following_owner = dump
        .as_array()
        .expect("전체 쪽")
        .iter()
        .find(|candidate| {
            candidate["columns"]
                .as_array()
                .expect("단")
                .iter()
                .any(|column| {
                    column["items"]
                        .as_array()
                        .expect("항목")
                        .iter()
                        .any(|item| item["paraIndex"].as_u64() == Some(following_para as u64))
                })
        })
        .expect("뒤 본문 소유 쪽");
    let following_page = following_owner["pageIndex"].as_u64().expect("뒤 쪽") as u32;
    assert!(
        following_page >= page,
        "뒤 본문은 캡션보다 앞 쪽으로 가지 않음"
    );
    let following_tree = core
        .build_page_render_tree(following_page)
        .expect("뒤 본문 출력");
    let following_y = line_top(&following_tree.root, "○ 42 CFR Part 482").expect("뒤 본문 보존");
    if following_page == page {
        assert!(
            following_y >= occupied_end - 1.5,
            "표/캡션 종료 뒤 본문 비충돌: {following_y}/{occupied_end}"
        );
    }
}

fn assert_terminal_caption_budget(
    margin: i16,
    gap: i16,
    reduced_body_hu: u32,
    protect_terminal_rows: bool,
) {
    let mut core = core();
    let mut doc = core.document().clone();
    let rhwp::model::control::Control::Table(table) =
        &mut doc.sections[0].paragraphs[HOST_PARA].controls[0]
    else {
        panic!("표23")
    };
    if protect_terminal_rows {
        // 이 반례는 새 쪽에 들어가는 통째 캡션 유닛만 분리한다.
        // 원표의 각주 여섯 개와 키운 캡션은 한 새 쪽에 함께 들어가지 않으므로
        // 각주 소유는 별도의 실제 원본 검사로 확인한다.
        for cell in &mut table.cells {
            for paragraph in &mut cell.paragraphs {
                paragraph.controls.retain(|control| {
                    !matches!(control, rhwp::model::control::Control::Footnote(_))
                });
            }
        }
        // 마지막 행 병합 셀은 두 행의 텍스트를 모두 포함하는 분할 유닛 하나다.
        let lower = table
            .cells
            .iter()
            .position(|cell| cell.row == 6 && cell.col == 0)
            .expect("끝행 첫 셀");
        let tail = table.cells.remove(lower);
        let upper = table
            .cells
            .iter_mut()
            .find(|cell| cell.row == 5 && cell.col == 0)
            .expect("끝행 보호 블록 시작");
        upper.row_span = 2;
        upper.height += tail.height;
        upper.paragraphs.extend(tail.paragraphs);
    }
    table.outer_margin_bottom = margin;
    table.caption.as_mut().expect("아래 캡션").spacing = gap;
    doc.sections[0].section_def.page_def.margin_bottom += reduced_body_hu;
    core.set_document(doc);
    let mut rows = Vec::new();
    let mut caption_count = 0;
    // 물리 예산을 줄이면 앞 본문 문단도 이동할 수 있다.
    // 표의 실제 소유 쪽을 찾는다. 합성 입력에는 기준 PDF 페이지 번호가 없다.
    let owners: Vec<u32> = core
        .dump_page_items_json(None)
        .as_array()
        .expect("물리 페이지")
        .iter()
        .filter(|page| {
            page["columns"].as_array().expect("단").iter().any(|col| {
                col["items"]
                    .as_array()
                    .expect("항목")
                    .iter()
                    .any(|item| item["paraIndex"].as_u64() == Some(HOST_PARA as u64))
            })
        })
        .map(|page| page["pageIndex"].as_u64().expect("페이지 번호") as u32)
        .collect();
    for page in owners {
        let tree = core.build_page_render_tree(page).expect("변형 경계 쪽");
        if let Some(table) = host_table(&tree.root) {
            let owned_rows = visible_rows(table);
            if protect_terminal_rows && (owned_rows.contains(&5) || owned_rows.contains(&6)) {
                assert!(
                    owned_rows.contains(&5) && owned_rows.contains(&6),
                    "캡션 예산 때문에 마지막 rowspan 소유 유닛을 절단하지 않음: {owned_rows:?}"
                );
            }
            rows.extend(owned_rows.iter().copied());
            if reduced_body_hu > 0 && !owned_rows.contains(&6) {
                let dump = core.dump_page_items_json(Some(page));
                let column = &dump[0]["columns"][0];
                if column["itemCount"].as_u64() == Some(1) {
                    let body_y = dump[0]["bodyArea"]["y"].as_f64().expect("본문 원점");
                    let reserved = column["usedHeight"].as_f64().expect("예약 높이");
                    let painted_end = table.bbox.y + table.bbox.height - body_y;
                    // 종료 전 조각은 수용한 행만 소유한다.
                    // 캡션과 종료 아래 여백은 아직 소비하지 않았다.
                    assert!((reserved - painted_end).abs() <= 0.5,
                        "중간 조각 내용/물리 공간: page={}, rows={owned_rows:?}, reserved={reserved}, painted_end={painted_end}", page + 1);
                }
            }
        }
        if let Some(caption_y) = line_top(&tree.root, "표 23.") {
            caption_count += 1;
            let body = tree
                .root
                .children
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
                .expect("본문 영역");
            let boundary =
                notes(&tree.root).map_or(body.bbox.y + body.bbox.height, |area| area.bbox.y);
            let occupied_end = caption_y + 1000.0 / 75.0 + f64::from(margin) / 75.0;
            assert!(
                occupied_end <= boundary + 1.0,
                "margin={margin}, page={}, caption end={occupied_end}, footer={boundary}",
                page + 1
            );
        }
    }
    assert_eq!(rows, (0..7).collect::<Vec<_>>(), "행 소유 margin={margin}");
    assert_eq!(caption_count, 1, "캡션 중복/누락 margin={margin}");
}

/// 각주116 하나만 footer에 남겨 용량과 같은 행 안의 marker 소유를 분리한다.
/// 다른 note는 빈 미주로 바꾸되 원래 extended-control 슬롯/저장 줄은 보존한다.
/// 원본 한컴 출력이 아닌 합성 계약이며, 실제 최종 표의 marker와 footer를 대조한다.
#[test]
fn intra_row_cut_does_not_publish_a_later_line_footnote() {
    use rhwp::model::{control::Control, footnote::Endnote};
    fn has_marker(node: &RenderNode, number: u16) -> bool {
        matches!(&node.node_type, RenderNodeType::FootnoteMarker(marker) if marker.number == number)
            || node.children.iter().any(|child| has_marker(child, number))
    }
    let mut core = core();
    let mut doc = core.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[885].controls[0] else {
        panic!("표25")
    };
    for cell in &mut table.cells {
        for para in &mut cell.paragraphs {
            for control in &mut para.controls {
                if let Control::Footnote(note) = control {
                    if note.number != 116 {
                        *control = Control::Endnote(Box::new(Endnote {
                            number: note.number,
                            before_decoration_letter: note.before_decoration_letter,
                            after_decoration_letter: note.after_decoration_letter,
                            number_shape: note.number_shape,
                            instance_id: note.instance_id,
                            list_header_property: note.list_header_property,
                            decoration_is_user_char: note.decoration_is_user_char,
                            paragraphs: Vec::new(),
                        }));
                    }
                }
            }
        }
    }
    core.set_document(doc);
    let mut saw_prefix = false;
    let mut marker_pages = 0;
    let mut footer_pages = 0;
    for page in 73..84 {
        let tree = core.build_page_render_tree(page).expect("표25 주변 쪽");
        let marker = table_for_para(&tree.root, 885).is_some_and(|table| has_marker(table, 116));
        let footer = notes(&tree.root).is_some_and(|area| text(area).contains("116)"));
        if let Some(table) = table_for_para(&tree.root, 885) {
            let rows = visible_rows(table);
            if rows.contains(&3) && !rows.contains(&4) {
                assert!(!marker, "분할 행의 뒤쪽 표시가 앞 조각에 중복됨");
                assert!(
                    !footer,
                    "같은 행의 아직 표시되지 않은116 각주가 앞 쪽에 등록됨"
                );
                saw_prefix = true;
            }
        }
        if marker {
            marker_pages += 1;
        }
        if footer {
            assert!(
                marker,
                "단일 작은 각주의 몸통은 marker를 출력한 쪽에 소속되어야 함"
            );
            footer_pages += 1;
        }
    }
    assert!(saw_prefix, "같은 행 안에서 끝난 첫 조각을 실행해야 함");
    assert_eq!(marker_pages, 1, "marker 누락/중복 금지");
    assert_eq!(footer_pages, 1, "footer 누락/중복 금지");
}

/// 반례 대조군 — 표 안 각주가 **없는** 같은 형상의 표는 종전대로 두 쪽에 걸쳐 쪼개진다.
///
/// 문단 0.866 은 host·`RowBreak`·`treat_as_char`·`wrap`·`vert`/`horz`·행 수가 위 표와 같고
/// **표 안 각주만 0건**이다. 이 수정이 각주 없는 표의 분할을 건드리지 않았음을 잠근다.
///
/// 쪽 번호는 이 수정으로 앞쪽이 줄면서 밀리므로(수정 전 77/78 · 수정 후 76/77) **절대
/// 번호로 고정하지 않는다.** 그래서 이 시험은 수정 전에도 통과한다 — 결함 검출 증거가
/// 아니라 회귀 잠금이다.
#[test]
fn footnote_free_rowbreak_table_keeps_splitting() {
    const FOOTNOTE_FREE_HOST: usize = 866;
    fn count_on(core: &DocumentCore, page: u32, para: usize) -> usize {
        fn walk(node: &RenderNode, para: usize, out: &mut usize) {
            if let RenderNodeType::Table(table) = &node.node_type {
                if table.para_index == Some(para) {
                    *out += 1;
                }
            }
            for child in &node.children {
                walk(child, para, out);
            }
        }
        let Ok(tree) = core.build_page_render_tree(page) else {
            return 0;
        };
        let mut n = 0;
        walk(&tree.root, para, &mut n);
        n
    }
    let core = core();
    // 70..85쪽 구간에서 이 표가 나타나는 쪽을 모은다(번호 고정 없이).
    let pages: Vec<u32> = (70..85)
        .filter(|&p| count_on(&core, p, FOOTNOTE_FREE_HOST) > 0)
        .collect();
    assert_eq!(
        pages.len(),
        2,
        "표 안 각주가 없는 표(문단 {FOOTNOTE_FREE_HOST})가 두 쪽에 걸쳐 쪼개지지 않는다. 실제 쪽: {pages:?}"
    );
    assert_eq!(
        pages[1],
        pages[0] + 1,
        "두 조각이 연속한 쪽에 있지 않다: {pages:?}"
    );
}

/// 개수가 적다는 이유로 각주 영역을 표 아래 남은 공간보다 크게 수용하지 않는다.
/// 원본 marker/저장 줄을 유지한 합성 IR이며 한컴 출력의 대용은 아니다.
#[test]
fn small_note_queue_reserves_the_actual_painted_footnote_area() {
    use rhwp::model::{control::Control, footnote::Endnote};
    let mut core = core();
    let mut doc = core.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[885].controls[0] else {
        panic!("표25")
    };
    for cell in &mut table.cells {
        for para in &mut cell.paragraphs {
            for control in &mut para.controls {
                if let Control::Footnote(note) = control {
                    if ![107, 108].contains(&note.number) {
                        *control = Control::Endnote(Box::new(Endnote {
                            number: note.number,
                            before_decoration_letter: note.before_decoration_letter,
                            after_decoration_letter: note.after_decoration_letter,
                            number_shape: note.number_shape,
                            instance_id: note.instance_id,
                            list_header_property: note.list_header_property,
                            decoration_is_user_char: note.decoration_is_user_char,
                            paragraphs: Vec::new(),
                        }));
                    }
                }
            }
        }
    }
    core.set_document(doc);
    let mut fragments = 0;
    let mut published = [0usize; 2];
    let mut following_lines = 0;
    fn check_following_lines(node: &RenderNode, top: f64, checked: &mut usize) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.para_index.is_some_and(|pi| (886..=889).contains(&pi)) {
                *checked += 1;
                assert!(
                    node.bbox.y + node.bbox.height <= top + 0.5,
                    "terminal뒤본문도같은각주예약소비: line={:?}, 각주위={top}",
                    node.bbox
                );
            }
        }
        for child in &node.children {
            check_following_lines(child, top, checked);
        }
    }
    for page in 73..84 {
        let tree = core.build_page_render_tree(page).expect("표 주변");
        if let Some(area) = notes(&tree.root) {
            if let Some(body_node) = tree
                .root
                .children
                .iter()
                .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
            {
                check_following_lines(body_node, area.bbox.y, &mut following_lines);
            }
            let body = text(area);
            for (index, number) in [107, 108].iter().enumerate() {
                if body.contains(&format!("{number})")) {
                    published[index] += 1;
                }
            }
            if let Some(table) = table_for_para(&tree.root, 885) {
                fragments += 1;
                assert!(
                    table.bbox.y + table.bbox.height <= area.bbox.y + 0.5,
                    "각주 개수로 물리 충돌을 숨기지 않음: page={}, 표끝={}, 각주위={}",
                    page + 1,
                    table.bbox.y + table.bbox.height,
                    area.bbox.y
                );
            }
        }
    }
    assert!(fragments > 0, "표/각주 공동 소유 쪽을 실제 실행");
    assert!(following_lines > 0, "각주 공동 소유 후속 본문을 실제 실행");
    assert_eq!(published, [1, 1], "각주 몸통 누락/중복 금지");
}

/// 원본 note240의 두 번째0은 PDF178/179의 실제 물리 각주 경계다.
#[test]
fn body_note_repeated_page_top_survives_hwpx_parser() {
    use rhwp::model::control::Control;
    let core = core();
    let Control::Footnote(note) = &core.document().sections[0].paragraphs[1865].controls[0] else {
        panic!("원본 본문 각주");
    };
    assert_eq!(note.number, 240);
    assert_eq!(
        note.paragraphs[0]
            .line_segs
            .iter()
            .map(|s| s.vertical_pos)
            .collect::<Vec<_>>(),
        vec![0, 0, 1172]
    );
}

/// 번호 없는 이월 꼬리에도 한컴의 해당 물리 쪽 각주 구분선은 남는다.
fn assert_continued_body_note_separator(core: DocumentCore) {
    for (page, expected_y) in [(30, 1018.725), (31, 1018.725)] {
        let tree = core.build_page_render_tree(page).expect("실제 각주 쪽");
        let area = notes(&tree.root).expect("각주 영역");
        let lines: Vec<_> = area
            .children
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::Line(line) => Some(line),
                _ => None,
            })
            .collect();
        assert_eq!(lines.len(), 1, "물리{}쪽 구분선 누락·중복 금지", page + 1);
        let line = lines[0];
        for (actual, expected) in [
            (line.x1, 94.509),
            (line.x2, 283.528),
            (line.y1, expected_y),
            (line.y2, expected_y),
        ] {
            assert!(
                (actual - expected).abs() <= 1.5,
                "구분선 좌표{actual} vs 독립PDF{expected}"
            );
        }
        let note = text(area);
        if page == 30 {
            assert!(note.contains("30)"));
        } else {
            assert!(
                !note.contains("30)") && note.contains("Transplantationszentren"),
                "번호 없는 꼬리 보존"
            );
            let y = line_top(area, "Transplantationszentren").expect("꼬리 줄");
            assert!((y - 1027.569).abs() <= 1.5, "꼬리 위치{y}");
        }
    }
    // 꼬리와 정상 각주가 함께 있어도 페이지당 구분선을 한 번만 칠한다.
    let tree = core
        .build_page_render_tree(178)
        .expect("각주240 꼬리와241/242");
    let area = notes(&tree.root).expect("뒤 각주 영역");
    assert_eq!(
        area.children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Line(_)))
            .count(),
        1
    );
}

/// 한컴 PDF121의 표시와 각주159/160은 같은 쪽에 있고122에는161만 있다.
/// 문단1297의 앞7줄·뒤3줄 소유와 각주의 등록 시점을 구분한다.
fn assert_stored_body_multi_note_owner(core: DocumentCore) {
    let first = core.build_page_render_tree(120).expect("표시와 각주121쪽");
    let next = core.build_page_render_tree(121).expect("본문 꼬리122쪽");
    let area = notes(&first.root).expect("121쪽 각주");
    let following = notes(&next.root).expect("122쪽 각주");
    for (needle, expected) in [("159)", 1011.728597), ("160)", 1027.568604)] {
        let y = line_top(area, needle).expect("독립 PDF의 표시 쪽 각주");
        assert!(
            (y - expected).abs() <= 1.5,
            "각주{needle} 원점{y} vs 독립 PDF{expected}"
        );
    }
    assert!(
        !text(following).contains("160)"),
        "각주160 꼬리 쪽 중복·잘못된 소유 금지"
    );
    assert!(text(following).contains("161)"), "후행 정상 각주 보존");
    let y = line_top(&first.root, "Royal Decree 2070/1999").expect("실제 표시 첫 줄");
    assert!((y - 803.012614).abs() <= 1.5, "각주 표시의 본문 원점{y}");
    let y = line_top(&next.root, "야 함이 조건으로 추가됨.(Article 11)").expect("실제 뒤 본문");
    assert!((y - 136.421071).abs() <= 1.5, "다음 쪽 본문{y}");
    assert_eq!(core.page_count(), 215);
}

fn assert_saved_caption_table_reference(extra_offset: u32, hwpx: bool) {
    use rhwp::model::control::Control;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(if hwpx {
        SAMPLE.to_owned()
    } else {
        SAMPLE.replace(".hwpx", ".hwp")
    });
    let bytes = std::fs::read(path).expect("원본 HWP");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWP 로드");
    let mut doc = core.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[962].controls[0] else {
        panic!("원본 표27");
    };
    table.common.vertical_offset += extra_offset;
    core.set_document(doc);
    let first = core.build_page_render_tree(89).expect("캡션과 첫 조각");
    let next = core.build_page_render_tree(90).expect("이어받는 끝 행");
    let table = table_for_para(&first.root, 962).expect("첫 표 조각");
    let expected_top = 718.094727 + extra_offset as f64 * 96.0 / 7200.0;
    assert!(
        (table.bbox.y - expected_top).abs() <= 1.5,
        "문단 기준 표 상단: {} vs {expected_top}",
        table.bbox.y
    );
    if extra_offset == 0 {
        assert!(
            (table.bbox.y + table.bbox.height - 995.710693).abs() <= 1.5,
            "첫 조각 하단: {}",
            table.bbox.y + table.bbox.height
        );
    }
    let area = notes(&first.root).expect("첫 쪽 기존 각주 영역");
    assert!(
        table.bbox.y + table.bbox.height <= area.bbox.y + 0.5,
        "표가 실제 각주 영역을 침범하면 안 됨"
    );
    let caption = line_top(&first.root, "표 27.").expect("첫 쪽 캡션");
    assert!((caption - 696.421061).abs() <= 1.5, "캡션 원점: {caption}");
    assert!(
        caption + 13.333333 <= table.bbox.y + 0.5,
        "캡션 줄 상자가 표 괘선과 겹치면 안 됨"
    );
    assert!(visible_rows(table).contains(&5), "첫 쪽 관계 행 소유");
    let tail = table_for_para(&next.root, 962).expect("이어받기 표");
    assert_eq!(
        visible_rows(tail),
        BTreeSet::from([6]),
        "끝 행만 한 번 소비"
    );
    assert!(line_top(&next.root, "표 27.").is_none(), "캡션 중복 없음");
}

fn assert_caption_note_owner(hwpx: bool, page: u32, number: u16, expected_y: f64) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(if hwpx {
        SAMPLE.to_owned()
    } else {
        SAMPLE.replace(".hwpx", ".hwp")
    });
    let bytes = std::fs::read(path).expect("정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 215, "독립 기준 PDF 쪽 수");
    let needle = format!("{number})");
    for index in [page - 2, page - 1, page] {
        let tree = core
            .build_page_render_tree(index)
            .expect("각주 소유 인접 쪽");
        fn note_line<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
            if matches!(&node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(needle)
            {
                return Some(node);
            }
            node.children
                .iter()
                .find_map(|child| note_line(child, needle))
        }
        let found = notes(&tree.root).and_then(|area| note_line(area, &needle));
        if index == page - 1 {
            let line = found.expect("분할 표 캡션 각주가 끝 조각 쪽에 있어야 함");
            // PDF는 가시 글자의 상단이고 렌더 트리는 줄 상자다. 동일 값으로 비교하지 않는다.
            // 기준 글자 상단이 실제 해당 각주 줄 상자에 속하는지와 인접 쪽 소유를 확인한다.
            assert!(
                line.bbox.y - 0.5 <= expected_y
                    && expected_y <= line.bbox.y + line.bbox.height + 0.5,
                "각주{number} 기준 글자 상단 {expected_y}는 실제 줄 상자 {}..{}에 속해야 함",
                line.bbox.y,
                line.bbox.y + line.bbox.height
            );
        } else {
            assert!(found.is_none(), "각주{number} 인접 쪽 중복/잘못된 소유");
        }
    }
}

fn original_picture_wrapper_core(hwpx: bool) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(if hwpx {
        SAMPLE.to_owned()
    } else {
        SAMPLE.replace(".hwpx", ".hwp")
    });
    DocumentCore::from_bytes(&std::fs::read(path).expect("정식 원본")).expect("문서 로드")
}

fn find_picture(root: &RenderNode) -> Option<&RenderNode> {
    if matches!(root.node_type, RenderNodeType::Image(_)) {
        Some(root)
    } else {
        root.children.iter().find_map(find_picture)
    }
}

fn assert_empty_opening_picture_fragment(hwpx: bool) {
    let core = original_picture_wrapper_core(hwpx);
    assert_eq!(core.page_count(), 215, "독립 기준 쪽 수");
    let page = core.build_page_render_tree(11).expect("12쪽");
    let table = table_for_para(&page.root, 250).expect("그림 앞의 빈 첫 물리 조각");
    // 두 셀의 괘선만 표시한 독립 한컴 PDF의 상단/하단이다.
    assert!(
        (table.bbox.y - 820.062663).abs() <= 1.5,
        "빈 조각 상단 {}",
        table.bbox.y
    );
    assert!(
        (table.bbox.y + table.bbox.height - 1002.263997).abs() <= 1.5,
        "빈 조각 하단 {}",
        table.bbox.y + table.bbox.height
    );
    assert!(
        find_picture(table).is_none(),
        "그림 유닛은 첫 빈 조각에서 소비하지 않음"
    );
    assert!(
        !text(table).contains("그림 8."),
        "캡션은 이어받는 쪽의 소유"
    );
    let area = notes(&page.root).expect("기존 각주7");
    assert!(text(area).contains("7)"));
    assert!(
        !text(area).contains("8)"),
        "이어받는 캡션 각주를 앞쪽에 등록하지 않음"
    );
}

fn assert_picture_fragment_remainder(hwpx: bool) {
    let core = original_picture_wrapper_core(hwpx);
    assert_eq!(core.page_count(), 215, "독립 기준 쪽 수");
    let page = core.build_page_render_tree(12).expect("13쪽");
    let table = table_for_para(&page.root, 250).expect("그림과 캡션의 이어받기 조각");
    let image = find_picture(table).expect("그림8");
    // 원본 PDF의 그림 외곽과 괘선 대조군의 동일 배치다.
    assert!(
        (image.bbox.y - 88.702637).abs() <= 1.5,
        "그림 상단 {}",
        image.bbox.y
    );
    assert!(
        (image.bbox.y + image.bbox.height - 325.403971).abs() <= 1.5,
        "그림 하단 {}",
        image.bbox.y + image.bbox.height
    );
    assert!(
        (table.bbox.y + table.bbox.height - 344.422689).abs() <= 1.5,
        "캡션 포함 표 하단 {}",
        table.bbox.y + table.bbox.height
    );
    assert!(
        text(table).contains("그림 8. 장기매매 유형"),
        "이어받는 캡션 보존"
    );
    let caption_top = line_top(table, "그림 8.").expect("그림8 캡션 줄");
    // 가시 글자 상단329.541056px는 저장 논리 줄의 내부에 있다.
    assert!(
        caption_top <= 329.541056 + 0.5 && 329.541056 <= caption_top + 1000.0 / 75.0 + 0.5,
        "독립 캡션 글자 상단과 실제 줄의 포함 관계 {caption_top}"
    );
    let next = line_top(&page.root, "2. 미국").expect("그림 뒤 본문");
    // 저장 줄 vpos23902HU와 본문 원점6239HU로 정한 논리 줄 상단이다.
    let expected = (6239.0 + 23902.0) * 96.0 / 7200.0;
    assert!(
        (next - expected).abs() <= 1.5,
        "뒤 본문 상단 {next}, 독립 저장 줄 {expected}"
    );
    let area = notes(&page.root).expect("13쪽 각주");
    for marker in ["8)", "9)", "10)"] {
        assert!(text(area).contains(marker), "각주 {marker}의 물리 쪽 소유");
    }
    assert!(!text(area).contains("7)"), "앞쪽 각주 중복 없음");
}

fn assert_interior_control_picture_table_anchor(hwpx: bool) {
    let core = original_picture_wrapper_core(hwpx);
    let host = &core.document().sections[0].paragraphs[246];
    assert_eq!(
        host.control_text_positions(),
        [207],
        "실제 원문 컨트롤 위치"
    );
    assert_eq!(host.line_segs[3].text_start, 171, "컨트롤 소유 줄 시작");
    assert_eq!(host.line_segs[4].text_start, 230, "다음 줄 시작");
    let tree = core.build_page_render_tree(11).expect("12쪽 그림7");
    let table = table_for_para(&tree.root, 246).expect("그림7 표");
    let image = find_picture(table).expect("그림7");
    // 원본의 원시 컨트롤207은 저장 줄171..230에 속한다. 줄vpos12000HU와
    // 개체offset3618HU·위여백283HU가 독립 PDF의 표/그림 상단을 결정한다.
    let top = (6239.0 + 12000.0 + 3618.0 + 283.0) / 75.0;
    assert!(
        (table.bbox.y - top).abs() <= 1.5,
        "컨트롤 소유 줄의 표 상단 {} vs {top}",
        table.bbox.y
    );
    assert!(
        (image.bbox.y - 296.794667).abs() <= 1.5,
        "독립 PDF 그림7 상단 {}",
        image.bbox.y
    );
    let next = line_top(&tree.root, "질적으로 매매가 이루어짐").or_else(|| {
        fn para_line(node: &RenderNode) -> Option<f64> {
            if matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(247)) {
                return Some(node.bbox.y);
            }
            node.children.iter().find_map(para_line)
        }
        para_line(&tree.root)
    }).expect("그림7 뒤 문단247");
    assert!(
        (next - (6239.0 + 34718.0) / 75.0).abs() <= 1.5,
        "다음 문단 상단 {next}"
    );
}

fn assert_square_sibling_table_outer_frames(hwpx: bool) {
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(259)) {
            out.push(node);
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let core = original_picture_wrapper_core(hwpx);
    assert_eq!(core.page_count(), 215, "원본 독립 PDF 쪽 수");
    let page = core.build_page_render_tree(12).expect("13쪽");
    let mut tables = Vec::new();
    collect(&page.root, &mut tables);
    assert_eq!(tables.len(), 2, "두 어울림 형제 표의 같은 쪽 소유");
    // 네 셀의 NONE 괘선만 SOLID로 바꾼 한컴 PDF의 실제 바깥 프레임이다.
    // 원본과 대조군은 215쪽 전체 텍스트·그림 bbox가 정확히 같다.
    for (table, x, y) in [
        (tables[0], 98.346670, 625.395996),
        (tables[1], 410.338664, 620.921346),
    ] {
        assert!(
            (table.bbox.x - x).abs() < 0.8,
            "바깥 왼쪽 여백 소유: {} != {x}",
            table.bbox.x
        );
        assert!(
            (table.bbox.y - y).abs() < 0.8,
            "바깥 위여백과 호스트 오프셋 소유: {} != {y}",
            table.bbox.y
        );
    }
    // 원본 HU 프레임:본문6239 + 호스트40102 + 개체오프셋/위여백
    // + 첫행(common.height-캡션행1282) + 캡션 셀 위패딩141이다.
    // PDF 가시 glyph 상단897.861/880.101과 논리 줄 원점을 동일값으로 비교하지 않는다.
    for (table, caption, logical_top) in [
        (
            tables[0],
            "표 2. OPTN",
            (6239.0 + 40102.0 + 334.0 + 283.0 + 21531.0 - 1282.0 + 141.0) / 75.0,
        ),
        (
            tables[1],
            "그림 9. OPTN",
            (6239.0 + 40102.0 + 283.0 + 20525.0 - 1282.0 + 141.0) / 75.0,
        ),
    ] {
        let y = line_top(table, caption).expect("원래 캡션 소유");
        assert!(
            (y - logical_top).abs() < 0.1,
            "독립 저장 HU 캡션 논리 줄: {y}/{logical_top}"
        );
        assert_eq!(text(table).matches(caption).count(), 1, "캡션 무중복");
    }
}

fn square_sibling_frames(root: &RenderNode) -> Vec<(f64, f64)> {
    fn collect(node: &RenderNode, frames: &mut Vec<(f64, f64)>) {
        if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(259)) {
            frames.push((node.bbox.x, node.bbox.y));
        }
        for child in &node.children {
            collect(child, frames);
        }
    }
    let mut frames = Vec::new();
    collect(root, &mut frames);
    frames
}

fn assert_empty_picture_table_closed_outer_frame(hwpx: bool) {
    let core = original_picture_wrapper_core(hwpx);
    assert_eq!(core.page_count(), 215, "독립 PDF 쪽 수");
    let page = core.build_page_render_tree(10).expect("11쪽 그림6");
    let table = table_for_para(&page.root, 240).expect("원래 그림6 표 소유");
    let image = find_picture(table).expect("원래 그림6");
    // 원본 호스트21265+오프셋319+위여백283은 논리 표 상단이다.
    // 그림의 가시 외곽은 같은 원본 한컴 PDF에서 독립적으로 읽었다.
    let table_top = (6239.0 + 21265.0 + 319.0 + 283.0) / 75.0;
    assert!(
        (table.bbox.y - table_top).abs() <= 0.5,
        "표 원점 {}/{table_top}",
        table.bbox.y
    );
    assert!(
        (table.bbox.height - 22400.0 / 75.0).abs() <= 0.1,
        "실제 전체 표 높이 {}",
        table.bbox.height
    );
    assert!(
        (image.bbox.y - 376.229329).abs() <= 1.5,
        "독립 그림 상단 {}",
        image.bbox.y
    );
    assert!(
        (image.bbox.y + image.bbox.height - 653.685303).abs() <= 1.5,
        "독립 그림 하단 {}",
        image.bbox.y + image.bbox.height
    );
    let caption = line_top(table, "그림 6.").expect("캡션 한 번 보존");
    assert!(
        (caption - 658.0526).abs() <= 1.5,
        "독립 캡션 상단 {caption}"
    );
    assert_eq!(text(table).matches("그림 6.").count(), 1, "캡션 무중복");
    let heading = line_top(&page.root, "다. 장기 매매 현황").expect("뒤 제목");
    assert!(
        (heading - (6239.0 + 46550.0) / 75.0).abs() <= 0.5,
        "독립 저장 줄 제목 {heading}"
    );
    let body = line_top(&page.root, "장기거래는 인간의 존엄").expect("뒤 본문");
    assert!((body - 730.5011).abs() <= 1.5, "독립 뒤 본문 {body}");
    assert!(text(notes(&page.root).expect("같은 쪽 각주6")).contains("6)"));
}

/// 수동 메타데이터 대조군은 한컴 출력의 일치 증거가 아니다.
/// 저장 상자의 증거를 없애면 앞 본문의 실제 줄 끝에서 흐름 배치해야 한다.
#[test]
fn unproven_empty_picture_table_frame_keeps_measured_flow_origin() {
    use rhwp::model::{control::Control, paragraph::LineSeg};
    for variant in 0..4 {
        let mut core = core();
        let mut doc = core.document().clone();
        match variant {
            0 => {
                doc.sections[0].paragraphs[240].line_segs[0].tag |=
                    LineSeg::TAG_IMPLEMENTATION_PROPERTY
            }
            1 => {
                doc.sections[0].paragraphs[241].line_segs[0].tag |=
                    LineSeg::TAG_IMPLEMENTATION_PROPERTY
            }
            2 => doc.sections[0].paragraphs[241].line_segs[0].vertical_pos += 75,
            3 => {
                let Control::Table(table) = &mut doc.sections[0].paragraphs[240].controls[0] else {
                    panic!("원래 그림6 표");
                };
                table.common.height -= 75;
            }
            _ => unreachable!(),
        }
        core.set_document(doc);
        let page = core.build_page_render_tree(10).expect("반례 실제 쪽");
        let table = table_for_para(&page.root, 240).expect("일반 흐름 표 보존");
        let previous = line_top(&page.root, "에는 생존 간 이식이").expect("직전 실제 글줄");
        // 직전 줄1000HU와 뒤 간격1000HU를 소비한 흐름에 개체 오프셋319HU를 더한다.
        // 증거가 없는 위여백 원점을 저장 상자로 승격하지 않는다.
        let flow_top = previous + (1000.0 + 1000.0 + 319.0) / 75.0;
        assert!(
            (table.bbox.y - flow_top).abs() <= 0.1,
            "반례{variant}: 일반 원점{}/{flow_top}",
            table.bbox.y
        );
        assert!(find_picture(table).is_some(), "그림 유닛 누락 없음");
        assert_eq!(
            text(table).matches("그림 6.").count(),
            1,
            "캡션 유닛 중복 없음"
        );
        assert!(
            line_top(&page.root, "다. 장기 매매 현황").is_some(),
            "뒤 본문 보존"
        );
    }
}

fn assert_centered_cell_pictures_match_independent_pdf(native: bool) {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    if native {
        path.set_extension("hwp");
    }
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    for (page, para, expected) in [
        (10, 237, [88.704, 95.096029]),
        (22, 339, [151.994670, 166.378662]),
    ] {
        let tree = core.build_page_render_tree(page).unwrap();
        let table = table_for_para(&tree.root, para).expect("원본 그림을 소유한 표");
        fn images<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
            if matches!(node.node_type, RenderNodeType::Image(_)) {
                out.push(node);
            }
            for child in &node.children {
                images(child, out);
            }
        }
        let mut pictures = Vec::new();
        images(table, &mut pictures);
        pictures.sort_by(|a, b| a.bbox.x.total_cmp(&b.bbox.x));
        assert_eq!(pictures.len(), 2, "두 셀 그림의 누락·중복 없음");
        // 기대 상단은 원본 한컴2024 PDF의 그림 사각형이다.
        for (picture, y) in pictures.iter().zip(expected) {
            assert!(
                (picture.bbox.y - y).abs() < 0.5,
                "{}쪽 셀 그림의 독립 정렬 원점: {:?}, PDF {y}",
                page + 1,
                picture.bbox
            );
        }
    }
    assert_eq!(core.page_count(), 215);
}

fn assert_cell_picture_bottom_caption_has_one_owner(native: bool) {
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::CaptionControlKind;
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    if native {
        path.set_extension("hwp");
    }
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let tree = core.build_page_render_tree(22).unwrap();
    let table = table_for_para(&tree.root, 339).unwrap();
    let Control::Table(source_table) = &core.document().sections[0].paragraphs[339].controls[0]
    else {
        panic!("원본 두 그림의 표");
    };
    for (col, caption, pdf_top) in [(0, "그림 21", 498.321655), (1, "그림 22", 481.361654)] {
        let cell = table.children.iter().find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.col == col)
        }).unwrap();
        assert_eq!(
            text(cell).matches(caption).count(),
            1,
            "{caption} 캡션 단일 소유"
        );
        let lines: Vec<_> = cell.children.iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line) if line.caption_owner.is_some())
        }).collect();
        assert_eq!(lines.len(), 5, "원본 저장 캡션5줄의 누락·중복 없음");
        let Control::Picture(source_picture) =
            &source_table.cells[col as usize].paragraphs[0].controls[0]
        else {
            panic!("원본 셀 그림");
        };
        let source_caption = source_picture.caption.as_ref().unwrap();
        let original: String = source_caption
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        let rendered: String = lines.iter().map(|line| text(line)).collect();
        assert_eq!(rendered, original, "원본 캡션 전체 내용과 순서 보존");
        for (index, node) in lines.iter().enumerate() {
            let RenderNodeType::TextLine(line) = &node.node_type else {
                unreachable!()
            };
            let owner = line.caption_owner.unwrap();
            assert_eq!(
                (
                    owner.sec_idx,
                    owner.para_idx,
                    owner.control_idx,
                    owner.caption_ordinal
                ),
                (0, 339, 0, 0)
            );
            assert_eq!(owner.control_kind, CaptionControlKind::Image);
            // 독립 PDF 첫 글자 상단과 원본900HU 줄 높이·540HU 간격을 대조한다.
            assert!(
                (node.bbox.y - (pdf_top + index as f64 * 19.2)).abs() < 0.5,
                "{caption} {index}번째 줄: {:?}, PDF 시작{pdf_top}",
                node.bbox
            );
        }
    }
    assert!(
        line_top(&tree.root, "장기 유형별 이식 횟수 또한 증가 추세").is_some(),
        "뒤 본문 보존"
    );
    assert_eq!(core.page_count(), 215);
}

fn assert_figure11_closed_source_frame(hwpx: bool) {
    let core = original_picture_wrapper_core(hwpx);
    assert_eq!(core.page_count(), 215, "원본/독립 PDF 쪽 수");
    let page = core.build_page_render_tree(13).expect("그림11의 14쪽");
    let table = table_for_para(&page.root, 273).expect("원본 그림11 표");
    // 호스트45803+문단 오프셋948+바깥 위여백283은 같은 원본의 논리 표 상단이다.
    let table_top = (6239.0 + 45803.0 + 948.0 + 283.0) / 75.0;
    assert!(
        (table.bbox.y - table_top).abs() < 0.5,
        "그림11 저장 바깥 프레임 원점: {:?}, {table_top}",
        table.bbox
    );
    assert!((table.bbox.height - 17819.0 / 75.0).abs() < 0.1);
    let image = find_picture(table).expect("그림11 원본 그림");
    // 두 형식의 원본 자르기 정보를 적용한 HWPX PDF 가시 상자. HWP PDF 원시 상자는 자르기 전이다.
    assert!(
        (image.bbox.y - 711.382650).abs() < 1.5,
        "독립 가시 그림 위치: {:?}",
        image.bbox
    );
    let caption = line_top(table, "그림 11.").expect("그림11 캡션");
    assert!(
        (caption - 932.581055).abs() < 1.5,
        "독립 캡션 위치: {caption}"
    );
    assert_eq!(text(table).matches("그림 11.").count(), 1);
    assert!(text(table).contains("성인, 소아, 재이식, 다기관 이식대상자"));
    let footnotes = text(notes(&page.root).expect("같은 쪽 원본 각주"));
    assert!(
        footnotes.contains("11)") && footnotes.contains("12)"),
        "각주11/12 보존"
    );
}

/// 수동 IR 본문 예산으로 통째 수용·행 분할·첫 조각 이월을 직접 검사한다.
/// 한컴 재저장 증거와 구분하며, 원본의 닫힌 행·그림·캡션을 그대로 사용한다.
#[test]
fn figure11_closed_frame_preserves_units_across_body_budgets() {
    fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
    for hwpx in [true, false] {
        for body_height in [24000, 19500, 18000] {
            let mut core = original_picture_wrapper_core(hwpx);
            let mut doc = core.document().clone();
            let mut section = doc.sections[0].clone();
            let mut host = section.paragraphs[273].clone();
            host.line_segs[0].vertical_pos = 1000;
            let mut prefix = rhwp::model::paragraph::Paragraph::new_empty_like(&host);
            prefix.insert_text_at(0, "앞 본문");
            prefix.line_segs = host.line_segs.clone();
            let line = &mut prefix.line_segs[0];
            line.vertical_pos = 0;
            line.line_height = 1000;
            line.text_height = 1000;
            line.baseline_distance = 850;
            line.line_spacing = 0;
            line.segment_width = 45352;
            line.tag = 0x60000;
            let mut guide = section.paragraphs[274].clone();
            guide.controls.clear();
            guide.line_segs[0].vertical_pos = 20333;
            let mut tail = rhwp::model::paragraph::Paragraph::new_empty_like(&host);
            tail.insert_text_at(0, "보정34 뒤 문단");
            tail.invalidate_layout_inputs();
            section.paragraphs = vec![prefix, host, guide, tail];
            let page = &mut section.section_def.page_def;
            page.height = page.margin_top
                + page.margin_bottom
                + page.margin_header
                + page.margin_footer
                + body_height;
            doc.sections = vec![section];
            core.set_document(doc);
            let mut images = 0;
            let mut captions = 0;
            let mut tails = 0;
            let mut owners = Vec::new();
            let mut owned_rows = Vec::new();
            let mut last_table = None;
            let mut tail_position = None;
            for page in 0..core.page_count() {
                let tree = core.build_page_render_tree(page).unwrap();
                let mut nodes = Vec::new();
                walk(&tree.root, &mut nodes);
                let body = nodes
                    .iter()
                    .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
                    .unwrap();
                assert!((body.bbox.height - f64::from(body_height) / 75.0).abs() < 0.1);
                if let Some(table) = table_for_para(&tree.root, 1) {
                    owners.push(page);
                    owned_rows.push(visible_rows(table));
                    last_table = Some((page, table.bbox.y + table.bbox.height));
                    assert!(table.bbox.y >= body.bbox.y - 0.5);
                    assert!(
                        table.bbox.y + table.bbox.height <= body.bbox.y + body.bbox.height + 0.5,
                        "형식{hwpx} 예산{body_height} 쪽{page} 표 {:?}, 본문 {:?}",
                        table.bbox,
                        body.bbox
                    );
                    if page == 0 {
                        let expected = body.bbox.y + (1000.0 + 948.0 + 283.0) / 75.0;
                        assert!(
                            (table.bbox.y - expected).abs() < 0.5,
                            "첫 원본 프레임 {:?}, 기대{expected}",
                            table.bbox
                        );
                    }
                    let mut contents = Vec::new();
                    walk(table, &mut contents);
                    images += contents
                        .iter()
                        .filter(|node| matches!(&node.node_type, RenderNodeType::Image(_)))
                        .count();
                    captions += text(table).matches("그림 11.").count();
                }
                for node in nodes {
                    if matches!(node.node_type, RenderNodeType::TextLine(_))
                        && text(node).contains("보정34 뒤 문단")
                    {
                        tails += 1;
                        tail_position = Some((page, node.bbox.y));
                    }
                }
            }
            assert_eq!(images, 1, "그림 단일 소유 {hwpx}/{body_height}");
            assert_eq!(captions, 1, "캡션 단일 소유 {hwpx}/{body_height}");
            assert_eq!(tails, 1, "뒤 문단 단일 소유 {hwpx}/{body_height}");
            let (last_page, last_end) = last_table.unwrap();
            let (tail_page, tail_y) = tail_position.unwrap();
            assert!(
                tail_page > last_page || (tail_page == last_page && tail_y >= last_end - 0.5),
                "뒤 문단 점유: 표{last_page}/{last_end}, 문단{tail_page}/{tail_y}"
            );
            assert!(core.page_count() <= 3, "불필요한 빈 쪽 없음");
            if body_height == 19500 {
                assert_eq!(owners, vec![0, 1], "그림 행과 캡션 행의 실제 분할 쪽");
                assert_eq!(
                    owned_rows,
                    vec![BTreeSet::from([0]), BTreeSet::from([1])],
                    "온전한 행의 단일 소유·누락·중복 없음"
                );
            } else if body_height == 18000 {
                assert!(
                    owners[0] > 0,
                    "첫 행을 담지 못한 실제 이월 경계: 형식{hwpx}, 소유 쪽{owners:?}"
                );
            }
        }
    }
}

fn figure64_source_picture_count(node: &RenderNode) -> usize {
    usize::from(
        matches!(&node.node_type, RenderNodeType::Image(image) if image.para_index == Some(1692) && image.control_index == Some(1)),
    ) + node
        .children
        .iter()
        .map(figure64_source_picture_count)
        .sum::<usize>()
}

fn assert_figure64_next_page_source_frame(hwpx: bool) {
    fn source_picture(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Image(image) if image.para_index == Some(1692) && image.control_index == Some(1))
        {
            return Some(node);
        }
        node.children.iter().find_map(source_picture)
    }
    let core = original_picture_wrapper_core(hwpx);
    assert_eq!(core.page_count(), 215);
    let previous = core.build_page_render_tree(154).unwrap();
    assert!(
        source_picture(&previous.root).is_none(),
        "원본 그림64는155쪽본문/표와겹치지않고다음쪽이소유한다"
    );
    assert!(!text(&previous.root).contains("그림 64. 일본 평가절차"));
    assert!(text(&previous.root).contains("일본 각 병원에서 일반적으로 진행되는 절차"));
    let page = core.build_page_render_tree(155).unwrap();
    let picture = source_picture(&page.root).expect("원본156쪽단일그림64");
    assert_eq!(figure64_source_picture_count(&page.root), 1);
    let following = core.build_page_render_tree(156).unwrap();
    assert!(
        source_picture(&following.root).is_none(),
        "뒤 쪽에 그림 중복 없음"
    );
    assert!(!text(&following.root).contains("그림 64. 일본 평가절차"));
    assert!(
        (picture.bbox.y - 89.981363).abs() < 0.5,
        "독립PDF그림원점: {:?}",
        picture.bbox
    );
    let caption = line_top(&page.root, "그림 64. 일본 평가절차").expect("같은쪽캡션");
    assert!(
        (caption - 404.101033).abs() < 0.5,
        "독립PDF캡션원점: {caption}"
    );
    assert_eq!(
        text(&page.root).matches("그림 64. 일본 평가절차").count(),
        1
    );
    assert!(text(&page.root).contains("교토대병원은 생존 간 기증자의 검사 내용"));
}

/// 동일 원본 PDF120쪽 괘선으로 표 위여백의 단일 소비를 확인한다.
fn assert_terminal_table_frame_uses_outer_top_once(hwpx: bool) {
    let core = original_picture_wrapper_core(hwpx);
    assert_eq!(core.page_count(), 215);
    let page = core.build_page_render_tree(119).unwrap();
    let table = table_for_para(&page.root, 1283).expect("원본120쪽 표");
    let pdf_top = 65.208984 * 4.0 / 3.0;
    assert!(
        (table.bbox.y - pdf_top).abs() < 0.5,
        "독립 PDF 괘선 상단: {:?}, 기대{pdf_top}",
        table.bbox
    );
    assert!((table.bbox.height - 23790.0 / 75.0).abs() < 0.1);
    assert!((table.bbox.width - 41954.0 / 75.0).abs() < 0.1);
    assert_eq!(text(table).matches("문서:").count(), 1);
    assert!(text(&page.root).contains("규정하고 있음."));
    let next = core.build_page_render_tree(120).unwrap();
    assert!(text(&next.root).contains("A) 기증자가 법적으로 가능한 연령이 되어야 하고"));
    assert!(table_for_para(&next.root, 1283).is_none());
}

fn assert_captioned_empty_host_original_outer_frame(sample: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(path).expect("원본 표31");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 로드");
    assert_eq!(core.page_count(), 215);
    let page = core.build_page_render_tree(125).expect("126쪽");
    let table = table_for_para(&page.root, 1350).expect("표31");
    assert!(
        (table.bbox.y - 466.65).abs() <= 0.5,
        "표31 상단: {}",
        table.bbox.y
    );
    assert!(
        (table.bbox.height - 232.426667).abs() <= 0.5,
        "표31 높이: {}",
        table.bbox.height
    );
    let caption = line_top(&page.root, "표 31.").expect("표31 캡션");
    assert!((caption - 710.34).abs() <= 0.5, "표31 캡션: {caption}");
    let following = line_top(&page.root, "나. 장기기증 승인업무절차").expect("뒤 제목");
    assert!((following - 782.18).abs() <= 0.5, "뒤 제목: {following}");
    for index in [124, 126] {
        let other = core.build_page_render_tree(index).expect("앞뒤 쪽");
        assert!(table_for_para(&other.root, 1350).is_none(), "표31 중복");
        assert!(line_top(&other.root, "표 31.").is_none(), "캡션 중복");
    }
}

/// 원본 쪽의 문단을 독립 IR로 옮기고 본문 예산만 줄이는 분할 경계 반례다.
/// 한컴 출력의 대용이 아니며 두 행·캡션·뒤 제목의 단일 소유를 확인한다.
#[test]
fn captioned_closed_empty_host_frame_preserves_rows_and_caption_when_budget_requires_split() {
    let mut core = core();
    let mut doc = core.document().clone();
    doc.sections[0].paragraphs = doc.sections[0].paragraphs[1343..1356].to_vec();
    doc.sections.truncate(1);
    doc.sections[0].section_def.page_def.margin_bottom += 20_000;
    use rhwp::model::control::Control;
    let Control::Table(original) = &doc.sections[0].paragraphs[7].controls[0] else {
        panic!("원본 표31");
    };
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|ch| !ch.is_whitespace() && *ch != '•')
            .collect::<String>()
    };
    let mut expected_paragraphs = std::collections::BTreeMap::new();
    for cell in &original.cells {
        for (pi, para) in cell.paragraphs.iter().enumerate() {
            expected_paragraphs.insert((cell.row, cell.col, pi), normalize(&para.text));
        }
    }
    let mut actual_paragraphs = std::collections::BTreeMap::new();
    fn collect_cell_lines(
        node: &RenderNode,
        row: u16,
        col: u16,
        out: &mut std::collections::BTreeMap<(u16, u16, usize), String>,
    ) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            let pi = line.para_index.expect("셀 문단 원본 소유");
            out.entry((row, col, pi)).or_default().push_str(&text(node));
            return;
        }
        for child in &node.children {
            collect_cell_lines(child, row, col, out);
        }
    }
    let source_page = &doc.sections[0].section_def.page_def;
    let body_top = f64::from(source_page.margin_top + source_page.margin_header) / 75.0;
    let body_height = f64::from(
        source_page.height
            - source_page.margin_top
            - source_page.margin_header
            - source_page.margin_bottom
            - source_page.margin_footer,
    ) / 75.0;
    let body_bottom = body_top + body_height;
    core.set_document(doc);
    let mut row_owners = [0usize; 2];
    let mut caption_owners = Vec::new();
    let mut following_owners = Vec::new();
    let mut table_pages = Vec::new();
    let mut all_text = String::new();
    for index in 0..core.page_count() {
        let page = core.build_page_render_tree(index).expect("분할 쪽");
        all_text.push_str(&text(&page.root));
        if let Some(table) = table_for_para(&page.root, 7) {
            table_pages.push(index);
            // 행 번호가 반복되어도 원본 셀/문단 축의 모든 텍스트를 합쳐 검증한다.
            for child in &table.children {
                if let RenderNodeType::TableCell(cell) = &child.node_type {
                    collect_cell_lines(child, cell.row, cell.col, &mut actual_paragraphs);
                }
            }

            for row in visible_rows(table) {
                row_owners[usize::from(row)] += 1;
            }
            assert!(
                table.bbox.y + table.bbox.height <= body_bottom + 0.5,
                "표 물리 하단"
            );
        }
        if line_top(&page.root, "표 31.").is_some() {
            caption_owners.push(index);
        }
        if line_top(&page.root, "나. 장기기증 승인업무절차").is_some() {
            following_owners.push(index);
        }
    }
    let actual_paragraphs: std::collections::BTreeMap<_, _> = actual_paragraphs
        .into_iter()
        .map(|(key, value)| (key, normalize(&value)))
        .collect();
    assert_eq!(
        actual_paragraphs, expected_paragraphs,
        "원본15개 셀 문단의 모든 내용 보존"
    );
    assert_eq!(row_owners[0], 1, "머리행 단일 소유");
    assert!(row_owners[1] >= 1, "본문행 존재");
    // 본문행은 실제 글줄 컷으로 이어질 수 있다. 같은 행 번호의 두 조각을
    // 중복으로 세지 않고 원본 내용의 누락/중복을 직접 검사한다.
    for marker in [
        "기관윤리위원회",
        "이식 전후 평가체제",
        "설명동의 절차 및 서식",
        "감염병전문의",
        "병리진단",
        "간호체제",
        "이식대상자 이식코디네이터가 배치되어 있을 것",
        "긴급상황에 대한 24시간 체제",
        "세균검사",
        "면역억제제의 혈중농도 측정",
        "수술현미경에 의한 혈관문합",
    ] {
        assert_eq!(
            all_text.matches(marker).count(),
            1,
            "내용 단일 소유: {marker}"
        );
    }
    assert_eq!(table_pages.len(), 2, "실제 분할");
    assert_eq!(
        caption_owners,
        vec![*table_pages.last().expect("끝 조각")],
        "끝 캡션 소유"
    );
    assert_eq!(following_owners.len(), 1, "뒤 제목 단일 소유");
    assert!(following_owners[0] >= caption_owners[0], "뒤 제목의 순서");
}

fn assert_numbered_footnote_hanging_indent(sample: &str) {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).expect("원본");
    let core = DocumentCore::from_bytes(&bytes).expect("각주 원본");
    assert_eq!(core.page_count(), 215);
    let page = core.build_page_render_tree(125).expect("126쪽");
    let area = notes(&page.root).expect("각주 영역");
    let lines: Vec<_> = area
        .children
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .collect();
    for (number, count) in [(171, 4), (172, 3)] {
        let marker = format!("{number})");
        let start = lines
            .iter()
            .position(|n| text(n).starts_with(&marker))
            .expect("번호 소유");
        let first = lines[start].children.first().expect("번호 run");
        assert!((first.bbox.x - 94.56).abs() <= 0.5, "첫 줄 번호 위치");
        let source_note = core.document().sections[0].paragraphs[1349]
            .controls
            .iter()
            .find_map(|control| match control {
                rhwp::model::control::Control::Footnote(note) if note.number == number => {
                    Some(note)
                }
                _ => None,
            })
            .expect("원본 각주 내용");
        let normalize = |value: &str| {
            value
                .chars()
                .filter(|ch| !ch.is_whitespace() && !ch.is_control())
                .collect::<String>()
        };
        let actual = lines[start..start + count]
            .iter()
            .map(|line| text(line))
            .collect::<String>();
        let content = actual.strip_prefix(&marker).expect("첫 줄 번호 한 번");
        assert_eq!(
            normalize(content),
            normalize(&source_note.paragraphs[0].text),
            "각주{number} 전체 내용 보존"
        );
        for pair in lines[start..start + count].windows(2) {
            assert!(
                (pair[1].bbox.y - pair[0].bbox.y - 1172.0 / 75.0).abs() <= 0.01,
                "저장 줄 간격 보존"
            );
        }
        for line in &lines[start + 1..start + count] {
            let run = line.children.first().expect("이어지는 글줄");
            assert!(
                (run.bbox.x - 112.0).abs() <= 0.5,
                "각주{number} 내어쓰기: {}",
                run.bbox.x
            );
        }
    }
    for index in [124, 126] {
        let other = core.build_page_render_tree(index).expect("앞뒤 쪽");
        assert!(
            !text(notes(&other.root).expect("앞뒤 각주")).contains("홋카이도지역"),
            "각주171 중복"
        );
    }
}

fn assert_captioned_terminal_fragment_outer_frame(sample: &str) {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).expect("원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 표23");
    assert_eq!(core.page_count(), 215);
    let first = core.build_page_render_tree(65).expect("66쪽");
    let next = core.build_page_render_tree(66).expect("67쪽");
    assert_eq!(
        visible_rows(host_table(&first.root).expect("첫 조각")),
        (0..5).collect()
    );
    let table = host_table(&next.root).expect("끝 조각");
    assert_eq!(visible_rows(table), (5..7).collect());
    assert!(
        (table.bbox.y - 86.945312).abs() <= 0.5,
        "끝 조각 원점: {}",
        table.bbox.y
    );
    let caption = line_top(&next.root, "표 23.").expect("끝 캡션");
    assert!((caption - 156.434347).abs() <= 0.5, "캡션 원점: {caption}");
    let following = line_top(
        &next.root,
        "42 CFR Part 482 (CONDITIONS OF PARTICIPATION FOR HOSPITALS)",
    )
    .expect("뒤 제목");
    assert!(
        (following - 200.261047).abs() <= 0.5,
        "뒤 제목: {following}"
    );
    assert!(
        line_top(&first.root, "표 23.").is_none(),
        "앞 조각 캡션 중복"
    );
    let after = core.build_page_render_tree(67).expect("68쪽");
    assert!(host_table(&after.root).is_none(), "표 조각 중복");
    assert!(line_top(&after.root, "표 23.").is_none(), "캡션 중복");
    assert!(
        text(notes(&first.root).expect("66쪽 각주")).contains("77)"),
        "원래 번호 소유"
    );
    let tail = text(notes(&next.root).expect("67쪽 각주"));
    assert!(tail.contains("Part 482(CONDITIONS"), "77 꼬리 소유");
    assert!(!tail.contains("77)"), "번호 반복 금지");
}

/// 실제 셀 편집의 재조판 결과는 저장 표 프레임을 그대로 재사용하지 않는다.
/// 작은 삽입과 너비 부족 줄바꿈에서 전체 내용·캡션·물리 점유를 확인한다.
#[test]
fn actual_cell_edit_preserves_reflowed_table_payload_and_budget_hwpx() {
    assert_actual_cell_edit_table_contract(SAMPLE);
}

#[test]
fn actual_cell_edit_preserves_reflowed_table_payload_and_budget_hwp() {
    assert_actual_cell_edit_table_contract(&SAMPLE.replace(".hwpx", ".hwp"));
}

fn assert_actual_cell_edit_table_contract(sample: &str) {
    use rhwp::model::control::Control;
    use std::collections::BTreeMap;
    fn normalize(value: &str) -> String {
        value
            .chars()
            .filter(|ch| !ch.is_whitespace() && !ch.is_control() && *ch != '•')
            .collect()
    }
    fn collect(
        node: &RenderNode,
        row: u16,
        col: u16,
        out: &mut BTreeMap<(u16, u16, usize), String>,
        cell_top: f64,
        cell_bottom: f64,
    ) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            assert!(
                node.bbox.y >= cell_top - 0.5
                    && node.bbox.y + node.bbox.height <= cell_bottom + 0.5,
                "수용한 글줄이 셀 상자 안에 표시됨: {:?}, 셀{cell_top}..{cell_bottom}",
                node.bbox
            );
            out.entry((row, col, line.para_index.expect("원본 셀 문단")))
                .or_default()
                .push_str(&text(node));
            return;
        }
        for child in &node.children {
            collect(child, row, col, out, cell_top, cell_bottom);
        }
    }
    for inserted in [" ".to_owned(), "표셀편집검증".repeat(20)] {
        let bytes =
            std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample)).expect("원본");
        let mut core = DocumentCore::from_bytes(&bytes).expect("원본 표29");
        let Control::Table(original) = &core.document().sections[0].paragraphs[1136].controls[0]
        else {
            panic!("원래 표");
        };
        let prior_lines = original.cells[0].paragraphs[0].line_segs.len();
        let prior_text = original.cells[0].paragraphs[0].text.clone();
        core.insert_text_in_cell_native(0, 1136, 0, 0, 0, 0, &inserted)
            .expect("실제 셀 편집");
        let Control::Table(edited) = &core.document().sections[0].paragraphs[1136].controls[0]
        else {
            panic!("편집 표");
        };
        let paragraph = &edited.cells[0].paragraphs[0];
        assert_eq!(
            paragraph.text,
            format!("{inserted}{prior_text}"),
            "편집 명령의 실제 문자열"
        );
        assert!(!paragraph.line_segs.is_empty(), "실제 재조판 줄 생성");
        assert!(
            paragraph
                .line_segs
                .windows(2)
                .all(|pair| pair[1].vertical_pos > pair[0].vertical_pos),
            "원래 되감김 대신 연속 재조판 줄"
        );
        if inserted.len() > 1 {
            assert!(
                paragraph.line_segs.len() > prior_lines,
                "실제 너비 부족 줄바꿈"
            );
        }
        let mut expected = BTreeMap::new();
        for cell in &edited.cells {
            for (pi, para) in cell.paragraphs.iter().enumerate() {
                expected.insert((cell.row, cell.col, pi), normalize(&para.text));
            }
        }
        let mut actual: BTreeMap<_, _> = expected.keys().map(|key| (*key, String::new())).collect();
        let def = &core.document().sections[0].section_def.page_def;
        let top = f64::from(def.margin_top + def.margin_header) / 75.0;
        let bottom = f64::from(def.height - def.margin_bottom - def.margin_footer) / 75.0;
        let mut table_pages = Vec::new();
        let mut caption_pages = Vec::new();
        let mut following_pages = Vec::new();
        for index in 0..core.page_count() {
            let page = core.build_page_render_tree(index).expect("편집 뒤 실제 쪽");
            if let Some(table) = table_for_para(&page.root, 1136) {
                table_pages.push(index);
                if let Ok(root) = std::env::var("RHWP_PR7382_EDIT_EVIDENCE_DIR") {
                    let kind = if sample.ends_with(".hwpx") {
                        "hwpx"
                    } else {
                        "hwp"
                    };
                    let case = if inserted.len() == 1 {
                        "small"
                    } else {
                        "growth"
                    };
                    let dir = Path::new(&root).join(format!("{kind}-{case}"));
                    std::fs::create_dir_all(&dir).expect("output 증적 폴더");
                    let svg = core
                        .render_page_svg_with_fonts(
                            index as u32,
                            rhwp::renderer::svg::FontEmbedMode::Style,
                            &[],
                        )
                        .expect("폰트 공급 규칙을 포함한 실제 편집 상태 SVG");
                    std::fs::write(dir.join(format!("page_{:03}.svg", index + 1)), svg)
                        .expect("실제 편집 상태 증적");
                }
                let limit = notes(&page.root).map_or(bottom, |area| area.bbox.y);
                assert!(
                    table.bbox.y >= top - 0.5 && table.bbox.y + table.bbox.height <= limit + 0.5,
                    "실제 조각의 본문/각주 예산: 쪽{index}, 표{:?}, 끝{limit}",
                    table.bbox
                );
                for child in &table.children {
                    if let RenderNodeType::TableCell(cell) = &child.node_type {
                        collect(
                            child,
                            cell.row,
                            cell.col,
                            &mut actual,
                            child.bbox.y,
                            child.bbox.y + child.bbox.height,
                        );
                    }
                }
            }
            if let Some(caption_top) = line_top(&page.root, "표 29.") {
                caption_pages.push(index);
                let last_table = table_for_para(&page.root, 1136).expect("캡션의 끝 표 조각");
                assert!(
                    caption_top + 0.5 >= last_table.bbox.y + last_table.bbox.height,
                    "끝 캡션이 표 내용과 겹치지 않음"
                );
                if let Some(following_top) = line_top(&page.root, "O 미성년자") {
                    assert!(
                        following_top >= caption_top + 0.5,
                        "같은 쪽 뒤 본문이 캡션 뒤에 배치됨"
                    );
                }
            }
            if line_top(&page.root, "O 미성년자").is_some() {
                following_pages.push(index);
            }
        }
        let actual: BTreeMap<_, _> = actual
            .into_iter()
            .map(|(key, value)| (key, normalize(&value)))
            .collect();
        assert_eq!(
            actual, expected,
            "편집된 원본의 전체 셀 문단 내용 누락/중복 없음"
        );
        assert!(!table_pages.is_empty(), "원래 표 내용 보존");
        assert_eq!(
            caption_pages,
            vec![*table_pages.last().expect("끝 조각")],
            "끝 조각 캡션 한 번"
        );
        assert_eq!(following_pages.len(), 1, "뒤 본문 한 번");
        assert!(
            following_pages[0] >= caption_pages[0],
            "뒤 본문의 실제 순서"
        );
        println!("실제 셀 편집: {sample}, 삽입{}자, 전체{}쪽, 표조각{table_pages:?}, 캡션{caption_pages:?}, 뒤본문{following_pages:?}", inserted.chars().count(), core.page_count());
    }
}
