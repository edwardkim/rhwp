//! Issue #1385: replaceAll/replaceText 변경이 exportHwp 직렬화에 반영되지 않음
//!
//! HWP5 로드 시 `section.raw_stream` 캐시가 보존되는데, `replace_all_native` 는
//! (`insert_text_native`/`delete_text_native` 와 달리) 문단을 직접 조작하면서
//! 캐시를 무효화하지 않아 `export_hwp` 가 원본 바이트를 그대로 반환했다.
//! 메모리 모델(getTextRange)에는 반영되지만 저장 시 유실되는 증상.
//!
//! 본 테스트는 parse → replaceAll → export → reparse 라운드트립으로
//! 본문 문단 경로와 표 셀 경로 양쪽의 치환 보존을 단언한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::ShapeObject;

/// 본문 + 표 셀 + 글상자 문단 텍스트를 전부 이어붙인다 (치환 보존 검증용).
/// `replace_all_native` 의 검색 범위(search_all)와 동일하게 순회한다.
fn collect_all_text(doc: &Document) -> String {
    let mut acc = String::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            acc.push_str(&para.text);
            acc.push('\n');
            for ctrl in &para.controls {
                match ctrl {
                    Control::Table(table) => {
                        for cell in &table.cells {
                            for cell_para in &cell.paragraphs {
                                acc.push_str(&cell_para.text);
                                acc.push('\n');
                            }
                        }
                    }
                    Control::Shape(shape) => {
                        // 글상자를 가질 수 있는 도형 종류 — document_core::helpers::
                        // get_textbox_from_shape 와 동일 분기 (테스트는 공개 모델로 접근)
                        let drawing = match shape.as_ref() {
                            ShapeObject::Rectangle(s) => Some(&s.drawing),
                            ShapeObject::Ellipse(s) => Some(&s.drawing),
                            ShapeObject::Polygon(s) => Some(&s.drawing),
                            ShapeObject::Curve(s) => Some(&s.drawing),
                            _ => None,
                        };
                        if let Some(tb) = drawing.and_then(|d| d.text_box.as_ref()) {
                            for tb_para in &tb.paragraphs {
                                acc.push_str(&tb_para.text);
                                acc.push('\n');
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    acc
}

/// `replace_all_native` 반환 JSON을 파싱해 (ok, count)를 돌려준다.
/// 직렬화 포맷 변화에 흔들리지 않도록 substring 매칭 대신 필드로 단언한다.
fn parse_replace_result(result: &str) -> (bool, u64) {
    let v: serde_json::Value = serde_json::from_str(result)
        .unwrap_or_else(|e| panic!("replace_all 반환 JSON 파싱 실패: {e}: {result}"));
    (
        v["ok"].as_bool().unwrap_or(false),
        v["count"].as_u64().unwrap_or(0),
    )
}

fn load(sample: &str) -> (DocumentCore, Vec<u8>) {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let core = DocumentCore::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", path.display(), e));
    (core, bytes)
}

/// 본문 문단 경로: replaceAll 후 export → reparse 하면 치환 결과가 보존되어야 한다.
#[test]
fn replace_all_body_text_survives_hwp_export() {
    let (mut core, _) = load("samples/2022년 국립국어원 업무계획.hwp");

    let needle = "말뭉치";
    let marker = "RT1385BODY";
    let before = collect_all_text(core.document());
    assert!(before.contains(needle), "sample should contain the needle");
    assert!(!before.contains(marker), "marker must not pre-exist");

    let result = core
        .replace_all_native(needle, marker, true)
        .expect("replace_all_native should succeed");
    let (ok, count) = parse_replace_result(&result);
    assert!(ok, "replace_all should report ok: {result}");
    assert!(count > 0, "replace_all should hit at least once: {result}");

    let exported = core
        .export_hwp_with_adapter()
        .expect("export_hwp should succeed after replace_all");
    let reparsed = DocumentCore::from_bytes(&exported).expect("reparse exported bytes");

    let after = collect_all_text(reparsed.document());
    assert!(
        after.contains(marker),
        "replaced marker must survive export → reparse (#1385)"
    );
    assert!(
        !after.contains(needle),
        "original needle must be fully replaced in exported document"
    );
}

/// 표 셀 경로: 셀 내부 치환도 동일 캐시를 거치므로 함께 보존되어야 한다.
#[test]
fn replace_all_table_cell_text_survives_hwp_export() {
    let (mut core, _) = load("samples/복학원서.hwp");

    let needle = "복학원을 제출합니다";
    let marker = "RT1385CELL";
    let before = collect_all_text(core.document());
    assert!(before.contains(needle), "sample should contain cell needle");
    assert!(!before.contains(marker), "marker must not pre-exist");

    let result = core
        .replace_all_native(needle, marker, true)
        .expect("replace_all_native should succeed");
    let (ok, count) = parse_replace_result(&result);
    assert!(ok && count > 0, "cell replace_all should hit: {result}");

    let exported = core
        .export_hwp_with_adapter()
        .expect("export_hwp should succeed after cell replace_all");
    let reparsed = DocumentCore::from_bytes(&exported).expect("reparse exported bytes");

    let after = collect_all_text(reparsed.document());
    assert!(
        after.contains(marker),
        "cell replacement must survive export → reparse (#1385)"
    );
}
