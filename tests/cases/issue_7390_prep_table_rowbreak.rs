//! #7390 / PR #7406: KoPub 줄 폭 변경 뒤 PrEP 69쪽의 다행 RowBreak 표가
//! 선행 본문과 캡션 위로 끌려 올라오지 않고 70쪽에 이어져야 한다.
//!
//! 독립 기준: `pdf/issue2006/1790387_prep_final_report-2024.pdf` (원 HWPX를
//! 한컴 2024 엔진으로 변환). 물리 69쪽은 `<표 27>` 캡션 뒤에 표가 시작하고,
//! 물리 70쪽에서 마지막 행들을 이어 그린다. 수정 전에는 38행 전체가 69쪽
//! 본문 상단 y=94.5px에 놓여 앞 글·캡션을 덮고 70쪽 후속 조각은 없다.

use std::collections::BTreeSet;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue2006/1790387_prep_final_report.hwpx";

#[test]
fn prep_page_33_keeps_complete_final_word_on_saved_sixth_line() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(32).expect("물리 33쪽");
    let mut lines = Vec::new();
    paragraph_lines(&page.root, 345, &mut lines);
    // 원본 저장 LineSeg 6개와 한컴 2024 PDF 물리 33쪽의 여섯 줄이
    // 같은 어절 경계를 보존한다. 더 적은 줄로 채울 수 있다는 사실만으로
    // 이 정상 저장 분할을 무효로 분류하면 안 된다.
    assert_eq!(lines.len(), 6, "정상 저장 여섯 줄: {lines:?}");
    assert!(lines[0].trim_end().ends_with("한다는"), "첫 줄: {lines:?}");
    assert_eq!(lines[5].trim(), "왜곡될 수 있음.");
    let last = line_top_containing(&page.root, "왜곡될 수 있음.").expect("마지막 줄");
    // PDF yMin=430.848pt, 96dpi.
    assert!((last - 574.464).abs() <= 1.5, "마지막 줄 위치: {last:.3}");
}

#[test]
fn prep_page_76_single_cell_does_not_reserve_last_line_spacing() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(75).expect("물리 76쪽");
    let first = line_top_containing(&page.root, "대상자 1 (30대 후반").expect("표 첫 줄");
    let following = line_top_containing(&page.root, "다양한 트랜지션 과정을").expect("표 뒤 본문");
    // 한컴 PDF bbox yMin 113.232/494.568pt, 96dpi. 저장 표 높이는
    // 37100HU = 마지막 줄 끝(34400+1000) + 위아래 여백(850+850).
    // 마지막 줄의 후행 600HU는 이 완결 프레임 뒤에 다시 더하지 않는다.
    assert!((first - 150.976).abs() <= 1.5, "표 첫 줄: {first:.3}");
    assert!(
        (following - 659.424).abs() <= 1.5,
        "표 뒤 본문: {following:.3}"
    );
}

fn target_table(node: &RenderNode) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(27)
            && table.control_index == Some(0)
            && table.row_count == 38
            && table.col_count == 4
        {
            return Some(node);
        }
    }
    node.children.iter().find_map(target_table)
}

fn line_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        line_text(child, text);
    }
}

fn caption_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        line_text(node, &mut text);
        if text.contains("표 27") {
            return Some(node.bbox.y + node.bbox.height);
        }
    }
    node.children.iter().find_map(caption_bottom)
}

fn line_top_containing(node: &RenderNode, needle: &str) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        line_text(node, &mut text);
        if text.contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| line_top_containing(child, needle))
}

fn visible_rows(table: &RenderNode) -> BTreeSet<u16> {
    table
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TableCell(cell) => Some(cell.row),
            _ => None,
        })
        .collect()
}

fn paragraph_lines(node: &RenderNode, para_index: usize, lines: &mut Vec<String>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) {
            let mut text = String::new();
            line_text(node, &mut text);
            lines.push(text);
        }
    }
    for child in &node.children {
        paragraph_lines(child, para_index, lines);
    }
}

fn footnote_text(node: &RenderNode) -> Option<String> {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        let mut text = String::new();
        line_text(node, &mut text);
        return Some(text);
    }
    node.children.iter().find_map(footnote_text)
}

#[test]
fn prep_kopub_paragraph_does_not_orphan_final_syllable() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(66).expect("물리 67쪽");
    let mut lines = Vec::new();
    paragraph_lines(&page.root, 14, &mut lines);
    assert_eq!(
        lines.len(),
        4,
        "한컴 2024 PDF 물리 67쪽의 문단은 네 줄: {lines:?}"
    );
    assert!(
        lines.last().is_some_and(|line| line.ends_with("하였음")),
        "마지막 음절은 네 번째 줄에 있어야 한다: {lines:?}"
    );
    let mut next_paragraph = Vec::new();
    paragraph_lines(&page.root, 16, &mut next_paragraph);
    assert!(
        next_paragraph
            .last()
            .is_some_and(|line| line.ends_with("연령을 정리하면 다음과 같음.")),
        "한컴 PDF에서는 다음 문단의 마지막 줄까지 물리 67쪽에 있다: {next_paragraph:?}"
    );
}

#[test]
fn prep_chart_caption_preserves_following_saved_line_spacing() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(26).expect("물리 27쪽");
    let caption =
        line_top_containing(&page.root, "[그림 6] 서울시민 성생활 비율").expect("여성 차트 캡션");
    let following =
        line_top_containing(&page.root, "남성에서는 20대의 성생활 비율").expect("차트 뒤 본문");
    // 한컴 2024 PDF: 캡션 y=426.7px, 뒤 본문 y=476.7px. 저장 chart
    // LineSeg의 trailing spacing 720HU(9.6px)가 두 글줄 사이에 들어간다.
    assert!((caption - 426.7).abs() <= 1.5, "캡션 위치: {caption:.1}");
    assert!(
        (following - 476.7).abs() <= 1.5,
        "차트 뒤 본문은 저장 줄간격을 점유해야 한다: {following:.1}"
    );
}

#[test]
fn prep_single_cell_continuation_keeps_vertical_padding_and_following_flow() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(38).expect("물리 39쪽");
    let next = core.build_page_render_tree(39).expect("물리 40쪽");
    // 독립 한컴 2024 PDF bbox: p39 A1 y=81.552pt, p40 A8의 두 번째
    // 선택지 y=79.872pt, 뒤 본문 y=165.648pt. 96dpi로 환산했다.
    let a1 = line_top_containing(&first.root, "A1. 귀하의 출생년도를").expect("A1 첫 줄");
    let a8_tail = line_top_containing(&next.root, "2) 없다").expect("A8 이어지는 선택지");
    let following = line_top_containing(&next.root, "관심집단과 대조군은").expect("표 뒤 본문");
    assert!((a1 - 108.736).abs() <= 1.5, "39쪽 첫 줄: {a1:.1}");
    assert!(
        (a8_tail - 106.496).abs() <= 1.5,
        "40쪽 이어지는 줄: {a8_tail:.1}"
    );
    assert!(
        (following - 220.864).abs() <= 1.5,
        "표 뒤 본문: {following:.1}"
    );
}

#[test]
fn prep_intra_paragraph_cell_split_keeps_last_line_on_page_34() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(33).expect("물리 34쪽");
    let next = core.build_page_render_tree(34).expect("물리 35쪽");
    // 원본 1×1 CELL 표의 마지막 문단은 저장 두 줄의 vertpos가
    // 28200/29800HU이다. 한컴 PDF p34는 두 번째 줄을 표 하단에 두고,
    // p35는 바로 (4) Jenness 항목으로 시작한다.
    let tail = line_top_containing(&first.root, "는 경향을 보임").expect("34쪽 마지막 줄");
    assert!((tail - 993.696).abs() <= 1.5, "34쪽 마지막 줄 y: {tail:.2}");
    fn find_cell_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.row_count == 1
                && table.col_count == 1
                && line_top_containing(node, "는 경향을 보임").is_some()
            {
                return Some(node);
            }
        }
        node.children.iter().find_map(find_cell_table)
    }
    fn body_bottom(node: &RenderNode) -> Option<f64> {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            return Some(node.bbox.y + node.bbox.height);
        }
        node.children.iter().find_map(body_bottom)
    }
    let table = find_cell_table(&first.root).expect("34쪽 단일 셀 표");
    let bottom = body_bottom(&first.root).expect("34쪽 본문 바닥");
    assert!(
        table.bbox.y + table.bbox.height <= bottom + 0.5,
        "34쪽 표 바닥 {:.2} > 본문 바닥 {bottom:.2}",
        table.bbox.y + table.bbox.height
    );
    assert!(line_top_containing(&next.root, "는 경향을 보임").is_none());
    let next_item = line_top_containing(&next.root, "(4) Jenness").expect("35쪽 첫 항목");
    assert!(
        (next_item - 106.496).abs() <= 1.5,
        "35쪽 첫 항목 y: {next_item:.2}"
    );
}

#[test]
fn prep_population_table_keeps_last_fitting_rows_with_first_fragment() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(49).expect("물리 50쪽");
    let next = core.build_page_render_tree(50).expect("물리 51쪽");
    fn population_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(480) && table.row_count == 21 && table.col_count == 5 {
                return Some(node);
            }
        }
        node.children.iter().find_map(population_table)
    }
    let first_rows = visible_rows(population_table(&first.root).expect("50쪽 인구집단 표"));
    let next_rows = visible_rows(population_table(&next.root).expect("51쪽 인구집단 표"));
    // 한컴 2024 PDF p50은 치과의사·약사 두 행까지 놓고, p51은
    // 반복 제목행 뒤 간호사 행부터 재개한다.
    assert!(
        first_rows.contains(&11) && first_rows.contains(&12),
        "50쪽 행: {first_rows:?}"
    );
    assert!(!next_rows.contains(&11) && !next_rows.contains(&12));
    assert!(next_rows.contains(&13), "51쪽 행: {next_rows:?}");
}

#[test]
fn prep_centered_inline_table_uses_host_paragraph_margin() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(56).expect("물리 57쪽");
    fn demographic_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(535) && table.row_count == 29 && table.col_count == 2 {
                return Some(node);
            }
        }
        node.children.iter().find_map(demographic_table)
    }
    let table = demographic_table(&page.root).expect("성별 정체성 표");
    // 한컴 2024 PDF p57의 표 좌우 괘선은 96dpi에서 x=206/599px.
    // 원본 host의 1000HU 왼쪽 여백을 가운데 정렬 폭에도 반영해야 한다.
    assert!(
        (table.bbox.x - 206.0).abs() <= 1.5,
        "표 왼쪽: {:.2}",
        table.bbox.x
    );
    assert!((table.bbox.x + table.bbox.width - 599.0).abs() <= 1.5);
}

#[test]
fn prep_paragraph_relative_chart_image_uses_host_left_margin() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(60).expect("물리 61쪽");
    fn chart_image(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Image(image) = &node.node_type {
            if image.section_index == Some(1)
                && image.para_index == Some(7)
                && image.control_index == Some(0)
            {
                return Some(node);
            }
        }
        node.children.iter().find_map(chart_image)
    }
    let image = chart_image(&page.root).expect("성 파트너 수 분포 그림");
    // 한컴 2024 PDF p61의 그림 원점은 96dpi에서 x=110.3px.
    // 같은 TIFF가 rhwp에서는 문단 왼쪽 여백 1000HU를 빠뜨려 x=96.9px였다.
    assert!(
        (image.bbox.x - 110.3).abs() <= 1.5,
        "그림 x: {:.2}",
        image.bbox.x
    );
    // 캡션 뒤 본문은 저장 vpos=43579HU에서 곧바로 시작한다.
    // 한컴 PDF 첫 줄/다음 문단은 각각 676.2/762.0px이다.
    let next =
        line_top_containing(&page.root, "HIV 검사를 한 적 있는 사람은").expect("그림 뒤 첫 문단");
    let following = line_top_containing(&page.root, "성연결망 분석은").expect("그림 뒤 둘째 문단");
    assert!((next - 676.2).abs() <= 1.5, "그림 뒤 첫 문단 y: {next:.2}");
    assert!(
        (following - 762.0).abs() <= 1.5,
        "그림 뒤 둘째 문단 y: {following:.2}"
    );
}

#[test]
fn prep_saved_table_tail_keeps_subject_seven_on_page_81() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(80).expect("물리 81쪽");
    let next = core.build_page_render_tree(81).expect("물리 82쪽");
    // 한컴 2024 PDF는 대상자 7의 라벨·응답을 81쪽 표의 마지막에,
    // 저장 vpos가 0으로 돌아간 대상자 8은 82쪽 표 처음에 둔다.
    assert!(line_top_containing(&first.root, "대상자 7 (여, 40대)").is_some());
    assert!(line_top_containing(&first.root, "물 중단 후 금단 증상으로").is_some());
    assert!(line_top_containing(&next.root, "대상자 7 (여, 40대)").is_none());
    assert!(line_top_containing(&next.root, "대상자 8 (남, 30대)").is_some());
}

#[test]
fn prep_page_90_keeps_saved_leading_spacing_after_page_start() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(89).expect("물리 90쪽");
    // 한컴 2024 PDF p90: 첫 문단의 500HU 앞 간격은 첫 줄 vpos에도
    // 들어 있고, 뒤 문단 사이의 500HU 간격도 보존된다.
    let first =
        line_top_containing(&page.root, "전파경로에 대한 응답률이 적어").expect("90쪽 첫 문단");
    let next =
        line_top_containing(&page.root, "HIV 감염인 대상으로 감염 경로 확인").expect("뒤 소제목");
    let heading = line_top_containing(&page.root, "3. 생존 HIV 감염인의 인구학적 특성")
        .expect("다음 절 제목");
    let caption = line_top_containing(&page.root, "<표 37> 2024년 HIV 내국인").expect("표 37 캡션");
    assert!((first - 101.8).abs() <= 1.5, "첫 문단 y: {first:.1}");
    assert!((next - 161.2).abs() <= 1.5, "소제목 y: {next:.1}");
    assert!((heading - 346.5).abs() <= 1.5, "절 제목 y: {heading:.1}");
    assert!((caption - 456.9).abs() <= 1.5, "표 캡션 y: {caption:.1}");
}

#[test]
fn prep_page_79_keeps_last_quote_before_saved_cell_reset() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let before = core.build_page_render_tree(78).expect("물리 79쪽");
    let after = core.build_page_render_tree(79).expect("물리 80쪽");
    // 한컴 2024 PDF p79는 표의 '필로폰이 제게' 한 줄을 같은 쪽에 두고,
    // p80은 저장 vpos가 0으로 되감긴 다음 문단부터 시작한다.
    assert!(
        line_top_containing(&before.root, "필로폰이 제게 잘 맞는다고").is_some(),
        "저장 프레임 마지막 응답은 79쪽에 남아야 함"
    );
    assert!(
        line_top_containing(&after.root, "필로폰이 제게 잘 맞는다고").is_none(),
        "80쪽에 마지막 응답이 중복되거나 이월되면 안 됨"
    );
    assert!(
        line_top_containing(&after.root, "채팅 어플로 사람들을 만나기도").is_some(),
        "80쪽은 저장 reset 뒤의 다음 응답에서 재개"
    );
}

#[test]
fn prep_page_92_caption_keeps_saved_negative_empty_line_advance() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(91).expect("물리 92쪽");
    // 한컴 2024 PDF p92 caption bbox: 742.872pt × 96/72 = 990.496px.
    // 저장 빈 문단(66533HU) 뒤 캡션은 67163HU: 1050HU 줄 상자와
    // -420HU 간격의 630HU 전진을 보존한다. 빈 글자와 0 높이는 다르다.
    let caption = line_top_containing(&page.root, "[그림 14] QUANTPrEP을").expect("그림 14 캡션");
    assert!((caption - 990.496).abs() <= 1.5, "캡션 y: {caption:.3}");
}

#[test]
fn prep_page_93_caption_starts_after_saved_picture_bottom() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(92).expect("물리 93쪽");
    // 한컴 2024 PDF p93 bbox(72dpi → 96dpi): 그림 15 캡션 572.1px,
    // 뒤 첫 문단 613.0px, CDC 문단 685.8px.
    let caption = line_top_containing(&page.root, "[그림 15] QUANTPrEP을").expect("그림 15 캡션");
    let first = line_top_containing(&page.root, "2024년 주민등록연앙인구를 활용하였을 때")
        .expect("그림 뒤 첫 문단");
    let cdc = line_top_containing(&page.root, "다음으로 미국 CDC에서").expect("CDC 문단");
    assert!((caption - 572.1).abs() <= 1.5, "캡션 y: {caption:.1}");
    assert!((first - 613.0).abs() <= 1.5, "그림 뒤 문단 y: {first:.1}");
    assert!((cdc - 685.8).abs() <= 1.5, "CDC 문단 y: {cdc:.1}");
}

#[test]
fn prep_page_105_merged_budget_table_keeps_saved_row_boundaries() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let page = core.build_page_render_tree(104).expect("물리 105쪽");
    fn budget_table(node: &RenderNode) -> Option<&RenderNode> {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(336) && table.row_count == 20 && table.col_count == 6 {
                return Some(node);
            }
        }
        node.children.iter().find_map(budget_table)
    }
    let table = budget_table(&page.root).expect("연구비 사용 내역 표");
    let row_top = |row| {
        table
            .children
            .iter()
            .find_map(|child| match &child.node_type {
                RenderNodeType::TableCell(cell) if cell.row == row && cell.col == 3 => {
                    Some(child.bbox.y)
                }
                _ => None,
            })
    };
    // 한컴 2024 PDF p105의 수평 괘선(96dpi): 첫 행 243/279px,
    // 중간 10행 634px, 끝에서 둘째 행 975px, 표 끝 1019px.
    for (row, expected) in [(1, 279.0), (10, 634.0), (19, 975.0)] {
        let actual = row_top(row).expect("해당 행의 세로 병합 없는 셀");
        assert!(
            (actual - expected).abs() <= 2.0,
            "행 {row} 괘선: 실제 {actual:.1}, PDF {expected:.1}"
        );
    }
    assert!((table.bbox.y + table.bbox.height - 1019.0).abs() <= 2.0);
}

#[test]
fn prep_footnote_four_starts_on_next_physical_page() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let marker_page = core.build_page_render_tree(66).expect("물리 67쪽");
    let note_page = core.build_page_render_tree(67).expect("물리 68쪽");
    let chart_caption_y = line_top_containing(&note_page.root, "연도별 성주체성장애 진단 건수")
        .expect("68쪽 차트 캡션");
    assert!(
        (chart_caption_y - 705.7).abs() <= 1.5,
        "한컴 2024 PDF 68쪽 차트 캡션 y=705.7px, 실제={chart_caption_y:.1}"
    );
    let marker_page_notes = footnote_text(&marker_page.root).unwrap_or_default();
    let next_page_notes = footnote_text(&note_page.root).unwrap_or_default();
    assert!(
        !marker_page_notes.contains("F64 코드를 가장 처음 진단받은"),
        "한컴 PDF의 각주 4 본문은 67쪽에 없다: {marker_page_notes}"
    );
    assert!(
        next_page_notes.contains("F64 코드를 가장 처음 진단받은"),
        "한컴 PDF의 각주 4 본문은 68쪽에 있다: {next_page_notes}"
    );
    assert!(
        next_page_notes.contains("주상병 및 배제 상병"),
        "각주 5도 같은 68쪽에 남아야 한다: {next_page_notes}"
    );

    let mut lines = Vec::new();
    paragraph_lines(&note_page.root, 25, &mut lines);
    assert_eq!(lines.len(), 3, "각주 위 본문은 세 줄이어야 한다: {lines:?}");
    assert!(
        lines
            .last()
            .is_some_and(|line| line.trim_end().ends_with("105,534명이")),
        "한컴 PDF에서 각주 위 본문 세 번째 줄도 68쪽에 들어간다: {lines:?}"
    );
}

#[test]
fn prep_table_starts_after_caption_and_continues_next_page() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("PrEP 정식 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("PrEP 로드");
    let first = core.build_page_render_tree(68).expect("물리 69쪽");
    let next = core.build_page_render_tree(69).expect("물리 70쪽");

    let first_table = target_table(&first.root).expect("69쪽 표 27의 첫 조각");
    let caption_end = caption_bottom(&first.root).expect("69쪽 표 27 캡션");
    assert!(
        first_table.bbox.y + 0.5 >= caption_end,
        "표 첫 조각이 선행 캡션을 덮는다: 표 y={:.1}, 캡션 끝={caption_end:.1}",
        first_table.bbox.y
    );

    let first_rows = visible_rows(first_table);
    let next_table = target_table(&next.root).expect("70쪽 표 27의 후속 조각");
    let next_rows = visible_rows(next_table);
    assert!(first_rows.contains(&0), "첫 조각은 첫 행을 소유한다");
    assert!(
        !first_rows.contains(&37),
        "마지막 행은 첫 조각 밖에 있어야 한다"
    );
    assert!(
        next_rows.contains(&37),
        "다음 쪽에서 마지막 행을 이어 그린다"
    );
    assert!(
        (1..38).all(|row| first_rows.contains(&row) ^ next_rows.contains(&row)),
        "헤더 외 모든 원본 행은 두 쪽에서 정확히 한 번 소유한다: 첫={first_rows:?}, 다음={next_rows:?}"
    );
}
