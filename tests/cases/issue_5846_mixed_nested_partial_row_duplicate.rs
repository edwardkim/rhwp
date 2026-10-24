//! #5846: 혼재 호스트의 내부 행은 쪽별 원본 내용과 표/문단 순서를 보존한다.
//!
//! 독립 기준은 `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`이다.
//! 2·3쪽의 현재 Native/fresh WASM raw 99%대와 직접 판독을 근거로 아래 경계를 보호한다.
//! 과거 전 66쪽 미달은 stage8 C436에, 현재 90% 이상·65/66쪽 100%는 Stage53·54에 기록한다.
//! 59쪽은 앞 행, 60쪽은 단기연체정보부터 서명까지의 뒤 행을 소유한다.
//! 쪽·셀 소속, 포함 및 표 뒤 문단 관계를 유지하고 완료된 2·3쪽만 독립 PDF의 ±2px를 쓴다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;
use rhwp::DocumentCore;
use std::path::Path;

const SAMPLE: &str = "samples/task2097/75544_pii_bunseok.hwpx";
// 좌표 기대값이 아니라 HU 변환의 부동소수 끝자리만 흡수한다.
const FLOAT_EPSILON: f64 = 1e-6;

fn text(node: &RenderNode) -> String {
    if !node.visible {
        return String::new();
    }
    let mut out = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.extend(run.display_or_text().chars().filter(|c| !c.is_whitespace()));
    }
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

fn owning_cell<'a>(node: &'a RenderNode, marker: &str) -> Option<&'a RenderNode> {
    node.children
        .iter()
        .find_map(|child| owning_cell(child, marker))
        .or_else(|| {
            (matches!(node.node_type, RenderNodeType::TableCell(_)) && text(node).contains(marker))
                .then_some(node)
        })
}

fn line<'a>(node: &'a RenderNode, marker: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(marker) {
        return Some(node);
    }
    node.children.iter().find_map(|child| line(child, marker))
}

fn table_with<'a>(cell: &'a RenderNode, marker: &str) -> &'a RenderNode {
    cell.children
        .iter()
        .find(|child| {
            matches!(child.node_type, RenderNodeType::Table(_)) && text(child).contains(marker)
        })
        .unwrap_or_else(|| panic!("{marker} 표가 실제 소유 셀에 없음"))
}

fn owned_table(node: &RenderNode, para: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(para) && table.control_index == Some(0))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| owned_table(child, para))
}

fn source_cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col)
        })
        .unwrap_or_else(|| panic!("원본 칸 ({row}, {col}) 소실"))
}

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

#[test]
fn issue_5846_cut_fragment_defers_partial_nested_tail_row() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("개인정보 원본 읽기");
    let source = DocumentCore::from_bytes(&bytes).expect("독립 원본 속성 읽기");
    let source_paras = &source.document().sections[0].paragraphs;
    let mut doc = HwpDocument::from_bytes(&bytes).expect("개인정보 원본 파싱");
    // 전쪽 시각 수용과 같은 선언 글꼴 환경에서 관계를 검사한다.
    // 미달이 남은 기본 환경의 corpus 수용을 대신하지 않는다.
    let environment = rhwp::renderer::font_environment::FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
    ))
    .expect("독립 Print와 대조한 선언 글꼴 환경");
    doc.set_font_environment(Some(environment))
        .expect("측정과 paint에 같은 선언 환경 적용");
    assert_eq!(doc.page_count(), 66, "독립 Print PDF와 전체 쪽수");
    // 독립 Print 2→3쪽 경계: 영향평가 내용은 앞쪽에서 끝나지만 같은
    // 행의 빈 물리 공간은 뒤쪽 상단에 남는다. 비용 행은 그 공간 뒤에서
    // 시작해야 하며, 내용이 끝났다는 이유로 행 상자를 생략하지 않는다.
    let opening = doc.build_page_render_tree(1).expect("2쪽 렌더");
    let closing = doc.build_page_render_tree(2).expect("3쪽 렌더");
    let first_table = owned_table(&opening.root, 13).expect("앞 규제 표 조각");
    let next_table = owned_table(&closing.root, 13).expect("뒤 규제 표 조각");
    for (tree, table) in [(&opening.root, first_table), (&closing.root, next_table)] {
        let area = body(tree).expect("문서 본문");
        assert!(
            table.bbox.y >= area.bbox.y - FLOAT_EPSILON
                && table.bbox.y + table.bbox.height
                    <= area.bbox.y + area.bbox.height + FLOAT_EPSILON,
            "행 조각의 예약과 실제 상자는 본문 세로 영역 안에 있어야 함"
        );
    }
    let cost = source_cell(next_table, 10, 2);
    // 192DPI 전쪽 sweep의 완료 페이지 2·3: Native 99.69840/99.53257%,
    // fresh WASM 99.70746/99.54130%, review/overlay 확대 판독 완료.
    // 아래 기대값은 동일 Print PDF의 원래 벡터 선 좌표를 96dpi CSS px로 환산한 값이다.
    // 칸 높이 또는 이번 remainder helper의 출력을 복사하지 않는다.
    // 증적: mydocs/pr/assets/issue7445/completed-20261009/
    // rebase-4a07d3ae-20261019/pii-independent-row-minimum/README.md
    let opening_row = source_cell(first_table, 9, 2);
    let opening_bottom = opening_row.bbox.y + opening_row.bbox.height;
    for (name, actual, pdf) in [
        (
            "2쪽 영향평가 행의 하단",
            opening_bottom,
            1_042.698_666_666_666_7,
        ),
        (
            "3쪽 이어받기 표의 상단",
            next_table.bbox.y,
            77.514_666_666_666_7,
        ),
        (
            "3쪽 빈 행 뒤 비용 행의 시작",
            cost.bbox.y,
            95.734_666_666_666_7,
        ),
    ] {
        assert!(
            (actual - pdf).abs() <= 2.0,
            "{name}: 실제 {actual}, 독립 Print {pdf} CSS px"
        );
    }
    for col in [2, 6, 10] {
        let first = source_cell(first_table, 9, col);
        let remaining = source_cell(next_table, 9, col);
        assert_eq!(text(first), "해당없음", "영향평가 응답은 앞쪽 소유");
        assert!(text(remaining).is_empty(), "응답을 뒤쪽에 중복하지 않음");
        assert!(
            remaining.bbox.height > 0.0,
            "소비한 내용 뒤의 물리 잔여 보존"
        );
        assert!(
            (remaining.bbox.y - next_table.bbox.y).abs() <= FLOAT_EPSILON
                && (remaining.bbox.y + remaining.bbox.height - cost.bbox.y).abs() <= FLOAT_EPSILON,
            "같은 빈 행의 잔여가 표 위에서 비용 행까지 이어져야 함"
        );
    }
    let cost_marker = "종전과동일한수준으로서";
    assert!(
        !text(first_table).contains(cost_marker),
        "비용 본문을 앞쪽에 소비하지 않음"
    );
    assert_eq!(
        text(cost).matches(cost_marker).count(),
        1,
        "비용 본문은 뒤쪽에 한 번 보존"
    );
    let before = doc.build_page_render_tree(58).expect("59쪽 렌더");
    let after = doc.build_page_render_tree(59).expect("60쪽 렌더");
    assert!(
        text(&before.root).contains("채무불이행정보"),
        "앞 행의 끝 내용은 59쪽 소유"
    );
    for marker in [
        "단기연체정보",
        "금융질서문란정보",
        "※작성요령",
        "○○금융회사대표이사△△△",
    ] {
        assert!(
            !text(&before.root).contains(marker),
            "뒤 행 {marker}가 앞쪽에 중복됨"
        );
        assert_eq!(
            text(&after.root).matches(marker).count(),
            1,
            "뒤 행 {marker}가 60쪽에 한 번 표시되어야 함"
        );
    }
    let cell = owning_cell(&after.root, "금융질서문란정보").expect("뒤 행 소유 셀");
    if let RenderNodeType::TableCell(model) = &cell.node_type {
        assert_eq!(model.row, 1, "원본 둘째 행 소유");
    }
    let first = line(cell, "단기연체정보").expect("첫 항목 줄");
    let following = line(cell, "금융질서문란정보").expect("다음 항목 줄");
    let first_table = table_with(cell, "연체금액");
    let second_table = table_with(cell, "사유");
    assert!(
        first.bbox.y + first.bbox.height <= first_table.bbox.y + FLOAT_EPSILON,
        "첫 항목 뒤에 연체 표가 놓여야 함"
    );
    assert!(
        first_table.bbox.y + first_table.bbox.height <= following.bbox.y + FLOAT_EPSILON,
        "다음 항목이 첫 표 안으로 올라가면 안 됨"
    );
    assert!(
        following.bbox.y + following.bbox.height <= second_table.bbox.y + FLOAT_EPSILON,
        "다음 항목 뒤에 해당 표가 놓여야 함"
    );
    for marker in [
        "단기연체정보",
        "금융질서문란정보",
        "※작성요령",
        "○○금융회사대표이사△△△",
    ] {
        let content = line(cell, marker).unwrap_or_else(|| panic!("{marker} 줄 소실"));
        assert!(
            content.bbox.y >= cell.bbox.y - FLOAT_EPSILON
                && content.bbox.y + content.bbox.height
                    <= cell.bbox.y + cell.bbox.height + FLOAT_EPSILON,
            "{marker} 줄이 원본 소유 셀 안에 있어야 함"
        );
    }

    // 전66쪽 Native/fresh 수용 후 보강한 경계다. 표 하단은 후속 원본
    // 문단의 원점에서 아래 바깥여백을 제외한 곳이다. 절대 px 위치나
    // 구현 helper의 반환값 대신 원본 속성과 실제 두 상자의 관계를 검사한다.
    let mut frame_failures = Vec::new();
    for (page, host) in [(56, 517), (57, 520)] {
        let Control::Table(original) = &source_paras[host].controls[0] else {
            panic!("원본 종료 프레임의 표 소실");
        };
        let next = &source_paras[host + 1].line_segs[0];
        let original_cell = &original.cells[0];
        assert_eq!(
            i64::from(original_cell.height) - i64::from(original.common.height)
                + i64::from(original.outer_margin_top)
                + i64::from(original.outer_margin_bottom),
            i64::from(next.vertical_pos),
            "독립 원본의 두 표 상자와 후속 원점이 같은 프레임을 닫음"
        );
        let tree = doc.build_page_render_tree(page).expect("종료 프레임 렌더");
        let area = body(&tree.root).expect("원본 쪽 본문");
        let painted = owned_table(&tree.root, host).expect("원본 소유 종료 표");
        let cell = source_cell(painted, 0, 0);
        let successor_origin = hwpunit_to_px(next.vertical_pos, DEFAULT_DPI);
        let closed_origin = cell.bbox.y
            + cell.bbox.height
            + hwpunit_to_px(i32::from(original.outer_margin_bottom), DEFAULT_DPI)
            - area.bbox.y;
        if (closed_origin / successor_origin - 1.0).abs() > FLOAT_EPSILON {
            frame_failures.push(format!(
                "{}쪽: 표 하단과 후속 원점 사이 아래여백을 중복 예약함 (비율 {})",
                page + 1,
                closed_origin / successor_origin
            ));
        }
    }

    // 비점유 ColumnDef(1)이 있어도 첫 빈 저장 줄의 줄 상자는 보존한다.
    // 안내 제목과 중첩 표는 각자 원본의 상대 원점을 한 번만 사용한다.
    let Control::Table(original) = &source_paras[527].controls[0] else {
        panic!("원본 안내서식 소실");
    };
    let original_cell = &original.cells[0];
    let empty = &original_cell.paragraphs[0];
    assert!(empty.text.is_empty());
    assert!(empty.controls.iter().all(|control| {
        matches!(control, Control::ColumnDef(columns) if columns.column_count == 1)
    }));
    let original_title = &original_cell.paragraphs[1];
    let original_child = original_title
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table.as_ref())
            } else {
                None
            }
        })
        .expect("원본 안내 제목의 중첩 표");
    let outer = owned_table(&before.root, 527).expect("59쪽 안내서식 소유 표");
    let outer_cell = source_cell(outer, 0, 0);
    let title = line(outer_cell, "자동화평가결과안내").expect("원본 안내 제목 줄");
    let empty_line = outer_cell.children.iter().find(|child| {
        matches!(&child.node_type, RenderNodeType::TextLine(data) if data.para_index == Some(0))
    }).expect("원본 첫 빈 줄의 물리 상자");
    let source_lead = original_title.line_segs[0].vertical_pos - empty.line_segs[0].vertical_pos;
    assert_eq!(
        source_lead,
        empty.line_segs[0].line_height + empty.line_segs[0].line_spacing
    );
    if ((title.bbox.y - empty_line.bbox.y) / hwpunit_to_px(source_lead, DEFAULT_DPI) - 1.0).abs()
        > FLOAT_EPSILON
    {
        frame_failures.push("59쪽: 첫 빈 줄과 제목의 원본 줄 상자 연결을 잃음".into());
    }
    let child = table_with(outer_cell, "귀하가");
    let child_origin = source_cell(child, 0, 0).bbox.y;
    let source_offset = hwpunit_to_px(original_child.common.vertical_offset as i32, DEFAULT_DPI);
    assert!(source_offset > 0.0, "독립 원본의 양수 문단 상대 앵커");
    if ((child_origin - title.bbox.y) / source_offset - 1.0).abs() > FLOAT_EPSILON {
        frame_failures.push(format!(
            "59쪽: 제목과 중첩 표가 서로 다른 원점을 소비함 (비율 {})",
            (child_origin - title.bbox.y) / source_offset
        ));
    }

    // 마지막 내부 표의 정확한 원본 컷 뒤에는 이미 소유한 유닛을 다시
    // 예약하지 않는다. 두 쪽의 텍스트가 원문을 정확히 한 번 분할하고,
    // 마지막 빈 host 줄 뒤의 여유가 또 하나의 줄 상자가 되지 않아야 한다.
    let Control::Table(original) = &source_paras[619].controls[0] else {
        panic!("원본 마지막 설명 표 소실");
    };
    let original_cell = original
        .cells
        .iter()
        .find(|cell| cell.row == 4 && cell.col == 1)
        .expect("원본 마지막 설명 칸");
    let original_child = original_cell.paragraphs[0]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table.as_ref())
            } else {
                None
            }
        })
        .expect("원본 설명의 내부 표");
    let expected_text: String = original_child.cells[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.text.chars())
        .filter(|ch| !ch.is_whitespace())
        .collect();
    // 독립 Print의 제목은 원 글리프를 유지하며 결측 한점 리더만
    // 한컴바탕으로 출력한다. 이 기호는 옆 한글보다 좁아야 하며,
    // 다음 글자도 측정한 같은 전진폭 뒤에서 시작해야 한다.
    // 좌표나 현재 쪽 전체 폭을 고정하지 않는다.
    let mut default_doc = HwpDocument::from_bytes(&bytes).expect("기본 제품 경로의 같은 원본");
    for (title_doc, explicit_glyph_advance) in [(&mut doc, true), (&mut default_doc, false)] {
        assert_eq!(
            title_doc.font_environment().is_some(),
            explicit_glyph_advance,
            "결측 기호 공급만으로 기본 세션에 글꼴 환경을 승격하지 않음"
        );
        for page_index in [39, 64] {
            let page = title_doc
                .build_page_render_tree(page_index)
                .expect("원본 대안 분석 제목 쪽");
            let title = line(&page.root, "각대안의활동별비용․편익분석결과")
                .expect("원본 제목 내용과 쪽 소유");
            assert_eq!(
                text(title).matches('․').count(),
                1,
                "원본 제목 기호를 한 번 보존"
            );
            // 독립 Print 40·65쪽의 표 없는 소제목처럼 보이는 줄은 실제로
            // 빈 host의 1×1 표다. 저장 후속 줄이 높이+상하 바깥여백을
            // 닫으면 title의 쪽 원점에서 위여백 안쪽에 표를 그려야 한다.
            // 좌우여백 유무가 세로 inset을 없애지 않으며 절대 px를 고정하지 않는다.
            let area = body(&page.root).expect("소제목의 본문 원점");
            for (owner, host) in source_paras.iter().enumerate() {
                let Some(painted) = owned_table(&page.root, owner) else {
                    continue;
                };
                let Some(Control::Table(original)) = host.controls.first() else {
                    continue;
                };
                if original.outer_margin_top <= 0 || original.cells.len() != 1 {
                    continue;
                }
                let Some(anchor) = host.line_segs.first() else {
                    continue;
                };
                let Some(next) = source_paras
                    .get(owner + 1)
                    .and_then(|p| p.line_segs.first())
                else {
                    continue;
                };
                let reserved = i64::from(original.common.height)
                    + i64::from(original.outer_margin_top)
                    + i64::from(original.outer_margin_bottom);
                if i64::from(next.vertical_pos) - i64::from(anchor.vertical_pos) != reserved {
                    continue;
                }
                let outer_top = area.bbox.y + hwpunit_to_px(anchor.vertical_pos, DEFAULT_DPI);
                let margin = hwpunit_to_px(i32::from(original.outer_margin_top), DEFAULT_DPI);
                assert!(((painted.bbox.y - outer_top) / margin - 1.0).abs() < FLOAT_EPSILON,
                    "저장 바깥 상자의 위여백을 테두리 원점에서 한 번 소비: owner={owner}, inset/margin={}",
                    (painted.bbox.y - outer_top) / margin);
                let occupied_bottom = area.bbox.y + hwpunit_to_px(next.vertical_pos, DEFAULT_DPI);
                assert!(
                    painted.bbox.y + painted.bbox.height <= occupied_bottom + FLOAT_EPSILON,
                    "소제목 paint와 다음 host가 같은 예약 상자 안에 있어야 함"
                );
            }
            let svg = title_doc
                .render_page_svg(page_index as u32)
                .expect("동일 제목의 실제 SVG 출력");
            let xml = roxmltree::Document::parse(&svg).expect("최종 SVG 파싱");
            let dot = xml
                .descendants()
                .find(|node| {
                    node.has_tag_name("text")
                        && node.text() == Some("․")
                        && node
                            .attribute("font-family")
                            .is_some_and(|family| family.contains("휴먼명조"))
                })
                .expect("원본 제목 기호의 실제 paint 소유");
            let next = dot.next_sibling_element().expect("기호 다음 실제 글자");
            let following = next.next_sibling_element().expect("그 다음 실제 글자");
            assert_eq!(next.text(), Some("편"));
            assert_eq!(following.text(), Some("익"));
            let origin = |node: roxmltree::Node<'_, '_>| {
                node.attribute("x").unwrap().parse::<f64>().unwrap()
            };
            let dot_advance = origin(next) - origin(dot);
            let hangul_advance = origin(following) - origin(next);
            assert!(
                dot_advance > 0.0 && dot_advance < hangul_advance / 2.0,
                "독립 Print의 결측 기호 공급자는 기본/명시 세션 모두 자연 전진을 사용함: \
                 explicit={explicit_glyph_advance}, dot={dot_advance}, Hangul={hangul_advance}"
            );
            if page_index == 64 {
                // 같은 제목 동작의40쪽은97.84795/97.86223%라 관계만 검사한다.
                // 전66쪽 양쪽90% 이상,65쪽은98.17737/98.18230%와 확대 판독을
                // 충족했다. 독립 Print 기호와 뒤 한글의 원점을 ±2 CSSpx로 보호한다.
                let baseline = dot.attribute("y").unwrap().parse::<f64>().unwrap();
                for (actual, independent) in [
                    (origin(dot), 320.8),
                    (origin(next), 324.32),
                    (baseline, 92.64),
                ] {
                    assert!(
                        (actual - independent).abs() <= 2.0,
                        "독립 Print65쪽의 기호/다음 글자 원점: 실제{actual}, 독립{independent}"
                    );
                }
            }
            assert_eq!(
                dot.attribute("font-size"),
                next.attribute("font-size"),
                "대체 글리프 공급이 원본 제목의 글자 크기를 바꾸지 않음"
            );
            assert!(
                dot.attribute("font-family")
                    .unwrap()
                    .contains("Haansoft Batang"),
                "실제 paint도 독립 Print에서 확인한 글리프 공급 체인을 소비함"
            );
        }
    }
    let page65 = doc.build_page_render_tree(64).expect("65쪽 내부 표 시작");
    let page66 = doc.build_page_render_tree(65).expect("66쪽 내부 표 종료");
    let opening = source_cell(owned_table(&page65.root, 619).expect("앞 설명 표"), 4, 1);
    let closing = source_cell(owned_table(&page66.root, 619).expect("뒤 설명 표"), 4, 1);
    let first_child = table_with(opening, "인공지능");
    let last_child = table_with(closing, "맞추어");
    // 부모 선언이 앞 행 + 자식 첫 프레임 + 부모 상하 안 여백으로 닫힌다.
    // 인증된 첫 조각에서 아래 안 여백을 열린 컷의 여분으로 회수하지 않는다.
    let padding = original_cell.effective_padding(&original.padding);
    let source_opening_height = hwpunit_to_px(
        i32::try_from(original_child.common.height).expect("원본 첫 자식 상자")
            + i32::from(padding.top)
            + i32::from(padding.bottom),
        DEFAULT_DPI,
    );
    assert!(
        (opening.bbox.height / source_opening_height - 1.0).abs() < FLOAT_EPSILON,
        "첫 부모 조각이 자식의 원본 상하 여백을 한 번 소유해야 함: 실제 {}, 원본 {}",
        opening.bbox.height,
        source_opening_height,
    );
    // Native/fresh WASM 전 66쪽 raw 최저 96.57165/96.56850%, 65쪽은 양쪽 100%.
    // 독립 Print 벡터 선의 행 높이(1011.37337→1041.26131 CSSpx)를 ±2px로 보호한다.
    // stage53-pii-stored-child-frame-20261022 및 stage54 후속 증적을 참조한다.
    assert!(
        (opening.bbox.height - 29.887_939_453_125).abs() <= 2.0,
        "65쪽 마지막 행 높이: 실제 {}, 독립 Print 29.88794 CSSpx",
        opening.bbox.height,
    );
    // 독립 H2GTRM hmtx와 Print는 작은따옴표에 1em을 준다. 실제 잉크가
    // 글리프 오른쪽에 있어 좁힌 전진폭은 뒤 한글과 겹쳐 여는 기호를 숨겼다.
    // 원본 글자모양의 크기·장평과 이웃 원점 관계를 검사하며 절대 px를 고정하지 않는다.
    let first_line = line(last_child, "맞추어").expect("66쪽 원본 이어받기 첫 줄");
    let first_text = text(first_line);
    assert!(first_text.contains("맞추어‘완전히자동화된시스템’으로개인정보를처리"));
    assert_eq!(first_text.matches('‘').count(), 1, "여는 기호 한 번 보존");
    assert_eq!(first_text.matches('’').count(), 1, "닫는 기호 한 번 보존");
    let runs: Vec<_> = first_line
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if node.visible => Some((node, run)),
            _ => None,
        })
        .collect();
    for quote in ["‘", "’"] {
        let index = runs
            .iter()
            .position(|(_, run)| run.display_or_text() == quote)
            .expect("원본 언어 슬롯의 작은따옴표 run");
        let (painted, run) = runs[index];
        let original_shape = &source.document().doc_info.char_shapes
            [run.char_shape_id.expect("원본 글자모양 소유") as usize];
        // 이 원본의 작은따옴표는 영문 슬롯이며, 독립 Print와 동일하게 자간이 없다.
        assert_eq!(original_shape.spacings[1], 0);
        let source_em = hwpunit_to_px(original_shape.base_size, DEFAULT_DPI)
            * f64::from(original_shape.relative_sizes[1])
            / 100.0
            * f64::from(original_shape.ratios[1])
            / 100.0;
        assert!(source_em > 0.0);
        assert!(run.style.font_family.contains("HYGothic-Medium"));
        assert!(
            (painted.bbox.width / source_em - 1.0).abs() < 0.01,
            "검증된 TrueType 작은따옴표는 원본 글자모양의 1em을 점유: {quote}, 비율 {}",
            painted.bbox.width / source_em
        );
        assert!(
            run.style.font_metric_trusted,
            "확인된 TrueType 프로그램의 측정 경로"
        );
        let next = runs.get(index + 1).expect("기호 뒤 원문 run").0;
        assert!(
            painted.bbox.x + source_em <= next.bbox.x + source_em * FLOAT_EPSILON,
            "작은따옴표의 독립 프로그램 전진폭 안에 다음 원문 원점이 들어오면 안 됨: {quote}"
        );
    }
    // Studio의 local() 공급은 형식 미확인을 유지한다. 같은 원본의 비신뢰
    // 작은따옴표 폭은 Canvas 실제 측정 준비에 포함되어야 한다. 전쪽 수용을
    // 대신하는 별도 합성 문서를 만들지 않고 원본 요청과 출력 소유를 함께 보호한다.
    let mut local_environment: serde_json::Value = serde_json::from_str(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
    ))
    .unwrap();
    local_environment
        .as_object_mut()
        .unwrap()
        .remove("trueTypeFaces");
    local_environment["id"] = "canvas-local-format-unknown".into();
    let mut canvas = DocumentCore::from_bytes(&bytes).unwrap();
    canvas
        .set_font_environment(Some(
            rhwp::renderer::font_environment::FontEnvironment::from_json(
                &local_environment.to_string(),
            )
            .unwrap(),
        ))
        .unwrap();
    let context = rhwp::renderer::supplemental_metrics::MetricContext {
        document_generation: 1,
        font_generation: 1,
        backend: rhwp::renderer::supplemental_metrics::MetricBackend::Canvas2d,
    };
    let requests = canvas.collect_canvas_metric_requests(context).unwrap();
    for quote in ["‘", "’"] {
        assert!(
            requests.requests.iter().any(|request| {
                request.cluster == quote && request.font.contains("HYGothic-Medium")
            }),
            "원본 작은따옴표 {quote}의 실제 Canvas 폭을 같은 조판 경로에 공급해야 함"
        );
    }
    assert_eq!(
        text(first_child) + &text(last_child),
        expected_text,
        "정확한 자식 컷이 원본 내용을 누락·중복 없이 두 쪽에 한 번 보존"
    );
    let last_para = original_cell.paragraphs.len() - 1;
    assert!(original_cell.paragraphs[last_para].text.is_empty());
    let tail = closing.children.iter().find(|child| {
        matches!(&child.node_type, RenderNodeType::TextLine(data) if data.para_index == Some(last_para))
    }).expect("내부 표 뒤의 원본 빈 host 줄");
    let remaining_band = closing.bbox.y + closing.bbox.height - (tail.bbox.y + tail.bbox.height);
    if remaining_band < -FLOAT_EPSILON || remaining_band >= tail.bbox.height {
        frame_failures.push(format!(
            "66쪽: 정확한 자식 컷 뒤에 이미 소비한 줄 상자를 다시 예약함 (꼬리/줄 비율 {})",
            remaining_band / tail.bbox.height
        ));
    }
    assert!(
        frame_failures.is_empty(),
        "원본 종료 프레임의 관계 위반: {frame_failures:#?}"
    );
}

// Stage55: 동일 소스 Native/fresh WASM 전 66쪽 최저 96.57165/96.56850%, 미달 0.
// 41쪽 99.99530/99.99527%, 42쪽 99.82579/99.82542%와 review/overlay 확대 판독 후 보강한다.
// 독립 Print의 라벨 소유·순서·Center 정렬과 ±2px 원점을 함께 보호한다.
#[test]
fn issue_5846_closed_block_cut_keeps_cost_label_owners_and_alignment() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut doc = HwpDocument::from_bytes(&std::fs::read(path).expect("개인정보 원본")).unwrap();
    let environment_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json",
    );
    let environment = rhwp::renderer::font_environment::FontEnvironment::from_json(
        &std::fs::read_to_string(environment_path).unwrap(),
    )
    .unwrap();
    doc.set_font_environment(Some(environment)).unwrap();
    assert_eq!(doc.page_count(), 66, "독립 Print와 같은 전쪽 수");
    // 독립 Print 41→42쪽의 비용편익 라벨: 첫 줄은 앞쪽, 나머지는 뒤쪽에서 한 번만 소유한다.
    let cost_opening = doc.build_page_render_tree(40).expect("41쪽 비용편익 시작");
    let cost_closing = doc
        .build_page_render_tree(41)
        .expect("42쪽 비용편익 이어받기");
    let cost_first = source_cell(
        owned_table(&cost_opening.root, 401).expect("앞 규제 표"),
        10,
        1,
    );
    let cost_remaining = source_cell(
        owned_table(&cost_closing.root, 401).expect("뒤 규제 표"),
        10,
        1,
    );
    assert_eq!(
        text(cost_first),
        "11.비용편익",
        "닫힌 물리 상자에 맞는 첫 줄의 앞쪽 소유"
    );
    assert_eq!(
        text(cost_remaining),
        "분석(정성분석)",
        "남은 두 줄의 순서·누락·중복"
    );
    let cost_line = line(cost_remaining, "분석").expect("이어받기 첫 라벨 줄");
    let cost_last_line = line(cost_remaining, "정성분석").expect("이어받기 마지막 라벨 줄");
    let label_center =
        (cost_line.bbox.y + cost_last_line.bbox.y + cost_last_line.bbox.height) / 2.0;
    let cell_center = cost_remaining.bbox.y + cost_remaining.bbox.height / 2.0;
    assert!(
        (label_center - cell_center).abs() <= cost_line.bbox.height,
        "완전히 선택한 이어받기 라벨은 원본 Center 정렬을 유지해야 함"
    );
    // 독립 Print PDF에서 추출한 실제 라벨 잉크 상단(CSS96dpi).
    for (actual, expected) in [
        (
            line(cost_first, "11.비용편익").unwrap().bbox.y,
            1_022.118_896_484_375,
        ),
        (cost_line.bbox.y, 194.278_910_319_010_4),
        (cost_last_line.bbox.y, 222.118_916_829_427_06),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print의 비용편익 라벨 원점: 실제 {actual}, 독립 {expected}"
        );
    }
}
