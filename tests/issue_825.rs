//! Issue #825: rhwp-studio 머리말 영역 그림 우클릭 → "개체 속성" dialog 미표시.
//!
//! 본질: WASM `get_picture_properties_native(sec, ppi, ci)` 가 본문 paragraph
//! `section.paragraphs[ppi].controls[ci]` 만 lookup. 머리말/꼬리말 그림은
//! `Control::Header(h).paragraphs[hdr_para].controls[pic_ctrl]` 안에 위치하므로
//! 본 API 로 도달 불가.
//!
//! 정정 (Stage 3): WASM API 에 optional header/footer path 파라미터 추가
//! → header_para_idx + footer_para_idx 등을 받아 lookup 분기.

use rhwp::model::control::Control;
use std::fs;
use std::path::Path;

/// 머리말 picture 위치를 IR 에서 찾아 path 5-tuple 반환.
fn find_header_picture(hwp: &str) -> Option<(usize, usize, usize, usize, usize)> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(hwp);
    let data = fs::read(&path).ok()?;
    let doc = rhwp::parser::hwp3::parse_hwp3(&data).ok()?;
    for (si, sec) in doc.sections.iter().enumerate() {
        for (bi, para) in sec.paragraphs.iter().enumerate() {
            for (hi, ctrl) in para.controls.iter().enumerate() {
                if let Control::Header(h) = ctrl {
                    for (ipi, ipara) in h.paragraphs.iter().enumerate() {
                        for (ici, ictrl) in ipara.controls.iter().enumerate() {
                            if matches!(ictrl, Control::Picture(_)) {
                                return Some((si, bi, hi, ipi, ici));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[test]
fn issue_825_sample11_has_header_picture() {
    // 사전 조건: sample11.hwp 머리말에 그림이 존재함을 IR 로 확인.
    let path = find_header_picture("samples/hwp3-sample11.hwp");
    assert!(
        path.is_some(),
        "sample11.hwp 머리말에 그림이 있어야 함 (테스트 fixture 정합성 검증)",
    );
}

#[test]
fn issue_825_header_picture_lookup_via_existing_api_fails() {
    // 현행 get_picture_properties_native 는 본문 lookup 만 지원 →
    // 머리말 그림 path 로는 결과 부정확/실패. 회귀 가드 (직접 호출 시 여전히 error).
    use rhwp::wasm_api::HwpDocument;

    let repo_root = env!("CARGO_MANIFEST_DIR");
    let p = Path::new(repo_root).join("samples/hwp3-sample11.hwp");
    let data = fs::read(&p).expect("read");
    let doc = HwpDocument::from_bytes(&data).expect("parse");

    let (si, bi, hi, _ipi, _ici) =
        find_header_picture("samples/hwp3-sample11.hwp").expect("header picture must exist");

    // 현행 API: (sec, body_para_idx, header_ctrl_idx) 호출 → 실패 (Header 컨트롤이지 Picture 아님)
    let result = doc.get_picture_properties_native(si, bi, hi);
    assert!(
        result.is_err(),
        "현행 API 는 머리말 그림에 도달할 수 없어야 함 (Header 컨트롤 자체가 반환되어 Picture 변환 실패). got: Ok",
    );
}

#[test]
fn issue_825_header_picture_lookup_via_new_api_succeeds() {
    // GREEN: 신규 get_header_footer_picture_properties_native 로 머리말 그림 도달.
    use rhwp::wasm_api::HwpDocument;

    let repo_root = env!("CARGO_MANIFEST_DIR");
    let p = Path::new(repo_root).join("samples/hwp3-sample11.hwp");
    let data = fs::read(&p).expect("read");
    let doc = HwpDocument::from_bytes(&data).expect("parse");

    let (si, bi, hi, ipi, ici) =
        find_header_picture("samples/hwp3-sample11.hwp").expect("header picture must exist");

    // 신규 API: (sec, outer_body_para, outer_header_ctrl, inner_para, inner_ctrl)
    let result = doc.get_header_footer_picture_properties_native(si, bi, hi, ipi, ici);
    assert!(
        result.is_ok(),
        "신규 API 는 머리말 그림 속성을 정상 반환해야 함. got: {:?}",
        result.as_ref().err(),
    );
    let json = result.unwrap();
    assert!(json.contains("\"width\":"), "JSON 에 width 필드 포함");
    assert!(json.contains("\"height\":"), "JSON 에 height 필드 포함");
}
