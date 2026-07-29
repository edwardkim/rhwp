//! HWP3 PageNumberPos는 본문 marker를 만들지 않는다는 실제 fixture 회귀.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::model::control::Control;
use rhwp::parse_document_with_password;

const FIXTURE: &str = "samples/HWP3-password-123456.hwp";
const FIXTURE_PASSWORD: &[u8] = &[49, 50, 51, 52, 53, 54];

fn fixture_bytes() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(Path::new(FIXTURE));
    std::fs::read(path).expect("암호 HWP3 fixture를 읽어야 함")
}

#[test]
fn page_number_position_does_not_prefix_hwp3_title_with_object_marker() {
    let document = parse_document_with_password(&fixture_bytes(), FIXTURE_PASSWORD)
        .expect("실제 암호 HWP3 fixture를 열어야 함");
    let title = &document.sections[0].paragraphs[1];

    assert!(
        title
            .controls
            .iter()
            .any(|control| matches!(control, Control::PageNumberPos(_))),
        "PageNumberPos 설정 control은 보존해야 함"
    );
    assert!(
        title.text.starts_with("ᄒᆞᆫ글 97 안내문"),
        "PageNumberPos는 title 앞에 U+FFFC를 남기면 안 됨: {:?}",
        title.text
    );
    assert!(
        !title.text.starts_with('\u{FFFC}'),
        "title 첫 가시 문자는 object marker가 아니라 ᄒ여야 함"
    );
    assert_eq!(
        title.line_segs[0].text_start, 0,
        "PageNumberPos는 첫 줄의 text start를 이동시키면 안 됨"
    );
}
