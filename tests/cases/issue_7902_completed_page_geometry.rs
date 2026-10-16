//! #7902: 전쪽90%·선택 쪽 양쪽98%를 확인한 원본의 중요 기하를 보호한다.
//!
//! 독립 Print의 image bbox·표 괘선·글자 원점을 72pt→96dpi로 환산한 기대값이다.
//! 원본/Print 해시·producer·전쪽 TSV·선정 이유·같은 영역의 직접 판독은
//! mydocs/working/task_m100_7902_stage3.md의 문서별 완료 페이지 표에 연결한다.
//! 위치로 대상을 검색하지 않고 원본 소유·내용으로 선택하며 관계 검사도 유지한다.
//! 기존 번호/overfill/nested 검사는 issue_7902_original_frame_and_content에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn load(sample: &str, compatibility_2024: bool) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(compatibility_2024);
    core
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn unique(root: &RenderNode, predicate: impl Fn(&RenderNode) -> bool) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    let selected: Vec<_> = nodes.into_iter().filter(|node| predicate(node)).collect();
    assert_eq!(
        selected.len(),
        1,
        "원본 소유·내용으로 선택한 대상의 단일 배치"
    );
    selected[0]
}

fn table(root: &RenderNode, para: usize, control: usize, rows: u16, cols: u16) -> &RenderNode {
    unique(root, |node| {
        matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(para) && t.control_index == Some(control)
                && t.row_count == rows && t.col_count == cols)
    })
}

fn image(root: &RenderNode) -> &RenderNode {
    unique(root, |node| {
        matches!(node.node_type, RenderNodeType::Image(_))
    })
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

fn compact(value: &str) -> String {
    value.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn near(actual: f64, expected: f64, description: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{description}: 실제 {actual:.6}, 독립 Print {expected:.6}, 허용±2px"
    );
}

fn bottom(node: &RenderNode) -> f64 {
    node.bbox.y + node.bbox.height
}

fn edges(node: &RenderNode, expected: [f64; 4]) {
    let b = node.bbox;
    for ((actual, expected), label) in [b.x, b.y, b.x + b.width, b.y + b.height]
        .into_iter()
        .zip(expected)
        .zip(["왼쪽", "위쪽", "오른쪽", "아래쪽"])
    {
        near(actual, expected, label);
    }
}

fn contained(child: &RenderNode, parent: &RenderNode) {
    let BoundingBox {
        x,
        y,
        width,
        height,
    } = parent.bbox;
    let c = child.bbox;
    assert!(c.width > 0.0 && c.height > 0.0, "빈 개체 상자");
    assert!(
        c.x >= x - 0.05
            && c.y >= y - 0.05
            && c.x + c.width <= x + width + 0.05
            && c.y + c.height <= y + height + 0.05,
        "개체는 물리 소유 상자 안에 있어야 한다: {c:?}, {:?}",
        parent.bbox
    );
}

fn cell_owner<'a>(
    node: &'a RenderNode,
    target: &RenderNode,
    owner: Option<&'a RenderNode>,
) -> Option<&'a RenderNode> {
    if std::ptr::eq(node, target) {
        return owner;
    }
    let owner = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        owner
    };
    node.children
        .iter()
        .find_map(|child| cell_owner(child, target, owner))
}

fn baseline<'a>(root: &'a RenderNode, needle: &str, expected: f64) -> &'a RenderNode {
    let line = unique(root, |node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
            && compact(&text(node)).contains(&compact(needle))
    });
    let mut nodes = Vec::new();
    walk(line, &mut nodes);
    let runs: Vec<_> = nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if !run.display_or_text().trim().is_empty() => {
                Some(node.bbox.y + run.baseline)
            }
            _ => None,
        })
        .collect();
    assert!(!runs.is_empty(), "가시 본문을 가진 글줄");
    for y in runs {
        near(y, expected, needle);
    }
    line
}

#[test]
fn square_lane_first_page_keeps_certification_image_and_wrapped_body() {
    let core = load(
        "samples/issue6778/156757920-animal-welfare-husbandry-guidelines.hwp",
        false,
    );
    let root = core.build_page_render_tree(0).unwrap().root;
    let frame = table(&root, 12, 0, 2, 1);
    let picture = image(frame);
    // pdf/pr7704/6778-square-2020.pdf 1쪽 인증마크 image bbox.
    edges(picture, [86.033335, 771.635986, 299.838664, 941.050618]);
    contained(picture, frame);
    assert!(compact(&text(frame)).contains("동물복지축산농장인증마크"));
    let lane = unique(&root, |node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(13) && line.line_index == Some(0))
    });
    assert!(lane.bbox.x >= frame.bbox.x + frame.bbox.width);
    assert!(lane.bbox.y >= frame.bbox.y && lane.bbox.y < bottom(frame));
    // 같은 Print에서 인증마크 옆 첫 본문 글자의 원점도 독립적으로 확인했다.
    near(lane.bbox.x, 310.391785, "Square 옆 첫 본문의 왼쪽 원점");
    baseline(
        &root,
        "국립축산과학원이 학계·산업계와 협력해 그동안",
        777.550293,
    );
}

#[test]
fn nested_fragment_page_four_keeps_all_ten_rows_inside_the_physical_cell() {
    let core = load("samples/issue7336/nested_table_fragment_cut.hwp", true);
    let root = core.build_page_render_tree(3).unwrap().root;
    let nested = table(&root, 6, 0, 10, 2);
    // pdf/issue7336/nested_table_fragment_cut-2024.pdf 4쪽 10×2 괘선.
    edges(nested, [168.869334, 546.602661, 705.379964, 990.916016]);
    contained(
        nested,
        cell_owner(&root, nested, None).expect("중첩 표의 물리 셀"),
    );
    let body = compact(&text(nested));
    for needle in ["재료비", "무형자산취득비", "회계정산비"] {
        assert_eq!(
            body.matches(needle).count(),
            1,
            "4쪽 중첩 표 내용의 단일 소유"
        );
    }
}

#[test]
fn overfill_opening_pages_keep_the_last_definition_and_next_heading_origins() {
    let core = load("samples/table_giant_cell_overfill.hwpx", true);
    let first = core.build_page_render_tree(0).unwrap().root;
    let second = core.build_page_render_tree(1).unwrap().root;
    // pdf/table_giant_cell_overfill-hwpx-2024.pdf 1쪽 마지막 정의와 2쪽 첫 제목.
    // 선언 첫 프레임의 양수 재시작이 사라졌던 경계를 좌표와 쪽/셀 소유로 함께 보호한다.
    let closing = baseline(
        &first,
        "내경을 나사 가공해서 구동 스크류와 결합하여",
        1049.413167,
    );
    let opening = baseline(&second, "3.7 구동방식", 114.594767);
    for (root, line) in [(&first, closing), (&second, opening)] {
        near(line.bbox.x, 81.076060, "정의 셀의 원본 글줄 왼쪽 원점");
        let owner = cell_owner(root, line, None).expect("정의 글줄의 물리 셀");
        assert!(
            matches!(&owner.node_type, RenderNodeType::TableCell(c) if c.row == 4 && c.col == 0)
        );
        contained(line, owner);
    }
    // 같은 검증 compat로 전쪽 공용 래칫도 복원한다. 빈 운반 줄을
    // 실제 그리기 범위로 합산하던45쪽 오탐을 정상 baseline으로 등록하지 않는다.
    let anomalies = rhwp::diagnostics::layout_anomaly::scan_document(
        &core,
        &rhwp::diagnostics::layout_anomaly::AnomalyOptions::default(),
    )
    .unwrap();
    assert_eq!(
        anomalies.off_canvas_count(),
        0,
        "완성한 원본의 용지 밖 기하"
    );
    assert_eq!(anomalies.text_overlap_count(), 0, "완성한 원본의 글자 겹침");
}

#[test]
fn stored_frame_first_page_keeps_form_title_signature_and_printed_bottom() {
    let core = load(
        "samples/issue7336/stored_frame_page_larger_rowbreak.hwpx",
        true,
    );
    let root = core.build_page_render_tree(0).unwrap().root;
    let frame = table(&root, 1, 0, 24, 8);
    // pdf/issue7336/stored_frame_page_larger_rowbreak-2024.pdf 1쪽.
    // 첫 두 행은 외곽선을 그리지 않으므로 인쇄된 왼쪽/아래 괘선과 글자 기준선을 쓴다.
    near(frame.bbox.x, 79.285332, "신청서 왼쪽 괘선");
    near(bottom(frame), 1020.323975, "신청서 마지막 괘선");
    let title = baseline(frame, "2026년 상주시 중소기업", 148.957214);
    let signature = baseline(frame, "경북테크노파크원장 귀하", 1004.479980);
    assert!(bottom(title) < signature.bbox.y);
    contained(signature, frame);
}

#[test]
fn scattered_header_pages_two_and_three_keep_the_continuation_and_next_table() {
    let core = load("samples/table_scattered_header_rowbreak.hwp", true);
    let second = core.build_page_render_tree(1).unwrap().root;
    let third = core.build_page_render_tree(2).unwrap().root;
    let first = table(&second, 6, 0, 84, 8);
    // pdf/table_scattered_header_rowbreak-hwp-2024.pdf 2쪽은 양쪽100%다.
    edges(first, [77.185333, 77.356038, 716.681315, 1081.537354]);
    let continued = table(&third, 6, 0, 84, 8);
    let next = table(&third, 9, 0, 260, 9);
    // 사용자 지적3쪽은 첫 표 종료와 후속 표 시작이라는 별도 경계도 보호한다.
    edges(continued, [77.185333, 77.356038, 717.640652, 725.126628]);
    edges(next, [77.185333, 796.568034, 717.640652, 1081.537354]);
    // 바깥 크기만 유지하면서 첫 행을 잃거나 내부 행을 이동하는 경우도 검출한다.
    let first_item = baseline(continued, "인장시험", 114.720001);
    near(first_item.bbox.x, 215.839986, "이어받은 첫 시험 항목 원점");
    let next_item = baseline(next, "절대 건조 밀도 및 흡수율", 833.439941);
    near(next_item.bbox.x, 173.599996, "후속 표 첫 시험 항목 원점");
    for frame in [first, continued, next] {
        let body = compact(&text(frame));
        assert!(body.starts_with("종별시험종목시험방법시험빈도비고"));
        assert_eq!(
            body.matches("종별").count(),
            1,
            "반복 머리글은 조각마다 한 번"
        );
    }
    assert!(
        bottom(continued) < next.bbox.y,
        "앞 표 종료 뒤 후속 표 배치"
    );
    assert!(compact(&text(next)).contains("콘크리트용골재"));
}

fn rowbreak_example(sample: &str, compatibility_2024: bool) {
    let core = load(sample, compatibility_2024);
    let root = core.build_page_render_tree(16).unwrap().root;
    let outer = table(&root, 28, 0, 1, 1);
    let frame = unique(outer, |node| {
        matches!(node.node_type, RenderNodeType::Rectangle(_))
            && compact(&text(node)).contains("예시:데이터개방체계구축예시")
    });
    // 두 독립 Print의17쪽은 같은 diagram과 첫 예시 글상자 좌표를 가진다.
    edges(frame, [120.734670, 479.794678, 672.438680, 749.739990]);
    let picture = image(frame);
    edges(picture, [140.884003, 520.230713, 652.129313, 686.928060]);
    contained(picture, frame);
    contained(frame, outer);
    let tail = table(outer, 28, 0, 4, 2);
    assert!(bottom(frame) <= tail.bbox.y);
    assert!(compact(&text(tail)).contains("오픈API개발"));
}

#[test]
fn rowbreak_hwp_page_seventeen_keeps_the_example_diagram_and_following_table() {
    rowbreak_example("samples/rowbreak-problem-pages.hwp", true);
}

#[test]
fn rowbreak_hwpx_page_seventeen_keeps_the_example_diagram_and_following_table() {
    rowbreak_example("samples/rowbreak-problem-pages.hwpx", false);
}

fn vpos_title(sample: &str) {
    let core = load(sample, false);
    let root = core.build_page_render_tree(0).unwrap().root;
    let frame = table(&root, 3, 0, 3, 3);
    // pdf/table-vpos-01-{hwp,hwpx}-2020.pdf 1쪽 제목 chars[].origin.
    let title = baseline(frame, "정부혁신으로 달라지는 모습", 426.719971);
    contained(title, frame);
    let body = unique(frame, |node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
            && compact(&text(node)).contains("범정부국민참여플랫폼이구축되어")
    });
    assert!(bottom(title) <= body.bbox.y, "제목 뒤에 본문을 배치한다");
    assert!(body.bbox.y < bottom(frame), "본문은 같은 표 소유다");
}

#[test]
fn vpos_hwp_first_page_keeps_the_table_title_baseline_and_body_order() {
    vpos_title("samples/table-vpos-01.hwp");
}

#[test]
fn vpos_hwpx_first_page_keeps_the_table_title_baseline_and_body_order() {
    vpos_title("samples/table-vpos-01.hwpx");
}

#[test]
fn toggled_picture_first_page_keeps_the_cell_picture_and_successor_origin() {
    let core = load("samples/pic-in-table-with-toggle.hwp", false);
    let root = core.build_page_render_tree(0).unwrap().root;
    let frame = table(&root, 0, 2, 2, 2);
    let in_cell = image(frame);
    // pdf/pic-in-table-with-toggle-2020.pdf 1쪽 세 image bbox.
    edges(in_cell, [170.148000, 248.368042, 333.420003, 353.373372]);
    contained(
        in_cell,
        cell_owner(&root, in_cell, None).expect("그림의 원본 셀"),
    );
    let band = unique(&root, |node| {
        matches!(&node.node_type, RenderNodeType::Image(i)
            if i.para_index == Some(0) && i.control_index == Some(3))
    });
    edges(band, [113.378662, 656.402669, 339.177327, 671.745321]);
    let following = unique(&root, |node| {
        matches!(&node.node_type, RenderNodeType::Image(i)
            if i.para_index == Some(1) && i.control_index == Some(0))
    });
    edges(following, [113.378662, 679.736043, 276.650675, 784.741374]);
    assert!(bottom(frame) <= band.bbox.y && bottom(band) <= following.bbox.y);
}

#[test]
fn carried_anchor_first_page_keeps_the_last_body_line_and_footnote_baselines() {
    let core = load("samples/issue5543_carried_anchor_ladder.hwpx", false);
    let root = core.build_page_render_tree(0).unwrap().root;
    // pdf/issue5543_carried_anchor_ladder-2020.pdf 1쪽 rawdict의 실제 기준선.
    let last = baseline(&root, "화에 크게 이바지할 것으로 기대된다", 993.953776);
    let note = baseline(
        &root,
        "수입 건초 가격: 2023년 9월 거래 가격 기준",
        1020.484782,
    );
    assert!(
        bottom(last) <= note.bbox.y,
        "후속 주석과 본문이 겹치면 안 된다"
    );
}

fn stack_picture(sample: &str) {
    let core = load(sample, false);
    let root = core.build_page_render_tree(4).unwrap().root;
    let frame = table(&root, 42, 0, 1, 1);
    let picture = image(frame);
    // pdf/issue2004_cell_image_stack-{hwp,hwpx}-2020.pdf 5쪽은 양쪽100%다.
    edges(picture, [107.142670, 88.702637, 686.031982, 928.583984]);
    contained(
        picture,
        cell_owner(&root, picture, None).expect("스택 그림의 물리 셀"),
    );
}

#[test]
fn image_stack_hwp_page_five_keeps_all_four_picture_edges_inside_its_cell() {
    stack_picture("samples/issue2004_cell_image_stack.hwp");
}

#[test]
fn image_stack_hwpx_page_five_keeps_all_four_picture_edges_inside_its_cell() {
    stack_picture("samples/issue2004_cell_image_stack.hwpx");
}

#[test]
fn hft_hwpx_page_104_keeps_source_face_and_post_table_body_origins() {
    let core = load(
        "samples/issue1949_giant_cell_nested_tables_perf.hwpx",
        false,
    );
    let root = core.build_page_render_tree(103).unwrap().root;
    // 독립 Print p104의 span origin을 72pt→96dpi로 환산한다.
    // 양쪽98%·직접 판독: #7902 10/11단계의 frozen source3a 증거와 최종 재검증.
    let heading = baseline(&root, "11.2.8조종장치와조작장치의손잡이", 374.310954);
    let body = baseline(&root, "11.2.8.1육상착륙장치가있는경우", 425.455037);
    let successor = baseline(&root, "11.2.8.2동력장치조작손잡이", 882.235840);
    let picture = image(&root);
    edges(picture, [270.894674, 514.157308, 532.513346, 833.487956]);
    assert!(bottom(heading) < body.bbox.y);
    assert!(
        bottom(body) < picture.bbox.y && bottom(picture) < successor.bbox.y,
        "앞 본문·그림·뒤 본문의 순서와 비겹침"
    );
    for line in [heading, body, successor] {
        let mut descendants = Vec::new();
        walk(line, &mut descendants);
        let hangul: Vec<_> = descendants
            .into_iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run)
                    if run
                        .display_or_text()
                        .chars()
                        .any(|ch| ('가'..='힣').contains(&ch)) =>
                {
                    Some(run)
                }
                _ => None,
            })
            .collect();
        assert!(!hangul.is_empty(), "실제 원본의 한글 run을 검사한다");
        for run in hangul {
            assert!(
                run.style.source_font_is_hft,
                "원본 HFT 선언이 resolver에서 실제 paint style까지 전달돼야 한다"
            );
        }
    }
}
