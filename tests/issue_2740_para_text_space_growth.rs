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

/// 문단 텍스트를 경로와 함께 수집 (본문 + 컨트롤 하위 문단 1단계).
fn first_paragraph_text(doc: &Document) -> String {
    doc.sections[0].paragraphs[0].text.clone()
}

/// `samples/eq-002.hwp` 문단 0 은 자동번호 컨트롤이 문단 끝에 온다.
/// 저장을 반복해도 텍스트가 더 자라면 안 된다(멱등).
///
/// 주의: **1회차 저장**에서 공백 1개가 생기는 것은 이 결함과 원인이 다르다.
/// 원본은 자동번호를 쪽 컨트롤 계열 문자(`0x0015`)로 담고 있어 파서가 placeholder 를
/// 만들지 않는데, 직렬화기는 `Control::AutoNumber` 를 항상 `0x0012` 로 쓴다
/// (`control_char_code_and_id`). 그래서 1회차에만 placeholder 가 새로 생긴다.
/// 컨트롤 문자 계열 보존은 별도 사안이라 이 테스트의 단언 대상이 아니다(이슈 #2740 §7).
/// 여기서 고정하는 것은 **누적되지 않는다**는 성질이다.
#[test]
fn para_text_is_idempotent_after_first_save() {
    let bytes = std::fs::read("samples/eq-002.hwp").expect("샘플 읽기 실패");
    let doc0 = parse_document(&bytes).expect("파싱 실패");

    let mut doc = save_as_edited(&doc0);
    let settled = first_paragraph_text(&doc);

    let mut observed = vec![settled.clone()];
    for _ in 0..3 {
        doc = save_as_edited(&doc);
        observed.push(first_paragraph_text(&doc));
    }

    assert!(
        observed.iter().all(|t| *t == settled),
        "저장을 반복할수록 문단 텍스트가 자랐다 (공백 누적) — 라운드마다: {observed:?}"
    );
}

/// 누적성 자체를 고정한다 — 1회 저장과 3회 저장의 결과가 같아야 한다.
/// (1회만 보면 '한 번만 늘고 수렴'하는 정규화와 구분되지 않는다.)
#[test]
fn para_text_growth_is_not_cumulative() {
    let bytes = std::fs::read("samples/eq-002.hwp").expect("샘플 읽기 실패");
    let doc0 = parse_document(&bytes).expect("파싱 실패");

    let after1 = save_as_edited(&doc0);
    let after2 = save_as_edited(&after1);
    let after3 = save_as_edited(&after2);

    assert_eq!(
        first_paragraph_text(&after1),
        first_paragraph_text(&after3),
        "저장 횟수에 따라 텍스트가 계속 자란다 (무한 누적)"
    );
    assert_eq!(
        after1.sections[0].paragraphs[0].char_count, after3.sections[0].paragraphs[0].char_count,
        "char_count 가 저장 횟수에 비례해 증가한다"
    );
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
