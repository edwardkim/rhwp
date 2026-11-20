//! 관계형·고정형 — [#9239] 캡션 문단에 걸린 떠 있는 표를 그리고, 나눌 수 없는 한 행 자리차지 표는 저장된 다음 쪽으로 통째 넘긴다.
//! 주석 갱신: 2026-11-20 22:27 KST (새 파일)
//!
//! 입력: 국민권익위원회 공개 문서(「고령자 취업지원 제도개선」, HWP5 5.0.2.4, 62쪽)
//! `samples/issue9239/senior-employment-policy-caption-float-table.hwp`.
//! 기준: 한컴 Print PDF `pdf/issue9239/senior-employment-policy-caption-float-table-2020.pdf`(62쪽).
//!
//! 43쪽 — 문단 519 는 글자처럼 취급 표(21×7) 하나를 가졌고 그 표의 위 캡션이 285문단이다. 캡션
//! 첫 문단에 자리차지 표(49×12 「3. 시·군·구별 고령인구」, 글자처럼 취급 아님)가 걸려 있다. 한/글은
//! 이 캡션 안 표를 쪽 맨 위에 그린다. 바깥 표는 캡션(5408px) 아래라 한/글에서도 쪽 밖이다(한컴
//! PDF 전체에 바깥 표 글자가 없다). 수정 전 rhwp 는 캡션 문단의 글줄과 자리차지 그림만 그려 쪽이
//! 비었다.
//!
//! 11→12쪽 — 문단 179 는 빈 문단에 걸린 1×2 자리차지 표(3580HU, 바깥 여백 위·아래 283HU)다.
//! 다음 문단 180 의 저장 줄은 쪽을 다시 시작해 `vpos=4146`(= 283 + 3580 + 283)에서 시작한다 —
//! 한/글이 표 전체를 12쪽 맨 위에 뒀다는 기록이다. 수정 전 rhwp 는 첫 행 내용(23.8px)이 11쪽 남은
//! 27.7px 에 든다는 이유로 표를 11쪽 끝 각주 영역 위에 겹쳐 그렸다.
//! - 고정형: 전체 쪽수 62(= PDF).
//! - 관계형: 43쪽에 캡션 안 표가 정확히 하나 있고 쪽 상자 안에 들며 바깥 표보다 위에 있다. 문단
//!   179 의 표는 11쪽에 없고 12쪽에서 문단 181 표보다 위에 있다. 11쪽의 본문 표는 각주 영역과
//!   겹치지 않는다.
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = upstream/devel `411a9f944c`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 62쪽 / 62쪽, 11쪽 99.51(base 94.67), 12쪽 95.21(base 35.89), 43쪽 99.42(base 0.00), 전쪽 최저 92.03(34쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 62쪽 / 62쪽, 11쪽 99.90, 12쪽 95.82, 43쪽 99.23, 전쪽 최저 91.89(33쪽)
//! - 증거: `mydocs/pr/assets/issue_9239/`(native·wasm 전쪽 TSV, 12·43쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9239/senior-employment-policy-caption-float-table.hwp";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 62;
/// 위 캡션을 가진 글자처럼 취급 표의 문단.
const CAPTION_OWNER_PARA: usize = 519;
/// 다음 쪽으로 통째 넘어가는 1×2 제목 표의 문단.
const TITLE_TABLE_PARA: usize = 179;
/// 제목 표 뒤 12쪽의 첫 글자처럼 취급 표 문단.
const FOLLOWING_TABLE_PARA: usize = 181;

struct TableBox {
    rows: u16,
    cols: u16,
    para: Option<usize>,
    bbox: BoundingBox,
    text: String,
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn collect_tables(
    node: &RenderNode,
    tables: &mut Vec<TableBox>,
    footnote: &mut Option<BoundingBox>,
) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            let mut text = String::new();
            collect_text(node, &mut text);
            tables.push(TableBox {
                rows: table.row_count,
                cols: table.col_count,
                para: table.para_index,
                bbox: node.bbox,
                text,
            });
        }
        RenderNodeType::FootnoteArea => {
            *footnote = Some(node.bbox);
            return;
        }
        _ => {}
    }
    for child in &node.children {
        collect_tables(child, tables, footnote);
    }
}

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );
    core
}

#[test]
fn caption_paragraph_float_table_is_drawn_once_above_its_owner_table() {
    let core = open();
    let page = core.build_page_render_tree(42).expect("43쪽");
    let (mut tables, mut footnote) = (Vec::new(), None);
    collect_tables(&page.root, &mut tables, &mut footnote);

    let owner = tables
        .iter()
        .find(|t| t.para == Some(CAPTION_OWNER_PARA) && (t.rows, t.cols) == (21, 7))
        .expect("43쪽에 문단 519 의 바깥 표가 있어야 한다");
    let caption_tables: Vec<_> = tables
        .iter()
        .filter(|t| (t.rows, t.cols) == (49, 12) && t.text.contains("고령인구"))
        .collect();
    assert_eq!(
        caption_tables.len(),
        1,
        "캡션 문단의 자리차지 표(49×12)는 43쪽에 정확히 한 번 그려져야 한다"
    );
    let caption_table = caption_tables[0];
    let page_box = page.root.bbox;
    let bottom = caption_table.bbox.y + caption_table.bbox.height;
    assert!(
        caption_table.bbox.y >= page_box.y && bottom <= page_box.y + page_box.height,
        "캡션 안 표 {:.2}~{bottom:.2} 가 쪽 상자 {:.2}~{:.2} 안에 있어야 한다",
        caption_table.bbox.y,
        page_box.y,
        page_box.y + page_box.height
    );
    assert!(
        bottom <= owner.bbox.y + 0.5,
        "위 캡션의 표 아래 {bottom:.2} 는 바깥 표 위 {:.2} 보다 위여야 한다",
        owner.bbox.y
    );
    assert!(
        caption_table.text.contains("고흥군") && caption_table.text.contains("주민등록인구"),
        "캡션 안 표의 칸 글이 그려져야 한다"
    );
}

#[test]
fn unsplittable_single_row_float_table_moves_whole_to_the_stored_next_page() {
    let core = open();

    let previous = core.build_page_render_tree(10).expect("11쪽");
    let (mut tables, mut footnote) = (Vec::new(), None);
    collect_tables(&previous.root, &mut tables, &mut footnote);
    assert!(
        tables.iter().all(|t| t.para != Some(TITLE_TABLE_PARA)),
        "문단 179 의 제목 표는 11쪽에 남지 않아야 한다"
    );
    let footnote = footnote.expect("11쪽에 각주 영역이 있어야 한다");
    assert!(!tables.is_empty(), "11쪽에 본문 표가 있어야 한다");
    for table in &tables {
        let bottom = table.bbox.y + table.bbox.height;
        assert!(
            bottom <= footnote.y + 0.5,
            "11쪽 표(문단 {:?}) 아래 {bottom:.2} 가 각주 영역 위 {:.2} 를 넘는다",
            table.para,
            footnote.y
        );
    }

    let next = core.build_page_render_tree(11).expect("12쪽");
    let (mut tables, mut footnote) = (Vec::new(), None);
    collect_tables(&next.root, &mut tables, &mut footnote);
    let title = tables
        .iter()
        .find(|t| t.para == Some(TITLE_TABLE_PARA))
        .expect("문단 179 의 제목 표는 12쪽에 있어야 한다");
    assert!(
        title.text.contains("문제점 및 개선방안"),
        "제목 표의 글이 12쪽에 그려져야 한다"
    );
    let following = tables
        .iter()
        .find(|t| t.para == Some(FOLLOWING_TABLE_PARA))
        .expect("문단 181 의 표는 12쪽에 있어야 한다");
    let title_bottom = title.bbox.y + title.bbox.height;
    assert!(
        title_bottom <= following.bbox.y + 0.5,
        "제목 표 아래 {title_bottom:.2} 는 문단 181 표 위 {:.2} 보다 위여야 한다",
        following.bbox.y
    );
    for table in tables.iter().filter(|t| t.para != Some(TITLE_TABLE_PARA)) {
        assert!(
            table.bbox.y + 0.5 >= title_bottom,
            "12쪽 표(문단 {:?}) 위 {:.2} 가 제목 표 아래 {title_bottom:.2} 보다 위다",
            table.para,
            table.bbox.y
        );
    }
}
