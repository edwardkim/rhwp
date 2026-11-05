//! 검사 유형: 관계형. 독립 한컴 Print가 같은 원본·재저장본의 수식 단 소속을 검사한다.
//! 작성 시 Native 전 21쪽 중 7쪽 미달·11쪽 53.60031%. 보정 후보의 11쪽은 99.66679%다.
//! 추가한 저장 예약 소유 검사의 17쪽 Native 작성 기준은 94.69512%다.
//! 현재 장식 프레임 검사의 19쪽 Native 작성 기준은 95.41891%다.
//! 장식 검사는 절대 화면 좌표가 아니라 현재 저장 프레임의 단위 보존과 실제 단 소속을 비교한다.
//! 작성 기준에서 전쪽 90% 조건을 충족하지 못해 절대 px 핀을 두지 않았다.
//! 이 단계의 fresh WASM은 미실행이며 최신 전쪽 판정은 계획서와 단계 증적을 따른다.
//! 주석 갱신: 2026-11-06 00:13:00 KST.
//! 근거: mydocs/plans/task_m100_7640_plan.md 및 같은 입력의 한컴 2020 Print 11쪽.
//! 수식 프레임의 저장 캐시 차이로 앞 단에 들어갈 마지막 수식이 다음 단으로 밀리면 실패한다.

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
