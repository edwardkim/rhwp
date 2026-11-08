//! 동일 face의 부분 cmap가 실제 표시 글자를 대체 글꼴로 넘기지 않게 검사한다.
#![cfg(not(target_arch = "wasm32"))]

#[path = "../../src/renderer/svg/system_fonts.rs"]
#[allow(dead_code)]
mod system_fonts;

use std::path::PathBuf;
use system_fonts::family_file_from_database;
use usvg::fontdb::Database;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fonts")
        .join(name)
}

#[test]
fn same_family_complete_face_supplies_the_missing_owned_character() {
    let partial = fixture("RHWPHostFixture-CoverageSubset.ttf");
    let complete = fixture("RHWPHostFixture-Regular.ttf");
    let mut fonts = Database::new();
    fonts.load_font_file(&partial).unwrap();
    fonts.load_font_file(&complete).unwrap();
    let family = ["RHWP Host Fixture".to_owned()];
    let empty = std::collections::HashSet::new();
    assert_eq!(
        family_file_from_database(&fonts, &family, false, &empty),
        Some(partial.clone()),
        "수정 전 family 선택은 부분 파일을 먼저 반환한다"
    );
    assert_eq!(
        family_file_from_database(&fonts, &family, false, &['A', '가'].into()),
        Some(complete),
        "원문 두 문자의 같은 face 공급"
    );
    assert_eq!(
        family_file_from_database(&fonts, &family, false, &['가'].into()),
        Some(partial),
        "이미 공급되는 문자의 기존 우선순위 보존"
    );
}

#[test]
fn missing_character_does_not_select_an_italic_face_or_fake_bold() {
    let mut fonts = Database::new();
    fonts
        .load_font_file(fixture("RHWPHostFixture-CoverageSubset.ttf"))
        .unwrap();
    fonts
        .load_font_file(fixture("RHWPHostFixture-Italic.ttf"))
        .unwrap();
    let family = ["RHWP Host Fixture".to_owned()];
    let chars = ['A'].into();
    assert_eq!(
        family_file_from_database(&fonts, &family, false, &chars),
        None
    );
    assert_eq!(
        family_file_from_database(&fonts, &family, true, &chars),
        None
    );
    assert_eq!(
        family_file_from_database(&fonts, &["없는 face".to_owned()], false, &chars),
        None
    );
}
