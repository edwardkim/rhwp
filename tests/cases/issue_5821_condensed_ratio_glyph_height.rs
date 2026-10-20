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
