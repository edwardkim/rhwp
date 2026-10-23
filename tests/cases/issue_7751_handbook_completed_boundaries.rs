//! #7751: 독립 KoPub 한컴2024 Print와 대조한 편람의 완료 페이지 경계.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const INPUT: &str = "samples/2025 행정업무운영 편람(최종).hwp";

fn bytes() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT)).expect("편람 원본")
}

fn open() -> DocumentCore {
    let mut core = DocumentCore::from_bytes(&bytes()).expect("편람 로드");
    core.set_hangul2024_compat(true);
    core
}

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn descendants(node: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(descendants(child));
    }
    result
}

fn table(root: &RenderNode, section: usize, para: usize) -> &RenderNode {
    let matches: Vec<_> = descendants(root)
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(section) && t.para_index == Some(para))
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "표 소유는 한 곳이어야 한다: {section}.{para}"
    );
    matches[0]
}

// 본문 문자열에서 분리 저장되는 중첩 표·묶음 글상자의 내용도 원본에서 수집한다.
fn source_paragraph_text(para: &rhwp::model::paragraph::Paragraph) -> String {
    let mut result = para.text.clone();
    for control in &para.controls {
        match control {
            Control::Table(table) => {
                for cell in &table.cells {
                    for para in &cell.paragraphs {
                        result.push_str(&source_paragraph_text(para));
                    }
                }
            }
            Control::Shape(shape) => result.push_str(&source_shape_text(shape)),
            _ => {}
        }
    }
    result
}

fn source_shape_text(shape: &ShapeObject) -> String {
    let mut result = String::new();
    if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
        for para in &tb.paragraphs {
            result.push_str(&source_paragraph_text(para));
        }
    }
    if let ShapeObject::Group(group) = shape {
        for child in &group.children {
            result.push_str(&source_shape_text(child));
        }
    }
    result
}

// 이어받기는 반복 머리글 외의 모든 원본 셀 내용을 앞뒤 조각에서 정확히 한 번 보존한다.
fn assert_cells_preserved(section: usize, para: usize, fragments: &[&RenderNode]) {
    let parsed = rhwp::parser::parse_hwp(&bytes()).expect("원본 셀 내용");
    assert_document_cells_preserved(&parsed, section, para, fragments);
}

fn assert_document_cells_preserved(
    parsed: &rhwp::model::document::Document,
    section: usize,
    para: usize,
    fragments: &[&RenderNode],
) {
    let source = parsed.sections[section].paragraphs[para]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table.as_ref())
            } else {
                None
            }
        })
        .expect("원본 표");
    for cell in &source.cells {
        let expected = compact(
            &cell
                .paragraphs
                .iter()
                .map(source_paragraph_text)
                .collect::<String>(),
        );
        let mut actual = String::new();
        let mut repeated_header = 0;
        for fragment in fragments {
            let matches: Vec<_> = fragment
                .children
                .iter()
                .filter(|n| {
                    matches!(&n.node_type, RenderNodeType::TableCell(c)
                    if c.row == cell.row && c.col == cell.col)
                })
                .collect();
            assert!(
                matches.len() <= 1,
                "한 조각의 셀 중복: {}.{}",
                cell.row,
                cell.col
            );
            for node in matches {
                let part = compact(&text(node));
                if cell.is_header && source.repeat_header {
                    assert_eq!(part, expected, "반복 머리글 내용 보존");
                    repeated_header += 1;
                } else {
                    actual.push_str(&part);
                }
            }
        }
        if cell.is_header && source.repeat_header {
            assert_eq!(repeated_header, fragments.len(), "각 조각의 반복 머리글");
        } else {
            assert_eq!(
                actual, expected,
                "원본 셀 전체 내용·순서·누락·중복: {}.{}",
                cell.row, cell.col
            );
        }
    }
}

fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{label}: 실제 {actual:.3}, 독립 Print {expected:.3}, 허용 ±2px"
    );
}

#[test]
fn issue_7751_intro_reflow_preserves_three_paragraphs_and_print_origins() {
    let core = open();
    assert_eq!(core.page_count(), 383);
    let page = core.build_page_render_tree(2).expect("3쪽");
    let boxes: Vec<_> = descendants(&page.root)
        .into_iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::TextBox) && text(n).contains("이 편람은 1991년")
        })
        .collect();
    assert_eq!(boxes.len(), 1, "서문 글상자의 중복·누락");
    let tb = boxes[0];
    let parsed = rhwp::parser::parse_hwp(&bytes()).expect("원본 내용");
    let Control::Shape(shape) = &parsed.sections[0].paragraphs[4].controls[0] else {
        panic!("묶음")
    };
    let ShapeObject::Group(group) = shape.as_ref() else {
        panic!("묶음")
    };
    let original = group.children[2]
        .drawing()
        .unwrap()
        .text_box
        .as_ref()
        .unwrap();
    assert_eq!(original.paragraphs.len(), 3);

    // PDF -bbox-layout의 첫 행 xMin/yMin을 72pt→96dpi로 환산한다.
    let origins = [
        (200.262813, 536.665688),
        (200.262813, 613.954599),
        (200.262813, 690.426951),
    ];
    let mut previous_bottom = tb.bbox.y;
    for (pi, para) in original.paragraphs.iter().enumerate() {
        let lines: Vec<_> = descendants(tb)
            .into_iter()
            .filter(
                |n| matches!(&n.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(pi)),
            )
            .collect();
        assert_eq!(lines.len(), 3, "한컴 서문의 문단{pi}는3줄");
        assert_eq!(
            compact(&lines.iter().map(|n| text(n)).collect::<String>()),
            compact(&para.text),
            "문단{pi}의 내용·순서·중복 보존"
        );
        assert!(lines[0].bbox.y >= previous_bottom);
        for line in &lines {
            assert!(line.bbox.y >= tb.bbox.y);
            assert!(line.bbox.y + line.bbox.height <= tb.bbox.y + tb.bbox.height);
        }
        near(lines[0].bbox.x, origins[pi].0, "서문 시작 x");
        near(lines[0].bbox.y, origins[pi].1, "서문 시작 y");
        previous_bottom = lines.last().unwrap().bbox.y + lines.last().unwrap().bbox.height;
    }
    let images: Vec<_> = descendants(&page.root)
        .into_iter()
        .filter_map(|n| {
            if let RenderNodeType::Image(i) = &n.node_type {
                Some(i)
            } else {
                None
            }
        })
        .filter(|i| i.bin_data_id == 354)
        .collect();
    assert_eq!(images.len(), 1);
    assert_eq!(
        (images[0].brightness, images[0].contrast),
        (8, 0),
        "한컴 HWP→HWPX의 bright=8, contrast=0"
    );
}

#[test]
fn issue_7751_continuation_end_keeps_heading_and_next_table_in_print_order() {
    let core = open();
    let page = core.build_page_render_tree(240).expect("241쪽");
    let fragment = table(&page.root, 7, 405);
    let next = table(&page.root, 7, 408);
    let previous = core.build_page_render_tree(239).expect("240쪽");
    assert_cells_preserved(7, 405, &[table(&previous.root, 7, 405), fragment]);
    let headings: Vec<_> = descendants(&page.root)
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(407))
                && text(n).contains("점검지표별")
        })
        .collect();
    assert_eq!(headings.len(), 1);
    let heading = headings[0];
    assert!(fragment.bbox.y + fragment.bbox.height <= heading.bbox.y);
    assert!(heading.bbox.y + heading.bbox.height <= next.bbox.y);
    // 독립 PDF의 제목 yMin=282.336pt, 다음 표 첫 행 괘선 y=314.249pt.
    near(heading.bbox.y, 376.448, "이어받기 뒤 제목");
    let cells: Vec<_> = next
        .children
        .iter()
        .filter(|n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 0))
        .collect();
    assert_eq!(cells.len(), 3);
    for cell in cells {
        near(cell.bbox.y, 418.998667, "다음 표 시작 괘선");
    }
    let body = compact(&text(fragment));
    for marker in [
        "⑥연구결과활용의적정성",
        "⑦전년도귄익위",
        "국민집단지성활용사례정성평가",
    ] {
        assert_eq!(
            body.matches(marker).count(),
            1,
            "이어받기 말미 내용의 누락·중복: {marker}"
        );
    }
}

#[test]
fn issue_7751_whole_table_tail_margin_preserves_qa_sequence_and_print_frames() {
    let core = open();
    let previous = core.build_page_render_tree(288).expect("289쪽");
    let page = core.build_page_render_tree(289).expect("290쪽");
    let q13 = table(&previous.root, 10, 27);
    let q14_start = table(&previous.root, 10, 28);
    let q14_end = table(&page.root, 10, 28);
    let q15 = table(&page.root, 10, 29);
    let q16 = table(&page.root, 10, 30);
    assert_cells_preserved(10, 28, &[q14_start, q14_end]);
    assert_cells_preserved(10, 29, &[q15]);
    assert_cells_preserved(10, 30, &[q16]);
    assert!(q13.bbox.y + q13.bbox.height < q14_start.bbox.y);
    assert!(q14_end.bbox.y + q14_end.bbox.height < q15.bbox.y);
    assert!(q15.bbox.y + q15.bbox.height < q16.bbox.y);
    assert!(text(q15).contains("협조의 순서"));
    assert!(text(q16).contains("문서 수정이나 반려"));
    // PDF의 배경 이미지 cm과 실제 표 괘선의 동일한 top-left(96dpi).
    near(q14_start.bbox.y, 397.081333, "Q13 뒤 Q14 시작");
    near(q15.bbox.y, 279.333333, "Q15 시작");
    near(q16.bbox.y, 473.553333, "Q15 종료 여백 뒤 Q16 시작");
    near(q16.bbox.x, 113.317333, "Q16 왼쪽 괘선");
    for (owner, marker) in [
        (q15, "협조의순서는어떻게되는지요"),
        (q16, "협조자는문서수정이나반려를할수없습니다"),
    ] {
        assert_eq!(
            compact(&text(owner)).matches(marker).count(),
            1,
            "Q&A 내용의 누락·중복: {marker}"
        );
    }
    // 같은 입력의 독립 Print p289 B 경로를 CTM 적용 후 96dpi 좌상단으로 변환했다.
    // 문단 여백을 누락했던 그룹도 표/문단 소유를 보존하며 화살표 각각이 제 위치에 있어야 한다.
    let diagrams: Vec<_> = descendants(&previous.root)
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Group(group)
            if group.section_index == Some(10) && group.para_index == Some(28))
                && node.children.iter().any(|child| {
                    matches!(&child.node_type,
                RenderNodeType::Line(line) if line.cell_index == Some(11)
                && line.cell_para_index == Some(2))
                })
        })
        .collect();
    assert_eq!(diagrams.len(), 2, "Q14 두 묶음의 소유·누락·중복");
    for diagram in diagrams {
        let RenderNodeType::Group(group) = &diagram.node_type else {
            unreachable!()
        };
        let expected: &[(f64, f64)] = match group.control_index {
            Some(1) => &[
                (258.76, 669.214667),
                (335.477333, 683.933333),
                (373.536, 720.269333),
                (451.192, 681.853333),
                (478.862667, 718.190667),
            ],
            Some(0) => &[
                (256.522667, 782.964),
                (366.804, 781.204),
                (479.501333, 830.658667),
            ],
            _ => panic!("Q14 묶음 제어 소유"),
        };
        let arrows: Vec<_> = diagram
            .children
            .iter()
            .filter_map(|child| {
                if let RenderNodeType::Line(line) = &child.node_type {
                    Some(line)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(arrows.len(), expected.len(), "묶음 내부 화살표의 누락·중복");
        for (arrow, &(x, y)) in arrows.into_iter().zip(expected) {
            assert_eq!(
                (
                    arrow.section_index,
                    arrow.para_index,
                    arrow.cell_index,
                    arrow.cell_para_index,
                    arrow.outer_table_control_index
                ),
                (Some(10), Some(28), Some(11), Some(2), Some(0)),
                "화살표의 표·셀·문단 소유 보존"
            );
            near(arrow.x1, x, "독립 Print 묶음 화살표 시작 x");
            near(arrow.y1, y, "독립 Print 묶음 화살표 시작 y");
            assert_eq!(arrow.style.end_arrow, rhwp::renderer::ArrowStyle::Arrow);
            assert!(arrow.x2 >= arrow.x1, "화살표는 오른쪽 또는 아래로 진행한다");
        }
        assert!(diagram.bbox.y >= q14_start.bbox.y);
        assert!(diagram.bbox.y + diagram.bbox.height <= q14_start.bbox.y + q14_start.bbox.height);
    }

    // 중첩 표 안의 일반 화살표는 이미 Print 위치였다. 그룹 보정으로 함께 밀면 안 된다.
    let mut pending = vec![q14_start];
    let mut ordinary = Vec::new();
    while let Some(node) = pending.pop() {
        if matches!(node.node_type, RenderNodeType::Group(_)) {
            continue;
        }
        if let RenderNodeType::Line(line) = &node.node_type {
            if line.style.start_arrow != rhwp::renderer::ArrowStyle::None
                || line.style.end_arrow != rhwp::renderer::ArrowStyle::None
            {
                ordinary.push(node);
            }
        }
        pending.extend(node.children.iter());
    }
    ordinary.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
    assert_eq!(ordinary.len(), 2, "중첩 표 일반 화살표의 누락·중복");
    for (arrow, (x, y)) in ordinary
        .into_iter()
        .zip([(364.885333, 795.922667), (366.164, 847.437333)])
    {
        near(arrow.bbox.x, x, "그룹 밖 화살표의 독립 Print x");
        near(arrow.bbox.y, y, "그룹 밖 화살표의 독립 Print y");
    }
}

#[test]
fn issue_7751_reference_mark_preserves_print_advance_before_year_and_law() {
    let core = open();
    let page = core.build_page_render_tree(296).expect("297쪽");
    let owner = table(&page.root, 10, 65);
    let lines: Vec<_> = descendants(owner)
        .into_iter()
        .filter(|n| {
            matches!(n.node_type, RenderNodeType::TextLine(_))
                && text(n).contains("2002.")
                && text(n).contains("사무관리규정")
        })
        .collect();
    assert_eq!(lines.len(), 1, "Q33의 법령 개정 안내 줄 소속·누락·중복");
    let line = lines[0];
    assert_eq!(
        compact(&text(line)),
        compact("※ 2002. 12. 26. ｢사무관리규정｣ 개정(2004. 1. 1. 시행)으로 선람 폐지"),
        "독립 Print 안내 문구의 전체 내용·순서"
    );
    let run_with =
        |needle: &str| {
            let found: Vec<_> = descendants(line).into_iter().filter(|n|
            matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text.contains(needle))
        ).collect();
            assert_eq!(found.len(), 1, "안내 줄 run 소속: {needle}");
            found[0]
        };
    let mark = run_with("※");
    let year = run_with("2002.");
    let law = run_with("사무관리규정");
    assert!(mark.bbox.x + mark.bbox.width <= year.bbox.x);
    assert!(year.bbox.x + year.bbox.width <= law.bbox.x);
    assert!(
        line.bbox.y >= owner.bbox.y
            && line.bbox.y + line.bbox.height <= owner.bbox.y + owner.bbox.height
    );
    // Native/fresh WASM 양쪽 raw 최솟값99%대의 독립 Print297쪽(96dpi)을 우선 선정한다.
    near(mark.bbox.x, 168.48, "주의표 시작");
    near(year.bbox.x, 186.56, "주의표 원본 폭 이후 연도 시작");
    near(law.bbox.x, 274.56, "연도 다음 법령명 시작");
}

fn assert_print_form_tables(input: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(input);
    let bytes = std::fs::read(path).expect("커밋된 편람 원본");
    let mut core = DocumentCore::from_bytes(&bytes).expect("편람 로드");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "같은 원본의 독립 KoPub Print 쪽수");
    let page = core.build_page_render_tree(374).expect("375쪽 서식 두 개");
    let body = descendants(&page.root)
        .into_iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .expect("본문");
    let column = |index| {
        body.children
            .iter()
            .find(|n| matches!(n.node_type, RenderNodeType::Column(i) if i == index))
            .expect("서식 소유 단")
    };
    let left = table(&page.root, 12, 18);
    let right = table(&page.root, 12, 43);
    // 독립 Print의 서식 clipping frame을96dpi로 환산한다. stroke 중심과 구분한다.
    // 양쪽99%대인 완료 페이지의 논리 서식틀과 아래 내용·여백 관계를 함께 보호한다.
    near(left.bbox.x, 96.470667, "왼쪽 서식 외곽 x");
    near(left.bbox.y, 96.216, "왼쪽 서식 외곽 y");
    near(right.bbox.x, 526.509333, "오른쪽 서식 외곽 x");
    near(right.bbox.y, 96.216, "오른쪽 서식 외곽 y");
    let source = rhwp::parse_document(&bytes).expect("원본 서식");
    let Control::Table(left_source) = &source.sections[12].paragraphs[18].controls[0] else {
        panic!("왼쪽 서식");
    };
    let left_margin = f64::from(left_source.outer_margin_left) * 96.0 / 7200.0;
    assert!(left_margin > 0.0);
    assert!(
        (left.bbox.x - column(0).bbox.x - left_margin).abs() < 96.0 / 7200.0,
        "문단 기준 글앞 서식의 선언 바깥여백을 한 번 소비해야 한다"
    );
    assert_document_cells_preserved(&source, 12, 18, &[left]);
    assert_document_cells_preserved(&source, 12, 43, &[right]);
    assert!(
        left.bbox.x + left.bbox.width < right.bbox.x,
        "두 단 서식의 소유·겹침"
    );
    let prefix = &source.sections[12].paragraphs[43];
    assert_eq!(prefix.text, " ");
    assert_eq!(
        prefix.control_text_positions(),
        vec![1],
        "공백은 표 앞에 있다"
    );
    let font = &source.doc_info.char_shapes[prefix.char_shapes[0].char_shape_id as usize];
    let half_em = f64::from(font.base_size) * 96.0 / 7200.0 / 2.0;
    let center_shift =
        right.bbox.x + right.bbox.width / 2.0 - (column(1).bbox.x + column(1).bbox.width / 2.0);
    // 독립 Print에서도 앞 반각 공백 때문에 표 중심이 줄 중심보다 약 반 공백 오른쪽이다.
    // 절대 좌표 핀 대신 원본 글자 크기와 줄·개체 중심의 비율 관계를 검사한다.
    assert!(
        (0.4..0.6).contains(&(center_shift / half_em)),
        "표 앞 공백을 포함한 줄 전체 가운데 정렬: 이동={center_shift}, 반각={half_em}"
    );

    let mut no_margin = source;
    let Control::Table(t) = &mut no_margin.sections[12].paragraphs[18].controls[0] else {
        unreachable!()
    };
    let top_margin = f64::from(t.outer_margin_top) * 96.0 / 7200.0;
    t.outer_margin_left = 0;
    t.outer_margin_top = 0;
    let mut contrast = DocumentCore::new_empty();
    contrast.set_document(no_margin);
    contrast.set_hangul2024_compat(true);
    let other_page = contrast
        .build_page_render_tree(374)
        .expect("바깥여백 없는 대조군");
    let other = table(&other_page.root, 12, 18);
    assert!(
        (left.bbox.x - other.bbox.x - left_margin).abs() < 96.0 / 7200.0,
        "가로 바깥여백 이중 가산 금지"
    );
    assert!(
        (left.bbox.y - other.bbox.y - top_margin).abs() < 96.0 / 7200.0,
        "세로 바깥여백 누락·이중 가산 금지"
    );
    assert!(
        (left.bbox.width - other.bbox.width).abs() < 1e-9
            && (left.bbox.height - other.bbox.height).abs() < 1e-9,
        "바깥여백은 원점을 바꾸며 셀 크기를 바꾸지 않는다"
    );
}

#[test]
fn issue_7751_form_tables_preserve_outer_margins_and_prefix_alignment() {
    assert_print_form_tables(INPUT);
}

#[test]
fn issue_7752_hwpx_completed_form_tables_preserve_owners_content_and_print_frames() {
    assert_print_form_tables("samples/2025 행정업무운영 편람(최종).hwpx");
}

#[test]
fn issue_7751_paragraph_relative_cell_rectangle_matches_print_and_stays_inside_picture() {
    let core = open();
    let page = core.build_page_render_tree(134).expect("135쪽");
    let owner = table(&page.root, 3, 47);
    let rectangles: Vec<_> = descendants(owner)
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Rectangle(rectangle)
            if rectangle.section_index == Some(3) && rectangle.para_index == Some(47)
            && rectangle.control_index == Some(0) && rectangle.cell_index == Some(0)
            && rectangle.cell_para_index == Some(1)
            && rectangle.outer_table_control_index == Some(0))
        })
        .collect();
    assert_eq!(
        rectangles.len(),
        1,
        "원본 강조 사각형의 표·셀·문단 소유·누락·중복"
    );
    let rectangle = rectangles[0];
    let pictures: Vec<_> = descendants(owner)
        .into_iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
        .collect();
    assert_eq!(pictures.len(), 1, "강조할 원본 화면 그림");
    let picture = pictures[0];
    assert!(rectangle.bbox.x >= picture.bbox.x && rectangle.bbox.y >= picture.bbox.y);
    assert!(rectangle.bbox.x + rectangle.bbox.width <= picture.bbox.x + picture.bbox.width);
    assert!(rectangle.bbox.y + rectangle.bbox.height <= picture.bbox.y + picture.bbox.height);
    assert!(compact(&text(owner)).contains("기능분류체계관리"));
    // 독립 Print p135의 빨간 S 경로, CTM 적용 후 96dpi 좌상단 좌표와 크기.
    near(rectangle.bbox.x, 112.198667, "일반 사각형 문단 기준 x");
    near(rectangle.bbox.y, 358.365333, "일반 사각형 문단 기준 y");
    near(rectangle.bbox.width, 66.968, "원본 강조 너비");
    near(rectangle.bbox.height, 43.836, "원본 강조 높이");
}
