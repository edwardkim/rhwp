//! 검사 유형: 고정형과 관계형. 고정형은 한글 2020 재조판 저장본에서 읽은 수식 프레임 값·문단 저장
//! 세로 위치와의 일치, 관계형은 비적용 경로의 기존 추정 크기 유지·문단 사이 간격 보존·큰 합·곱
//! 한계값과 연산자 잉크의 분리다.
//! 주석 갱신: 2026-11-27T12:28:12+09:00, 새 파일.
//! 측정: 2026-11-27, source 82d7439a98(대조 devel a165c6c7d2). 96 DPI·Print·2px raw 실루엣
//! (threshold_boundary_color_support_v1, 색상 허용 32, 경계 보정 0px). 입력은 `base-a4.hwp`에 CLI로
//! 수식 7개를 넣은 `a4-inserted-*.hwp`, 기준은 한글 2022 12.0.0.4645 Print(Hancom PDF, A4, 본문
//! KoPubWorld바탕체 Light 설치) `pdf/issue-8927/a4-inserted-*-2022.pdf`. 수식 원 글꼴(HyhwpEQ·
//! HancomEQN)은 같은 한컴 설치본에서 Native·fresh WASM에 공급했다(저장소에는 없다). 전1쪽,
//! 쪽수 1=1, 90% 미만 없음: HYhwpEQ Native 99.93%(미대응 11px) / fresh WASM 99.95%(8px),
//! HancomEQN Native 99.92%(12px) / fresh WASM 99.94%(10px). devel 렌더러는 같은 입력에서
//! 99.05/99.07%, 99.03/99.05%다. 이 파일은 화소를 비교하지 않는다.
//!
//! # 무엇이 깨져 있었나
//!
//! [#8927] `insert_equation_native`는 공개 글꼴 추정 레이아웃(`intrinsic_size_hwp`)의 폭·높이와
//! 상수 기준선 85를 저장했다. 한글 2020(11.0.0.9166)과 한글 2022 최신 빌드(12.0.0.4637·4645)는
//! 파일을 열 때 저장된 수식 프레임을 그대로 쓰므로, 폭이 모자란 수식은 뒤 글자와 겹치고
//! 기준선이 85로 고정된 분수는 글줄 위로 떴다. 한글 2022 12.0.0.4645 Print에서 글줄 오른쪽 끝의
//! 이동은 저장 폭 오차와 두 글꼴 12줄 모두 1px 안에서 맞았다.
//!
//! # 기대값의 출처
//!
//! - `samples/issue8927/base.hwp`: 이슈 첨부 기준 문서(본문 일곱 문단, 220×280mm). 한글 2020으로
//!   저장됐지만 본문 문단에 저장 줄(LINE_SEG)이 없다.
//! - `samples/issue8927/base-a4.hwp`: `base.hwp`를 한글 2022(12.0.0.4645)에서 열어 편집 용지만 A4로
//!   바꿔 저장한 파일. 저장 줄이 모두 있어 삽입 뒤 세로 위치와 시각 대조의 기준으로 쓴다.
//! - `samples/issue8927/rhwp-HYhwpEQ.hancom-relayout.hwp`, `rhwp-HancomEQN.hancom-relayout.hwp`:
//!   이슈 첨부. 같은 수식 7개를 한글 2020(11.0.0.9166)이 다시 조판해 저장한 파일이다.
//!   테스트는 이 파일에서 수식 프레임을 읽어 기대값으로 쓴다. 구현의 계산값을 재인용하지 않는다.
//!
//! # 남은 차이(검사하지 않는 축)
//!
//! - HancomEQN `prod _{i} x`의 폭: 75HU 넓다(HYhwpEQ는 같다). 곱 뒤 `x`까지의 HancomEQN GDI
//!   전진이 확인되지 않았다. 높이·기준선만 검사한다.
//! - HancomEQN `a ^{2} + b ^{2} = c ^{2}`의 폭: 위첨자 숫자의 한컴 전진폭이 같은 논리 글자 크기에서도
//!   글자 크기마다 달라(10pt 5px, 10.5~11.5pt 6px) 225HU 넓다. 높이·기준선만 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Equation};
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// (문단, 글자 위치, script). 이슈의 재현 순서와 첨부 결과 파일의 배치다.
const CASES: [(usize, usize, &str); 7] = [
    (0, 0, "a ^{2} + b ^{2} = c ^{2}"),
    (1, 4, "alpha + beta LEQ gamma"),
    (2, 4, "a times b"),
    (3, 4, "e ^{i pi} + 1 = 0"),
    (4, 4, "LFLOOR a RFLOOR"),
    (5, 4, "1 over {2 over 3}"),
    (6, 4, "prod _{i} x"),
];

fn sample(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8927")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap()
}

/// 구역 0 문단의 첫 수식과 그 컨트롤 번호.
fn equation_in(doc: &DocumentCore, para: usize) -> (usize, Equation) {
    doc.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::Equation(equation) => Some((index, equation.as_ref().clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("문단 {para}에 수식이 없다"))
}

fn frame(equation: &Equation) -> (u32, u32, i16) {
    (
        equation.common.width,
        equation.common.height,
        equation.baseline,
    )
}

fn inserted(font: Option<&str>) -> DocumentCore {
    let mut doc = sample("base.hwp");
    for (para, offset, script) in CASES {
        doc.insert_equation_native(0, para, offset, script, 1000, 0)
            .unwrap();
    }
    if let Some(font) = font {
        for (para, _, _) in CASES {
            let (index, _) = equation_in(&doc, para);
            doc.set_equation_properties_native(
                0,
                para,
                index,
                None,
                None,
                &format!(r#"{{"fontName":"{font}"}}"#),
            )
            .unwrap();
        }
    }
    doc
}

/// 남은 차이가 있는 수식은 폭을 빼고 높이·기준선만 비교한다.
fn assert_matches_reference(doc: &DocumentCore, reference: &str, width_unresolved: &[&str]) {
    let reference = sample(reference);
    for (para, _, script) in CASES {
        let (_, got) = equation_in(doc, para);
        let (_, want) = equation_in(&reference, para);
        assert_eq!(got.script, script);
        assert_eq!(want.script, script, "재조판본의 문단 {para} 수식");
        assert_eq!(got.font_name, want.font_name, "{script}");
        let (got, want) = (frame(&got), frame(&want));
        if width_unresolved.contains(&script) {
            assert_eq!((got.1, got.2), (want.1, want.2), "{script} 높이·기준선");
        } else {
            assert_eq!(got, want, "{script} 폭·높이·기준선");
        }
    }
}

#[test]
fn inserted_hyhwpeq_frames_match_hancom_relayout() {
    assert_matches_reference(&inserted(None), "rhwp-HYhwpEQ.hancom-relayout.hwp", &[]);
}

#[test]
fn hancomeqn_font_change_frames_match_hancom_relayout() {
    assert_matches_reference(
        &inserted(Some("HancomEQN")),
        "rhwp-HancomEQN.hancom-relayout.hwp",
        &["a ^{2} + b ^{2} = c ^{2}", "prod _{i} x"],
    );
}

#[test]
fn inserted_frames_survive_hwp_round_trip() {
    let doc = inserted(None);
    let reopened = DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap();
    for (para, _, script) in CASES {
        assert_eq!(
            frame(&equation_in(&reopened, para).1),
            frame(&equation_in(&doc, para).1),
            "{script}"
        );
    }
}

fn flow_end(paragraph: &rhwp::model::paragraph::Paragraph) -> i32 {
    let line = paragraph.line_segs.last().expect("저장 줄");
    line.vertical_pos + line.line_height + line.line_spacing
}

/// 수식으로 높아진 줄만큼 뒤 문단의 저장 세로 위치가 밀리고, 문단 사이 간격은 기준 문서의
/// 저장 간격 그대로다. 한글 2020 재조판본도 같은 관계다(문단 0 끝 1763 → 문단 1 시작 2263).
#[test]
fn insertion_rechains_following_paragraph_positions() {
    let base = sample("base-a4.hwp");
    let mut doc = sample("base-a4.hwp");
    for (para, offset, script) in CASES {
        doc.insert_equation_native(0, para, offset, script, 1000, 0)
            .unwrap();
    }
    let before = &base.document().sections[0].paragraphs;
    let after = &doc.document().sections[0].paragraphs;
    assert_eq!(before.len(), after.len());
    for index in 1..after.len() {
        let gap = before[index].line_segs[0].vertical_pos - flow_end(&before[index - 1]);
        assert_eq!(
            after[index].line_segs[0].vertical_pos - flow_end(&after[index - 1]),
            gap,
            "문단 {index}의 시작은 앞 문단 끝에서 기준 문서와 같은 간격만큼 떨어진다"
        );
    }
    let reference = sample("rhwp-HYhwpEQ.hancom-relayout.hwp");
    let reference = &reference.document().sections[0].paragraphs;
    for index in 0..after.len() {
        assert_eq!(
            after[index].line_segs[0].vertical_pos, reference[index].line_segs[0].vertical_pos,
            "문단 {index}의 저장 세로 위치 = 한글 재조판본"
        );
    }
}

fn blank_with_equation(script: &str, font_size: u32) -> (DocumentCore, usize) {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "끝").unwrap();
    doc.insert_equation_native(0, 0, 0, script, font_size, 0)
        .unwrap();
    let (index, _) = equation_in(&doc, 0);
    (doc, index)
}

/// 한컴 글줄 프레임이 측정되지 않은 글자 크기는 측정 근거가 없으므로 기존 추정 크기와
/// 기존 기준선을 그대로 쓴다.
#[test]
fn unmeasured_font_size_keeps_estimated_frame() {
    for script in ["a times b", "1 over {2 over 3}"] {
        let (doc, _) = blank_with_equation(script, 1050);
        let (width, height) = rhwp::renderer::equation::intrinsic_size_hwp(script, 1050);
        assert_eq!(
            frame(&equation_in(&doc, 0).1),
            (width, height, 85),
            "{script}"
        );
    }
}

#[test]
fn empty_script_keeps_estimated_frame() {
    let (doc, _) = blank_with_equation("", 1000);
    let (width, height) = rhwp::renderer::equation::intrinsic_size_hwp("", 1000);
    assert_eq!(frame(&equation_in(&doc, 0).1), (width, height, 85));
}

/// 크기를 정하지 않는 속성만 바꾸면 프레임을 다시 만들지 않고, 봉지가 지정한 축은 그대로 쓴다.
#[test]
fn property_bag_owns_explicit_axes() {
    let (mut doc, index) = blank_with_equation("a times b", 1000);
    let before = frame(&equation_in(&doc, 0).1);
    doc.set_equation_properties_native(0, 0, index, None, None, r#"{"color":255}"#)
        .unwrap();
    assert_eq!(frame(&equation_in(&doc, 0).1), before, "색만 바꿈");

    doc.set_equation_properties_native(
        0,
        0,
        index,
        None,
        None,
        r#"{"script":"1 over {2 over 3}","width":5000,"height":4000,"baseline":70}"#,
    )
    .unwrap();
    assert_eq!(frame(&equation_in(&doc, 0).1), (5000, 4000, 70));
}

fn equation_layouts<'a>(node: &'a RenderNode, out: &mut Vec<(&'a str, &'a LayoutBox)>) {
    if let RenderNodeType::Equation(equation) = &node.node_type {
        out.push((equation.script.as_str(), &equation.layout_box));
    }
    for child in &node.children {
        equation_layouts(child, out);
    }
}

fn big_operator(layout: &LayoutBox) -> Option<&LayoutBox> {
    match &layout.kind {
        LayoutKind::BigOp { .. } => Some(layout),
        LayoutKind::Row(children) => children.iter().find_map(big_operator),
        _ => None,
    }
}

/// 큰 합·곱의 위·아래 한계값은 연산자 글리프의 잉크와 겹치지 않는다. 표시 글리프인
/// Latin Modern Math의 ∑·∏(`assets/fonts/LatinModernMath-Regular.woff2`)는 잉크가 둘 다
/// -0.25..0.75em이다. 한글 2022 4645 Print에서도 곱의 잉크(352.0~368.7px)와 아래 한계값
/// `i`(370.9px부터)는 떨어져 있다. 한컴 합·곱 프레임의 한계값 칸과 원본 face 연산자
/// 글리프가 다른 조건을 쓰면 아래 한계값이 곱의 다리에 걸린다.
#[test]
fn big_operator_limits_clear_the_operator_ink() {
    const SCRIPTS: [&str; 4] = [
        "prod _{i} x",
        "prod _{i=1} ^{n} x",
        "sum _{i} x",
        "sum _{i=1} ^{n} x",
    ];
    for font in [None, Some("HancomEQN")] {
        let mut doc = DocumentCore::new_empty();
        doc.create_blank_document_native().unwrap();
        doc.insert_text_native(0, 0, 0, "끝").unwrap();
        for script in SCRIPTS {
            doc.insert_equation_native(0, 0, 0, script, 1000, 0)
                .unwrap();
        }
        if let Some(font) = font {
            let indices: Vec<usize> = doc.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .enumerate()
                .filter_map(|(index, control)| {
                    matches!(control, Control::Equation(_)).then_some(index)
                })
                .collect();
            for index in indices {
                doc.set_equation_properties_native(
                    0,
                    0,
                    index,
                    None,
                    None,
                    &format!(r#"{{"fontName":"{font}"}}"#),
                )
                .unwrap();
            }
        }
        let page = doc.build_page_render_tree(0).unwrap();
        let mut layouts = Vec::new();
        equation_layouts(&page.root, &mut layouts);
        assert_eq!(
            layouts
                .iter()
                .map(|(script, _)| *script)
                .collect::<Vec<_>>(),
            SCRIPTS,
            "{font:?} 수식 순서·누락·중복"
        );
        for (script, layout) in layouts {
            let Some(op) = big_operator(layout) else {
                // HancomEQN의 두 한계값 합은 한컴 PUA 글리프와 실측 오프셋으로 따로 배치한다.
                assert_eq!((font, script), (Some("HancomEQN"), "sum _{i=1} ^{n} x"));
                continue;
            };
            let LayoutKind::BigOp {
                op_font_size,
                op_baseline,
                sub,
                sup,
                ..
            } = &op.kind
            else {
                unreachable!()
            };
            let ink_top = op_baseline - 0.75 * op_font_size;
            let ink_bottom = op_baseline + 0.25 * op_font_size;
            if let Some(sup) = sup {
                assert!(
                    sup.y + sup.height <= ink_top + 1e-6,
                    "{font:?} {script}: 위 한계값 끝 {} > 연산자 잉크 위 {ink_top}",
                    sup.y + sup.height
                );
            }
            let sub = sub.as_ref().expect("아래 한계값");
            assert!(
                sub.y >= ink_bottom - 1e-6,
                "{font:?} {script}: 아래 한계값 시작 {} < 연산자 잉크 아래 {ink_bottom}",
                sub.y
            );
        }
    }
}
