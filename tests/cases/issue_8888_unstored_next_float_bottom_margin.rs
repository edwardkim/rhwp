//! 관계형·고정형 — [#8888] 빈 앵커 자리차지 표 뒤 저장 줄 없는 문단과 표 아래 바깥 여백.
//! 주석 갱신: 2026-11-10 12:52 KST (새 파일)
//!
//! 입력: 공개 결재문서 36384898(한컴오피스 2020 저장) 원본. `pi=0` 은 글자 없는 앵커(누름틀만)와
//! 문단 기준 자리차지 6×3 표(높이 16653HU, 바깥 여백 위 140·아래 852HU, 쪽나눔 None)를 갖고,
//! host 줄은 vpos 0·폭 0 으로 저장돼 있다. `pi=1`·`pi=2` 는 저장 줄(`linesegarray`)이 없다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8888/36384898-unstored-next-float-bottom-margin-2020.pdf`(2쪽).
//! - 고정형: 전체 쪽수 2.
//! - 관계형: `pi=1` 줄 위끝 − 표 아래끝 = 표 아래 바깥 여백 852HU. 한컴은 `pi=1` 을
//!   0 + 140 + 16653 + 852 = 17645HU 에 둔다(PDF 「1.」 글자 위끝 311.5px).
//! - 관계형: `pi=2` 줄 위끝 − `pi=1` 줄 위끝 = `pi=1` 줄 전진 + `pi=2` 문단 앞 간격(1300 + 780 + 1000HU).
//!
//! 반례: `pi=1` 에 한컴 값(vertpos 17645)으로 저장 줄을 넣은 변형. 저장 사다리가 여백을 복원하므로
//! 아래 여백이 두 번 들어가지 않고 같은 관계를 지킨다.
//!
//! 측정(2026-11-10, 코드 head c9548826a1, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, 문서 face 굴림·맑은 고딕 공급):
//! - Native: 2/2쪽, p1 99.73 · p2 100.00, 전쪽 최저 99.73.
//! - fresh WASM: 2/2쪽, p1 100.00 · p2 100.00, 전쪽 최저 100.00.
//! - 수정 전 Native(devel b5064fae1e 코드): p1 32.34 · p2 100.00.
//! - 증거: `mydocs/pr/assets/issue_8888/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8888/36384898-unstored-next-float-bottom-margin.hwpx";
const TOLERANCE_PX: f64 = 0.5;
const PARA1_TAIL: &str = r#"<hp:run charPrIDRef="3"><hp:t>와 관련한 내용입니다.</hp:t></hp:run><hp:run charPrIDRef="2"/></hp:p>"#;
const PARA1_STORED_LINE: &str = r#"<hp:linesegarray><hp:lineseg textpos="0" vertpos="17645" vertsize="1300" textheight="1300" baseline="1105" spacing="780" horzpos="0" horzsize="49608" flags="393216"/></hp:linesegarray>"#;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// `pi=1` 에 한컴 값으로 저장 줄을 넣은 HWPX.
fn with_stored_para1_line() -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(sample_bytes())).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().expect("name").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("utf-8");
            assert_eq!(xml.matches(PARA1_TAIL).count(), 1, "원본 pi=1 끝");
            let stored = PARA1_TAIL.replace("</hp:p>", &format!("{PARA1_STORED_LINE}</hp:p>"));
            data = xml.replace(PARA1_TAIL, &stored).into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        out.start_file(name, options).expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

fn body_line_top(page: &RenderNode, para: usize) -> f64 {
    nodes(page)
        .into_iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.section_index == Some(0)
                    && run.para_index == Some(para)
                    && run.cell_context.is_none() =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("1쪽 pi={para} 글줄"))
}

fn header_table_bottom(page: &RenderNode) -> f64 {
    let table = nodes(page)
        .into_iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(0) && table.control_index == Some(2))
        })
        .expect("1쪽 머리 표 pi=0 ci=2");
    table.bbox.y + table.bbox.height
}

fn assert_flow_after_header_table(bytes: &[u8], label: &str) {
    let core = DocumentCore::from_bytes(bytes).unwrap_or_else(|e| panic!("open {label}: {e:?}"));
    assert_eq!(
        core.page_count(),
        2,
        "{label}: 한컴 engine 2020 Print PDF 는 2쪽"
    );
    let page = core.build_page_render_tree(0).expect("1쪽");
    let table_bottom = header_table_bottom(&page.root);
    let para1 = body_line_top(&page.root, 1);
    let para2 = body_line_top(&page.root, 2);

    let margin = hu_to_px(852);
    let gap = para1 - table_bottom;
    assert!(
        (gap - margin).abs() <= TOLERANCE_PX,
        "{label}: pi=1 글줄이 머리 표 아래끝에서 {gap:.2}px 떨어졌다 — 표 아래 바깥 여백 {margin:.2}px 이어야 한다"
    );
    let step = hu_to_px(1300 + 780 + 1000);
    let advance = para2 - para1;
    assert!(
        (advance - step).abs() <= TOLERANCE_PX,
        "{label}: pi=2 글줄이 pi=1 에서 {advance:.2}px 아래다 — 줄 전진 + 문단 앞 간격 {step:.2}px 이어야 한다"
    );
}

#[test]
fn unstored_next_paragraph_starts_below_table_bottom_margin() {
    assert_flow_after_header_table(&sample_bytes(), "원본");
}

#[test]
fn stored_next_paragraph_does_not_take_the_bottom_margin_twice() {
    assert_flow_after_header_table(&with_stored_para1_line(), "pi=1 저장 줄 변형");
}
