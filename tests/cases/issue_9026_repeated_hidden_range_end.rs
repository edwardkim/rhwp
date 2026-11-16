//! [#9026·#9044·#9028] 관계형·고정형 계약 — 반복·짝 없는 숨김 범위 끝 표지가 있는 HWPX 를 열고,
//! 숨김 삭제 사슬의 저장 줄과 꼬리 조각의 저장 줄 위치를 따른다.
//!
//! 주석 갱신: 2026-11-14 08:53 KST(+09:00). 직전 파일 변경 커밋 45e210f5bb
//! (2026-11-14 04:54:25 +0900).
//!
//! 유형: 쪽수(5쪽·8쪽)와 사슬 줄 개수는 고정형(독립 한컴 Print·원본 XML 기대값), 쪽·문단 소속과
//! 줄 순서, 그림 위끝과 칸 조각 위끝의 저장값 관계는 관계형이다. 절대 px 좌표는 고정하지 않는다.
//!
//! 시각 측정(2026-11-14, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/hwp`·`ttfs/windows`):
//! - `samples/issue9026/press_release_repeated_hidden_end.hwpx` 5쪽 = 한컴 5쪽.
//!   Native 99.82·99.81·99.49·99.85·99.56(최저 99.49), fresh WASM 99.99·99.99·99.98·99.99·99.69
//!   (최저 99.69). 4쪽은 수정 전 Native 74.27.
//! - `samples/issue9026/press_release_orphan_hidden_end.hwpx` 8쪽 = 한컴 8쪽.
//!   Native 99.15·94.96·99.93·99.91·100.00·99.36·96.49·99.96(최저 94.96), fresh WASM
//!   99.81·95.49·99.99·99.91·100.00·99.40·96.17·99.96(최저 95.49). 수정 전 Native 3쪽 32.79·4쪽 6.47.
//! - 증거: `mydocs/pr/assets/issue_9026/`(전쪽 Native·fresh WASM TSV, 대표 review·overlay PNG).
//!
//! - 입력: 공개 보도자료 `samples/issue9026/press_release_repeated_hidden_end.hwpx`
//!   (질병관리청 156594737, rhwp-issue-docs#1088). 한 문단에서 숨김 삭제 범위
//!   `deleteBegin Id="7" TcId="3"` 뒤 `deleteEnd Id="7"` 가 run 분할 지점에 두 번 저장되어 있다.
//! - 기준: `pdf/issue9026/press_release_repeated_hidden_end-print-2022.pdf`
//!   (Hwp 2022 12.0.0.4547 Print, Hancom PDF 1.3.0.547) 5쪽. 삭제 범위는 숨겨지고 문단은
//!   「박현영 국립보건연구원장은 …」으로 시작한다.
//! - 수정 전 rhwp(devel `31a4225757`)는 `tracking structure: hidden range end without begin`
//!   으로 문서 전체를 거부했다.
//!
//! - 둘째 입력: 공개 보도자료 `samples/issue9026/press_release_orphan_hidden_end.hwpx`
//!   (2026 돌봄로봇 네트워크 포럼, korea.kr 배포). 반복 끝(deleteEnd 24·25·127, insertEnd
//!   120·138·150)에 더해, 문서의 어느 XML 에도 시작 표지가 없는 숨김 삭제 끝(deleteEnd 103·107·126,
//!   TcId 139, `paraend="0"`)이 있다. 기준 `pdf/issue9026/press_release_orphan_hidden_end-hwpx-2020.pdf`
//!   (로컬 한컴오피스 2022 Print, Hwp 2022 12.0.0.4547) 8쪽은 그 끝 표지 앞 글(「다섯 번째 세션부터
//!   아홉 번째 세션까지 …」)을 그대로 출력한다. 열린 적 없는 끝은 아무것도 감싸지 않는다.
//!
//! 검사: 문서 열기 성공, 전체 쪽수 = 한컴 Print(5쪽·8쪽), 숨김 삭제 범위 내용(「미래의료연구부장」,
//! 「실제 돌봄 현장의 특성을 고려하고」)이 본문에 없고, 삭제 범위 뒤 문단 머리·삽입 글·짝 없는 끝
//! 앞 글이 한컴과 같이 남는다. 숨김 삭제 사슬의 저장 줄(#9044)과 꼬리 조각의 저장 줄 위치(#9028)는
//! 아래 각 검사의 주석에 근거를 적었다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::path::PathBuf;

const SOURCE: &str = "samples/issue9026/press_release_repeated_hidden_end.hwpx";
/// 한컴 2022 Print 의 전체 쪽수.
const PRINT_PAGES: u32 = 5;
/// 숨김 삭제 범위(Id 7) 안에만 있는 원문 조각.
const HIDDEN: &str = "미래의료연구부장";
/// 삭제 범위 뒤에 남는 문단 머리(한컴 Print 의 문단 시작).
const KEPT_HEAD: &str = "박현영국립보건연구원장은";

fn ink(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn texts(paragraphs: &[Paragraph]) -> Vec<String> {
    paragraphs.iter().map(|p| ink(&p.text)).collect()
}

#[test]
fn repeated_end_of_a_closed_hidden_range_still_opens_the_document() {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).expect(SOURCE);
    let document = rhwp::parse_document(&bytes).expect("반복 끝 표지가 있어도 문서를 연다");
    let paragraphs = texts(&document.sections[0].paragraphs);

    assert!(
        paragraphs.iter().all(|text| !text.contains(HIDDEN)),
        "숨김 삭제 범위 내용은 본문에 남지 않는다"
    );
    assert!(
        paragraphs.iter().any(|text| text.starts_with(KEPT_HEAD)),
        "삭제 범위 뒤 내용은 한컴 Print 와 같은 문단 머리로 남는다"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), PRINT_PAGES, "한컴 Print 의 전체 쪽수");
}

/// 문서 전체에 시작 표지가 없는 숨김 삭제 끝 표지(Id 103·107·126)와 반복 끝을 함께 가진 공개 보도자료.
const ORPHAN_SOURCE: &str = "samples/issue9026/press_release_orphan_hidden_end.hwpx";
/// 한컴 2022 Print 의 전체 쪽수.
const ORPHAN_PRINT_PAGES: u32 = 8;
/// 숨김 삭제 범위(Id 124) 안에만 있는 원문 조각.
const ORPHAN_HIDDEN: &str = "실제돌봄현장의특성을고려하고";
/// 같은 자리의 삽입 범위(Id 118) 글.
const ORPHAN_INSERTED: &str = "의료와복지가동시에필요한돌봄현장";
/// 짝 없는 끝(Id 103) 앞에 있는 같은 문단의 글. 한컴 Print 는 이 글을 숨기지 않는다.
const ORPHAN_KEPT_BEFORE_END: &str = "다섯번째세션부터아홉번째세션까지";

#[test]
fn end_of_a_never_opened_hidden_range_hides_nothing() {
    let bytes = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ORPHAN_SOURCE))
        .expect(ORPHAN_SOURCE);
    let document = rhwp::parse_document(&bytes).expect("짝 없는 끝 표지가 있어도 문서를 연다");
    let paragraphs = texts(&document.sections[0].paragraphs);

    assert!(
        paragraphs.iter().all(|text| !text.contains(ORPHAN_HIDDEN)),
        "열린 숨김 삭제 범위 내용은 본문에 남지 않는다"
    );
    assert!(
        paragraphs.iter().any(|text| text.contains(ORPHAN_INSERTED)),
        "삽입 범위 글은 남는다"
    );
    assert!(
        paragraphs
            .iter()
            .any(|text| text.contains(ORPHAN_KEPT_BEFORE_END)),
        "짝 없는 끝 표지 앞 글은 숨기지 않는다(한컴 Print 와 같음)"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        ORPHAN_PRINT_PAGES,
        "한컴 Print 의 전체 쪽수"
    );
}

fn paragraph_lines(node: &RenderNode, para: usize, out: &mut Vec<(f64, f64)>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.section_index == Some(0) && line.para_index == Some(para) {
            out.push((node.bbox.y, node.bbox.height));
        }
    }
    for child in &node.children {
        paragraph_lines(child, para, out);
    }
}

fn lines_on_page(core: &DocumentCore, page: u32, para: usize) -> Vec<(f64, f64)> {
    let tree = core.build_page_render_tree(page).expect("쪽 렌더 트리");
    let mut lines = Vec::new();
    paragraph_lines(&tree.root, para, &mut lines);
    lines
}

/// [#9044] 저장 줄이 정한 사슬의 줄 경계.
///
/// 로고 표 문단(원본 XML 문단 37) 뒤의 `merged="1"` 문단들은 앞 문단 끝이 숨김 삭제가 아니다
/// (문단 37 에는 범위 표지가 없다). 한컴은 그 사슬의 저장 줄을 문단 37 의 줄 배열에 두 줄
/// (`textpos` 0·159)로 적었고, 둘째 줄은 숨김 내용 뒤 첫 보이는 자리(문단 69 의 시작)에서 시작한다.
/// 한컴 2022 로 저장 줄을 지우고 다시 저장한 변형에서도 같은 두 줄이 나온다. 숨김 삭제된
/// 문단 끝(`paraend="1"`)만 다음 문단을 같은 줄로 잇는다.
///
/// 관계: 로고 표 문단 바로 뒤에 빈 문단이 하나 있고, 그 줄은 같은 쪽에서 표 문단 아래·다음 빈
/// 문단 위에 놓인다. 그 줄이 없으면 「붙임 1」 제목 표가 3쪽에 남는다(한컴은 4쪽 맨 위).
#[test]
fn visible_mark_before_a_merged_chain_keeps_the_stored_line() {
    let bytes = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ORPHAN_SOURCE))
        .expect(ORPHAN_SOURCE);
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let paragraphs = &core.document().sections[0].paragraphs;
    let has_table = |index: usize| {
        paragraphs[index]
            .controls
            .iter()
            .any(|control| matches!(control, Control::Table(_)))
    };
    let attachment = paragraphs
        .iter()
        .position(|p| ink(&p.text).starts_with("<붙임>1.2026"))
        .expect("붙임 목록 첫 줄");
    let logo = (attachment..paragraphs.len())
        .filter(|&index| has_table(index))
        .nth(1)
        .expect("담당자 표 다음의 로고 표 문단");
    let title = (logo + 1..paragraphs.len())
        .find(|&index| has_table(index))
        .expect("「붙임 1」 제목 표 문단");
    let chain_line = logo + 1;
    assert!(
        paragraphs[chain_line].text.is_empty() && paragraphs[chain_line].controls.is_empty(),
        "사슬의 둘째 저장 줄은 보이는 글이 없는 빈 문단이다"
    );
    assert_eq!(
        title - logo - 1,
        17,
        "로고 표와 제목 표 사이의 빈 줄은 사슬 줄 1개와 삽입된 빈 문단 16개다"
    );

    // 「<붙임> 1.」 목록은 2쪽 끝의 빈 줄 다음 쪽에서 시작한다(한컴 3쪽 맨 위).
    assert!(lines_on_page(&core, 1, attachment).is_empty());
    assert_eq!(lines_on_page(&core, 1, attachment - 1).len(), 1);
    assert_eq!(lines_on_page(&core, 2, attachment).len(), 1);

    let chain = lines_on_page(&core, 2, chain_line);
    let next = lines_on_page(&core, 2, chain_line + 1);
    assert_eq!(chain.len(), 1, "사슬 줄은 로고 표와 같은 쪽에 있다");
    assert_eq!(next.len(), 1);
    assert!(chain[0].0 + chain[0].1 <= next[0].0 + 0.5);

    // 제목 표 뒤 첫 글 문단은 한컴과 같이 4쪽에 있다.
    let after_title = (title + 1..paragraphs.len())
        .find(|&index| !paragraphs[index].text.trim().is_empty())
        .expect("제목 표 뒤 첫 글 문단");
    assert!(lines_on_page(&core, 2, after_title).is_empty());
    assert_eq!(lines_on_page(&core, 3, after_title).len(), 1);
}

fn find_nodes<'a>(
    node: &'a RenderNode,
    pick: &dyn Fn(&RenderNode) -> bool,
    out: &mut Vec<&'a RenderNode>,
) {
    if pick(node) {
        out.push(node);
    }
    for child in &node.children {
        find_nodes(child, pick, out);
    }
}

/// [#9028] 남은 내용 전체를 담은 꼬리 조각은 저장 줄 위치에서 시작한다.
///
/// 세로 가운데 정렬 1×1 표(`pageBreak="CELL"`)의 4쪽 조각은 저장 쪽 프레임이 되감긴 빈 문단
/// (저장 vpos 0)에서 시작해 칸 끝까지 담는다. 조각 상자가 그 유닛 점유만으로 닫히므로 나눌
/// 여분이 없고, 한컴 2022 Print 는 그림을 저장 vpos(2240HU) 그대로 칸 내용 시작 아래에 둔다.
///
/// 관계: 4쪽 그림의 위끝 − 칸 조각 위끝 = 칸 위 안여백 + 그림 문단의 저장 vpos. 기대값은 원본
/// 저장값에서 계산하며 절대 좌표를 고정하지 않는다. 수정 전에는 줄간격의 절반(5.6px)이 더해졌다.
#[test]
fn complete_suffix_fragment_starts_at_the_stored_line_position() {
    let bytes =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).expect(SOURCE);
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let paragraphs = &core.document().sections[0].paragraphs;
    let (host, table) = paragraphs
        .iter()
        .enumerate()
        .find_map(|(index, paragraph)| {
            paragraph.controls.iter().find_map(|control| match control {
                Control::Table(table)
                    if table.cells.len() == 1
                        && table.cells[0].paragraphs.iter().any(|p| {
                            p.controls.iter().any(|c| matches!(c, Control::Picture(_)))
                        }) =>
                {
                    Some((index, table))
                }
                _ => None,
            })
        })
        .expect("그림을 담은 1×1 표");
    let cell = &table.cells[0];
    let picture_para = cell
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Picture(_))))
        .expect("그림 문단");
    let stored_vpos = cell.paragraphs[picture_para].line_segs[0].vertical_pos;
    assert_eq!(
        cell.paragraphs[picture_para - 1].line_segs[0].vertical_pos,
        0,
        "그림 앞 빈 문단에서 저장 쪽 프레임이 되감긴다"
    );
    let expected = rhwp::renderer::hwpunit_to_px(i32::from(cell.padding.top) + stored_vpos, 96.0);

    let tree = core.build_page_render_tree(3).expect("4쪽");
    let mut cells = Vec::new();
    find_nodes(
        &tree.root,
        &|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 0 && cell.col == 0 && cell.page_fragment),
        &mut cells,
    );
    let mut pictures = Vec::new();
    find_nodes(
        &tree.root,
        &|node| matches!(&node.node_type, RenderNodeType::Image(image) if image.para_index == Some(host) || image.section_index == Some(0)),
        &mut pictures,
    );
    let fragment = cells.first().expect("4쪽의 칸 조각");
    let picture = pictures
        .iter()
        .filter(|node| {
            node.bbox.y >= fragment.bbox.y
                && node.bbox.y + node.bbox.height <= fragment.bbox.y + fragment.bbox.height + 0.5
                && node.bbox.height > 100.0
        })
        .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .expect("칸 조각 안의 그림");
    let offset = picture.bbox.y - fragment.bbox.y;
    assert!(
        (offset - expected).abs() <= 1.0,
        "그림은 칸 내용 시작 + 저장 vpos 에 놓인다: offset={offset:.2}px expected={expected:.2}px"
    );
}
