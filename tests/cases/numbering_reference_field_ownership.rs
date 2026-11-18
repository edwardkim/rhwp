//! 관계형: 번호 형식의 고정 예약 칸과 실제 인스턴스 폭이 본문 원점·이어지는 줄을 구분한다.
//! 주석 작성: 2026-11-07 09:26:20 +09:00. 새 파일이므로 직전 변경 커밋은 없다.
//! 원본 samples/basic/interview.hwp와 독립 pdf/basic/interview-hwp-2020.pdf 전2쪽.
//! 2026-11-07, 96DPI·Print·보정 전2px raw: R11 Native/fresh WASM 전쪽 최저 각각96.01%.
//! 두 경로 모두 PDF와2쪽 일치. pr8617/r11/interview/{native,wasm}.tsv와 같은 쪽
//! review/overlay 직접 판독을 사용한다. R28 Native도 전2쪽 최저96.01%다.
//! 증거: mydocs/pr/assets/planet6897_20261106/pr8617/r11/interview/provenance.json.
//! 독립 Print1쪽은 각 단의1~10번 본문을 같은 원점에 두고, 원본 번호 attr0x8은
//! 표40 bit2(실제 인스턴스 폭)를 끈다. 절대 px·폰트 설치·원본 PDF의 글꼴 공급은 검사하지 않는다.
//! bit2를 켠 IR 반례는 사양 계약이며 별도의 한컴 실물 출력 일치 증거로 표시하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/interview.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("정식 면접 원본")).unwrap()
}

#[derive(Default)]
struct Body {
    text: String,
    rows: Vec<(f64, f64)>,
}

fn collect(node: &RenderNode, paragraph: usize, body: &mut Body) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.section_index == Some(0)
            && run.para_index == Some(paragraph)
            && run.char_start.is_some()
            && !run.text.is_empty()
        {
            body.text.push_str(&run.text);
            if body.rows.last().is_none_or(|(y, _)| *y != node.bbox.y) {
                body.rows.push((node.bbox.y, node.bbox.x));
            }
        }
    }
    for child in &node.children {
        collect(child, paragraph, body);
    }
}

fn bodies(core: &DocumentCore, paragraphs: std::ops::RangeInclusive<usize>) -> Vec<Body> {
    let trees: Vec<_> = (0..core.page_count())
        .map(|page| {
            core.build_page_render_tree(page)
                .expect("전쪽 실제 render tree")
        })
        .collect();
    paragraphs
        .map(|paragraph| {
            let mut body = Body::default();
            for tree in &trees {
                collect(&tree.root, paragraph, &mut body);
            }
            assert_eq!(
                body.text,
                core.document().sections[0].paragraphs[paragraph].text,
                "번호의 폭 변경으로 원문을 누락·중복하거나 순서를 바꾸면 안 된다"
            );
            assert!(
                !body.rows.is_empty(),
                "모든 원본 본문에 소유 글줄이 있어야 한다"
            );
            body
        })
        .collect()
}

#[test]
fn declared_reference_format_keeps_one_to_ten_and_continuations_on_the_same_body_origin() {
    let core = load();
    for paragraphs in [14..=23, 27..=36] {
        let rows = bodies(&core, paragraphs);
        let reference_x = rows[0].rows[0].1;
        for body in rows {
            for (_, x) in body.rows {
                assert_eq!(
                    x, reference_x,
                    "bit2가 꺼진 같은 번호 형식의 본문과 이어지는 줄은 같은 시작점을 공유한다"
                );
            }
        }
    }
}

#[test]
fn explicit_instance_width_still_reserves_the_expanded_number() {
    let mut core = load();
    let mut document = core.document().clone();
    for numbering in &mut document.doc_info.numberings {
        for head in &mut numbering.heads {
            head.attr |= 1 << 2;
        }
    }
    core.set_document(document);
    for paragraphs in [14..=23, 27..=36] {
        let rows = bodies(&core, paragraphs);
        assert!(
            rows.last().unwrap().rows[0].1 > rows[0].rows[0].1,
            "실제 인스턴스 폭을 켜면 두 자리 번호가 한 자리보다 넓은 영역을 소유한다"
        );
    }
}
