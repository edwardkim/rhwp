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

fn has_short_dots(dash: StrokeDash) -> bool {
    match dash {
        StrokeDash::Dot => true,
        // 독립 Print의 이 박스는 짧은 선과 그보다 긴 공백을 반복한다.
        // 물리 반복 구간을 방출하는 경로를 옛 열거형의 이름으로 거부하지 않는다.
        StrokeDash::Explicit(values) => {
            values[0] > 0.0
                && values[1] > values[0]
                && values[2..].iter().all(|value| *value == 0.0)
        }
        _ => false,
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
        assert_eq!(border_fill.borders[0].line_type, BorderLineType::Dot);
        assert!(
            border_fill
                .borders
                .iter()
                .all(|border| border.line_type == BorderLineType::Dot),
            "{sample}: 지문 박스 테두리는 네 면의 선 모양이 같아야 함"
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
        let dotted: Vec<_> = nodes
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::Line(line)
                    if has_short_dots(line.style.dash)
                        && line.style.color == border_fill.borders[0].color =>
                {
                    Some(line)
                }
                _ => None,
            })
            .collect();
        let vertical: Vec<_> = dotted
            .iter()
            .filter(|line| {
                line.x1 == line.x2 && line.y1.min(line.y2) <= min_y && line.y1.max(line.y2) >= max_y
            })
            .collect();
        let horizontal: Vec<_> = dotted
            .iter()
            .filter(|line| {
                line.y1 == line.y2 && line.x1.min(line.x2) <= min_x && line.x1.max(line.x2) >= max_x
            })
            .collect();
        let left = vertical
            .iter()
            .filter(|line| line.x1 <= min_x)
            .max_by(|a, b| a.x1.total_cmp(&b.x1))
            .expect("지문 왼쪽 점선");
        let right = vertical
            .iter()
            .filter(|line| line.x1 >= max_x)
            .min_by(|a, b| a.x1.total_cmp(&b.x1))
            .expect("지문 오른쪽 점선");
        let top = horizontal
            .iter()
            .filter(|line| line.y1 <= min_y)
            .max_by(|a, b| a.y1.total_cmp(&b.y1))
            .expect("지문 위쪽 점선");
        let bottom = horizontal
            .iter()
            .filter(|line| line.y1 >= max_y)
            .min_by(|a, b| a.y1.total_cmp(&b.y1))
            .expect("지문 아래쪽 점선");
        for line in [top, bottom] {
            assert_eq!(line.x1.min(line.x2), left.x1, "네 면의 왼쪽 연결");
            assert_eq!(line.x1.max(line.x2), right.x1, "네 면의 오른쪽 연결");
        }
        for line in [left, right] {
            assert_eq!(line.y1.min(line.y2), top.y1, "네 면의 위쪽 연결");
            assert_eq!(line.y1.max(line.y2), bottom.y1, "네 면의 아래쪽 연결");
        }
    }
}

/// #6970: 저장 `LINE_SEG` 가 없는(합성 줄로 조판되는) 문서에서 Square 어울림 그림·
/// 인라인 아이콘을 낀 다단 문단이 단 하단을 넘고 본문이 개체 위로 흐르지 않아야 한다.
///
/// 픽스처는 실문서 익명화본(한글 음절 순환 치환·이미지 더미화·기하 보존)으로 3쪽이
/// 정답이다. 수정 전 `devel` 은 쪽수는 맞추면서도 `layout-anomaly` 가 offCanvas 20 ·
/// overflow 27 · overlap 3 을 보고했고(2쪽 좌측 단 내용이 상단 배너 위로 올라가고
/// 아이콘·텍스트·스크린샷이 서로 겹침), 엔진 스스로 `LAYOUT_OVERFLOW` 를 찍었다.
#[test]
fn issue_6970_no_ls_square_wrap_columns_stay_inside_body_without_overlap() {
    use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
    use rhwp::document_core::DocumentCore;

    let bytes = read_fixture("tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp");
    let doc = DocumentCore::from_bytes(&bytes).expect("#6970 fixture parse");
    assert_eq!(doc.page_count(), 3, "#6970: 쪽수는 정답(3)과 같아야 한다");

    let opts = AnomalyOptions {
        overflow_tolerance_px: 2.0,
        ..AnomalyOptions::default()
    };
    for page in 0..doc.page_count() {
        let tree = doc
            .build_page_render_tree(page)
            .expect("#6970 fixture render");
        let result = scan_page(page, &tree.root, doc.page_count(), &opts);
        assert!(
            result.off_canvas.is_empty(),
            "#6970 p{}: 단 밖으로 나간 항목 {:?}",
            page + 1,
            result.off_canvas
        );
        assert!(
            result.overflow.iter().all(|item| item.over_bottom <= 2.0),
            "#6970 p{}: 단 하단 넘침 {:?}",
            page + 1,
            result.overflow
        );
        assert!(
            result.overlap.is_empty(),
            "#6970 p{}: 개체 겹침 {:?}",
            page + 1,
            result.overlap
        );
    }
}

/// HWP 5.0 사양 표 139의 단 방향 2는 맞쪽이다. 독립 기준인 한컴 PDF에서는
/// 짝수인 2쪽의 77번 문단이 69번 문단 왼쪽에 놓이고, 100번 문단은 한 줄에
/// 겹치지 않고 서로 다른 두 줄을 차지한다.
#[test]
fn issue_6970_mirror_columns_and_square_paragraph_keep_source_geometry() {
    let bytes = read_fixture("tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp");
    let source = rhwp::parser::parse_document(&bytes).expect("parse source");
    let columns = source.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::ColumnDef(columns) = c {
                Some(columns)
            } else {
                None
            }
        })
        .expect("source columns");
    assert_eq!((columns.raw_attr >> 10) & 3, 2);
    assert_eq!(format!("{:?}", columns.direction), "Mirror");
    let core = rhwp::document_core::DocumentCore::from_bytes(&bytes).expect("open");
    let page = core.build_page_render_tree(1).expect("second page");
    let mut nodes = Vec::new();
    collect_nodes(&page.root, &mut nodes);
    let lines = |pi| {
        nodes.iter().filter(|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(pi))
    }).map(|node| node.bbox.clone()).collect::<Vec<_>>()
    };
    assert!(lines(77)[0].x < lines(69)[0].x, "even page column order");
    let wrapped = lines(100);
    assert_eq!(
        wrapped.len(),
        2,
        "Hancom PDF p2 has two lines beside the picture"
    );
    assert!(
        wrapped[0].y + wrapped[0].height <= wrapped[1].y,
        "distinct line boxes"
    );
    assert!(
        wrapped[1].y + wrapped[1].height <= lines(102)[0].y,
        "following paragraph preserved"
    );
}

#[test]
fn issue_6970_mirror_direction_survives_hwp_and_hwpx_save() {
    let bytes = read_fixture("tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp");
    let mut source = rhwp::parser::parse_document(&bytes).expect("parse source");
    // 원본 0x1808 값을 그대로 통과시키지 않고 속성을 재구성하는 저장 경로를 검사한다.
    source.sections[0].raw_stream = None;
    for control in &mut source.sections[0].paragraphs[0].controls {
        if let Control::ColumnDef(columns) = control {
            columns.raw_attr = 0;
        }
    }
    for saved in [
        rhwp::serializer::serialize_document(&source).expect("save HWP"),
        rhwp::serializer::hwpx::serialize_hwpx(&source).expect("save HWPX"),
    ] {
        let reparsed = rhwp::parser::parse_document(&saved).expect("reopen");
        let direction = reparsed.sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|c| {
                if let Control::ColumnDef(columns) = c {
                    Some(format!("{:?}", columns.direction))
                } else {
                    None
                }
            })
            .expect("saved columns");
        assert_eq!(direction, "Mirror");
    }
}

#[test]
fn issue_6970_source_column_direction_preserves_rectangles_and_page_parity() {
    use rhwp::model::page::{ColumnDef, ColumnDirection, PageDef};
    use rhwp::renderer::page_layout::PageLayoutInfo;

    for direction in [
        ColumnDirection::LeftToRight,
        ColumnDirection::RightToLeft,
        ColumnDirection::Mirror,
    ] {
        let columns = ColumnDef {
            column_count: 3,
            same_width: true,
            spacing: 300,
            direction,
            ..Default::default()
        };
        let page = PageDef {
            width: 59528,
            height: 84188,
            margin_left: 3000,
            margin_right: 3000,
            ..Default::default()
        };
        let mut layout = PageLayoutInfo::from_page_def_for_page(&page, &columns, 96.0, 1);
        let total: f64 = layout.column_areas.iter().map(|r| r.width).sum();
        for number in [1, 2, 2, 3, 4, 1] {
            layout.apply_page_number_margins(&page, number);
            let reversed = direction == ColumnDirection::RightToLeft
                || (direction == ColumnDirection::Mirror && number.is_multiple_of(2));
            assert_eq!(
                layout.column_areas[0].x > layout.column_areas[2].x,
                reversed
            );
            assert!(
                (layout.column_areas.iter().map(|r| r.width).sum::<f64>() - total).abs() < 0.001
            );
            for area in &layout.column_areas {
                assert!(
                    area.x >= layout.body_area.x
                        && area.x + area.width
                            <= layout.body_area.x + layout.body_area.width + 0.001
                );
            }
        }
    }
}
