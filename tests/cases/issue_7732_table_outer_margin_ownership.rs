//! 검사 유형: 관계형. 독립 Print의 바깥여백 변경량과 실제 표 원점의 소유를 비교한다.
//! 주석 갱신: 2026-11-16 02:28:09 KST. 신규 파일이며 이전 변경 커밋은 없다.
//! 96DPI·PDFium·2px raw, 원본 nested 전17쪽의 em 단계 Native 최저98.62%,
//! em 적용 전 fresh WASM 최저98.65%. 1쪽 N/W=98.62/98.65, 10쪽=99.47/99.44.
//! 단위 관계를 검사하며 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴에 의존하지 않는다.
//! 독립 한컴2020 Print: 위여백140HU 제거 시 1쪽 표 윗변 -1.439pt,
//! 오른쪽141HU 제거 시 10쪽 표 가로선 +1.439pt. TAC 15쪽의 같은 변경은 원점 불변.
//! 정식 분석: mydocs/pr/assets/issue7732_split_span_20261115/print-ttf-em-stage/README.md.
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::io::{Cursor, Read, Write};

const INPUT: &[u8] =
    include_bytes!("../../samples/basic/issue2007_nested_cell_pagination_42065.hwp");

// 원본 파일은 보존하며 TABLE common attribute의 여백 한 필드만 바꾼다.
// offset은 원본 레코드 감사로 확인한 시험 입력의 위치이며 제품 분기에는 쓰지 않는다.
fn without_margin(record_offset: usize, field_offset: usize, expected: u16) -> Vec<u8> {
    let mut compound = cfb::CompoundFile::open(Cursor::new(INPUT.to_vec())).unwrap();
    let mut compressed = Vec::new();
    compound
        .open_stream("/BodyText/Section0")
        .unwrap()
        .read_to_end(&mut compressed)
        .unwrap();
    let mut raw = Vec::new();
    flate2::read::DeflateDecoder::new(&compressed[..])
        .read_to_end(&mut raw)
        .unwrap();
    let start = record_offset + 4 + field_offset;
    assert_eq!(
        u16::from_le_bytes(raw[start..start + 2].try_into().unwrap()),
        expected
    );
    raw[start..start + 2].copy_from_slice(&0_u16.to_le_bytes());
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(&raw).unwrap();
    let data = encoder.finish().unwrap();
    compound
        .create_stream("/BodyText/Section0")
        .unwrap()
        .write_all(&data)
        .unwrap();
    compound.into_inner().into_inner()
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}
fn tables(node: &RenderNode, width_hu: u32, marker: Option<&str>, out: &mut Vec<BoundingBox>) {
    if matches!(&node.node_type, RenderNodeType::Table(_))
        && (node.bbox.width - f64::from(width_hu) / 75.0).abs() < 0.05
        && marker.is_none_or(|marker| text(node).contains(marker))
    {
        out.push(node.bbox);
    }
    for child in &node.children {
        tables(child, width_hu, marker, out);
    }
}
fn table(bytes: &[u8], page: u32, width_hu: u32) -> BoundingBox {
    let core = DocumentCore::from_bytes(bytes).unwrap();
    let tree = core.build_page_render_tree(page).unwrap();
    let mut found = Vec::new();
    tables(
        &tree.root,
        width_hu,
        (page == 14).then_some("조달사업"),
        &mut found,
    );
    assert_eq!(
        found.len(),
        1,
        "변경한 원본 표의 소속은 유일해야 한다: {found:?}"
    );
    found[0]
}

#[test]
fn paragraph_anchored_table_top_consumes_its_own_outer_margin() {
    let original = table(INPUT, 0, 47404);
    let zero = table(&without_margin(1108, 32, 140), 0, 47404);
    assert!(
        (original.y - zero.y - 140.0 / 75.0).abs() < 1.0 / 75.0,
        "위 바깥여백만 제거하면 표 전체가 그만큼 이동해야 한다: {original:?} / {zero:?}"
    );
    assert_eq!(original.x, zero.x);
    assert_eq!(original.width, zero.width);
}

#[test]
fn nested_right_anchor_closes_the_outer_box_at_the_same_cell_edge() {
    let original = table(INPUT, 9, 47624);
    let zero = table(&without_margin(60854, 30, 141), 9, 47624);
    assert!(
        (zero.x - original.x - 141.0 / 75.0).abs() < 1.0 / 75.0,
        "오른쪽 바깥여백만 제거하면 같은 셀 오른끝으로 표가 이동해야 한다: {original:?} / {zero:?}"
    );
    assert_eq!(original.y, zero.y);
    assert_eq!(original.width, zero.width);
}

#[test]
fn inline_table_keeps_the_source_line_pen_when_right_margin_changes() {
    let original = table(INPUT, 14, 47624);
    let zero = table(&without_margin(101828, 30, 141), 14, 47624);
    assert_eq!(
        original.x, zero.x,
        "글자처럼 취급한 표의 원점은 호스트 글줄이 소유한다"
    );
    assert_eq!(original.y, zero.y);
    assert_eq!(original.width, zero.width);
}
