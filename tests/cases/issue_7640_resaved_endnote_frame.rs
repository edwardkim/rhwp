//! 검사 유형: 관계형(소유·순서·원본 프레임·비겹침), 고정형(독립 Print 21쪽 쪽수).
//! 주석 갱신: 2026-11-06 02:08:21 KST.
//! 직전 파일 변경: 2026-11-06 01:18:55 KST, e9c789b7ab (Native 단계 증적).
//! 2026-11-06 canonical raw 2px, 96DPI, compat2022, Native 직접 PNG 전쪽 측정.
//! 재저장본 21/21쪽 Native 최저 99.03802%; 11쪽 99.66679%, 17쪽 99.14958%,
//! 18쪽 99.03802%, 19쪽 99.47824%, 20쪽 99.73017%.
//! 같은 구현의 fresh WASM 재저장본 21/21쪽 최저 99.02457%(18쪽), 원본 최저 98.77736%.
//! 원본 21/21쪽 Native 최저 98.78595%; PR7600 input-09 Print로 따로 대조했다.
//! 최초 작성 시 재저장본 7쪽 미달·11쪽 53.60031%, 17쪽 94.69512%, 18쪽 85.61347%였다.
//! 실물 절대 px 좌표는 고정하지 않으며 현재 저장 프레임의 정수 단위 보존을 비교한다.
//! 근거: mydocs/plans/task_m100_7640_plan.md 및 같은 입력의 독립 한컴 2020 Print.
//! 단계 증적: mydocs/pr/assets/issue_7640/verification-20261106/current-frame-advance/.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn equation_columns(node: &RenderNode, column: Option<u16>, out: &mut Vec<u16>) {
    let column = match node.node_type {
        RenderNodeType::Column(index) => Some(index),
        _ => column,
    };
    if let RenderNodeType::Equation(equation) = &node.node_type {
        if equation.note_ref.as_ref().is_some_and(|source| {
            source.kind == "endnote"
                && source.section_index == 0
                && source.para_index == 99
                && source.control_index == 0
                && source.note_para_index == 8
        }) {
            out.push(column.expect("미주 수식의 실제 단 소속"));
        }
    }
    for child in &node.children {
        equation_columns(child, column, out);
    }
}

#[test]
fn original_and_resaved_last_integral_remain_in_the_first_column() {
    for input in [
        "samples/3-11월_실전_통합_2022.hwp",
        "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp",
    ] {
        let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
        document.set_hangul2024_compat(false);
        assert_eq!(document.page_count(), 21, "독립 Print의 전체 쪽수: {input}");
        let page = document.build_page_render_tree(10).unwrap();
        let mut columns = Vec::new();
        equation_columns(&page.root, None, &mut columns);
        assert_eq!(
            columns,
            [0, 0],
            "한컴 Print 11쪽 왼쪽 단 끝의 g(x)와 적분식이 누락·중복 없이 같은 단에 남는다: {input}"
        );
    }
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn host_height(input: &str, protected: bool, fraction: bool) -> f64 {
    let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let mut source = document.document().clone();
    let Control::Endnote(note) = &mut source.sections[0].paragraphs[88].controls[0] else {
        panic!("원본 미주 11의 소유");
    };
    let Control::Equation(equation) = &mut note.paragraphs[2].controls[0] else {
        panic!("코사인 법칙 앞의 원본 수식");
    };
    equation.common.size_protect = protected;
    if fraction {
        // 별도 복합 수식의 선언 프레임을 단순 글자 줄의 논리 프레임으로 바꾸지 않는다.
        equation.script = "1 over 2".into();
        equation.common.height = equation.font_size;
        equation.baseline = 86;
    }
    document.set_document(source);
    document.set_hangul2024_compat(false);
    let page = document.build_page_render_tree(10).unwrap();
    let mut all = Vec::new();
    walk(&page.root, &mut all);
    let owners: Vec<_> = all
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Equation(equation)
                if equation.note_ref.as_ref().is_some_and(|source| {
                    source.kind == "endnote"
                        && source.para_index == 88
                        && source.note_para_index == 2
                }) =>
            {
                equation.para_index
            }
            _ => None,
        })
        .collect();
    assert_eq!(owners.len(), 1, "실제 수식의 소유가 유일해야 한다");
    let rows: Vec<_> = all.iter().filter(|node| matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(owners[0]) && line.line_index == Some(0)))
        .collect();
    assert_eq!(rows.len(), 1, "실제 호스트 줄이 유일해야 한다");
    rows[0].bbox.height
}

#[test]
fn protected_and_complex_frames_keep_their_separate_stored_row() {
    let original = "samples/3-11월_실전_통합_2022.hwp";
    let resaved = "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp";
    let normal = host_height(original, false, false);
    assert_eq!(
        normal,
        host_height(resaved, false, false),
        "같은 현재 수식의 줄 점유"
    );
    let protected = host_height(original, true, false);
    assert!(
        protected > normal,
        "보호된 저장 예약 높이는 자연 글자 줄보다 크다"
    );
    assert_eq!(protected, host_height(resaved, true, false));
    assert_eq!(
        protected,
        host_height(resaved, false, true),
        "복합 수식의 저장 줄 예약 유지"
    );
}

fn reserved_rows(input: &str, remove_following_owner: bool) -> (f64, f64) {
    let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    if remove_following_owner {
        let mut source = document.document().clone();
        let Control::Endnote(note) = &mut source.sections[0].paragraphs[318].controls[0] else {
            panic!("원본 미주 28의 소유");
        };
        let Control::Equation(equation) = &mut note.paragraphs[3].controls[2] else {
            panic!("다음 줄의 분수 수식");
        };
        // 저장 줄 숫자만 같은 경우와 실제 후속 개체가 큰 높이를 소유하는 경우를 구분한다.
        equation.common.height = equation.font_size;
        document.set_document(source);
    }
    document.set_hangul2024_compat(false);
    let page = document.build_page_render_tree(16).unwrap();
    let mut all = Vec::new();
    walk(&page.root, &mut all);
    let para = all
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Equation(equation)
                if equation.control_index == Some(0)
                    && equation.note_ref.as_ref().is_some_and(|source| {
                        source.kind == "endnote"
                            && source.section_index == 0
                            && source.para_index == 318
                            && source.control_index == 0
                            && source.note_para_index == 3
                    }) =>
            {
                equation.para_index
            }
            _ => None,
        })
        .expect("미주 28 수식 줄의 실제 원본 소유");
    let height = |index| {
        let rows: Vec<_> = all.iter().filter(|node| matches!(&node.node_type,
            RenderNodeType::TextLine(line) if line.para_index == Some(para) && line.line_index == Some(index)))
            .collect();
        assert_eq!(rows.len(), 1, "같은 소유의 글줄은 한 번만 표시된다");
        rows[0].bbox.height
    };
    (height(0), height(1))
}

#[test]
fn next_fraction_owns_reserved_height_after_the_short_equation_row() {
    let original = reserved_rows("samples/3-11월_실전_통합_2022.hwp", false);
    let input = "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp";
    let resaved = reserved_rows(input, false);
    assert!(
        original.0 < original.1,
        "독립 Print의 짧은 첫 수식과 다음 분수 줄"
    );
    assert!(
        resaved.0 < resaved.1,
        "다음 분수의 큰 높이를 앞의 짧은 수식이 중복 예약하지 않는다"
    );
    assert_eq!(
        original, resaved,
        "원본과 동일하게 인쇄되는 재저장본의 줄별 점유"
    );
    assert!(
        reserved_rows(input, true).0 > resaved.0,
        "뒤 개체가 큰 높이를 소유하지 않으면 저장 캐시 숫자만으로 예약 높이를 앞줄에서 제거하지 않는다"
    );
}

#[test]
fn current_bar_frames_preserve_their_source_height_unit() {
    let input = "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp";
    let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    document.set_hangul2024_compat(false);
    let Control::Endnote(note) = &document.document().sections[0].paragraphs[404].controls[0]
    else {
        panic!("타원 문제 미주의 원본 소유");
    };
    let expected: Vec<_> = [1, 2]
        .into_iter()
        .flat_map(|paragraph| {
            note.paragraphs[paragraph]
                .controls
                .iter()
                .enumerate()
                .filter_map(move |(control, value)| match value {
                    Control::Equation(equation) if equation.script.contains("bar") => {
                        Some((paragraph, control, equation.common.height))
                    }
                    _ => None,
                })
        })
        .collect();
    assert_eq!(expected.len(), 5, "프라임 유무를 포함한 실제 윗줄 수식");
    let page = document.build_page_render_tree(18).unwrap();
    let mut all = Vec::new();
    walk(&page.root, &mut all);
    for (paragraph, control, source_height) in expected {
        let nodes: Vec<_> = all
            .iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::Equation(equation)
                if equation.control_index == Some(control)
                    && equation.note_ref.as_ref().is_some_and(|source|
                        source.kind == "endnote" && source.section_index == 0
                            && source.para_index == 404 && source.control_index == 0
                            && source.note_para_index == paragraph))
            })
            .collect();
        assert_eq!(nodes.len(), 1, "현재 장식 수식은 누락·중복 없이 표시된다");
        assert_eq!(
            nodes[0].bbox.height,
            rhwp::renderer::hwpunit_to_px(source_height as i32, 96.0),
            "현재 논리 프레임과 같은 저장 높이에서 상쇄 연산으로 정수 단위를 잃지 않는다: {paragraph}:{control}"
        );
    }
}

#[test]
fn ellipse_tail_integral_keeps_its_print_column_after_resave() {
    for input in [
        "samples/3-11월_실전_통합_2022.hwp",
        "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp",
    ] {
        let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
        document.set_hangul2024_compat(false);
        let page = document.build_page_render_tree(18).unwrap();
        fn collect(node: &RenderNode, column: Option<u16>, out: &mut Vec<u16>) {
            let column = match node.node_type {
                RenderNodeType::Column(index) => Some(index),
                _ => column,
            };
            if let RenderNodeType::Equation(equation) = &node.node_type {
                if equation.control_index == Some(0)
                    && equation.note_ref.as_ref().is_some_and(|source| {
                        source.kind == "endnote"
                            && source.section_index == 0
                            && source.para_index == 404
                            && source.control_index == 0
                            && source.note_para_index == 6
                    })
                {
                    out.push(column.expect("실제 미주 단 소속"));
                }
            }
            for child in &node.children {
                collect(child, column, out);
            }
        }
        let mut columns = Vec::new();
        collect(&page.root, None, &mut columns);
        assert_eq!(
            columns,
            [1],
            "독립 Print 19쪽 오른쪽 단의 마지막 적분식이 다음 쪽으로 밀리지 않는다: {input}"
        );
    }
}

#[test]
fn refreshed_equation_host_rebuilds_the_shared_print_line_partition() {
    for input in [
        "samples/3-11월_실전_통합_2022.hwp",
        "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp",
    ] {
        let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
        document.set_hangul2024_compat(false);
        let page = document.build_page_render_tree(17).unwrap();
        let mut all = Vec::new();
        walk(&page.root, &mut all);
        let mut equations: Vec<_> = all
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::Equation(equation)
                    if equation.note_ref.as_ref().is_some_and(|source| {
                        source.kind == "endnote"
                            && source.section_index == 0
                            && source.para_index == 340
                            && source.control_index == 0
                            && source.note_para_index == 26
                    }) =>
                {
                    Some((
                        equation.control_index.unwrap(),
                        equation.para_index.unwrap(),
                        node.bbox,
                    ))
                }
                _ => None,
            })
            .collect();
        equations.sort_by_key(|(control, _, _)| *control);
        assert_eq!(equations.len(), 3, "같은 문단의 원본 수식 소속·누락·중복");
        assert_eq!(
            equations
                .iter()
                .map(|(control, _, _)| *control)
                .collect::<Vec<_>>(),
            [0, 1, 2],
            "원본 수식 순서"
        );
        assert!(
            equations[2].2.y >= equations[0].2.y + equations[0].2.height,
            "독립 Print 18쪽의 마지막 f'(0)=0은 다음 글줄에 남는다: {input}"
        );
        let rows: Vec<_> = all
            .iter()
            .filter(|node| {
                matches!(&node.node_type,
            RenderNodeType::TextLine(line) if line.para_index == Some(equations[0].1))
            })
            .collect();
        assert_eq!(rows.len(), 2, "두 입력의 Print 글줄 소속: {input}");
        fn text(node: &RenderNode, value: &mut String) {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                value.push_str(&run.text);
            }
            for child in &node.children {
                text(child, value);
            }
        }
        let mut actual_text = String::new();
        for row in rows {
            text(row, &mut actual_text);
        }
        assert_eq!(actual_text, "에서  또는 이다.", "본문의 누락·중복 없음");
    }
}

#[test]
fn validated_fraction_rows_keep_their_non_overlapping_print_occupancy() {
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
    for input in [
        "samples/3-11월_실전_통합_2022.hwp",
        "tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp",
    ] {
        let mut document = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
        document.set_hangul2024_compat(false);
        let page = document.build_page_render_tree(18).unwrap();
        let mut all = Vec::new();
        walk(&page.root, &mut all);
        let owners: Vec<_> = all
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextLine(line) if text(node).contains("을 대입하여 정리하면") => {
                    line.para_index
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            owners.len(),
            1,
            "독립 Print의 대입·미분 문단이 한 번만 표시된다"
        );
        let rows: Vec<_> = all
            .iter()
            .filter(|node| {
                matches!(&node.node_type,
            RenderNodeType::TextLine(line) if line.para_index == Some(owners[0]))
            })
            .collect();
        assert_eq!(rows.len(), 2, "유효한 분수 문단의 두 글줄 소속");
        assert!(
            rows[1].bbox.y >= rows[0].bbox.y + rows[0].bbox.height,
            "뒤 글줄이 앞 분수의 실제 점유 상자를 침범하지 않는다: {input}"
        );
        assert_eq!(text(rows[1]), "이다.", "뒤 본문의 소속·누락·중복 없음");
    }
}
