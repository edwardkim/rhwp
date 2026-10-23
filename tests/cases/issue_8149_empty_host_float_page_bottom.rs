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
