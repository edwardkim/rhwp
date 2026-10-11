//! [#7235] NONE 채우기는 원본 그림의 종횡비를 지키며 소유 셀·도형에 맞춘다.
//! 독립 한컴 Print PDF와 Native/fresh WASM 전 14쪽 최저 97.76365% 확인 후
//! 절대 좌표·최근접 상자·3px/5% 핀을 원본 그림 식별과 소유 영역 관계로 교체했다.
//! 새 쪽에 통째로 옮겨진 TAC의 저장 줄간격과 뒤 문단 소유도 같은 원본에서 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::ImageFillMode;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/issue7235/156086935_none_image_fill.hwp";

fn original_bytes() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE))
        .expect("#7235 원본 읽기")
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if !node.visible || node.editor_only {
        return;
    }
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 수치 해석의 반올림만 허용한다. 페이지 위치나 허용 픽셀 영역을 고정하지 않는다.
fn same_value(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= expected.abs().max(1.0) * 1e-9,
        "원본 소유 영역 관계: actual={actual}, expected={expected}"
    );
}

fn fill_frames<'a>(node: &'a RenderNode, cells: bool, out: &mut Vec<&'a RenderNode>) {
    let owns_fill = if cells {
        matches!(node.node_type, RenderNodeType::TableCell(_))
    } else {
        matches!(node.node_type, RenderNodeType::Rectangle(_))
    };
    for child in &node.children {
        if owns_fill
            && matches!(&child.node_type, RenderNodeType::Image(image)
            if image.fill_mode == Some(ImageFillMode::None))
        {
            for (actual, expected) in [
                (child.bbox.x, node.bbox.x),
                (child.bbox.y, node.bbox.y),
                (child.bbox.width, node.bbox.width),
                (child.bbox.height, node.bbox.height),
            ] {
                same_value(actual, expected);
            }
            out.push(child);
        }
        fill_frames(child, cells, out);
    }
}

/// 실제 SVG payload로 원본 그림을 찾는다. 비슷한 위치의 다른 그림으로 대체하지 않는다.
fn assert_fill_output(core: &DocumentCore, cells: bool, expected_images: &[(u32, u32)]) {
    let mut seen = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("소유 영역 출력");
        let mut frames = Vec::new();
        fill_frames(&tree.root, cells, &mut frames);
        if frames.is_empty() {
            continue;
        }
        let svg = core.render_page_svg_native(page).expect("최종 SVG 출력");
        let xml = roxmltree::Document::parse(&svg).expect("출력 XML");
        for frame in frames {
            let RenderNodeType::Image(image_node) = &frame.node_type else {
                unreachable!();
            };
            let original = image::load_from_memory(image_node.data.as_ref().expect("원본 그림"))
                .expect("원본 그림 해독");
            let intrinsic = (original.width(), original.height());
            let images: Vec<_> = xml
                .descendants()
                .filter(|node| {
                    if !node.has_tag_name("image") {
                        return false;
                    }
                    let Some(href) = node.attributes().find(|a| a.name() == "href") else {
                        return false;
                    };
                    let Some((_, payload)) = href.value().split_once(',') else {
                        return false;
                    };
                    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(payload)
                    else {
                        return false;
                    };
                    image::load_from_memory(&bytes)
                        .is_ok_and(|decoded| (decoded.width(), decoded.height()) == intrinsic)
                })
                .collect();
            assert_eq!(images.len(), 1, "원본 그림의 누락·중복: {intrinsic:?}");
            let painted = images[0];
            assert_eq!(
                painted.attribute("preserveAspectRatio"),
                Some("xMidYMid meet")
            );
            for (name, expected) in [
                ("x", frame.bbox.x),
                ("y", frame.bbox.y),
                ("width", frame.bbox.width),
                ("height", frame.bbox.height),
            ] {
                same_value(
                    painted
                        .attribute(name)
                        .expect("그림 영역")
                        .parse()
                        .expect("영역 수치"),
                    expected,
                );
            }
            let scale =
                (frame.bbox.width / intrinsic.0 as f64).min(frame.bbox.height / intrinsic.1 as f64);
            let width = intrinsic.0 as f64 * scale;
            let height = intrinsic.1 as f64 * scale;
            assert!(width <= frame.bbox.width && height <= frame.bbox.height);
            same_value(width / height, intrinsic.0 as f64 / intrinsic.1 as f64);
            seen.push(intrinsic);
        }
    }
    seen.sort_unstable();
    let mut expected = expected_images.to_vec();
    expected.sort_unstable();
    assert_eq!(
        seen, expected,
        "원본 NONE 채우기 소유를 빠짐없이 한 번씩 출력한다"
    );
}

fn assert_rebased_object_row(core: &DocumentCore, bytes: &[u8]) {
    let source = rhwp::parse_document(bytes).expect("독립 원본 모델");
    let mut checked = 0;
    for (section_index, section) in source.sections.iter().enumerate() {
        for (paragraph_index, pair) in section.paragraphs.windows(2).enumerate() {
            let (host, next) = (&pair[0], &pair[1]);
            let ([line], Some(next_line)) = (host.line_segs.as_slice(), next.line_segs.first())
            else {
                continue;
            };
            let Some((control_index, table)) =
                host.controls
                    .iter()
                    .enumerate()
                    .find_map(|(index, control)| match control {
                        Control::Table(table) if table.common.treat_as_char => Some((index, table)),
                        _ => None,
                    })
            else {
                continue;
            };
            let source_height = table.common.height as i64
                + table.common.margin.top as i64
                + table.common.margin.bottom as i64;
            let following_vpos = line.line_height as i64 + line.line_spacing as i64;
            if source_height != line.line_height as i64
                || line.vertical_pos <= next_line.vertical_pos
                || following_vpos != next_line.vertical_pos as i64
                || line.line_spacing <= 0
            {
                continue;
            }
            let mut tables = Vec::new();
            let mut following = Vec::new();
            for page in 0..core.page_count() {
                let tree = core.build_page_render_tree(page).expect("전체 쪽 소유");
                let mut nodes = Vec::new();
                walk(&tree.root, &mut nodes);
                for node in nodes {
                    if let RenderNodeType::Table(meta) = &node.node_type {
                        if meta.section_index == Some(section_index)
                            && meta.para_index == Some(paragraph_index)
                            && meta.control_index == Some(control_index)
                            && meta.cell_context.is_none()
                        {
                            let compact = |text: &str| -> String {
                                text.chars()
                                    .filter(|ch| {
                                        !ch.is_whitespace() && !ch.is_control() && *ch != '\u{fffc}'
                                    })
                                    .collect()
                            };
                            let mut owners = Vec::new();
                            for child in &node.children {
                                let RenderNodeType::TableCell(cell) = &child.node_type else {
                                    continue;
                                };
                                let index = cell.model_cell_index.expect("원본 셀 소속") as usize;
                                let source_cell = table.cells.get(index).expect("원본 범위의 셀");
                                owners.push(index);
                                let mut runs = Vec::new();
                                walk(child, &mut runs);
                                let text: String = runs
                                    .into_iter()
                                    .filter_map(|run| match &run.node_type {
                                        RenderNodeType::TextRun(run) => Some(run.display_or_text()),
                                        _ => None,
                                    })
                                    .collect();
                                let expected: String = source_cell
                                    .paragraphs
                                    .iter()
                                    .map(|p| p.text.as_str())
                                    .collect();
                                assert_eq!(
                                    compact(&text),
                                    compact(&expected),
                                    "새 쪽 표 셀의 내용 누락·중복: {index}"
                                );
                            }
                            owners.sort_unstable();
                            assert_eq!(
                                owners,
                                (0..table.cells.len()).collect::<Vec<_>>(),
                                "원본 셀을 정확히 한 번 소유한다"
                            );
                            tables.push((page, node.bbox));
                        }
                    }
                }
                // 셀의 문단 번호를 본문 번호로 혼동하지 않도록 셀 하위는 탐색하지 않는다.
                let mut body = vec![&tree.root];
                while let Some(node) = body.pop() {
                    if !node.visible
                        || node.editor_only
                        || matches!(node.node_type, RenderNodeType::TableCell(_))
                    {
                        continue;
                    }
                    if let RenderNodeType::TextLine(meta) = &node.node_type {
                        if meta.section_index == Some(section_index)
                            && meta.para_index == Some(paragraph_index + 1)
                        {
                            following.push((page, node.bbox));
                        }
                    }
                    body.extend(&node.children);
                }
            }
            assert_eq!(tables.len(), 1, "통째로 이동한 원본 표의 누락·중복");
            assert_eq!(
                following.len(),
                next.line_segs.len(),
                "뒤 문단의 줄 소유 보존"
            );
            assert_eq!(
                tables[0].0, following[0].0,
                "닫힌 새 프레임의 뒤 문단은 표와 같은 쪽을 소유한다"
            );
            let outer_top = tables[0].1.y - table.common.margin.top as f64 / 75.0;
            let expected_offset = following_vpos as f64 / 75.0;
            same_value(following[0].1.y - outer_top, expected_offset);
            assert!(following[0].1.y >= tables[0].1.y + tables[0].1.height);
            checked += 1;
        }
    }
    assert!(checked > 0, "원본의 완전한 새 프레임 줄을 실제로 검사한다");
}

#[test]
fn none_fill_in_table_cell_fits_cell_keeping_aspect() {
    let bytes = original_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 14, "독립 Print PDF의 전체 쪽수");
    assert_fill_output(&core, true, &[(454, 152), (1009, 383)]);
    assert_rebased_object_row(&core, &bytes);
}

#[test]
fn none_fill_in_shape_fits_shape_keeping_aspect() {
    let core = DocumentCore::from_bytes(&original_bytes()).expect("문서 로드");
    assert_fill_output(&core, false, &[(1500, 166)]);
}
