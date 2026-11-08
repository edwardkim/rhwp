//! 검사 유형: 관계형·고정형.
//! 2026-10-28 측정 raw(96dpi, PDFium):
//! rowfit-2020.hwp: 1쪽 Native 100.00000% / fresh WASM 100.00000%; 33쪽 Native 100.00000% / fresh WASM 100.00000%; 전33쪽 최저 100.00000% / 100.00000%.
//! reservefit-2020.hwp: 1쪽 Native 100.00000% / fresh WASM 100.00000%; 65쪽 Native 100.00000% / fresh WASM 100.00000%; 전65쪽 최저 100.00000% / 100.00000%.
//! headerrow-2020.hwp: 1쪽 Native 100.00000% / fresh WASM 100.00000%; 31쪽 Native 100.00000% / fresh WASM 100.00000%; 전31쪽 최저 100.00000% / 100.00000%.
//! 한글문서파일형식_5.0_revision1.3.hwp: 20쪽 Native 99.99560% / fresh WASM 99.92004%; 24쪽 Native 99.99751% / fresh WASM 99.99752%; 전71쪽 최저 95.31577% / 95.19516%.
//! hwpspec.hwp: 23쪽 Native 99.99389% / fresh WASM 99.79148%; 54쪽 Native 99.99765% / fresh WASM 99.99765%; 전178쪽 최저 92.45593% / 92.83271%.
//! 근거: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json,
//! mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json.
//!
//! #8149 쪽 바닥에서 나뉘는 빈 host 문단 기준 자리차지 표 — 첫 조각에 실리는 행과 다음 쪽 행.
//!
//! 독립 기준은 같은 HWP 의 한/글 2020 Print PDF(`pdf/issue8149/*-2020.pdf`)다. 생성기는
//! `samples/issue8149/gen_*.py` 이며, 아래 기대 행 구간은 PDF 본문 글자(`Rxx 행N`)에서 쪽마다 읽었다.
//! - `rowfit`: 아래 캡션 유무 × 바깥 아래 여백(1984/0HU) × 행 높이(1300~1900HU) 16사례.
//!   한/글은 첫 조각 아래에 `바깥 아래 여백 + 아래 캡션` 을 남긴다.
//! - `reservefit`: 같은 표를 같은 자리에 두고 바깥 아래 여백만 0..3000HU(200HU 간격) × 캡션 유무 32사례.
//!   예약이 여백에 2.7px 단위로 따라 움직이는지(한 행 줄어드는 문턱) 본다.
//! - `headerrow`: 제목 줄 반복 × 첫 행 제목 칸 × 채움 줄 15사례. 한/글은 제목 행만 쪽 끝에 남기는
//!   분할도 한다(H03·H08·H13). 남은 칸이 첫 행보다 작으면 표 전체가 다음 쪽으로 간다(H04 등).
//!
//! 검사는 쪽 소속(어느 행이 어느 쪽에 있나)·누락·중복이다. 좌표는 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn line_texts(node: &RenderNode, out: &mut Vec<String>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if let RenderNodeType::TextRun(run) = &n.node_type {
                text.push_str(&run.text);
            }
            stack.extend(n.children.iter().rev());
        }
        out.push(text);
        return;
    }
    for child in &node.children {
        line_texts(child, out);
    }
}

/// 사례 표지(`Rxx 캡션=` / `Hxx 반복=`)가 나온 쪽부터 다음 표지 앞 쪽까지, 쪽마다 그 사례의 행 번호.
fn rows_by_case(core: &DocumentCore, marker: &str) -> Vec<(String, Vec<Vec<u32>>)> {
    let mut cases: Vec<(String, Vec<Vec<u32>>)> = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 배치");
        let mut lines = Vec::new();
        line_texts(&tree.root, &mut lines);
        if let Some(label) = lines.iter().find_map(|line| {
            let (label, rest) = line.trim().split_once(' ')?;
            rest.starts_with(marker).then(|| label.to_string())
        }) {
            cases.push((label, Vec::new()));
        }
        let Some((label, pages)) = cases.last_mut() else {
            continue;
        };
        let prefix = format!("{label} 행");
        let mut rows: Vec<u32> = lines
            .iter()
            .filter_map(|line| line.trim().strip_prefix(&prefix)?.trim().parse().ok())
            .collect();
        rows.sort_unstable();
        pages.push(rows);
    }
    for (_, pages) in &mut cases {
        while pages.last().is_some_and(|rows| rows.is_empty()) {
            pages.pop();
        }
    }
    cases
}

fn expand(ranges: &[(u32, u32)]) -> Vec<u32> {
    ranges.iter().flat_map(|&(a, b)| a..=b).collect()
}

fn assert_cases(sample: &str, marker: &str, pages: u32, expected: &[(&str, &[&[(u32, u32)]])]) {
    let core = open(sample);
    assert_eq!(core.page_count(), pages, "{sample}: 한/글 PDF 쪽수");
    let actual = rows_by_case(&core, marker);
    assert_eq!(
        actual
            .iter()
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>(),
        expected.iter().map(|(label, _)| *label).collect::<Vec<_>>(),
        "{sample}: 사례 순서"
    );
    let mut wrong = Vec::new();
    for ((label, got), (_, want)) in actual.iter().zip(expected) {
        let want: Vec<Vec<u32>> = want.iter().map(|ranges| expand(ranges)).collect();
        if *got != want {
            wrong.push(format!("{label}: rhwp {got:?} / 한/글 {want:?}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "{sample}: 쪽별 행 소속이 한/글과 다르다
{}",
        wrong.join(
            "
"
        )
    );
}

const ROWFIT: &[(&str, &[&[(u32, u32)]])] = &[
    ("R01", &[&[(1, 15)], &[(16, 30)]]),
    ("R02", &[&[(1, 14)], &[(15, 30)]]),
    ("R03", &[&[(1, 12)], &[(13, 30)]]),
    ("R04", &[&[(1, 11)], &[(12, 30)]]),
    ("R05", &[&[(1, 17)], &[(18, 30)]]),
    ("R06", &[&[(1, 15)], &[(16, 30)]]),
    ("R07", &[&[(1, 13)], &[(14, 30)]]),
    ("R08", &[&[(1, 12)], &[(13, 30)]]),
    ("R09", &[&[(1, 17)], &[(18, 30)]]),
    ("R10", &[&[(1, 15)], &[(16, 30)]]),
    ("R11", &[&[(1, 13)], &[(14, 30)]]),
    ("R12", &[&[(1, 12)], &[(13, 30)]]),
    ("R13", &[&[(1, 19)], &[(20, 30)]]),
    ("R14", &[&[(1, 16)], &[(17, 30)]]),
    ("R15", &[&[(1, 15)], &[(16, 30)]]),
    ("R16", &[&[(1, 13)], &[(14, 30)]]),
];
const HEADERROW: &[(&str, &[&[(u32, u32)]])] = &[
    ("H01", &[&[(1, 3)], &[(1, 1), (4, 10)]]),
    ("H02", &[&[(1, 3)], &[(1, 1), (4, 10)]]),
    ("H03", &[&[(1, 1)], &[(1, 10)]]),
    ("H04", &[&[], &[(1, 10)]]),
    ("H05", &[&[], &[(1, 10)]]),
    ("H06", &[&[(1, 4)], &[(5, 10)]]),
    ("H07", &[&[(1, 3)], &[(4, 10)]]),
    ("H08", &[&[(1, 1)], &[(2, 10)]]),
    ("H09", &[&[], &[(1, 10)]]),
    ("H10", &[&[], &[(1, 10)]]),
    ("H11", &[&[(1, 4)], &[(5, 10)]]),
    ("H12", &[&[(1, 3)], &[(4, 10)]]),
    ("H13", &[&[(1, 1)], &[(2, 10)]]),
    ("H14", &[&[], &[(1, 10)]]),
    ("H15", &[&[], &[(1, 10)]]),
];
const RESERVEFIT: &[(&str, &[&[(u32, u32)]])] = &[
    ("R01", &[&[(1, 14)], &[(15, 30)]]),
    ("R02", &[&[(1, 15)], &[(16, 30)]]),
    ("R03", &[&[(1, 15)], &[(16, 30)]]),
    ("R04", &[&[(1, 15)], &[(16, 30)]]),
    ("R05", &[&[(1, 15)], &[(16, 30)]]),
    ("R06", &[&[(1, 14)], &[(15, 30)]]),
    ("R07", &[&[(1, 14)], &[(15, 30)]]),
    ("R08", &[&[(1, 14)], &[(15, 30)]]),
    ("R09", &[&[(1, 14)], &[(15, 30)]]),
    ("R10", &[&[(1, 14)], &[(15, 30)]]),
    ("R11", &[&[(1, 14)], &[(15, 30)]]),
    ("R12", &[&[(1, 14)], &[(15, 30)]]),
    ("R13", &[&[(1, 14)], &[(15, 30)]]),
    ("R14", &[&[(1, 13)], &[(14, 30)]]),
    ("R15", &[&[(1, 13)], &[(14, 30)]]),
    ("R16", &[&[(1, 13)], &[(14, 30)]]),
    ("R17", &[&[(1, 16)], &[(17, 30)]]),
    ("R18", &[&[(1, 16)], &[(17, 30)]]),
    ("R19", &[&[(1, 16)], &[(17, 30)]]),
    ("R20", &[&[(1, 16)], &[(17, 30)]]),
    ("R21", &[&[(1, 16)], &[(17, 30)]]),
    ("R22", &[&[(1, 16)], &[(17, 30)]]),
    ("R23", &[&[(1, 16)], &[(17, 30)]]),
    ("R24", &[&[(1, 15)], &[(16, 30)]]),
    ("R25", &[&[(1, 15)], &[(16, 30)]]),
    ("R26", &[&[(1, 15)], &[(16, 30)]]),
    ("R27", &[&[(1, 15)], &[(16, 30)]]),
    ("R28", &[&[(1, 15)], &[(16, 30)]]),
    ("R29", &[&[(1, 15)], &[(16, 30)]]),
    ("R30", &[&[(1, 15)], &[(16, 30)]]),
    ("R31", &[&[(1, 15)], &[(16, 30)]]),
    ("R32", &[&[(1, 14)], &[(15, 30)]]),
];

#[test]
fn rowfit_first_fragment_leaves_outer_bottom_and_bottom_caption() {
    assert_cases("samples/issue8149/rowfit-2020.hwp", "캡션=", 33, ROWFIT);
}

#[test]
fn reservefit_first_fragment_reserve_follows_outer_bottom_margin() {
    assert_cases(
        "samples/issue8149/reservefit-2020.hwp",
        "캡션=",
        65,
        RESERVEFIT,
    );
}

#[test]
fn headerrow_split_keeps_header_only_fragment_and_moves_unfit_table() {
    assert_cases(
        "samples/issue8149/headerrow-2020.hwp",
        "반복=",
        31,
        HEADERROW,
    );
}

// ── 실물: 한글문서파일형식 5.0 revision 1.3 — 독립 기준은 한/글 Print PDF(`…-hancom-official.pdf`, 71쪽).

const SPEC: &str = "samples/한글문서파일형식_5.0_revision1.3.hwp";
const SPEC_SECTION: usize = 3;

#[test]
fn spec_coanchored_following_table_stays_at_print_origin() {
    let core = open(SPEC);
    let tree = core.build_page_render_tree(19).expect("Print 20쪽");
    let mut stack = vec![&tree.root];
    let mut top = None;
    let mut following = None;
    let mut title = None;
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.section_index == Some(SPEC_SECTION) && table.para_index == Some(133) {
                match table.control_index {
                    Some(2) => top = Some(node.bbox.y),
                    Some(3) => following = Some(node.bbox),
                    _ => {}
                }
            }
        }
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.para_index == Some(133) && run.display_or_text() == "참고문헌" {
                title = Some((node.bbox.y, node.bbox.y + run.baseline));
            }
        }
        stack.extend(node.children.iter());
    }
    // 독립 한컴 Print 20쪽의 파란 표 상단은 96dpi 502px에 있다.
    let top = top.expect("공동 앵커 뒤 첫 표");
    assert!((top - 502.5).abs() <= 2.0, "표 상단 {top:.2}px");
    // 다음 표 외곽선과 제목 baseline도 독립 Print에서 직접 확인한다.
    let following = following.expect("두 번째 공동 앵커 표");
    assert!(
        (following.y - 681.49).abs() <= 2.0,
        "두 번째 표 상단 {:.2}px",
        following.y
    );
    let (title_top, title_baseline) = title.expect("표 뒤 제목");
    assert!(
        title_top >= following.y + following.height,
        "제목은 두 표 뒤에 있어야 한다"
    );
    assert!(
        (title_baseline - 862.26).abs() <= 2.0,
        "제목 baseline {title_baseline:.2}px"
    );
}

#[test]
fn spec_table_at_page_start_keeps_printed_top_margin() {
    let core = open(SPEC);
    let tree = core.build_page_render_tree(23).expect("Print 24쪽");
    let mut stack = vec![&tree.root];
    let mut top = None;
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.section_index == Some(SPEC_SECTION)
                && table.para_index == Some(163)
                && table.control_index == Some(0)
            {
                top = Some(node.bbox.y);
            }
        }
        stack.extend(node.children.iter());
    }
    // 독립 한컴 Print의 표 상단은 96dpi 117~118px이다.
    let top = top.expect("24쪽 첫 표");
    assert!((top - 117.5).abs() <= 2.0, "표 상단 {top:.2}px");
}

#[test]
fn hwpspec_second_coanchored_table_keeps_print_origin() {
    let core = open("samples/hwpspec.hwp");
    let tree = core.build_page_render_tree(22).expect("Print 23쪽");
    let mut stack = vec![&tree.root];
    let mut top = None;
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.section_index == Some(2)
                && table.para_index == Some(129)
                && table.control_index == Some(3)
            {
                top = Some(node.bbox.y);
            }
        }
        stack.extend(node.children.iter());
    }
    // 독립 한컴 Print에서 같은 표의 파란 상단은 96dpi 672~673px이다.
    let top = top.expect("공동 앵커 두 번째 표");
    assert!((top - 673.0).abs() <= 1.5, "표 상단 {top:.2}px");
}

#[test]
fn spec_table_caption_preserves_its_printed_number() {
    let core = open(SPEC);
    let para = &core.document().sections[SPEC_SECTION].paragraphs[163];
    let rhwp::model::control::Control::Table(table) = &para.controls[0] else {
        panic!("원본 표 컨트롤");
    };
    assert_eq!((table.common.attr >> 26) & 7, 6, "원본 번호 범주");
    let saved = table
        .caption
        .as_ref()
        .expect("원본 표 캡션")
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            rhwp::model::control::Control::AutoNumber(number) => Some(number.number),
            _ => None,
        });
    assert_eq!(saved, Some(18), "원본에 저장된 표 번호");
    let tree = core.build_page_render_tree(23).expect("Print 24쪽");
    let mut stack = vec![&tree.root];
    let mut captions = Vec::new();
    while let Some(node) = stack.pop() {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.caption_owner.is_some() {
                let text: String = node
                    .children
                    .iter()
                    .filter_map(|child| match &child.node_type {
                        RenderNodeType::TextRun(run) => Some(run.display_or_text()),
                        _ => None,
                    })
                    .collect();
                captions.push(text);
            }
        }
        stack.extend(node.children.iter());
    }
    assert!(
        captions
            .iter()
            .any(|text| { text.contains("18") && text.contains("바이너리 데이터 속성") }),
        "한컴 Print 표 18 캡션: {captions:?}"
    );
}

/// 구역 3 의 문단 `para` 표가 놓인 쪽들(0부터).
fn table_pages(core: &DocumentCore, para: usize) -> Vec<u32> {
    let mut pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 배치");
        let mut stack = vec![&tree.root];
        let mut found = false;
        while let Some(node) = stack.pop() {
            if let RenderNodeType::Table(table) = &node.node_type {
                found |=
                    table.section_index == Some(SPEC_SECTION) && table.para_index == Some(para);
            }
            stack.extend(node.children.iter());
        }
        if found {
            pages.push(page);
        }
    }
    pages
}

#[test]
fn spec_header_only_split_follows_declared_first_fragment() {
    let core = open(SPEC);
    assert_eq!(core.page_count(), 71, "한/글 PDF 쪽수");
    // 표 199(글머리표, 7행): 선언 높이가 7행 전체를 덮는다 — 한/글은 32쪽 바닥에 제목 행만
    // 남기지 않고 33쪽에 통째로 둔다(첫 조각 예약 = 바깥 아래 여백 + 아래 캡션).
    assert_eq!(table_pages(&core, 199), vec![32], "표 199 는 33쪽에만");
    // 표 334: 선언 높이가 제목 행 하나 — 한/글도 51쪽에 제목 행, 52쪽에 나머지를 둔다.
    assert_eq!(
        table_pages(&core, 334),
        vec![50, 51],
        "표 334 는 51·52쪽에 나뉜다"
    );
}

/// 쪽 `page`(0부터)에서 구역 `section` 문단 `para` 첫 글줄의 위끝과, 같은 쪽 단 위끝 + 그 문단의
/// 저장 첫 줄 vpos. 한/글 저장 사다리는 쪽마다 vpos 를 0 에서 다시 센다.
fn first_line_top_and_stored(
    core: &DocumentCore,
    page: u32,
    section: usize,
    para: usize,
) -> (f64, f64) {
    let source = &core.document().sections[section].paragraphs[para];
    let head: String = source.text.chars().take(12).collect();
    let stored = source.line_segs.first().expect("저장 첫 줄").vertical_pos as f64 * 96.0 / 7200.0;
    let tree = core.build_page_render_tree(page).expect("쪽 배치");
    let mut column_top = None;
    let mut line_top = None;
    let mut stack = vec![&tree.root];
    while let Some(node) = stack.pop() {
        match &node.node_type {
            RenderNodeType::Column(_) if column_top.is_none() => column_top = Some(node.bbox.y),
            RenderNodeType::TextLine(_) if line_top.is_none() => {
                let mut texts = Vec::new();
                line_texts(node, &mut texts);
                // 번호 문단은 줄 앞에 번호 글자가 붙는다.
                if texts.concat().contains(&head) {
                    line_top = Some(node.bbox.y);
                }
            }
            _ => {}
        }
        stack.extend(node.children.iter().rev());
    }
    (
        line_top.expect("문단 첫 글줄"),
        column_top.expect("단") + stored,
    )
}

/// 상자 뒤 앞 간격 흡수는 다음 문단의 저장 첫 줄이 «상자 끝 + 앞 간격» 을 증언하면 하지 않는다.
/// hwpspec 54쪽: 이어지는 마지막 조각(표 340) 뒤 테두리 문단 341 — 한/글 Print 와 저장 줄 모두
/// 앞 간격 13.3px 아래에 첫 줄을 둔다. 반례(흡수가 맞는 쪽): 사양서 55쪽 통째 표 348 뒤 같은
/// 모양의 문단 349, hwpspec 53쪽 이어지는 조각(표 336) 뒤 제목 337.
#[test]
fn successor_keeps_spacing_when_its_stored_line_says_so() {
    let hwpspec = open("samples/hwpspec.hwp");
    let (line, stored) = first_line_top_and_stored(&hwpspec, 53, 2, 341);
    assert!(
        (line - stored).abs() <= 1.0,
        "hwpspec 54쪽 문단 341 첫 줄 {line:.1} 이 저장 줄 {stored:.1} 에 있다"
    );
    let (line, stored) = first_line_top_and_stored(&hwpspec, 52, 2, 337);
    assert!(
        (line - stored).abs() <= 1.0,
        "hwpspec 53쪽 문단 337 첫 줄 {line:.1} 이 저장 줄 {stored:.1} 에 있다(흡수)"
    );
    let spec = open(SPEC);
    let (line, stored) = first_line_top_and_stored(&spec, 54, SPEC_SECTION, 349);
    assert!(
        (line - stored).abs() <= 1.0,
        "사양서 55쪽 문단 349 첫 줄 {line:.1} 이 저장 줄 {stored:.1} 에 있다(흡수)"
    );
}
