//! [Issue #8306] `export-hwpx --verify` 는 HWPX 저장기의 의도된 정규화를 손실로 세지 않는다.
//!
//! 저장기는 같은 문단의 두 번째 이후 쪽 번호 위치(`pgnp`)를 내지 않고 그 자리만큼 뒤 위치를
//! 당기며(#6869), 줄이 모두 reflow 합성(bit 31)인 문단의 `linesegarray` 를 내지 않는다
//! (#5847). 종전 `--verify` 는 원본 IR 과 재파싱 IR 을 그대로 비교해 이 둘을 손실로 보고했다.
//! 기대 IR(`hwpx_export_expected_document`)은 같은 규칙을 먼저 적용하고, 그 밖의 차이는
//! 그대로 손실로 남긴다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::serializer::hwpx::roundtrip::{
    diff_documents, hwpx_export_expected_document, strip_hwp_to_hwpx_noise,
};

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 파싱 실패: {e}"))
}

fn reopen_as_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기")
}

fn reopen_as_hwp(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_with_adapter().expect("HWP 저장"))
        .expect("HWP 다시 열기")
}

/// `--verify` 와 같은 비교: 양쪽에 기대 IR 정규화를 적용한 뒤의 차이 목록. HWP5 원본은
/// CLI 와 같이 HWPX 와 표현 자리가 다른 필드 메타데이터(#3739)를 걸러 낸다.
fn verify_differences(source: &Document, reloaded: &Document, hwp5_source: bool) -> Vec<String> {
    let diff = diff_documents(
        &hwpx_export_expected_document(source),
        &hwpx_export_expected_document(reloaded),
    );
    let diff = if hwp5_source {
        strip_hwp_to_hwpx_noise(diff)
    } else {
        diff
    };
    diff.differences.iter().map(|d| d.to_string()).collect()
}

fn raw_differences(source: &Document, reloaded: &Document) -> Vec<String> {
    diff_documents(source, reloaded)
        .differences
        .iter()
        .map(|d| d.to_string())
        .collect()
}

fn page_number_pos_count(para: &Paragraph) -> usize {
    para.controls
        .iter()
        .filter(|c| matches!(c, Control::PageNumberPos(_)))
        .count()
}

/// 쪽 번호 위치 접기: 한 문단에 여러 개인 문서(본문·표 셀·뒤따르는 표·한컴 HWPX 출처)에서
/// 정규화 전에는 차이가 나고, 정규화 뒤에는 차이가 없다.
#[test]
fn collapsed_page_number_positions_are_not_reported_as_loss() {
    for path in [
        // 한 문단 7개, 글자 모양 경계가 접힌 자리 안팎에 있다.
        "samples/issue7015/30442-acrc-recommendation-business-burden.hwp",
        // 접힌 쪽 번호 위치 뒤에 표 — 컨트롤 인덱스 대응이 접기로 어긋난다.
        "samples/issue5699/37787_regulatory_impact.hwp",
        // 표 셀 안 문단.
        "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
        // 한컴 HWPX 출처.
        "samples/issue6465/press_release_footer_logos.hwpx",
    ] {
        let source = load(path);
        assert!(
            source
                .document()
                .sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .any(|p| page_number_pos_count(p) > 1)
                || path.contains("issue2007"),
            "{path}: 한 문단에 쪽 번호 위치가 둘 이상이어야 한다"
        );
        let reloaded = reopen_as_hwpx(&source);
        assert!(
            !raw_differences(source.document(), reloaded.document()).is_empty(),
            "{path}: 정규화 전에는 접기가 차이로 보인다"
        );
        assert_eq!(
            verify_differences(
                source.document(),
                reloaded.document(),
                path.ends_with(".hwp")
            ),
            Vec::<String>::new(),
            "{path}"
        );
    }
}

/// 기대 IR 은 저장기와 같은 좌표로 자리를 걷어낸다 — 접힌 자리 안팎의 글자 모양 경계가
/// 재파싱 결과와 같다(같은 자리로 모인 경계도 합치지 않는다).
#[test]
fn expected_ir_pulls_char_shapes_like_the_serializer() {
    let source = load("samples/issue7015/30442-acrc-recommendation-business-burden.hwp");
    let expected = hwpx_export_expected_document(source.document());
    let reloaded = reopen_as_hwpx(&source);
    let shapes = |doc: &Document| -> Vec<(u32, u32)> {
        doc.sections[0].paragraphs[0]
            .char_shapes
            .iter()
            .map(|c| (c.start_pos, c.char_shape_id))
            .collect()
    };
    assert_eq!(
        shapes(source.document()),
        [(0, 141), (24, 201), (40, 240), (64, 52)]
    );
    assert_eq!(
        shapes(&expected),
        [(0, 141), (24, 201), (24, 240), (24, 52)]
    );
    assert_eq!(shapes(&expected), shapes(reloaded.document()));
    assert_eq!(
        page_number_pos_count(&expected.sections[0].paragraphs[0]),
        1
    );
}

/// 합성 줄: 저장 줄 없는 HWPX 를 HWP 로 저장하면 합성 줄이 HWP 에 실리고, 그 HWP 를
/// HWPX 로 다시 저장하면 합성 줄은 쓰지 않는다.
#[test]
fn synthetic_line_segs_are_not_reported_as_loss() {
    for path in [
        "samples/issue7418/trailing_space_synthetic.hwpx",
        "samples/issue7816/odd_spacing_prev.hwpx",
    ] {
        let hwp = reopen_as_hwp(&load(path));
        let reloaded = reopen_as_hwpx(&hwp);
        assert!(
            !raw_differences(hwp.document(), reloaded.document()).is_empty(),
            "{path}: 정규화 전에는 합성 줄 생략이 차이로 보인다"
        );
        assert_eq!(
            verify_differences(hwp.document(), reloaded.document(), true),
            Vec::<String>::new(),
            "{path}"
        );
    }
}

/// 반례: 정규화가 설명하지 못하는 차이는 그대로 남는다 — 남긴 쪽 번호 위치를 잃거나
/// 저장 줄 일부만 합성인 문단의 줄을 잃으면 손실이다.
#[test]
fn losses_outside_the_normalization_are_still_reported() {
    let source = load("samples/issue1921/59043_regulatory_analysis.hwp");
    let mut reloaded = reopen_as_hwpx(&source).document().clone();
    let (si, pi) = reloaded
        .sections
        .iter()
        .enumerate()
        .find_map(|(si, s)| {
            s.paragraphs
                .iter()
                .position(|p| page_number_pos_count(p) == 1)
                .map(|pi| (si, pi))
        })
        .expect("쪽 번호 위치가 있는 문단");
    let para = &mut reloaded.sections[si].paragraphs[pi];
    let index = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::PageNumberPos(_)))
        .unwrap();
    para.controls.remove(index);
    assert!(
        verify_differences(source.document(), &reloaded, true)
            .iter()
            .any(|d| d.contains("controls")),
        "남긴 쪽 번호 위치의 손실"
    );

    let source = load("samples/issue1921/59043_regulatory_analysis.hwp");
    let mut reloaded = reopen_as_hwpx(&source).document().clone();
    let para = reloaded.sections[0]
        .paragraphs
        .iter_mut()
        .find(|p| p.line_segs.len() > 1)
        .expect("줄이 여럿인 문단");
    para.line_segs[0].tag |= 0x8000_0000;
    para.line_segs.truncate(1);
    assert!(
        verify_differences(source.document(), &reloaded, true)
            .iter()
            .any(|d| d.contains("linesegs")),
        "일부만 합성인 문단의 줄 손실"
    );
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn export_hwpx_verify_exit(input: &std::path::Path, name: &str) -> Option<i32> {
    let out = std::env::temp_dir().join(format!("rhwp_8306_{}_{name}.hwpx", std::process::id()));
    let status = std::process::Command::new(rhwp_bin())
        .args(["export-hwpx"])
        .arg(input)
        .arg(&out)
        .arg("--verify")
        .output()
        .expect("rhwp 실행");
    let _ = std::fs::remove_file(&out);
    status.status.code()
}

/// CLI 계약: 이슈의 재현 명령이 exit 0 으로 끝난다(종전 exit 3). 실제 손실은 위 반례처럼
/// 계속 exit 3 으로 잡힌다.
#[test]
fn export_hwpx_verify_passes_intended_normalization() {
    for path in [
        "samples/issue1921/59043_regulatory_analysis.hwp",
        "samples/issue6465/press_release_footer_logos.hwpx",
    ] {
        assert_eq!(
            export_hwpx_verify_exit(std::path::Path::new(path), "direct"),
            Some(0),
            "{path}"
        );
    }
    // 저장 줄 없는 HWPX 를 HWP 로 저장한 파일.
    let hwp = std::env::temp_dir().join(format!("rhwp_8306_{}_synthetic.hwp", std::process::id()));
    std::fs::write(
        &hwp,
        load("samples/issue7418/trailing_space_synthetic.hwpx")
            .export_hwp_with_adapter()
            .expect("HWP 저장"),
    )
    .expect("HWP 쓰기");
    let code = export_hwpx_verify_exit(&hwp, "synthetic");
    let _ = std::fs::remove_file(&hwp);
    assert_eq!(code, Some(0), "합성 줄 HWP → HWPX");
}
