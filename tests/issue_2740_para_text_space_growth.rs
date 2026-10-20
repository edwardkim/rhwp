//! [#2740] 문단 텍스트에 공백이 저장할 때마다 1개씩 무한 누적되던 문제.
//!
//! `serialize_para_text` 는 파서가 자동번호 컨트롤(`0x0012`) 자리에 넣어 둔 공백
//! placeholder 를 다시 컨트롤로 되돌려야 한다. 그런데 판정식이 **다음 문자의 오프셋**을
//! 요구해서, placeholder 가 문단의 **마지막 문자**면 판정이 실패했다. 그러면 공백을
//! 리터럴로 쓰고 컨트롤도 뒤에 다시 방출하므로, 재파싱 때 placeholder 가 하나 더 생긴다.
//! 저장 N회 → 공백 N개로 **수렴하지 않는다**.
//!
//! 자동번호가 문단 끝에 오는 문서(쪽번호가 든 머리말/꼬리말이 대표적)에서 발생한다.
//!
//! 검증 경로는 **레코드 재생성**(편집 후 저장)이다. `raw_stream` 이 살아 있으면
//! 직렬화기가 원본 바이트를 그대로 되돌려주므로 이 결함이 드러나지 않는다.

use rhwp::model::document::Document;
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;

/// 편집이 일어난 문서와 동일한 무효화 후 저장 → 재파싱
/// (`document_core/commands/*` 의 `raw_stream = None` 관례).
fn save_as_edited(doc: &Document) -> Document {
    let mut edited = doc.clone();
    edited.doc_info.raw_stream_dirty = true;
    for s in &mut edited.sections {
        s.raw_stream = None;
    }
    let out = serialize_document(&edited).expect("직렬화 실패");
    parse_document(&out).expect("재파싱 실패")
}

/// 꼬리말(머리말) 안의 문단도 같은 경로를 탄다 — 쪽번호 자동번호가 대표 사례.
#[test]
fn footer_para_text_is_stable_across_repeated_saves() {
    let bytes = std::fs::read("samples/task-001.hwp").expect("샘플 읽기 실패");
    let doc0 = parse_document(&bytes).expect("파싱 실패");

    let footer_text = |doc: &Document| -> Option<(String, u32)> {
        for para in &doc.sections[0].paragraphs {
            for ctrl in &para.controls {
                let inner = match ctrl {
                    rhwp::model::control::Control::Footer(f) => &f.paragraphs,
                    rhwp::model::control::Control::Header(h) => &h.paragraphs,
                    _ => continue,
                };
                if let Some(p) = inner.first() {
                    return Some((p.text.clone(), p.char_count));
                }
            }
        }
        None
    };

    let base = footer_text(&doc0).expect("머리말/꼬리말 문단을 찾지 못함");
    let after1 = save_as_edited(&doc0);
    let after2 = save_as_edited(&after1);

    assert_eq!(
        footer_text(&after1),
        Some(base.clone()),
        "1회 저장에서 꼬리말 문단이 변했다"
    );
    assert_eq!(
        footer_text(&after2),
        Some(base),
        "저장을 반복할수록 꼬리말 문단 텍스트가 자랐다"
    );
}
