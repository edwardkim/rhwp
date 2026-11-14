//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-15 08:02:09 KST (Asia/Seoul).
//! 직전 파일 변경: f899c873d6989eee1d556cfbc3455de4ac8854c7 2026-11-15T04:27:06+09:00.
//!
//! 입력: `samples/issue9060/86036-rowbreak-first-line-fragment.hwpx`.
//! 입력 SHA-256: f224d490226b3673554b484d6ce1e2b1c3bbaf0aac02340706edf5fc9d0a4025.
//! 독립 Print: `pdf/issue9060/86036-rowbreak-first-line-fragment-2020.pdf`, 14쪽.
//! PDF SHA-256: c9ba8e9d4b4e2cb32277d030161dedb59d9067eba0a4665b40f9c0b46cc47317.
//! PDF Creator는 Hwp 2022 12.0.0.4547, Producer는 Hancom PDF 1.3.0.547이다.
//! 파일명·원 PR의 engine 2020 설명과 구분하고 기존 PDF를 재인쇄하지 않았다.
//!
//! 측정일: 2026-11-15. 소스 43adf3f7fe412e71797456887e2be44b8ef8d14f.
//! 방법: Visual Sweep, compat 2022·Print·96 DPI·색상 경계 보정 전 2px raw.
//! Native: 전 14쪽 최저 100.00%, 대상 5·6쪽 각각 100.00%.
//! fresh WASM: 전 14쪽 최저 100.00%, 대상 5·6쪽 각각 100.00%.
//! Native binary SHA-256: 5e9c68af4bb8b902b56b5a80678156da193bbcd983080a7b804e0d16ca1dccd6.
//! WASM binary SHA-256: b2fd15c785c7201c63b5c1942f47ae2b244db9c303614338638daa439d1b9f3c.
//! 시각 측정은 실제 공급 프로그램·명시적 글꼴 환경으로 수행했다.
//! 증거: `mydocs/pr/assets/issue_9060/final/main/`의 두 경로 전쪽 TSV·5·6쪽 review/overlay,
//! `conditions.md` 및 `mydocs/pr/assets/issue_9060/verification-record.md`.
//!
//! 고정형은 독립 PDF의 전체 쪽수 14와 원본 저장 줄의 vpos 0·0·2252다.
//! 관계형은 제목·표 조각·뒤 본문의 소유와 순서, 셀 전체 내용의 누락·중복 방지다.
//! 첫 조각의 소유 줄은 5쪽 1개, 나머지는 6쪽 2개이며 후속 제목은 7쪽에 한 번 존재한다.
//! 실물의 절대 px 좌표를 고정하지 않으며 회귀 실행에 설치된 OTF·시스템 글꼴을 요구하지 않는다.
//! 수정 전 devel은 15쪽으로 독립 PDF와 달랐다. 기존 세 검사에 내용 보존 검사를 새 이름으로 더한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9060/86036-rowbreak-first-line-fragment.hwpx";
const TABLE_PARA: usize = 41;

fn document() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

/// 표 밖 본문 글줄(문단, 위끝)과 최상위 표(문단, 위끝, 아래끝).
fn collect(
    node: &RenderNode,
    in_table: bool,
    lines: &mut Vec<(usize, f64)>,
    tables: &mut Vec<(usize, f64, f64)>,
) {
    let mut inside = in_table;
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index.filter(|_| !in_table) {
                tables.push((para, node.bbox.y, node.bbox.y + node.bbox.height));
            }
            inside = true;
        }
        RenderNodeType::TextLine(line) if !in_table => {
            if let Some(para) = line.para_index {
                lines.push((para, node.bbox.y));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, inside, lines, tables);
    }
}

struct Page {
    lines: Vec<(usize, f64)>,
    tables: Vec<(usize, f64, f64)>,
}

impl Page {
    fn of(core: &DocumentCore, page: u32) -> Self {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1));
        let (mut lines, mut tables) = (Vec::new(), Vec::new());
        collect(&tree.root, false, &mut lines, &mut tables);
        Self { lines, tables }
    }

    fn table(&self, para: usize) -> Option<(f64, f64)> {
        self.tables
            .iter()
            .find(|(p, _, _)| *p == para)
            .map(|(_, top, bottom)| (*top, *bottom))
    }

    fn line_top(&self, para: usize) -> Option<f64> {
        self.lines
            .iter()
            .filter(|(p, _)| *p == para)
            .map(|(_, y)| *y)
            .min_by(|a, b| a.total_cmp(b))
    }
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(
        document().page_count(),
        14,
        "첨부 독립 한컴 Print PDF는 14쪽"
    );
}

/// 원본 전제: 칸 문단의 저장 줄이 첫 줄 뒤에서 쪽을 다시 시작한다.
#[test]
fn source_cell_lines_restart_the_page_after_the_first_line() {
    let core = document();
    let host = &core.document().sections[0].paragraphs[TABLE_PARA];
    let [Control::Table(table)] = host.controls.as_slice() else {
        panic!("문단 {TABLE_PARA} 컨트롤: {}", host.controls.len());
    };
    assert_eq!((table.row_count, table.col_count), (1, 1));
    assert!(!table.common.treat_as_char);
    let vpos: Vec<i32> = table.cells[0].paragraphs[0]
        .line_segs
        .iter()
        .map(|seg| seg.vertical_pos)
        .collect();
    assert_eq!(vpos, [0, 0, 2252]);
}

/// [#9060] 첫 줄 조각은 5쪽 아래에, 나머지는 6쪽 위에 놓인다.
#[test]
fn first_line_fragment_stays_at_the_bottom_of_the_page() {
    let core = document();
    let fifth = Page::of(&core, 4);
    let (first_top, _) = fifth
        .table(TABLE_PARA)
        .expect("5쪽에 문단 41 표의 첫 조각이 있다");
    let heading = fifth
        .line_top(39)
        .expect("5쪽에 문단 39 「3. 규제목표」가 있다");
    assert!(
        heading < first_top,
        "「3. 규제목표」 위끝 {heading:.2} < 표 첫 조각 위끝 {first_top:.2}"
    );

    let sixth = Page::of(&core, 5);
    let (rest_top, _) = sixth
        .table(TABLE_PARA)
        .expect("6쪽에 문단 41 표의 나머지 조각이 있다");
    // 나머지 조각이 6쪽 본문을 연다: 표 앞 문단(39·40)은 5쪽에 남고 6쪽에서 표 위에 오지 않는다.
    assert!(
        sixth.line_top(39).is_none(),
        "「3. 규제목표」는 5쪽에만 있다"
    );
    assert!(
        sixth
            .lines
            .iter()
            .filter(|(para, _)| *para > TABLE_PARA)
            .all(|(_, y)| rest_top <= *y),
        "6쪽의 표 뒤 본문 글줄은 표 나머지 조각 아래에 있다"
    );
    assert!(
        Page::of(&core, 6).table(TABLE_PARA).is_none(),
        "표는 7쪽까지 이어지지 않는다"
    );
}

// 원 PR의 위치 관계 검사를 유지하면서 두 조각의 실제 내용과 뒤 본문까지 보호한다.
// 기대 문자열은 원본 셀과 독립 Print 5·6쪽, 뒤 제목은 독립 Print 7쪽에서 확인했다.
#[test]
fn split_fragments_preserve_all_source_text_and_following_heading() {
    fn text_and_lines(node: &RenderNode, text: &mut String, lines: &mut usize) {
        match &node.node_type {
            RenderNodeType::TextRun(run) => text.push_str(&run.text),
            RenderNodeType::TextLine(_) => *lines += 1,
            _ => {}
        }
        for child in &node.children {
            text_and_lines(child, text, lines);
        }
    }
    fn ownership(
        node: &RenderNode,
        in_table: bool,
        page: u32,
        fragments: &mut Vec<(u32, usize, String)>,
        following: &mut Vec<(u32, String)>,
    ) {
        let mut inside = in_table;
        match &node.node_type {
            RenderNodeType::Table(table) => {
                if !in_table && table.para_index == Some(TABLE_PARA) {
                    let (mut text, mut lines) = (String::new(), 0);
                    text_and_lines(node, &mut text, &mut lines);
                    assert!(!text.is_empty(), "빈 조각은 내용 보존 증거가 아니다");
                    fragments.push((page, lines, text));
                }
                inside = true;
            }
            RenderNodeType::TextLine(line) if !in_table && line.para_index == Some(43) => {
                let (mut text, mut lines) = (String::new(), 0);
                text_and_lines(node, &mut text, &mut lines);
                following.push((page, text));
            }
            _ => {}
        }
        for child in &node.children {
            ownership(child, inside, page, fragments, following);
        }
    }
    let core = document();
    let (mut fragments, mut following) = (Vec::new(), Vec::new());
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .expect("원본의 실제 출력 트리");
        ownership(&tree.root, false, page + 1, &mut fragments, &mut following);
    }
    assert_eq!(
        fragments
            .iter()
            .map(|(page, lines, _)| (*page, *lines))
            .collect::<Vec<_>>(),
        [(5, 1), (6, 2)],
        "독립 Print처럼 첫 줄과 나머지 두 줄을 한 번씩 소유한다"
    );
    let expected = concat!(
        "공공재정 부정청구등 행위로 신고자가 받은 불이익조치를 해소하기 위한 ",
        "신분보장등조치결정의 실효성을 제고하기 위해 제재 수준을 상향함으로써 ",
        "신고자가 안심하고 신고할 수 있는 환경 조성"
    );
    assert_eq!(
        fragments
            .iter()
            .map(|(_, _, text)| text.as_str())
            .collect::<String>(),
        expected,
        "두 조각을 합친 내용이 원본 셀 전체와 같으며 누락·중복이 없어야 한다"
    );
    assert_eq!(
        following,
        [(7, "Ⅱ. 규제의 적정성".to_owned())],
        "후속 본문은 실제 내용을 가진 채 독립 Print 7쪽에 한 번만 놓인다"
    );
}
