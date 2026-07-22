//! 머리말/꼬리말 그림 treatAsChar 토글 시 앵커 문단 line_segs 미갱신 회귀 가드.
//!
//! 본문 그림(set_picture_properties_native)은 [Task #1151 v2] 마이그레이션으로
//! treatAsChar false→true 시 앵커 line_segs[0].line_height 를 그림 높이로 갱신한다.
//! 머리말/꼬리말 경로(set_header_footer_picture_properties_native)는 이 마이그레이션이
//! 누락되어 있었다.

use rhwp::model::control::Control;
use std::fs;
use std::path::Path;

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
fn header_picture_tac_on_updates_anchor_line_height() {
    use rhwp::document_core::DocumentCore;

    let repo_root = env!("CARGO_MANIFEST_DIR");
    let p = Path::new(repo_root).join("samples/hwp3-sample11.hwp");
    let data = fs::read(&p).expect("read");
    let mut core = DocumentCore::from_bytes(&data).expect("parse");

    let (si, bi, hi, ipi, ici) =
        find_header_picture("samples/hwp3-sample11.hwp").expect("header picture must exist");

    // 샘플의 초기 tac 상태와 무관하게 false→true 전환을 강제로 재현한다:
    // 먼저 false 로 만들어 floating 상태로 정규화한 뒤, true 로 토글해 마이그레이션을
    // 실제로 발동시킨다 (샘플 그림이 이미 tac=true 이면 두 번째 호출이 no-op 이 되어
    // 마이그레이션 미실행을 놓치는 것을 방지).
    core.set_header_footer_picture_properties_native(
        si,
        bi,
        hi,
        ipi,
        ici,
        r#"{"treatAsChar":false}"#,
    )
    .expect("set_header_footer_picture_properties_native(treatAsChar:false)");

    let pic_height = match &core.document().sections[si].paragraphs[bi].controls[hi] {
        Control::Header(h) => match &h.paragraphs[ipi].controls[ici] {
            Control::Picture(p) => p.common.height as i32,
            _ => panic!("expected picture"),
        },
        _ => panic!("expected header"),
    };

    core.set_header_footer_picture_properties_native(
        si,
        bi,
        hi,
        ipi,
        ici,
        r#"{"treatAsChar":true}"#,
    )
    .expect("set_header_footer_picture_properties_native(treatAsChar:true)");

    let line_height = match &core.document().sections[si].paragraphs[bi].controls[hi] {
        Control::Header(h) => h.paragraphs[ipi].line_segs.first().map(|s| s.line_height),
        _ => panic!("expected header"),
    };

    assert_eq!(
        line_height,
        Some(pic_height),
        "머리말 그림 treatAsChar on 후 앵커 문단 line_segs[0].line_height 가 그림 높이({pic_height})로 갱신되어야 함"
    );
}
