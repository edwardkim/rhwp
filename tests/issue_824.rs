//! Issue #824: HWP3 임베디드 그림이 외부 파일 참조로 잘못 표시됨
//!
//! 본질: src/parser/hwp3/mod.rs 의 `pic_type` 분기 누락 — pic_type 0/1/2 를
//! 동일 처리하여 임베디드 그림(pic_type == 2)에도 `external_path` 가 설정됨.
//! 한컴오피스 2022 는 임베디드 그림에 대해 "문서에 포함" 으로 표시하지만,
//! rhwp-studio 그림 속성 dialog 는 `E$$00000.jpg` 같은 내부 참조명을 외부 파일로
//! 잘못 표시한다.
//!
//! 정정: pic_type == 0 (외부 파일) 만 external_path 설정.

use rhwp::model::control::Control;
use std::fs;
use std::path::Path;

fn first_picture_external_path(hwp: &str) -> Option<String> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(hwp);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", hwp, e));
    let doc =
        rhwp::parser::hwp3::parse_hwp3(&data).unwrap_or_else(|e| panic!("parse {}: {:?}", hwp, e));
    for sec in &doc.sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                if let Control::Picture(pic) = ctrl {
                    return pic.image_attr.external_path.clone();
                }
            }
        }
    }
    None
}

#[test]
fn issue_824_embedded_picture_no_external_path() {
    let path = first_picture_external_path("samples/hwp3-sample11.hwp");
    assert!(
        path.is_none(),
        "임베디드 그림(pic_type=2)은 external_path 가 None 이어야 함 \
         (현행 결함: Some(\"E$$00000.jpg\")). got: {:?}",
        path,
    );
}

#[test]
fn issue_824_external_picture_keeps_external_path() {
    // 회귀 가드: 외부 file path 그림(pic_type=0)은 본 정정 후에도 external_path 유지.
    let path = first_picture_external_path("samples/hwp3-sample10.hwp");
    assert!(
        path.is_some(),
        "외부 file path 그림(pic_type=0)은 external_path 가 Some 이어야 함",
    );
}
