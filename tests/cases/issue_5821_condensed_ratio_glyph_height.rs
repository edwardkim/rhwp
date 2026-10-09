//! #5821: 장평은 가로 축에 적용하고 선언된 세로 크기는 유지한다.
//! 독립 Print의 Tf·CTM·Tz가 근거다. 추출 span의 기하평균 size와 구분한다.
//! 전체 1쪽 Native 99.98879% / fresh WASM 100%와 직접 판독은 단계 910에 기록했다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph};
use rhwp::parser::parse_document;

const SAMPLE: &str = "samples/issue5821/condensed_ratio_title_box.hwpx";

fn used_shapes(paragraphs: &[Paragraph], ids: &mut HashSet<u32>) {
    for paragraph in paragraphs {
        if !paragraph.text.is_empty() {
            ids.extend(
                paragraph
                    .char_shapes
                    .iter()
                    .map(|shape| shape.char_shape_id),
            );
        }
        for control in &paragraph.controls {
            if let Control::Table(table) = control {
                for cell in &table.cells {
                    used_shapes(&cell.paragraphs, ids);
                }
                if let Some(caption) = &table.caption {
                    used_shapes(&caption.paragraphs, ids);
                }
            }
        }
    }
}

#[test]
fn issue_5821_condensed_title_preserves_source_height_and_horizontal_ratio() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("원본 입력");
    let document = parse_document(&bytes).expect("원본 속성");
    let mut used = HashSet::new();
    for section in &document.sections {
        used_shapes(&section.paragraphs, &mut used);
    }
    // 이 입력의 가장 큰 실제 사용 모양은 압축 장평을 가진 한글 제목이다.
    // 사용하지 않은 header의 큰 모양이나 화면 좌표로 제목을 고르지 않는다.
    let title = used
        .iter()
        .map(|id| &document.doc_info.char_shapes[*id as usize])
        .max_by_key(|shape| shape.base_size * i32::from(shape.relative_sizes[0]))
        .expect("제목 글자 모양");
    let expected_size = f64::from(title.base_size) * f64::from(title.relative_sizes[0]) / 100.0
        * rhwp::renderer::DEFAULT_DPI
        / rhwp::renderer::HWPUNIT_PER_INCH;
    let expected_ratio = f64::from(title.ratios[0]) / 100.0;
    assert!(expected_size > 0.0 && expected_ratio > 0.0 && expected_ratio < 1.0);

    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    let svg = core.render_page_svg_native(0).expect("첫 쪽 SVG");
    let xml = roxmltree::Document::parse(&svg).expect("SVG 구조");
    let candidates: Vec<_> = xml
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| {
            let size = node.attribute("font-size")?.parse::<f64>().ok()?;
            (size >= expected_size * expected_ratio).then_some((node, size))
        })
        .collect();
    assert!(
        !candidates.is_empty(),
        "제목 글리프가 실제 출력에 있어야 한다"
    );
    for (node, size) in candidates {
        assert!(
            (size / expected_size - 1.0).abs() < 0.001,
            "선언 세로 크기를 보존해야 한다: source={expected_size}, actual={size}"
        );
        let transform = node.attribute("transform").expect("장평 변환");
        let scale = transform
            .split("scale(")
            .nth(1)
            .expect("가로 축 변환")
            .split(')')
            .next()
            .expect("변환 끝");
        let values: Vec<f64> = scale
            .split(',')
            .map(|value| value.trim().parse().expect("배율"))
            .collect();
        assert_eq!(values.len(), 2);
        assert!(
            (values[0] - expected_ratio).abs() < 0.001,
            "가로 축은 원본 장평이다"
        );
        assert_eq!(values[1], 1.0, "세로 축은 변하지 않는다");
    }

}

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("정식 원본 읽기")).expect("문서 파싱")
}

fn title(core: &DocumentCore) -> &rhwp::model::paragraph::Paragraph {
    let table = core.document().sections[0]
        .paragraphs
        .iter()
        .find_map(|para| {
            para.controls.iter().find_map(|control| match control {
                Control::Table(table) => Some(table),
                _ => None,
            })
        })
        .expect("제목 표");
    let para = &table.cells[0].paragraphs[0];
    assert!(para.text.starts_with("경찰청,"), "원본 제목 소속");
    para
}

struct Glyph {
    text: String,
    x: f64,
    y: f64,
    size: f64,
    ratio: f64,
}

fn title_glyphs(svg: &str) -> Vec<Glyph> {
    let xml = roxmltree::Document::parse(svg).expect("실제 SVG");
    xml.descendants()
        .filter(|node| {
            node.has_tag_name("text")
                && node
                    .attribute("font-family")
                    .is_some_and(|family| family.starts_with("'함초롬돋움'"))
                && node.attribute("font-weight") == Some("bold")
        })
        .map(|node| {
            let transform = node.attribute("transform").expect("장평 변환");
            let translate = transform
                .strip_prefix("translate(")
                .expect("글자 원점")
                .split(')')
                .next()
                .unwrap();
            let mut xy = translate
                .split(',')
                .map(|value| value.parse::<f64>().unwrap());
            let scale = transform
                .split("scale(")
                .nth(1)
                .expect("가로 장평")
                .split(',')
                .next()
                .unwrap();
            Glyph {
                text: node.text().expect("제목 글자").to_owned(),
                x: xy.next().unwrap(),
                y: xy.next().unwrap(),
                size: node.attribute("font-size").unwrap().parse().unwrap(),
                ratio: scale.parse().unwrap(),
            }
        })
        .collect()
}

#[test]
fn issue_5821_title_keeps_declared_glyph_size_and_content() {
    let core = core();
    assert_eq!(core.page_count(), 1, "독립 PDF와 원본의 쪽수");
    let para = title(&core);
    let shapes: Vec<_> = para
        .char_shapes
        .iter()
        .map(|reference| &core.document().doc_info.char_shapes[reference.char_shape_id as usize])
        .collect();
    let declared_size = f64::from(shapes[0].base_size) * 96.0 / 7200.0;
    let declared_ratio = f64::from(shapes[0].ratios[0]) / 100.0;
    assert!(shapes
        .iter()
        .all(|shape| shape.base_size == shapes[0].base_size
            && shape.ratios[0] == shapes[0].ratios[0]));
    let svg = core.render_page_svg_native(0).expect("실제 제목 렌더링");
    let glyphs = title_glyphs(&svg);
    let actual: String = glyphs.iter().map(|glyph| glyph.text.as_str()).collect();
    let expected: String = para.text.chars().filter(|ch| !ch.is_whitespace()).collect();
    assert_eq!(actual, expected, "두 제목 줄의 내용·순서·누락·중복");
    for glyph in glyphs {
        assert!(
            (glyph.size - declared_size).abs() <= declared_size * f64::EPSILON * 64.0,
            "장평이 선언 세로 크기를 바꾸면 안 된다: {} / 선언 {}",
            glyph.size,
            declared_size
        );
        assert_eq!(glyph.ratio, declared_ratio, "장평은 원래 가로 비율이다");
    }
}

fn title_cell(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 0 && cell.col == 0)
    {
        return Some(node);
    }
    node.children.iter().find_map(title_cell)
}

#[test]
fn issue_5821_first_title_visible_span_is_centered_in_its_cell() {
    let core = core();
    let svg = core.render_page_svg_native(0).expect("실제 제목 SVG");
    let glyphs = title_glyphs(&svg);
    let first_y = glyphs
        .iter()
        .map(|glyph| glyph.y)
        .min_by(f64::total_cmp)
        .expect("첫 제목 줄");
    let first_line: Vec<_> = glyphs.iter().filter(|glyph| glyph.y == first_y).collect();
    let first = first_line.first().unwrap();
    let last = first_line.last().unwrap();
    assert_eq!(last.text, "여", "독립 PDF와 같은 첫 제목 줄 종료");
    // 독립 PDF의 마지막 한글 advance와 실제 공급 프로그램은 970/1000이다.
    // 마지막 글자 뒤에는 다음 글자가 없으므로 자간을 뺀 전진이 아닌 자연 폭을 쓴다.
    let visible_end = last.x + last.size * last.ratio * 970.0 / 1000.0;
    let tree = core.build_page_render_tree(0).expect("제목 셀의 실제 상자");
    let cell = title_cell(&tree.root).expect("제목 셀 소속");
    let span_center = (first.x + visible_end) / 2.0;
    let cell_center = cell.bbox.x + cell.bbox.width / 2.0;
    // HU 한 단위의 변환 정밀도만 허용한다. 실제 문서의 절대 좌표를 고정하지 않는다.
    assert!(
        (span_center - cell_center).abs() <= 96.0 / 7200.0,
        "첫 제목 가시 폭이 셀 중심에 맞아야 한다: 제목 {span_center}, 셀 {cell_center}"
    );
}
