//! 검사 유형: 관계형·고정형(독립 Print의 80pt 탭 간격); 구역·편집·재열기 대조 포함.
//! 주석 갱신: 2026-11-05T02:21:40+09:00; Git 직전 파일 변경: 2587dd58dac9a8bf312d7cd2f59b37e803d1a387 2026-10-29T07:20:55+09:00.
//! 선행 측정 source: 5c540c6c3e. 96 DPI·Print·PDFium exact physical DPI v2·2px raw 실루엣.
//! 측정 실행 UTC: Native 2026-11-04T16:52:30.180764+00:00 ~ 2026-11-04T16:52:34.122717+00:00; WASM 2026-11-04T17:02:17.994972+00:00 ~ 2026-11-04T17:02:25.995881+00:00.
//! section_default_tab_spacing: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! 증거: mydocs/pr/assets/semanticist21-20261105/validation/candidate/{native,wasm}/section_default_tab_spacing.tsv.
//! 실물 1쪽 N/W 100.00/100.00%; 다중 구역 변형은 독립 1구역 대조군과 비교하는 API 계약이며 별도 Print 일치 주장 없음.
//!
//! 기본 탭 자리를 구역 정의의 기본 탭 간격으로 정한다.
//!
//! 시험 문서는 구역 정의의 기본 탭 간격이 16000(한/글 대화상자 80pt)이고, 둘째 문단이
//! `가\t나\t다\t라\t마`다. 한컴 한글 12.30(Mac) 인쇄 PDF(`pdf/section_default_tab_spacing/`)에서
//! 글자 왼끝은 85.08, 165.12, 245.04, 325.08, 405.12pt로 80pt(106.67px)씩 떨어진다.
//! 같은 문서를 기본값 8000으로 저장하면 한/글은 40pt(53.33px)씩 둔다. 구역 값을 읽지 않으면
//! 두 문서 모두 40pt 간격이 된다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "tests/fixtures/section_default_tab_spacing/tab-spacing-16000.hwp";
const PARA: usize = 1;

fn caret_x(core: &DocumentCore, offset: usize) -> f64 {
    let json = core
        .get_cursor_rect_native(0, PARA, offset)
        .expect("cursor rect");
    let start = json.find("\"x\":").expect("x 필드") + "\"x\":".len();
    json[start..]
        .split([',', '}'])
        .next()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or_else(|| panic!("x 파싱 실패: {json}"))
}

/// 탭 뒤 글자(나·다·라·마)의 왼끝이 첫 글자에서 `step` px 배수만큼 떨어지는지 본다.
fn assert_tab_step(core: &DocumentCore, step: f64) {
    let origin = caret_x(core, 0);
    for k in 1..=4 {
        let x = caret_x(core, 2 * k) - origin;
        let expected = step * k as f64;
        assert!(
            (x - expected).abs() < 1.0,
            "{k}번째 탭 뒤 글자: {x:.2}px, 기대 {expected:.2}px"
        );
    }
}

#[test]
fn default_tab_stops_follow_section_tab_spacing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_tab_step(&core, 80.0 * 96.0 / 72.0);

    core.set_section_def_native(0, r#"{"defaultTabSpacing":8000}"#)
        .expect("set section def");
    assert_tab_step(&core, 40.0 * 96.0 / 72.0);
}

#[test]
fn section_tabs_match_independent_controls_after_edits_and_reopen() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let mut wide = DocumentCore::from_bytes(&bytes).unwrap();
    let mut narrow = DocumentCore::from_bytes(&bytes).unwrap();
    narrow
        .set_section_def_native(0, r#"{"defaultTabSpacing":8000}"#)
        .unwrap();
    let mut multi = DocumentCore::from_bytes(&bytes).unwrap();
    let mut document = multi.document().clone();
    document.sections.push(document.sections[0].clone());
    multi.set_document(document);
    multi
        .set_section_def_native(1, r#"{"defaultTabSpacing":8000}"#)
        .unwrap();

    let gap = |core: &DocumentCore, section, offset| {
        let coordinate = |offset| {
            let value: serde_json::Value =
                serde_json::from_str(&core.get_cursor_rect_native(section, PARA, offset).unwrap())
                    .unwrap();
            value["x"].as_f64().unwrap()
        };
        coordinate(offset) - coordinate(0)
    };
    // 독립 1구역 대조군과 비교한다. 원본 16000:8000 비율도 보존해야 하므로
    // 두 구역 모두 기존의 같은 기본값을 쓰는 구현은 통과하지 못한다.
    assert!((gap(&wide, 0, 2) - 2.0 * gap(&narrow, 0, 2)).abs() < 0.2);
    for order in [[0, 1], [1, 0], [0, 1]] {
        for section in order {
            let expected = gap(if section == 0 { &wide } else { &narrow }, 0, 2);
            assert!((gap(&multi, section, 2) - expected).abs() < 0.11);
        }
    }
    multi.insert_text_native(1, PARA, 0, "A").unwrap();
    narrow.insert_text_native(0, PARA, 0, "A").unwrap();
    assert!((gap(&multi, 1, 3) - gap(&narrow, 0, 3)).abs() < 0.11);
    multi
        .set_section_def_native(0, r#"{"defaultTabSpacing":24000}"#)
        .unwrap();
    wide.set_section_def_native(0, r#"{"defaultTabSpacing":24000}"#)
        .unwrap();
    assert!((gap(&multi, 0, 2) - gap(&wide, 0, 2)).abs() < 0.11);
    assert!((gap(&multi, 1, 3) - gap(&narrow, 0, 3)).abs() < 0.11);

    let saved = rhwp::serializer::hwpx::serialize_hwpx(multi.document()).unwrap();
    let reopened = DocumentCore::from_bytes(&saved).unwrap();
    assert!((gap(&reopened, 0, 2) - gap(&wide, 0, 2)).abs() < 0.11);
    assert!((gap(&reopened, 1, 3) - gap(&narrow, 0, 3)).abs() < 0.11);
}
