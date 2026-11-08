//! 검사 유형: 관계형(들여쓰기 유무에 따른 표 조각 가로 위치의 동일성); 절대 페이지 좌표 핀 없음.
//! 주석 갱신: 2026-11-08T22:20:00+09:00.
//! 측정 source: aab541f6c6(기준 upstream/devel 7f76cee60e). 2026-11-08 측정, 96 DPI·Print·PDFium·2px raw 실루엣.
//! 측정 실행: Native 2026-11-08T08:38:17+09:00 ~ 08:41:54+09:00, fresh WASM 08:41:54+09:00 ~ 08:48:54+09:00.
//!   기준 PDF는 한컴 2024 Print(pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf), compat 2024.
//! 2025 행정업무운영 편람(최종).hwpx: 대상 47·48·49쪽 Native 100.00% / fresh WASM 100.00%;
//!   전383쪽 최저 Native 99.96% / fresh WASM 99.93%; 누락 없음, 쪽수 PDF와 같음.
//! 재측정: upstream/devel 41e08ae19e 로 rebase 한 코드 커밋 6c2ec2dbd0 에서 같은 조건으로 다시 산출했다
//!   (Native 21:38:38+09:00, fresh WASM 21:45:01+09:00 완료). 전383쪽 Native·fresh WASM TSV 가 위 증거와 바이트까지 같다.
//! 증거: mydocs/pr/assets/issue_8748/{native,wasm}/handbook_silhouette.tsv. 들여쓰기 0/2000 변형은
//!   원본을 메모리에서 바꾼 합성 입력이라 시각 일치 측정 대상이 아니며, 두 변형 사이 관계만 잠근다.
//!
//! [Issue #8748] 쪽을 넘어 나뉜 자리차지 표(글자처럼 취급 아님)는 host 문단의 첫 줄
//! 들여쓰기만큼 오른쪽으로 밀리지 않는다. 문단 기준 개체의 원점은 문단 상자이며, 첫 줄
//! 들여쓰기는 표가 글줄에 참여할 때 그 표가 소유한 글줄에만 적용된다. 온전한 표 경로와
//! 나뉜 조각 경로가 같은 host 여백을 써야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwpx";
const PAGES: u32 = 383;
/// 구역 2 의 온전한 자리차지 표(47쪽)와 세 쪽에 나뉜 자리차지 표(47~49쪽)의 host 문단.
/// 둘 다 글자처럼 취급하지 않는 어울림 표다. 원본 host 의 들여쓰기는 음수라서
/// 0 과 2000HU 두 변형을 비교한다.
const SECTION: usize = 2;
const WHOLE_HOST: usize = 214;
const SPLIT_HOST: usize = 216;
const TOL_PX: f64 = 0.01;

fn open() -> DocumentCore {
    let full = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&full).expect("read sample")).expect("open")
}

fn table_left(node: &RenderNode, para: usize) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Table(table) if table.para_index == Some(para) => Some(node.bbox.x),
        _ => None,
    };
    own.or_else(|| node.children.iter().find_map(|c| table_left(c, para)))
}

/// host 문단들에 첫 줄 들여쓰기만 `indent` 로 바꾼 문단 모양을 새로 붙인다.
fn with_host_indent(core: &mut DocumentCore, hosts: &[usize], indent: i32) {
    let mut doc = core.document().clone();
    let paras = &mut doc.sections[SECTION].paragraphs;
    for &host in hosts {
        let mut shape = doc.doc_info.para_shapes[paras[host].para_shape_id as usize].clone();
        shape.indent = indent;
        doc.doc_info.para_shapes.push(shape);
        paras[host].para_shape_id = (doc.doc_info.para_shapes.len() - 1) as u16;
    }
    core.set_document(doc);
}

fn lefts(core: &DocumentCore) -> [f64; 4] {
    let at = |page: u32, para: usize| {
        let tree = core.build_page_render_tree(page).expect("page");
        table_left(&tree.root, para).unwrap_or_else(|| panic!("{}쪽 표 pi={para}", page + 1))
    };
    [
        at(46, WHOLE_HOST),
        at(46, SPLIT_HOST),
        at(47, SPLIT_HOST),
        at(48, SPLIT_HOST),
    ]
}

#[test]
fn issue_8748_split_float_table_ignores_host_first_line_indent() {
    let mut plain = open();
    with_host_indent(&mut plain, &[WHOLE_HOST, SPLIT_HOST], 0);
    assert_eq!(plain.page_count(), PAGES, "전제: 쪽 구성");
    let before = lefts(&plain);

    let mut indented = open();
    with_host_indent(&mut indented, &[WHOLE_HOST, SPLIT_HOST], 2000);
    assert_eq!(
        indented.page_count(),
        PAGES,
        "전제: 들여쓰기는 쪽 구성을 바꾸지 않는다"
    );
    let after = lefts(&indented);

    for (label, (b, a)) in ["온전한 표", "첫 조각", "둘째 조각", "셋째 조각"]
        .iter()
        .zip(before.iter().zip(after.iter()))
    {
        assert!(
            (a - b).abs() <= TOL_PX,
            "{label}: host 첫 줄 들여쓰기가 자리차지 표를 옮겼다 {b:.2} → {a:.2}px"
        );
    }
    // 같은 쪽의 온전한 표와 첫 조각은 host 왼쪽 여백(둘 다 0)과 가로 기준(문단, 0)이
    // 같으므로 같은 가로 원점에 놓인다. 이어진 조각은 홀짝 쪽 여백이 달라 비교하지 않는다.
    assert!(
        (after[0] - after[1]).abs() <= TOL_PX,
        "같은 쪽 온전한 표와 첫 조각의 가로 원점이 다르다: {:.2} / {:.2}",
        after[0],
        after[1]
    );
}
