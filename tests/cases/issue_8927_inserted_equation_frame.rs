//! 검사 유형: 고정형(한글 2020 재조판 저장본에서 읽은 수식 프레임 값·문단 저장 세로 위치와의 일치)
//! + 관계형(비적용 경로는 기존 추정 크기와 같음, 문단 사이 간격은 기준 문서와 같음).
//! 주석 갱신: 2026-11-27T09:45:53+09:00, 새 파일.
//! 측정: 2026-11-27, source fd42b4ed42(대조 devel 3fdaa04258). 96 DPI·Print·2px raw 실루엣
//! (threshold_boundary_color_support_v1, 색상 허용 32, 경계 보정 0px). 입력은 `base-a4.hwp`에 CLI로
//! 수식 7개를 넣은 `a4-inserted-*.hwp`, 기준은 한글 2022 12.0.0.4645 Print(A4, 본문 KoPubWorld바탕체
//! Light 설치) `pdf/issue-8927/a4-inserted-*-2022.pdf`. 전1쪽, 쪽수 1=1, 90% 미만 없음:
//! HYhwpEQ Native 99.57% / fresh WASM 98.02%, HancomEQN Native 99.53% / fresh WASM 97.82%
//! (devel 렌더러 98.97/97.65%, 98.94/97.47%). fresh WASM Canvas2D는 HYhwpEQ 사설 영역 글리프를
//! □로 그려(devel 같음) WASM의 수식 글자 모양은 미검증이다. 이 파일은 화소를 비교하지 않는다.
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
//! - `prod _{i} x`의 폭: ∏ 글리프의 한컴 전진폭 표가 없어 10pt에서 HYhwpEQ 66HU·HancomEQN 141HU
//!   넓다. 높이·기준선만 검사한다.
//! - HancomEQN `a ^{2} + b ^{2} = c ^{2}`의 폭: 위첨자 숫자의 한컴 전진폭이 같은 논리 글자 크기에서도
//!   글자 크기마다 달라(10pt 5px, 10.5~11.5pt 6px) 225HU 넓다. 높이·기준선만 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Equation};

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
    assert_matches_reference(
        &inserted(None),
        "rhwp-HYhwpEQ.hancom-relayout.hwp",
        &["prod _{i} x"],
    );
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
