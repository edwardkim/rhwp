//! Issue #1440: 온새미로 35쪽 그림 어울림 본문 줄이 그림 영역을 침범하는 회귀 방지.

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{ShapeObject, TextWrap};
use rhwp::model::style::BorderLineType;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::renderer::StrokeDash;
use std::fs;
use std::path::Path;

const SAMPLES: &[&str] = &[
    "samples/[2027] 온새미로 1 본교재.hwp",
    "samples/[2027] 온새미로 1 본교재.hwpx",
];
const TARGET_PAGE: u32 = 34; // 35쪽, 0-based
const TARGET_PARA: usize = 8;
const BOX_PAGE: u32 = 5; // 6쪽, 0-based
const BOX_PARA: usize = 32;

fn read_fixture(path: &str) -> Vec<u8> {
    fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn collect_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect_nodes(child, out);
    }
}

fn vertically_overlaps(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.y < b.y + b.height && a.y + a.height > b.y
}

fn horizontally_overlaps(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x
}

fn expected_dash(line_type: BorderLineType) -> StrokeDash {
    match line_type {
        BorderLineType::Dash | BorderLineType::LongDash => StrokeDash::Dash,
        BorderLineType::Dot | BorderLineType::Circle => StrokeDash::Dot,
        BorderLineType::DashDot => StrokeDash::DashDot,
        BorderLineType::DashDotDot => StrokeDash::DashDotDot,
        _ => StrokeDash::Solid,
    }
}

fn control_is_square_picture(ctrl: &Control) -> bool {
    match ctrl {
        Control::Picture(pic) => {
            !pic.common.treat_as_char && matches!(pic.common.text_wrap, TextWrap::Square)
        }
        Control::Shape(shape) => match shape.as_ref() {
            ShapeObject::Picture(pic) => {
                !pic.common.treat_as_char && matches!(pic.common.text_wrap, TextWrap::Square)
            }
            other => {
                !other.common().treat_as_char
                    && matches!(other.common().text_wrap, TextWrap::Square)
            }
        },
        _ => false,
    }
}

fn collect_source_paragraphs<'a>(
    paragraphs: &'a [Paragraph],
    path: &str,
    out: &mut Vec<(String, &'a Paragraph)>,
) {
    for (pi, para) in paragraphs.iter().enumerate() {
        let para_path = format!("{path}/p{pi}");
        out.push((para_path.clone(), para));

        for (ci, ctrl) in para.controls.iter().enumerate() {
            match ctrl {
                Control::Table(table) => {
                    for (cell_idx, cell) in table.cells.iter().enumerate() {
                        collect_source_paragraphs(
                            &cell.paragraphs,
                            &format!("{para_path}/c{ci}/cell{cell_idx}"),
                            out,
                        );
                    }
                    if let Some(caption) = &table.caption {
                        collect_source_paragraphs(
                            &caption.paragraphs,
                            &format!("{para_path}/c{ci}/table_caption"),
                            out,
                        );
                    }
                }
                Control::Picture(pic) => {
                    if let Some(caption) = &pic.caption {
                        collect_source_paragraphs(
                            &caption.paragraphs,
                            &format!("{para_path}/c{ci}/picture_caption"),
                            out,
                        );
                    }
                }
                Control::Shape(shape) => {
                    if let Some(drawing) = shape.drawing() {
                        if let Some(text_box) = &drawing.text_box {
                            collect_source_paragraphs(
                                &text_box.paragraphs,
                                &format!("{para_path}/c{ci}/shape_text"),
                                out,
                            );
                        }
                        if let Some(caption) = &drawing.caption {
                            collect_source_paragraphs(
                                &caption.paragraphs,
                                &format!("{para_path}/c{ci}/shape_caption"),
                                out,
                            );
                        }
                    }
                    if let ShapeObject::Picture(pic) = shape.as_ref() {
                        if let Some(caption) = &pic.caption {
                            collect_source_paragraphs(
                                &caption.paragraphs,
                                &format!("{para_path}/c{ci}/shape_picture_caption"),
                                out,
                            );
                        }
                    }
                    if let ShapeObject::Group(group) = shape.as_ref() {
                        for (child_idx, child) in group.children.iter().enumerate() {
                            if let Some(drawing) = child.drawing() {
                                if let Some(text_box) = &drawing.text_box {
                                    collect_source_paragraphs(
                                        &text_box.paragraphs,
                                        &format!("{para_path}/c{ci}/group{child_idx}_text"),
                                        out,
                                    );
                                }
                                if let Some(caption) = &drawing.caption {
                                    collect_source_paragraphs(
                                        &caption.paragraphs,
                                        &format!("{para_path}/c{ci}/group{child_idx}_caption"),
                                        out,
                                    );
                                }
                            }
                        }
                    }
                }
                Control::HiddenComment(comment) => {
                    collect_source_paragraphs(
                        &comment.paragraphs,
                        &format!("{para_path}/c{ci}/hidden_comment"),
                        out,
                    );
                }
                Control::Field(field) => {
                    collect_source_paragraphs(
                        &field.memo_paragraphs,
                        &format!("{para_path}/c{ci}/field_memo"),
                        out,
                    );
                }
                _ => {}
            }
        }
    }
}

fn all_source_paragraphs(doc: &rhwp::wasm_api::HwpDocument) -> Vec<(String, &Paragraph)> {
    let mut out = Vec::new();
    for (si, section) in doc.document().sections.iter().enumerate() {
        collect_source_paragraphs(&section.paragraphs, &format!("s{si}"), &mut out);
    }
    out
}

#[test]
fn issue_1440_page35_text_lines_do_not_cross_square_picture() {
    for sample in SAMPLES {
        let bytes = read_fixture(sample);
        let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("parse {sample}: {e}"));
        let tree = doc
            .build_page_render_tree(TARGET_PAGE)
            .expect("build page 35 render tree");

        let mut nodes = Vec::new();
        collect_nodes(&tree.root, &mut nodes);

        let target_image = nodes
            .iter()
            .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
            .map(|node| &node.bbox)
            .filter(|bbox| bbox.width > 150.0 && bbox.height > 120.0)
            .max_by(|a, b| {
                (a.width * a.height)
                    .partial_cmp(&(b.width * b.height))
                    .unwrap()
            })
            .expect("35쪽 대상 어울림 그림 bbox");

        let mut offenders = Vec::new();
        for node in nodes {
            let RenderNodeType::TextLine(line) = &node.node_type else {
                continue;
            };
            if line.para_index != Some(TARGET_PARA) {
                continue;
            }
            if vertically_overlaps(&node.bbox, target_image)
                && horizontally_overlaps(&node.bbox, target_image)
            {
                offenders.push((
                    line.line_index.unwrap_or(u32::MAX),
                    node.bbox.x,
                    node.bbox.y,
                    node.bbox.width,
                    node.bbox.height,
                ));
            }
        }

        assert!(
            offenders.is_empty(),
            "{sample}: 35쪽 pi={TARGET_PARA} 본문 줄이 그림 bbox를 침범함: image=[x={:.1} y={:.1} w={:.1} h={:.1}], offenders={:?}",
            target_image.x,
            target_image.y,
            target_image.width,
            target_image.height,
            offenders
        );
    }
}

#[test]
fn issue_1440_source_linesegs_encode_wrap_zone_for_target_paragraph() {
    for sample in SAMPLES {
        let bytes = read_fixture(sample);
        let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("parse {sample}: {e}"));
        let paragraphs = all_source_paragraphs(&doc);
        let picture_hosts: Vec<_> = paragraphs
            .iter()
            .filter(|(_, para)| para.controls.iter().any(control_is_square_picture))
            .collect();
        assert!(
            !picture_hosts.is_empty(),
            "{sample}: square-wrap picture host paragraph"
        );

        let wrap_text_paras: Vec<_> = paragraphs
            .iter()
            .filter(|(_, para)| {
                para.line_segs.iter().any(|seg| {
                    seg.column_start > 0 || (seg.segment_width > 0 && seg.segment_width < 30_000)
                })
            })
            .collect();
        assert!(
            !wrap_text_paras.is_empty(),
            "{sample}: source should carry precomputed wrap-zone line segments"
        );

        let target_text_para = paragraphs
            .iter()
            .find(|(path, para)| path == "s3/p8" && para.text.contains("기차 안에서처럼"))
            .expect("35쪽 대상 본문 문단 s3/p8");
        assert!(
            target_text_para
                .1
                .line_segs
                .iter()
                .take(7)
                .all(|seg| seg.column_start == 850 && seg.segment_width == 20_999),
            "{sample}: 35쪽 대상 본문 첫 7줄은 그림 왼쪽 wrap-zone LineSeg여야 함"
        );
    }
}

#[test]
fn issue_1440_page6_box_paragraph_does_not_double_apply_lineseg_column_start() {
    for sample in SAMPLES {
        let bytes = read_fixture(sample);
        let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("parse {sample}: {e}"));
        let tree = doc
            .build_page_render_tree(BOX_PAGE)
            .expect("build page 6 render tree");

        let mut nodes = Vec::new();
        collect_nodes(&tree.root, &mut nodes);
        let mut box_lines: Vec<_> = nodes
            .into_iter()
            .filter_map(|node| {
                let RenderNodeType::TextLine(line) = &node.node_type else {
                    return None;
                };
                if line.para_index == Some(BOX_PARA) {
                    Some((line.line_index.unwrap_or(u32::MAX), node.bbox.x))
                } else {
                    None
                }
            })
            .collect();
        box_lines.sort_by_key(|(line_index, _)| *line_index);

        assert!(
            box_lines.len() >= 2,
            "{sample}: 6쪽 지문 박스 문단 pi={BOX_PARA}의 줄을 찾지 못함: {box_lines:?}"
        );

        let first_x = box_lines[0].1;
        let second_x = box_lines[1].1;
        assert!(
            first_x < 235.0 && second_x < 222.0,
            "{sample}: 6쪽 지문 박스에 LineSeg.column_start가 이중 적용됨: first_x={first_x:.1}, second_x={second_x:.1}, lines={box_lines:?}"
        );
    }
}

#[test]
fn issue_1440_page6_box_border_connect_and_dash_line_are_preserved() {
    for sample in SAMPLES {
        let bytes = read_fixture(sample);
        let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("parse {sample}: {e}"));
        let paragraphs = all_source_paragraphs(&doc);
        let (_, box_para) = paragraphs
            .iter()
            .find(|(_, para)| para.text.starts_with("수많은 SF 영화나 소설이 유토피아"))
            .unwrap_or_else(|| panic!("{sample}: 6쪽 지문 박스 문단을 찾지 못함"));
        let ps = doc
            .document()
            .doc_info
            .para_shapes
            .get(box_para.para_shape_id as usize)
            .unwrap_or_else(|| panic!("{sample}: 지문 박스 ParaShape 없음"));
        assert!(
            (ps.attr1 >> 28) & 1 != 0,
            "{sample}: 문단 테두리 연결(bit 28)이 보존되어야 함"
        );

        let border_fill = doc
            .document()
            .doc_info
            .border_fills
            .get(ps.border_fill_id.saturating_sub(1) as usize)
            .unwrap_or_else(|| panic!("{sample}: 지문 박스 BorderFill 없음"));
        let expected_line_dash = expected_dash(border_fill.borders[0].line_type);
        assert!(
            border_fill
                .borders
                .iter()
                .all(|border| expected_dash(border.line_type) == expected_line_dash),
            "{sample}: 지문 박스 테두리는 네 면의 선 모양이 같아야 함"
        );
        assert!(
            expected_line_dash != StrokeDash::Solid,
            "{sample}: 지문 박스 테두리는 실선 최적화 대상이 아니어야 함"
        );

        let tree = doc
            .build_page_render_tree(BOX_PAGE)
            .expect("build page 6 render tree");
        let mut nodes = Vec::new();
        collect_nodes(&tree.root, &mut nodes);
        let box_text_bounds: Vec<_> = nodes
            .iter()
            .filter_map(|node| {
                let RenderNodeType::TextLine(line) = &node.node_type else {
                    return None;
                };
                (line.para_index == Some(BOX_PARA)).then_some(node.bbox.clone())
            })
            .collect();
        assert!(
            !box_text_bounds.is_empty(),
            "{sample}: 6쪽 지문 박스 TextLine bbox 없음"
        );
        let min_x = box_text_bounds
            .iter()
            .map(|b| b.x)
            .fold(f64::INFINITY, f64::min);
        let max_x = box_text_bounds
            .iter()
            .map(|b| b.x + b.width)
            .fold(0.0, f64::max);
        let min_y = box_text_bounds
            .iter()
            .map(|b| b.y)
            .fold(f64::INFINITY, f64::min);
        let max_y = box_text_bounds
            .iter()
            .map(|b| b.y + b.height)
            .fold(0.0, f64::max);
        let dotted_near_box = nodes.iter().any(|node| {
            let RenderNodeType::Line(line) = &node.node_type else {
                return false;
            };
            line.style.dash == expected_line_dash
                && node.bbox.x >= min_x - 80.0
                && node.bbox.x <= max_x + 80.0
                && node.bbox.y >= min_y - 80.0
                && node.bbox.y <= max_y + 80.0
        });
        assert!(
            dotted_near_box,
            "{sample}: 6쪽 지문 박스 주변에 원본 선 모양 LineNode가 렌더되어야 함"
        );
    }
}
