//! [Issue #6803] 쪽 분할 조각의 **시작 행에서 시작해 끝 행을 넘는** rowspan 셀이
//! 끝 컷을 못 받아 **다음 쪽이 다시 그릴 몫까지** 배치했다.
//!
//! `1376496` (전주·완주 혁신도시 [별표 6]) 의 `pi=6 ci=0` 표는 14행×5열
//! `쪽나눔=RowBreak`. 셀 `r=8,c=0 rs=6` 이 문단 67개(빈 문단으로 세로 가운데를 만든
//! 수작업 서식, 156,749HU = 2,090px)를 담고 3·4쪽에 걸친다.
//!
//! ```text
//! DIAG_FRAG pi=6 ci=0 rows=8..11  start_cut=[1] end_cut=[1]   ← end_cut 은 행 10 의 부기
//! DIAG_FRAG pi=6 ci=0 rows=10..14 start_cut=[1] end_cut=[5]
//! ```
//!
//! `table_partial.rs` 의 행 술어가 셀을 **시작 행으로만** 분류한다.
//!
//! ```text
//! is_split_start_row = !start_cut.is_empty() && cell_row == start_row      // 8 == 8  → true
//! is_split_end_row   = !end_cut.is_empty()   && cell_row == end_row - 1    // 8 == 10 → false
//! ```
//!
//! `apply_end == false` 라 `cell_cut_window` 가 `eu = usize::MAX` 를 내고, `#1748` 의
//! 높이-기반 구제(`is_rowbreak_straddle`)는 `!is_in_split_row` 조건에서 막힌다.
//! 그래서 3쪽이 `pi=0..66` 을 전부 그렸다 — 글줄이 용지 아래 **1,015.1px**,
//! `export-text` 3쪽의 `A1` 이 **2 대신 6**.
//!
//! ⭐ 같은 셀이 4쪽에서는 `is_in_split_row=false` → `is_rowbreak_straddle` 로 높이 컷을
//! 받아 `pi=34` 에서 **정확히 이어받는다**. 컷 위치는 처음부터 옳았고 3쪽만 멈추지
//! 않았다 — 두 경로의 비대칭만 메운다.
//!
//! 아래 시험은 **양쪽을 잠근다**: 늦게 끊으면(중복) 3쪽 마지막이 33을 넘고, 일찍
//! 끊으면(소실) 3쪽 끝과 4쪽 시작 사이에 구멍이 난다.
//!
//! 재현 원본은 samples/issue6803/에 정식 등록했다. 파일 부재는 실패다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6803/1376496-neighborhood-facility-land-table.hwp";

/// 작업 디렉터리나 비공개 환경 변수와 무관한 정식 회귀 입력.
fn document_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 문단 67개를 담은 `r=8,c=0 rs=6` 병합 셀.
const TALL_CELL_ROW: u16 = 8;
const TALL_CELL_COL: u16 = 0;
const TALL_CELL_ROW_SPAN: u16 = 6;

/// 그 셀이 이 쪽에 그린 직계 글줄의 문단 인덱스.
fn tall_cell_para_indices(node: &RenderNode, out: &mut Vec<usize>) {
    if !node.visible || node.editor_only {
        return;
    }
    if let RenderNodeType::Table(meta) = &node.node_type {
        if meta.section_index != Some(0)
            || meta.para_index != Some(6)
            || meta.control_index != Some(0)
            || meta.cell_context.is_some()
        {
            return;
        }
    }
    if let RenderNodeType::TableCell(meta) = &node.node_type {
        if meta.row == TALL_CELL_ROW
            && meta.col == TALL_CELL_COL
            && meta.row_span == TALL_CELL_ROW_SPAN
        {
            for child in &node.children {
                if let RenderNodeType::TextLine(line) = &child.node_type {
                    if let Some(pi) = line.para_index {
                        out.push(pi);
                    }
                }
            }
            return;
        }
    }
    for child in &node.children {
        tall_cell_para_indices(child, out);
    }
}

fn paper_bottom(node: &RenderNode) -> f64 {
    node.bbox.y + node.bbox.height
}

/// 이 쪽에서 용지 아래로 가장 많이 내려간 글줄의 초과폭.
fn worst_text_line_overhang(node: &RenderNode, bottom: f64) -> f64 {
    let mut worst = 0.0f64;
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        worst = worst.max(node.bbox.y + node.bbox.height - bottom);
    }
    for child in &node.children {
        worst = worst.max(worst_text_line_overhang(child, bottom));
    }
    worst
}

fn source_table_fragment(node: &RenderNode) -> Option<&RenderNode> {
    if let RenderNodeType::Table(meta) = &node.node_type {
        if meta.section_index == Some(0)
            && meta.para_index == Some(6)
            && meta.control_index == Some(0)
            && meta.cell_context.is_none()
        {
            return Some(node);
        }
    }
    node.children.iter().find_map(source_table_fragment)
}

fn painted_text(node: &RenderNode, out: &mut String) {
    if !node.visible || node.editor_only {
        return;
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        painted_text(child, out);
    }
}

/// 원본 표 소속의 이 셀이 전쪽에서 그린 문단 인덱스 — 실제 paint 순서.
fn fragment_ranges() -> Vec<Vec<usize>> {
    let path = document_path();
    let bytes = std::fs::read(&path).expect("#6803 정식 회귀 sample 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        5,
        "쪽수 형상이 바뀌면 이 시험의 전제가 깨진다"
    );

    (0..core.page_count())
        .map(|page| {
            let tree = core
                .build_page_render_tree(page as u32)
                .expect("render tree");
            let mut pis = Vec::new();
            tall_cell_para_indices(&tree.root, &mut pis);
            pis
        })
        .filter(|pis| !pis.is_empty())
        .collect()
}

/// 조각들이 이 셀의 문단을 **빠짐없이·겹침없이** 나눠 갖는다.
///
/// 수정 전: 3쪽 조각이 `pi=0..66` 을 **전부** 그려 4쪽(`34..66`)과 33개가 겹쳤고,
/// 그중 `pi=34..66` 은 용지 아래로 나갔다(최대 +1,015.1px).
///
/// 이 하나로 양쪽이 잠긴다 — 늦게 끊으면 **중복**이, 일찍 끊으면 **구멍**이 생긴다.
#[test]
fn the_fragments_partition_the_cell_exactly_once() {
    let original = rhwp::parse_document(&std::fs::read(document_path()).expect("원본 읽기"))
        .expect("독립 원본 모델");
    let rhwp::model::control::Control::Table(source) =
        &original.sections[0].paragraphs[6].controls[0]
    else {
        panic!("분할 표의 원본 소속");
    };
    let source_cell = source
        .cells
        .iter()
        .find(|cell| {
            cell.row == TALL_CELL_ROW
                && cell.col == TALL_CELL_COL
                && cell.row_span == TALL_CELL_ROW_SPAN
        })
        .expect("병합 셀의 원본 소속");
    let ranges = fragment_ranges();
    let mut seen: Vec<usize> = ranges.iter().flatten().copied().collect();
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();

    assert_eq!(
        total,
        seen.len(),
        "조각들이 같은 문단을 두 번 그렸다 — #6803 회귀 (조각별 범위 {ranges:?})"
    );
    assert_eq!(
        seen.first().copied(),
        Some(0),
        "첫 조각은 이 셀의 첫 문단부터 그린다 (조각별 범위 {ranges:?})"
    );
    assert_eq!(
        seen.len(),
        seen.last().copied().expect("마지막") + 1,
        "조각 사이에 구멍이 있다 — #6803 회귀 (조각별 범위 {ranges:?})"
    );
    // 원본 셀의 67문단은 각각 저장 줄 하나다. 앞부분만 연속이고 마지막
    // 문단들이 누락된 경우와 조각 내부의 역순도 함께 검출한다.
    assert_eq!(
        ranges.iter().flatten().copied().collect::<Vec<_>>(),
        (0..source_cell.paragraphs.len()).collect::<Vec<_>>(),
        "원본의 모든 문단을 순서대로 한 번씩 보존해야 한다"
    );

    // 전쪽 90% 이상 확인 뒤 같은 표 종료를 검증할 수 있는 완료 페이지 중
    // Native/fresh WASM 양쪽 100%인 5쪽을 선택했다(96CSS DPI, compat2022).
    // 독립 Print의 실제 외곽 path 좌표다. bbox 측정값에서 기대값을 만들지 않는다.
    // 증적: regression-policy-20261015/land-default-environment-20261016/README.md.
    let core = DocumentCore::from_bytes(&std::fs::read(document_path()).expect("원본 읽기"))
        .expect("마지막 조각 문서");
    let tree = core
        .build_page_render_tree((core.page_count() - 1) as u32)
        .expect("마지막 쪽 실제 배치");
    let fragment = source_table_fragment(&tree.root).expect("같은 원본 표의 종료 조각");
    for (edge, actual, expected) in [
        ("left", fragment.bbox.x, 58.368),
        ("top", fragment.bbox.y, 92.538656),
        ("right", fragment.bbox.x + fragment.bbox.width, 698.663981),
        ("bottom", fragment.bbox.y + fragment.bbox.height, 376.068034),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print 5쪽 종료 표의 {edge}: expected={expected}, actual={actual}"
        );
    }
    let last_source_cell = source
        .cells
        .iter()
        .max_by_key(|cell| (cell.row, cell.col))
        .expect("원본 마지막 셀");
    let last_paragraph = last_source_cell
        .paragraphs
        .iter()
        .rev()
        .find(|paragraph| !paragraph.text.trim().is_empty())
        .expect("원본의 마지막 내용");
    let last_cell = fragment
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell)
            if cell.row == last_source_cell.row && cell.col == last_source_cell.col)
        })
        .expect("종료 조각 안의 같은 원본 셀");
    assert!(
        last_cell.bbox.y >= fragment.bbox.y
            && last_cell.bbox.y + last_cell.bbox.height <= fragment.bbox.y + fragment.bbox.height,
        "마지막 셀은 종료 조각 안에 포함돼야 한다"
    );
    let mut text = String::new();
    painted_text(last_cell, &mut text);
    let compact = |text: &str| {
        text.chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
    };
    assert!(
        compact(&text).ends_with(&compact(&last_paragraph.text)),
        "종료 위치만 맞추고 원본 마지막 문단을 잘라서는 안 된다"
    );
}

/// 각 조각은 **자기 몫에서 멈춘다** — 다음 조각이 이어받을 문단을 미리 그리지 않는다.
///
/// 수정 전: 3쪽 조각이 `pi=66` 까지 그렸다(4쪽이 다시 그리는 `pi=34..66` 포함).
#[test]
fn each_fragment_stops_where_the_next_one_resumes() {
    let ranges = fragment_ranges();
    for pair in ranges.windows(2) {
        let (Some(end), Some(next_start)) = (pair[0].last(), pair[1].first()) else {
            continue;
        };
        assert_eq!(
            *next_start,
            end + 1,
            "조각 이음매가 어긋났다 — #6803 회귀              (앞 조각 끝 pi={end}, 다음 조각 시작 pi={next_start};               수정 전 3쪽 조각이 pi=66 까지 그렸다)"
        );
    }
}

/// 용지 밖 글줄이 남지 않는다 (`layout-anomaly` off-canvas 1 → 0 과 같은 축).
#[test]
fn no_text_line_falls_off_the_paper() {
    let path = document_path();
    let bytes = std::fs::read(&path).expect("#6803 정식 회귀 sample 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let mut worst = 0.0f64;
    let mut worst_page = 0u32;
    let mut coordinate_scale = 1.0f64;
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page as u32)
            .expect("render tree");
        let bottom = paper_bottom(&tree.root);
        coordinate_scale = coordinate_scale.max(bottom.abs());
        let overhang = worst_text_line_overhang(&tree.root, bottom);
        if overhang > worst {
            worst = overhang;
            worst_page = page as u32 + 1;
        }
    }

    assert!(
        worst <= f64::EPSILON * 64.0 * coordinate_scale,
        "글줄이 용지 밖으로 나갔다 — #6803 회귀          (최대 초과 {worst:.1}px @ {worst_page}쪽; 수정 전 +1,015.1px @ 3쪽)"
    );
}

/// 원본 개체가 닫은 본문 프레임에 끝 행 상자가 포함된다.
/// 6803 4쪽의 분할 행은 내용 컷과 별개로 이 저장 상한을 지킨다.
/// 글줄·개체 순서와 전쪽 문단 보존은 위의 기존 관계 검사도 함께 유지한다.
///
/// 9d66b9277,96dpi/print/compat2022: 전5쪽 Native/fresh WASM 최저96.09324/96.90417%.
/// 원본 SHA-256:2482b695bc92c0cafe661af63c54acea834b6927bf9c666bdf47299910e3801d.
/// pdf/1376496-neighborhood-facility-land-table-2020.pdf:
/// ef7ac0c12d37f8b96f1af5e2d67e812151d9b9862c4f42824fee8bc04a5dda12.
/// 독립 Print와 양쪽4쪽 review를 직접 판독했다. 원본 한 저장 단위의 양자화만 허용하고
/// 화면의 절대 px 위치는 고정하지 않는다. 계획서의 후보42 검증 기록을 따른다.
#[test]
fn closing_row_stays_inside_the_original_body_frame() {
    use rhwp::model::control::Control;
    let bytes = std::fs::read(document_path()).expect("#6803 정식 입력");
    let mut core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 5, "독립 Print 전쪽과 내용 쪽수");
    let Control::Table(source) = &core.document().sections[0].paragraphs[6].controls[0] else {
        panic!("원본 문단 소유 표");
    };
    let source_height = f64::from(source.common.height) * 96.0 / 7200.0;
    let source_unit = 96.0 / 7200.0;
    fn find_table(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(meta)
            if meta.section_index == Some(0) && meta.para_index == Some(6)
                && meta.control_index == Some(0) && meta.cell_context.is_none())
        {
            return Some(node);
        }
        node.children.iter().find_map(find_table)
    }
    fn cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
        let mut cells = table.children.iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(meta) if meta.row == row && meta.col == col)
        });
        let selected = cells.next().expect("직접 소유한 칸");
        assert!(cells.next().is_none(), "행·열 소유는 유일");
        selected
    }
    let page = core.build_page_render_tree(3).expect("4쪽 render tree");
    let table = find_table(&page.root).expect("원본 표의 4쪽 조각");
    let header = cell(table, 0, 0);
    let ending = cell(table, 13, 4);
    assert!(
        ending.bbox.y >= header.bbox.y + header.bbox.height,
        "끝 행은 반복 머리행 뒤"
    );
    let frame_end = header.bbox.y + source_height;
    let ending_bottom = ending.bbox.y + ending.bbox.height;
    assert!(
        ending_bottom <= frame_end + source_unit,
        "끝 행 상자가 원본 본문 프레임 밖의 빈 띠를 다시 수용했다: end={ending_bottom:.8}, frame={frame_end:.8} (원본1HU 허용)"
    );
}

/// 독립 Print 1쪽의 결측 U+2024는 한컴바탕 글리프의 자연 폭을 쓴다.
/// 이 쪽의 Native/fresh WASM은 192DPI 98% 이상과 확대 판독을 확인했다.
/// source 기호 소속·전진 관계와 독립 Print의 후속 항목 원점을 함께 보호한다.
#[test]
fn title_missing_dot_uses_the_print_supplier_advance() {
    let core = DocumentCore::from_bytes(&std::fs::read(document_path()).expect("정식 sample"))
        .expect("문서 로드");
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(0)
            .expect("1쪽 공개 글자 배치"),
    )
    .expect("공개 배치 JSON");
    let expected = "근린생활시설용지 건축물 용도 ․ 건폐율 ․ 용적률 ․ 높이";
    let runs = layout["runs"].as_array().expect("runs");
    let anchors: Vec<_> = runs
        .iter()
        .filter(|run| {
            run["parentParaIdx"].is_null()
                && run["text"]
                    .as_str()
                    .is_some_and(|text| text.starts_with("근린생활시설용지 건축물 용도 "))
        })
        .collect();
    assert_eq!(anchors.len(), 1, "제목 원문의 누락·중복이 없어야 한다");
    let anchor = anchors[0];
    let start = anchor["charStart"].as_u64().expect("제목 원문 시작");
    let end = start + expected.chars().count() as u64;
    // 기호 슬롯 경계는 별도 run일 수 있다. 원문 소유를 따라 이어 붙이고
    // 한 run이라는 구현 형태 대신 내용·연속 소속·실제 전진을 검증한다.
    let title_runs: Vec<_> = runs
        .iter()
        .filter(|run| {
            run["parentParaIdx"].is_null()
                && run["secIdx"] == anchor["secIdx"]
                && run["paraIdx"] == anchor["paraIdx"]
                && run["charStart"]
                    .as_u64()
                    .is_some_and(|position| (start..end).contains(&position))
        })
        .collect();
    let mut cursor = start;
    let mut characters = Vec::new();
    for run in title_runs {
        assert_eq!(
            run["charStart"].as_u64(),
            Some(cursor),
            "제목 원문 소속의 틈·중복"
        );
        assert_eq!(run["y"], anchor["y"], "제목은 같은 글줄에 있어야 한다");
        let text = run["text"].as_str().expect("제목 조각");
        let positions = run["charX"].as_array().expect("source scalar 전진");
        assert_eq!(positions.len(), text.chars().count() + 1);
        let origin = run["x"].as_f64().expect("조각 원점");
        let font_size = run["fontSize"].as_f64().expect("글자 크기");
        for (index, ch) in text.chars().enumerate() {
            let x = positions[index].as_f64().unwrap();
            let advance = positions[index + 1].as_f64().unwrap() - x;
            characters.push((ch, origin + x, advance, font_size));
        }
        cursor += text.chars().count() as u64;
    }
    assert_eq!(cursor, end);
    assert_eq!(
        characters.iter().map(|item| item.0).collect::<String>(),
        expected
    );
    let mut dots = 0;
    for (index, &(ch, _, advance, font_size)) in characters.iter().enumerate() {
        if ch != '․' {
            continue;
        }
        dots += 1;
        let &(next, _, next_space, _) = characters.get(index + 1).expect("기호 뒤 공백");
        assert_eq!(next, ' ');
        assert!(advance > 0.0 && advance < next_space && advance < font_size / 4.0,
            "기호를 전각으로 늘리면 안 된다: 기호={advance}, 이웃 공백={next_space}, em={font_size}");
    }
    assert_eq!(dots, 3);
    // Print trace의 원점×(.119935pt/HU)×(96/72). 구현 결과에서 역산하지 않는다.
    for (label, pdf_css_x) in [("건폐율", 357.566), ("용적률", 418.813), ("높이", 479.900)]
    {
        let byte_index = expected.find(label).expect("후속 항목 보존");
        let index = expected[..byte_index].chars().count();
        let actual = characters[index].1;
        assert!(
            (actual - pdf_css_x).abs() <= 2.0,
            "독립 Print의 {label} 원점: expected={pdf_css_x}, actual={actual}"
        );
    }
}
