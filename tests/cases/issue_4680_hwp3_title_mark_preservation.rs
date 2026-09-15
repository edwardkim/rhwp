//! [Issue #4680] HWP3 제목 차례 표시(코드 25)를 버려서 문단이 본문 레코드째 사라진다.
//!
//! 직렬화기에는 `title_marks` 채널과 코드 `0x0008` 8유닛 인라인 컨트롤 방출이 이미
//! 있는데 HWP3 파서가 채우지 않았다. 그래서 이 표식 **하나만** 있던 문단은
//! `PARA_TEXT` 레코드째 사라지고, `PARA_HEADER` 가 글자 수를 선언해 놓고 본문이 없는
//! 자기모순 레코드가 됐다.
//!
//! # 기대값의 출처
//!
//! `samples/` 에 같은 문서의 HWP3 판과 **한/글이 저장한 HWP5 판**이 짝으로 있다
//! (`hwp3-sample10.hwp` ↔ `hwp3-sample10-hwp5.hwp`). HWP5 쪽 표식 수는 한/글이 실제로
//! 파일에 실은 `0x0008` 인라인 컨트롤 수이므로, 우리 HWP3 파스가 그 수를 재현해야 한다.
//! 수정 전에는 HWP3 쪽이 0 이었다.
//!
//! 264쪽 HWP3 문서(저장소 밖)의 저장 축 실측도 같은 방향이다 — 한/글은 `control_mask`
//! 비트 8 을 125문단에 세우고 `PARA_TEXT` 안 `0x0008` 148건이 전부 `Mtit` 인데, 수정
//! 전 우리는 하나도 세우지 않았다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::document::Document;
use std::path::Path;

const HWP3: &str = "samples/hwp3-sample10.hwp";
const HWP5: &str = "samples/hwp3-sample10-hwp5.hwp";

fn parse(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(path).expect("표본 읽기")).expect("표본 파싱")
}

fn title_mark_count(doc: &Document) -> usize {
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| p.title_marks.len())
        .sum()
}

#[test]
fn hwp3_title_mark_count_matches_the_hangul_conversion() {
    let a = title_mark_count(parse(HWP3).document());
    let b = title_mark_count(parse(HWP5).document());
    assert!(
        b > 0,
        "정답지(한/글 HWP5 판)에 차례 표식이 없다 — 표본이 바뀌었는지 확인하라"
    );
    assert_eq!(
        a, b,
        "HWP3 판의 차례 표식이 한/글 판과 다르다 — 종전처럼 코드 25 를 버리면 0 이 된다"
    );
}

#[test]
fn hwp3_title_mark_paragraph_keeps_its_body_record() {
    // 표식이 있는 문단은 본문 레코드를 갖는다고 주장해야 한다. 종전에는 표식만 있던
    // 문단이 `PARA_TEXT` 없이 헤더만 글자 수를 주장하는 자기모순 레코드가 됐다.
    let core = parse(HWP3);
    let doc = core.document();
    let mut checked = 0usize;
    for section in &doc.sections {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            if para.title_marks.is_empty() {
                continue;
            }
            checked += 1;
            assert!(
                para.has_para_text,
                "문단 {pi}: 차례 표식이 있는데 본문 레코드를 안 내겠다고 한다"
            );
            // 표식은 저장본에서 8유닛을 차지한다 — 글자 수가 그만큼은 되어야 한다.
            let marks = para.title_marks.len() as u32;
            assert!(
                para.char_count >= 8 * marks,
                "문단 {pi}: 표식 {marks}개인데 char_count 가 {} 뿐이다 (8유닛씩 필요)",
                para.char_count
            );
        }
    }
    assert!(checked > 0, "차례 표식이 든 문단을 하나도 못 찾았다");
}

#[test]
fn hwp3_title_mark_offsets_survive_a_save_round_trip() {
    // 반례 — 8유닛을 오프셋 축에 반영하지 않으면 뒤따르는 글자 위치가 밀린다
    // (#3495 미주 표시 0→5 · #3532 글자모양 끝 경계 53→43 이 그 증상이었다).
    let mut core = parse(HWP3);
    let before: Vec<(usize, Option<u32>)> = core
        .document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| (p.title_marks.len(), p.char_offsets.last().copied()))
        .collect();

    let hwp5 = core.export_hwp_with_adapter().expect("HWP5 직렬화");
    let back = DocumentCore::from_bytes(&hwp5).expect("HWP5 재파싱");
    let after: Vec<(usize, Option<u32>)> = back
        .document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .map(|p| (p.title_marks.len(), p.char_offsets.last().copied()))
        .collect();

    assert_eq!(before.len(), after.len(), "저장 왕복에서 문단 수가 변했다");
    // 표식이 **있는** 문단만 본다. 표식 없는 문단의 오프셋이 왕복에서 움직이는 것은
    // 이 수정과 무관한 기존 성질이다(실측: 밀린 6문단 전부 표식 0개).
    let moved = before
        .iter()
        .zip(after.iter())
        .position(|(a, b)| a.0 > 0 && a.1 != b.1);
    assert!(
        moved.is_none(),
        "문단 {} 의 마지막 글자 오프셋이 저장 왕복에서 밀렸다",
        moved.unwrap()
    );
}
